use enigo::{Enigo, Keyboard, Settings};

/// Output controller: types text directly into the active window.
/// No clipboard involved — avoids Murmure interference and clipboard crashes.
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

    /// Type the text directly via simulated keystrokes (no clipboard).
    pub fn paste(&mut self, text: &str) -> Result<(), String> {
        if !self.auto_paste {
            return Ok(());
        }
        // Release any potentially stuck modifiers
        use enigo::{Direction, Key};
        let _ = self.enigo.key(Key::Control, Direction::Release);
        let _ = self.enigo.key(Key::Shift, Direction::Release);
        let _ = self.enigo.key(Key::Alt, Direction::Release);
        let _ = self.enigo.key(Key::Meta, Direction::Release);
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Type each character as keystrokes
        self.enigo
            .text(text)
            .map_err(|e| format!("Failed to type text: {}", e))?;
        Ok(())
    }
}