use super::*;
use crate::config::ProviderConfig;
use reqwest::Client;
use std::time::Instant;

fn api_key_from_env(env_var: &str) -> Result<String, ()> {
    crate::config::read_env_var(env_var).map_err(|_| ())
}

pub struct AnthropicCompatProvider {
    config: ProviderConfig,
    client: Client,
}

impl AnthropicCompatProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_sec))
            .build()
            .unwrap_or_default();
        Self { config, client }
    }
}

#[async_trait::async_trait]
impl TranslationProvider for AnthropicCompatProvider {
    async fn translate(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslationResponse, TranslationError> {
        let api_key = api_key_from_env(&self.config.env_var)
            .map_err(|_| TranslationError::ApiKeyNotSet)?;

        let url = format!(
            "{}/messages",
            self.config.api_base_url.trim_end_matches('/')
        );

        let system_prompt = request
            .system_prompt
            .clone()
            .unwrap_or_else(|| {
                super::openai_compat::build_default_prompt_for_anthropic(&request)
            });

        let body = serde_json::json!({
            "model": request.model,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "system": system_prompt,
            "messages": [
                { "role": "user", "content": request.text }
            ]
        });

        let start = Instant::now();

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
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

        if status == 429 {
            return Err(TranslationError::RateLimited);
        }

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

        let translated_text = json["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = json.get("usage").map(|u| TokenUsage {
            prompt_tokens: u["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: (u["input_tokens"].as_u64().unwrap_or(0)
                + u["output_tokens"].as_u64().unwrap_or(0)) as u32,
        });

        Ok(TranslationResponse {
            translated_text,
            detected_source_lang: None,
            provider: self.config.name.clone(),
            model: request.model,
            latency_ms: latency,
            token_usage: usage,
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
            "{}/messages",
            self.config.api_base_url.trim_end_matches('/')
        );

        let body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": 16,
            "messages": [{ "role": "user", "content": "Hello" }]
        });

        let start = Instant::now();

        match self
            .client
            .post(&url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
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
                    ConnectionTestResult {
                        success: false,
                        message: format!("HTTP {}", status.as_u16()),
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
