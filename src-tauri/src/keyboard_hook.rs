use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_CONTROL, VK_MENU, VK_SHIFT, VK_LWIN, VK_RWIN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetForegroundWindow, SetWindowPos, ShowWindow,
    SetWindowsHookExW, UnhookWindowsHookEx, HWND_NOTOPMOST, HWND_TOPMOST,
    KBDLLHOOKSTRUCT, MSG, SWP_NOMOVE, SWP_NOSIZE, SW_SHOW, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

const VK_C: u32 = 0x43;

struct HookState {
    app_handle: Option<AppHandle>,
    threshold_ms: u64,
    ctrl_session_active: bool,
    last_ctrl_c: Option<Instant>,
    c_is_down: bool,
}

static HOOK_STATE: Mutex<Option<HookState>> = Mutex::new(None);

fn show_and_focus_window(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        #[cfg(target_os = "windows")]
        {
            use std::ffi::c_void;
            if let Ok(hwnd) = window.hwnd() {
                let hwnd = windows::Win32::Foundation::HWND(hwnd.0 as *mut c_void);
                unsafe {
                    let _ = SetWindowPos(hwnd, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                    let _ = SetWindowPos(hwnd, Some(HWND_NOTOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                    let _ = ShowWindow(hwnd, SW_SHOW);
                    let _ = SetForegroundWindow(hwnd);
                }
            }
        }
    }
}

fn ctrl_is_pressed() -> bool {
    unsafe {
        (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0
    }
}

fn any_other_modifier_pressed() -> bool {
    unsafe {
        let shift = (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0;
        let alt = (GetAsyncKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0;
        let lwin = (GetAsyncKeyState(VK_LWIN.0 as i32) as u16 & 0x8000) != 0;
        let rwin = (GetAsyncKeyState(VK_RWIN.0 as i32) as u16 & 0x8000) != 0;
        shift || alt || lwin || rwin
    }
}

unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let is_down = wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize;
    let is_up = wparam.0 == WM_KEYUP as usize || wparam.0 == WM_SYSKEYUP as usize;
    let kbd = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
    let vk = kbd.vkCode;

    if let Ok(mut guard) = HOOK_STATE.lock() {
        if let Some(state) = guard.as_mut() {
            // ── Ctrl keydown ──
            if is_down && (vk == 0xA2 || vk == 0xA3 || vk == VK_CONTROL.0 as u32) {
                #[cfg(debug_assertions)] println!("[kbhook] Ctrl DOWN session_active={}", state.ctrl_session_active);
                state.ctrl_session_active = true;
            }

            // ── Ctrl keyup ──
            if is_up && (vk == 0xA2 || vk == 0xA3 || vk == VK_CONTROL.0 as u32) {
                #[cfg(debug_assertions)] println!("[kbhook] Ctrl UP → reset session");
                state.ctrl_session_active = false;
                state.last_ctrl_c = None;
                state.c_is_down = false;
            }

            // ── C keyup ──
            if is_up && vk == VK_C {
                #[cfg(debug_assertions)] println!("[kbhook] C UP");
                state.c_is_down = false;
            }

            // ── C keydown ──
            if is_down && vk == VK_C {
                // Guard: Ctrl must be physically held right now
                let actual_ctrl = ctrl_is_pressed();

                // Detect state desync: ctrl_session says active but Ctrl not actually held → reset
                if state.ctrl_session_active && !actual_ctrl {
                    state.ctrl_session_active = false;
                    state.last_ctrl_c = None;
                    state.c_is_down = false;
                }

                // All conditions must be met:
                //   - Ctrl currently held
                //   - Ctrl session is active (we saw Ctrl down)
                //   - No Shift / Alt / Win (pure Ctrl+C, not Ctrl+Shift+C etc.)
                //   - C was not already down (key repeat exclusion)
                let ctrl_pressed = actual_ctrl;
                let session = state.ctrl_session_active;
                let no_other = !any_other_modifier_pressed();
                let not_repeat = !state.c_is_down;
                let now = Instant::now();
                #[cfg(debug_assertions)] {
                    let elapsed_str = match state.last_ctrl_c {
                        Some(prev) => format!("{}ms", now.duration_since(prev).as_millis()),
                        None => "-".to_string(),
                    };
                    println!("[kbhook] C DOWN ctrl={} session={} no_other={} not_repeat={} last_c={} elapsed={}", ctrl_pressed, session, no_other, not_repeat, state.last_ctrl_c.is_some(), elapsed_str);
                }

                if ctrl_pressed && session && no_other {
                    #[cfg(debug_assertions)] if state.c_is_down { println!("[kbhook] C DOWN ignored repeat"); }
                    if !state.c_is_down {
                        state.c_is_down = true;

                        if let Some(prev) = state.last_ctrl_c {
                            let elapsed = now.duration_since(prev).as_millis();
                            if elapsed <= state.threshold_ms as u128 {
                                // Second C within threshold → trigger!
                                state.last_ctrl_c = None;
                                let handle = state.app_handle.clone();
                                let th = state.threshold_ms;
                                drop(guard);
                                if let Some(h) = handle {
                                    show_and_focus_window(&h);
                                    println!("[trigger-translate] source=keyboard_hook_double_copy threshold={}ms elapsed={}ms", th, elapsed);
                                    let _ = h.emit("trigger-translate", ());
                                }
                                #[cfg(debug_assertions)] println!("[kbhook] after trigger: last_ctrl_c=None");
                                return CallNextHookEx(None, code, wparam, lparam);
                            } else {
                                #[cfg(debug_assertions)] println!("[kbhook] C DOWN threshold expired elapsed={}ms -> reset first C", elapsed);
                            }
                        }
                        // First C in this session, or threshold expired
                        state.last_ctrl_c = Some(now);
                    }
                }
            }
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

pub fn start_global_hook(app_handle: AppHandle, threshold_ms: u32) {
    std::thread::spawn(move || {
        {
            let mut guard = HOOK_STATE.lock().unwrap();
            *guard = Some(HookState {
                app_handle: Some(app_handle),
                threshold_ms: threshold_ms as u64,
                ctrl_session_active: false,
                last_ctrl_c: None,
                c_is_down: false,
            });
        }

        let hook = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_hook_proc),
                None,
                0,
            ).ok()
        };
        if hook.is_none() {
            println!("[keyboard_hook] SetWindowsHookExW FAILED! double-copy will not work");
            return;
        }
        let hook = hook.unwrap();
        println!("[keyboard_hook] SetWindowsHookExW SUCCEEDED (hook=0x{:x})", hook.0 as usize);

        let mut msg = MSG::default();
        loop {
            let ret = unsafe { GetMessageW(&mut msg, None, 0, 0) };
            if ret.0 == 0 || ret.0 == -1 {
                break;
            }
        }

        unsafe {
            UnhookWindowsHookEx(hook).ok();
        }

        let mut guard = HOOK_STATE.lock().unwrap();
        *guard = None;
    });
}
