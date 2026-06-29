use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub shortcut: ShortcutConfig,
    pub providers: Vec<ProviderConfig>,
    pub translation: TranslationConfig,
    pub history: HistoryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GeneralConfig {
    pub ui_language: String,
    #[serde(default)]
    pub start_minimized: bool,
    #[serde(default)]
    pub auto_launch: bool,
    pub always_on_top: bool,
    pub focus_on_translate: bool,
    pub close_on_escape: bool,
    pub close_on_outside_click: bool,
    pub notification_sound: bool,
    #[serde(default = "default_toolbar_mode")]
    pub google_translate_toolbar: String,
    #[serde(default)]
    pub google_translate_debug_tool: bool,
    #[serde(default)]
    pub chatgpt_translate_debug_tool: bool,
    #[serde(default)]
    pub chatgpt_translate_html_css_debug_tool: bool,
    #[serde(default = "default_true")]
    pub chatgpt_translate_hide_lp: bool,
}

fn default_true() -> bool {
    true
}

fn default_toolbar_mode() -> String {
    "hide_on_translate".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ShortcutConfig {
    pub primary: String,
    pub open_window: String,
    pub open_history: String,
    pub double_copy_enabled: bool,
    pub double_copy_threshold_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub api_type: ApiType,
    pub api_base_url: String,
    pub env_var: String,
    pub model: String,
    pub model_mapping: std::collections::HashMap<String, ModelRole>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub timeout_sec: u64,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiType {
    OpenAiCompat,
    AnthropicCompat,
    GoogleTranslateCloud,
    GoogleTranslateAppsScript,
    DeepL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ModelRole {
    pub model: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TranslationConfig {
    pub source_lang: String,
    pub target_lang: String,
    pub mode: String,
    pub tone: String,
    pub preset_id: String,
    pub preserve_line_breaks: bool,
    pub show_original: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HistoryConfig {
    pub enabled: bool,
    pub max_items: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        let providers = vec![
            ProviderConfig {
                id: "google_translate".to_string(),
                name: "Google Translate / Cloud".to_string(),
                api_type: ApiType::GoogleTranslateCloud,
                api_base_url: "https://translation.googleapis.com/language/translate/v2".to_string(),
                env_var: "GOOGLE_TRANSLATE_API_KEY".to_string(),
                model: "google-translate-v2".to_string(),
                model_mapping: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("default".into(), ModelRole { model: "google-translate-v2".into(), mode: "normal".into() });
                    m.insert("fast".into(), ModelRole { model: "google-translate-v2".into(), mode: "normal".into() });
                    m
                },
                temperature: 0.0,
                max_tokens: None,
                timeout_sec: 30,
                is_default: false,
            },
            ProviderConfig {
                id: "deepl_free".to_string(),
                name: "DeepL / Free".to_string(),
                api_type: ApiType::DeepL,
                api_base_url: "https://api-free.deepl.com".to_string(),
                env_var: "DEEPL_FREE_API_KEY".to_string(),
                model: "deepl".to_string(),
                model_mapping: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("default".into(), ModelRole { model: "deepl".into(), mode: "normal".into() });
                    m.insert("fast".into(), ModelRole { model: "deepl".into(), mode: "normal".into() });
                    m
                },
                temperature: 0.0,
                max_tokens: None,
                timeout_sec: 30,
                is_default: false,
            },
            ProviderConfig {
                id: "deepl_pro".to_string(),
                name: "DeepL / Pro".to_string(),
                api_type: ApiType::DeepL,
                api_base_url: "https://api.deepl.com".to_string(),
                env_var: "DEEPL_PRO_API_KEY".to_string(),
                model: "deepl".to_string(),
                model_mapping: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("default".into(), ModelRole { model: "deepl".into(), mode: "normal".into() });
                    m.insert("fast".into(), ModelRole { model: "deepl".into(), mode: "normal".into() });
                    m
                },
                temperature: 0.0,
                max_tokens: None,
                timeout_sec: 30,
                is_default: false,
            },
            ProviderConfig {
                id: "openai".to_string(),
                name: "OpenAI / ChatGPT".to_string(),
                api_type: ApiType::OpenAiCompat,
                api_base_url: "https://api.openai.com/v1".to_string(),
                env_var: "OPENAI_API_KEY".to_string(),
                model: "gpt-5.5".to_string(),
                model_mapping: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("default".into(), ModelRole { model: "gpt-5.5".into(), mode: "thinking".into() });
                    m.insert("fast".into(), ModelRole { model: "gpt-5.4-mini".into(), mode: "normal".into() });
                    m
                },
                temperature: 0.2,
                max_tokens: Some(4096),
                timeout_sec: 60,
                is_default: false,
            },
            ProviderConfig {
                id: "gemini".to_string(),
                name: "Gemini / Google".to_string(),
                api_type: ApiType::OpenAiCompat,
                api_base_url: "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
                env_var: "GEMINI_API_KEY".to_string(),
                model: "gemini-3.1-pro".to_string(),
                model_mapping: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("default".into(), ModelRole { model: "gemini-3.1-pro".into(), mode: "thinking".into() });
                    m.insert("fast".into(), ModelRole { model: "gemini-3.5-flash".into(), mode: "normal".into() });
                    m
                },
                temperature: 0.2,
                max_tokens: Some(4096),
                timeout_sec: 60,
                is_default: false,
            },
            ProviderConfig {
                id: "anthropic".to_string(),
                name: "Claude / Anthropic".to_string(),
                api_type: ApiType::AnthropicCompat,
                api_base_url: "https://api.anthropic.com".to_string(),
                env_var: "ANTHROPIC_API_KEY".to_string(),
                model: "claude-opus-4-8".to_string(),
                model_mapping: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("default".into(), ModelRole { model: "claude-opus-4-8".into(), mode: "thinking".into() });
                    m.insert("fast".into(), ModelRole { model: "claude-haiku-4-5".into(), mode: "normal".into() });
                    m
                },
                temperature: 0.2,
                max_tokens: Some(4096),
                timeout_sec: 60,
                is_default: false,
            },
            ProviderConfig {
                id: "ollama".to_string(),
                name: "Ollama / Local".to_string(),
                api_type: ApiType::OpenAiCompat,
                api_base_url: "http://127.0.0.1:11434/v1".to_string(),
                env_var: String::new(),
                model: String::new(),
                model_mapping: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("default".into(), ModelRole { model: String::new(), mode: "normal".into() });
                    m.insert("fast".into(), ModelRole { model: String::new(), mode: "normal".into() });
                    m
                },
                temperature: 0.2,
                max_tokens: Some(4096),
                timeout_sec: 120,
                is_default: false,
            },
            ProviderConfig {
                id: "mimo".to_string(),
                name: "MiMo / Xiaomi".to_string(),
                api_type: ApiType::OpenAiCompat,
                api_base_url: "https://api.xiaomimimo.com/v1".to_string(),
                env_var: "XIAOMI_API_KEY".to_string(),
                model: "mimo-v2.5-pro".to_string(),
                model_mapping: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("default".into(), ModelRole { model: "mimo-v2.5-pro".into(), mode: "thinking".into() });
                    m.insert("fast".into(), ModelRole { model: "mimo-v2.5".into(), mode: "normal".into() });
                    m
                },
                temperature: 0.2,
                max_tokens: Some(4096),
                timeout_sec: 60,
                is_default: true,
            },
            ProviderConfig {
                id: "deepseek".to_string(),
                name: "DeepSeek / DeepSeek".to_string(),
                api_type: ApiType::OpenAiCompat,
                api_base_url: "https://api.deepseek.com".to_string(),
                env_var: "DEEPSEEK_API_KEY".to_string(),
                model: "deepseek-v4-pro".to_string(),
                model_mapping: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("default".into(), ModelRole { model: "deepseek-v4-pro".into(), mode: "thinking".into() });
                    m.insert("fast".into(), ModelRole { model: "deepseek-v4-flash".into(), mode: "normal".into() });
                    m
                },
                temperature: 0.2,
                max_tokens: Some(4096),
                timeout_sec: 60,
                is_default: false,
            },
            ProviderConfig {
                id: "moonshot".to_string(),
                name: "Kimi / Moonshot".to_string(),
                api_type: ApiType::OpenAiCompat,
                api_base_url: "https://api.moonshot.ai/v1".to_string(),
                env_var: "MOONSHOT_API_KEY".to_string(),
                model: "kimi-k2.7-code".to_string(),
                model_mapping: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("default".into(), ModelRole { model: "kimi-k2.7-code".into(), mode: "thinking".into() });
                    m.insert("fast".into(), ModelRole { model: "kimi-k2.7".into(), mode: "normal".into() });
                    m
                },
                temperature: 0.2,
                max_tokens: Some(4096),
                timeout_sec: 60,
                is_default: false,
            },
            ProviderConfig {
                id: "qwen".to_string(),
                name: "Qwen / Alibaba".to_string(),
                api_type: ApiType::OpenAiCompat,
                api_base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
                env_var: "DASHSCOPE_API_KEY".to_string(),
                model: "qwen3.7-max".to_string(),
                model_mapping: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("default".into(), ModelRole { model: "qwen3.7-max".into(), mode: "thinking".into() });
                    m.insert("fast".into(), ModelRole { model: "qwen3.6-flash".into(), mode: "normal".into() });
                    m
                },
                temperature: 0.2,
                max_tokens: Some(4096),
                timeout_sec: 60,
                is_default: false,
            },
            ProviderConfig {
                id: "minimax".to_string(),
                name: "MiniMAX / MiniMAX".to_string(),
                api_type: ApiType::OpenAiCompat,
                api_base_url: "https://api.minimax.io/v1".to_string(),
                env_var: "MINIMAX_API_KEY".to_string(),
                model: "MiniMax-M2.7".to_string(),
                model_mapping: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("default".into(), ModelRole { model: "MiniMax-M2.7".into(), mode: "thinking".into() });
                    m.insert("fast".into(), ModelRole { model: "MiniMax-M2.7".into(), mode: "normal".into() });
                    m
                },
                temperature: 0.2,
                max_tokens: Some(4096),
                timeout_sec: 60,
                is_default: false,
            },
            ProviderConfig {
                id: "google_translate_apps_script".to_string(),
                name: "Google Translate / Apps Script".to_string(),
                api_type: ApiType::GoogleTranslateAppsScript,
                api_base_url: String::new(),
                env_var: "GOOGLE_TRANSLATE_SCRIPT_URL".to_string(),
                model: "google-apps-script".to_string(),
                model_mapping: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("default".into(), ModelRole { model: "google-apps-script".into(), mode: "normal".into() });
                    m.insert("fast".into(), ModelRole { model: "google-apps-script".into(), mode: "normal".into() });
                    m
                },
                temperature: 0.0,
                max_tokens: None,
                timeout_sec: 30,
                is_default: false,
            },
        ];

        Self {
            general: GeneralConfig {
                ui_language: "en".to_string(),
                start_minimized: false,
                auto_launch: false,
                always_on_top: true,
                focus_on_translate: true,
                close_on_escape: true,
                close_on_outside_click: false,
                notification_sound: false,
                google_translate_toolbar: "hide_on_translate".to_string(),
                google_translate_debug_tool: false,
                chatgpt_translate_debug_tool: false,
                chatgpt_translate_html_css_debug_tool: false,
                chatgpt_translate_hide_lp: true,
            },
            shortcut: ShortcutConfig {
                primary: "Ctrl+Shift+C".to_string(),
                open_window: "Ctrl+Alt+T".to_string(),
                open_history: "Ctrl+Alt+H".to_string(),
                double_copy_enabled: true,
                double_copy_threshold_ms: 400,
            },
            providers,
            translation: TranslationConfig {
                source_lang: "auto".to_string(),
                target_lang: "ja".to_string(),
                mode: "news".to_string(),
                tone: "auto".to_string(),
                preset_id: "news".to_string(),
                preserve_line_breaks: true,
                show_original: true,
            },
            history: HistoryConfig {
                enabled: false,
                max_items: 100,
            },
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("LLMTranslator")
}

