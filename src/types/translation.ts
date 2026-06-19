export interface TranslationRequest {
  text: string;
  source_lang?: string;
  target_lang: string;
  mode: string;
  tone: string;
  preset_id?: string;
  provider_id?: string;
  model?: string;
  system_prompt?: string;
}

export interface TranslationResponse {
  translated_text: string;
  detected_source_lang: string | null;
  provider: string;
  model: string;
  latency_ms: number;
  token_usage: TokenUsage | null;
}

export interface TokenUsage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface HistoryEntry {
  id: string;
  created_at: string;
  source_text: string;
  translated_text: string;
  source_lang: string;
  target_lang: string;
  provider: string;
  model: string;
  mode: string;
  tone: string;
  preset_id: string;
  latency_ms: number;
}

export interface ModeInfo {
  id: string;
}

export interface LanguageInfo {
  code: string;
}

export interface TranslationState {
  sourceText: string;
  translatedText: string;
  isTranslating: boolean;
  error: string | null;
  lastResponse: TranslationResponse | null;
}
