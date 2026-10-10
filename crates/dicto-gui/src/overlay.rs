use native_windows_gui as nwg;
use crate::DictoError;

/// Recording overlay — transparent, always-on-top, non-focusable window.
pub struct RecordingOverlay {
    window: nwg::Window,
    text_label: nwg::Label,
    cancel_button: nwg::Button,
}

impl RecordingOverlay {
    /// Create the overlay window (hidden initially).
    pub fn new() -> Result<Self, nwg::NwgError> {
        let mut window = Default::default();
        let mut text_label = Default::default();
        let mut cancel_button = Default::default();

        // WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE
        // Combined with WS_POPUP and WS_VISIBLE
        let flags = nwg::WindowFlags::POPUP | nwg::WindowFlags::VISIBLE;
        // Plus extended styles via builder

        nwg::Window::builder()
            .flags(flags)
            .size((500, 200))
            .position((100, 100))
            .title("Dicto Overlay")
            .build(&mut window)?;

        // Make it click-through and always-on-top via winapi extended styles
        unsafe {
            let hwnd = window.handle.0 as winapi::shared::windef::HWND;
            let ex_style = winapi::um::winuser::GetWindowLongW(hwnd, winapi::um::winuser::GWL_EXSTYLE)
                | winapi::um::winuser::WS_EX_LAYERED
                | winapi::um::winuser::WS_EX_TRANSPARENT
                | winapi::um::winuser::WS_EX_TOOLWINDOW
                | winapi::um::winuser::WS_EX_NOACTIVATE
                | winapi::um::winuser::WS_EX_APPWINDOW;
            winapi::um::winuser::SetWindowLongW(hwnd, winapi::um::winuser::GWL_EXSTYLE, ex_style);
            // Set transparency to 90%
            winapi::um::winuser::SetLayeredWindowAttributes(hwnd, 0, 230, winapi::um::winuser::LWA_ALPHA);
        }

        // Text label for real-time transcription
        nwg::Label::builder()
            .text("Recording...")
            .font_size(16)
            .parent(&window)
            .build(&mut text_label)?;

        // Cancel button
        nwg::Button::builder()
            .text("✕ Cancel")
            .parent(&window)
            .build(&mut cancel_button)?;

        Ok(Self {
            window,
            text_label,
            cancel_button,
        })
    }

    pub fn show(&self) {
        nwg::set_window_pos(&self.window, 100, 100);
        // WS_EX_LAYERED already set in constructor
        let _ = nwg::show_window(&self.window);
    }

    pub fn hide(&self) {
        let _ = nwg::hide_window(&self.window);
    }

    pub fn update_text(&self, text: &str) {
        self.text_label.set_text(text);
    }

    pub fn window_handle(&self) -> nwg::ControlHandle {
        self.window.handle.clone()
    }
}

impl Drop for RecordingOverlay {
    fn drop(&mut self) {
        self.hide();
    }
}