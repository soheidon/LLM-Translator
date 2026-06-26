import type { ModeInfo } from '../types/translation';
import type { ProviderConfig } from '../types/provider';
import type { TranslateTab } from './TabBar';
import { useT } from '../i18n/I18nContext';

interface Props {
  modes: ModeInfo[];
  mode: string;
  tone: string;
  availableProviders: ProviderConfig[];
  activeProviderId: string | null;
  activeTab: TranslateTab;
  onChangeMode: (mode: string) => void;
  onChangeTone: (tone: string) => void;
  onChangeProvider: (providerId: string | null) => void;
  onSettings: () => void;
  chatgptDebugEnabled?: boolean;
  onDebugChatgptDom?: () => void;
}

export function StatusBar({ modes, mode, tone, availableProviders, activeProviderId, activeTab, onChangeMode, onChangeTone, onChangeProvider, onSettings, chatgptDebugEnabled, onDebugChatgptDom }: Props) {
  const { t } = useT();
  const showLlmControls = activeTab === 'llm';
  return (
    <footer className="status-bar">
      <div className="status-left">
        <span className="status-hint">{t('status_bar.hint')}</span>
      </div>
      <div className="status-right">
        <span className="status-version">{t('app.version')}</span>
        {activeTab === 'chatgpt' && chatgptDebugEnabled && onDebugChatgptDom && (
          <button className="google-debug-btn" onClick={onDebugChatgptDom} style={{ marginLeft: 8, fontSize: 11 }}>DOM診断</button>
        )}
        {showLlmControls && (
          <>
            <div className="status-control-group">
              <span className="toolbar-label">{t('status_bar.label_model')}</span>
              {availableProviders.length > 0 ? (
                <select
                  className="select model-select"
                  value={activeProviderId ?? ''}
                  onChange={e => onChangeProvider(e.target.value || null)}
                >
                  <option value="">{t('status_bar.default_provider')}</option>
                  {availableProviders.map(p => (
                    <option key={p.id} value={p.id}>{p.name}</option>
                  ))}
                </select>
              ) : (
                <select className="select model-select" value="" disabled>
                  <option value="">{t('status_bar.no_api_key')}</option>
                </select>
              )}
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
