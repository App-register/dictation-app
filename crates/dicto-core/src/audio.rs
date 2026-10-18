use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Audio capture engine — push-to-talk semantics.
pub struct AudioCapture {
    device: Device,
    config: StreamConfig,
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
                let id = name.clone(); // simplified; use hash in real impl
                Some((id, name))
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

        let config = device
            .default_input_config()
            .map_err(|e| format!("No default config: {}", e))?;

        // We want 16kHz mono f32 for whisper.cpp
        let stream_config = StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(16000),
            buffer_size: cpal::BufferSize::Fixed(1024),
        };

        Ok(Self {
            device,
            config: stream_config,
            buffer: Arc::new(Mutex::new(Vec::with_capacity(16000 * 30))), // 30s max
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
                &self.config,
                move |data: &[f32], _info| {
                    if is_recording.load(Ordering::SeqCst) {
                        let mut buf = buffer.lock().unwrap();
                        buf.extend_from_slice(data);
                        // Cap at 5 min (300s × 16000 samples)
                        if buf.len() > 4_800_000 {
                            buf.clear();
                        }
                    }
                },
                move |err| {
                    let _ = err_tx.send(err);
                },
                None, // timeout
            )
            .map_err(|e| format!("Failed to build stream: {}", e))?;

        stream.play().map_err(|e| format!("Failed to play stream: {}", e))?;
        self.stream = Some(stream);
        Ok(())
    }

    /// Stop recording and return the captured audio (16kHz mono f32).
    pub fn stop(&mut self) -> Vec<f32> {
        self.is_recording.store(false, Ordering::SeqCst);
        if let Some(s) = self.stream.take() {
            let _ = s.pause();
            drop(s);
        }
        let buf = self.buffer.lock().unwrap().clone();
        buf
    }
}