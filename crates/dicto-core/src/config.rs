use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Central configuration — one file, one place.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // --- Model ---
    /// Model path: "tiny", "base", or absolute path to .bin
    pub model_size: String,
    /// Force language: "auto", "en", "fr"
    pub language: String,

    // --- Hotkey ---
    /// Virtual-key code (e.g. "VK_F4" or "VK_OEM_3" for `)
    pub hotkey_key: String,
    /// Modifiers: comma-separated, e.g. "ctrl,shift"
    pub hotkey_modifiers: String,
    /// "push-to-talk" (hold) or "toggle-to-talk" (press once each)
    pub hotkey_mode: String,

    // --- Microphone ---
    /// Device name filter: "default" or partial name
    pub mic_name: String,

    // --- Overlay ---
    /// "always", "recording", "never"
    pub overlay_mode: String,
    pub overlay_x: i32,
    pub overlay_y: i32,
    pub overlay_width: u32,
    pub overlay_font_size: u32,
    pub overlay_max_lines: u32,

    // --- Output ---
    pub auto_paste: bool,
    pub trailing_space: bool,

    // --- Formatting rules toggles ---
    pub space_before_punctuation: bool,
    pub text_to_number: bool,
    pub auto_capitalize_threshold: usize,

    // --- LLM reformulation (optional) ---
    pub llm_enabled: bool,
    pub llm_api_url: String,
    pub llm_api_key: String,
    pub llm_model: String,
    pub llm_preset: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model_size: "base".into(),
            language: "auto".into(),
            hotkey_key: "VK_F4".into(),
            hotkey_modifiers: "ctrl,shift".into(),
            hotkey_mode: "push-to-talk".into(),
            mic_name: "default".into(),
            overlay_mode: "recording".into(),
            overlay_x: 100,
            overlay_y: 100,
            overlay_width: 500,
            overlay_font_size: 16,
            overlay_max_lines: 5,
            auto_paste: true,
            trailing_space: false,
            space_before_punctuation: true,
            text_to_number: false,
            auto_capitalize_threshold: 3,
            llm_enabled: false,
            llm_api_url: "https://openrouter.ai/api/v1".into(),
            llm_api_key: "".into(),
            llm_model: "deepseek/deepseek-v4-flash".into(),
            llm_preset: "voice-dictation".into(),
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        let base = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs_or_home()
            });
        base.join("Dicto")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            let cfg = Config::default();
            let _ = cfg.save();
            cfg
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }
}

fn dirs_or_home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}