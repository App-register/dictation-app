/// Tray icon (stub — NWG system tray integration).
pub struct TrayIcon;

impl TrayIcon {
    pub fn new(_tooltip: &str) -> Result<Self, native_windows_gui::NwgError> {
        Ok(Self)
    }

    pub fn set_recording(&self, _active: bool) {
        // Change icon color when recording
    }
}