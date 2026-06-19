use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{MenuBuilder, MenuItemBuilder},
    Emitter, Manager,
};
use crate::commands::AppState;

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItemBuilder::with_id("show", "メインウィンドウを表示").build(app)?;
    let translate_item = MenuItemBuilder::with_id("translate", "翻訳 (Ctrl+Alt+C)").build(app)?;
    let history_item = MenuItemBuilder::with_id("history", "履歴").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "終了").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show_item)
        .item(&translate_item)
        .item(&history_item)
        .separator()
        .item(&quit_item)
        .build()?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("LLM Translator Desktop")
        .menu(&menu)
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "translate" => {
                    if let Some(window) = app.get_webview_window("main") {
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
                        let _ = window.emit("trigger-translate", ());
                    }
                }
                "history" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.emit("show-history", ());
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
