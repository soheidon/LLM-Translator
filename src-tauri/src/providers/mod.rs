pub mod anthropic_compat;
pub mod openai_compat;
pub mod google_translate;
pub mod deepl;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    pub text: String,
    pub source_lang: Option<String>,
    pub target_lang: String,
    pub mode: String,
    pub tone: String,
    pub preset_id: Option<String>,
    pub provider: String,
    pub model: String,
    pub model_mode: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResponse {
    pub translated_text: String,
    pub detected_source_lang: Option<String>,
    pub provider: String,
    pub model: String,
    pub latency_ms: u128,
    pub token_usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub message_code: Option<String>,
    pub latency_ms: Option<u128>,
}

#[async_trait::async_trait]
pub trait TranslationProvider: Send + Sync {
    async fn translate(&self, request: TranslationRequest) -> Result<TranslationResponse, TranslationError>;
    async fn test_connection(&self) -> ConnectionTestResult;
}

#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error("API key not configured")]
    ApiKeyNotSet,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Timeout after {0}s")]
    Timeout(u64),
    #[error("API error ({status}): {message}")]
    ApiError { status: u16, message: String },
    #[error("Response parse error: {0}")]
    ParseError(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Text too long ({0} chars)")]
    TextTooLong(usize),
}
