use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT, MOUSE_EVENT_FLAGS,
    VIRTUAL_KEY, VK_BACK, VK_RETURN, VK_TAB,
};

static LEFT_DOWN: AtomicBool = AtomicBool::new(false);
static RIGHT_DOWN: AtomicBool = AtomicBool::new(false);
static MIDDLE_DOWN: AtomicBool = AtomicBool::new(false);

fn button_state(name: &str) -> &'static AtomicBool {
    match name {
        "right" => &RIGHT_DOWN,
        "middle" => &MIDDLE_DOWN,
        _ => &LEFT_DOWN,
    }
}

pub fn mouse_move(dx: i32, dy: i32) {
    if dx == 0 && dy == 0 {
        return;
    }
    send_mouse(dx, dy, 0, MOUSEEVENTF_MOVE);
}

pub fn mouse_button(name: &str, action: &str) {
    let (down, up) = match name {
        "right" => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        "middle" => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
        _ => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
    };
    let state = button_state(name);
    match action {
        "down" => {
            state.store(true, Ordering::SeqCst);
            send_mouse(0, 0, 0, down);
        }
        "up" => {
            // Only emit an up event for a button we actually pressed. A lone
            // RIGHTUP after a disconnect can make Explorer/other apps open a
            // context menu even though no right press happened.
            if state.swap(false, Ordering::SeqCst) {
                send_mouse(0, 0, 0, up);
            }
        }
        _ => {
            state.store(false, Ordering::SeqCst);
            let inputs = [mouse_input(0, 0, 0, down), mouse_input(0, 0, 0, up)];
            unsafe {
                SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            }
        }
    }
}

/// `x` and `y` are wheel notches (1 notch = 120 wheel units).
pub fn wheel(x: i32, y: i32) {
    if y != 0 {
        send_mouse(0, 0, y.saturating_mul(120) as u32, MOUSEEVENTF_WHEEL);
    }
    if x != 0 {
        send_mouse(0, 0, x.saturating_mul(120) as u32, MOUSEEVENTF_HWHEEL);
    }
}

/// Locks the workstation by sending the Win+L hotkey, for environments where
/// `LockWorkStation` is refused.
pub fn lock_workstation_hotkey() {
    let inputs = [
        key_input(VIRTUAL_KEY(0x5B), 0, KEYBD_EVENT_FLAGS(0)), // VK_LWIN
        key_input(VIRTUAL_KEY(0x4C), 0, KEYBD_EVENT_FLAGS(0)), // L
        key_input(VIRTUAL_KEY(0x4C), 0, KEYEVENTF_KEYUP),
        key_input(VIRTUAL_KEY(0x5B), 0, KEYEVENTF_KEYUP),
    ];
    send(&inputs);
}

/// Tap the left/right arrow key in the focused window. Media players use these
/// for small seek jumps (YouTube, VLC, most browser players).
pub fn arrow_key(left: bool) {
    let vk = VIRTUAL_KEY(if left { 0x25 } else { 0x27 });
    send(&[
        key_input(vk, 0, KEYBD_EVENT_FLAGS(0)),
        key_input(vk, 0, KEYEVENTF_KEYUP),
    ]);
}

pub fn release_all() {
    let mut inputs = Vec::new();
    if LEFT_DOWN.swap(false, Ordering::SeqCst) {
        inputs.push(mouse_input(0, 0, 0, MOUSEEVENTF_LEFTUP));
    }
    if RIGHT_DOWN.swap(false, Ordering::SeqCst) {
        inputs.push(mouse_input(0, 0, 0, MOUSEEVENTF_RIGHTUP));
    }
    if MIDDLE_DOWN.swap(false, Ordering::SeqCst) {
        inputs.push(mouse_input(0, 0, 0, MOUSEEVENTF_MIDDLEUP));
    }
    if !inputs.is_empty() {
        unsafe {
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }
}

fn mouse_input(dx: i32, dy: i32, data: u32, flags: MOUSE_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send_mouse(dx: i32, dy: i32, data: u32, flags: MOUSE_EVENT_FLAGS) {
    let input = mouse_input(dx, dy, data, flags);
    unsafe {
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

fn key_input(vk: VIRTUAL_KEY, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

pub fn type_text(text: &str) {
    let mut inputs: Vec<INPUT> = Vec::new();
    for ch in text.chars() {
        match ch {
            '\n' | '\r' => {
                inputs.push(key_input(VK_RETURN, 0, KEYBD_EVENT_FLAGS(0)));
                inputs.push(key_input(VK_RETURN, 0, KEYEVENTF_KEYUP));
            }
            '\t' => {
                inputs.push(key_input(VK_TAB, 0, KEYBD_EVENT_FLAGS(0)));
                inputs.push(key_input(VK_TAB, 0, KEYEVENTF_KEYUP));
            }
            '\u{8}' => {
                inputs.push(key_input(VK_BACK, 0, KEYBD_EVENT_FLAGS(0)));
                inputs.push(key_input(VK_BACK, 0, KEYEVENTF_KEYUP));
            }
            _ => {
                let mut buf = [0u16; 2];
                for unit in ch.encode_utf16(&mut buf).iter() {
                    inputs.push(key_input(VIRTUAL_KEY(0), *unit, KEYEVENTF_UNICODE));
                    inputs.push(key_input(
                        VIRTUAL_KEY(0),
                        *unit,
                        KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                    ));
                }
            }
        }
        if inputs.len() >= 120 {
            send(&inputs);
            inputs.clear();
        }
    }
    if !inputs.is_empty() {
        send(&inputs);
    }
}

fn send(inputs: &[INPUT]) {
    unsafe {
        SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
    }
}
