use std::cell::RefCell;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::um::winuser::{
    CallNextHookEx, DispatchMessageW, GetAsyncKeyState, GetMessageW, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN,
    WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

type HookFn = Box<dyn FnMut(bool, u32) + 'static>;

std::thread_local! {
    static CALLBACK: RefCell<Option<HookFn>> = const { RefCell::new(None) };
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let msg = wparam as u32;
        if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN || msg == WM_KEYUP || msg == WM_SYSKEYUP {
            let kbd = lparam as *const KBDLLHOOKSTRUCT;
            let vk = (*kbd).vkCode as u32;
            let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
            CALLBACK.with(|cb| {
                if let Some(f) = cb.borrow_mut().as_mut() {
                    f(is_down, vk);
                }
            });
        }
    }
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

/// Install a low-level keyboard hook and run a blocking message loop on
/// the CURRENT thread. The callback receives `(is_down, vk_code)`.
/// No `Send` requirement — everything must live on the calling thread.
pub fn run_keyboard_listener<F: FnMut(bool, u32) + 'static>(
    callback: F,
) -> Result<(), String> {
    CALLBACK.with(|cb| {
        *cb.borrow_mut() = Some(Box::new(callback));
    });

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

/// Check if a modifier key is currently held (e.g. VK_CONTROL).
pub fn is_modifier_down(vk: u32) -> bool {
    unsafe { GetAsyncKeyState(vk as i32) < 0 }
}

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

/// Parse a key name ("F4", "A", "Space", …) into its virtual-key code.
pub fn parse_key_name(name: &str) -> Option<u32> {
    let n = name.trim().to_uppercase();
    let n = n.strip_prefix("VK_").unwrap_or(&n);

    if n.starts_with('F') {
        if let Ok(num) = n[1..].parse::<u32>() {
            if (1..=24).contains(&num) {
                return Some(vk::VK_F1 + num - 1);
            }
        }
    }
    let bytes = n.as_bytes();
    if bytes.len() == 1 {
        let c = bytes[0];
        if c.is_ascii_uppercase() { return Some(c as u32); }
        if c.is_ascii_digit() { return Some(c as u32); }
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

/// Virtual-key code to human-readable name.
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