use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Manages the global hotkey using win-hotkeys.
pub struct HotkeyManager {
    manager: Option<win_hotkeys::HotkeyManager>,
    pub is_recording: Arc<AtomicBool>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            manager: None,
            is_recording: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the hotkey event loop (blocking — runs until app exit).
    pub fn start(
        &mut self,
        key: &str,
        modifiers: &[&str],
        on_activate: impl Fn(bool) + Send + 'static,
    ) -> Result<(), String> {
        use win_hotkeys::{HotkeyManager as Hkm, VKey};

        let mut hkm = Hkm::new();
        let recording = self.is_recording.clone();

        // Parse trigger key
        let trigger = VKey::from_keyname(key)
            .or_else(|| VKey::from_keyname(&format!("VK_{}", key)))
            .or_else(|| {
                key.chars().next().and_then(|c| {
                    if c.is_alphanumeric() {
                        VKey::from_keyname(&c.to_ascii_uppercase().to_string())
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| format!("Invalid key: {}", key))?;

        // Parse modifiers
        let mods: Vec<VKey> = modifiers.iter()
            .filter_map(|m| {
                let m = m.trim().to_lowercase();
                match m.as_str() {
                    "ctrl" | "control" | "lctrl" => Some(VKey::LControl),
                    "rctrl" => Some(VKey::RControl),
                    "alt" | "lalt" | "menu" => Some(VKey::LMenu),
                    "ralt" => Some(VKey::RMenu),
                    "shift" | "lshift" => Some(VKey::LShift),
                    "rshift" => Some(VKey::RShift),
                    "win" | "lwin" => Some(VKey::LWin),
                    "rwin" => Some(VKey::RWin),
                    _ => VKey::from_keyname(m.trim()),
                }
            })
            .collect();

        hkm.register_hotkey(trigger, &mods, move || {
            let currently = recording.load(Ordering::SeqCst);
            recording.store(!currently, Ordering::SeqCst);
            on_activate(!currently);
        }).map_err(|e| format!("Hotkey registration failed: {}", e))?;

        log::info!("Hotkey registered: {:?} + {:?}", mods, key);

        // Blocking event loop
        hkm.event_loop();
        Ok(())
    }
}