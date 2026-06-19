import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useT } from '../i18n/I18nContext';
import { SUPPORTED_LANGUAGES } from '../lang';

import type { AppConfig } from '../types/settings';
import type { ProviderConfig, ConnectionTestResult, OllamaModel } from '../types/provider';

interface Props {
  config: AppConfig;
  onUpdateGeneral: (updates: Partial<AppConfig['general']>) => void;
  onUpdateTranslation: (updates: Partial<AppConfig['translation']>) => void;
  onUpdateShortcut: (updates: Partial<AppConfig['shortcut']>) => void;
  onUpdateHistory: (updates: Partial<AppConfig['history']>) => void;
  onSaveProvider: (provider: ProviderConfig) => void;
  onClose: () => void;
}

type SettingsTab = 'general' | 'api' | 'presets' | 'history';

export function SettingsPanel({ config, onUpdateGeneral, onUpdateShortcut, onUpdateHistory, onSaveProvider, onClose }: Props) {
  const [tab, setTab] = useState<SettingsTab>('general');
  const { t } = useT();

  return (
    <div className="app-shell">
      {/* Sidebar */}
      <nav className="sidebar">
        <div className="sidebar-header">
          <div className="sidebar-title">{t('app.title')}</div>
          <div className="sidebar-version">{t('app.version')}</div>
        </div>
        <ul className="sidebar-nav">
          <li>
            <a className={`sidebar-item ${tab === 'general' ? 'active' : ''}`} onClick={() => setTab('general')}>
              <span className="icon">⚙</span>
              <span>{t('settings.sidebar.general')}</span>
            </a>
          </li>
          <li>
            <a className={`sidebar-item ${tab === 'api' ? 'active' : ''}`} onClick={() => setTab('api')}>
              <span className="icon">🔌</span>
              <span>{t('settings.sidebar.api')}</span>
            </a>
          </li>
          <li>
            <a className={`sidebar-item ${tab === 'presets' ? 'active' : ''}`} onClick={() => setTab('presets')}>
              <span className="icon">🎛</span>
              <span>{t('settings.sidebar.presets')}</span>
            </a>
          </li>
          <li>
            <a className={`sidebar-item ${tab === 'history' ? 'active' : ''}`} onClick={() => setTab('history')}>
              <span className="icon">📋</span>
              <span>{t('settings.sidebar.history')}</span>
            </a>
          </li>
        </ul>
        <div className="sidebar-close">
          <button className="sidebar-close-btn" onClick={onClose}>
            {t('settings.sidebar.close')}
          </button>
        </div>
      </nav>

      {/* Main */}
      <div className="main-content">
        <header className="top-bar">
          <h1 className="top-bar-title">{t('app.title')}</h1>
          <button className="settings-close-button" onClick={onClose} title={t('settings.sidebar.close')}>
            <span className="settings-close-icon">×</span>
          </button>
        </header>

        <div className="main-content-body">
          {tab === 'general' && (
            <GeneralSettings config={config} onUpdateGeneral={onUpdateGeneral} onUpdateShortcut={onUpdateShortcut} onUpdateHistory={onUpdateHistory} />
          )}
          {tab === 'api' && (
            <ApiSettings providers={config.providers} onSaveProvider={onSaveProvider} />
          )}
          {tab === 'presets' && (
            <PresetSettings />
          )}
          {tab === 'history' && (
            <HistorySettings config={config} onUpdateHistory={onUpdateHistory} />
          )}
        </div>
      </div>
    </div>
  );
}

