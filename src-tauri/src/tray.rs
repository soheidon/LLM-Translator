use std::sync::Mutex;
use std::time::Instant;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

static LAST_TRAY_CLICK: Mutex<Option<Instant>> = Mutex::new(None);
static LAST_TRAY_ACTIVATE: Mutex<Option<Instant>> = Mutex::new(None);

fn should_activate_tray() -> bool {
    let mut last = LAST_TRAY_ACTIVATE.lock().unwrap();
    if let Some(prev) = *last {
        if prev.elapsed().as_millis() < 700 {
            return false;
        }
    }
    *last = Some(Instant::now());
    true
}

fn show_and_focus_from_tray(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        #[cfg(target_os = "windows")]
        {
            use std::ffi::c_void;
            use windows::Win32::UI::WindowsAndMessaging::{
                SetForegroundWindow, SetWindowPos, ShowWindow, HWND_NOTOPMOST, HWND_TOPMOST,
                SWP_NOMOVE, SWP_NOSIZE, SW_SHOW,
            };
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

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let quit_item = MenuItemBuilder::with_id("quit", "終了").build(app)?;

    let menu = MenuBuilder::new(app).item(&quit_item).build()?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("LLM Translator Desktop")
        .menu(&menu)
        .on_menu_event(move |app, event| {
            if event.id().as_ref() == "quit" {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            println!("[tray] event: {:?}", event);

            // DoubleClick (works on most environments)
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                if should_activate_tray() {
                    println!("[tray] DoubleClick detected");
                    show_and_focus_from_tray(tray.app_handle());
                }
                return;
            }

            // Click fallback: manual double-click detection for environments
            // where DoubleClick doesn't fire. Only count Left + Up.
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let mut last = LAST_TRAY_CLICK.lock().unwrap();
                if let Some(prev) = *last {
                    if prev.elapsed().as_millis() < 500 {
                        *last = None;
                        drop(last);
                        if should_activate_tray() {
                            println!("[tray] manual double-click detected");
                            show_and_focus_from_tray(tray.app_handle());
                        }
                        return;
                    }
                }
                *last = Some(Instant::now());
            }
        })
        .build(app)?;

    Ok(())
}
