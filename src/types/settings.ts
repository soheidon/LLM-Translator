import type { ProviderConfig } from './provider';

export interface AppConfig {
  general: GeneralConfig;
  shortcut: ShortcutConfig;
  providers: ProviderConfig[];
  translation: TranslationConfig;
  history: HistoryConfig;
}

export interface GeneralConfig {
  ui_language: string;
  start_minimized: boolean;
  auto_launch: boolean;
  always_on_top: boolean;
  focus_on_translate: boolean;
  close_on_escape: boolean;
  close_on_outside_click: boolean;
  notification_sound: boolean;
  google_translate_toolbar: string;
  google_translate_debug_tool: boolean;
  chatgpt_translate_debug_tool: boolean;
  chatgpt_translate_html_css_debug_tool: boolean;
  chatgpt_translate_console_log_enabled: boolean;
  chatgpt_translate_hide_lp: boolean;
}

export interface ShortcutConfig {
  primary: string;
  open_window: string;
  open_history: string;
  double_copy_enabled: boolean;
  double_copy_threshold_ms: number;
}

export interface TranslationConfig {
  source_lang: string;
  target_lang: string;
  mode: string;
  tone: string;
  preset_id: string;
  preserve_line_breaks: boolean;
  show_original: boolean;
}

export interface HistoryConfig {
  enabled: boolean;
  max_items: number;
}
