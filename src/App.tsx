import { useState, useEffect, useCallback, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { MainTranslate } from './components/MainTranslate';
import { SettingsPanel } from './components/SettingsPanel';
import { HistoryPanel } from './components/HistoryPanel';
import { StatusBar } from './components/StatusBar';
import { useSettings } from './hooks/useSettings';
import { useTranslationState } from './hooks/useTranslationState';
import { I18nProvider, useT } from './i18n/I18nContext';
import type { LanguageCode } from './lang';
import type { HistoryEntry, ModeInfo } from './types/translation';
import './styles/app.css';

type View = 'translate' | 'settings' | 'history';

export default function App() {
  const [view, setView] = useState<View>('translate');
  const { config, loading, updateGeneral, updateTranslation, updateShortcut, updateHistory, saveProvider } = useSettings();
  const translation = useTranslationState();
  const [modes, setModes] = useState<ModeInfo[]>([]);
  const [mode, setMode] = useState('news');
  const [tone, setTone] = useState('auto');
  const [activeProviderId, setActiveProviderId] = useState<string | null>(null);
  const [enabledProviderIds, setEnabledProviderIds] = useState<Set<string>>(new Set());

  useEffect(() => {
    invoke<ModeInfo[]>('get_modes').then(setModes).catch(console.error);
  }, []);

  // Check which providers have env vars set (ollama always available)
  useEffect(() => {
    if (!config) return;
    config.providers.forEach(p => {
      if (p.id === 'ollama') {
        setEnabledProviderIds(prev => new Set(prev).add('ollama'));
        return;
      }
      invoke<{ is_set: boolean }>('check_env_var', { envVar: p.env_var })
        .then(s => {
          if (s.is_set) setEnabledProviderIds(prev => new Set(prev).add(p.id));
        })
        .catch(() => {});
    });
  }, [config]);

  useEffect(() => {
    if (config) {
      setMode(config.translation.mode);
      setTone(config.translation.tone);
    }
  }, [config?.translation.mode, config?.translation.tone]);

  const availableProviders = config?.providers.filter(p => enabledProviderIds.has(p.id)) ?? [];

  // Resolve model from active provider config
  const resolveModel = useCallback((pid: string | null): string | undefined => {
    if (!pid || !config) return undefined;
    const p = config.providers.find(p => p.id === pid);
    if (!p) return undefined;
    return p.model || p.model_mapping?.default?.model || undefined;
  }, [config]);

  // Stable refs for values used in long-lived effects (avoid re-registration on every render)
  const translateRef = useRef(translation);
  translateRef.current = translation;
  const configRef = useRef(config);
  configRef.current = config;
  const activeProviderIdRef = useRef(activeProviderId);
  activeProviderIdRef.current = activeProviderId;
  const resolveModelRef = useRef(resolveModel);
  resolveModelRef.current = resolveModel;

  // Listen for Tauri events (shortcut triggers from Rust)
  useEffect(() => {
    const unlistenTranslate = listen('trigger-translate', () => {
      setView('translate');
      const t = translateRef.current;
      const c = configRef.current;
      if (c?.general.focus_on_translate) {
        invoke('focus_window').catch(() => {});
      }
      t.translateFromClipboard({
        source_lang: c?.translation.source_lang,
        target_lang: c?.translation.target_lang || 'ja',
        mode: c?.translation.mode || 'news',
        tone: c?.translation.tone || 'auto',
        preset_id: c?.translation.preset_id,
        provider_id: activeProviderIdRef.current || undefined,
        model: resolveModelRef.current(activeProviderIdRef.current),
      });
    });

    const unlistenHistory = listen('show-history', () => {
      setView('history');
    });

    return () => {
      unlistenTranslate.then(fn => fn());
      unlistenHistory.then(fn => fn());
    };
  }, []);

  // Clipboard polling: external-app copy detection (debounced 600ms)
  // Double-Ctrl+C is now handled globally by the Rust keyboard hook
  const lastTranslatedRef = useRef('');
  const isTranslatingRef = useRef(false);
  useEffect(() => { isTranslatingRef.current = translation.isTranslating; }, [translation.isTranslating]);
  useEffect(() => {
    let lastSeen = '';
    let debounce: ReturnType<typeof setTimeout> | null = null;

    const interval = setInterval(async () => {
      try {
        // Skip clipboard polling when double-Ctrl+C hook is active
        if (configRef.current?.shortcut.double_copy_enabled) return;

        const text = await readText();
        if (!text || !text.trim()) return;
        const trimmed = text.trim();
        if (trimmed === lastSeen) return;

        lastSeen = trimmed;
        if (debounce) clearTimeout(debounce);
        debounce = setTimeout(() => {
          // Skip if already translating or this text was just translated
          if (isTranslatingRef.current) return;
          if (trimmed === lastTranslatedRef.current) return;

          lastTranslatedRef.current = trimmed;
          const p = configRef.current;
          if (p?.general.focus_on_translate) {
            invoke('focus_window').catch(() => {});
          }
          const t = translateRef.current;
          setView('translate');
          t.setSourceText(trimmed);
          t.translate({
            text: trimmed,
            source_lang: p?.translation.source_lang || 'auto',
            target_lang: p?.translation.target_lang || 'ja',
            mode: p?.translation.mode || 'news',
            tone: p?.translation.tone || 'auto',
            preset_id: p?.translation.preset_id,
            provider_id: activeProviderIdRef.current || undefined,
            model: resolveModelRef.current(activeProviderIdRef.current),
          });
        }, 600);
      } catch {
        // clipboard read error, ignore
      }
    }, 400);

    return () => {
      clearInterval(interval);
      if (debounce) clearTimeout(debounce);
    };
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && config?.general.close_on_escape) {
        if (view !== 'translate') {
          setView('translate');
        }
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [config, view]);

  const handleRetranslate = useCallback(async (entry: HistoryEntry) => {
    translation.setSourceText(entry.source_text);
    setView('translate');
    await translation.translate({
      text: entry.source_text,
      source_lang: entry.source_lang,
      target_lang: entry.target_lang,
      mode: entry.mode,
      tone: entry.tone,
      provider_id: activeProviderId || undefined,
    });
  }, [translation]);

  const handleChangeLanguage = useCallback((lang: LanguageCode) => {
    updateGeneral({ ui_language: lang } as any);
  }, [updateGeneral]);

  if (loading || !config) {
    return (
      <div className="loading-screen">
        <div className="loading-spinner" />
      </div>
    );
  }

  return (
    <I18nProvider language={config.general.ui_language as LanguageCode} onChangeLanguage={handleChangeLanguage}>
      <AppContent
        view={view}
        setView={setView}
        config={config}
        translation={translation}
        modes={modes}
        mode={mode}
        tone={tone}
        availableProviders={availableProviders}
        activeProviderId={activeProviderId}
        setMode={setMode}
        setTone={setTone}
        setActiveProviderId={setActiveProviderId}
        updateGeneral={updateGeneral}
        updateTranslation={updateTranslation}
        updateShortcut={updateShortcut}
        updateHistory={updateHistory}
        saveProvider={saveProvider}
        handleRetranslate={handleRetranslate}
      />
    </I18nProvider>
  );
}

function AppContent({
  view, setView, config, translation,
  modes, mode, tone, availableProviders, activeProviderId,
  setMode, setTone, setActiveProviderId,
  updateGeneral, updateTranslation, updateShortcut, updateHistory, saveProvider,
  handleRetranslate,
}: any) {
  const { t } = useT();

  if (view === 'settings') {
    return (
      <SettingsPanel
        config={config}
        onUpdateGeneral={updateGeneral}
        onUpdateTranslation={updateTranslation}
        onUpdateShortcut={updateShortcut}
        onUpdateHistory={updateHistory}
        onSaveProvider={saveProvider}
        onClose={() => setView('translate')}
      />
    );
  }

  return (
    <div className="app-layout">
      <header className="titlebar" data-tauri-drag-region>
        <div className="titlebar-left" data-tauri-drag-region>
          <span className="titlebar-title" data-tauri-drag-region>{t('app.title')}</span>
        </div>
        <div className="titlebar-right">
          <div className="window-controls">
            <button className="window-btn" onClick={() => invoke('window_minimize')} title={t('titlebar.minimize')}>
              <span className="minimize-icon" />
            </button>
            <button className="window-btn" onClick={() => invoke('window_maximize')} title={t('titlebar.maximize')}>
              <span className="maximize-icon" />
            </button>
            <button className="window-btn window-btn-close" onClick={() => invoke('window_close')} title={t('titlebar.close')}>
              <span className="close-icon">✕</span>
            </button>
          </div>
        </div>
      </header>

      {view === 'translate' ? (
        <MainTranslate
          config={config}
          translation={translation}
          mode={mode}
          tone={tone}
          providerId={activeProviderId}
        />
      ) : (
        <div className="app-content">
          <HistoryPanel onRetranslate={handleRetranslate} onClose={() => setView('translate')} />
        </div>
      )}

      <StatusBar
        modes={modes}
        mode={mode}
        tone={tone}
        availableProviders={availableProviders}
        activeProviderId={activeProviderId}
        onChangeMode={setMode}
        onChangeTone={setTone}
        onChangeProvider={setActiveProviderId}
        onSettings={() => setView('settings')}
      />
    </div>
  );
}
