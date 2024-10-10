use crate::{protocol, F8};

const SPECTRUM_SIZE: usize = 1024;

// TODO: use 8bit integers to save space
pub type Spectrum = [F8; SPECTRUM_SIZE];

#[derive(Copy, Clone, Debug)]
pub struct Candidate {
    pub dt: usize,
    pub freq: usize,
    pub strength: f32,
    pub reliability: f32,

    // supply default impl
    pub data: [F8; protocol::PAYLOAD_BITS],
}

impl Default for Candidate {
    fn default() -> Self {
        Self {
            dt: 0,
            freq: 0,
            strength: 0.0,
            reliability: 0.0,
            data: [F8::ZERO; protocol::PAYLOAD_BITS],
        }
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.reliability.partial_cmp(&other.reliability).unwrap()
    }
}
impl Eq for Candidate {}
impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.strength == other.strength
    }
}

pub struct Decoder {
    pub time_step: usize,

    pub spectrum_buffer: [Spectrum; Self::BUFFER_SIZE],

    pub candidates: [Candidate; Self::CANDIDATES_COUNT],
}

impl Default for Decoder {
    fn default() -> Self {
        Self {
            time_step: 0,
            spectrum_buffer: [[F8::ZERO; SPECTRUM_SIZE]; Self::BUFFER_SIZE],
            candidates: [Candidate::default(); Self::CANDIDATES_COUNT],
        }
    }
}

impl Decoder {
    const TIME_SCALE: usize = 4; // 4segments per symbol (i.e. 160ms / 4 = 40ms)
    const FREQ_SCALE: usize = 2; // 2segments per freq bin (i.e. 6.25Hz / 2 = 3.125Hz)

    const DECODE_THRESHOLD: f32 = 1.5; // theorethical limit

    const FREQ_WIDTH: usize = (protocol::FSK_ARITY - 1) * Self::FREQ_SCALE + 1;

    const BUFFER_SYMBOLS: usize = protocol::PAYLOAD_LEN / 2 + protocol::COSTAS_SIZE * 2;
    const BUFFER_SIZE: usize = Self::TIME_SCALE * (Self::BUFFER_SYMBOLS - 1) + 1;
    const CANDIDATES_BUCKET_SIZE: usize = 8;
    const CANDIDATES_COUNT: usize =
        (SPECTRUM_SIZE + Self::CANDIDATES_BUCKET_SIZE - 1) / Self::CANDIDATES_BUCKET_SIZE;

    // update decoder with new spectrum data
    // expects spectrum with 3.125Hz per bin, 160ms long, 40ms step
    pub fn put_spectrum(&mut self, data: &Spectrum) {
        let buf_idx = self.time_step % Self::BUFFER_SIZE;
        // find markers
        self.spectrum_buffer[buf_idx] = *data;
        // update candidates

        if self.time_step < Self::BUFFER_SIZE - 1 {
            // data not enough; do nothing
        } else if self.time_step < Self::BUFFER_SIZE * 2 {
            // find markers
            for i in 0..SPECTRUM_SIZE - Self::FREQ_WIDTH {
                let mut power: f32 = 0.0;
                let mut band_power: f32 = 0.0;
                for j in [1, Self::BUFFER_SIZE - 24] {
                    for (k, &marker) in protocol::MARKER_COSTAS.iter().enumerate() {
                        let idx = (self.time_step + j + k * Self::TIME_SCALE) % Self::BUFFER_SIZE;
                        power += self.spectrum_buffer[idx][i + marker * Self::FREQ_SCALE].as_f32();
                        for k in 0..protocol::COSTAS_SIZE {
                            band_power +=
                                self.spectrum_buffer[idx][i + k * Self::FREQ_SCALE].as_f32();
                        }
                    }
                }
                band_power = (band_power - power) / (protocol::COSTAS_SIZE - 1) as f32;
                let reliability = power / band_power;
                if reliability > Self::DECODE_THRESHOLD {
                    let candidate = Candidate {
                        dt: self.time_step + 1 - Self::BUFFER_SIZE,
                        freq: i,
                        strength: power,
                        reliability,
                        data: [F8::ZERO; protocol::PAYLOAD_BITS],
                    };
                    let old_candidate = &mut self.candidates[i / Self::CANDIDATES_BUCKET_SIZE];

                    if *old_candidate < candidate {
                        *old_candidate = candidate;
                        // decode data
                        for j in 0..protocol::PAYLOAD_HALF_LEN {
                            Self::get_likelihood(
                                &self.spectrum_buffer[(self.time_step
                                    + 1
                                    + (protocol::COSTAS_SIZE + j) * Self::TIME_SCALE)
                                    % Self::BUFFER_SIZE][i..i + Self::FREQ_WIDTH],
                                &mut old_candidate.data
                                    [j * protocol::FSK_DEPTH..(j + 1) * protocol::FSK_DEPTH],
                            );
                        }
                    }
                }
            }
        }

        for c in self.candidates.iter_mut() {
            if c.strength == 0.0 {
                // ignore empty candidates
                continue;
            }
            if (self.time_step - c.dt) % Self::TIME_SCALE == 0 {
                let idx = (self.time_step - c.dt) / Self::TIME_SCALE - protocol::COSTAS_SIZE * 2;
                if idx >= protocol::PAYLOAD_LEN {
                    // ignore final marker and beyond
                    continue;
                }
                Self::get_likelihood(
                    &data[c.freq..c.freq + Self::FREQ_WIDTH],
                    &mut c.data[idx * protocol::FSK_DEPTH..(idx + 1) * protocol::FSK_DEPTH],
                );
            }
        }

        self.time_step += 1;
    }

    fn get_likelihood(data: &[F8], out: &mut [F8]) {
        assert_eq!(data.len(), Self::FREQ_WIDTH);
        assert_eq!(out.len(), protocol::FSK_DEPTH);

        let mut outf = [[0.0f32; 2]; protocol::FSK_DEPTH];
        for i in 0..protocol::FSK_ARITY {
            for j in 0..protocol::FSK_DEPTH {
                let bit = (protocol::GRAY_CODE[i] & (4 >> j) != 0) as usize;
                outf[j][bit] += data[i * Self::FREQ_SCALE].as_f32().powf(2.);
            }
        }

        for i in 0..protocol::FSK_DEPTH {
            let v = outf[i][1].ln() - outf[i][0].ln();
            out[i] = F8::from_f32(v);
        }
    }
}