pub fn config_path() -> PathBuf {
    config_dir().join("settings.json")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    let mut config = if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => AppConfig::default(),
        }
    } else {
        AppConfig::default()
    };

    // Merge missing default providers into existing config
    let defaults = AppConfig::default();
    let mut existing_ids: std::collections::HashSet<String> =
        config.providers.iter().map(|p| p.id.clone()).collect();
    for p in &defaults.providers {
        if !existing_ids.contains(&p.id) {
            config.providers.push(p.clone());
            existing_ids.insert(p.id.clone());
        }
    }

    // Migrate: split DEEPL_API_KEY into DEEPL_FREE_API_KEY / DEEPL_PRO_API_KEY
    for p in &mut config.providers {
        if p.id == "deepl_free" && p.env_var == "DEEPL_API_KEY" {
            p.env_var = "DEEPL_FREE_API_KEY".to_string();
        }
        if p.id == "deepl_pro" && p.env_var == "DEEPL_API_KEY" {
            p.env_var = "DEEPL_PRO_API_KEY".to_string();
        }
    }

    // Reorder providers to match default order (Google Translate → DeepL → LLM → Local)
    let default_order: Vec<&str> = defaults.providers.iter().map(|p| p.id.as_str()).collect();
    config.providers.sort_by_key(|p| {
        default_order
            .iter()
            .position(|id| *id == p.id)
            .unwrap_or(usize::MAX)
    });

    let _ = save_config(&config);
    config
}

pub fn save_config(config: &AppConfig) -> anyhow::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let path = config_path();
    let json = serde_json::to_string_pretty(config)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Read an environment variable, falling back to Windows registry (HKCU\Environment).
/// `setx` writes to the registry but does NOT update the current process tree.
/// Without this, env vars set via setx only appear after a full logout/login.
pub fn read_env_var(name: &str) -> Result<String, std::env::VarError> {
    // 1) Try process environment (catches vars set since process start or inherited)
    if let Ok(val) = std::env::var(name) {
        if !val.is_empty() {
            return Ok(val);
        }
    }

    // 2) Fallback: read from Windows registry (where setx writes)
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey("Environment")
        {
            if let Ok(val) = hkcu.get_value::<String, _>(name) {
                if !val.is_empty() {
                    // Also set it in this process so subsequent reads are fast
                    std::env::set_var(name, &val);
                    return Ok(val);
                }
            }
        }
    }

    Err(std::env::VarError::NotPresent)
}

