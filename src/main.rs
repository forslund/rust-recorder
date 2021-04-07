extern crate portaudio;
use portaudio as pa;

use std::{thread, time};

use std::sync::mpsc::channel;

fn main() {
    match pa::PortAudio::new() {
        Ok(pa) => {
            match print_inputs(&pa) {
                Ok(_) => {}
                e => {
                    eprintln!("Example failed with the following: {:?}", e);
                }
            }
            match record(pa) {
                Ok(_) => {}
                e => {
                    eprintln!("Example failed with the following: {:?}", e);
                }
            }
            
        }
        e => {
            eprintln!("Example failed with the following: {:?}", e);
        }
    }
}

fn print_inputs(pa: &pa::PortAudio) -> Result<(), pa::Error> {

    let num_devices = pa.device_count()?;
    println!("Number of devices = {}", num_devices);

    println!("Default input device: {:?}", pa.default_input_device());
    println!("Input devices:");
    for device in pa.devices()? {
        let (_, info) = device?;
        let in_channels = info.max_input_channels;
        if in_channels > 0 {
            println!("- Device {}", info.name);
        }
    }
    Ok(())
}


const CHANNELS: i32 = 1;
//const NUM_SECONDS: u64 = 5;
const SAMPLE_RATE: f64 = 16000.0;
const FRAMES_PER_BUFFER: u32 = 64;


fn record(pa: pa::PortAudio) -> Result<(), pa::Error> {
    let (sender, receiver) = channel();
    let settings = pa.default_input_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
    let callback = move |pa::InputStreamCallbackArgs { buffer, .. }| {
        let buffer : &[i16] = buffer;
        match sender.send(buffer) {
            Ok(_) => portaudio::Continue,
            Err(_) => portaudio::Complete
        }
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    stream.start().expect("Unable to start stream"); 
    let start = time::Instant::now();
    while start.elapsed() < time::Duration::new(5, 0) {
        match  receiver.try_recv() {
            Ok(buff) => mean(buff),
            Err(_) => ()
        }
        thread::sleep(time::Duration::new(0, 100000));
    }
    Ok(())
}

fn mean(audio_buffer: &[i16]) {
    let mut mean = 0i32;
    for sample in audio_buffer.iter() {
        mean += i32::from(*sample);
    }
    println!("AUDIO {}", mean as f32 / 64f32);
}
