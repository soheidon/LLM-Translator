import { useState, useEffect, useCallback, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { MainTranslate } from './components/MainTranslate';
import { SettingsPanel } from './components/SettingsPanel';
import { HistoryPanel } from './components/HistoryPanel';
import { StatusBar } from './components/StatusBar';
import { TabBar, type TranslateTab } from './components/TabBar';
import { useSettings } from './hooks/useSettings';
import { useTranslationState } from './hooks/useTranslationState';
import { I18nProvider, useT } from './i18n/I18nContext';
import type { LanguageCode } from './lang';
import type { HistoryEntry, ModeInfo } from './types/translation';
import { GOOGLE_TRANSLATE_LANGUAGES } from './data/googleTranslateLanguages';
import { AppIcon } from './components/AppIcon';
import './styles/app.css';

type View = 'translate' | 'settings' | 'history';

const TAB_STORAGE_KEY = 'llm-translator:last-active-tab';
const VALID_TABS: readonly TranslateTab[] = ['llm', 'google', 'chatgpt'];

function loadInitialTab(): TranslateTab {
  try {
    const saved = localStorage.getItem(TAB_STORAGE_KEY);
    if (saved && VALID_TABS.includes(saved as TranslateTab)) {
      return saved as TranslateTab;
    }
  } catch { /* storage unavailable */ }
  return 'llm';
}

export default function App() {
  const [view, setView] = useState<View>('translate');
  const [activeTab, setActiveTab] = useState<TranslateTab>(loadInitialTab);
  const { config, loading, updateGeneral, updateTranslation, updateShortcut, updateHistory, saveProvider } = useSettings();

  useEffect(() => {
    try {
      localStorage.setItem(TAB_STORAGE_KEY, activeTab);
    } catch { /* storage unavailable */ }
  }, [activeTab]);
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
  const activeTabRef = useRef(activeTab);
  activeTabRef.current = activeTab;

  // Route clipboard text based on active tab
  const routeClipboard = useCallback(async (text: string) => {
    const tab = activeTabRef.current;
    if (tab === 'google') {
      console.log('[Ctrl+C+C] to google', { textLength: text.length });
      await invoke('set_google_translate_text', { text });
      return;
    }
    if (tab === 'chatgpt') {
      console.log('[Ctrl+C+C] to chatgpt', { textLength: text.length });
      await invoke('set_chatgpt_translate_text', { text });
      return;
    }
    // llm tab
    const t = translateRef.current;
    const c = configRef.current;
    t.setSourceText(text);
    t.translate({
      text,
      source_lang: c?.translation.source_lang || 'auto',
      target_lang: c?.translation.target_lang || 'ja',
      mode: c?.translation.mode || 'news',
      tone: c?.translation.tone || 'auto',
      preset_id: c?.translation.preset_id,
      provider_id: activeProviderIdRef.current || undefined,
      model: resolveModelRef.current(activeProviderIdRef.current),
    });
  }, []);

  // Listen for Tauri events (shortcut triggers from Rust)
  useEffect(() => {
    const unlistenTranslate = listen('trigger-translate', async () => {
      setView('translate');
      const c = configRef.current;
      if (c?.general.focus_on_translate) {
        invoke('focus_window').catch(() => {});
      }
      try {
        const text = await readText();
        if (text && text.trim()) {
          await routeClipboard(text.trim());
        }
      } catch { /* clipboard read error */ }
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
          setView('translate');
          routeClipboard(trimmed);
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
        activeTab={activeTab}
        setActiveTab={setActiveTab}
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

function getGoogleTranslateUrl(): string {
  try {
    const source = localStorage.getItem('googleTranslateSourceLang') || 'auto';
    const target = localStorage.getItem('googleTranslateTargetLang') || 'ja';
    return `https://translate.google.com/?sl=${source}&tl=${target}&op=translate`;
  } catch { /* storage unavailable */ }
  return 'https://translate.google.com/?sl=auto&tl=ja&op=translate';
}

const CHATGPT_TRANSLATE_URL = 'https://chatgpt.com/ja-JP/translate/';

function BackIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M15 18l-6-6 6-6" />
    </svg>
  );
}

function ForwardIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M9 6l6 6-6 6" />
    </svg>
  );
}

function ReloadIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M20 11a8 8 0 1 0-2.34 5.66" />
      <path d="M20 4v7h-7" />
    </svg>
  );
}

function HomeIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M3 11l9-8 9 8" />
      <path d="M5 10v10h14V10" />
      <path d="M9 20v-6h6v6" />
    </svg>
  );
}

function GoogleTranslatePanel({ googleTranslateToolbar, debugTool }: { googleTranslateToolbar: string; debugTool: boolean }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [ready, setReady] = useState(false);
  const [debugDom, setDebugDom] = useState('');
  const showToolbar = googleTranslateToolbar === 'always' || debugTool;
  const toolbarH = showToolbar ? 36 : 0;

  const getRect = useCallback(() => {
    if (!containerRef.current) return null;
    const rect = containerRef.current.getBoundingClientRect();
    return {
      x: Math.round(rect.left),
      y: Math.round(rect.top),
      width: Math.round(rect.width),
      height: Math.round(rect.height),
    };
  }, []);

  useEffect(() => {
    const syncPosition = () => {
      const r = getRect();
      if (!r) return;
      invoke('set_google_translate_visible', {
        visible: true,
        x: r.x,
        y: r.y + toolbarH,
        width: r.width,
        height: r.height - toolbarH,
      }).catch(() => {});
    };

    const rect = getRect();
    if (!rect) return;

    const url = getGoogleTranslateUrl();

    invoke('open_google_translate', { url, ...rect })
      .then(() => setReady(true))
      .catch(console.error);

    requestAnimationFrame(() => {
      requestAnimationFrame(syncPosition);
    });

    const interval = setInterval(syncPosition, 500);

    return () => {
      clearInterval(interval);
      invoke('set_google_translate_visible', { visible: false, x: 0, y: 0, width: 0, height: 0 }).catch(() => {});
    };
  }, [getRect]);

  const handleBack = () => invoke('google_translate_back').catch(console.error);
  const handleForward = () => invoke('google_translate_forward').catch(console.error);
  const handleReload = () => invoke('google_translate_reload').catch(console.error);
  const handleHome = () => {
    const homeUrl = getGoogleTranslateUrl();
    invoke('google_translate_home', { url: homeUrl }).catch(console.error);
  };

  const handleDebugDom = async () => {
    try {
      const result = await invoke<string>('debug_google_translate_dom');
      console.log('[Google DOM debug]', result);
      setDebugDom(result);
    } catch (e) {
      console.error('[Google DOM debug failed]', e);
      setDebugDom('Error: ' + String(e));
    }
  };

  return (
    <div ref={containerRef} className="translate-container" style={{ background: 'transparent', position: 'relative' }}>
      {ready && showToolbar && (
        <div className="browser-toolbar">
          <button className="browser-toolbar-btn" onClick={handleBack} aria-label="戻る" title="戻る">
            <BackIcon />
          </button>
          <button className="browser-toolbar-btn" onClick={handleForward} aria-label="進む" title="進む">
            <ForwardIcon />
          </button>
          <button className="browser-toolbar-btn" onClick={handleReload} aria-label="再読み込み" title="再読み込み">
            <ReloadIcon />
          </button>
          <button className="browser-toolbar-btn" onClick={handleHome} aria-label="Google翻訳へ戻る" title="Google翻訳へ戻る">
            <HomeIcon />
          </button>
          <span style={{ flex: 1 }} />
            {debugTool && (
              <button className="google-debug-btn" onClick={handleDebugDom}>DOM診断</button>
            )}
        </div>
      )}
      {debugTool && debugDom && (
        <textarea
          className="google-debug-textarea"
          value={debugDom}
          readOnly
          onClick={e => (e.target as HTMLTextAreaElement).select()}
        />
      )}
      {!ready && (
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
          <div className="loading-spinner" />
        </div>
      )}
    </div>
  );
}

