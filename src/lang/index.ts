export type LanguageCode = 'en' | 'ja' | 'zh-CN' | 'zh-TW' | 'ko' | 'fr' | 'de' | 'es' | 'pt' | 'ru' | 'it';

export interface Translations {
  app: { title: string; version: string };
  titlebar: Record<string, string>;
  main: Record<string, string>;
  settings: {
    sidebar: Record<string, string>;
    general: Record<string, string>;
    api: Record<string, string>;
    presets: Record<string, string>;
    history: Record<string, string>;
  };
  history_panel: Record<string, string>;
  status_bar: Record<string, string>;
  languages: Record<string, string>;
  modes: Record<string, string>;
  tones: Record<string, string>;
  preset_names: Record<string, string>;
  preset_descs: Record<string, string>;
  preset_tags: Record<string, string>;
  errors: Record<string, string>;
  lang_selector: Record<string, string>;
}

export const SUPPORTED_LANGUAGES: { code: LanguageCode; nativeName: string }[] = [
  { code: 'en', nativeName: 'English' },
  { code: 'ja', nativeName: '日本語' },
  { code: 'zh-CN', nativeName: '中文(简体)' },
  { code: 'zh-TW', nativeName: '中文(繁體)' },
  { code: 'ko', nativeName: '한국어' },
  { code: 'fr', nativeName: 'Français' },
  { code: 'de', nativeName: 'Deutsch' },
  { code: 'es', nativeName: 'Español' },
  { code: 'pt', nativeName: 'Português' },
  { code: 'ru', nativeName: 'Русский' },
  { code: 'it', nativeName: 'Italiano' },
];

const translationsCache: Record<string, Translations> = {};

export async function loadTranslations(lang: LanguageCode): Promise<Translations> {
  if (translationsCache[lang]) return translationsCache[lang];
  const mod = await import(`./${lang}.json`);
  translationsCache[lang] = mod.default ?? mod;
  return translationsCache[lang];
}
