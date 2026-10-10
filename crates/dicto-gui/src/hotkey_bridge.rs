/// Hotkey bridge — wraps win-hotkeys and translates to Dicto actions.
pub struct HotkeyBridge;

impl HotkeyBridge {
    /// Register a global hotkey and return a receiver.
    pub fn register(
        key: &str,
        modifiers: &[&str],
        callback: impl Fn() + Send + 'static,
    ) -> Result<win_hotkeys::HotkeyManager, String> {
        use win_hotkeys::{HotkeyManager, VKey};

        let mut hkm = HotkeyManager::new();

        // Parse the trigger key
        let trigger = VKey::from_keyname(key)
            .or_else(|| VKey::from_keyname(&format!("VK_{}", key)))
            .or_else(|| {
                // Try OEM keys: if key is a single char, use its VKey
                key.chars().next().and_then(|c| {
                    if c.is_alphanumeric() {
                        VKey::from_keyname(&c.to_ascii_uppercase().to_string())
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| format!("Invalid key code: {}", key))?;

        let mods: Vec<VKey> = modifiers.iter()
            .filter_map(|m| VKey::from_keyname(m))
            .collect();

        hkm.register_hotkey(trigger, &mods, callback)
            .map_err(|e| format!("Failed to register hotkey: {}", e))?;

        Ok(hkm)
    }
}