use crate::config::{AppConfig, ApiType, ProviderConfig};
use crate::providers::anthropic_compat::AnthropicCompatProvider;
use crate::providers::openai_compat::OpenAiCompatProvider;
use crate::providers::google_translate::GoogleTranslateProvider;
use crate::providers::deepl::DeeplProvider;
use crate::providers::{TranslationError, TranslationProvider, TranslationRequest, TranslationResponse};
use std::sync::Arc;

pub fn build_provider(config: &ProviderConfig) -> Arc<dyn TranslationProvider> {
    match config.api_type {
        ApiType::OpenAiCompat => Arc::new(OpenAiCompatProvider::new(config.clone())),
        ApiType::AnthropicCompat => Arc::new(AnthropicCompatProvider::new(config.clone())),
        ApiType::GoogleTranslateCloud => Arc::new(GoogleTranslateProvider::new(config.clone(), true)),
        ApiType::GoogleTranslateAppsScript => Arc::new(GoogleTranslateProvider::new(config.clone(), false)),
        ApiType::DeepL => Arc::new(DeeplProvider::new(config.clone())),
    }
}

pub fn get_default_provider(app_config: &AppConfig) -> Option<(ProviderConfig, Arc<dyn TranslationProvider>)> {
    app_config
        .providers
        .iter()
        .find(|p| p.is_default)
        .map(|p| (p.clone(), build_provider(p)))
}

pub fn get_provider_by_id(app_config: &AppConfig, id: &str) -> Option<(ProviderConfig, Arc<dyn TranslationProvider>)> {
    app_config
        .providers
        .iter()
        .find(|p| p.id == id)
        .map(|p| (p.clone(), build_provider(p)))
}

pub async fn translate_with_provider(
    provider: &dyn TranslationProvider,
    _provider_config: &ProviderConfig,
    request: TranslationRequest,
) -> Result<TranslationResponse, TranslationError> {
    provider.translate(request).await
}

pub async fn test_provider_connection(
    provider_config: &ProviderConfig,
) -> crate::providers::ConnectionTestResult {
    let provider = build_provider(provider_config);
    provider.test_connection().await
}
