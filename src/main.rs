extern crate portaudio;
use portaudio as pa;
use hound;

use std::{thread, time};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};

const NUM_SECONDS: u64 = 3;

fn main() {
    let port_audio = match pa::PortAudio::new() {
        Ok(pa) => pa,
        Err(e) => {
            eprintln!("Example failed with the following: {:?}", e);
            return ();
        }
    };

    match print_inputs(&port_audio) {
        Ok(_) => {},
        e => eprintln!("Example failed with the following: {:?}", e)
    };

    let (sender, receiver) : (Sender<&[i16]>, Receiver<&[i16]>) = channel();
    let mut stream = match record(port_audio, sender) {
        Ok(s) => s,
        Err(e) => {
            println!("ERROR STARTING RECORDING {:?}", e);
            return ();
        }
    };


    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create("out.wav", spec).unwrap();

    let start = time::Instant::now();
    while start.elapsed() < time::Duration::new(NUM_SECONDS, 0) {
        match  receiver.try_recv() {
            Ok(buff) => {
                //mean(buff);
                for sample in buff.iter() {
                    writer.write_sample(*sample).unwrap();
                }
            },
            Err(_) => {}
        }
        thread::sleep(time::Duration::new(0, 100000));
    }
    writer.finalize().unwrap();
    stream.close().unwrap();
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
const SAMPLE_RATE: f64 = 16000.0;
const FRAMES_PER_BUFFER: u32 = 4096;


fn record(pa: pa::PortAudio, sender : Sender<&'static [i16]>) ->
        Result<pa::Stream<pa::NonBlocking, pa::Input<i16>>, pa::Error> {
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
    return Ok(stream);
}
