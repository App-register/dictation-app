use dicto_core::Config;
use dicto_gui::{SettingsWindow, AppState};
use nwg::NativeUi;
use std::sync::{Arc, Mutex};

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .ok();

    log::info!("Dicto starting...");

    let state = AppState::new();

    // Show settings window (blocking)
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set font");

    let settings = SettingsWindow::new(state.config.clone(), state.dictionary.clone());
    if let Err(e) = SettingsWindow::build_ui(settings) {
        log::error!("Settings window failed: {}", e);
        return;
    }

    nwg::dispatch_thread_events();
    log::info!("Settings closed. Starting dictation engine...");

    // After settings, read final config
    let cfg = state.config.lock().unwrap().clone();

    log::info!("Dicto ready! Hotkey: {} + {}", cfg.hotkey_modifiers, cfg.hotkey_key);
}