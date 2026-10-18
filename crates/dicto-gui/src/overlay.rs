use native_windows_gui as nwg;

/// Recording overlay — transparent, always-on-top, click-through.
pub struct RecordingOverlay {
    window: nwg::Window,
    text_label: nwg::Label,
    cancel_button: nwg::Button,
    visible: bool,
}

impl RecordingOverlay {
    pub fn new() -> Result<Self, nwg::NwgError> {
        let mut window = Default::default();
        let mut text_label = Default::default();
        let mut cancel_button = Default::default();

        nwg::Window::builder()
            .flags(nwg::WindowFlags::POPUP | nwg::WindowFlags::VISIBLE)
            .size((500, 100))
            .position((100, 100))
            .title("")
            .build(&mut window)?;

        // Extended styles: transparent, click-through, tool window, no activation
        unsafe {
            let hwnd: winapi::shared::windef::HWND = std::mem::transmute(
                window.handle.0
            );
            let ex = winapi::um::winuser::GetWindowLongW(hwnd, winapi::um::winuser::GWL_EXSTYLE)
                | winapi::um::winuser::WS_EX_LAYERED
                | winapi::um::winuser::WS_EX_TRANSPARENT
                | winapi::um::winuser::WS_EX_TOOLWINDOW
                | winapi::um::winuser::WS_EX_NOACTIVATE;
            winapi::um::winuser::SetWindowLongW(hwnd, winapi::um::winuser::GWL_EXSTYLE, ex);
            winapi::um::winuser::SetLayeredWindowAttributes(hwnd, 0, 220, winapi::um::winuser::LWA_ALPHA);
        }

        nwg::Label::builder()
            .text("🎤 Press hotkey and speak...")
            .font_size(18)
            .parent(&window)
            .build(&mut text_label)?;

        nwg::Button::builder()
            .text("✕")
            .parent(&window)
            .build(&mut cancel_button)?;

        nwg::GridLayout::builder()
            .parent(&window)
            .spacing(4)
            .child(0, 0, &text_label)
            .child_item(nwg::GridLayoutItem::new(&cancel_button, 0, 1, 1, 1))
            .build()?;

        Ok(Self {
            window,
            text_label,
            cancel_button,
            visible: false,
        })
    }

    pub fn show(&mut self) {
        if !self.visible {
            nwg::show_window(&self.window);
            self.visible = true;
        }
    }

    pub fn hide(&mut self) {
        if self.visible {
            nwg::hide_window(&self.window);
            self.visible = false;
        }
    }

    pub fn update_text(&self, text: &str) {
        self.text_label.set_text(text);
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_position(&self, x: i32, y: i32) {
        nwg::set_window_pos(&self.window, x, y);
    }

    pub fn on_cancel<F: Fn() + 'static>(&self, callback: F) {
        let clb = nwg::Event::OnButtonClick(
            nwg::EventHandler::new(move |_, _, _| callback())
        );
        nwg::bind_event_handler(&self.cancel_button.handle, clb);
    }
}