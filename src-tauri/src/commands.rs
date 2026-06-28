use crate::config::{AppConfig, ProviderConfig};
use crate::history::{self, HistoryEntry};
use crate::providers::{ConnectionTestResult, TranslationRequest, TranslationResponse};
use crate::translator;
use std::sync::Mutex;
use tauri::State;

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

    let model_id = model
        .filter(|m| !m.is_empty())
        .or_else(|| {
            if !provider_config.model.is_empty() {
                Some(provider_config.model.clone())
            } else {
                provider_config.model_mapping.get("default").and_then(|r| {
                    if !r.model.is_empty() { Some(r.model.clone()) } else { None }
                })
            }
        })
        .ok_or_else(|| "errors.no_model_configured".to_string())?;

    let request = TranslationRequest {
        text: text.clone(),
        source_lang: source_lang.clone(),
        target_lang: target_lang.clone(),
        mode: mode.clone(),
        tone: tone.clone(),
        preset_id: preset_id.clone(),
        provider: provider_config.name.clone(),
        model: model_id,
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
pub async fn window_close(window: tauri::Window) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
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
        // Quote the path to handle spaces
        let quoted = format!("\"{}\"", path_str);
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

fn apply_chatgpt_translate_cleanup(w: &tauri::Webview) -> Result<(), String> {
    let js = r#"
(function () {
  const STYLE_ID = 'llm-translator-chatgpt-cleanup-style';
  const HIDDEN_ATTR = 'data-llm-chatgpt-hidden';

  if (!document.getElementById(STYLE_ID)) {
    const style = document.createElement('style');
    style.id = STYLE_ID;
    style.textContent = `
      [data-llm-chatgpt-hidden="true"] {
        display: none !important;
      }
      body {
        overflow-x: hidden !important;
      }

      /* Flex layout: page fills viewport height */
      main#main {
        height: 100vh !important;
        min-height: 0 !important;
        display: flex !important;
        flex-direction: column !important;
        overflow: hidden !important;
        padding-top: 0 !important;
        margin-top: 0 !important;
      }

      main#main > div {
        flex: 1 1 auto !important;
        min-height: 0 !important;
        display: flex !important;
        flex-direction: column !important;
        padding-top: 0 !important;
        margin-top: 0 !important;
      }

      main#main h1 {
        display: none !important;
        margin: 0 !important;
        padding: 0 !important;
        height: 0 !important;
        min-height: 0 !important;
      }

      [data-llm-chatgpt-container="true"] {
        flex: 1 1 auto !important;
        min-height: 0 !important;
        max-width: none !important;
        width: 100% !important;
        display: flex !important;
        flex-direction: column !important;
        gap: 12px !important;
        padding-top: 4px !important;
        padding-bottom: 12px !important;
        margin-top: 0 !important;
      }

      [data-llm-chatgpt-form="true"] {
        flex: 1 1 auto !important;
        min-height: 0 !important;
        display: flex !important;
        flex-direction: column !important;
        gap: 12px !important;
        margin-top: 0 !important;
        padding-top: 0 !important;
      }

      [data-llm-chatgpt-translate-row="true"] {
        flex: 1 1 auto !important;
        min-height: 0 !important;
        display: flex !important;
        flex-direction: row !important;
        gap: 24px !important;
        width: 100% !important;
      }

      [data-llm-chatgpt-column="true"] {
        flex: 1 1 0 !important;
        min-width: 0 !important;
        min-height: 0 !important;
        display: flex !important;
        flex-direction: column !important;
      }

      [data-llm-chatgpt-result-wrapper="true"] {
        flex: 1 1 auto !important;
        min-height: 0 !important;
        display: flex !important;
        width: 100% !important;
      }

      [data-llm-chatgpt-source-textarea="true"],
      [data-llm-chatgpt-target-textarea="true"] {
        flex: 1 1 auto !important;
        width: 100% !important;
        height: 100% !important;
        min-height: 0 !important;
        max-height: none !important;
        box-sizing: border-box !important;
      }

      [data-llm-chatgpt-column="true"] > textarea {
        width: 100% !important;
      }

      /* Header: minimal height for login row, right-aligned */
      [data-llm-chatgpt-login-header="true"] {
        height: 44px !important;
        min-height: 44px !important;
        padding: 4px 24px 4px 24px !important;
        margin: 0 !important;
        background: transparent !important;
        display: flex !important;
        align-items: center !important;
        justify-content: flex-end !important;
        overflow: visible !important;
        position: relative !important;
        z-index: 40 !important;
      }

      /* Login block: normal flow, right-aligned inside the header */
      [data-llm-chatgpt-login-block="true"] {
        position: static !important;
        display: flex !important;
        align-items: center !important;
        justify-content: flex-end !important;
        gap: 8px !important;
        pointer-events: auto !important;
        z-index: 41 !important;
      }

      main#main [data-testid="signup-button"] {
        display: none !important;
      }

      main#main [data-testid="login-button"] {
        display: inline-flex !important;
        visibility: visible !important;
        opacity: 1 !important;
      }
    `;
    document.documentElement.appendChild(style);
  }

  function norm(s) {
    return (s || '').replace(/\s+/g, ' ').trim();
  }

  function hide(el) {
    if (!el) return;
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
    var comboButtons = Array.from(document.querySelectorAll('button[role="combobox"]'))
      .filter(function(button) {
        var text = (button.textContent || '').replace(/\s+/g, ' ').trim();
        return text.length > 0;
      });

    if (comboButtons.length < 2) return;

    var row = closestCommonAncestor(comboButtons[0], comboButtons[1]);
    if (!row) return;

    row.setAttribute('data-llm-chatgpt-language-row', 'true');
  }

  function markTranslateLayout() {
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
      container.setAttribute('data-llm-chatgpt-container', 'true');
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

  function applyCleanup() {
    try {
      // 1. Left sidebar — exact ID match only
      document.querySelectorAll('#stage-slideover-sidebar').forEach(hide);

      // 2. Login area: collapse header, float login block, hide signup
      markLoginHeader();
      markLoginBlock();
      document.querySelectorAll('[data-testid="signup-button"]').forEach(hide);

      // 3. Bottom suggestion cards — hide only the matched interactive elements themselves
      function normStrict(s) { return (s||'').replace(/\s+/g,'').trim(); }
      const suggestionTexts = [
        'ビジネス用にする',
        '5 歳児にもわかるように説明して',
        '洗練されたビジネス向けのトーンにします。',
        'とてもやさしい言葉で書き直します。',
        'より自然な表現に',
        'ビジネス向けにする',
        '5歳でもわかるように',
        '学術向けにする',
        'Make it more natural',
        'Make it business-friendly',
        'Explain it like I am 5',
        'Make it academic'
      ];
      const strictTexts = suggestionTexts.map(normStrict);

      document.querySelectorAll('button, [role="button"], a').forEach((el) => {
        const text = norm(el.textContent);
        if (text.length > 0 && text.length < 120) {
          const st = normStrict(text);
          if (strictTexts.some((t) => st.includes(t))) {
            el.style.setProperty('display', 'none', 'important');
          }
        }
      });

      // 4. Footer tag only
      document.querySelectorAll('footer').forEach(hide);

      // 5. Hide heading "ChatGPT を使用して翻訳"
      document.querySelectorAll('h1').forEach((el) => {
        const text = norm(el.textContent);
        if (text.includes('ChatGPT') && (text.includes('翻訳') || text.includes('Translate'))) {
          hide(el);
        }
      });

      // 6. Mark translate layout with custom attributes for flex expansion
      markLoginHeader();
      markLoginBlock();
      markTranslateLayout();

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
        applyCleanup();
      }, 200);
    });

    observer.observe(document.documentElement, {
      childList: true,
      subtree: true,
      characterData: true,
      attributes: true
    });
  }
})();
"#;
    w.eval(js)
        .map_err(|e| format!("failed to apply ChatGPT Translate cleanup: {}", e))
}