function ChatGptTranslatePanel() {
  const containerRef = useRef<HTMLDivElement>(null);
  const [ready, setReady] = useState(false);
  const toolbarH = 0;

  const getRect = useCallback(() => {
    if (!containerRef.current) return null;
    const rect = containerRef.current.getBoundingClientRect();
    return {
      x: Math.round(rect.left),
      y: Math.round(rect.top),
      width: Math.round(rect.width),
      height: Math.round(rect.height),
    };
  }, []);

  useEffect(() => {
    const syncPosition = () => {
      const r = getRect();
      if (!r) return;
      invoke('set_chatgpt_translate_visible', {
        visible: true,
        x: r.x,
        y: r.y + toolbarH,
        width: r.width,
        height: r.height - toolbarH,
      }).catch(() => {});
    };

    const rect = getRect();
    if (!rect) return;

    invoke('open_chatgpt_translate', { url: CHATGPT_TRANSLATE_URL, ...rect })
      .then(() => {
        setReady(true);
        const sourceCode = (() => {
          try { return localStorage.getItem('chatgptTranslateSourceLang') || 'auto'; }
          catch { return 'auto'; }
        })();
        const targetCode = (() => {
          try { return localStorage.getItem('chatgptTranslateTargetLang') || 'ja'; }
          catch { return 'ja'; }
        })();
        const sourceLang = GOOGLE_TRANSLATE_LANGUAGES.find(l => l.code === sourceCode);
        const targetLang = GOOGLE_TRANSLATE_LANGUAGES.find(l => l.code === targetCode);
        const sourceLabel = sourceCode === 'auto' ? '言語を検出する' : (sourceLang?.nameJa || '言語を検出する');
        const targetLabel = targetLang?.nameJa || '日本語';
        return invoke('set_chatgpt_translate_languages', { sourceLabel, targetLabel });
      })
      .catch(console.error);

    requestAnimationFrame(() => {
      requestAnimationFrame(syncPosition);
    });

    const interval = setInterval(syncPosition, 500);

    return () => {
      clearInterval(interval);
      invoke('set_chatgpt_translate_visible', { visible: false, x: 0, y: 0, width: 0, height: 0 }).catch(() => {});
    };
  }, [getRect]);

  return (
    <div ref={containerRef} className="translate-container" style={{ background: 'transparent', position: 'relative' }}>
      {!ready && (
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
          <div className="loading-spinner" />
        </div>
      )}
    </div>
  );
}

function AppContent({
  view, setView, activeTab, setActiveTab, config, translation,
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
      <header className="titlebar" data-tauri-drag-region onMouseDown={() => invoke('start_drag')}>
        <div className="titlebar-left" data-tauri-drag-region>
          <AppIcon />
          <span className="titlebar-title" data-tauri-drag-region>{t('app.title')}</span>
        </div>
        <div className="titlebar-right">
          <div className="window-controls">
            <button className="window-btn" onClick={() => invoke('window_minimize')} onMouseDown={(e) => e.stopPropagation()} title={t('titlebar.minimize')}>
              <span className="minimize-icon" />
            </button>
            <button className="window-btn" onClick={() => invoke('window_maximize')} onMouseDown={(e) => e.stopPropagation()} title={t('titlebar.maximize')}>
              <span className="maximize-icon" />
            </button>
            <button className="window-btn window-btn-close" onClick={() => invoke('window_close')} onMouseDown={(e) => e.stopPropagation()} title={t('titlebar.close')}>
              <span className="close-icon">✕</span>
            </button>
          </div>
        </div>
      </header>

      <TabBar activeTab={activeTab} onChangeTab={setActiveTab} />

      {view === 'translate' ? (
        activeTab === 'llm' ? (
          <MainTranslate
            config={config}
            translation={translation}
            mode={mode}
            tone={tone}
            providerId={activeProviderId}
          />
        ) : activeTab === 'google' ? (
          <GoogleTranslatePanel googleTranslateToolbar={config.general.google_translate_toolbar} debugTool={config.general.google_translate_debug_tool} />
        ) : (
          <ChatGptTranslatePanel />
        )
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
        activeTab={activeTab}
        onChangeMode={setMode}
        onChangeTone={setTone}
        onChangeProvider={setActiveProviderId}
        onSettings={() => setView('settings')}
      />
    </div>
  );
}
