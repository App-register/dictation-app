use enigo::{Enigo, Keyboard, Settings};

/// Output controller: paste text to active window.
pub struct OutputController {
    enigo: Enigo,
    auto_paste: bool,
}

impl OutputController {
    pub fn new(auto_paste: bool) -> Self {
        let enigo = Enigo::new(&Settings::default())
            .expect("Failed to init Enigo");
        Self { enigo, auto_paste }
    }

    /// Copy text to clipboard and paste into active window.
    /// Uses the Windows clipboard API directly (not Enigo's clipboard,
    /// which is unreliable) + Enigo for Ctrl+V.
    pub fn paste(&mut self, text: &str) -> Result<(), String> {
        // 1. Set clipboard via winapi (more reliable than enigo clipboard)
        self.set_clipboard(text)?;

        // 2. If auto-paste is on, simulate Ctrl+V
        if self.auto_paste {
            self.simulate_paste()?;
        }

        Ok(())
    }

    fn set_clipboard(&self, text: &str) -> Result<(), String> {
        // Use winapi directly for clipboard
        unsafe {
            use std::ptr::null_mut;
            use winapi::um::winuser::{OpenClipboard, EmptyClipboard, CloseClipboard,
                                       SetClipboardData, CF_UNICODETEXT};
            use winapi::um::winbase::GlobalAlloc, winapi::um::winnt::GMEM_MOVEABLE;
            use winapi::shared::ntdef::NULL;

            if OpenClipboard(null_mut()) == 0 {
                return Err("Failed to open clipboard".into());
            }
            if EmptyClipboard() == 0 {
                CloseClipboard();
                return Err("Failed to empty clipboard".into());
            }

            let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let size = wide.len() * 2;
            let h = GlobalAlloc(GMEM_MOVEABLE, size);
            if h == NULL {
                CloseClipboard();
                return Err("GlobalAlloc failed".into());
            }

            let locked = std::mem::transmute::<_, *mut u16>(h);
            std::ptr::copy_nonoverlapping(wide.as_ptr(), locked, wide.len());

            if SetClipboardData(CF_UNICODETEXT, h) == NULL {
                CloseClipboard();
                return Err("SetClipboardData failed".into());
            }

            CloseClipboard();
        }
        Ok(())
    }

    fn simulate_paste(&mut self) -> Result<(), String> {
        // Simulate Ctrl+V
        self.enigo.key(enigo::Key::Control, enigo::Direction::Click)
            .map_err(|e| format!("Key error: {}", e))?;
        self.enigo.key(enigo::Key::Unicode('v'), enigo::Direction::Click)
            .map_err(|e| format!("Key error: {}", e))?;
        self.enigo.key(enigo::Key::Control, enigo::Direction::Release)
            .map_err(|e| format!("Key error: {}", e))?;
        Ok(())
    }
}