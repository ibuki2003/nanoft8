use crate::{
    minifloat::{FloatS, FloatU},
    protocol,
};

#[cfg(feature = "no_std")]
use micromath::F32Ext;

const SPECTRUM_SIZE: usize = 1024;

const TIME_SCALE: usize = 4; // 4segments per symbol (i.e. 160ms / 4 = 40ms)
const FREQ_SCALE: usize = 2; // 2segments per freq bin (i.e. 6.25Hz / 2 = 3.125Hz)

const DECODE_THRESHOLD: f32 = 1.5; // theorethical limit

const FREQ_WIDTH: usize = (protocol::FSK_ARITY - 1) * FREQ_SCALE + 1;

const BUFFER_SYMBOLS: usize = protocol::PAYLOAD_LEN / 2 + protocol::COSTAS_SIZE * 2;
const BUFFER_SIZE: usize = TIME_SCALE * (BUFFER_SYMBOLS - 1) + 1;
const CANDIDATES_BUCKET_SIZE: usize = 8;
const CANDIDATES_COUNT: usize = SPECTRUM_SIZE.div_ceil(CANDIDATES_BUCKET_SIZE);

#[derive(Copy, Clone)]
pub struct Candidate<LLRFloat: FloatS> {
    pub dt: usize,
    pub freq: usize,
    pub power: f32,
    pub band_power: f32,
    pub reliability: f32,

    // supply default impl
    pub data: [LLRFloat; protocol::PAYLOAD_BITS],
}

impl<LLRFloat: FloatS> Candidate<LLRFloat> {
    fn new(dt: usize, freq: usize, reliability: f32) -> Self {
        Self {
            dt,
            freq,
            reliability,
            ..Self::default()
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.power == 0.0
    }

    #[inline]
    pub fn snr_db(&self) -> f32 {
        if self.band_power == 0.0 {
            return f32::NEG_INFINITY;
        }
        (self.power / self.band_power).log10() * 10.0 - 31.5 // magic number :)
    }

    fn update_power<SpecFloat: FloatU>(&mut self, spec: &[SpecFloat]) {
        debug_assert!(spec.len() == FREQ_WIDTH);

        self.power += fsquare(
            (*spec
                .iter()
                .step_by(FREQ_SCALE)
                .max_by_key(|x| x.to_inner())
                .unwrap())
            .into(),
        );
        self.band_power += fsquare((*spec.iter().min_by_key(|x| x.to_inner()).unwrap()).into());
    }
}

impl<LLRFloat: FloatS> Default for Candidate<LLRFloat> {
    fn default() -> Self {
        Self {
            dt: 0,
            freq: 0,
            power: 0.0,
            band_power: 0.0,
            reliability: 0.0,
            data: [LLRFloat::default(); protocol::PAYLOAD_BITS],
        }
    }
}

pub struct Decoder<SpecFloat: FloatU, LLRFloat: FloatS> {
    pub time_step: usize,

    pub spectrum_buffer: [[SpecFloat; SPECTRUM_SIZE]; BUFFER_SIZE],

    pub candidates: [Candidate<LLRFloat>; CANDIDATES_COUNT],
}

impl<SpecFloat: FloatU, LLRFloat: FloatS> Default for Decoder<SpecFloat, LLRFloat> {
    fn default() -> Self {
        Self {
            time_step: 0,
            spectrum_buffer: [[SpecFloat::default(); SPECTRUM_SIZE]; BUFFER_SIZE],
            candidates: [Candidate::default(); CANDIDATES_COUNT],
        }
    }
}

impl<SpecFloat: FloatU, LLRFloat: FloatS> Decoder<SpecFloat, LLRFloat> {
    pub type Spectrum = [SpecFloat; SPECTRUM_SIZE];

    // update decoder with new spectrum data
    // expects spectrum with 3.125Hz per bin, 160ms long, 40ms step
    pub fn put_spectrum(&mut self, data: &Self::Spectrum) {
        let buf_idx = self.time_step % BUFFER_SIZE;
        // find markers
        self.spectrum_buffer[buf_idx] = *data;
        // update candidates

        if self.time_step < BUFFER_SIZE - 1 {
            // data not enough; do nothing
        } else if self.time_step < BUFFER_SIZE * 3 / 2 {
            // find markers
            for i in 0..SPECTRUM_SIZE - FREQ_WIDTH {
                let mut power: f32 = 0.0;
                let mut band_power: f32 = 0.0;
                for j in [1, BUFFER_SIZE - 24] {
                    for (k, &marker) in protocol::MARKER_COSTAS.iter().enumerate() {
                        let idx = (self.time_step + j + k * TIME_SCALE) % BUFFER_SIZE;
                        power += self.spectrum_buffer[idx][i + marker * FREQ_SCALE].into();
                        for k in 0..protocol::COSTAS_SIZE {
                            band_power += self.spectrum_buffer[idx][i + k * FREQ_SCALE].into();
                        }
                    }
                }
                band_power = (band_power - power) / (protocol::COSTAS_SIZE - 1) as f32;
                let reliability = power / band_power;
                if reliability > DECODE_THRESHOLD {
                    let candidate = &mut self.candidates[i / CANDIDATES_BUCKET_SIZE];

                    if candidate.reliability < reliability {
                        *candidate =
                            Candidate::new(self.time_step + 1 - BUFFER_SIZE, i, reliability);
                        // decode data
                        for j in 0..protocol::PAYLOAD_HALF_LEN {
                            let targ = &self.spectrum_buffer[(self.time_step
                                + 1
                                + (protocol::COSTAS_SIZE + j) * TIME_SCALE)
                                % BUFFER_SIZE][i..i + FREQ_WIDTH];
                            Self::get_likelihood(
                                targ,
                                &mut candidate.data
                                    [j * protocol::FSK_DEPTH..(j + 1) * protocol::FSK_DEPTH],
                            );
                            candidate.update_power(targ);
                        }
                    }
                }
            }
        }

        for c in self.candidates.iter_mut() {
            if c.is_empty() {
                // ignore empty candidates
                continue;
            }
            if (self.time_step - c.dt) % TIME_SCALE == 0 {
                let idx = (self.time_step - c.dt) / TIME_SCALE - protocol::COSTAS_SIZE * 2;
                if idx >= protocol::PAYLOAD_LEN {
                    // ignore final marker and beyond
                    continue;
                }
                let targ = &data[c.freq..c.freq + FREQ_WIDTH];
                Self::get_likelihood(
                    targ,
                    &mut c.data[idx * protocol::FSK_DEPTH..(idx + 1) * protocol::FSK_DEPTH],
                );
                c.update_power(targ);
            }
        }

        self.time_step += 1;
    }

    fn get_likelihood(data: &[SpecFloat], out: &mut [LLRFloat]) {
        assert_eq!(data.len(), FREQ_WIDTH);
        assert_eq!(out.len(), protocol::FSK_DEPTH);

        let mut sm = [[0.0f32; 2]; protocol::FSK_DEPTH];
        for i in 0..protocol::FSK_ARITY {
            for (j, row) in sm.iter_mut().enumerate() {
                let bit = (protocol::GRAY_CODE[i] & (4 >> j) != 0) as usize;
                row[bit] += fsquare(data[i * FREQ_SCALE].into());
            }
        }

        for i in 0..protocol::FSK_DEPTH {
            let v = sm[i][1].ln() - sm[i][0].ln();
            out[i] = v.into();
        }
    }
}

fn fsquare(x: f32) -> f32 {
    x * x
}
