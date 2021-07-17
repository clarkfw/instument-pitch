use std::borrow::BorrowMut;
use std::ptr::read;

use pitch_detection::detector::PitchDetector;
use soundio::{Context, Error};

fn main() -> Result<(), String> {
    let mut ctx = soundio::Context::new();
    ctx.set_app_name("Player");
    ctx.connect()?;
    ctx.flush_events();
    let dev = ctx.default_input_device().expect("No input device");

    if !dev.supports_layout(soundio::ChannelLayout::get_builtin(soundio::ChannelLayoutId::Mono)) {
        return Err("Device doesn't support mono".to_string());
    }
    if !dev.supports_format(soundio::Format::S16LE) {
        return Err("Device doesn't support S16LE".to_string());
    }
    if !dev.supports_sample_rate(44100) {
        return Err("Device doesn't 44.1 kHz".to_string());
    }

    let mut rec: Vec<i16> = Vec::new();
    let mut count = 0i32;

    let read_callback = move |stream: &mut soundio::InStreamReader| {
        let frame_count_max = stream.frame_count_max();
        if let Err(e) = stream.begin_read(frame_count_max) {
            println!("Error reading from stream: {}", e);
            return;
        }

        for f in 0..stream.frame_count() {
            for c in 0..stream.channel_count() {
                let sp = stream.sample::<i16>(c, f);
                rec.push(sp);
                if rec.len() > 1024 {
                    rec.remove(0);
                }
                count += 1;
                if count % 10000 == 0 {
                    pitch(&mut rec);
                };
            }
        }
    };

    let mut input_stream = dev.open_instream(
        44100,
        soundio::Format::S16LE,
        soundio::ChannelLayout::get_builtin(soundio::ChannelLayoutId::Mono),
        2.0,
        read_callback,
        None::<fn()>,
        None::<fn(soundio::Error)>,
    )?;

    input_stream.start()?;
    ctx.wait_events();
    return Ok(());
}


fn pitch(signal: &Vec<i16>) {
    use pitch_detection::detector::mcleod::McLeodDetector;
    use pitch_detection::detector::autocorrelation::AutocorrelationDetector;
    const SAMPLE_RATE: usize = 44100;
    const SIZE: usize = 1024;
    const PADDING: usize = SIZE / 2;
    const POWER_THRESHOLD: f64 = 5.0;
    const CLARITY_THRESHOLD: f64 = 0.7;

    if signal.len() < SIZE { return; };

    let signal = signal.iter().rev().take(SIZE).rev().map(|it| *it as f64).collect::<Vec<f64>>();
    let mut detector = McLeodDetector::new(SIZE, PADDING);

    let pitch = detector.get_pitch(&signal, SAMPLE_RATE, POWER_THRESHOLD, CLARITY_THRESHOLD);

    match pitch {
        Some(p) => {
            println!("Frequency: {}, Clarity: {}", p.frequency, p.clarity)
        }
        None => ()
    }
}
