export type ApiType = 'OpenAiCompat' | 'AnthropicCompat' | 'GoogleTranslateCloud' | 'GoogleTranslateAppsScript' | 'DeepL';

export interface ProviderConfig {
  id: string;
  name: string;
  api_type: ApiType;
  api_base_url: string;
  env_var: string;
  model: string;
  model_mapping: Record<string, ModelRole>;
  temperature: number;
  max_tokens: number | null;
  timeout_sec: number;
  is_default: boolean;
}

export interface ModelRole {
  model: string;
  mode: string;
}

export interface ConnectionTestResult {
  success: boolean;
  message: string;
  message_code: string | null;
  latency_ms: number | null;
}

export interface EnvVarStatus {
  env_var: string;
  is_set: boolean;
  value_length: number;
}

export interface OllamaModel {
  name: string;
  size: string;
}
