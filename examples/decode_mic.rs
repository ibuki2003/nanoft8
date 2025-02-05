#![feature(generic_const_exprs)]
use chrono::Timelike as _;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait as _};
use nanoft8::{
    decoder::Decoder,
    protocol::{
        self,
        crc::check_crc,
        message::{callsign::hash::CallsignHashTable, Message},
    },
};
use num_complex::Complex32;
use std::collections::BTreeMap;

#[inline]
fn sec() -> u32 {
    chrono::Utc::now().second()
}

type SpecFloat = f32;
type LLRFloat = f32;
type Dec = Decoder<SpecFloat, LLRFloat>;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    println!("Decoder size: {} Bytes", std::mem::size_of::<Dec>());

    let mut hashtable = BTreeMap::<u32, [u8; 11]>::new();

    if args.len() < 2 {
        // from mic
        let host = cpal::default_host();
        let device = host.default_input_device().unwrap();
        println!("Using input device: '{}'", device.name().unwrap());
        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(8000),
            buffer_size: cpal::BufferSize::Default,
        };

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let (tx, rx) = std::sync::mpsc::sync_channel(8192);

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[i16], _: &_| {
                    data.iter().for_each(|x| {
                        tx.try_send(*x as f32 / 32768.0).ok();
                    });
                },
                err_fn,
                None,
            )
            .unwrap();

        stream.play().unwrap();

        loop {
            let mut last_sec = 0;
            let mut iter = std::iter::from_fn(|| {
                if last_sec != 0 && sec() % 15 == 0 {
                    return None;
                }
                last_sec = sec() % 15;

                rx.recv().ok()
            });
            process(&mut iter, 8000, &mut hashtable);
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
                let mut iter = samples.by_ref().take(15 * rate as usize);
                process(&mut iter, rate, &mut hashtable);
            }
        }
    }
}

fn process(
    source: &mut dyn Iterator<Item = f32>,
    rate: u32,
    hashtable: &mut impl CallsignHashTable,
) {
    let mut decoder = Dec::default();

    let step: usize = (rate * 40 / 1000) as usize;
    let size: usize = (rate * 160 / 1000) as usize;

    let mut buf = vec![0f32; size];

    let mut fftbuf = vec![Complex32::new(0.0, 0.0); size * 2];

    let mut planner = rustfft::FftPlanner::new();
    let fft = planner.plan_fft_forward(size * 2);

    let mut spectrum = [SpecFloat::default(); 1024];

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
            let x = (x / 10.).norm();
            spectrum[i] = x;
        });
        decoder.put_spectrum(&spectrum);
    }
    print_candidates(&decoder, hashtable);
}

fn hanning_window(data: &mut [Complex32]) {
    let n = data.len();
    for (i, x) in data.iter_mut().enumerate() {
        *x *= 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / n as f32).cos();
    }
}

const COLOR_GRAY: &str = "\x1b[38;5;240m";
const COLOR_RESET: &str = "\x1b[0m";

fn print_candidates(dec: &Dec, hashtable: &mut impl CallsignHashTable) {
    let mut c = dec.candidates().iter().collect::<Vec<_>>();
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

        let (bs, err) = protocol::ldpc::solve(&i.data);

        let res = check_crc(&bs);

        if bs.0.iter().all(|&x| x == 0) {
            // empty message
            continue;
        }

        if !res && cnt >= 10 {
            continue;
        }

        let bs = bs.with_size::<77>();
        let msg = Message::decode(&bs);

        let str = msg
            .as_ref()
            .and_then(|msg| {
                let n = msg.write_str(&mut buf, Some(hashtable))?;
                std::str::from_utf8(&buf[..n]).ok()
            })
            .unwrap_or("(invalid)");

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
        if let Some(msg) = msg {
            msg.register_callsigns(hashtable);
        }
    }
    println!();
}