fn schedule_chatgpt_translate_cleanup(app: tauri::AppHandle) {
    use tauri::Manager;
    tauri::async_runtime::spawn(async move {
        let delays = [0_u64, 500, 1500, 3000];
        for delay in delays {
            if delay > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }
            if let Some(w) = app.get_webview(CHATGPT_TRANSLATE_LABEL) {
                let _ = apply_chatgpt_translate_cleanup(&w);
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
        let _ = w.navigate(parsed_url);
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
      candidates.push(info);
    }
  }
  candidates.sort(function(a,b){return b.priority-a.priority||a.tag.localeCompare(b.tag);});
  candidates=candidates.slice(0,80);
  var result={location:location.href,candidateCount:candidates.length,candidates:candidates};
  window.location.hash='__llm_dbg='+encodeURIComponent(JSON.stringify(result));
})()"#;
    w.eval(js).map_err(|e| format!("eval failed: {e}"))?;
    // Wait for hash to propagate to Tauri's URL cache
    for i in 0..15 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let url = w.url().map_err(|e| e.to_string())?;
        if let Some(frag) = url.fragment() {
            if let Some(data) = frag.strip_prefix("__llm_dbg=") {
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
pub async fn set_chatgpt_translate_text(app: tauri::AppHandle, text: String) -> Result<(), String> {
    use tauri::Manager;
    println!("[Ctrl+C+C] set_chatgpt_translate_text, len={}", text.len());
    let w = app.get_webview(CHATGPT_TRANSLATE_LABEL).ok_or("webview not found")?;
    let text_json = serde_json::to_string(&text).map_err(|e| e.to_string())?;
    let js = format!(r#"(function(){{var t={text_json};function s(e,v){{var d=Object.getOwnPropertyDescriptor(Object.getPrototypeOf(e),'value');if(d&&d.set)d.set.call(e,v);else e.value=v;e.dispatchEvent(new Event('input',{{bubbles:true}}));e.dispatchEvent(new Event('change',{{bubbles:true}}));}}function f(){{var ts=document.querySelectorAll('textarea');for(var i=0;i<ts.length;i++){{if(!ts[i].hasAttribute('readonly'))return ts[i];}}return null;}}function trySet(){{var el=f();if(!el)return false;el.focus();s(el,t);return true;}}if(trySet())return;var n=0;var iv=setInterval(function(){{n++;if(trySet()||n>=40)clearInterval(iv);}},250);}})();"#);
    w.eval(&js).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_chatgpt_translate_languages(app: tauri::AppHandle, source_label: String, target_label: String) -> Result<(), String> {
    use tauri::Manager;
    let w = app.get_webview(CHATGPT_TRANSLATE_LABEL).ok_or("webview not found")?;
    let sl = serde_json::to_string(&source_label).map_err(|e| e.to_string())?;
    let tl = serde_json::to_string(&target_label).map_err(|e| e.to_string())?;
    let js = format!(r#"(function(){{var sl={sl};var tl={tl};function n(s){{return(s||'').replace(/\s+/g,' ').trim();}}function g(){{return Array.from(document.querySelectorAll('button[role="combobox"]'));}}function f(l){{var cs=Array.from(document.querySelectorAll('[role="option"],[role="menuitem"]'));return cs.find(function(el){{return n(el.textContent)===l;}});}}function s(b,l){{if(!b)return false;var c=n(b.textContent);if(c===l||c.indexOf(l)>=0)return true;b.click();setTimeout(function(){{var o=f(l);if(o)o.click();}},200);return false;}}function t(){{var bs=g();if(bs.length<2)return false;var so=s(bs[0],sl);var to=s(bs[1],tl);return so&&to;}}if(t())return;var c=0;var m=40;var iv=setInterval(function(){{c++;if(t()||c>=m)clearInterval(iv);}},250);}})();"#);
    w.eval(&js).map_err(|e| e.to_string())
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
