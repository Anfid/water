mod midi_box;
mod music_box;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use midi_box::{MidiBox, MidiEvent, MidiMsg};
use music_box::MusicBox;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "full"))]
fn main() {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();

    let connection = MidiBox::connect().expect("unable to connect to midi port");

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), connection.rx).unwrap(),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), connection.rx).unwrap(),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), connection.rx).unwrap(),
    }
}

fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    rx: std::sync::mpsc::Receiver<MidiMsg>,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let mut music_box = MusicBox::new();
    let mut next_value = move || {
        for msg in rx.try_iter() {
            match msg.event {
                MidiEvent::KeyPress { key, velocity: _ } => music_box.press(key),
                MidiEvent::KeyRelease { key } => music_box.release(key),
                MidiEvent::Volume { level } => music_box.set_volume(level),
            }
        }
        music_box.increase_clock(1.0 / sample_rate);
        music_box.get_sample()
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let value: T = cpal::Sample::from::<f32>(&next_value());
                for sample in frame.iter_mut() {
                    *sample = value;
                }
            }
        },
        err_fn,
    )?;
    stream.play()?;

    // just let it be
    loop {
        std::thread::park();
    }
}
