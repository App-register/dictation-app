use dicto_core::{AudioCapture, Config, Dictionary, FormattingEngine, OutputController, SttEngine};
use dicto_gui::hook;
use dicto_gui::{AppState, RecordingOverlay, SettingsWindow};
use nwg::NativeUi;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .ok();

    log::info!("Dicto is starting...");

    let state = AppState::new();

    // ── 1. Settings window (blocking) ──
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").ok();

    let settings = SettingsWindow::new(state.config.clone(), state.dictionary.clone());
    match SettingsWindow::build_ui(settings) {
        Ok(_ui) => {
            nwg::dispatch_thread_events();
        }
        Err(e) => {
            log::error!("Settings window failed: {}", e);
            return;
        }
    }
    log::info!("Settings closed.");

    let cfg = state.config.lock().unwrap().clone();
    let dict = state.dictionary.lock().unwrap().clone();

    // ── 2. Dictation engine on its own thread (own NWG init + message loop) ──
    let handler = std::thread::spawn(move || {
        dictation_loop(cfg, dict);
    });
    let _ = handler.join();
}

fn dictation_loop(cfg: Config, dict: Dictionary) {
    nwg::init().expect("Failed to init NWG (listener)");
    nwg::Font::set_global_family("Segoe UI").ok();

    // Overlay
    let overlay = RecordingOverlay::new().expect("Failed to create overlay");
    overlay.set_position(cfg.overlay_x, cfg.overlay_y);
    overlay.hide();

    // Pre-compute values the Send closure needs (no borrows of cfg inside)
    let trigger_vk = match hook::parse_key_name(&cfg.hotkey_key) {
        Some(vk) => vk,
        None => {
            log::error!("Invalid hotkey key: '{}'", cfg.hotkey_key);
            return;
        }
    };
    let mods: Vec<String> = cfg
        .hotkey_modifiers
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let ov_x = cfg.overlay_x;
    let ov_y = cfg.overlay_y;
    let mic_name = cfg.mic_name.clone();
    let language = cfg.language.clone();
    let model_path = SttEngine::resolve_model_path(&cfg.model_size);
    let auto_paste = cfg.auto_paste;

    let mut stt = SttEngine::new(&model_path, &language);
    let mut output = OutputController::new(auto_paste);
    let mut capture: Option<AudioCapture> = None;
    let formatting = FormattingEngine::from_config(&cfg);

    // Safe: the overlay lives on THIS thread and is only touched here.
    #[derive(Copy, Clone)]
    struct OverlayRef(*const RecordingOverlay);
    unsafe impl Send for OverlayRef {}
    let overlay_ref = OverlayRef(&overlay as *const RecordingOverlay);

    let recording = Arc::new(AtomicBool::new(false));
    let rec = recording.clone();

    // Cancel button: stop recording and hide (same thread as overlay)
    overlay.on_cancel({
        let rec = recording.clone();
        let ov_ptr = overlay_ref.0;
        move || {
            rec.store(false, Ordering::SeqCst);
            let ov = unsafe { &*ov_ptr };
            ov.update_text("Cancelled");
            ov.hide();
        }
    });

    log::info!(
        "Listening for hotkey: {} (mods: {:?})",
        cfg.hotkey_key, mods
    );

    let _ = hook::run_keyboard_listener(move |is_down, vk| {
        if vk != trigger_vk || !mods_match(&mods) {
            return;
        }

        if is_down {
            // ── START recording ──
            if rec.load(Ordering::SeqCst) {
                return; // key auto-repeat guard
            }
            rec.store(true, Ordering::SeqCst);

            let ov = unsafe { &*overlay_ref.0 };
            ov.set_position(ov_x, ov_y);
            ov.show();
            ov.update_text("Recording…");

            match AudioCapture::open(&mic_name) {
                Ok(mut cap) => match cap.start() {
                    Ok(_) => {
                        capture = Some(cap);
                        log::info!("▶ Recording started");
                    }
                    Err(e) => {
                        log::error!("Mic start failed: {}", e);
                        ov.update_text("Mic error — check settings");
                        rec.store(false, Ordering::SeqCst);
                    }
                },
                Err(e) => {
                    log::error!("Mic open failed: {}", e);
                    ov.update_text("Mic not found — check settings");
                    rec.store(false, Ordering::SeqCst);
                }
            }
        } else {
            // ── STOP recording → transcribe → format → paste ──
            if !rec.load(Ordering::SeqCst) {
                return;
            }
            rec.store(false, Ordering::SeqCst);

            let ov = unsafe { &*overlay_ref.0 };
            ov.update_text("Transcribing…");

            let samples = match capture.take() {
                Some(mut cap) => cap.stop(),
                None => Vec::new(),
            };

            if samples.len() < 1600 {
                log::warn!("Audio too short ({} samples)", samples.len());
                ov.update_text("Too short — try again");
                ov.hide();
                return;
            }

            // STT (latency-critical)
            let raw = match stt.transcribe(&samples) {
                Ok(t) => t,
                Err(e) => {
                    log::error!("STT error: {}", e);
                    ov.update_text(&format!("STT error: {}", e));
                    ov.hide();
                    return;
                }
            };
            log::info!("Raw: {}", raw);

            // Post-processing: dictionary → formatting rules
            let corrected = dict.apply(&raw);
            let final_text = formatting.process(&corrected);
            log::info!("Final: {}", final_text);

            ov.update_text(&final_text);

            if !final_text.is_empty() {
                if let Err(e) = output.paste(&final_text) {
                    log::error!("Paste failed: {}", e);
                }
            }

            // Brief display then hide
            std::thread::sleep(std::time::Duration::from_millis(600));
            ov.hide();
        }
    });
}

/// Check configured modifiers are currently held via GetAsyncKeyState.
fn mods_match(mods: &[String]) -> bool {
    let mut any = false;
    for m in mods {
        any = true;
        let vk = match m.as_str() {
            "ctrl" | "control" => dicto_gui::hook::vk::VK_CONTROL,
            "alt" | "menu" => dicto_gui::hook::vk::VK_MENU,
            "shift" => dicto_gui::hook::vk::VK_SHIFT,
            "win" | "lwin" => dicto_gui::hook::vk::VK_LWIN,
            _ => continue,
        };
        if !dicto_gui::hook::is_modifier_down(vk) {
            return false;
        }
    }
    any
}