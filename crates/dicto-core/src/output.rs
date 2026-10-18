use enigo::{Direction, Enigo, Key, Settings};
use winapi::shared::minwindef::NULL;
use winapi::um::winbase::{GlobalAlloc, GlobalLock, GlobalUnlock};
use winapi::um::winnt::GMEM_MOVEABLE;
use winapi::um::winuser::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData, CF_UNICODETEXT,
};

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
    pub fn paste(&mut self, text: &str) -> Result<(), String> {
        // 1. Set clipboard via winapi (fast, reliable)
        self.set_clipboard(text)?;

        // 2. If auto-paste is on, simulate Ctrl+V
        if self.auto_paste {
            self.simulate_paste()?;
        }

        Ok(())
    }

    fn set_clipboard(&self, text: &str) -> Result<(), String> {
        unsafe {
            if OpenClipboard(NULL) == 0 {
                return Err("Failed to open clipboard".into());
            }
            if EmptyClipboard() == 0 {
                CloseClipboard();
                return Err("Failed to empty clipboard".into());
            }

            let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let size = wide.len() * 2;
            let h = GlobalAlloc(GMEM_MOVEABLE, size as usize);
            if h.is_null() {
                CloseClipboard();
                return Err("GlobalAlloc failed".into());
            }

            let locked = GlobalLock(h) as *mut u16;
            std::ptr::copy_nonoverlapping(wide.as_ptr(), locked, wide.len());
            GlobalUnlock(h);

            if SetClipboardData(CF_UNICODETEXT, h).is_null() {
                CloseClipboard();
                return Err("SetClipboardData failed".into());
            }

            CloseClipboard();
        }
        Ok(())
    }

    fn simulate_paste(&mut self) -> Result<(), String> {
        self.enigo.key(Key::Control, Direction::Press)
            .map_err(|e| format!("Key error: {}", e))?;
        self.enigo.key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| format!("Key error: {}", e))?;
        self.enigo.key(Key::Control, Direction::Release)
            .map_err(|e| format!("Key error: {}", e))?;
        Ok(())
    }
}