// --- General Settings ---
function GeneralSettings({ config, onUpdateGeneral, onUpdateShortcut, onUpdateHistory }: {
  config: AppConfig;
  onUpdateGeneral: (u: Partial<AppConfig['general']>) => void;
  onUpdateShortcut: (u: Partial<AppConfig['shortcut']>) => void;
  onUpdateHistory: (u: Partial<AppConfig['history']>) => void;
}) {
  const { t } = useT();
  return (
    <div className="settings-section">
      <h3>{t('settings.general.title')}</h3>
      <p className="settings-section-desc">{t('settings.general.desc')}</p>

      {/* UI Language */}
      <div className="settings-group">
        <div className="settings-row">
          <div>
            <div className="settings-label">{t('settings.general.ui_language')}</div>
          </div>
          <div className="settings-control">
            <select className="select" style={{ width: 200 }} value={config.general.ui_language} onChange={e => onUpdateGeneral({ ui_language: e.target.value })}>
              {SUPPORTED_LANGUAGES.map(l => (
                <option key={l.code} value={l.code}>{l.nativeName}</option>
              ))}
            </select>
          </div>
        </div>
      </div>

      {/* Startup / Residence */}
      <div className="settings-group" style={{ marginTop: 24 }}>
        <div className="settings-row">
          <div>
            <div className="settings-label">{t('settings.general.start_minimized')}</div>
            <div className="settings-description">{t('settings.general.start_minimized_desc')}</div>
          </div>
          <div className="settings-control">
            <button className={`toggle ${config.general.start_minimized ? 'active' : ''}`} onClick={() => onUpdateGeneral({ start_minimized: !config.general.start_minimized })} />
          </div>
        </div>
        <div className="settings-row">
          <div>
            <div className="settings-label">{t('settings.general.always_on_top')}</div>
            <div className="settings-description">{t('settings.general.always_on_top_desc')}</div>
          </div>
          <div className="settings-control">
            <button className={`toggle ${config.general.always_on_top ? 'active' : ''}`} onClick={() => onUpdateGeneral({ always_on_top: !config.general.always_on_top })} />
          </div>
        </div>
        <div className="settings-row">
          <div>
            <div className="settings-label">{t('settings.general.focus_on_translate')}</div>
            <div className="settings-description">{t('settings.general.focus_on_translate_desc')}</div>
          </div>
          <div className="settings-control">
            <button className={`toggle ${config.general.focus_on_translate ? 'active' : ''}`} onClick={() => onUpdateGeneral({ focus_on_translate: !config.general.focus_on_translate })} />
          </div>
        </div>
        <div className="settings-row">
          <div>
            <div className="settings-label">{t('settings.general.close_on_escape')}</div>
            <div className="settings-description">{t('settings.general.close_on_escape_desc')}</div>
          </div>
          <div className="settings-control">
            <button className={`toggle ${config.general.close_on_escape ? 'active' : ''}`} onClick={() => onUpdateGeneral({ close_on_escape: !config.general.close_on_escape })} />
          </div>
        </div>
        <div className="settings-row">
          <div>
            <div className="settings-label">{t('settings.general.close_on_outside_click')}</div>
            <div className="settings-description">{t('settings.general.close_on_outside_click_desc')}</div>
          </div>
          <div className="settings-control">
            <button className={`toggle ${config.general.close_on_outside_click ? 'active' : ''}`} onClick={() => onUpdateGeneral({ close_on_outside_click: !config.general.close_on_outside_click })} />
          </div>
        </div>
      </div>

      {/* Shortcuts */}
      <div className="settings-group" style={{ marginTop: 24 }}>
        <div className="settings-row">
          <div>
            <div className="settings-label">{t('settings.general.quick_translate')}</div>
            <div className="settings-description">{t('settings.general.quick_translate_desc')}</div>
          </div>
          <div className="settings-control">
            <button className={`toggle ${config.shortcut.double_copy_enabled ? 'active' : ''}`} onClick={() => onUpdateShortcut({ double_copy_enabled: !config.shortcut.double_copy_enabled })} />
          </div>
        </div>
        <div className="settings-row">
          <div>
            <div className="settings-label">{t('settings.general.alt_shortcut')}</div>
          </div>
          <div className="settings-control">
            <input className="input" style={{ width: 160 }} value={config.shortcut.primary} onChange={e => onUpdateShortcut({ primary: e.target.value })} />
          </div>
        </div>
        <div className="settings-row">
          <div>
            <div className="settings-label">{t('settings.general.open_window')}</div>
          </div>
          <div className="settings-control">
            <input className="input" style={{ width: 160 }} value={config.shortcut.open_window} onChange={e => onUpdateShortcut({ open_window: e.target.value })} />
          </div>
        </div>
        <div className="settings-row">
          <div>
            <div className="settings-label">{t('settings.general.open_history')}</div>
          </div>
          <div className="settings-control">
            <input className="input" style={{ width: 160 }} value={config.shortcut.open_history} onChange={e => onUpdateShortcut({ open_history: e.target.value })} />
          </div>
        </div>
      </div>

      {/* History */}
      <div className="settings-group" style={{ marginTop: 24 }}>
        <div className="settings-row">
          <div>
            <div className="settings-label">{t('settings.general.history_enabled')}</div>
            <div className="settings-description">{t('settings.general.history_enabled_desc')}</div>
          </div>
          <div className="settings-control">
            <button className={`toggle ${config.history.enabled ? 'active' : ''}`} onClick={() => onUpdateHistory({ enabled: !config.history.enabled })} />
          </div>
        </div>
      </div>

    </div>
  );
}

