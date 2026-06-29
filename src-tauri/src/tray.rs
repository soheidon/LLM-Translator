use std::sync::Mutex;
use std::time::Instant;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    Manager, WebviewWindow,
};

static LAST_TRAY_CLICK: Mutex<Option<Instant>> = Mutex::new(None);
static LAST_TRAY_ACTIVATE: Mutex<Option<Instant>> = Mutex::new(None);
static TRAY_ICON: Mutex<Option<TrayIcon>> = Mutex::new(None);
static MAIN_WINDOW: Mutex<Option<WebviewWindow>> = Mutex::new(None);

pub fn remember_main_window(window: WebviewWindow) {
    *MAIN_WINDOW.lock().unwrap() = Some(window);
    println!("[tray] remembered main window");
}

fn should_activate_tray() -> bool {
    let mut last = LAST_TRAY_ACTIVATE.lock().unwrap();
    if let Some(prev) = *last {
        if prev.elapsed().as_millis() < 700 {
            println!("[tray] activation skipped by debounce ({}ms since last)", prev.elapsed().as_millis());
            return false;
        }
    }
    *last = Some(Instant::now());
    true
}

fn restore_window(window: &tauri::WebviewWindow) {
    match window.show() {
        Ok(_) => println!("[tray] window.show ok"),
        Err(e) => println!("[tray] window.show error: {:?}", e),
    }
    match window.unminimize() {
        Ok(_) => println!("[tray] window.unminimize ok"),
        Err(e) => println!("[tray] window.unminimize error: {:?}", e),
    }
    match window.set_focus() {
        Ok(_) => println!("[tray] window.set_focus ok"),
        Err(e) => println!("[tray] window.set_focus error: {:?}", e),
    }
    #[cfg(target_os = "windows")]
    {
        use std::ffi::c_void;
        use windows::Win32::UI::WindowsAndMessaging::{
            SetForegroundWindow, SetWindowPos, ShowWindow, HWND_NOTOPMOST, HWND_TOPMOST,
            SWP_NOMOVE, SWP_NOSIZE, SW_RESTORE,
        };
        match window.hwnd() {
            Ok(hwnd) => {
                let hwnd = windows::Win32::Foundation::HWND(hwnd.0 as *mut c_void);
                println!("[tray] hwnd ok: {:?}", hwnd);
                unsafe {
                    let _ = SetWindowPos(hwnd, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                    let _ = SetWindowPos(hwnd, Some(HWND_NOTOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                    let fg_ok = SetForegroundWindow(hwnd).as_bool();
                    println!("[tray] SetForegroundWindow: {}", fg_ok);
                }
            }
            Err(e) => println!("[tray] window.hwnd error: {:?}", e),
        }
    }
}

fn show_and_focus_from_tray(app: &tauri::AppHandle) {
    println!("[tray] restoring main window");

    if let Some(window) = app.get_webview_window("main") {
        println!("[tray] found main via get_webview_window");
        restore_window(&window);
        return;
    }

    if let Some(window) = app.webview_windows().get("main").cloned() {
        println!("[tray] found main via webview_windows map");
        restore_window(&window);
        return;
    }

    let all_labels: Vec<String> = app.webview_windows().keys().map(|k| k.to_string()).collect();
    println!("[tray] available webview windows: {:?}", all_labels);

    let remembered = MAIN_WINDOW.lock().unwrap().as_ref().cloned();
    if let Some(window) = remembered {
        println!("[tray] found main via remembered window");
        restore_window(&window);
    } else {
        println!("[tray] main window not found (no manager entry, no remembered)");
    }
}

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let quit_item = MenuItemBuilder::with_id("quit", "終了").build(app)?;

    let menu = MenuBuilder::new(app).item(&quit_item).build()?;

    let tray = TrayIconBuilder::new()
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
                println!("[tray] DoubleClick Left detected");
                if should_activate_tray() {
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
                println!("[tray] Click Left Up detected");
                let mut last = LAST_TRAY_CLICK.lock().unwrap();
                if let Some(prev) = *last {
                    let elapsed = prev.elapsed().as_millis();
                    println!("[tray] elapsed since last click: {}ms", elapsed);
                    if elapsed < 500 {
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

    *TRAY_ICON.lock().unwrap() = Some(tray);

    println!("[tray] tray icon created successfully");
    Ok(())
}
