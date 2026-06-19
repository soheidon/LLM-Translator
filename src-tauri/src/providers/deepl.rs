use super::*;
use crate::config::ProviderConfig;
use reqwest::Client;
use std::time::Instant;

fn api_key_from_env(env_var: &str) -> Result<String, ()> {
    crate::config::read_env_var(env_var).map_err(|_| ())
}

/// Convert internal language code to DeepL format (uppercase region codes)
fn to_deepl_lang(code: &str) -> String {
    match code {
        "en" => "EN".into(),
        "ja" => "JA".into(),
        "zh" => "ZH".into(),
        "ko" => "KO".into(),
        "fr" => "FR".into(),
        "de" => "DE".into(),
        "es" => "ES".into(),
        "pt" => "PT".into(),
        "ru" => "RU".into(),
        "it" => "IT".into(),
        other => {
            // Handle regional codes like en-US, pt-BR
            if let Some((lang, region)) = other.split_once('-') {
                format!("{}-{}", lang.to_uppercase(), region.to_uppercase())
            } else {
                other.to_uppercase()
            }
        }
    }
}

pub struct DeeplProvider {
    config: ProviderConfig,
    client: Client,
}

impl DeeplProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_sec))
            .build()
            .unwrap_or_default();
        Self { config, client }
    }
}

#[async_trait::async_trait]
impl TranslationProvider for DeeplProvider {
    async fn translate(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslationResponse, TranslationError> {
        let api_key = api_key_from_env(&self.config.env_var)
            .map_err(|_| TranslationError::ApiKeyNotSet)?;

        let url = format!(
            "{}/v2/translate",
            self.config.api_base_url.trim_end_matches('/')
        );

        let mut body = serde_json::json!({
            "text": [request.text],
            "target_lang": to_deepl_lang(&request.target_lang),
        });

        let source_is_auto = request.source_lang.as_deref() == Some("auto")
            || request.source_lang.is_none();
        if !source_is_auto {
            if let Some(ref sl) = request.source_lang {
                body["source_lang"] = serde_json::Value::String(to_deepl_lang(sl));
            }
        }

        let start = Instant::now();

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("DeepL-Auth-Key {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    TranslationError::Timeout(self.config.timeout_sec)
                } else {
                    TranslationError::NetworkError(e.to_string())
                }
            })?;

        let latency = start.elapsed().as_millis();
        let status = resp.status();

        let resp_text = resp
            .text()
            .await
            .map_err(|e| TranslationError::NetworkError(e.to_string()))?;

        if !status.is_success() {
            return Err(TranslationError::ApiError {
                status: status.as_u16(),
                message: resp_text,
            });
        }

        let json: serde_json::Value = serde_json::from_str(&resp_text)
            .map_err(|e| TranslationError::ParseError(e.to_string()))?;

        let translated_text = json["translations"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let detected = json["translations"][0]["detected_source_language"]
            .as_str()
            .map(|s| s.to_string());

        Ok(TranslationResponse {
            translated_text,
            detected_source_lang: detected,
            provider: self.config.name.clone(),
            model: self.config.model.clone(),
            latency_ms: latency,
            token_usage: None,
        })
    }

    async fn test_connection(&self) -> ConnectionTestResult {
        let api_key = match api_key_from_env(&self.config.env_var) {
            Ok(k) => k,
            Err(_) => {
                return ConnectionTestResult {
                    success: false,
                    message: format!("Environment variable {} is not set", self.config.env_var),
                    message_code: Some("env_var_not_set".into()),
                    latency_ms: None,
                };
            }
        };

        let url = format!(
            "{}/v2/translate",
            self.config.api_base_url.trim_end_matches('/')
        );

        let body = serde_json::json!({
            "text": ["Hello"],
            "target_lang": "JA",
        });

        let start = Instant::now();

        match self
            .client
            .post(&url)
            .header("Authorization", format!("DeepL-Auth-Key {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(resp) => {
                let latency = start.elapsed().as_millis();
                let status = resp.status();
                if status.is_success() {
                    // Try to get usage after successful test
                    let usage_msg = self.fetch_usage(&api_key).await;

                    let has_usage = usage_msg.is_some();
                    let msg = if let Some(u) = usage_msg {
                        format!("Connection successful — {}", u)
                    } else {
                        "Connection successful".to_string()
                    };
                    ConnectionTestResult {
                        success: true,
                        message: msg,
                        message_code: if has_usage {
                            Some("connection_success_usage".into())
                        } else {
                            Some("connection_success".into())
                        },
                        latency_ms: Some(latency),
                    }
                } else {
                    let body = resp.text().await.unwrap_or_default();
                    ConnectionTestResult {
                        success: false,
                        message: format!("HTTP {}: {}", status.as_u16(), body),
                        message_code: Some("connection_failed".into()),
                        latency_ms: Some(latency),
                    }
                }
            }
            Err(e) => ConnectionTestResult {
                success: false,
                message: e.to_string(),
                message_code: None,
                latency_ms: None,
            },
        }
    }
}

impl DeeplProvider {
    async fn fetch_usage(&self, api_key: &str) -> Option<String> {
        let url = format!(
            "{}/v2/usage",
            self.config.api_base_url.trim_end_matches('/')
        );

        match self
            .client
            .get(&url)
            .header("Authorization", format!("DeepL-Auth-Key {}", api_key))
            .send()
            .await
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    return None;
                }
                let text = resp.text().await.ok()?;
                let json: serde_json::Value = serde_json::from_str(&text).ok()?;
                let count = json["character_count"].as_u64()?;
                let limit = json["character_limit"].as_u64()?;
                Some(format!("usage: {} / {} characters", count, limit))
            }
            Err(_) => None,
        }
    }
}
