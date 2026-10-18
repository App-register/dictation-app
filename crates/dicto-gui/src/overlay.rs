use native_windows_gui as nwg;
use winapi::um::winuser::{
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
};

/// Recording overlay — transparent, always-on-top, click-through.
pub struct RecordingOverlay {
    window: nwg::Window,
    text_label: nwg::Label,
    cancel_button: nwg::Button,
}

impl RecordingOverlay {
    pub fn new() -> Result<Self, nwg::NwgError> {
        let mut window = Default::default();
        let mut text_label = Default::default();
        let mut cancel_button = Default::default();

        nwg::Window::builder()
            .flags(nwg::WindowFlags::POPUP | nwg::WindowFlags::VISIBLE)
            // Layered + click-through + no taskbar entry + never activates
            .ex_flags(WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE)
            .topmost(true)
            .size((420, 90))
            .position((100, 100))
            .title("")
            .build(&mut window)?;

        nwg::Label::builder()
            .text("Press the hotkey and speak…")
            .size((380, 40))
            .position((10, 5))
            .parent(&window)
            .build(&mut text_label)?;

        nwg::Button::builder()
            .text("Cancel")
            .size((60, 24))
            .position((350, 55))
            .parent(&window)
            .build(&mut cancel_button)?;

        Ok(Self {
            window,
            text_label,
            cancel_button,
        })
    }

    pub fn show(&self) {
        self.window.set_visible(true);
    }

    pub fn hide(&self) {
        self.window.set_visible(false);
    }

    pub fn update_text(&self, text: &str) {
        self.text_label.set_text(text);
        self.window.invalidate();
    }

    pub fn set_position(&self, x: i32, y: i32) {
        self.window.set_position(x, y);
    }

    pub fn on_cancel<F: Fn() + 'static>(&self, callback: F) {
        let clb = nwg::EventHandler::new(callback);
        nwg::bind_event_handler(&self.cancel_button.handle, clb);
    }
}