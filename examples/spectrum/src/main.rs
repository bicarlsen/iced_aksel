//! Professional-style audio spectrum analyzer built on the new charting engine.
//!
//! The example focuses on a single smooth curve with a subtle glow/fill to mimic
//! modern mastering tools (FabFilter Pro-Q, Izotope). Real-time audio is captured
//! through `cpal`, transformed with an FFT and rendered on a logarithmic axis.

use std::{
    collections::VecDeque,
    f32::consts::PI,
    sync::{Arc, Mutex},
    time::Instant,
};

use aksel::{PlotPoint, scale};
use cpal::{
    Sample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use iced::{
    Element, Pixels, Subscription, Task, Theme,
    theme::{Base, Mode},
    widget::{column, pick_list, row, text},
    window,
};
use iced_aksel::{
    Axis, Chart, Length, Plot, State, Stroke,
    axis::{GridLine, Label, Position, TickLabelContext, TickLine},
    plot, shape,
};
use realfft::RealFftPlanner;

type AxisId = &'static str;

const FREQ_AXIS_ID: AxisId = "freq";
const DB_AXIS_ID: AxisId = "db";

const MIN_FREQ: f64 = 15.0;
const MAX_FREQ: f64 = 22_000.0;
const MIN_DB: f64 = -90.0;
const MAX_DB: f64 = 12.0;

const FFT_SIZE: usize = 8192;
const HOP_SIZE: usize = 512;
const MAX_BUFFER_SIZE: usize = FFT_SIZE * 6;
const TEMPORAL_SMOOTHING: f32 = 0.9;
const POINTS_PER_OCTAVE: usize = 72;
const FFT_GAIN_CORRECTION: f32 = 2.0;

const SMOOTHING_BAND_SHAPE: &[(f64, f64)] = &[
    (20.0, 1.1),
    (50.0, 0.95),
    (100.0, 0.7),
    (200.0, 0.5),
    (400.0, 0.35),
    (1000.0, 0.25),
    (3000.0, 0.19),
    (8000.0, 0.16),
    (20_000.0, 0.13),
];

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    DeviceSelected(String),
    SwitchTheme(iced::Theme),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeviceInfo {
    name: String,
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
struct AudioData {
    spectrum: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<Mutex<f32>>,
}

impl AudioData {
    fn new() -> Self {
        Self {
            spectrum: Arc::new(Mutex::new(vec![0.0; FFT_SIZE / 2 + 1])),
            sample_rate: Arc::new(Mutex::new(48_000.0)),
        }
    }
}

#[derive(Default, Clone)]
struct SpectrumLayer {
    curve: Vec<PlotPoint<f64>>,
}

impl SpectrumLayer {
    fn set_curve(&mut self, curve: Vec<PlotPoint<f64>>) {
        self.curve = curve;
    }
}

impl<R: plot::Renderer> plot::Items<f64, R> for SpectrumLayer {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, theme: &iced::Theme) {
        if self.curve.len() < 2 {
            return;
        }

        let palette = theme.extended_palette();

        let mut fill_points = Vec::with_capacity(self.curve.len() + 2);
        fill_points.push(PlotPoint::new(MIN_FREQ, MIN_DB));
        fill_points.extend(self.curve.iter().copied());
        fill_points.push(PlotPoint::new(MAX_FREQ, MIN_DB));

        plot.add_shape(
            shape::Polygon::new(fill_points).fill(palette.primary.base.color.scale_alpha(0.4)),
        );

        let glow_color = if theme.mode() == Mode::Light {
            palette.primary.strong.color
        } else {
            palette.primary.weak.color
        };

        let glow_stroke = Stroke::new(glow_color, Length::Screen(6.0));
        plot.add_shape(shape::Polyline::new(self.curve.clone(), glow_stroke));

        let line_stroke = Stroke::new(palette.background.base.text, Length::Screen(2.2));
        plot.add_shape(shape::Polyline::new(self.curve.clone(), line_stroke));
    }
}

struct AnalyzerApp {
    current_theme: iced::Theme,
    state: State<AxisId, f64>,
    spectrum_layer: SpectrumLayer,
    audio_data: AudioData,
    _stream: Option<cpal::Stream>,
    available_devices: Vec<DeviceInfo>,
    selected_device: Option<DeviceInfo>,
    last_frame_time: Option<Instant>,
    fps: f32,
}

impl AnalyzerApp {
    fn init() -> (Self, Task<Message>) {
        let mut state: State<AxisId, f64> = State::new();

        let axis_x = Axis::new(
            scale::Logarithmic::new(10.0, MIN_FREQ, MAX_FREQ),
            Position::Bottom,
        )
        .with_grid_renderer(|_tick| {
            Some(GridLine {
                thickness: 1.0.into(),
            })
        })
        .with_tick_renderer(frequency_tick_renderer)
        .skip_overlapping_labels(8.0);
        let axis_y = Axis::new(scale::Linear::new(MIN_DB, MAX_DB), Position::Left)
            .with_grid_renderer(|tick| {
                if tick.level > 1 {
                    return None;
                }

                Some(GridLine {
                    thickness: 1.0.into(),
                })
            })
            .with_tick_renderer(db_tick_renderer)
            .skip_overlapping_labels(8.0)
            .with_cursor_formatter(|value| {
                Some(Label {
                    size: 10.0.into(),
                    content: format_db_label(value),
                })
            });

        state.set_axis(FREQ_AXIS_ID, axis_x);
        state.set_axis(DB_AXIS_ID, axis_y);

        let audio_data = AudioData::new();
        let host = cpal::default_host();
        let available_devices = enumerate_devices(&host);
        let (stream, selected_device) = setup_audio_capture_with_device(audio_data.clone(), None);

        (
            Self {
                current_theme: iced::Theme::Dark,
                state,
                spectrum_layer: SpectrumLayer::default(),
                audio_data,
                _stream: stream,
                available_devices,
                selected_device,
                last_frame_time: None,
                fps: 0.0,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(now) => {
                if let Some(last) = self.last_frame_time {
                    let dt = now.duration_since(last).as_secs_f32();
                    if dt > 0.0 {
                        let instant = 1.0 / dt;
                        self.fps = self.fps.mul_add(0.85, instant * 0.15);
                    }
                }
                self.last_frame_time = Some(now);

                self.rebuild_curve();
            }
            Message::DeviceSelected(device_name) => {
                println!("Switching to device: {}", device_name);
                let (stream, selected_device) =
                    setup_audio_capture_with_device(self.audio_data.clone(), Some(device_name));
                self._stream = stream;
                self.selected_device = selected_device;
            }
            Message::SwitchTheme(theme) => {
                self.current_theme = theme;
            }
        }
    }

    fn rebuild_curve(&mut self) {
        let magnitudes = self.audio_data.spectrum.lock().unwrap().clone();
        let sample_rate = *self.audio_data.sample_rate.lock().unwrap() as f64;

        let log_min = MIN_FREQ.log10();
        let log_max = MAX_FREQ.log10();
        let octaves = (log_max - log_min) / (2.0_f64).log10();
        let num_points = (octaves * POINTS_PER_OCTAVE as f64).round().max(32.0) as usize;
        let step = (log_max - log_min) / num_points as f64;

        let mut curve = Vec::with_capacity(num_points);
        for i in 0..num_points {
            let freq = 10_f64.powf(log_min + step * i as f64);
            let width = fractional_width(freq);
            let db = sample_fractional_octave(&magnitudes, freq, sample_rate, width);
            curve.push(PlotPoint::new(freq, db));
        }

        self.spectrum_layer.set_curve(curve);
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.state).layer(&self.spectrum_layer, FREQ_AXIS_ID, DB_AXIS_ID);

        let pick_row = row![
            text("Audio Input: "),
            pick_list(
                self.available_devices.as_slice(),
                self.selected_device.as_ref(),
                |device| Message::DeviceSelected(device.name)
            ),
            text("Theme: "),
            pick_list(iced::Theme::ALL, Some(&self.current_theme), |t| {
                Message::SwitchTheme(t)
            }),
        ]
        .spacing(12);

        let sample_rate = *self.audio_data.sample_rate.lock().unwrap();
        let info = row![
            text(format!("SR: {:.0} Hz", sample_rate)).size(16),
            text(format!("FPS: {:.1}", self.fps)).size(16),
        ]
        .spacing(24);

        column![pick_row, info, chart]
            .spacing(16)
            .padding(16)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::frames().map(Message::Tick)
    }

    fn theme(&self) -> Theme {
        self.current_theme.clone()
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Self::theme)
            .subscription(Self::subscription)
            .antialiasing(true)
            .run()
    }
}

