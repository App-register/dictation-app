use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Audio capture engine — push-to-talk semantics.
pub struct AudioCapture {
    device: Device,
    /// The config we actually built the stream with (device's default).
    device_config: StreamConfig,
    buffer: Arc<Mutex<Vec<f32>>>,
    is_recording: Arc<AtomicBool>,
    stream: Option<Stream>,
}

impl AudioCapture {
    /// List all available input devices.
    pub fn list_devices() -> Vec<(String, String)> {
        let host = cpal::default_host();
        host.input_devices()
            .unwrap()
            .filter_map(|d| {
                let name = d.name().ok()?;
                Some((name.clone(), name))
            })
            .collect()
    }

    /// Open a device by name (partial match), or default.
    pub fn open(device_name: &str) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = if device_name == "default" || device_name.is_empty() {
            host.default_input_device()
                .ok_or_else(|| "No default input device".to_string())?
        } else {
            host.input_devices()
                .unwrap()
                .find(|d| d.name().ok().map(|n| n.contains(device_name)).unwrap_or(false))
                .ok_or_else(|| format!("Device '{}' not found", device_name))?
        };

        // Use the device's native configuration — guaranteed supported.
        let supported = device
            .default_input_config()
            .map_err(|e| format!("No default config: {}", e))?;

        let rate = supported.sample_rate.max(cpal::SampleRate(8000));
        let stream_config = StreamConfig {
            channels: supported.channels,
            sample_rate: rate,
            buffer_size: cpal::BufferSize::Default,
        };

        Ok(Self {
            device,
            device_config: stream_config,
            buffer: Arc::new(Mutex::new(Vec::with_capacity(16000 * 30))),
            is_recording: Arc::new(AtomicBool::new(false)),
            stream: None,
        })
    }

    /// Start capturing audio into the ring buffer.
    pub fn start(&mut self) -> Result<(), String> {
        self.is_recording.store(true, Ordering::SeqCst);
        *self.buffer.lock().unwrap() = Vec::with_capacity(16000 * 30);

        let buffer = self.buffer.clone();
        let is_recording = self.is_recording.clone();
        let (err_tx, _err_rx) = std::sync::mpsc::channel();

        let stream = self.device
            .build_input_stream(
                &self.device_config,
                move |data: &[f32], _info| {
                    if is_recording.load(Ordering::SeqCst) {
                        let mut buf = buffer.lock().unwrap();
                        buf.extend_from_slice(data);
                        if buf.len() > 4_800_000 {
                            buf.clear();
                        }
                    }
                },
                move |err| {
                    let _ = err_tx.send(err);
                },
                None,
            )
            .map_err(|e| format!("Failed to build stream: {}\nDevice config: {:?}\nTry selecting a different microphone in Settings.", e, self.device_config))?;

        stream.play().map_err(|e| format!("Failed to play stream: {}", e))?;
        self.stream = Some(stream);
        Ok(())
    }

    /// Stop recording and return resampled 16kHz mono f32 audio.
    pub fn stop(&mut self) -> Vec<f32> {
        self.is_recording.store(false, Ordering::SeqCst);
        if let Some(s) = self.stream.take() {
            let _ = s.pause();
            drop(s);
        }
        let raw = self.buffer.lock().unwrap().clone();
        let src_rate = self.device_config.sample_rate.0;
        let src_channels = self.device_config.channels;
        // Only resample if needed
        if src_rate == 16000 && src_channels == 1 {
            raw
        } else {
            resample_to_16khz_mono(&raw, src_rate, src_channels)
        }
    }
}

/// Resample any audio to 16kHz mono f32 using nearest-neighbour interpolation.
fn resample_to_16khz_mono(samples: &[f32], src_rate: u32, src_channels: u16) -> Vec<f32> {
    let src_frames = samples.len() / src_channels as usize;
    if src_frames == 0 {
        return Vec::new();
    }
    let dst_frames = (src_frames as f64 * 16000.0 / src_rate as f64).ceil() as usize;
    let mut out = Vec::with_capacity(dst_frames);

    for i in 0..dst_frames {
        let src_idx = ((i as f64 * src_rate as f64 / 16000.0) as usize).min(src_frames - 1);
        let offset = src_idx * src_channels as usize;
        let mono: f32 = if src_channels == 1 {
            samples[offset]
        } else {
            samples[offset..offset + src_channels as usize].iter().sum::<f32>() / src_channels as f32
        };
        out.push(mono);
    }
    out
}