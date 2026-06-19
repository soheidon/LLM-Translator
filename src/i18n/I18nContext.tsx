import { createContext, useContext, useEffect, useState, useCallback, type ReactNode } from 'react';
import type { LanguageCode, Translations } from '../lang';
import { SUPPORTED_LANGUAGES, loadTranslations } from '../lang';

interface I18nContextType {
  language: LanguageCode;
  setLanguage: (lang: LanguageCode) => void;
  t: (path: string, params?: Record<string, string | number>) => string;
  translations: Translations | null;
  supportedLanguages: typeof SUPPORTED_LANGUAGES;
}

const I18nContext = createContext<I18nContextType | null>(null);

export function I18nProvider({ language, onChangeLanguage, children }: {
  language: LanguageCode;
  onChangeLanguage: (lang: LanguageCode) => void;
  children: ReactNode;
}) {
  const [translations, setTranslations] = useState<Translations | null>(null);

  useEffect(() => {
    let cancelled = false;
    loadTranslations(language).then(mod => {
      if (!cancelled) setTranslations(mod);
    });
    return () => { cancelled = true; };
  }, [language]);

  const t = useCallback((path: string, params?: Record<string, string | number>): string => {
    if (!translations) return path;
    const keys = path.split('.');
    let value: any = translations;
    for (const key of keys) {
      if (value == null) return path;
      value = value[key];
    }
    if (typeof value !== 'string') return path;
    if (params) {
      return value.replace(/\{(\w+)\}/g, (_, k) => String(params[k] ?? `{${k}}`));
    }
    return value;
  }, [translations]);

  return (
    <I18nContext.Provider value={{
      language,
      setLanguage: onChangeLanguage,
      t,
      translations,
      supportedLanguages: SUPPORTED_LANGUAGES,
    }}>
      {children}
    </I18nContext.Provider>
  );
}

export function useT(): I18nContextType {
  const ctx = useContext(I18nContext);
  if (!ctx) {
    // Fallback for usage outside provider (returns raw keys)
    return {
      language: 'en',
      setLanguage: () => {},
      t: (path: string) => path,
      translations: null,
      supportedLanguages: SUPPORTED_LANGUAGES,
    };
  }
  return ctx;
}
