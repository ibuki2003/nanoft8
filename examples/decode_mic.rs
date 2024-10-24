use chrono::Timelike as _;
use nanoft8::decoder::{Candidate, Decoder, Spectrum};
use nanoft8::protocol::crc::check_crc;
use nanoft8::protocol::message::Message;
use nanoft8::{protocol, Bitset, F8};
use num_complex::Complex32;

#[inline]
fn sec() -> u32 {
    chrono::Utc::now().second()
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        // from mic
        let rec = pulse_simple::Record::new("nanoft8", "hoge", None, 48000);
        let mut buf = [[0f32; 1]; 1024];
        loop {
            let mut last_sec = 0;
            let mut idx = 0;
            let mut iter = std::iter::from_fn(|| {
                if last_sec != 0 && sec() % 15 == 0 {
                    return None;
                }
                last_sec = sec() % 15;

                if idx == 0 {
                    rec.read(&mut buf);
                }
                let ret = Some(buf[idx][0]);
                idx = (idx + 1) % 1024;
                ret
            });
            process(&mut iter, 48000);
        }
    } else {
        // from file
        for f in &args[1..] {
            println!("reading file: {}", f);

            let mut reader = hound::WavReader::open(f).unwrap();
            let spec = reader.spec();
            println!("{:?}", spec);
            let rate = spec.sample_rate;
            let ch = spec.channels;
            let fmt = spec.sample_format;
            let len = reader.len() / ch as u32;
            println!("has {} samples ({:.1} sec)", len, len as f32 / rate as f32);

            let mut samples: Box<dyn Iterator<Item = f32>> = match fmt {
                hound::SampleFormat::Float => Box::new(
                    reader
                        .samples::<f32>()
                        .step_by(ch as usize)
                        .map(|x| x.unwrap()),
                ),
                hound::SampleFormat::Int => Box::new(
                    reader
                        .samples::<i16>()
                        .step_by(ch as usize)
                        .map(|x| x.unwrap() as f32 / 32768.0),
                ),
            };
            // run for each 15sec
            let cnt = (len - (5 * rate) + (rate * 15 - 1)) / rate / 15; // ceil
            for i in 0..cnt {
                println!("processing at {}", i * 15);
                let mut iter = TakeAndSkip::new(samples, 15 * rate as usize);
                process(&mut iter, rate);
                samples = iter.destroy();
            }
        }
    }
}

fn process(source: &mut dyn Iterator<Item = f32>, rate: u32) {
    let mut decoder = Decoder::default();

    let step: usize = (rate * 40 / 1000) as usize;
    let size: usize = (rate * 160 / 1000) as usize;

    let mut buf = vec![0f32; size];

    let mut fftbuf = vec![Complex32::new(0.0, 0.0); size * 2];

    let mut planner = rustfft::FftPlanner::new();
    let fft = planner.plan_fft_forward(size * 2);

    let mut spectrum: Spectrum = [F8::ZERO; 1024];

    'outer: for i in 0.. {
        for j in 0..step {
            match source.next() {
                Some(v) => buf[(i % 4) * step + j] = v,
                None => break 'outer,
            }
        }

        fftbuf
            .iter_mut()
            .for_each(|x| *x = Complex32::new(0.0, 0.0));
        for j in 0..size {
            fftbuf[j].re = buf[((i + 1) * step + j) % size] as f32;
        }
        hanning_window(&mut fftbuf[..size]);
        fft.process(&mut fftbuf);
        fftbuf[..1024].iter().enumerate().for_each(|(i, x)| {
            spectrum[i] = x.norm().into();
        });
        decoder.put_spectrum(&spectrum);
    }
    print_candidates(&decoder.candidates);
}

fn hanning_window(data: &mut [Complex32]) {
    let n = data.len();
    for (i, x) in data.iter_mut().enumerate() {
        *x *= 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / n as f32).cos();
    }
}

const COLOR_GRAY: &str = "\x1b[38;5;240m";
const COLOR_RESET: &str = "\x1b[0m";

fn print_candidates(c: &[Candidate]) {
    let mut c = Vec::from(c);
    // c.sort_by_cached_key(|x| x.strength.to_bits());
    c.sort_by_cached_key(|x| x.reliability.to_bits());
    c.reverse();
    // c.sort_by_cached_key(|x| x.freq);

    // print!("\x1b[2J\x1b[1;1H"); // clear screen

    let mut buf = [0; 256];
    println!(
        "{:>5} {:>8} {:>8} {:>8}  {:>3} {}",
        "dt", "freq", "strength", "reliab", "err", "message"
    );
    let mut cnt = 0;
    for i in c.iter() {
        if i.reliability < 1.0 {
            continue;
        }
        cnt += 1;

        let mut bs = Bitset::default();
        let err = protocol::ldpc::solve(&i.data, &mut bs);

        let res = check_crc(&bs);

        if !res && cnt >= 10 {
            continue;
        }

        let str = Message::decode(&bs)
            .map(|msg| {
                msg.to_string(&mut buf);
                String::from_utf8_lossy(buf.trim_ascii())
            })
            .unwrap_or("(invalid)".into());

        println!(
            "{}{:>5} {:>8.1} {:>8.2} {:>8.2}  {:>3} {}{}",
            if res { "" } else { COLOR_GRAY },
            i.dt * 40,
            i.freq as f32 * 3.125,
            i.snr_db(),
            i.reliability,
            err,
            str,
            COLOR_RESET
        );
    }
    println!();
}

struct TakeAndSkip<I: Iterator> {
    iter: I,
    count: usize,
}

impl<I: Iterator> Iterator for TakeAndSkip<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None;
        }
        self.count -= 1;
        self.iter.next()
    }
}

impl<I: Iterator> TakeAndSkip<I> {
    fn new(iter: I, count: usize) -> Self {
        Self { iter, count }
    }

    fn destroy(self) -> I {
        self.iter
    }
}
