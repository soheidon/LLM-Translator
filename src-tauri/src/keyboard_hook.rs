use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetForegroundWindow, SetWindowPos, ShowWindow,
    SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, HWND_TOPMOST, HWND_NOTOPMOST,
    KBDLLHOOKSTRUCT, MSG, SWP_NOMOVE, SWP_NOSIZE, SW_SHOW, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_SYSKEYDOWN,
};

const VK_C: u32 = 0x43;

struct HookState {
    app_handle: AppHandle,
    threshold_ms: u32,
    last_ctrl_c: Option<Instant>,
}

static HOOK_STATE: Mutex<Option<HookState>> = Mutex::new(None);

fn show_and_focus_window(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        // Windows: force to foreground using SetWindowPos + SetForegroundWindow
        #[cfg(target_os = "windows")]
        {
            use std::ffi::c_void;
            if let Ok(hwnd) = window.hwnd() {
                let hwnd = windows::Win32::Foundation::HWND(hwnd.0 as *mut c_void);
                unsafe {
                    // Trick: temporarily set TOPMOST, then remove it
                    let _ = SetWindowPos(hwnd, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                    let _ = SetWindowPos(hwnd, Some(HWND_NOTOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                    let _ = ShowWindow(hwnd, SW_SHOW);
                    let _ = SetForegroundWindow(hwnd);
                }
            }
        }
    }
}

unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    if wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize {
        let kbd = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        let ctrl_held = (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;

        if ctrl_held && kbd.vkCode == VK_C {
            if let Ok(mut guard) = HOOK_STATE.lock() {
                if let Some(state) = guard.as_mut() {
                    let now = Instant::now();
                    let should_trigger = state
                        .last_ctrl_c
                        .map(|t| now.duration_since(t).as_millis() < state.threshold_ms as u128)
                        .unwrap_or(false);

                    if should_trigger {
                        state.last_ctrl_c = None;
                        let handle = state.app_handle.clone();
                        drop(guard);
                        show_and_focus_window(&handle);
                        let _ = handle.emit("trigger-translate", ());
                    } else {
                        state.last_ctrl_c = Some(now);
                    }
                }
            }
        } else {
            if let Ok(mut guard) = HOOK_STATE.lock() {
                if let Some(state) = guard.as_mut() {
                    state.last_ctrl_c = None;
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
                app_handle,
                threshold_ms,
                last_ctrl_c: None,
            });
        }

        let hook: HHOOK = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_hook_proc),
                None,
                0,
            )
        }
        .expect("SetWindowsHookExW failed");

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
