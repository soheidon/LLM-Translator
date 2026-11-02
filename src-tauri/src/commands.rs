use crate::config::{AppConfig, ProviderConfig};
use crate::history::{self, HistoryEntry};
use crate::providers::{ConnectionTestResult, TranslationRequest, TranslationResponse};
use crate::translator;
use std::sync::Mutex;
use tauri::{Manager, State};

static CHATGPT_LANGUAGE_BRIDGE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub struct AppState {
    pub config: Mutex<AppConfig>,
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
pub fn save_config(state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    crate::config::save_config(&config).map_err(|e| e.to_string())?;
    *state.config.lock().unwrap() = config;
    Ok(())
}

#[tauri::command]
pub fn get_providers(state: State<'_, AppState>) -> Vec<ProviderConfig> {
    state.config.lock().unwrap().providers.clone()
}

#[tauri::command]
pub fn save_provider(state: State<'_, AppState>, provider: ProviderConfig) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    if let Some(existing) = config.providers.iter_mut().find(|p| p.id == provider.id) {
        *existing = provider;
    } else {
        config.providers.push(provider);
    }
    crate::config::save_config(&config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn test_connection(provider: ProviderConfig) -> ConnectionTestResult {
    translator::test_provider_connection(&provider).await
}

#[tauri::command]
pub async fn translate(
    state: State<'_, AppState>,
    text: String,
    source_lang: Option<String>,
    target_lang: String,
    mode: String,
    tone: String,
    preset_id: Option<String>,
    provider_id: Option<String>,
    model: Option<String>,
    model_role: Option<String>,
    system_prompt: Option<String>,
) -> Result<TranslationResponse, String> {
    let config = state.config.lock().unwrap().clone();

    let (provider_config, provider) = if let Some(pid) = provider_id {
        translator::get_provider_by_id(&config, &pid)
            .ok_or_else(|| format!("Provider '{}' not found", pid))?
    } else {
        translator::get_default_provider(&config)
            .ok_or_else(|| "No default provider configured".to_string())?
    };

    let explicit_model = model.filter(|m| !m.trim().is_empty());

    let requested_role = match model_role.as_deref() {
        Some("fast") => "fast",
        _ => "default",
    };

    let selected_mapping = provider_config
        .model_mapping
        .get(requested_role)
        .filter(|role| !role.model.trim().is_empty())
        .or_else(|| {
            provider_config
                .model_mapping
                .get("default")
                .filter(|role| !role.model.trim().is_empty())
        });

    let model_id = explicit_model
        .clone()
        .or_else(|| selected_mapping.map(|role| role.model.clone()))
        .or_else(|| {
            if provider_config.model.trim().is_empty() {
                None
            } else {
                Some(provider_config.model.clone())
            }
        })
        .ok_or_else(|| "errors.no_model_configured".to_string())?;

    let model_mode = if explicit_model.is_some() {
        "normal".to_string()
    } else {
        selected_mapping
            .map(|role| match role.mode.trim() {
                "thinking" => "thinking".to_string(),
                _ => "normal".to_string(),
            })
            .unwrap_or_else(|| "normal".to_string())
    };

    #[cfg(debug_assertions)]
    println!(
        "[translate] provider={} role={} model={} mode={}",
        provider_config.id, requested_role, model_id, model_mode,
    );

    let request = TranslationRequest {
        text: text.clone(),
        source_lang: source_lang.clone(),
        target_lang: target_lang.clone(),
        mode: mode.clone(),
        tone: tone.clone(),
        preset_id: preset_id.clone(),
        provider: provider_config.name.clone(),
        model: model_id,
        model_mode,
        temperature: provider_config.temperature,
        max_tokens: provider_config.max_tokens,
        system_prompt,
    };

    let response = translator::translate_with_provider(provider.as_ref(), &provider_config, request)
        .await
        .map_err(|e| e.to_string())?;

    // Save to history if enabled
    if config.history.enabled {
        let entry = HistoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now(),
            source_text: text,
            translated_text: response.translated_text.clone(),
            source_lang: source_lang.unwrap_or_else(|| "auto".to_string()),
            target_lang,
            provider: response.provider.clone(),
            model: response.model.clone(),
            mode,
            tone,
            preset_id: preset_id.unwrap_or_default(),
            latency_ms: response.latency_ms,
        };
        let _ = history::save_history_entry(&entry);
    }

    Ok(response)
}

#[tauri::command]
pub fn get_history(
    _state: State<'_, AppState>,
    offset: usize,
    limit: usize,
    search: Option<String>,
) -> Vec<HistoryEntry> {
    let mut entries = history::load_history();
    entries.reverse(); // newest first

    if let Some(q) = search {
        let q_lower = q.to_lowercase();
        entries.retain(|e| {
            e.source_text.to_lowercase().contains(&q_lower)
                || e.translated_text.to_lowercase().contains(&q_lower)
        });
    }

    entries.into_iter().skip(offset).take(limit).collect()
}

#[tauri::command]
pub fn delete_history(id: String) -> Result<(), String> {
    history::delete_history_entry(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_all_history() -> Result<(), String> {
    history::clear_history().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_modes() -> Vec<ModeInfo> {
    vec![
        ModeInfo { id: "news".into() },
        ModeInfo { id: "academic".into() },
        ModeInfo { id: "technical".into() },
        ModeInfo { id: "email".into() },
        ModeInfo { id: "subtitle".into() },
        ModeInfo { id: "natural".into() },
        ModeInfo { id: "literal".into() },
        ModeInfo { id: "formal".into() },
        ModeInfo { id: "casual".into() },
        ModeInfo { id: "friendly".into() },
    ]
}

#[derive(serde::Serialize, Clone)]
pub struct ModeInfo {
    pub id: String,
}

#[tauri::command]
pub fn get_languages() -> Vec<LanguageInfo> {
    vec![
        LanguageInfo { code: "auto".into() },
        LanguageInfo { code: "en".into() },
        LanguageInfo { code: "ja".into() },
        LanguageInfo { code: "zh".into() },
        LanguageInfo { code: "ko".into() },
        LanguageInfo { code: "fr".into() },
        LanguageInfo { code: "de".into() },
        LanguageInfo { code: "es".into() },
        LanguageInfo { code: "pt".into() },
        LanguageInfo { code: "ru".into() },
        LanguageInfo { code: "it".into() },
    ]
}

#[tauri::command]
pub async fn window_minimize(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn window_maximize(window: tauri::Window) -> Result<(), String> {
    if window.is_maximized().unwrap_or(false) {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        window.maximize().map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn window_close(app: tauri::AppHandle, window: tauri::Window) -> Result<(), String> {
    println!("[window_close] hiding main window (label={})", window.label());
    let labels: Vec<String> = app.webview_windows().keys().map(|k| k.to_string()).collect();
    println!("[window_close] available webview windows before hide: {:?}", labels);
    window.hide().map_err(|e| e.to_string())?;
    let labels_after: Vec<String> = app.webview_windows().keys().map(|k| k.to_string()).collect();
    println!("[window_close] available webview windows after hide: {:?}", labels_after);
    Ok(())
}

#[tauri::command]
pub async fn focus_window(window: tauri::Window) -> Result<(), String> {
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    // Windows: force foreground with SetWindowPos + SetForegroundWindow
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
    Ok(())
}

#[tauri::command]
pub async fn focus_main_window(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let window = app.get_webview_window("main").ok_or("main window not found")?;
    window.show().map_err(|e| e.to_string())?;
    let _ = window.unminimize();
    window.set_focus().map_err(|e| e.to_string())?;
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
    Ok(())
}

#[tauri::command]
pub async fn set_always_on_top(window: tauri::Window, always_on_top: bool) -> Result<(), String> {
    window.set_always_on_top(always_on_top).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_drag(window: tauri::Window) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_auto_launch(enabled: bool) -> Result<(), String> {
    set_auto_launch_impl(enabled)
}

#[cfg(target_os = "windows")]
fn set_auto_launch_impl(enabled: bool) -> Result<(), String> {
    use std::io::ErrorKind;
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
            winreg::enums::KEY_SET_VALUE | winreg::enums::KEY_QUERY_VALUE,
        )
        .map_err(|e| format!("Failed to open Run key: {}", e))?;

    if enabled {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;
        let path_str = exe_path.to_string_lossy().to_string();
        // Quote the path to handle spaces, add --auto-start flag for detection
        let quoted = format!("\"{}\" --auto-start", path_str);
        run_key
            .set_value("LLMTranslator", &quoted)
            .map_err(|e| format!("Failed to set Run key: {}", e))?;
        println!("[auto_launch] enabled: {}", quoted);
    } else {
        match run_key.delete_value("LLMTranslator") {
            Ok(_) => println!("[auto_launch] disabled: Run key removed"),
            Err(e) if e.kind() == ErrorKind::NotFound => {
                // Key didn't exist — that's fine
                println!("[auto_launch] disabled: Run key did not exist");
            }
            Err(e) => return Err(format!("Failed to delete Run key: {}", e)),
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn set_auto_launch_impl(_enabled: bool) -> Result<(), String> {
    Err("Auto launch is only supported on Windows.".to_string())
}

#[tauri::command]
pub fn get_auto_launch_status() -> Result<bool, String> {
    get_auto_launch_status_impl()
}

#[cfg(target_os = "windows")]
fn get_auto_launch_status_impl() -> Result<bool, String> {
    use std::io::ErrorKind;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
            KEY_READ,
        )
        .map_err(|e| format!("Failed to open Run key: {}", e))?;

    match run_key.get_value::<String, _>("LLMTranslator") {
        Ok(value) => {
            let exe_path = std::env::current_exe()
                .map_err(|e| format!("Failed to get executable path: {}", e))?;
            let expected = format!("\"{}\" --auto-start", exe_path.to_string_lossy());
            Ok(value == expected)
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("Failed to read Run value: {}", e)),
    }
}

#[cfg(not(target_os = "windows"))]
fn get_auto_launch_status_impl() -> Result<bool, String> {
    Ok(false)
}

fn apply_google_translate_cleanup(w: &tauri::Webview) -> Result<(), String> {
    // Uses selectors verified against real Google Translate DOM (2026-06).
    // Hamburger: [aria-label="メインメニュー"] .gb_7c
    // Logo:      a[aria-label="Google 翻訳"].gb_je.gb_2c
    // Feedback:  [jsname="N7Eqid"] or text exact match
    // History:   a[jsname="cpynB"].mqNsCe (entire card)
    // Saved:     a[jsname="GxFwEe"].mqNsCe (entire card)
    // Apps icon: [aria-label="Google アプリ"] top-right
    // nav.U0xwnf, account icon, settings icon, input/result areas are NOT touched.
    let js = r#"
(function(){
  var S='llm-translator-google-cleanup-style',A='data-llm-tsl-hidden';
  if(!document.getElementById(S)){
    var st=document.createElement('style');
    st.id=S;
    st.textContent='['+A+'="true"]{display:none!important}';
    document.documentElement.appendChild(st);
  }
  function norm(s){return(s||'').replace(/\s+/g,' ').trim();}
  function rect(el){try{return el.getBoundingClientRect();}catch(e){return{top:9999,left:9999,width:0,height:0};}}
  function hide(el){if(!el||el.getAttribute(A)==='true')return;el.setAttribute(A,'true');}

  var hc=0;
  function apply(){
    hc=0;
    try{
      // 1. Hamburger — aria-label exact match, or gb_7c button at top-left
      document.querySelectorAll('[aria-label="メインメニュー"][role="button"], [aria-label="Main menu"][role="button"]').forEach(function(el){hc++;hide(el);});
      document.querySelectorAll('.gb_7c[role="button"]').forEach(function(el){
        var r=rect(el);
        if(r.top>=0&&r.top<80&&r.left>=0&&r.left<80){hc++;hide(el);}
      });

      // 2. Logo — a[aria-label="Google 翻訳"] or a.gb_je.gb_2c at top-left
      document.querySelectorAll('a[aria-label="Google 翻訳"], a[aria-label="Google Translate"]').forEach(function(el){
        var r=rect(el);
        if(r.top>=0&&r.top<80&&r.left>=0&&r.left<260){hc++;hide(el);}
      });
      document.querySelectorAll('a.gb_je.gb_2c').forEach(function(el){
        var r=rect(el);
        if(r.top>=0&&r.top<80&&r.left>=0&&r.left<260){hc++;hide(el);}
      });

      // 3. Feedback — jsname or exact text match
      document.querySelectorAll('[jsname="N7Eqid"]').forEach(function(el){hc++;hide(el);});
      document.querySelectorAll('a, button, [role="button"], [role="link"]').forEach(function(el){
        var t=norm(el.textContent);
        if(t==='フィードバックを送信'||t==='Send feedback'){hc++;hide(el);}
      });

      // 4. History — a[jsname="cpynB"] or a.mqNsCe with exact text (entire card, not child)
      document.querySelectorAll('a[jsname="cpynB"]').forEach(function(el){hc++;hide(el);});
      document.querySelectorAll('a.mqNsCe').forEach(function(el){
        var t=norm(el.textContent);
        if(t==='履歴'||t==='History'){hc++;hide(el);}
      });

      // 5. Saved — a[jsname="GxFwEe"] or a.mqNsCe with exact text (entire card, not child)
      document.querySelectorAll('a[jsname="GxFwEe"]').forEach(function(el){hc++;hide(el);});
      document.querySelectorAll('a.mqNsCe').forEach(function(el){
        var t=norm(el.textContent);
        if(t==='保存済み'||t==='Saved'){hc++;hide(el);}
      });

      // 6. Google apps grid icon (9-dot icon) — top-right corner, not the account/settings icons
      document.querySelectorAll('[aria-label="Google アプリ"], [aria-label="Google apps"], [aria-label="Google アプリ一覧"]').forEach(function(el){hc++;hide(el);});
      document.querySelectorAll('[role="button"], a, button, div').forEach(function(el){
        var label=el.getAttribute('aria-label')||'';
        var r=rect(el);
        if(r.top>=0&&r.top<80&&r.left>window.innerWidth-160&&/Google\s*アプリ|Google apps|アプリ一覧/i.test(label)){hc++;hide(el);}
      });

    }catch(e){console.warn('[LLM Translator] cleanup:',e);}
    console.debug('[LLM Translator Desktop] Google cleanup hidden:',hc);
  }

  apply();

  if(!window.__LLM_TRANSLATOR_GOOGLE_CLEANUP_OBSERVER__){
    window.__LLM_TRANSLATOR_GOOGLE_CLEANUP_OBSERVER__=true;
    var timer=null;
    var obs=new MutationObserver(function(){
      if(timer)return;
      timer=setTimeout(function(){timer=null;apply();},200);
    });
    obs.observe(document.documentElement,{childList:true,subtree:true,characterData:true});
  }
})()"#;
    w.eval(js).map_err(|e| format!("failed to apply google translate cleanup: {e}"))
}

fn schedule_google_translate_cleanup(app: tauri::AppHandle) {
    use tauri::Manager;
    tauri::async_runtime::spawn(async move {
        let delays = [0_u64, 500, 1500, 3000];
        for delay in delays {
            if delay > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }
            if let Some(w) = app.get_webview("google-translate") {
                let _ = apply_google_translate_cleanup(&w);
            }
        }
    });
}

#[tauri::command]
pub async fn open_google_translate(app: tauri::AppHandle, url: String, x: f64, y: f64, width: f64, height: f64) -> Result<(), String> {
    use tauri::{Manager, Position, Size, Url, WebviewBuilder, WebviewUrl};
    let label = "google-translate";
    let parsed_url: Url = url.parse().map_err(|e| format!("invalid url: {}", e))?;
    if let Some(w) = app.get_webview(label) {
        let _ = w.navigate(parsed_url);
        let _ = w.set_position(Position::Logical(tauri::LogicalPosition { x, y }));
        let _ = w.set_size(Size::Logical(tauri::LogicalSize { width, height }));
        let _ = w.show();
        schedule_google_translate_cleanup(app.clone());
        return Ok(());
    }
    let main_webview = app.get_webview("main").ok_or("main webview not found")?;
    let main_window = main_webview.window();

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("llm-translator")
        .join("google-translate-webview");

    let webview_builder = WebviewBuilder::new(label, WebviewUrl::External(parsed_url))
        .transparent(true)
        .auto_resize()
        .data_directory(data_dir);

    main_window
        .add_child(
            webview_builder,
            Position::Logical(tauri::LogicalPosition { x, y }),
            Size::Logical(tauri::LogicalSize { width, height }),
        )
        .map_err(|e| e.to_string())?;

    schedule_google_translate_cleanup(app.clone());

    Ok(())
}

#[tauri::command]
pub async fn set_google_translate_visible(
    app: tauri::AppHandle,
    visible: bool,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    use tauri::Manager;
    let label = "google-translate";
    let w = app.get_webview(label).ok_or("webview not found")?;
    if visible {
        let _ = w.set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
        let _ = w.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }));
        let _ = w.show();
    } else {
        let _ = w.hide();
    }
    Ok(())
}

#[tauri::command]
pub async fn google_translate_back(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let w = app.get_webview("google-translate").ok_or("webview not found")?;
    w.eval("window.history.back()").map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn google_translate_forward(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let w = app.get_webview("google-translate").ok_or("webview not found")?;
    w.eval("window.history.forward()").map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn google_translate_reload(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let w = app.get_webview("google-translate").ok_or("webview not found")?;
    w.eval("window.location.reload()").map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn google_translate_home(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri::Manager;
    let w = app.get_webview("google-translate").ok_or("webview not found")?;
    let parsed: tauri::Url = url.parse().map_err(|e| format!("invalid url: {}", e))?;
    w.navigate(parsed).map_err(|e| e.to_string())?;
    schedule_google_translate_cleanup(app);
    Ok(())
}

#[tauri::command]
pub async fn get_google_translate_url(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;
    let w = app.get_webview("google-translate").ok_or("webview not found")?;
    w.url().map(|u| u.to_string()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn debug_google_translate_dom(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;
    let w = app.get_webview("google-translate").ok_or("webview not found")?;
    let js = r#"
(function(){
  function norm(s){return(s||'').replace(/\s+/g,' ').trim();}
  function info(el){var r=el.getBoundingClientRect();return{tag:el.tagName,text:norm(el.textContent).slice(0,80),ariaLabel:el.getAttribute('aria-label')||'',role:el.getAttribute('role')||'',jsname:el.getAttribute('jsname')||'',className:typeof el.className==='string'?el.className:'',top:Math.round(r.top),left:Math.round(r.left),width:Math.round(r.width),height:Math.round(r.height)};}
  var topCandidates=[].slice.call(document.querySelectorAll('button,a,[role="button"],[role="link"],div,span')).filter(function(el){var r=el.getBoundingClientRect();return r.top>=0&&r.top<140&&r.left>=0&&r.left<260;}).map(info);
  var textCandidates=[].slice.call(document.querySelectorAll('button,a,[role="button"],[role="link"],div,span')).filter(function(el){var t=norm(el.textContent);var a=el.getAttribute('aria-label')||'';return /Google|翻訳|フィードバック|履歴|保存済み|History|Saved|feedback/i.test(t+' '+a);}).map(info);
  var result={location:location.href,topCandidates:topCandidates,textCandidates:textCandidates};
  var json=JSON.stringify(result,null,2);
  console.log('[LLM Translator DEBUG]\n'+json);
  var prev=document.getElementById('__llm_tsl_debug');
  if(prev) prev.remove();
  var box=document.createElement('div');
  box.id='__llm_tsl_debug';
  box.style.cssText='position:fixed;top:0;left:0;right:0;z-index:999999;background:#1e1e1e;color:#d4d4d4;font:12px monospace;border-bottom:2px solid #6ea8fe;padding:8px;max-height:60vh;display:flex;flex-direction:column;';
  var hdr=document.createElement('div');
  hdr.style.cssText='display:flex;align-items:center;justify-content:space-between;margin-bottom:4px;';
  var lbl=document.createElement('span');
  lbl.textContent='LLM Translator DOM診断 (Ctrl+A → Ctrl+C でコピー)';
  lbl.style.cssText='color:#6ea8fe;font-weight:bold;';
  var close=document.createElement('button');
  close.textContent='閉じる';
  close.style.cssText='background:#444;color:#fff;border:none;padding:2px 10px;border-radius:3px;cursor:pointer;font-size:11px;';
  close.onclick=function(){box.remove();};
  hdr.appendChild(lbl);hdr.appendChild(close);box.appendChild(hdr);
  var ta=document.createElement('textarea');
  ta.value=json;ta.readOnly=true;
  ta.style.cssText='width:100%;height:300px;background:#252526;color:#d4d4d4;border:1px solid #555;border-radius:4px;font:11px monospace;resize:vertical;padding:4px;';
  ta.onfocus=function(){ta.select();};
  box.appendChild(ta);
  document.body.insertBefore(box,document.body.firstChild);
  setTimeout(function(){ta.focus();ta.select();},100);
})()"#;
    w.eval(js).map_err(|e| format!("eval failed: {e}"))?;
    Ok("diagnostic rendered in Google Translate page".into())
}

#[tauri::command]
pub async fn set_google_translate_text(app: tauri::AppHandle, text: String) -> Result<(), String> {
    use tauri::Manager;
    println!("[Ctrl+C+C] set_google_translate_text, len={}", text.len());
    let w = app.get_webview("google-translate").ok_or("webview not found")?;
    let text_json = serde_json::to_string(&text).map_err(|e| e.to_string())?;
    // JS below depends on Google Translate's DOM structure.
    // If injection stops working after a Google Translate UI update,
    // the selectors in `f()` (textarea / contenteditable / role=textbox)
    // or the event dispatch pattern in `s()` may need adjustment.
    let js = format!(r#"(function(){{var t={text_json};function s(e,v){{var d=Object.getOwnPropertyDescriptor(Object.getPrototypeOf(e),'value');if(d&&d.set)d.set.call(e,v);else e.value=v;e.dispatchEvent(new Event('input',{{bubbles:true}}));e.dispatchEvent(new Event('change',{{bubbles:true}}));}}function f(){{return document.querySelector('textarea')||document.querySelector('[contenteditable="true"]')||document.querySelector('[role="textbox"]');}}function trySet(){{var el=f();if(!el)return false;el.focus();if(el.tagName==='TEXTAREA'||el.tagName==='INPUT'){{s(el,t);}}else{{el.textContent=t;el.dispatchEvent(new InputEvent('input',{{bubbles:true,inputType:'insertText',data:t}}));}}return true;}}if(trySet())return;var n=0;var iv=setInterval(function(){{n++;if(trySet()||n>=40)clearInterval(iv);}},250);}})();"#);
    w.eval(&js).map_err(|e| e.to_string())
}

// ── ChatGPT Translate ──────────────────────────────────────────────

const CHATGPT_TRANSLATE_LABEL: &str = "chatgpt-translate";

fn apply_chatgpt_translate_cleanup(w: &tauri::Webview, hide_lp: bool) -> Result<(), String> {
    let hide_lp_val = if hide_lp { "true" } else { "false" };
    let js = "(function () {\n  var hideLpElements = ".to_string() + hide_lp_val + ";\n" + r#"
  const STYLE_ID = 'llm-translator-chatgpt-cleanup-style';
  const HIDDEN_ATTR = 'data-llm-chatgpt-hidden';

  if (!document.getElementById(STYLE_ID)) {
    const style = document.createElement('style');
    style.id = STYLE_ID;
    style.textContent = `
      [data-llm-chatgpt-hidden="true"] {
        display: none !important;
      }
      html,
      body {
        height: 100vh !important;
        max-height: 100vh !important;
        overflow: hidden !important;
      }

      /* Collapse sidebar grid column and zero marketing/sidebar spacing variables */
      :root,
      body {
        --x13mh34a: 0px !important;
        --mkt-header-height: 0px !important;
      }

      /* Page chrome: app sidebar, sidebar headers, marketing headers, and footers */
      #stage-slideover-sidebar,
      #sidebar-header,
      #contentful-header,
      body > div aside,
      aside:not([data-testid*="translate"]),
      aside header,
      body > div > div > aside,
      body > div footer,
      footer:not([data-testid*="translate"]),
      [data-llm-chatgpt-outer="true"] > header,
      [data-llm-chatgpt-outer="true"] > footer,
      [data-llm-chatgpt-outer="true"] ~ footer,
      [data-llm-chatgpt-container="true"] > header,
      [data-llm-chatgpt-container="true"] h1,
      [data-testid="signup-button"],
      [data-testid="login-button"],
      .prompt-card,
      a.prompt-card,
      [data-llm-translator-hidden-suggestion="true"] {
        display: none !important;
      }

      /* Collapse sidebar column in grid container */
      [class*="x67ebes"],
      div:has(> aside) {
        grid-template-columns: 0px minmax(0, 1fr) !important;
      }

      /* Unconstrain ancestor wrappers above outer (Tailwind/StyleX max-w-* / mx-auto) */
      [data-llm-chatgpt-width-wrapper="true"] {
        width: 100% !important;
        max-width: none !important;
        margin-left: 0 !important;
        margin-right: 0 !important;
        overflow: visible !important;
      }

      /* Outer wrapper: fill full available width and height without right overflow */
      [data-llm-chatgpt-outer="true"] {
        padding: 0 !important;
        margin: 0 !important;
        width: 100% !important;
        max-width: 100% !important;
        min-height: 100vh !important;
        height: 100vh !important;
        max-height: 100vh !important;
        display: flex !important;
        flex-direction: column !important;
        overflow: visible !important;
        box-sizing: border-box !important;
      }

      /* Container (main): center and fit translation UI within available area */
      main[data-llm-chatgpt-container="true"],
      [data-llm-chatgpt-container="true"] {
        min-height: 100vh !important;
        height: 100vh !important;
        max-height: 100vh !important;
        width: 100% !important;
        max-width: none !important;
        box-sizing: border-box !important;
        display: flex !important;
        flex-direction: column !important;
        overflow: visible !important;
        padding: 12px 20px !important;
        margin: 0 !important;
        flex: 1 1 auto !important;
        gap: 12px !important;
      }

      main[data-llm-chatgpt-container="true"] > div,
      main[data-llm-chatgpt-container="true"] > section,
      [data-llm-chatgpt-container="true"] > div,
      [data-llm-chatgpt-container="true"] > section {
        flex: 1 1 auto !important;
        min-height: 0 !important;
        width: 100% !important;
        max-width: none !important;
        display: flex !important;
        flex-direction: column !important;
        padding-top: 0 !important;
        margin-top: 0 !important;
        overflow: visible !important;
      }

      main[data-llm-chatgpt-container="true"] > header,
      [data-llm-chatgpt-container="true"] > header,
      [data-llm-chatgpt-container="true"] h1 {
        display: none !important;
        margin: 0 !important;
        padding: 0 !important;
        height: 0 !important;
        min-height: 0 !important;
      }

      [data-llm-chatgpt-form="true"] {
        width: 100% !important;
        max-width: none !important;
        flex: 1 1 auto !important;
        min-height: 0 !important;
        display: flex !important;
        flex-direction: column !important;
        gap: 12px !important;
        margin-top: 0 !important;
        padding-top: 0 !important;
        overflow: visible !important;
      }

      /* Language selection row: keep visible above cards */
      [data-llm-chatgpt-language-row="true"] {
        width: 100% !important;
        flex: 0 0 auto !important;
        overflow: visible !important;
        position: relative !important;
        z-index: 100 !important;
      }

      /* Translation cards row: side-by-side two cards layout */
      [data-llm-chatgpt-translate-row="true"] {
        flex: 1 1 auto !important;
        min-height: 0 !important;
        display: grid !important;
        grid-template-columns: repeat(2, minmax(0, 1fr)) !important;
        gap: 16px !important;
        width: 100% !important;
        box-sizing: border-box !important;
        overflow: visible !important;
      }

      [data-llm-chatgpt-column="true"] {
        min-width: 0 !important;
        min-height: 0 !important;
        height: 100% !important;
        display: flex !important;
        flex-direction: column !important;
        box-sizing: border-box !important;
      }

      [data-llm-chatgpt-result-wrapper="true"] {
        flex: 1 1 auto !important;
        min-height: 0 !important;
        display: flex !important;
        width: 100% !important;
      }

      [data-llm-chatgpt-source-textarea="true"],
      [data-llm-chatgpt-target-textarea="true"],
      [data-llm-chatgpt-column="true"] textarea {
        flex: 1 1 auto !important;
        width: 100% !important;
        height: 100% !important;
        min-height: 0 !important;
        max-height: none !important;
        box-sizing: border-box !important;
        resize: none !important;
        overflow-y: auto !important;
      }

      /* Language selection popovers: ensure proper stacking without altering native fixed coordinates */
      #translate-source-language-popover,
      #translate-target-language-popover {
        z-index: 99999 !important;
      }
        `;
    document.documentElement.appendChild(style);
  }

  function norm(s) {
    return (s || '').replace(/\s+/g, ' ').trim();
  }

  function hide(el) {
    if (!el) return;
    if (el.getAttribute(HIDDEN_ATTR) === 'true') return;
    el.setAttribute(HIDDEN_ATTR, 'true');
  }

  function closestCommonAncestor(a, b) {
    var aAncestors = [];
    var cur = a;
    while (cur) {
      aAncestors.push(cur);
      cur = cur.parentElement;
    }
    cur = b;
    while (cur) {
      if (aAncestors.indexOf(cur) >= 0) return cur;
      cur = cur.parentElement;
    }
    return null;
  }

  function countTextareas(el) {
    return el ? el.querySelectorAll('textarea').length : 0;
  }

  function findTranslateRow(source, target) {
    var common = closestCommonAncestor(source, target);
    if (!common) return null;
    // Walk down from common to find the smallest child containing both textareas
    var row = common;
    var changed = true;
    while (changed) {
      changed = false;
      var children = Array.from(row.children);
      for (var i = 0; i < children.length; i++) {
        if (countTextareas(children[i]) >= 2) {
          row = children[i];
          changed = true;
          break;
        }
      }
    }
    return row;
  }

  function findColumnForTextarea(row, textarea) {
    if (!row || !textarea) return null;
    // Check direct children of row first
    var children = Array.from(row.children);
    for (var i = 0; i < children.length; i++) {
      if (children[i].contains(textarea)) {
        return children[i];
      }
    }
    // Fallback: walk up from textarea until we hit a child of row
    var cur = textarea.parentElement;
    while (cur && cur !== row) {
      var parent = cur.parentElement;
      if (parent === row) return cur;
      cur = parent;
    }
    return null;
  }

  function markLanguageRow() {
    function getLanguageCandidates() {
      var combos = Array.from(document.querySelectorAll('button[role="combobox"]'))
        .filter(function(button) {
          return (button.textContent || '').replace(/\s+/g, ' ').trim().length > 0;
        });
      if (combos.length === 2) return combos;

      var root =
        document.querySelector('[data-llm-chatgpt-form="true"]') ||
        document.querySelector('[data-llm-chatgpt-container="true"]') ||
        document;
      var fallback = Array.from(root.querySelectorAll('button.interactive-button.interactive-button-secondary'))
        .filter(function(b) {
          var r = b.getBoundingClientRect();
          var s = window.getComputedStyle(b);
          if (!(r.width > 0 && r.height > 0 && s.display !== 'none' && s.visibility !== 'hidden')) return false;
          var span = b.querySelector('span.truncate');
          return !!(span && (span.textContent || '').replace(/\s+/g, ' ').trim().length > 0);
        })
        .sort(function(a, b) { return a.getBoundingClientRect().left - b.getBoundingClientRect().left; });
      return fallback;
    }

    var buttons = getLanguageCandidates();
    if (buttons.length !== 2) return;

    var row = closestCommonAncestor(buttons[0], buttons[1]);
    if (!row) return;

    row.setAttribute('data-llm-chatgpt-language-row', 'true');
  }

  function markTranslateLayout() {
    // Clear stale width-wrapper attributes from previous DOM structure
    document.querySelectorAll('[data-llm-chatgpt-width-wrapper="true"]').forEach(function(el) {
      el.removeAttribute('data-llm-chatgpt-width-wrapper');
    });

    var textareas = Array.from(document.querySelectorAll('textarea'));
    if (textareas.length < 2) return;

    var source = textareas.find(function(t) { return !t.hasAttribute('readonly'); });
    var target = textareas.find(function(t) { return t.hasAttribute('readonly'); });

    if (!source || !target) return;

    source.setAttribute('data-llm-chatgpt-source-textarea', 'true');
    target.setAttribute('data-llm-chatgpt-target-textarea', 'true');

    var row = findTranslateRow(source, target);
    if (!row) return;

    row.setAttribute('data-llm-chatgpt-translate-row', 'true');

    var sourceCol = findColumnForTextarea(row, source);
    var targetCol = findColumnForTextarea(row, target);

    if (sourceCol) sourceCol.setAttribute('data-llm-chatgpt-column', 'true');
    if (targetCol) targetCol.setAttribute('data-llm-chatgpt-column', 'true');

    var targetRelative = target.closest('.relative') || target.parentElement;
    if (targetRelative) {
      targetRelative.setAttribute('data-llm-chatgpt-result-wrapper', 'true');
    }

    var form = row.parentElement;
    if (form) {
      form.setAttribute('data-llm-chatgpt-form', 'true');
    }

    var container = form ? form.parentElement : null;
    if (container) {
      if (container.getAttribute('data-llm-chatgpt-container') !== 'true') {
        container.setAttribute('data-llm-chatgpt-container', 'true');
      }

      // Mark outer wrapper (parent of container) to remove Tailwind padding/max-width
      var outer = container.parentElement;
      if (outer && outer.getAttribute('data-llm-chatgpt-outer') !== 'true') {
        outer.setAttribute('data-llm-chatgpt-outer', 'true');
      }

      // Walk up to 3 ancestor levels above outer to unconstrain Tailwind max-w-* / mx-auto
      if (outer) {
        var ancestor = outer.parentElement;
        for (var i = 0; ancestor && i < 3; i += 1) {
          if (
            ancestor !== document.body &&
            ancestor !== document.documentElement &&
            ancestor.contains(container)
          ) {
            ancestor.setAttribute('data-llm-chatgpt-width-wrapper', 'true');
          }
          ancestor = ancestor.parentElement;
        }
      }
    }

  }

  function markLoginHeader() {
    const loginButton = document.querySelector('[data-testid="login-button"]');
    if (!loginButton) return;

    let cur = loginButton.parentElement;

    while (cur && cur !== document.body) {
      const className = (cur.className || '').toString();

      if (
        className.includes('h-header-height') &&
        className.includes('sticky') &&
        className.includes('top-0')
      ) {
        cur.setAttribute('data-llm-chatgpt-login-header', 'true');
        return;
      }

      cur = cur.parentElement;
    }
  }

  function markLoginBlock() {
    const loginButton = document.querySelector('[data-testid="login-button"]');
    if (!loginButton) return;

    let cur = loginButton.parentElement;

    while (cur && cur !== document.body) {
      const className = (cur.className || '').toString();

      if (
        className.includes('flex') &&
        className.includes('items-center') &&
        className.includes('justify-end') &&
        className.includes('gap-2') &&
        className.includes('overflow-y-visible')
      ) {
        cur.setAttribute('data-llm-chatgpt-login-block', 'true');
        return;
      }

      cur = cur.parentElement;
    }
  }

  var debugCleanup = false;
  function cleanupLog(msg) {
    if (debugCleanup) console.log('[LLM Translator Desktop] ' + msg);
  }

  function applyCleanup() {
    try {
      cleanupLog('applyCleanup start');
      cleanupLog('cleanup: hide LP elements = ' + hideLpElements);
      // 1. Left sidebar — exact ID match only
      document.querySelectorAll('#stage-slideover-sidebar').forEach(hide);

      // 1d. Hide ChatGPT app shell sidebar (aside/nav with sidebar-like content)
      cleanupLog('app shell sidebar cleanup step');
      document.querySelectorAll('aside, [id*="sidebar"], [class*="sidebar"], nav').forEach(function(el) {
        var text = norm(el.textContent || el.innerText || '');

        var looksLikeChatGptSidebar =
          text.includes('新しいチャット') ||
          text.includes('Deep research') ||
          text.includes('プランと料金を見る') ||
          text.includes('自分に合った回答を得る');

        if (!looksLikeChatGptSidebar) return;
        if (hasTranslationUi(el)) return;

        hide(el);
      });

      // Slim down contentful-header to login-only bar (variant A)
      var contentfulHeader = document.getElementById('contentful-header');
      if (contentfulHeader) {
        contentfulHeader.removeAttribute('data-llm-chatgpt-hidden');

        contentfulHeader.querySelectorAll('nav, ul, li').forEach(hide);

        var firstChild = contentfulHeader.firstElementChild;
        if (firstChild) hide(firstChild);

        contentfulHeader.querySelectorAll('[data-testid="signup-button"]').forEach(hide);

        contentfulHeader.querySelectorAll('[data-testid="login-button"]').forEach(function(btn) {
          btn.style.display = 'inline-flex';
          btn.style.visibility = 'visible';
          btn.style.opacity = '1';
        });
      }

      // 1b. Header navigation (structure-based: ul/nav inside header, not the entire header)
      if (hideLpElements) {
      document.querySelectorAll('header ul, header nav').forEach(hide);
      }

      // 1c. Fallback: hide nav links by text (for environments without <header> tag)
      if (hideLpElements) {
      var navTexts = ['概要', '機能', '学ぶ', 'Codex', 'ビジネス', '料金', 'ダウンロード', '今すぐ試す'];
      document.querySelectorAll('a, button').forEach(function(el) {
        if (navTexts.indexOf(norm(el.textContent)) !== -1) {
          hide(el);
        }
      });
      }

      // 2. Login area: collapse header, float login block, hide signup
      markLoginHeader();
      markLoginBlock();
      document.querySelectorAll('[data-testid="signup-button"]').forEach(hide);

      // 2a. Hide signup buttons/links by text (app shell variant), preserve login buttons
      cleanupLog('signup cleanup step');
      document.querySelectorAll('button, a').forEach(function(el) {
        var text = norm(el.textContent || el.innerText || '');

        var isSignup =
          text === '無料でサインアップ' ||
          text === 'Sign up for free' ||
          text === 'Sign Up' ||
          text === 'Signup' ||
          text === '無料登録' ||
          text === '新規登録' ||
          text === '登録する' ||
          text === 'アカウント作成' ||
          (text.includes('サインアップ') && !text.includes('ログイン')) ||
          (text.includes('Sign up') && !text.includes('Log in'));

        if (!isSignup) return;

        // Preserve: element is dominated by login content
        var dominatedByLogin =
          (norm(el.textContent || '').indexOf('ログイン') !== -1 ||
           norm(el.textContent || '').indexOf('Log in') !== -1) &&
          !norm(el.textContent || '').match(/(無料で|無料|Sign up|サインアップ|アカウント作成|会員登録)/);

        if (dominatedByLogin) return;
        if (hasTranslationUi(el)) return;

        hide(el);
      });

      // 3. Bottom suggestion cards — hide only the matched interactive elements themselves
      function normStrict(s) { return (s||'').replace(/\s+/g,'').trim(); }
      const suggestionTexts = [
        'より自然な表現に',
        '自然でなめらかな表現にします。',
        'ビジネス用にする',
        '洗練されたビジネス向けのトーンにします。',
        '5 歳児にもわかるように説明して',
        '5歳児にもわかるように説明して',
        'とてもやさしい言葉で書き直します。'
      ];
      const strictTexts = suggestionTexts.map(normStrict);

      var hiddenCount = 0;

      document.querySelectorAll('button, [role="button"], a').forEach((el) => {
        const text = norm(el.textContent);
        if (text.length === 0 || text.length >= 120) return;
        const st = normStrict(text);
        if (!strictTexts.some((t) => st.includes(t))) return;

        el.style.setProperty('display', 'none', 'important');

        if (!el.dataset.llmTranslatorHiddenSuggestion) {
          el.dataset.llmTranslatorHiddenSuggestion = 'true';
          hiddenCount += 1;
        }
      });

      // Div-based suggestion cards (rendered without button/a/[role=button] on some PCs)
      function isTranslateUiContainerText(text) {
        var st = normStrict(text);
        return (
          st.includes('言語を検出する') &&
          st.includes('日本語') &&
          st.includes('より自然な表現に') &&
          st.includes('ビジネス用にする')
        );
      }

      function shouldSkipDiv(el, text) {
        var tag = el.tagName;
        if (tag === 'MAIN' || tag === 'SECTION') return true;
        if (tag === 'BODY' || tag === 'HTML') return true;
        if (isTranslateUiContainerText(text)) return true;
        var hitCount = strictTexts.filter(function(t) {
          return normStrict(text).includes(t);
        }).length;
        if (hitCount >= 3) return true;
        return false;
      }

      document.querySelectorAll('div').forEach(function(el) {
        var text = norm(el.textContent);
        if (text.length === 0 || text.length > 160) return;
        var st = normStrict(text);
        if (!strictTexts.some(function(t) { return st.includes(t); })) return;
        if (shouldSkipDiv(el, text)) return;

        el.style.setProperty('display', 'none', 'important');

        if (!el.dataset.llmTranslatorHiddenSuggestion) {
          el.dataset.llmTranslatorHiddenSuggestion = 'true';
          hiddenCount += 1;
        }
      });

      // 3a. Hide suggestion card parent sections (app shell variant)
      cleanupLog('suggestion parent cleanup step');
      document.querySelectorAll('section, div').forEach(function(el) {
        var rect = el.getBoundingClientRect();
        if (rect.height < 40 || rect.height > 220) return;

        var text = norm(el.textContent || '');
        var st = normStrict(text);
        var matchCount = suggestionTexts.filter(function(t) {
          return st.includes(normStrict(t));
        }).length;

        if (matchCount < 1) return;
        if (hasTranslationUi(el)) return;
        if (el.querySelector('[data-llm-chatgpt-form="true"]')) return;

        hide(el);
      });

      cleanupLog('cleanup: newly hidden ' + hiddenCount + ' suggestion cards');

      // 3b. Hide LP/marketing sections (scroll-mt-mkt-header-height container)
      function hasTranslationUi(el) {
        return !!el.querySelector(
          'textarea, input, [contenteditable="true"], [role="combobox"], button[role="combobox"], button.interactive-button.interactive-button-secondary'
        );
      }

      if (hideLpElements) {
      document.querySelectorAll('[class*="scroll-mt-mkt-header-height"], [class*="pt-mkt-header-height"]').forEach(function(el) {
        if (el.id === 'contentful-header') return;
        if (hasTranslationUi(el)) return;
        hide(el);
      });
      }

      // 4. Footer tag only
      document.querySelectorAll('footer').forEach(hide);

      // 5. Hide heading "ChatGPT を使用して翻訳"
      document.querySelectorAll('h1').forEach((el) => {
        const text = norm(el.textContent);
        if (text.includes('ChatGPT') && (text.includes('翻訳') || text.includes('Translate'))) {
          hide(el);
        }
      });

      // 5a. Hide translate hero section (contains heading + description, variant B/new app layout)
      cleanupLog('hero cleanup step');
      document.querySelectorAll('section, header').forEach(function(section) {
        var text = norm(section.textContent || section.innerText || '');

        var isTranslateHero =
          text.includes('ChatGPT を使用して翻訳') ||
          text.includes('元の意味やトーン、意図を保って翻訳します') ||
          text.includes('Translate with ChatGPT') ||
          text.includes('Preserve the original meaning');

        if (!isTranslateHero) return;
        if (hasTranslationUi(section)) return;

        hide(section);
      });

      // 5b. Hide LP headings: find heading by text, then walk up to hide LP section structurally
      if (hideLpElements) {
      document.querySelectorAll('h1, h2, div').forEach(function(el) {
        var text = norm(el.textContent);
        if (
          !text.includes('翻訳に ChatGPT を使う理由') &&
          !text.includes('Why use ChatGPT for translation') &&
          !text.includes('Use ChatGPT for translation') &&
          !text.includes('仕組み')
        ) {
          return;
        }

        var node = el;
        while (node && node !== document.body) {
          if (node.matches('[data-llm-chatgpt-container="true"], [data-llm-chatgpt-form="true"], main')) {
            break;
          }
          if (hasTranslationUi(node)) {
            break;
          }
          if (node.matches('[class*="scroll-mt-mkt-header-height"], section')) {
            hide(node);
            break;
          }
          node = node.parentElement;
        }
      });
      }

      // 5c. Hide CTA elements (variant A: "ChatGPT で翻訳を開始する" etc.)
      if (hideLpElements) {
      var ctaTexts = [
        'ChatGPT で翻訳を開始する',
        'Start translating with ChatGPT',
        '今すぐ試す',
        'Try now'
      ];

      document.querySelectorAll('[class*="col-span-full"], h1, h2, div, a, button').forEach(function(el) {
        var text = norm(el.textContent || el.innerText || '');
        if (!ctaTexts.some(function(t) { return text.includes(t); })) return;
        if (hasTranslationUi(el)) return;

        var node = el;
        while (node && node !== document.body) {
          if (node.matches('[data-llm-chatgpt-container="true"], [data-llm-chatgpt-form="true"], main')) break;
          if (hasTranslationUi(node)) break;

          if (
            node.matches('[class*="col-span-full"], section') ||
            ctaTexts.some(function(t) { return norm(node.textContent || '').includes(t); })
          ) {
            hide(node);
            break;
          }

          node = node.parentElement;
        }
      });
      }

      // 6. Mark translate layout with custom attributes for flex expansion
      markLoginHeader();
      markLoginBlock();
      markTranslateLayout();
      markLanguageRow();

      // Reset scroll position after layout markers and CSS-dependent changes
      try {
        window.scrollTo(0, 0);
        document.documentElement.scrollTop = 0;
        document.body.scrollTop = 0;
      } catch (_) {}

      setTimeout(function() {
        try {
          markTranslateLayout();
          markLanguageRow();
          window.scrollTo(0, 0);
          document.documentElement.scrollTop = 0;
          document.body.scrollTop = 0;
        } catch (_) {}
      }, 50);

    } catch (e) {
      console.warn('[LLM Translator Desktop] ChatGPT cleanup failed:', e);
    }
  }

  applyCleanup();

  if (!window.__LLM_TRANSLATOR_CHATGPT_CLEANUP_OBSERVER__) {
    window.__LLM_TRANSLATOR_CHATGPT_CLEANUP_OBSERVER__ = true;

    let timer = null;
    const observer = new MutationObserver(() => {
      if (timer) return;
      timer = setTimeout(() => {
        timer = null;
        observer.disconnect();
        try {
          applyCleanup();
        } catch (e) {
          console.warn('[LLM Translator Desktop] applyCleanup failed', e);
        } finally {
          observer.observe(document.documentElement, {
            childList: true,
            subtree: true
          });
        }
      }, 200);
    });

    observer.observe(document.documentElement, {
      childList: true,
      subtree: true
    });

    cleanupLog('cleanup: CSS injected, observer installed');
  }
})();
"#;
    w.eval(js)
        .map_err(|e| format!("failed to apply ChatGPT Translate cleanup: {}", e))
}

fn schedule_chatgpt_translate_cleanup(app: tauri::AppHandle) {
    use tauri::Manager;
    let hide_lp = app.state::<AppState>().config.lock()
        .map(|c| c.general.chatgpt_translate_hide_lp)
        .unwrap_or(true);
    tauri::async_runtime::spawn(async move {
        let delays = [0_u64, 300, 800, 1500, 3000, 5000, 8000];
        for delay in delays {
            if delay > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }
            if let Some(w) = app.get_webview(CHATGPT_TRANSLATE_LABEL) {
                let _ = apply_chatgpt_translate_cleanup(&w, hide_lp);
            }
        }
    });
}

#[tauri::command]
pub async fn open_chatgpt_translate(app: tauri::AppHandle, url: String, x: f64, y: f64, width: f64, height: f64) -> Result<(), String> {
    use tauri::{Manager, Position, Size, Url, WebviewBuilder, WebviewUrl};
    let label = CHATGPT_TRANSLATE_LABEL;
    let parsed_url: Url = url.parse().map_err(|e| format!("invalid url: {}", e))?;
    if let Some(w) = app.get_webview(label) {
        let _ = w.set_position(Position::Logical(tauri::LogicalPosition { x, y }));
        let _ = w.set_size(Size::Logical(tauri::LogicalSize { width, height }));
        let _ = w.show();
        schedule_chatgpt_translate_cleanup(app.clone());
        return Ok(());
    }
    let main_webview = app.get_webview("main").ok_or("main webview not found")?;
    let main_window = main_webview.window();

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("llm-translator")
        .join("chatgpt-translate-webview");

    let webview_builder = WebviewBuilder::new(label, WebviewUrl::External(parsed_url))
        .transparent(true)
        .auto_resize()
        .data_directory(data_dir);


    main_window
        .add_child(
            webview_builder,
            Position::Logical(tauri::LogicalPosition { x, y }),
            Size::Logical(tauri::LogicalSize { width, height }),
        )
        .map_err(|e| e.to_string())?;

    schedule_chatgpt_translate_cleanup(app.clone());
    Ok(())
}

#[tauri::command]
pub async fn set_chatgpt_translate_visible(
    app: tauri::AppHandle,
    visible: bool,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    use tauri::Manager;
    let w = app.get_webview(CHATGPT_TRANSLATE_LABEL).ok_or("webview not found")?;
    if visible {
        let _ = w.set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
        let _ = w.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }));
        let _ = w.show();
    } else {
        let _ = w.hide();
    }
    Ok(())
}

#[tauri::command]
pub async fn chatgpt_translate_back(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let w = app.get_webview(CHATGPT_TRANSLATE_LABEL).ok_or("webview not found")?;
    w.eval("window.history.back()").map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn chatgpt_translate_forward(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let w = app.get_webview(CHATGPT_TRANSLATE_LABEL).ok_or("webview not found")?;
    w.eval("window.history.forward()").map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn chatgpt_translate_reload(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let w = app.get_webview(CHATGPT_TRANSLATE_LABEL).ok_or("webview not found")?;
    w.eval("window.location.reload()").map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn chatgpt_translate_home(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri::Manager;
    let w = app.get_webview(CHATGPT_TRANSLATE_LABEL).ok_or("webview not found")?;
    let parsed: tauri::Url = url.parse().map_err(|e| format!("invalid url: {}", e))?;
    w.navigate(parsed).map_err(|e| e.to_string())?;
    schedule_chatgpt_translate_cleanup(app);
    Ok(())
}

#[tauri::command]
pub async fn get_chatgpt_translate_url(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;
    let w = app.get_webview(CHATGPT_TRANSLATE_LABEL).ok_or("webview not found")?;
    w.url().map(|u| u.to_string()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn debug_chatgpt_translate_dom(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;
    let w = app.get_webview(CHATGPT_TRANSLATE_LABEL).ok_or("webview not found")?;
    let js = r#"
(function(){
  if (location.hash.startsWith('#__llm_dbg=') || location.hash.startsWith('#__llm_htmlcss=')) {
    history.replaceState(null, '', location.pathname + location.search);
  }
  function norm(s){return(s||'').replace(/\s+/g,' ').trim();}
  function isVisible(el){
    var s=el.style;
    if(s.display==='none'||s.visibility==='hidden')return false;
    var r=el.getBoundingClientRect();
    return r.width>0&&r.height>0;
  }
  var skipTags={BODY:1,HTML:1,MAIN:1,HEAD:1,SCRIPT:1,STYLE:1,LINK:1,META:1,NOSCRIPT:1};
  var priorityKeywords=['ビジネスにする','洗練されたビジネス向けのトーンにします','5歳児にもわかるように説明して','とてもやさしい言葉で書き直します','より自然な表現に','ビジネス向けにする','5歳でもわかるように','学術向けにする','Make it more natural','Make it business-friendly','Explain it like I am 5','Make it academic'];
  var selector='button,[role="button"],a,div,span,[tabindex],textarea,[contenteditable="true"]';
  var all=document.querySelectorAll(selector);
  var candidates=[];
  for(var i=0;i<all.length;i++){
    var el=all[i];
    if(skipTags[el.tagName])continue;
    var text=norm(el.textContent);
    if(!text)continue;
    if(text.length>300)continue;
    if(!isVisible(el))continue;
    var r=el.getBoundingClientRect();
    var info={
      tag:el.tagName,
      role:el.getAttribute('role')||'',
      ariaLabel:el.getAttribute('aria-label')||'',
      className:(typeof el.className==='string'?el.className:'').slice(0,120),
      id:el.id||'',
      text:text.slice(0,120),
      rect:{x:Math.round(r.x),y:Math.round(r.y),w:Math.round(r.width),h:Math.round(r.height)},
      cursor:el.style.cursor||(el.onclick?'pointer':'')||'',
      tabIndex:el.tabIndex,
      parentTag:el.parentElement?el.parentElement.tagName:'',
      parentClass:el.parentElement?(typeof el.parentElement.className==='string'?el.parentElement.className:'').slice(0,80):'',
      parentRole:el.parentElement?el.parentElement.getAttribute('role')||'':'',
      parentText:el.parentElement?norm(el.parentElement.textContent).slice(0,100):''
    };
    var priority=0;
    for(var k=0;k<priorityKeywords.length;k++){
      if(text.indexOf(priorityKeywords[k])>=0){priority=1;break;}
    }
    info.priority=priority;
    if(priority||(r.width<600&&r.height<500)){
      if(r.width>window.innerWidth*0.8&&r.height>window.innerHeight*0.5){continue;}
	      candidates.push(info);
    }
  }
  candidates.sort(function(a,b){return b.priority-a.priority||a.tag.localeCompare(b.tag);});
  candidates=candidates.slice(0,80);
  // Child dump: find containers with all 3 suggestion keywords, dump their direct children
  var suggestionContainerChildren=[];
  var suggestionKeywords=['より自然な表現に','ビジネス用にする','5歳児にもわかるように説明して'];
  var allDivs=document.querySelectorAll('div');
  for(var d=0;d<allDivs.length;d++){
    var dv=allDivs[d];
    var dt=norm(dv.textContent);
    var allMatch=true;
    for(var m=0;m<suggestionKeywords.length;m++){
      if(dt.indexOf(suggestionKeywords[m])<0){allMatch=false;break;}
    }
    if(!allMatch)continue;
    var children=[];
    for(var c=0;c<dv.children.length;c++){
      var child=dv.children[c];
      var cr=child.getBoundingClientRect();
      children.push({
        tag:child.tagName,
        className:(typeof child.className==='string'?child.className:'').slice(0,120),
        text:norm(child.textContent).slice(0,160),
        rect:{x:Math.round(cr.x),y:Math.round(cr.y),w:Math.round(cr.width),h:Math.round(cr.height)},
        role:child.getAttribute('role')||'',
        ariaLabel:child.getAttribute('aria-label')||'',
        childCount:child.children.length
      });
    }
    suggestionContainerChildren.push({
      parentTag:dv.tagName,
      parentClass:(typeof dv.className==='string'?dv.className:'').slice(0,120),
      parentText:dt.slice(0,160),
      childCount:children.length,
      children:children
    });
  }
  var cleanLocation = location.origin + location.pathname + location.search;
  var result={location:cleanLocation,candidateCount:candidates.length,candidates:candidates,suggestionContainerChildren:suggestionContainerChildren};
  window.location.hash='__llm_dbg='+encodeURIComponent(JSON.stringify(result));
})()"#;
    w.eval(js).map_err(|e| format!("eval failed: {e}"))?;
    for i in 0..15 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let url = w.url().map_err(|e| e.to_string())?;
        if let Some(frag) = url.fragment() {
            if let Some(data) = frag.strip_prefix("__llm_dbg=") {
                let _ = w.eval("if(location.hash.startsWith('#__llm_dbg=')||location.hash.startsWith('#__llm_htmlcss=')){history.replaceState(null,'',location.pathname+location.search);}");
                return Ok(data.to_string());
            }
        }
        if i == 14 {
            return Err("timeout waiting for debug data from ChatGPT webview".into());
        }
    }
    Err("unexpected".into())
}

#[tauri::command]
pub async fn debug_chatgpt_translate_html_css(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;
    let w = app.get_webview(CHATGPT_TRANSLATE_LABEL).ok_or("webview not found")?;
    let js = r#"
(function(){
  if (location.hash.startsWith('#__llm_dbg=') || location.hash.startsWith('#__llm_htmlcss=')) {
    history.replaceState(null, '', location.pathname + location.search);
  }
  var targetedHtmlInspect=[];
  var inspectSelectors=['header','footer','#contentful-header','#sidebar-header','[class*="h-header-height"]','[class*="mkt-header-height"]','[class*="scroll-mt-mkt-header-height"]','[data-llm-chatgpt-login-header]','[data-llm-chatgpt-hidden]'];
  var seen=new Set();
  for(var s=0;s<inspectSelectors.length&&targetedHtmlInspect.length<20;s++){
    var els=document.querySelectorAll(inspectSelectors[s]);
    for(var e=0;e<els.length&&targetedHtmlInspect.length<20;e++){
      var el=els[e];
      if (seen.has(el)) continue;
      seen.add(el);
      var r=el.getBoundingClientRect();
      var cs=getComputedStyle(el);
      var children=[];
      for(var c=0;c<el.children.length;c++){
        var ch=el.children[c];
        var cr=ch.getBoundingClientRect();
        var chs=getComputedStyle(ch);
        children.push({
          tag:ch.tagName,
          className:(typeof ch.className==='string'?ch.className:'').slice(0,120),
          text:(ch.textContent||'').replace(/\s+/g,' ').trim().slice(0,80),
          rect:{x:Math.round(cr.x),y:Math.round(cr.y),w:Math.round(cr.width),h:Math.round(cr.height)},
          display:chs.display,
          visibility:chs.visibility,
          childElementCount:ch.childElementCount
        });
      }
      targetedHtmlInspect.push({
        tag:el.tagName,
        className:(typeof el.className==='string'?el.className:'').slice(0,200),
        id:el.id||'',
        role:el.getAttribute('role')||'',
        ariaLabel:el.getAttribute('aria-label')||'',
        textContent:(el.textContent||'').replace(/\s+/g,' ').trim().slice(0,120),
        outerHTML:(el.outerHTML||'').slice(0,600),
        rect:{x:Math.round(r.x),y:Math.round(r.y),w:Math.round(r.width),h:Math.round(r.height)},
        computedStyle:{
          display:cs.display,
          visibility:cs.visibility,
          position:cs.position,
          zIndex:cs.zIndex,
          overflow:cs.overflow,
          width:cs.width,
          height:cs.height,
          pointerEvents:cs.pointerEvents
        },
        childrenSummary:children
      });
    }
  }
  var cleanLocation = location.origin + location.pathname + location.search;
  var result={location:cleanLocation,targetedHtmlInspect:targetedHtmlInspect};
  window.location.hash='__llm_htmlcss='+encodeURIComponent(JSON.stringify(result));
})()"#;
    w.eval(js).map_err(|e| format!("eval failed: {e}"))?;
    for i in 0..15 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let url = w.url().map_err(|e| e.to_string())?;
        if let Some(frag) = url.fragment() {
            if let Some(data) = frag.strip_prefix("__llm_htmlcss=") {
                let _ = w.eval("if(location.hash.startsWith('#__llm_dbg=')||location.hash.startsWith('#__llm_htmlcss=')){history.replaceState(null,'',location.pathname+location.search);}");
                return Ok(data.to_string());
            }
        }
        if i == 14 {
            return Err("timeout waiting for HTML+CSS debug data from ChatGPT webview".into());
        }
    }
    Err("unexpected".into())
}

#[tauri::command]
pub async fn set_chatgpt_console_log_enabled(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    use tauri::Manager;
    let w = app.get_webview(CHATGPT_TRANSLATE_LABEL).ok_or("webview not found")?;
    let flag = if enabled { "true" } else { "false" };
    let js = format!(r#"(function(){{
  window.__llmChatgptConsoleLogEnabled = {flag};
  if (window.__llmChatgptConsolePatched) return;
  window.__llmChatgptConsolePatched = true;
  var CONSOLE_LOG_KEY = '__llmChatgptConsoleLog';
  function readConsoleLogs() {{
    try {{ var raw = sessionStorage.getItem(CONSOLE_LOG_KEY); var parsed = raw ? JSON.parse(raw) : []; return Array.isArray(parsed) ? parsed : []; }}
    catch(e) {{ return []; }}
  }}
  function writeConsoleLogs(entries) {{
    try {{ sessionStorage.setItem(CONSOLE_LOG_KEY, JSON.stringify(entries)); }} catch(e) {{}}
    window.__llmChatgptConsoleLog = entries;
  }}
  window.__llmChatgptConsoleLog = readConsoleLogs();
  function safeSerialize(v) {{
    if (v === null || v === undefined) return v;
    if (typeof v === 'string' || typeof v === 'number' || typeof v === 'boolean') return v;
    if (v instanceof Error) return {{ name: v.name, message: v.message, stack: v.stack }};
    if (typeof v === 'function') return '[Function]';
    if (v instanceof HTMLElement) return '<' + v.tagName.toLowerCase() + '>';
    try {{ return JSON.parse(JSON.stringify(v)); }}
    catch(e) {{ return '[Unserializable: ' + e.message + ']'; }}
  }}
  function appendConsoleEntry(entry) {{
    if (!window.__llmChatgptConsoleLogEnabled) return;
    var log = readConsoleLogs();
    log.push(entry);
    if (log.length > 500) log.splice(0, log.length - 500);
    writeConsoleLogs(log);
  }}
  ['log','warn','error','info','debug'].forEach(function(level) {{
    var orig = console[level].bind(console);
    console[level] = function() {{
      var args = Array.prototype.slice.call(arguments);
      appendConsoleEntry({{ ts: new Date().toISOString(), level: level, source: 'console', args: args.map(safeSerialize) }});
      orig.apply(console, args);
    }};
  }});
  window.addEventListener('error', function(ev) {{
    appendConsoleEntry({{ ts: new Date().toISOString(), level: 'error', source: 'window.onerror', args: [ev.message, ev.filename, ev.lineno, ev.colno, ev.error ? ev.error.stack : ''] }});
  }});
  window.addEventListener('unhandledrejection', function(ev) {{
    var reason = ev.reason;
    appendConsoleEntry({{ ts: new Date().toISOString(), level: 'error', source: 'unhandledrejection', args: [reason instanceof Error ? {{ name: reason.name, message: reason.message, stack: reason.stack }} : safeSerialize(reason)] }});
  }});
}})()"#);
    w.eval(&js).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_chatgpt_translate_console_log(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;
    let w = app.get_webview(CHATGPT_TRANSLATE_LABEL).ok_or("webview not found")?;
    let js = r#"(function(){
  if (location.hash.startsWith('#__llm_consolelog=')) {
    history.replaceState(null, '', location.pathname + location.search);
  }
  var windowValue = window.__llmChatgptConsoleLog;
  var storageRaw = null;
  var storageParsed = null;
  var storageError = null;
  try {
    storageRaw = sessionStorage.getItem('__llmChatgptConsoleLog');
    storageParsed = storageRaw ? JSON.parse(storageRaw) : null;
  } catch (e) { storageError = String(e); }
  var result = {
    href: location.href,
    hasWindowProperty: Object.prototype.hasOwnProperty.call(window, '__llmChatgptConsoleLog'),
    windowValueType: typeof windowValue,
    windowIsArray: Array.isArray(windowValue),
    windowLength: Array.isArray(windowValue) ? windowValue.length : null,
    hasSessionStorageValue: storageRaw !== null,
    sessionStorageRawLength: storageRaw ? storageRaw.length : 0,
    sessionStorageIsArray: Array.isArray(storageParsed),
    sessionStorageLength: Array.isArray(storageParsed) ? storageParsed.length : null,
    sessionStorageError: storageError,
    entries: Array.isArray(storageParsed) ? storageParsed : Array.isArray(windowValue) ? windowValue : null
  };
  location.hash = '__llm_consolelog=' + encodeURIComponent(JSON.stringify(result));
})()"#;
    w.eval(js).map_err(|e| format!("eval failed: {e}"))?;
    for i in 0..15 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let url = w.url().map_err(|e| e.to_string())?;
        if let Some(frag) = url.fragment() {
            if let Some(data) = frag.strip_prefix("__llm_consolelog=") {
                let _ = w.eval("if(location.hash.startsWith('#__llm_consolelog=')){history.replaceState(null,'',location.pathname+location.search);}");
                return Ok(data.to_string());
            }
        }
        if i == 14 {
            return Err("timeout waiting for console log from ChatGPT webview".into());
        }
    }
    Err("unexpected".into())
}

#[tauri::command]
pub async fn get_language_debug_log(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;

    let _bridge_guard = CHATGPT_LANGUAGE_BRIDGE_LOCK.lock().await;
    let webview = app.get_webview(CHATGPT_TRANSLATE_LABEL).ok_or("webview not found")?;
    let request_id = uuid::Uuid::new_v4().simple().to_string();
    let request_id_json = serde_json::to_string(&request_id).map_err(|error| error.to_string())?;
    let script_template = r#"(function() {
  const requestId = __REQUEST_ID__;
  const previousBridge = window.__llmChatgptLangBridge;
  const originalHash = previousBridge && location.hash === previousBridge.resultHash
    ? previousBridge.originalHash
    : location.hash;
  const windowValue = window.__llmChatgptLanguageDebugLog;
  let storageRaw = null;
  let storageParsed = null;
  let storageError = null;
  try {
    storageRaw = sessionStorage.getItem('__llmChatgptLanguageDebugLog');
    storageParsed = storageRaw ? JSON.parse(storageRaw) : null;
  } catch (error) {
    storageError = String(error);
  }
  const sourceEntries = Array.isArray(storageParsed)
    ? storageParsed
    : Array.isArray(windowValue) ? windowValue : null;
  function safeHref(value) {
    try {
      const parsed = new URL(value, location.href);
      return parsed.origin + parsed.pathname;
    } catch (_) {
      return location.origin + location.pathname;
    }
  }
  const entries = Array.isArray(sourceEntries) ? sourceEntries.map(function(entry) {
    if (!entry || typeof entry !== 'object') return entry;
    const sanitized = Object.assign({}, entry);
    if (typeof sanitized.href === 'string') sanitized.href = safeHref(sanitized.href);
    return sanitized;
  }) : sourceEntries;
  const result = {
    requestId,
    href: location.origin + location.pathname,
    entryCount: Array.isArray(entries) ? entries.length : null,
    maxEntries: 500,
    mayBeTruncated: Array.isArray(entries) && entries.length >= 500,
    readyState: document.readyState,
    hasWindowProperty: Object.prototype.hasOwnProperty.call(window, '__llmChatgptLanguageDebugLog'),
    windowValueType: typeof windowValue,
    windowIsArray: Array.isArray(windowValue),
    windowLength: Array.isArray(windowValue) ? windowValue.length : null,
    hasSessionStorageValue: storageRaw !== null,
    sessionStorageRawLength: storageRaw ? storageRaw.length : 0,
    sessionStorageIsArray: Array.isArray(storageParsed),
    sessionStorageLength: Array.isArray(storageParsed) ? storageParsed.length : null,
    sessionStorageError: storageError,
    entries
  };
  const resultHash = '#__llm_langlog=' + requestId + '=' + encodeURIComponent(JSON.stringify(result));
  window.__llmChatgptLangBridge = { requestId, kind: 'language-log', originalHash, resultHash };
  location.hash = resultHash;
})()"#;
    let script = script_template.replace("__REQUEST_ID__", &request_id_json);
    webview.eval(&script).map_err(|error| format!("language log collection eval failed: {error}"))?;

    let result_prefix = format!("__llm_langlog={request_id}=");
    for _ in 0..240 {
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        let url = webview.url().map_err(|error| error.to_string())?;
        if let Some(fragment) = url.fragment() {
            if let Some(payload) = fragment.strip_prefix(&result_prefix) {
                let request_id_js = serde_json::to_string(&request_id).map_err(|error| error.to_string())?;
                let restore_script = format!(
                    "(function(){{var b=window.__llmChatgptLangBridge;if(b&&b.requestId==={request_id_js}&&location.hash===b.resultHash){{history.replaceState(null,'',location.pathname+location.search+(b.originalHash||''));delete window.__llmChatgptLangBridge;}}}})();"
                );
                webview.eval(&restore_script).map_err(|error| format!("language log URL restore failed: {error}"))?;
                return Ok(payload.to_string());
            }
        }
    }

    let request_id_js = serde_json::to_string(&request_id).map_err(|error| error.to_string())?;
    let restore_script = format!(
        "(function(){{var b=window.__llmChatgptLangBridge;if(b&&b.requestId==={request_id_js}){{if(location.hash===b.resultHash)history.replaceState(null,'',location.pathname+location.search+(b.originalHash||''));delete window.__llmChatgptLangBridge;}}}})();"
    );
    let _ = webview.eval(&restore_script);
    Err("timeout waiting for language debug log from ChatGPT webview".into())
}
#[tauri::command]
pub async fn set_chatgpt_translate_text(app: tauri::AppHandle, text: String) -> Result<(), String> {
    use tauri::Manager;
    println!("[Ctrl+C+C] set_chatgpt_translate_text, len={}", text.len());
    let w = app.get_webview(CHATGPT_TRANSLATE_LABEL).ok_or("webview not found")?;
    let text_json = serde_json::to_string(&text).map_err(|e| e.to_string())?;
    let js = format!(r#"(function(){{var t={text_json};function s(e,v){{var d=Object.getOwnPropertyDescriptor(Object.getPrototypeOf(e),'value');if(d&&d.set)d.set.call(e,v);else e.value=v;e.dispatchEvent(new Event('input',{{bubbles:true}}));e.dispatchEvent(new Event('change',{{bubbles:true}}));}}function f(){{var ts=document.querySelectorAll('textarea');for(var i=0;i<ts.length;i++){{if(!ts[i].hasAttribute('readonly'))return ts[i];}}return null;}}function trySet(){{var el=f();if(!el)return false;el.focus();s(el,t);return true;}}if(trySet())return;var n=0;var iv=setInterval(function(){{n++;if(trySet()||n>=40)clearInterval(iv);}},250);}})();"#);
    w.eval(&js).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_chatgpt_translate_languages(
    app: tauri::AppHandle,
    source_code: String,
    target_code: String,
    source_label: String,
    target_label: String,
    source_label_en: String,
    target_label_en: String,
) -> Result<(), String> {
    use tauri::Manager;

    let _bridge_guard = CHATGPT_LANGUAGE_BRIDGE_LOCK.lock().await;
    let webview = app.get_webview(CHATGPT_TRANSLATE_LABEL).ok_or("webview not found")?;
    let request_id = uuid::Uuid::new_v4().simple().to_string();
    let params = serde_json::json!({
        "requestId": request_id.clone(),
        "sourceCode": source_code,
        "targetCode": target_code,
        "sourceLabel": source_label,
        "targetLabel": target_label,
        "sourceLabelEn": source_label_en,
        "targetLabelEn": target_label_en,
    });
    let params_json = serde_json::to_string(&params).map_err(|error| error.to_string())?;
    let script_template = r#"(function() {
  const params = __PARAMS__;
  const previousBridge = window.__llmChatgptLangBridge;
  const originalHash = previousBridge && location.hash === previousBridge.resultHash
    ? previousBridge.originalHash
    : location.hash;
  const controller = window.__llmChatgptLanguageController || { runId: 0, requestId: null, active: false };
  window.__llmChatgptLanguageController = controller;
  controller.runId += 1;
  controller.requestId = params.requestId;
  controller.active = true;
  const runId = controller.runId;
  const logKey = '__llmChatgptLanguageDebugLog';
  const maxLogs = 500;
  const sourceAliases = [params.sourceLabel, params.sourceLabelEn];
  const isAutoDetect = params.sourceCode === 'auto';
  if (isAutoDetect) {
    sourceAliases.push('言語を自動検出', '自動検出', 'Auto-detect', 'Auto detect');
  }
  const targetAliases = [params.targetLabel, params.targetLabelEn];

  function isCurrentRun() {
    return controller.active && controller.runId === runId && controller.requestId === params.requestId;
  }
  function normalize(value) {
    return (value || '').replace(/\s+/g, ' ').trim().toLocaleLowerCase();
  }
  function visible(element) {
    if (!element) return false;
    const rect = element.getBoundingClientRect();
    const style = window.getComputedStyle(element);
    return rect.width > 0 && rect.height > 0 && style.display !== 'none' && style.visibility !== 'hidden';
  }
  function safeHref() {
    return location.origin + location.pathname;
  }
  function readLogs() {
    try {
      const parsed = JSON.parse(sessionStorage.getItem(logKey) || '[]');
      return Array.isArray(parsed) ? parsed : [];
    } catch (_) {
      return [];
    }
  }
  function writeLogs(entries) {
    try { sessionStorage.setItem(logKey, JSON.stringify(entries)); } catch (_) {}
    window.__llmChatgptLanguageDebugLog = entries;
  }
  function debug(event, data) {
    try {
      const entries = readLogs();
      entries.push(Object.assign({
        ts: new Date().toISOString(),
        event,
        runId,
        requestId: params.requestId,
        desiredKey: JSON.stringify([params.sourceCode, params.targetCode]),
        href: safeHref(),
        readyState: document.readyState
      }, data || {}));
      if (entries.length > maxLogs) entries.splice(0, entries.length - maxLogs);
      writeLogs(entries);
      if (window.__llmChatgptConsoleLogEnabled) console.log('[LangDbg]', event, data || {});
    } catch (error) {
      try { console.error('[LangDbg] logging failed', String(error)); } catch (_) {}
    }
  }
  function findButton(kind) {
    const japanese = kind === 'source' ? '翻訳元の言語' : '翻訳先の言語';
    const english = kind === 'source' ? 'Source language' : 'Target language';
    return Array.from(document.querySelectorAll('button[aria-label]')).find(function(button) {
      const label = normalize(button.getAttribute('aria-label') || '');
      if (isSwapButton(button, label)) return false;
      return label === normalize(japanese) || label.startsWith(normalize(japanese) + ':') ||
        label === normalize(english) || label.startsWith(normalize(english) + ':');
    }) || null;
  }
  function isSwapButton(button, normalizedLabel) {
    const label = normalizedLabel || normalize(button && button.getAttribute('aria-label'));
    const text = normalize(button && (button.textContent || button.innerText));
    return /入れ替え|入れ換え|swap/.test(label) || /入れ替え|入れ換え|swap/.test(text);
  }
  function getLanguageButtons() {
    let source = findButton('source');
    let target = findButton('target');
    if (source && target) return { source, target };
    const root = document.querySelector('[data-llm-chatgpt-form="true"]') || document.querySelector('[data-llm-chatgpt-container="true"]') || document;
    const fallback = Array.from(root.querySelectorAll('button.interactive-button.interactive-button-secondary'))
      .filter(function(button) {
        if (!visible(button)) return false;
        if (isSwapButton(button)) return false;
        const span = button.querySelector('span.truncate');
        return !!(span && normalize(span.textContent));
      })
      .sort(function(a, b) { return a.getBoundingClientRect().left - b.getBoundingClientRect().left; });
    if (!source && fallback.length >= 1) source = fallback[0];
    if (!target && fallback.length >= 2) target = fallback[1];
    return { source, target };
  }
  function getLanguageButton(kind) {
    const buttons = getLanguageButtons();
    return kind === 'source' ? buttons.source : buttons.target;
  }
  function currentValue(button) {
    if (!button) return '';
    const text = normalize(button.textContent || button.innerText);
    if (text) return text;
    const label = (button.getAttribute('aria-label') || '').replace(/^[^:：]+[:：]\s*/, '');
    return normalize(label);
  }
  function optionPrefix(kind) {
    return 'translate-' + kind + '-language-';
  }
  function findMenu(kind, button) {
    const controls = button && button.getAttribute('aria-controls');
    if (controls) {
      const controlled = document.getElementById(controls);
      if (visible(controlled)) return controlled;
    }
    const prefix = optionPrefix(kind);
    const menus = Array.from(document.querySelectorAll('[role="listbox"], [role="dialog"]')).filter(visible);
    const prefixedMenu = menus.find(function(menu) {
      return Array.from(menu.querySelectorAll('[role="option"], [role="menuitem"], button'))
        .some(function(option) { return (option.id || '').startsWith(prefix); });
    });
    if (prefixedMenu) return prefixedMenu;
    const optionMenus = menus.filter(function(menu) {
      return menu.querySelector('[role="option"], [role="menuitem"]');
    });
    if (!button || optionMenus.length === 0) return null;
    const buttonRect = button.getBoundingClientRect();
    const buttonCenter = buttonRect.left + buttonRect.width / 2;
    const relatedMenus = optionMenus.map(function(menu) {
      const rect = menu.getBoundingClientRect();
      return { menu, distance: Math.abs((rect.left + rect.width / 2) - buttonCenter) };
    }).sort(function(a, b) { return a.distance - b.distance; });
    if (button.getAttribute('aria-expanded') === 'true' || optionMenus.length === 1) {
      return relatedMenus[0].menu;
    }
    const nearest = relatedMenus[0];
    return nearest && nearest.distance <= Math.max(buttonRect.width, 160) ? nearest.menu : null;
  }
  function isMenuOpen(kind, button) {
    return !!(button && button.getAttribute('aria-expanded') === 'true') || !!findMenu(kind, button);
  }
  function optionText(option) {
    return normalize(option.textContent || option.innerText || option.getAttribute('aria-label'));
  }
  function findOption(kind, code, aliases, button) {
    const prefix = optionPrefix(kind);
    const menu = findMenu(kind, button);
    const expectedId = kind === 'source' && code === 'auto' ? prefix + 'detect' : null;
    if (expectedId) {
      const detectOption = document.getElementById(expectedId);
      if (visible(detectOption) && (!menu || menu.contains(detectOption))) return detectOption;
    }
    const allOptions = menu
      ? Array.from(menu.querySelectorAll('[role="option"], [role="menuitem"], button')).filter(visible)
      : Array.from(document.querySelectorAll('[role="option"], [role="menuitem"]')).filter(visible);
    const prefixed = allOptions.filter(function(option) { return (option.id || '').startsWith(prefix); });
    const candidates = prefixed.length ? prefixed : allOptions;
    const normalizedAliases = aliases.filter(Boolean).map(normalize);
    return candidates.find(function(option) {
      const text = optionText(option);
      const aria = normalize(option.getAttribute('aria-label'));
      return normalizedAliases.includes(text) || normalizedAliases.includes(aria);
    }) || null;
  }
  function wait(ms) {
    return new Promise(function(resolve) { setTimeout(resolve, ms); });
  }
  async function waitForMenu(kind, button) {
    for (let attempt = 0; attempt < 20; attempt += 1) {
      if (!isCurrentRun()) return null;
      const activeButton = getLanguageButton(kind) || button;
      const menu = findMenu(kind, activeButton);
      if (menu) return menu;
      await wait(100);
    }
    return null;
  }
  async function closeMenu(kind, button) {
    if (button && isMenuOpen(kind, button)) {
      try { button.click(); } catch (_) {}
      await wait(100);
    }
  }
  async function selectLanguage(kind, code, button, label, aliases) {
    let activeButton = getLanguageButton(kind) || button;
    if (!activeButton) {
      debug('selection.missing_button', { kind, code, requested: label });
      return 'missing_button';
    }
    const initial = currentValue(activeButton);
    debug('selection.start', { kind, code, requested: label, current: initial, buttonAria: activeButton.getAttribute('aria-label') || '' });
    if (aliases.map(normalize).includes(initial)) {
      debug('selection.already_matched', { kind, code, current: initial });
      return 'matched';
    }
    if (!isMenuOpen(kind, activeButton)) {
      try { activeButton.click(); } catch (error) {
        debug('selection.button_click_error', { kind, code, error: String(error) });
        return 'apply_error';
      }
    }
    const menu = await waitForMenu(kind, activeButton);
    if (!menu) {
      activeButton = getLanguageButton(kind) || activeButton;
      debug('selection.menu_not_found', { kind, code, current: currentValue(activeButton) });
      await closeMenu(kind, activeButton);
      return 'not_found';
    }
    let option = null;
    for (let attempt = 0; attempt < 20; attempt += 1) {
      if (!isCurrentRun()) return 'cancelled';
      activeButton = getLanguageButton(kind) || activeButton;
      option = findOption(kind, code, aliases, activeButton);
      if (option) break;
      if (attempt === 0 || attempt === 10 || attempt === 19) {
        debug('selection.option_snapshot', {
          kind,
          code,
          menuRole: menu.getAttribute('role') || '',
          options: Array.from(menu.querySelectorAll('[role="option"], [role="menuitem"]')).filter(visible).slice(0, 30).map(function(item) {
            return { id: item.id || '', role: item.getAttribute('role') || '', text: optionText(item), aria: normalize(item.getAttribute('aria-label')) };
          })
        });
      }
      await wait(100);
    }
    if (!option) {
      debug('selection.option_not_found', { kind, code, requested: label });
      await closeMenu(kind, button);
      return 'not_found';
    }
    const optionId = option.id || '';
    const selectedText = optionText(option);
    try { option.click(); } catch (error) {
      debug('selection.option_click_error', { kind, code, optionId, selectedText, error: String(error) });
      await closeMenu(kind, button);
      return 'apply_error';
    }
    debug('selection.option_clicked', { kind, code, optionId, selectedText });
    for (let attempt = 0; attempt < 25; attempt += 1) {
      if (!isCurrentRun()) return 'cancelled';
      activeButton = getLanguageButton(kind) || activeButton;
      const current = currentValue(activeButton);
      if (aliases.map(normalize).includes(current)) {
        debug('selection.verified', { kind, code, optionId, current });
        return 'matched';
      }
      await wait(100);
    }
    activeButton = getLanguageButton(kind) || activeButton;
    const current = currentValue(activeButton);
    debug('selection.verification_timeout', { kind, code, optionId, expected: label, current });
    await closeMenu(kind, activeButton);
    return 'timeout';
  }
  function publishResult(sourceStatus, targetStatus) {
    if (!isCurrentRun()) return;
    controller.active = false;
    const resultHash = '#__llm_langresult=' + params.requestId + ':' + sourceStatus + ':' + targetStatus;
    window.__llmChatgptLangBridge = { requestId: params.requestId, kind: 'language-result', originalHash, resultHash };
    debug('selection.complete', { sourceStatus, targetStatus, sourceCurrent: currentValue(getLanguageButtons().source), targetCurrent: currentValue(getLanguageButtons().target) });
    location.hash = resultHash;
  }
  async function run() {
    debug('selection.request', {
      sourceCode: params.sourceCode,
      targetCode: params.targetCode,
      sourceLabel: params.sourceLabel,
      sourceLabelEn: params.sourceLabelEn,
      targetLabel: params.targetLabel,
      targetLabelEn: params.targetLabelEn
    });
    let buttons = getLanguageButtons();
    for (let attempt = 0; attempt < 40 && (!buttons.source || !buttons.target); attempt += 1) {
      if (!isCurrentRun()) return;
      await wait(250);
      buttons = getLanguageButtons();
    }
    buttons = getLanguageButtons();
    debug('selection.buttons', {
      sourceAria: buttons.source && buttons.source.getAttribute('aria-label') || '',
      sourceValue: currentValue(buttons.source),
      targetAria: buttons.target && buttons.target.getAttribute('aria-label') || '',
      targetValue: currentValue(buttons.target)
    });
    const sourceStatus = await selectLanguage('source', params.sourceCode, buttons.source, params.sourceLabel, sourceAliases);
    if (!isCurrentRun()) return;
    buttons = getLanguageButtons();
    if (sourceStatus !== 'matched') await closeMenu('source', buttons.source);
    const targetStatus = await selectLanguage('target', params.targetCode, buttons.target, params.targetLabel, targetAliases);
    if (!isCurrentRun()) return;
    buttons = getLanguageButtons();
    if (targetStatus !== 'matched') await closeMenu('target', buttons.target);
    publishResult(sourceStatus, targetStatus);
  }
  run().catch(function(error) {
    debug('selection.unhandled_error', { error: String(error), stack: error && error.stack ? String(error.stack).slice(0, 500) : '' });
    publishResult('apply_error', 'apply_error');
  });
})()"#;
    let script = script_template.replace("__PARAMS__", &params_json);
    if let Err(error) = webview.eval(&script) {
        return Err(format!("language selection script injection failed: {error}"));
    }

    let result_prefix = format!("__llm_langresult={request_id}:");
    for _ in 0..400 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let url = webview.url().map_err(|error| error.to_string())?;
        if let Some(fragment) = url.fragment() {
            if let Some(payload) = fragment.strip_prefix(&result_prefix) {
                let mut statuses = payload.split(':');
                let source_status = statuses.next().unwrap_or("apply_error");
                let target_status = statuses.next().unwrap_or("apply_error");
                let request_id_js = serde_json::to_string(&request_id).map_err(|error| error.to_string())?;
                let restore_script = format!(
                    "(function(){{var b=window.__llmChatgptLangBridge;if(b&&b.requestId==={request_id_js}&&location.hash===b.resultHash){{history.replaceState(null,'',location.pathname+location.search+(b.originalHash||''));delete window.__llmChatgptLangBridge;}}}})();"
                );
                webview.eval(&restore_script).map_err(|error| format!("language selection URL restore failed: {error}"))?;
                if source_status == "matched" && target_status == "matched" {
                    return Ok(());
                }
                return Err(format!("ChatGPT language selection failed: source={source_status}, target={target_status}"));
            }
        }
    }

    let request_id_js = serde_json::to_string(&request_id).map_err(|error| error.to_string())?;
    let restore_script = format!(
        "(function(){{var b=window.__llmChatgptLangBridge;if(b&&b.requestId==={request_id_js}){{if(location.hash===b.resultHash)history.replaceState(null,'',location.pathname+location.search+(b.originalHash||''));delete window.__llmChatgptLangBridge;}}var c=window.__llmChatgptLanguageController;if(c&&c.requestId==={request_id_js})c.active=false;}})();"
    );
    let _ = webview.eval(&restore_script);
    Err("timeout waiting for ChatGPT language selection result".into())
}
#[derive(serde::Serialize, Clone)]
pub struct LanguageInfo {
    pub code: String,
}

#[derive(serde::Serialize, Clone)]
pub struct EnvVarStatus {
    pub env_var: String,
    pub is_set: bool,
    pub value_length: usize,
}

#[tauri::command]
pub fn check_env_var(env_var: String) -> EnvVarStatus {
    let val = crate::config::read_env_var(&env_var).ok();
    let is_set = val.is_some();
    let value_length = val.map_or(0, |v| v.len());
    EnvVarStatus {
        env_var,
        is_set,
        value_length,
    }
}

#[tauri::command]
pub fn set_user_env_var(name: String, value: String) -> Result<String, String> {
    if name.trim().is_empty() {
        return Err("env_var_name_empty".into());
    }
    if value.trim().is_empty() {
        return Err("api_key_empty".into());
    }

    // Write to user env var via setx
    let output = std::process::Command::new("setx")
        .arg(&name)
        .arg(&value)
        .output()
        .map_err(|e| format!("setx_failed:{}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("env_var_save_failed:{}", stderr));
    }

    // Reflect in current process immediately
    std::env::set_var(&name, &value);

    Ok(name)
}

#[derive(serde::Serialize, Clone)]
pub struct OllamaModel {
    pub name: String,
    pub size: String,
}

fn normalize_ollama_root_url(base_url: &str) -> String {
    let mut url = base_url.trim().trim_end_matches('/').to_string();
    if url.ends_with("/v1") {
        url.truncate(url.len() - 3);
    }
    url
}

#[tauri::command]
pub async fn list_ollama_models(base_url: String) -> Result<Vec<OllamaModel>, String> {
    let root = normalize_ollama_root_url(&base_url);
    let url = format!("{}/api/tags", root);
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| format!("ollama_connect_failed:{}", e))?;

    if !resp.status().is_success() {
        return Err(format!("ollama_error:HTTP {}", resp.status().as_u16()));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("ollama_parse_failed:{}", e))?;

    let models: Vec<OllamaModel> = json
        .get("models")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|m| OllamaModel {
                    name: m
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?")
                        .to_string(),
                    size: format_size(
                        m.get("size")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    ),
                })
                .collect()
        })
        .unwrap_or_else(Vec::new);

    Ok(models)
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.0} MB", bytes as f64 / 1_000_000.0)
    } else {
        format!("{} KB", bytes / 1000)
    }
}