fn frequency_tick_renderer(ctx: TickLabelContext<f64>) -> Option<TickLine> {
    let mut line = TickLine {
        thickness: Pixels(1.0),
        length: Pixels(if ctx.tick.level == 0 { 12.0 } else { 6.0 }),
        label: None,
    };

    line.label = Some(Label {
        size: Pixels(10.0),
        content: format_frequency_label(ctx.tick.value),
    });

    Some(line)
}

fn db_tick_renderer(ctx: TickLabelContext<f64>) -> Option<TickLine> {
    let mut line = TickLine {
        thickness: Pixels(1.0),
        length: Pixels(if ctx.tick.level == 0 { 10.0 } else { 5.0 }),
        label: None,
    };

    if ctx.tick.level <= 1 {
        line.label = Some(Label {
            size: Pixels(8.0),
            content: format_db_label(ctx.tick.value),
        });
    }

    Some(line)
}

fn format_frequency_label(value: f64) -> String {
    if value >= 10_000.0 {
        format!("{:.0} kHz", value / 1000.0)
    } else if value >= 1000.0 {
        format!("{:.1} kHz", value / 1000.0)
    } else {
        format!("{:.0} Hz", value)
    }
}

fn format_db_label(value: f64) -> String {
    format!("{:+.0} dB", value)
}

