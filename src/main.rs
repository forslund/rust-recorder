//! A demonstration of recording to wav an input stream
//!
//! Audio from the default input device is stored in to a wav file

extern crate portaudio;

use std::time::{Duration, Instant};
use std::thread::sleep;
use std::sync::mpsc::channel;

extern crate hound;
use hound::WavWriter;

use std::fs;
use std::io;
use std::mem;

fn main() {
    const CHANNELS: i32 = 2;
    const SAMPLE_RATE: f64 = 16_000.0;
    const FRAMES: u32 = 256;

    let audio_port = match open_audio_port() {
        Ok(port) => port,
        Err(error) => panic!("{}", String::from(error))
    };

    let input_index = match get_input_device_index(&audio_port) {
        Ok(index) => index,
        Err(error) => panic!("{}", String::from(error))
    };

    let input_settings = match get_input_settings(input_index, &audio_port, SAMPLE_RATE, FRAMES, CHANNELS) {
        Ok(settings) => settings,
        Err(error) => panic!("{}", String::from(error))
    };

    let mut wav_writer = match get_wav_writer("recorded.wav", CHANNELS,
                                              SAMPLE_RATE) {
        Ok(writer) => writer,
        Err(error) => panic!("{}", error.to_string()),
    };

    let (sender, receiver) = channel();

    let callback = move |portaudio::InputStreamCallbackArgs { buffer, .. }| {
        sender.send(buffer).unwrap();
        portaudio::Continue
    };

    // Construct a stream with input and output sample types of i32
    let mut stream = match audio_port.open_non_blocking_stream(input_settings,
                                                               callback) {
        Ok(strm) => strm,
        Err(error) => panic!("{}", error.to_string()),
    };

    match stream.start() {
        Ok(_) => {}
        Err(error) => panic!("{}", error.to_string()),
    };

    let start = Instant::now();

    let time_to_wait = &(5 as u64);
    while start.elapsed().as_secs().lt(time_to_wait) {
        let recv_data = receiver.recv().unwrap();
        for sample in recv_data.iter() {
            wav_writer.write_sample(*sample).ok();
        }
    }

    match close_stream(stream){
        Ok(_) => {}
        Err(error) => panic!("{}", error.to_string()),
    };
}

fn get_wav_writer(path: &'static str, channels: i32, sample_rate: f64) -> Result<WavWriter<io::BufWriter<fs::File>>,String> {
    let spec = wav_spec(channels, sample_rate);
    match hound::WavWriter::create(path, spec) {
        Ok(writer) => Ok(writer),
        Err(error) => Err (String::from(format!("{}",error))),
    }
}

fn wav_spec(channels: i32, sample_rate: f64) -> hound::WavSpec {
    hound::WavSpec {
        channels: channels as _,
        sample_rate: sample_rate as _,
        bits_per_sample: (mem::size_of::<i16>() * 8) as _,
        sample_format: hound::SampleFormat::Int,
    }
}

fn close_stream(mut stream: portaudio::Stream<portaudio::NonBlocking, portaudio::Input<i16>>) -> Result<String, String> {
    match stream.stop() {
        Ok(_) => {
            Ok(String::from("Stream closed"))
        },
        Err(error) => {
            Err(error.to_string())
        },
    }
}

fn open_audio_port() -> Result<portaudio::PortAudio, String>
{
    portaudio::PortAudio::new().or_else(|error| Err(String::from(format!("{}", error))))
}

fn get_input_device_index(audio_port: &portaudio::PortAudio) -> Result<portaudio::DeviceIndex, String>
{
    audio_port.default_input_device().or_else(|error| Err(String::from(format!("{}", error))))
}

fn get_input_latency(audio_port: &portaudio::PortAudio, input_index: portaudio::DeviceIndex) -> Result<f64, String>
{
    let input_device_information = audio_port.device_info(input_index).or_else(|error| Err(String::from(format!("{}", error))));
    Ok(input_device_information.unwrap().default_low_input_latency)
}

fn get_input_stream_parameters(input_index: portaudio::DeviceIndex, latency: f64, channels: i32) -> Result<portaudio::StreamParameters<i16>, String>
{
    const INTERLEAVED: bool = true;
    Ok(portaudio::StreamParameters::<i16>::new(input_index, channels, INTERLEAVED, latency))
}

fn get_input_settings(input_index: portaudio::DeviceIndex, audio_port: &portaudio::PortAudio, sample_rate: f64, frames: u32, channels: i32) -> Result<portaudio::InputStreamSettings<i16>, String>
{
    Ok(
        portaudio::InputStreamSettings::new(
            (get_input_stream_parameters(
                input_index,
                (get_input_latency(
                    &audio_port,
                    input_index,
                ))?,
                channels
            ))?,
            sample_rate,
            frames,
        )
    )
}
