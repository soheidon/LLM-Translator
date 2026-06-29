mod commands;
mod config;
mod history;
mod keyboard_hook;
mod providers;
mod translator;
mod tray;

use commands::AppState;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::default().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            println!("[single-instance] second instance detected, activating existing window");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            // Load config
            let app_config = config::load_config();

            // Register state
            app.manage(AppState {
                config: Mutex::new(app_config),
            });

            // Setup system tray
            tray::setup_tray(app)?;

            // Apply always_on_top from config
            {
                let state = app.state::<AppState>();
                let always = state.config.lock().unwrap().general.always_on_top;
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_always_on_top(always);
                }
            }

            // Apply start_minimized from config (or hide on auto-start)
            {
                let state = app.state::<AppState>();
                let is_auto_start = std::env::args().any(|a| a == "--auto-start");
                let should_hide = is_auto_start || state.config.lock().unwrap().general.start_minimized;
                if should_hide {
                    if let Some(window) = app.get_webview_window("main") {
                        if let Err(e) = window.hide() {
                            eprintln!("[startup] failed to hide main window: {e}");
                        } else if is_auto_start {
                            println!("[startup] auto-start detected; main window hidden");
                        } else {
                            println!("[startup] start_minimized=true; main window hidden");
                        }
                    } else {
                        eprintln!("[startup] main window not found for start_minimized");
                    }
                }
            }

            // Register global shortcut
            let handle = app.handle().clone();
            let shortcut_config = {
                let state = app.state::<AppState>();
                let cfg = state.config.lock().unwrap();
                cfg.shortcut.clone()
            };

            use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
            let shortcut_str = shortcut_config.primary.clone();
            let shortcut = match shortcut_str.as_str() {
                "Ctrl+Alt+C" => Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyC),
                "Ctrl+Shift+C" => Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyC),
                "Ctrl+Alt+T" => Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyT),
                "Ctrl+Alt+H" => Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyH),
                _ => Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyC),
            };
            // Start global keyboard hook for double Ctrl+C detection (OS-wide, like DeepL)
            let shortcut_cfg = {
                let state = app.state::<AppState>();
                let cfg = state.config.lock().unwrap().shortcut.clone();
                cfg
            };
            if shortcut_cfg.double_copy_enabled {
                println!("[keyboard_hook] double_copy_enabled=true, threshold={}ms → starting global hook", shortcut_cfg.double_copy_threshold_ms);
                keyboard_hook::start_global_hook(handle.clone(), shortcut_cfg.double_copy_threshold_ms);
            } else {
                println!("[keyboard_hook] double_copy_enabled=false → hook not started (enable in settings and restart)");
            }

            let handle_clone = handle.clone();
            let _ = handle.global_shortcut().on_shortcut(
                shortcut,
                move |app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        if let Some(window) = handle_clone.get_webview_window("main") {
                            let focus = app
                                .state::<AppState>()
                                .config
                                .lock()
                                .map(|c| c.general.focus_on_translate)
                                .unwrap_or(true);
                            if focus {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                            println!("[trigger-translate] source=global_shortcut shortcut={:?}", shortcut);
                            let _ = window.emit("trigger-translate", ());
                        }
                    }
                },
            );

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::get_providers,
            commands::save_provider,
            commands::test_connection,
            commands::translate,
            commands::get_history,
            commands::delete_history,
            commands::clear_all_history,
            commands::get_modes,
            commands::get_languages,
            commands::check_env_var,
            commands::set_user_env_var,
            commands::list_ollama_models,
            commands::window_minimize,
            commands::window_maximize,
            commands::window_close,
            commands::focus_window,
            commands::focus_main_window,
            commands::set_always_on_top,
            commands::start_drag,
            commands::set_auto_launch,
            commands::open_google_translate,
            commands::set_google_translate_visible,
            commands::google_translate_back,
            commands::google_translate_forward,
            commands::google_translate_reload,
            commands::google_translate_home,
            commands::get_google_translate_url,
            commands::set_google_translate_text,
            commands::debug_google_translate_dom,
            commands::open_chatgpt_translate,
            commands::set_chatgpt_translate_visible,
            commands::chatgpt_translate_back,
            commands::chatgpt_translate_forward,
            commands::chatgpt_translate_reload,
            commands::chatgpt_translate_home,
            commands::get_chatgpt_translate_url,
            commands::set_chatgpt_translate_text,
            commands::set_chatgpt_translate_languages,
            commands::debug_chatgpt_translate_dom,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
