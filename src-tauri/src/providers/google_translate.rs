use super::*;
use crate::config::ProviderConfig;
use reqwest::Client;
use std::time::Instant;

fn api_key_from_env(env_var: &str) -> Result<String, ()> {
    crate::config::read_env_var(env_var).map_err(|_| ())
}

pub struct GoogleTranslateProvider {
    config: ProviderConfig,
    client: Client,
    is_cloud: bool,
}

impl GoogleTranslateProvider {
    pub fn new(config: ProviderConfig, is_cloud: bool) -> Self {
        let timeout_secs = if is_cloud { config.timeout_sec } else { 30 };
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_default();
        Self { config, client, is_cloud }
    }

    fn html_decode(s: &str) -> String {
        s.replace("&#39;", "'")
            .replace("&quot;", "\"")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
    }
}

#[async_trait::async_trait]
impl TranslationProvider for GoogleTranslateProvider {
    async fn translate(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslationResponse, TranslationError> {
        if self.is_cloud {
            self.translate_cloud(request).await
        } else {
            self.translate_apps_script(request).await
        }
    }

    async fn test_connection(&self) -> ConnectionTestResult {
        if self.is_cloud {
            self.test_cloud().await
        } else {
            self.test_apps_script().await
        }
    }
}

// ── Cloud translation API ──
impl GoogleTranslateProvider {
    async fn translate_cloud(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslationResponse, TranslationError> {
        let api_key = api_key_from_env(&self.config.env_var)
            .map_err(|_| TranslationError::ApiKeyNotSet)?;

        let url = format!(
            "{}?key={}",
            self.config.api_base_url.trim_end_matches('/'),
            api_key
        );

        let mut body = serde_json::json!({
            "q": request.text,
            "target": request.target_lang,
            "format": "text",
        });
        if request.source_lang.as_deref() != Some("auto") && request.source_lang.is_some() {
            body["source"] = serde_json::Value::String(
                request.source_lang.clone().unwrap_or_default()
            );
        }

        let start = Instant::now();

        let resp = self
            .client
            .post(&url)
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

        let translated_text = json["data"]["translations"][0]["translatedText"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let translated_text = Self::html_decode(&translated_text);

        let detected = json["data"]["translations"][0]["detectedSourceLanguage"]
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

    async fn test_cloud(&self) -> ConnectionTestResult {
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
            "{}?key={}",
            self.config.api_base_url.trim_end_matches('/'),
            api_key
        );

        let body = serde_json::json!({
            "q": "Hello",
            "target": "ja",
            "format": "text",
        });

        let start = Instant::now();

        match self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(resp) => {
                let latency = start.elapsed().as_millis();
                let status = resp.status();
                if status.is_success() {
                    ConnectionTestResult {
                        success: true,
                        message: "Connection successful".to_string(),
                        message_code: Some("connection_success".into()),
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

// ── Apps Script Web App ──
impl GoogleTranslateProvider {
    async fn translate_apps_script(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslationResponse, TranslationError> {
        let script_url = api_key_from_env(&self.config.env_var)
            .map_err(|_| TranslationError::ApiKeyNotSet)?;

        let source = if request.source_lang.as_deref() == Some("auto") {
            String::new()
        } else {
            request.source_lang.clone().unwrap_or_default()
        };

        let body = serde_json::json!({
            "text": request.text,
            "source": source,
            "target": request.target_lang,
        });

        let start = Instant::now();

        let resp = self
            .client
            .post(&script_url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    TranslationError::Timeout(30)
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

        let translated_text = json["translatedText"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(TranslationResponse {
            translated_text,
            detected_source_lang: None,
            provider: self.config.name.clone(),
            model: self.config.model.clone(),
            latency_ms: latency,
            token_usage: None,
        })
    }

    async fn test_apps_script(&self) -> ConnectionTestResult {
        let script_url = match api_key_from_env(&self.config.env_var) {
            Ok(url) => url,
            Err(_) => {
                return ConnectionTestResult {
                    success: false,
                    message: format!("Environment variable {} is not set", self.config.env_var),
                    message_code: Some("env_var_not_set".into()),
                    latency_ms: None,
                };
            }
        };

        let body = serde_json::json!({
            "text": "Hello",
            "source": "",
            "target": "ja",
        });

        let start = Instant::now();

        match self
            .client
            .post(&script_url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(resp) => {
                let latency = start.elapsed().as_millis();
                let status = resp.status();
                if status.is_success() {
                    ConnectionTestResult {
                        success: true,
                        message: "Connection successful".to_string(),
                        message_code: Some("connection_success".into()),
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