fn fractional_width(freq: f64) -> f64 {
    let freq = freq.clamp(MIN_FREQ, MAX_FREQ);
    let log_freq = freq.log10();

    if freq <= SMOOTHING_BAND_SHAPE[0].0 {
        return SMOOTHING_BAND_SHAPE[0].1;
    }

    for window in SMOOTHING_BAND_SHAPE.windows(2) {
        let (f0, w0) = window[0];
        let (f1, w1) = window[1];

        if freq <= f1 {
            if (f1 - f0).abs() < f64::EPSILON {
                return w1;
            }

            let t = (log_freq - f0.log10()) / (f1.log10() - f0.log10());
            return (w1 - w0).mul_add(t.clamp(0.0, 1.0), w0);
        }
    }

    SMOOTHING_BAND_SHAPE.last().map(|(_, w)| *w).unwrap_or(0.15)
}

fn sample_fractional_octave(
    magnitudes: &[f32],
    freq: f64,
    sample_rate: f64,
    width_octaves: f64,
) -> f64 {
    if magnitudes.len() < 2 || sample_rate <= 0.0 {
        return MIN_DB;
    }

    let nyquist = sample_rate * 0.5;
    let center = freq.clamp(MIN_FREQ, nyquist);
    let half_span = (width_octaves * 0.5).max(0.05);

    let sample_count = ((width_octaves * 96.0).round() as usize).clamp(12, 150);
    let mut weighted = 0.0;
    let mut weight_sum = 0.0;

    for i in 0..sample_count {
        let t = (i as f64 + 0.5) / sample_count as f64;
        let offset = width_octaves.mul_add(t, -half_span);
        let sample_freq = center * offset.exp2();
        if !(MIN_FREQ..=nyquist).contains(&sample_freq) {
            continue;
        }

        let dist = offset / half_span;
        let weight = (-0.5 * dist * dist).exp();
        let amplitude = interpolate_fft_bin(magnitudes, sample_freq, sample_rate);

        weighted += amplitude * weight;
        weight_sum += weight;
    }

    let amplitude = if weight_sum > 0.0 {
        weighted / weight_sum
    } else {
        0.0
    };
    amplitude_to_db(amplitude).clamp(MIN_DB, MAX_DB)
}

fn amplitude_to_db(value: f64) -> f64 {
    20.0 * value.max(1e-9).log10()
}

fn interpolate_fft_bin(magnitudes: &[f32], freq: f64, sample_rate: f64) -> f64 {
    if magnitudes.len() < 2 {
        return 0.0;
    }

    let fft_size = (magnitudes.len() - 1) * 2;
    if fft_size == 0 || sample_rate == 0.0 {
        return 0.0;
    }

    let nyquist = sample_rate * 0.5;
    let freq = freq.clamp(MIN_FREQ, nyquist);
    let bin_hz = sample_rate / fft_size as f64;
    let index = freq / bin_hz;

    let base = index.floor() as usize;
    if base >= magnitudes.len() - 1 {
        return magnitudes[magnitudes.len() - 1] as f64;
    }

    let next = base + 1;
    let frac = index - base as f64;
    let lower = magnitudes[base] as f64;
    let upper = magnitudes[next] as f64;

    (upper - lower).mul_add(frac, lower)
}

fn enumerate_devices(host: &cpal::Host) -> Vec<DeviceInfo> {
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

fn setup_audio_capture_with_device(
    audio_data: AudioData,
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

fn setup_audio_capture_for_device(
    audio_data: AudioData,
    device: &cpal::Device,
) -> Option<cpal::Stream> {
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
    audio_data: AudioData,
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
    audio_data: &AudioData,
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
        apply_blackman_harris(&mut block);

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

fn apply_blackman_harris(block: &mut [f32]) {
    let n = block.len().saturating_sub(1) as f32;
    if n <= 0.0 {
        return;
    }

    for (i, sample) in block.iter_mut().enumerate() {
        let k = i as f32 / n;
        let window = 0.35875 - 0.48829 * (2.0 * PI * k).cos() + 0.14128 * (4.0 * PI * k).cos()
            - 0.01168 * (6.0 * PI * k).cos();
        *sample *= window;
    }
}

fn main() -> iced::Result {
    AnalyzerApp::run()
}
