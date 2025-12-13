use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use cpal::{
    Sample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use realfft::RealFftPlanner;

use crate::{FFT_GAIN_CORRECTION, FFT_SIZE, HOP_SIZE, MAX_BUFFER_SIZE, TEMPORAL_SMOOTHING, math};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub name: String,
}

impl DeviceInfo {
    const fn new(name: String) -> Self {
        Self { name }
    }
}

impl std::fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone)]
pub struct Data {
    pub spectrum: Arc<Mutex<Vec<f32>>>,
    pub sample_rate: Arc<Mutex<f32>>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            spectrum: Arc::new(Mutex::new(vec![0.0; FFT_SIZE / 2 + 1])),
            sample_rate: Arc::new(Mutex::new(48_000.0)),
        }
    }
}

pub fn enumerate_devices(host: &cpal::Host) -> Vec<DeviceInfo> {
    let mut found = Vec::new();

    if let Ok(devices) = host.devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                found.push(DeviceInfo::new(name));
            }
        }
    }

    found
}

pub fn setup_capture_with_device(
    audio_data: Data,
    device_name: Option<String>,
) -> (Option<cpal::Stream>, Option<DeviceInfo>) {
    let host = cpal::default_host();

    let device = device_name.map_or_else(
        || find_monitor_device(&host).or_else(|| host.default_output_device()),
        |name| find_device_by_name(&host, &name),
    );

    let device = match device {
        Some(d) => d,
        None => {
            eprintln!("No audio device available");
            return (None, None);
        }
    };

    let device_info = device.name().ok().map(DeviceInfo::new);
    println!("Using device: {:?}", device_info);

    let stream = setup_audio_capture_for_device(audio_data, &device);

    (stream, device_info)
}

fn find_device_by_name(host: &cpal::Host, target_name: &str) -> Option<cpal::Device> {
    if let Ok(devices) = host.devices() {
        for device in devices {
            if let Ok(name) = device.name()
                && name == target_name
            {
                return Some(device);
            }
        }
    }
    None
}

fn find_monitor_device(host: &cpal::Host) -> Option<cpal::Device> {
    if let Ok(device_name) = std::env::var("AUDIO_DEVICE") {
        println!("Looking for AUDIO_DEVICE override: {}", device_name);
        if let Ok(devices) = host.devices() {
            for device in devices {
                if let Ok(name) = device.name()
                    && name.contains(&device_name)
                {
                    println!("Found override device: {}", name);
                    return Some(device);
                }
            }
        }
        eprintln!("Could not find device matching: {}", device_name);
    }

    println!("\n=== Available INPUT devices ===");
    let devices = match host.devices() {
        Ok(devices) => devices,
        Err(e) => {
            eprintln!("Failed to enumerate input devices: {}", e);
            return None;
        }
    };

    let mut pipewire_device = None;
    let mut monitor_device = None;
    let mut default_device = None;

    for (i, device) in devices.enumerate() {
        if let Ok(name) = device.name() {
            println!("  [{}] INPUT: {}", i, name);

            let name_lower = name.to_lowercase();
            if name_lower.contains("monitor") {
                monitor_device = Some(device);
                continue;
            }
            if name_lower == "pipewire" && pipewire_device.is_none() {
                pipewire_device = Some(device);
                continue;
            }
            if name_lower == "default" && default_device.is_none() {
                default_device = Some(device);
            }
        }
    }

    println!("\n=== Available OUTPUT devices ===");
    if let Ok(devices) = host.output_devices() {
        for (i, device) in devices.enumerate() {
            if let Ok(name) = device.name() {
                println!("  [{}] OUTPUT: {}", i, name);
            }
        }
    }

    println!("\nTip: Set AUDIO_DEVICE env var to select a specific device");
    println!("Example: AUDIO_DEVICE=pipewire cargo run --example chart_spectrum\n");

    monitor_device.or(pipewire_device).or(default_device)
}

fn setup_audio_capture_for_device(audio_data: Data, device: &cpal::Device) -> Option<cpal::Stream> {
    println!(
        "Setting up audio capture for device: {}",
        device.name().unwrap_or_default()
    );

    let config = match device.default_input_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to get default input config: {}", e);
            return None;
        }
    };

    let sample_rate = config.sample_rate().0 as f32;
    *audio_data.sample_rate.lock().unwrap() = sample_rate;

    let audio_buffer = Arc::new(Mutex::new(VecDeque::<f32>::with_capacity(MAX_BUFFER_SIZE)));
    let mut planner = RealFftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            build_input_stream::<f32>(device, &config.into(), audio_buffer, audio_data, fft)
        }
        cpal::SampleFormat::I16 => {
            build_input_stream::<i16>(device, &config.into(), audio_buffer, audio_data, fft)
        }
        cpal::SampleFormat::U16 => {
            build_input_stream::<u16>(device, &config.into(), audio_buffer, audio_data, fft)
        }
        _ => {
            eprintln!("Unsupported sample format: {:?}", config.sample_format());
            return None;
        }
    };

    match stream {
        Ok(stream) => {
            if let Err(e) = stream.play() {
                eprintln!("Failed to start stream: {}", e);
                None
            } else {
                println!("Audio capture running at {} Hz", sample_rate);
                Some(stream)
            }
        }
        Err(e) => {
            eprintln!("Failed to build input stream: {}", e);
            None
        }
    }
}

fn build_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    audio_buffer: Arc<Mutex<VecDeque<f32>>>,
    audio_data: Data,
    fft: Arc<dyn realfft::RealToComplex<f32>>,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::Sample + cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("Error on audio stream: {}", err);

    device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            process_audio_block(data, channels, &audio_buffer, &audio_data, &fft);
        },
        err_fn,
        None,
    )
}

#[allow(clippy::significant_drop_tightening)]
fn process_audio_block<T>(
    data: &[T],
    channels: usize,
    audio_buffer: &Arc<Mutex<VecDeque<f32>>>,
    audio_data: &Data,
    fft: &Arc<dyn realfft::RealToComplex<f32>>,
) where
    T: Sample,
    f32: cpal::FromSample<T>,
{
    let mut buffer = audio_buffer.lock().unwrap();

    for frame in data.chunks(channels) {
        let mono = frame.iter().map(|&s| f32::from_sample(s)).sum::<f32>() / channels as f32;
        buffer.push_back(mono);
    }

    while buffer.len() > MAX_BUFFER_SIZE {
        buffer.pop_front();
    }

    while buffer.len() >= FFT_SIZE {
        let mut block: Vec<f32> = buffer.iter().take(FFT_SIZE).copied().collect();
        math::apply_blackman_harris(&mut block);

        let mut spectrum = fft.make_output_vec();
        if fft.process(&mut block, &mut spectrum).is_ok() {
            let mut magnitudes = vec![0.0f32; FFT_SIZE / 2 + 1];
            for (i, value) in spectrum.iter().enumerate().take(magnitudes.len()) {
                let mut amplitude = value.norm() * FFT_GAIN_CORRECTION / FFT_SIZE as f32;
                if i == 0 || i == magnitudes.len() - 1 {
                    amplitude *= 0.5;
                }
                magnitudes[i] = amplitude;
            }

            let mut shared = audio_data.spectrum.lock().unwrap();
            for (slot, mag) in shared.iter_mut().zip(magnitudes.iter()) {
                *slot = TEMPORAL_SMOOTHING.mul_add(*slot, (1.0 - TEMPORAL_SMOOTHING) * *mag);
            }
        }

        let drain = HOP_SIZE.min(buffer.len());
        for _ in 0..drain {
            buffer.pop_front();
        }
    }
}
