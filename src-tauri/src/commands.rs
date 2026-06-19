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
    window.close().map_err(|e| e.to_string())
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
pub async fn set_always_on_top(window: tauri::Window, always_on_top: bool) -> Result<(), String> {
    window.set_always_on_top(always_on_top).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_drag(window: tauri::Window) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
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
