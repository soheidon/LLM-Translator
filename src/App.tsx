import { useState, useEffect, useCallback, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { readText, writeText } from '@tauri-apps/plugin-clipboard-manager';
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
import { getChatgptLanguage, resolveChatgptSourceLanguage, resolveChatgptTargetLanguage } from './data/chatgptTranslateLanguages';
import { AppIcon } from './components/AppIcon';
import './styles/app.css';

type View = 'translate' | 'settings' | 'history';
import type { ModelRole } from './types/model';

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
  const [activeModelRole, setActiveModelRole] = useState<ModelRole>('default');
  const activeModelRoleRef = useRef<ModelRole>('default');

  const handleChangeModelRole = useCallback((role: ModelRole) => {
    activeModelRoleRef.current = role;
    setActiveModelRole(role);
  }, []);

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

  const effectiveProvider = activeProviderId
    ? availableProviders.find(p => p.id === activeProviderId)
    : config?.providers.find(p => p.is_default);
  const hasFastModel = !!effectiveProvider?.model_mapping?.fast?.model?.trim();

  useEffect(() => {
    if (activeModelRole === 'fast' && !hasFastModel) {
      handleChangeModelRole('default');
    }
  }, [activeModelRole, hasFastModel, handleChangeModelRole]);

  // Stable refs for values used in long-lived effects (avoid re-registration on every render)
  const translateRef = useRef(translation);
  translateRef.current = translation;
  const configRef = useRef(config);
  configRef.current = config;
  const activeProviderIdRef = useRef(activeProviderId);
  activeProviderIdRef.current = activeProviderId;
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
      model_role: activeModelRoleRef.current,
    });
  }, []);

  // Listen for Tauri events (shortcut triggers from Rust)
  useEffect(() => {
    const unlistenTranslate = listen('trigger-translate', async () => {
      console.log('[trigger-translate] received', { activeTab: activeTabRef.current, focusOnTranslate: configRef.current?.general.focus_on_translate });
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
        activeModelRole={activeModelRole}
        onChangeModelRole={handleChangeModelRole}
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

function GoogleTranslatePanel({ debugTool }: { debugTool: boolean }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const toolbarHRef = useRef(0);
  const [ready, setReady] = useState(false);
  const [debugDom, setDebugDom] = useState('');
  const [currentUrl, setCurrentUrl] = useState('');
  const homeUrl = getGoogleTranslateUrl();

  function isGoogleTranslateHome(url: string): boolean {
    if (!url) return true;
    try {
      const u = new URL(url);
      return (
        (u.hostname === 'translate.google.com' || u.hostname === 'translate.google.co.jp') &&
        (u.pathname === '/' || u.pathname === '')
      );
    } catch {
      return false;
    }
  }

  const isTopPage = isGoogleTranslateHome(currentUrl);
  toolbarHRef.current = isTopPage ? 0 : 36;

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
    const tbh = toolbarHRef;
    const syncPosition = () => {
      const r = getRect();
      if (!r) return;
      invoke('set_google_translate_visible', {
        visible: true,
        x: r.x,
        y: r.y + tbh.current,
        width: r.width,
        height: r.height - tbh.current,
      }).catch(() => {});
    };

    const rect = getRect();
    if (!rect) return;

    invoke('open_google_translate', { url: homeUrl, ...rect })
      .then(() => setReady(true))
      .catch(console.error);

    requestAnimationFrame(() => {
      requestAnimationFrame(syncPosition);
    });

    const interval = setInterval(syncPosition, 500);

    const urlInterval = setInterval(async () => {
      try {
        const url = await invoke<string>('get_google_translate_url');
        setCurrentUrl(url);
      } catch {}
    }, 1000);

    return () => {
      clearInterval(interval);
      clearInterval(urlInterval);
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
      {ready && !isTopPage && (
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

function ChatGptTranslatePanel({ consoleLogEnabled }: { consoleLogEnabled?: boolean }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const toolbarHRef = useRef(0);
  const [ready, setReady] = useState(false);
  const [currentUrl, setCurrentUrl] = useState('');
  const isTopPage = currentUrl.startsWith(CHATGPT_TRANSLATE_URL) || currentUrl === '';
  toolbarHRef.current = isTopPage ? 0 : 36;

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
    const tbh = toolbarHRef;

    const syncPosition = () => {
      const r = getRect();
      if (!r) return;
      invoke('set_chatgpt_translate_visible', {
        visible: true,
        x: r.x,
        y: r.y + tbh.current,
        width: r.width,
        height: r.height - tbh.current,
      }).catch(() => {});
    };

    const rect = getRect();
    if (!rect) return;

    // Resolve saved language codes BEFORE opening webview — reject unsupported languages early
    const savedSourceCode = (() => {
      try { return localStorage.getItem('chatgptTranslateSourceLang'); }
      catch { return null; }
    })();
    const savedTargetCode = (() => {
      try { return localStorage.getItem('chatgptTranslateTargetLang'); }
      catch { return null; }
    })();
    const sourceCode = resolveChatgptSourceLanguage(savedSourceCode);
    const targetCode = resolveChatgptTargetLanguage(savedTargetCode);
    if (sourceCode !== savedSourceCode) {
      try { localStorage.setItem('chatgptTranslateSourceLang', sourceCode); } catch {}
    }
    if (targetCode !== savedTargetCode) {
      try { localStorage.setItem('chatgptTranslateTargetLang', targetCode); } catch {}
    }
    const sourceLang = getChatgptLanguage(sourceCode);
    const targetLang = getChatgptLanguage(targetCode);
    const sourceLabel = sourceLang?.nameJa || '言語を検出する';
    const sourceLabelEn = sourceLang?.nameEn || 'Detect language';
    const targetLabel = targetLang?.nameJa || '日本語';
    const targetLabelEn = targetLang?.nameEn || 'Japanese';

    invoke('open_chatgpt_translate', { url: CHATGPT_TRANSLATE_URL, ...rect })
      .then(() => {
        console.log('[ChatGPT Lang Invoke]', { sourceCode, targetCode, sourceLabel, targetLabel, sourceLabelEn, targetLabelEn, timestamp: new Date().toISOString() });
        return invoke('set_chatgpt_translate_languages', { sourceLabel, targetLabel, sourceLabelEn, targetLabelEn });
      })
      .then(() => {
        console.log('[ChatGPT Lang Invoke] success');
        if (consoleLogEnabled) {
          return invoke('set_chatgpt_console_log_enabled', { enabled: true });
        }
      })
      .then(() => setReady(true))
      .catch((e) => { console.error('[ChatGPT Lang Invoke] failed', e); setReady(true); });

    requestAnimationFrame(() => {
      requestAnimationFrame(syncPosition);
    });

    const interval = setInterval(syncPosition, 500);

    const urlInterval = setInterval(async () => {
      try {
        const url = await invoke<string>('get_chatgpt_translate_url');
        setCurrentUrl(url);
      } catch {}
    }, 1000);

    return () => {
      clearInterval(interval);
      clearInterval(urlInterval);
      invoke('set_chatgpt_translate_visible', { visible: false, x: 0, y: 0, width: 0, height: 0 }).catch(() => {});
    };
  }, [getRect]);

  const handleReload = () => invoke('chatgpt_translate_reload').catch(console.error);
  const handleHome = () => {
    invoke('chatgpt_translate_home', { url: CHATGPT_TRANSLATE_URL }).catch(console.error);
  };

  return (
    <div ref={containerRef} className="translate-container" style={{ background: 'transparent', position: 'relative' }}>
      {ready && !isTopPage && (
        <div className="browser-toolbar">
          <button className="browser-toolbar-btn" onClick={handleReload} aria-label="再読み込み" title="再読み込み">
            <ReloadIcon />
          </button>
          <button className="browser-toolbar-btn" onClick={handleHome} aria-label="chatGPT翻訳へ戻る" title="chatGPT翻訳へ戻る">
            <HomeIcon />
          </button>
          <span style={{ flex: 1 }} />
        </div>
      )}
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
  activeModelRole, onChangeModelRole,
  setMode, setTone, setActiveProviderId,
  updateGeneral, updateTranslation, updateShortcut, updateHistory, saveProvider,
  handleRetranslate,
}: any) {
  const { t } = useT();
  const [debugChatgptDom, setDebugChatgptDom] = useState('');
  const handleChatgptDebugDom = async () => {
    if (debugChatgptDom) { setDebugChatgptDom(''); return; }
    try {
      const result = await invoke<string>('debug_chatgpt_translate_dom');
      console.log('[ChatGPT DOM debug]', result);
      setDebugChatgptDom(result);
    } catch (e) {
      console.error('[ChatGPT DOM debug failed]', e);
      setDebugChatgptDom('Error: ' + String(e));
    }
  };
  const [debugChatgptHtmlCss, setDebugChatgptHtmlCss] = useState('');
  const handleDebugChatgptHtmlCss = async () => {
    if (debugChatgptHtmlCss) { setDebugChatgptHtmlCss(''); return; }
    try {
      const raw = await invoke<string>('debug_chatgpt_translate_html_css');
      try {
        console.log('[ChatGPT HTML+CSS debug]', JSON.parse(raw));
      } catch {
        console.log('[ChatGPT HTML+CSS debug]', raw);
      }
      setDebugChatgptHtmlCss(raw);
    } catch (e) {
      console.error('[ChatGPT HTML+CSS debug failed]', e);
      setDebugChatgptHtmlCss('Error: ' + String(e));
    }
  };
  const [debugMessage, setDebugMessage] = useState('');
  const debugTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const showDebugMessage = (msg: string) => {
    setDebugMessage(msg);
    if (debugTimerRef.current) clearTimeout(debugTimerRef.current);
    debugTimerRef.current = setTimeout(() => setDebugMessage(''), 3000);
  };
  const handleCopyConsoleLog = async () => {
    try {
      const raw = await invoke<string>('get_chatgpt_translate_console_log');
      const decoded = decodeURIComponent(raw);
      const result = JSON.parse(decoded);
      console.log('[ConsoleLogDiagnostic]', result);

      const entries = Array.isArray(result) ? result : result?.entries;

      if (!Array.isArray(entries)) {
        await writeText(JSON.stringify(result, null, 2));
        showDebugMessage('コンソールログ診断をコピーしました（ログ配列なし）');
        return;
      }

      if (entries.length === 0) {
        await writeText(JSON.stringify(result, null, 2));
        showDebugMessage('コンソールログ診断をコピーしました（0件）');
        return;
      }

      await writeText(JSON.stringify({
        diagnostic: {
          href: result.href,
          hasWindowProperty: result.hasWindowProperty,
          windowIsArray: result.windowIsArray,
          windowLength: result.windowLength,
          hasSessionStorageValue: result.hasSessionStorageValue,
          sessionStorageIsArray: result.sessionStorageIsArray,
          sessionStorageLength: result.sessionStorageLength,
          sessionStorageError: result.sessionStorageError,
        },
        entries,
      }, null, 2));
      showDebugMessage(`${entries.length} ${t('status_bar.msg_console_copied') || 'console entries copied'}`);
    } catch (e) {
      console.error('[CopyConsoleLog] failed', e);
      showDebugMessage(t('status_bar.msg_failed') || 'Failed');
    }
  };
  const handleCopyLanguageLog = async () => {
    try {
      const raw = await invoke<string>('get_language_debug_log');
      const decoded = decodeURIComponent(raw);
      const result = JSON.parse(decoded);
      console.log('[LanguageLogDiagnostic]', result);

      const entries = Array.isArray(result) ? result : result?.entries;

      if (!Array.isArray(entries)) {
        await writeText(JSON.stringify(result, null, 2));
        showDebugMessage('言語ログ診断をコピーしました（ログ配列なし）');
        return;
      }

      if (entries.length === 0) {
        await writeText(JSON.stringify(result, null, 2));
        showDebugMessage('言語ログ診断をコピーしました（0件）');
        return;
      }

      await writeText(JSON.stringify({
        diagnostic: {
          href: result.href,
          readyState: result.readyState,
          hasWindowProperty: result.hasWindowProperty,
          windowIsArray: result.windowIsArray,
          windowLength: result.windowLength,
          hasSessionStorageValue: result.hasSessionStorageValue,
          sessionStorageIsArray: result.sessionStorageIsArray,
          sessionStorageLength: result.sessionStorageLength,
          sessionStorageError: result.sessionStorageError,
        },
        entries,
      }, null, 2));
      showDebugMessage(`${entries.length} 件の言語ログをコピーしました`);
    } catch (e) {
      console.error('[CopyLanguageLog] failed', e);
      showDebugMessage(t('status_bar.msg_failed') || 'Failed');
    }
  };

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

      {view !== 'settings' && <TabBar activeTab={activeTab} onChangeTab={setActiveTab} />}

      {view === 'settings' ? (
        <SettingsPanel
          config={config}
          onUpdateGeneral={updateGeneral}
          onUpdateTranslation={updateTranslation}
          onUpdateShortcut={updateShortcut}
          onUpdateHistory={updateHistory}
          onSaveProvider={saveProvider}
          onClose={() => setView('translate')}
        />
      ) : view === 'translate' ? (
        activeTab === 'llm' ? (
          <MainTranslate
            config={config}
            translation={translation}
            mode={mode}
            tone={tone}
            providerId={activeProviderId}
            modelRole={activeModelRole}
          />
        ) : activeTab === 'google' ? (
          <GoogleTranslatePanel debugTool={config.general.google_translate_debug_tool} />
        ) : (
          <ChatGptTranslatePanel consoleLogEnabled={config.general.chatgpt_translate_console_log_enabled} />
        )
      ) : (
        <div className="app-content">
          <HistoryPanel onRetranslate={handleRetranslate} onClose={() => setView('translate')} />
        </div>
      )}

      {activeTab === 'chatgpt' && debugChatgptDom && (
        <div className="debug-log-panel">
          <div className="debug-log-header">
            <span>DOM診断ログ</span>
            <button className="debug-log-close" onClick={() => setDebugChatgptDom('')}>✕</button>
          </div>
          <textarea
            className="debug-log-textarea"
            value={debugChatgptDom}
            readOnly
            onClick={e => (e.target as HTMLTextAreaElement).select()}
          />
        </div>
      )}

      {activeTab === 'chatgpt' && debugChatgptHtmlCss && (
        <div className="debug-log-panel">
          <div className="debug-log-header">
            <span>HTML+CSS診断ログ</span>
            <button className="debug-log-close" onClick={() => setDebugChatgptHtmlCss('')}>✕</button>
          </div>
          <textarea
            className="debug-log-textarea"
            value={debugChatgptHtmlCss}
            readOnly
            onClick={e => (e.target as HTMLTextAreaElement).select()}
          />
        </div>
      )}

      {view !== 'settings' && (
      <StatusBar
        modes={modes}
        mode={mode}
        tone={tone}
        availableProviders={availableProviders}
        activeProviderId={activeProviderId}
        activeModelRole={activeModelRole}
        activeTab={activeTab}
        onChangeMode={setMode}
        onChangeTone={setTone}
        onChangeProvider={setActiveProviderId}
        onChangeModelRole={onChangeModelRole}
        onSettings={() => setView('settings')}
        chatgptDebugEnabled={config.general.chatgpt_translate_debug_tool}
        chatgptHtmlCssDebugEnabled={config.general.chatgpt_translate_html_css_debug_tool}
        chatgptConsoleLogEnabled={config.general.chatgpt_translate_console_log_enabled}
        onDebugChatgptDom={handleChatgptDebugDom}
        onDebugChatgptHtmlCss={handleDebugChatgptHtmlCss}
        onCopyConsoleLog={handleCopyConsoleLog}
        onCopyLanguageLog={handleCopyLanguageLog}
        debugMessage={debugMessage}
        defaultProvider={config?.providers?.find((p: any) => p.is_default)}
      />
      )}
    </div>
  );
}
