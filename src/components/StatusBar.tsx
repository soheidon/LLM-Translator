import type { ModeInfo } from '../types/translation';
import type { ProviderConfig } from '../types/provider';
import type { TranslateTab } from './TabBar';
import type { ModelRole } from '../types/model';
import { useT } from '../i18n/I18nContext';

function providerShortName(provider: ProviderConfig): string {
  const shortNames: Record<string, string> = {
    google_translate: 'Google',
    google_translate_apps_script: 'Google',
    deepl_free: 'DeepL',
    deepl_pro: 'DeepL',
    openai: 'OpenAI',
    gemini: 'Gemini',
    anthropic: 'Claude',
    ollama: 'Ollama',
    mimo: 'MiMo',
    deepseek: 'DeepSeek',
    moonshot: 'Kimi',
    qwen: 'Qwen',
    minimax: 'MiniMAX',
  };
  return shortNames[provider.id] ?? provider.name.split('/')[0].trim();
}

interface Props {
  modes: ModeInfo[];
  mode: string;
  tone: string;
  availableProviders: ProviderConfig[];
  activeProviderId: string | null;
  activeModelRole: ModelRole;
  activeTab: TranslateTab;
  onChangeMode: (mode: string) => void;
  onChangeTone: (tone: string) => void;
  onChangeProvider: (providerId: string | null) => void;
  onChangeModelRole: (role: ModelRole) => void;
  onSettings: () => void;
  chatgptDebugEnabled?: boolean;
  chatgptHtmlCssDebugEnabled?: boolean;
  chatgptConsoleLogEnabled?: boolean;
  onDebugChatgptDom?: () => void;
  onDebugChatgptHtmlCss?: () => void;
  onCopyConsoleLog?: () => void;
  onCopyLanguageLog?: () => void;
  debugMessage?: string;
  defaultProvider?: ProviderConfig;
}

export function StatusBar({ modes, mode, tone, availableProviders, activeProviderId, activeModelRole, activeTab, onChangeMode, onChangeTone, onChangeProvider, onChangeModelRole, onSettings, chatgptDebugEnabled, chatgptHtmlCssDebugEnabled, chatgptConsoleLogEnabled, onDebugChatgptDom, onDebugChatgptHtmlCss, onCopyConsoleLog, onCopyLanguageLog, debugMessage, defaultProvider }: Props) {
  const { t } = useT();
  const showLlmControls = activeTab === 'llm';

  const effectiveProvider = activeProviderId
    ? availableProviders.find(p => p.id === activeProviderId)
    : defaultProvider;
  const hasFastModel = !!effectiveProvider?.model_mapping?.fast?.model?.trim();

  const defaultLabel = defaultProvider
    ? `${t('status_bar.default_provider')} (${providerShortName(defaultProvider)})`
    : t('status_bar.default_provider');

  return (
    <footer className="status-bar">
      <div className="status-left">
        <span className="status-hint">{t('status_bar.hint')}</span>
      </div>
      <div className="status-right">
        <span className="status-version">{t('app.version')}</span>
        {activeTab === 'chatgpt' && chatgptDebugEnabled && onDebugChatgptDom && (
          <button className="google-debug-btn" onClick={onDebugChatgptDom} style={{ marginLeft: 8, fontSize: 11 }}>{t("status_bar.debug_dom")}</button>
        )}
        {activeTab === 'chatgpt' && chatgptHtmlCssDebugEnabled && onDebugChatgptHtmlCss && (
          <button className="google-debug-btn" onClick={onDebugChatgptHtmlCss} style={{ marginLeft: 8, fontSize: 11 }}>{t("status_bar.debug_html_css")}</button>
        )}
        {activeTab === 'chatgpt' && chatgptConsoleLogEnabled && onCopyConsoleLog && (
          <button className="google-debug-btn" onClick={onCopyConsoleLog} style={{ marginLeft: 8, fontSize: 11 }}>{t("status_bar.copy_console_log")}</button>
        )}
        {activeTab === 'chatgpt' && chatgptConsoleLogEnabled && onCopyLanguageLog && (
          <button className="google-debug-btn" onClick={onCopyLanguageLog} style={{ marginLeft: 8, fontSize: 11 }}>{t("status_bar.copy_lang_log")}</button>
        )}
        {activeTab === 'chatgpt' && debugMessage && (
          <span style={{ marginLeft: 8, fontSize: 11, color: 'var(--color-text-secondary, #999)' }}>{debugMessage}</span>
        )}
        {showLlmControls && (
          <>
            <div className="status-control-group">
              {availableProviders.length > 0 ? (
                <select
                  className="select provider-select"
                  value={activeProviderId ?? ''}
                  onChange={e => onChangeProvider(e.target.value || null)}
                >
                  <option value="">{defaultLabel}</option>
                  {availableProviders.map(p => (
                    <option key={p.id} value={p.id}>{providerShortName(p)}</option>
                  ))}
                </select>
              ) : (
                <select className="select provider-select" value="" disabled>
                  <option value="">{t('status_bar.no_api_key')}</option>
                </select>
              )}
            </div>
            <div className="status-control-group">
              <span className="toolbar-label">MODEL:</span>
              <select
                className="select model-role-select"
                value={activeModelRole}
                onChange={e => onChangeModelRole(e.target.value as ModelRole)}
              >
                <option value="default">{t('status_bar.model_quality')}</option>
                <option value="fast" disabled={!hasFastModel}>{t('status_bar.model_fast')}</option>
              </select>
            </div>
            <div className="status-control-group">
              <span className="toolbar-label">{t('status_bar.label_tone')}</span>
              <select className="select tone-select" value={tone} onChange={e => onChangeTone(e.target.value)}>
                {(['auto', 'plain', 'polite'] as const).map(tid => (
                  <option key={tid} value={tid}>{t(`tones.${tid}`)}</option>
                ))}
              </select>
            </div>
            <div className="status-control-group">
              <span className="toolbar-label">{t('status_bar.label_preset')}</span>
              <select className="select preset-select" value={mode} onChange={e => onChangeMode(e.target.value)}>
                {modes.map(m => (
                  <option key={m.id} value={m.id}>{t(`modes.${m.id}`)}</option>
                ))}
              </select>
            </div>
          </>
        )}
        <button className="settings-button" onClick={onSettings} title={t('titlebar.settings')}>
          <span>{t('titlebar.settings')}</span>
        </button>
      </div>
    </footer>
  );
}
