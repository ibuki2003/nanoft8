use chrono::Timelike as _;
use nanoft8::decoder::{Candidate, Decoder, Spectrum};
use nanoft8::protocol::message::Message;
use nanoft8::{protocol, Bitset};
use num_complex::Complex32;

trait SampleReader {
    fn read(&mut self, buf: &mut [i16]) -> bool;
}

impl SampleReader for pulse_simple::Record<[i16; 1]> {
    fn read(&mut self, buf: &mut [i16]) -> bool {
        let buf = unsafe {
            core::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut [i16; 1], buf.len())
        };
        pulse_simple::Record::read(self, buf);
        true
    }
}

impl SampleReader for std::vec::IntoIter<i16> {
    fn read(&mut self, buf: &mut [i16]) -> bool {
        for x in buf.iter_mut() {
            match self.next() {
                Some(v) => *x = v,
                None => return false,
            }
        }
        true
    }
}

#[inline]
fn sec() -> u32 {
    chrono::Utc::now().second()
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let incremental = args.len() < 2;

    let (mut source, rate): (Box<dyn SampleReader>, u32) = if incremental {
        let rec = pulse_simple::Record::new("nanoft8", "hoge", None, 48000);
        (Box::new(rec), 48000)
    } else {
        let filename = args[1].clone();
        let mut reader = hound::WavReader::open(filename).unwrap();
        println!("{:?}", reader.spec());
        let rate = reader.spec().sample_rate;
        let ch = reader.spec().channels;
        let samples = reader
            .samples::<i16>()
            .map(|x| x.unwrap())
            .step_by(ch as usize)
            .collect::<Vec<_>>();
        println!(
            "read {} samples ({} sec)",
            samples.len(),
            samples.len() as f32 / rate as f32
        );
        (Box::new(samples.into_iter()), rate)
    };

    let mut decoder = Decoder::default();

    let step: usize = (rate * 40 / 1000) as usize;
    let size: usize = (rate * 160 / 1000) as usize;

    let mut buf = vec![0; size];

    let mut fftbuf = vec![Complex32::new(0.0, 0.0); size * 2];

    let mut planner = rustfft::FftPlanner::new();
    let fft = planner.plan_fft_forward(size * 2);

    let mut spectrum: Spectrum = [0.0; 1024];

    let mut reset = false;
    for i in 0.. {
        if incremental && !reset && sec() % 15 == 0 {
            println!("reset");
            reset = true;
            print_candidates(&decoder.candidates);
            decoder = Decoder::default();
        }
        if sec() % 15 != 0 {
            reset = false;
        }
        if !source.read(&mut buf[(i % 4) * step..][..step]) {
            break;
        }

        fftbuf
            .iter_mut()
            .for_each(|x| *x = Complex32::new(0.0, 0.0));
        for j in 0..size {
            fftbuf[j].re = buf[(i * step + j) % size] as f32 / 32768.0;
        }
        hanning_window(&mut fftbuf);
        fft.process(&mut fftbuf);
        fftbuf[..1024].iter().enumerate().for_each(|(i, x)| {
            spectrum[i] = x.norm();
        });
        decoder.put_spectrum(&spectrum);

        // if incremental && i % 10 == 0 {
        //     println!("{}", i);
        //     print_candidates(&decoder.candidates);
        // }
    }
    print_candidates(&decoder.candidates);
}

fn hanning_window(data: &mut [Complex32]) {
    let n = data.len();
    for (i, x) in data.iter_mut().enumerate() {
        *x *= 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / n as f32).cos();
    }
}

fn print_candidates(c: &[Candidate]) {
    let mut c = Vec::from(c);
    c.sort_by_cached_key(|x| x.reliability.to_bits());
    c.reverse();

    // print!("\x1b[2J\x1b[1;1H"); // clear screen

    let mut buf = [0; 256];
    for i in c.iter().take(10) {
        let bs = to_bits(&i.data);

        let str = match Message::decode(&bs) {
            Ok(msg) => {
                msg.to_string(&mut buf);
                String::from_utf8_lossy(&buf).trim().to_string()
            }
            Err(_) => "(invalid)".to_string(),
        };
        println!(
            "{:>5} {:>8.1} {:>8.2} {:>8.2} {}",
            i.dt * 40,
            i.freq as f32 * 3.125,
            i.strength,
            i.reliability,
            str
        );
    }
    println!();
}

fn to_bits(data: &[f32]) -> Bitset {
    let mut bs = Bitset::default();
    for (i, &x) in data[..protocol::BODY_BITS].iter().enumerate() {
        bs.set(i, x > 0.0);
    }
    bs
}
