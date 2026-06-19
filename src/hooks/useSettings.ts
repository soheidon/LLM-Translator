import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { AppConfig } from '../types/settings';
import type { ProviderConfig } from '../types/provider';

export function useSettings() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(true);

  const loadConfig = useCallback(async () => {
    try {
      const c = await invoke<AppConfig>('get_config');
      setConfig(c);
    } catch (e) {
      console.error('Failed to load config:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  const saveConfig = useCallback(async (newConfig: AppConfig) => {
    await invoke('save_config', { config: newConfig });
    setConfig(newConfig);
  }, []);

  const updateGeneral = useCallback(async (updates: Partial<AppConfig['general']>) => {
    if (!config) return;
    const newConfig = { ...config, general: { ...config.general, ...updates } };
    await saveConfig(newConfig);
    // Apply always_on_top to window immediately
    if ('always_on_top' in updates) {
      invoke('set_always_on_top', { alwaysOnTop: updates.always_on_top }).catch(() => {});
    }
  }, [config, saveConfig]);

  const updateTranslation = useCallback(async (updates: Partial<AppConfig['translation']>) => {
    if (!config) return;
    const newConfig = { ...config, translation: { ...config.translation, ...updates } };
    await saveConfig(newConfig);
  }, [config, saveConfig]);

  const updateShortcut = useCallback(async (updates: Partial<AppConfig['shortcut']>) => {
    if (!config) return;
    const newConfig = { ...config, shortcut: { ...config.shortcut, ...updates } };
    await saveConfig(newConfig);
  }, [config, saveConfig]);

  const updateHistory = useCallback(async (updates: Partial<AppConfig['history']>) => {
    if (!config) return;
    const newConfig = { ...config, history: { ...config.history, ...updates } };
    await saveConfig(newConfig);
  }, [config, saveConfig]);

  const saveProvider = useCallback(async (provider: ProviderConfig) => {
    await invoke('save_provider', { provider });
    await loadConfig();
  }, [loadConfig]);

  return {
    config,
    loading,
    saveConfig,
    updateGeneral,
    updateTranslation,
    updateShortcut,
    updateHistory,
    saveProvider,
    reload: loadConfig,
  };
}