// --- API Settings ---
function ApiSettings({ providers, onSaveProvider }: {
  providers: ProviderConfig[];
  onSaveProvider: (p: ProviderConfig) => void;
}) {
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<Record<string, ConnectionTestResult>>({});
  const [testing, setTesting] = useState<Record<string, boolean>>({});
  const [envStatuses, setEnvStatuses] = useState<Record<string, boolean>>({});
  const { t } = useT();

  const handleSetDefault = (provider: ProviderConfig) => {
    providers.forEach(p => {
      if (p.id === provider.id) {
        if (!p.is_default) onSaveProvider({ ...p, is_default: true });
      } else if (p.is_default) {
        onSaveProvider({ ...p, is_default: false });
      }
    });
  };

  // Check env var statuses on mount (skip ollama - no API key needed)
  useEffect(() => {
    providers.forEach(p => {
      if (p.id === 'ollama') {
        setEnvStatuses(prev => ({ ...prev, [p.id]: true }));
        return;
      }
      invoke<{ is_set: boolean }>('check_env_var', { envVar: p.env_var })
        .then(s => setEnvStatuses(prev => ({ ...prev, [p.id]: s.is_set })))
        .catch(() => {});
    });
  }, [providers]);

  const handleTest = async (provider: ProviderConfig) => {
    setTesting(t => ({ ...t, [provider.id]: true }));
    try {
      const result = await invoke<ConnectionTestResult>('test_connection', { provider });
      setTestResults(r => ({ ...r, [provider.id]: result }));
    } catch (e: any) {
      setTestResults(r => ({ ...r, [provider.id]: { success: false, message: String(e), message_code: null, latency_ms: null } }));
    }
    setTesting(t => ({ ...t, [provider.id]: false }));
  };

  return (
    <div className="api-settings-page">
      <h2 className="api-settings-title">{t('settings.api.title')}</h2>
      <p className="api-settings-desc">{t('settings.api.desc')}</p>
      <table className="provider-table">
        <thead>
          <tr>
            <th>{t('settings.api.table_provider')}</th>
            <th>{t('settings.api.table_status')}</th>
          </tr>
        </thead>
        <tbody>
          {providers.map(p => {
            const result = testResults[p.id];
            const isExpanded = expandedId === p.id;
            const envOk = envStatuses[p.id];
            return (
              <ProviderRow
                key={p.id}
                provider={p}
                isExpanded={isExpanded}
                testResult={result}
                isTesting={testing[p.id]}
                envOk={envOk}
                onToggle={() => setExpandedId(isExpanded ? null : p.id)}
                onTest={() => handleTest(p)}
                onSetDefault={() => handleSetDefault(p)}
                onSave={onSaveProvider}
                onEnvSaved={p.id === 'ollama' ? () => {} : (envVar: string) => {
                  setEnvStatuses(prev => ({ ...prev, [p.id]: false }));
                  invoke<{ is_set: boolean }>('check_env_var', { envVar })
                    .then(s => setEnvStatuses(prev => ({ ...prev, [p.id]: s.is_set })))
                    .catch(() => {});
                }}
              />
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function ProviderRow({ provider, isExpanded, testResult, isTesting, envOk, onToggle, onTest, onSetDefault, onSave, onEnvSaved }: {
  provider: ProviderConfig;
  isExpanded: boolean;
  testResult?: ConnectionTestResult;
  isTesting?: boolean;
  envOk: boolean;
  onToggle: () => void;
  onTest: () => void;
  onSetDefault: () => void;
  onSave: (p: ProviderConfig) => void;
  onEnvSaved: (envVar: string) => void;
}) {
  const { t } = useT();
  const [editing, setEditing] = useState<ProviderConfig>({ ...provider });
  const [envVarDraft, setEnvVarDraft] = useState(provider.env_var);
  const [apiKeyDraft, setApiKeyDraft] = useState('');
  const [apiKeySaving, setApiKeySaving] = useState(false);
  const [apiKeyMsg, setApiKeyMsg] = useState('');
  const [ollamaModels, setOllamaModels] = useState<OllamaModel[]>([]);
  const [ollamaLoading, setOllamaLoading] = useState(false);
  const [ollamaError, setOllamaError] = useState('');
  const [showAdvanced, setShowAdvanced] = useState(false);

  const handleFetchOllamaModels = async () => {
    setOllamaLoading(true);
    setOllamaError('');
    try {
      const models = await invoke<OllamaModel[]>('list_ollama_models', { baseUrl: editing.api_base_url });
      setOllamaModels(models);
      if (models.length === 0) setOllamaError(t('settings.api.ollama_no_models'));
    } catch (e: any) {
      setOllamaError(String(e));
      setOllamaModels([]);
    }
    setOllamaLoading(false);
  };

  const handleSelectOllamaModel = (modelName: string) => {
    if (!modelName) return;
    const mm = {
      ...editing.model_mapping,
      default: { model: modelName, mode: editing.model_mapping?.default?.mode || 'normal' },
      fast: { model: modelName, mode: editing.model_mapping?.fast?.mode || 'normal' },
    };
    const u = { ...editing, model_mapping: mm, model: modelName };
    setEditing(u);
    onSave(u);
  };

  useEffect(() => {
    setEditing({ ...provider });
    setEnvVarDraft(provider.env_var);
  }, [provider]);

  const handleSaveEnvVar = () => {
    if (envVarDraft === provider.env_var) return;
    const updated = { ...editing, env_var: envVarDraft };
    setEditing(updated);
    onSave(updated);
    onEnvSaved(envVarDraft);
  };

  const handleSaveApiKey = async () => {
    if (!apiKeyDraft.trim()) return;
    setApiKeySaving(true);
    setApiKeyMsg('');
    try {
      const msg = await invoke<string>('set_user_env_var', { name: envVarDraft, value: apiKeyDraft });
      setApiKeyDraft('');
      setApiKeyMsg(msg);
      onEnvSaved(envVarDraft);
    } catch (e: any) {
      setApiKeyMsg(`${t('errors.translation_error')}: ${e}`);
    }
    setApiKeySaving(false);
  };

  const isOllama = provider.id === 'ollama';
  const isGoogleTranslate = provider.id === 'google_translate' || provider.id === 'google_translate_apps_script';
  const isGoogleAppsScript = provider.id === 'google_translate_apps_script';
  const isDeepL = provider.api_type === 'DeepL';
  const isNonLLM = isGoogleTranslate || isDeepL; // no prompt, no model mapping, no thinking/normal
  const ollamaHasModel = ollamaModels.length > 0 || editing.model !== '';
  const statusLabel = () => {
    if (isTesting) return <span className="status-missing">{t('settings.api.status_testing')}</span>;
    if (testResult?.success) return <span className="status-ok">{t('settings.api.status_connected')}</span>;
    if (testResult && !testResult.success) return <span className="status-badge" style={{ color: 'var(--color-error)' }}>{t('settings.api.status_failed')}</span>;
    if (isOllama) return ollamaHasModel ? <span className="status-ok">{t('settings.api.status_local_connected')}</span> : <span className="status-missing">{t('settings.api.status_no_model')}</span>;
    if (envOk) return <span className="status-neutral">{t('settings.api.status_configured_untested')}</span>;
    return <span className="status-missing">{t('settings.api.status_missing_env')}</span>;
  };

  const modelEntries = () => {
    const entries: { key: string; label: string; model: string; mode: string }[] = [];
    if (editing.model_mapping?.default) {
      entries.push({ key: 'default', label: t('settings.api.role_high_quality'), model: editing.model_mapping.default.model, mode: editing.model_mapping.default.mode });
    }
    if (editing.model_mapping?.fast) {
      entries.push({ key: 'fast', label: t('settings.api.role_fast'), model: editing.model_mapping.fast.model, mode: editing.model_mapping.fast.mode });
    }
    return entries;
  };

  return (
    <>
      <tr className={`provider-row ${isExpanded ? 'open' : ''}`} onClick={onToggle} style={{ cursor: 'pointer' }}>
        <td>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <span style={{ fontSize: 12, color: 'var(--color-outline)' }}>{isExpanded ? '▼' : '▶'}</span>
            <strong>{provider.name}</strong>
            {provider.is_default && <span className="badge badge-active">{t('settings.api.default_badge')}</span>}
          </div>
        </td>
        <td>{statusLabel()}</td>
      </tr>
      {isExpanded && (
        <tr className="provider-expanded-row">
          <td colSpan={2}>
            <div className="provider-panel">
              <div className="provider-grid">
                {/* Row 1: 環境変数 + APIキー */}
                {isOllama ? (
                <div>
                  <label className="settings-label">{t('settings.api.ollama_connection')}</label>
                  <p className="field-help" style={{ marginTop: 4 }}>
                    {t('settings.api.ollama_help')}
                  </p>
                </div>
                ) : isGoogleAppsScript ? (
                <div>
                  <label className="settings-label">{t('settings.api.apps_script_url')}</label>
                  <div className="inline-field">
                    <input
                      className="input"
                      style={{ width: 200, fontFamily: 'var(--font-code)', fontSize: 13 }}
                      value={envVarDraft}
                      onChange={e => setEnvVarDraft(e.target.value)}
                      onBlur={() => { handleSaveEnvVar(); }}
                    />
                    <input
                      className="input"
                      style={{ flex: 1 }}
                      value={apiKeyDraft}
                      onChange={e => setApiKeyDraft(e.target.value)}
                      placeholder={t('settings.api.apps_script_placeholder')}
                    />
                    <button
                      className="btn btn-primary btn-sm"
                      onClick={handleSaveApiKey}
                      disabled={apiKeySaving || !apiKeyDraft.trim()}
                    >
                      {apiKeySaving ? t('settings.api.saving') : t('settings.api.save_env_var')}
                    </button>
                  </div>
                  <p className="field-help">
                    {t('settings.api.apps_script_help', { name: envVarDraft })}
                  </p>
                  {apiKeyMsg && (
                    <p className="field-help" style={{ color: apiKeyMsg.startsWith('Error') || apiKeyMsg.includes('error') ? 'var(--color-error)' : '#007306' }}>
                      {apiKeyMsg}
                    </p>
                  )}
                </div>
                ) : (
                <div>
                  <label className="settings-label">{t('settings.api.env_var_label')}</label>
                  <div className="inline-field">
                    <input
                      className="input"
                      style={{ width: 180, fontFamily: 'var(--font-code)', fontSize: 13 }}
                      value={envVarDraft}
                      onChange={e => setEnvVarDraft(e.target.value)}
                      onBlur={() => { handleSaveEnvVar(); }}
                    />
                    <input
                      className="input"
                      type="password"
                      style={{ flex: 1 }}
                      value={apiKeyDraft}
                      onChange={e => setApiKeyDraft(e.target.value)}
                      placeholder="sk-..."
                    />
                    <button
                      className="btn btn-primary btn-sm"
                      onClick={handleSaveApiKey}
                      disabled={apiKeySaving || !apiKeyDraft.trim()}
                    >
                      {apiKeySaving ? t('settings.api.saving') : t('settings.api.save_env_var')}
                    </button>
                  </div>
                  <p className="field-help">
                    {t('settings.api.env_var_help')}
                  </p>
                  {apiKeyMsg && (
                    <p className="field-help" style={{ color: apiKeyMsg.startsWith('Error') || apiKeyMsg.includes('error') ? 'var(--color-error)' : '#007306' }}>
                      {apiKeyMsg}
                    </p>
                  )}
                </div>
                )}
                {/* Row 2: Ollamaローカルモデル */}
                {isOllama && (
                <div>
                  <label className="settings-label">{t('settings.api.ollama_local_models')}</label>
                  <div className="inline-field">
                    <button className="btn btn-secondary btn-sm" onClick={handleFetchOllamaModels} disabled={ollamaLoading}>
                      {ollamaLoading ? t('settings.api.ollama_fetching') : t('settings.api.ollama_fetch')}
                    </button>
                    {ollamaModels.length > 0 && (
                      <select
                        className="select"
                        style={{ flex: 1 }}
                        value={editing.model}
                        onChange={e => handleSelectOllamaModel(e.target.value)}
                      >
                        <option value="">{t('settings.api.ollama_select_model')}</option>
                        {ollamaModels.map(m => (
                          <option key={m.name} value={m.name}>{m.name} ({m.size})</option>
                        ))}
                      </select>
                    )}
                  </div>
                  <p className="field-help">
                    {t('settings.api.ollama_help_started')}
                  </p>
                  {ollamaError && <p className="field-help" style={{ color: 'var(--color-error)' }}>{ollamaError}</p>}
                  {ollamaModels.length > 0 && (
                    <p className="field-help" style={{ color: '#007306' }}>
                      {t('settings.api.ollama_models_found', { count: ollamaModels.length })}
                    </p>
                  )}
                </div>
                )}

                {/* Row 3: Base URL */}
                <div className="full-width">
                  <label className="settings-label">{t('settings.api.env_var_base_url')}</label>
                  <input
                    className="input"
                    style={{ width: '100%', marginTop: 4, fontFamily: 'var(--font-code)', fontSize: 13 }}
                    value={editing.api_base_url}
                    onChange={e => setEditing({ ...editing, api_base_url: e.target.value })}
                    onBlur={() => onSave(editing)}
                  />
                </div>
              </div>

              {/* Advanced Settings Toggle (LLM only) */}
              {!isOllama && !isNonLLM && (
                <div style={{ marginTop: 16, borderTop: '1px solid var(--color-outline)', paddingTop: 12 }}>
                  <button
                    className="btn btn-secondary btn-sm"
                    onClick={() => setShowAdvanced(!showAdvanced)}
                    style={{ fontSize: 12 }}
                  >
                    {showAdvanced ? t('settings.api.advanced_hide') : t('settings.api.advanced_show')}
                  </button>
                  {showAdvanced && (
                    <div className="advanced-provider-settings" style={{ marginTop: 12 }}>
                      <div>
                        <label className="settings-label">{t('settings.api.api_type')}</label>
                        <select
                          className="select"
                          style={{ width: '100%', marginTop: 4 }}
                          value={editing.api_type}
                          onChange={e => {
                            const u = { ...editing, api_type: e.target.value as ProviderConfig['api_type'] };
                            setEditing(u);
                            onSave(u);
                          }}
                        >
                          <option value="OpenAiCompat">OpenAI Compatible</option>
                          <option value="AnthropicCompat">Anthropic Compatible</option>
                        </select>
                      </div>
                    </div>
                  )}
                </div>
              )}

              {/* Model Mapping (LLM only) */}
              {!isNonLLM && (
              <>
              <h4 style={{ marginTop: 24, marginBottom: 12, fontSize: 14, fontWeight: 600 }}>{t('settings.api.model_mapping')}</h4>
              <table className="model-mapping-table">
                <thead>
                  <tr>
                    <th>{t('settings.api.model_role')}</th>
                    <th>{t('settings.api.model_id')}</th>
                    <th>{t('settings.api.model_mode')}</th>
                  </tr>
                </thead>
                <tbody>
                  {modelEntries().map(entry => (
                    <tr key={entry.key}>
                      <td>{entry.label}</td>
                      <td>
                        <input
                          className="input"
                          style={{ width: 160, fontFamily: 'var(--font-code)', fontSize: 12 }}
                          value={entry.model}
                          onChange={e => {
                            const mm = { ...editing.model_mapping, [entry.key]: { ...editing.model_mapping[entry.key], model: e.target.value } };
                            setEditing({ ...editing, model_mapping: mm });
                          }}
                          onBlur={() => onSave(editing)}
                        />
                      </td>
                      <td>
                        <select
                          className="select"
                          value={entry.mode}
                          onChange={e => {
                            const mm = { ...editing.model_mapping, [entry.key]: { ...editing.model_mapping[entry.key], mode: e.target.value } };
                            const u = { ...editing, model_mapping: mm };
                            setEditing(u);
                            onSave(u);
                          }}
                        >
                          <option value="thinking">{t('settings.api.mode_thinking')}</option>
                          <option value="normal">{t('settings.api.mode_normal')}</option>
                        </select>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
              </>
              )}

              {/* Actions */}
              <div className="provider-actions">
                <button className="btn btn-secondary" onClick={onTest} disabled={isTesting}>
                  {isTesting ? t('settings.api.status_testing') : t('settings.api.test_connection')}
                </button>
                {!provider.is_default && (
                  <button className="btn btn-secondary" onClick={onSetDefault}>
                    {t('settings.api.set_default')}
                  </button>
                )}
                {provider.is_default && (
                  <span className="badge badge-active" style={{ fontSize: 12 }}>{t('settings.api.default_badge')}</span>
                )}
                {testResult && (
                  <span style={{ display: 'flex', alignItems: 'center', gap: 4, fontSize: 13 }}>
                    {testResult.success ? (
                      <span style={{ color: 'var(--color-secondary)' }}>✓ {testResult.message}{testResult.latency_ms ? ` (${testResult.latency_ms}ms)` : ''}</span>
                    ) : (
                      <span style={{ color: 'var(--color-error)' }}>✗ {testResult.message}</span>
                    )}
                  </span>
                )}
                <span className="spacer" style={{ flex: 1 }} />
              </div>
            </div>
          </td>
        </tr>
      )}
    </>
  );
}

// --- Preset Settings ---

interface PresetDef {
  id: string;
  name: string;
  desc: string;
  tag: string;
  systemPrompt: string;
  userPromptTemplate: string;
}

function buildDefaultPresets(t: (path: string) => string): PresetDef[] {
  return [
    { id: 'news', name: t('preset_names.news'), desc: t('preset_descs.news'), tag: t('preset_tags.news'),
      systemPrompt: 'あなたはプロのニュース翻訳者です。\n英文ニュースを、自然で読みやすく、正確な日本語に翻訳してください。',
      userPromptTemplate: '以下の英文を日本語に翻訳してください。意味を省略せず、数字・日付・固有名詞を正確に保ってください。解説や要約は加えず、翻訳文のみ出力してください。\n\n{{text}}',
    },
    { id: 'academic', name: t('preset_names.academic'), desc: t('preset_descs.academic'), tag: t('preset_tags.academic'),
      systemPrompt: 'あなたは学術論文の翻訳に熟練した専門翻訳者です。\n専門用語、概念、論理構造を保ちながら、自然な日本語に翻訳してください。',
      userPromptTemplate: '以下の英文を日本語に翻訳してください。学術用語、引用、数値、方法論の記述を正確に保ってください。解説や要約は加えず、翻訳文のみ出力してください。\n\n{{text}}',
    },
    { id: 'technical', name: t('preset_names.technical'), desc: t('preset_descs.technical'), tag: t('preset_tags.technical'),
      systemPrompt: 'あなたは技術文書とソフトウェアドキュメントの翻訳者です。\n技術用語、コマンド、コード、API名を正確に保って翻訳してください。',
      userPromptTemplate: '以下の技術文書を日本語に翻訳してください。コード、コマンド、設定値、API名、パス名は不要に翻訳せず保持してください。翻訳文のみ出力してください。\n\n{{text}}',
    },
    { id: 'email', name: t('preset_names.email'), desc: t('preset_descs.email'), tag: t('preset_tags.email'),
      systemPrompt: 'あなたはビジネスメール翻訳者です。\n自然で丁寧な日本語メールとして読めるように翻訳してください。',
      userPromptTemplate: '以下の英文メールを日本語に翻訳してください。相手に失礼のない自然な敬体にしてください。翻訳文のみ出力してください。\n\n{{text}}',
    },
    { id: 'subtitle', name: t('preset_names.subtitle'), desc: t('preset_descs.subtitle'), tag: t('preset_tags.subtitle'),
      systemPrompt: 'あなたは映像字幕の翻訳者です。\n短く、自然で、読みやすい日本語に翻訳してください。',
      userPromptTemplate: '以下の字幕テキストを日本語に翻訳してください。冗長な表現を避け、自然な会話として読める訳にしてください。翻訳文のみ出力してください。\n\n{{text}}',
    },
    { id: 'natural', name: t('preset_names.natural'), desc: t('preset_descs.natural'), tag: t('preset_tags.natural'),
      systemPrompt: 'あなたは自然な日本語表現に優れた翻訳者です。',
      userPromptTemplate: '以下の英文を、直訳調にならない自然な日本語に翻訳してください。翻訳文のみ出力してください。\n\n{{text}}',
    },
    { id: 'literal', name: t('preset_names.literal'), desc: t('preset_descs.literal'), tag: t('preset_tags.literal'),
      systemPrompt: 'あなたは原文の構造と意味を忠実に保つ翻訳者です。',
      userPromptTemplate: '以下の英文を、原文の語順や構造をできるだけ保ちながら日本語に翻訳してください。翻訳文のみ出力してください。\n\n{{text}}',
    },
    { id: 'formal', name: t('preset_names.formal'), desc: t('preset_descs.formal'), tag: t('preset_tags.formal'),
      systemPrompt: 'あなたはフォーマルな文章表現に適した翻訳者です。',
      userPromptTemplate: '以下の英文を、改まった場面に適した自然な日本語に翻訳してください。翻訳文のみ出力してください。\n\n{{text}}',
    },
    { id: 'casual', name: t('preset_names.casual'), desc: t('preset_descs.casual'), tag: t('preset_tags.casual'),
      systemPrompt: 'あなたはカジュアルで自然な日本語表現に適した翻訳者です。',
      userPromptTemplate: '以下の英文を、くだけすぎない範囲でカジュアルな日本語に翻訳してください。翻訳文のみ出力してください。\n\n{{text}}',
    },
    { id: 'friendly', name: t('preset_names.friendly'), desc: t('preset_descs.friendly'), tag: t('preset_tags.friendly'),
      systemPrompt: 'あなたは親しみやすく、柔らかい日本語表現に適した翻訳者です。',
      userPromptTemplate: '以下の英文を、親しみやすく、読み手にやわらかく伝わる日本語に翻訳してください。翻訳文のみ出力してください。\n\n{{text}}',
    },
  ];
}

function PresetSettings() {
  const { t } = useT();
  const [presetList, setPresetList] = useState<PresetDef[]>(() => buildDefaultPresets(t));
  const [selectedId, setSelectedId] = useState('news');
  const [search, setSearch] = useState('');

  const selectedPreset = presetList.find(p => p.id === selectedId) ?? presetList[0];

  const [draft, setDraft] = useState({
    name: selectedPreset.name,
    desc: selectedPreset.desc,
    systemPrompt: selectedPreset.systemPrompt,
    userPromptTemplate: selectedPreset.userPromptTemplate,
  });

  useEffect(() => {
    const p = presetList.find(p => p.id === selectedId);
    if (!p) return;
    setDraft({
      name: p.name,
      desc: p.desc,
      systemPrompt: p.systemPrompt,
      userPromptTemplate: p.userPromptTemplate,
    });
  }, [selectedId, presetList]);

  // Auto-save draft changes (debounced 400ms)
  const autoSaveRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const draftRef = useRef(draft);
  draftRef.current = draft;
  useEffect(() => {
    if (autoSaveRef.current) clearTimeout(autoSaveRef.current);
    autoSaveRef.current = setTimeout(() => {
      const d = draftRef.current;
      setPresetList(prev =>
        prev.map(p =>
          p.id === selectedId
            ? { ...p, name: d.name, desc: d.desc, systemPrompt: d.systemPrompt, userPromptTemplate: d.userPromptTemplate }
            : p
        )
      );
    }, 400);
    return () => {
      if (autoSaveRef.current) clearTimeout(autoSaveRef.current);
    };
  }, [draft.name, draft.desc, draft.systemPrompt, draft.userPromptTemplate, selectedId]);

  const filtered = presetList.filter(p =>
    !search || p.name.toLowerCase().includes(search.toLowerCase()) || p.desc.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="settings-section">
      <h3>{t('settings.presets.title')}</h3>
      <div style={{ display: 'flex', gap: 24, alignItems: 'flex-start' }}>
        {/* List */}
        <div style={{ width: 280, flexShrink: 0 }}>
          <div className="search-wrapper" style={{ marginBottom: 12 }}>
            <span className="icon">🔍</span>
            <input className="search-input" placeholder={t('settings.presets.search')} value={search} onChange={e => setSearch(e.target.value)} />
          </div>
          <div className="card" style={{ maxHeight: 400, overflow: 'auto' }}>
            {filtered.map(p => (
              <div
                key={p.id}
                className="history-item"
                style={{
                  cursor: 'pointer',
                  borderLeft: selectedId === p.id ? '4px solid var(--color-primary)' : '4px solid transparent',
                  background: selectedId === p.id ? 'var(--color-primary-fixed)' : undefined,
                }}
                onClick={() => setSelectedId(p.id)}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <strong style={{ fontSize: 14 }}>{p.name}</strong>
                  <span className="badge badge-beta">{p.tag}</span>
                </div>
                <div style={{ fontSize: 12, color: 'var(--color-outline)', marginTop: 4 }}>{p.desc}</div>
              </div>
            ))}
          </div>
        </div>

        {/* Detail */}
        <div style={{ flex: 1 }}>
          <div className="settings-group">
            <div style={{ marginBottom: 12 }}>
              <label className="settings-label">{t('settings.presets.name')}</label>
              <input
                className="input"
                style={{ width: '100%', marginTop: 4 }}
                value={draft.name}
                onChange={e => setDraft({ ...draft, name: e.target.value })}
              />
            </div>
            <div style={{ marginBottom: 12 }}>
              <label className="settings-label">{t('settings.presets.description')}</label>
              <input
                className="input"
                style={{ width: '100%', marginTop: 4 }}
                value={draft.desc}
                onChange={e => setDraft({ ...draft, desc: e.target.value })}
              />
            </div>
            <div style={{ marginBottom: 12 }}>
              <label className="settings-label">{t('settings.presets.system_prompt')}</label>
              <textarea
                className="pane-textarea"
                style={{
                  width: '100%',
                  height: 120,
                  marginTop: 4,
                  border: '1px solid var(--color-outline-variant)',
                  borderRadius: 'var(--radius-sm)',
                  padding: 12,
                  fontFamily: 'var(--font-body)',
                  fontSize: 14,
                }}
                value={draft.systemPrompt}
                onChange={e => setDraft({ ...draft, systemPrompt: e.target.value })}
              />
            </div>
            <div>
              <label className="settings-label">{t('settings.presets.user_prompt_template')}</label>
              <textarea
                className="pane-textarea"
                style={{
                  width: '100%',
                  height: 120,
                  marginTop: 4,
                  border: '1px solid var(--color-outline-variant)',
                  borderRadius: 'var(--radius-sm)',
                  padding: 12,
                  fontFamily: 'var(--font-body)',
                  fontSize: 14,
                }}
                value={draft.userPromptTemplate}
                onChange={e => setDraft({ ...draft, userPromptTemplate: e.target.value })}
              />
              <p className="field-help" style={{ marginTop: 4 }}>
                {t('settings.presets.placeholder_hint')}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// --- History Settings ---
function HistorySettings({ config, onUpdateHistory }: {
  config: AppConfig;
  onUpdateHistory: (u: Partial<AppConfig['history']>) => void;
}) {
  const { t } = useT();
  return (
    <div className="settings-section">
      <h3>{t('settings.history.title')}</h3>
      <div className="settings-group">
        <div className="settings-row">
          <div>
            <div className="settings-label">{t('settings.general.history_enabled')}</div>
            <div className="settings-description">{t('settings.general.history_enabled_desc')}</div>
          </div>
          <div className="settings-control">
            <button className={`toggle ${config.history.enabled ? 'active' : ''}`} onClick={() => onUpdateHistory({ enabled: !config.history.enabled })} />
          </div>
        </div>
        <div className="settings-row">
          <div>
            <div className="settings-label">{t('settings.history.max_items')}</div>
          </div>
          <div className="settings-control">
            <input
              className="input"
              type="number"
              style={{ width: 100 }}
              value={config.history.max_items}
              onChange={e => onUpdateHistory({ max_items: parseInt(e.target.value) || 100 })}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
