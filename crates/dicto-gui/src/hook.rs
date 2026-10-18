use std::sync::Mutex;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::um::winuser::{
    CallNextHookEx, DispatchMessageW, GetAsyncKeyState, GetMessageW, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN,
    WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

type HookFn = Box<dyn FnMut(bool, u32) + Send + 'static>;

static CALLBACK: Mutex<Option<HookFn>> = Mutex::new(None);

/// Low-level keyboard hook proc. Runs inside the message loop of the
/// thread that installed the hook. Signature fixed by Windows.
unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let msg = wparam as u32;
        if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN || msg == WM_KEYUP || msg == WM_SYSKEYUP {
            let kbd = lparam as *const KBDLLHOOKSTRUCT;
            let vk = (*kbd).vkCode as u32;
            let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
            if let Ok(mut guard) = CALLBACK.lock() {
                if let Some(cb) = guard.as_mut() {
                    cb(is_down, vk);
                }
            }
        }
    }
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

/// Install the low-level keyboard hook and run a blocking message loop on
/// the current thread. The callback fires for every key event with
/// `(is_down, vk_code)`. This gives us REAL push-to-talk (down + up).
pub fn run_keyboard_listener<F: FnMut(bool, u32) + Send + 'static>(
    callback: F,
) -> Result<(), String> {
    *CALLBACK.lock().unwrap() = Some(Box::new(callback));

    unsafe {
        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(hook_proc),
            std::ptr::null_mut(),
            0,
        );
        if hook.is_null() {
            return Err("Failed to install keyboard hook".into());
        }

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnhookWindowsHookEx(hook);
    }
    Ok(())
}

/// Check if a virtual key (e.g. VK_CONTROL, VK_SHIFT) is currently held.
pub fn is_modifier_down(vk: u32) -> bool {
    unsafe { GetAsyncKeyState(vk as i32) < 0 }
}

/// Common Windows virtual-key codes we support in config.
pub mod vk {
    pub const VK_LSHIFT: u32 = 0xA0;
    pub const VK_RSHIFT: u32 = 0xA1;
    pub const VK_LCONTROL: u32 = 0xA2;
    pub const VK_RCONTROL: u32 = 0xA3;
    pub const VK_LMENU: u32 = 0xA4;
    pub const VK_RMENU: u32 = 0xA5;
    pub const VK_LWIN: u32 = 0x5B;
    pub const VK_RWIN: u32 = 0x5C;
    pub const VK_CONTROL: u32 = 0x11;
    pub const VK_SHIFT: u32 = 0x10;
    pub const VK_MENU: u32 = 0x12;
    pub const VK_F1: u32 = 0x70;
    pub const VK_F24: u32 = 0x87;
    pub const VK_SPACE: u32 = 0x20;
    pub const VK_RETURN: u32 = 0x0D;
    pub const VK_TAB: u32 = 0x09;
    pub const VK_BACK: u32 = 0x08;
    pub const VK_DELETE: u32 = 0x2E;
    pub const VK_INSERT: u32 = 0x2D;
}

/// Parse a user-config key name ("F4", "VK_F4", "A", "1", "Space", ...)
/// into its virtual-key code. Returns None if unknown.
pub fn parse_key_name(name: &str) -> Option<u32> {
    let n = name.trim().to_uppercase();
    let n = n.strip_prefix("VK_").unwrap_or(&n);

    // F1..F24
    if n.starts_with('F') {
        if let Ok(num) = n[1..].parse::<u32>() {
            if (1..=24).contains(&num) {
                return Some(vk::VK_F1 + num - 1);
            }
        }
    }
    // A..Z
    let bytes = n.as_bytes();
    if bytes.len() == 1 {
        let c = bytes[0];
        if c.is_ascii_uppercase() {
            return Some(c as u32);
        }
        if c.is_ascii_digit() {
            return Some(c as u32);
        }
    }

    match n {
        "SPACE" => Some(vk::VK_SPACE),
        "RETURN" | "ENTER" => Some(vk::VK_RETURN),
        "TAB" => Some(vk::VK_TAB),
        "BACK" | "BACKSPACE" => Some(vk::VK_BACK),
        "DELETE" | "DEL" => Some(vk::VK_DELETE),
        "INSERT" | "INS" => Some(vk::VK_INSERT),
        "CONTROL" | "CTRL" => Some(vk::VK_CONTROL),
        "SHIFT" => Some(vk::VK_SHIFT),
        "MENU" | "ALT" => Some(vk::VK_MENU),
        "LWIN" | "WIN" => Some(vk::VK_LWIN),
        "RWIN" => Some(vk::VK_RWIN),
        _ => None,
    }
}

/// Map a VKey code (A, 0-9, F1-F24, named keys) to its display name.
pub fn vk_display(vk_code: u32) -> String {
    let c = char::from_u32(vk_code);
    if let Some(c) = c {
        if c.is_ascii_alphanumeric() {
            return c.to_ascii_uppercase().to_string();
        }
    }
    match vk_code {
        vk::VK_SPACE => "Space".into(),
        vk::VK_RETURN => "Return".into(),
        vk::VK_TAB => "Tab".into(),
        vk::VK_BACK => "Backspace".into(),
        vk::VK_DELETE => "Delete".into(),
        vk::VK_INSERT => "Insert".into(),
        v if v >= vk::VK_F1 && v <= vk::VK_F24 => format!("F{}", v - vk::VK_F1 + 1),
        _ => format!("0x{:X}", vk_code),
    }
}