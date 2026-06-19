use super::*;
use crate::config::ProviderConfig;
use reqwest::Client;
use std::time::Instant;

fn api_key_from_env(env_var: &str) -> Result<String, ()> {
    crate::config::read_env_var(env_var).map_err(|_| ())
}

pub struct OpenAiCompatProvider {
    config: ProviderConfig,
    client: Client,
}

impl OpenAiCompatProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_sec))
            .build()
            .unwrap_or_default();
        Self { config, client }
    }

    fn build_messages(&self, request: &TranslationRequest) -> Vec<serde_json::Value> {
        let system_prompt = request
            .system_prompt
            .clone()
            .unwrap_or_else(|| build_default_prompt(request));

        vec![
            serde_json::json!({ "role": "system", "content": system_prompt }),
            serde_json::json!({ "role": "user", "content": request.text }),
        ]
    }
}

#[async_trait::async_trait]
impl TranslationProvider for OpenAiCompatProvider {
    async fn translate(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslationResponse, TranslationError> {
        let needs_auth = self.config.id != "ollama";
        let api_key = if needs_auth {
            api_key_from_env(&self.config.env_var)
                .map_err(|_| TranslationError::ApiKeyNotSet)?
        } else {
            String::new()
        };

        let url = format!("{}/chat/completions", self.config.api_base_url.trim_end_matches('/'));
        let messages = self.build_messages(&request);

        let body = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "temperature": request.temperature,
            "max_tokens": request.max_tokens.unwrap_or(4096),
        });

        let start = Instant::now();

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body);
        if needs_auth {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        let resp = req.send().await.map_err(|e| {
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

        let translated_text = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = json.get("usage").map(|u| TokenUsage {
            prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
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
        let needs_auth = self.config.id != "ollama";
        let api_key = if needs_auth {
            match api_key_from_env(&self.config.env_var) {
                Ok(k) => k,
                Err(_) => {
                    return ConnectionTestResult {
                        success: false,
                        message: format!("Environment variable {} is not set", self.config.env_var),
                        message_code: Some("env_var_not_set".into()),
                        latency_ms: None,
                    };
                }
            }
        } else {
            String::new()
        };

        let url = format!(
            "{}/models",
            self.config.api_base_url.trim_end_matches('/')
        );

        let start = Instant::now();

        let mut req = self.client.get(&url);
        if needs_auth {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        match req.send().await
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

pub fn build_default_prompt_for_anthropic(req: &TranslationRequest) -> String {
    build_default_prompt(req)
}

fn build_default_prompt(req: &TranslationRequest) -> String {
    let lang_name = match req.target_lang.as_str() {
        "ja" => "Japanese",
        "en" => "English",
        "zh" => "Chinese",
        "ko" => "Korean",
        "fr" => "French",
        "de" => "German",
        "es" => "Spanish",
        _ => &req.target_lang,
    };

    // Tone instruction for Japanese target language
    let tone_rule = if req.target_lang == "ja" {
        match req.tone.as_str() {
            "plain" => "\n- Use 常体 (だ・である調) throughout. Avoid polite forms (です・ます).\n",
            "polite" => "\n- Use 敬体 (です・ます調) throughout. Maintain polite, formal register.\n",
            _ => "", // auto: let preset decide
        }
    } else {
        ""
    };

    match req.mode.as_str() {
        "news" => format!(
            "You are a professional news translator.\n\
             Translate the following text into natural, readable {}.\n\n\
             Rules:\n\
             - Do not omit any meaning from the original\n\
             - Keep proper nouns accurate; add original in parentheses when helpful\n\
             - Preserve numbers, dates, and organization names exactly\n\
             - Use natural newspaper-style prose, not headline style\n\
             - Do not add commentary or summary\n\
             - Output ONLY the translation\n{}\
             \nText:\n{{{{text}}}}",
            lang_name, tone_rule
        ),
        "academic" => format!(
            "You are an academic paper translator.\n\
             Translate the following text into {}, preserving technical terminology.\n\n\
             Rules:\n\
             - Use formal academic style\n\
             - Do not mistranslate technical terms\n\
             - Preserve statistics, abbreviations, scale names, and proper nouns exactly\n\
             - Do not add explanations not in the original\n\
             - Output ONLY the translation\n{}\
             \nText:\n{{{{text}}}}",
            lang_name, tone_rule
        ),
        "email" => format!(
            "You are a professional email translator.\n\
             Translate the following email into natural {}.\n\n\
             Rules:\n\
             - Maintain the appropriate formality level\n\
             - Preserve names and email addresses exactly\n\
             - Keep the original tone\n\
             - Output ONLY the translation\n{}\
             \nText:\n{{{{text}}}}",
            lang_name, tone_rule
        ),
        "technical" => format!(
            "You are a technical documentation translator.\n\
             Translate the following technical text into {}.\n\n\
             Rules:\n\
             - Preserve all technical terms and code references\n\
             - Keep formatting (lists, code blocks) intact\n\
             - Use standard {} technical terminology\n\
             - Output ONLY the translation\n{}\
             \nText:\n{{{{text}}}}",
            lang_name, lang_name, tone_rule
        ),
        "subtitle" => format!(
            "You are a subtitle translator.\n\
             Translate the following subtitle text into {}.\n\n\
             Rules:\n\
             - Keep each line short and natural for subtitles\n\
             - Preserve line breaks\n\
             - Maintain conversational tone\n\
             - Output ONLY the translation\n{}\
             \nText:\n{{{{text}}}}",
            lang_name, tone_rule
        ),
        "natural" => format!(
            "Translate the following text into natural {}.\n\
             Output ONLY the translation.\n{}\
             \nText:\n{{{{text}}}}",
            lang_name, tone_rule
        ),
        "literal" => format!(
            "Translate the following text into {} literally, word for word as much as possible.\n\
             Output ONLY the translation.\n{}\n\
             \nText:\n{{{{text}}}}",
            lang_name, tone_rule
        ),
        "summary" => format!(
            "Translate the following text into {} and add a brief summary at the end.\n\
             Format: Translation first, then \"---\\nSummary: ...\"\n{}\n\
             \nText:\n{{{{text}}}}",
            lang_name, tone_rule
        ),
        _ => format!(
            "You are a professional translator.\n\
             Translate the following text into {}.\n\
             Output ONLY the translation.\n{}\n\
             \nText:\n{{{{text}}}}",
            lang_name, tone_rule
        ),
    }
    .replace("{{text}}", &req.text)
}
