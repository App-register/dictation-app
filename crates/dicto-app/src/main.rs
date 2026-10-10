use dicto_core::*;
use dicto_gui::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn main() {
    // 1. Init logger
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .ok();

    log::info!("Dicto starting...");

    // 2. Load config
    let config = Arc::new(Mutex::new(Config::load()));
    let is_recording = Arc::new(AtomicBool::new(false));
    let audio_capture = Arc::new(Mutex::new(None::<AudioCapture>));
    let stt_engine = Arc::new(Mutex::new(
        SttEngine::new(
            &SttEngine::resolve_model_path(&config.lock().unwrap().model_size),
            &config.lock().unwrap().language,
        )
    ));
    let output = Arc::new(Mutex::new(
        OutputController::new(config.lock().unwrap().auto_paste)
    ));

    // 3. Create overlay
    let overlay = Arc::new(RecordingOverlay::new().expect("Failed to create overlay"));
    overlay.hide();

    // 4. Register global hotkey
    let cfg = config.lock().unwrap().clone();
    let hk_is_recording = is_recording.clone();
    let hk_audio = audio_capture.clone();
    let hk_stt = stt_engine.clone();
    let hk_output = output.clone();
    let hk_overlay = overlay.clone();
    let hk_cfg = config.clone();

    let hotkey_callback = move || {
        let recording = hk_is_recording.load(Ordering::SeqCst);

        // --- START RECORDING ---
        if !recording {
            log::info!("▶️ Recording started");
            hk_is_recording.store(true, Ordering::SeqCst);
            hk_overlay.show();
            hk_overlay.update_text("🎤 Recording...");

            let mut audio = hk_audio.lock().unwrap();
            let cfg = hk_cfg.lock().unwrap();
            let mut capturer = AudioCapture::open(&cfg.mic_name)
                .expect("Failed to open microphone");
            capturer.start().expect("Failed to start capture");
            *audio = Some(capturer);

        // --- STOP RECORDING ---
        } else {
            log::info!("⏹ Recording stopped");
            hk_is_recording.store(false, Ordering::SeqCst);
            hk_overlay.update_text("⏳ Transcribing...");

            // Capture audio from mic
            let audio_samples = {
                let mut audio = hk_audio.lock().unwrap();
                audio.as_mut().map(|a| a.stop())
            }.unwrap_or_default();

            let duration_ms = (audio_samples.len() as f64 / 16000.0 * 1000.0) as u64;
            log::info!("Captured {} samples ({}.{}s)", audio_samples.len(),
                duration_ms / 1000, duration_ms % 1000);

            if audio_samples.len() < 1600 {
                log::warn!("Audio too short (< 0.1s), skipping");
                hk_overlay.update_text("Too short, try again");
                hk_overlay.hide();
                return;
            }

            // Transcribe (this is the latency-critical path)
            let transcribe_start = Instant::now();
            let raw_text = match hk_stt.lock().unwrap().transcribe(&audio_samples) {
                Ok(t) => t,
                Err(e) => {
                    log::error!("Transcription error: {}", e);
                    hk_overlay.update_text(&format!("Error: {}", e));
                    hk_overlay.hide();
                    return;
                }
            };
            let stt_duration = transcribe_start.elapsed();
            log::info!("Transcription done in {:?}: \"{}\"", stt_duration, &raw_text);

            // Post-process
            let cfg = hk_cfg.lock().unwrap();
            let dict = Dictionary::load();
            let formatting = FormattingEngine::from_config(&cfg);

            let dict_corrected = dict.apply(&raw_text);
            let final_text = formatting.process(&dict_corrected);
            let total_duration = transcribe_start.elapsed();
            log::info!("Post-processing done. Total: {:?}", total_duration);

            // Update overlay with result
            hk_overlay.update_text(&final_text);

            // Paste to active window
            if !final_text.is_empty() {
                hk_output.lock().unwrap().paste(&final_text)
                    .unwrap_or_else(|e| log::error!("Paste error: {}", e));
                log::info!("Text pasted: \"{}\"", &final_text);
            }

            // Hide overlay after brief delay
            std::thread::sleep(std::time::Duration::from_millis(500));
            hk_overlay.hide();
        }
    };

    let mut hotkey_mgr = HotkeyBridge::register(
        &cfg.hotkey_key,
        &cfg.hotkey_modifiers.split(',').collect::<Vec<_>>(),
        hotkey_callback,
    ).expect("Failed to register hotkey");

    log::info!("Dicto ready! Hotkey: {} + {}",
        cfg.hotkey_modifiers, cfg.hotkey_key);

    // 5. Main event loop (process window messages + hotkeys)
    hotkey_mgr.event_loop();
}