use enigo::{Direction, Enigo, Keyboard, Key, Settings};
use winapi::um::winbase::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_DDESHARE, GMEM_MOVEABLE};
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

    /// Copy text to clipboard, then simulate Ctrl+V.
    pub fn paste(&mut self, text: &str) -> Result<(), String> {
        self.set_clipboard(text)?;
        if self.auto_paste {
            self.simulate_paste()?;
        }
        Ok(())
    }

    fn set_clipboard(&self, text: &str) -> Result<(), String> {
        unsafe {
            for attempt in 0..3 {
                if OpenClipboard(std::ptr::null_mut()) == 0 {
                    // Clipboard busy — wait and retry
                    if attempt < 2 {
                        std::thread::sleep(std::time::Duration::from_millis(100 * (attempt + 1)));
                    }
                    continue;
                }
                let _ = EmptyClipboard();

                let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
                let size = wide.len() * 2;
                // GMEM_DDESHARE is required for clipboard allocations
                let h = GlobalAlloc(GMEM_MOVEABLE | GMEM_DDESHARE, size as usize);
                if h.is_null() {
                    CloseClipboard();
                    return Err("GlobalAlloc failed".into());
                }

                let locked = GlobalLock(h) as *mut u16;
                std::ptr::copy_nonoverlapping(wide.as_ptr(), locked, wide.len());
                GlobalUnlock(h);

                // SetClipboardData takes ownership of h — do NOT unlock afterward
                if SetClipboardData(CF_UNICODETEXT, h).is_null() {
                    CloseClipboard();
                    return Err("SetClipboardData failed".into());
                }

                CloseClipboard();
                return Ok(());
            }
        }
        Err("Could not open clipboard after 3 attempts".into())
    }

    fn simulate_paste(&mut self) -> Result<(), String> {
        self.enigo
            .key(Key::Control, Direction::Press)
            .and_then(|_| self.enigo.key(Key::Unicode('v'), Direction::Click))
            .and_then(|_| self.enigo.key(Key::Control, Direction::Release))
            .map_err(|e| format!("Paste failed: {}", e))?;
        Ok(())
    }
}