import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { useT } from '../i18n/I18nContext';
import type { HistoryEntry } from '../types/translation';

interface Props {
  onRetranslate: (entry: HistoryEntry) => void;
  onClose: () => void;
}

const PAGE_SIZE = 20;

export function HistoryPanel({ onRetranslate, onClose }: Props) {
  const { t, language } = useT();
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [search, setSearch] = useState('');
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const [loading, setLoading] = useState(false);

  const loadEntries = useCallback(async (reset = false) => {
    setLoading(true);
    try {
      const currentOffset = reset ? 0 : offset;
      const result = await invoke<HistoryEntry[]>('get_history', {
        offset: currentOffset,
        limit: PAGE_SIZE,
        search: search || null,
      });
      if (reset) {
        setEntries(result);
        setOffset(PAGE_SIZE);
      } else {
        setEntries(prev => [...prev, ...result]);
        setOffset(prev => prev + PAGE_SIZE);
      }
      setHasMore(result.length === PAGE_SIZE);
    } catch (e) {
      console.error('Failed to load history:', e);
    }
    setLoading(false);
  }, [offset, search]);

  useEffect(() => {
    loadEntries(true);
  }, [search]);

  const handleDelete = useCallback(async (id: string) => {
    try {
      await invoke('delete_history', { id });
      setEntries(prev => prev.filter(e => e.id !== id));
    } catch (e) {
      console.error('Failed to delete:', e);
    }
  }, []);

  const handleCopy = useCallback(async (text: string) => {
    try {
      await writeText(text);
    } catch (e) {
      console.error('Copy failed:', e);
    }
  }, []);

  const handleClearAll = useCallback(async () => {
    if (!confirm(t('history_panel.confirm_delete_all'))) return;
    try {
      await invoke('clear_all_history');
      setEntries([]);
    } catch (e) {
      console.error('Failed to clear history:', e);
    }
  }, []);

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
      {/* Header */}
      <div style={{ padding: '0 24px 24px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
          <h2 style={{ fontFamily: 'var(--font-body)', fontSize: 24, fontWeight: 700, color: 'var(--color-on-surface)' }}>
            {t('history_panel.title')}
          </h2>
          <div style={{ display: 'flex', gap: 8 }}>
            <button className="btn btn-danger" onClick={handleClearAll}>{t('history_panel.delete_all')}</button>
            <button className="btn btn-secondary" onClick={onClose}>{t('history_panel.close')}</button>
          </div>
        </div>
        <div className="search-wrapper" style={{ maxWidth: 400 }}>
          <span className="icon">🔍</span>
          <input
            className="search-input"
            placeholder={t('history_panel.search')}
            value={search}
            onChange={e => setSearch(e.target.value)}
          />
        </div>
      </div>

      {/* List */}
      <div style={{ flex: 1, overflow: 'auto', padding: '0 24px 24px' }}>
        <div className="card">
          {entries.length === 0 && !loading ? (
            <div style={{ padding: 48, textAlign: 'center', color: 'var(--color-outline)' }}>
              {t('history_panel.empty')}
            </div>
          ) : (
            entries.map(entry => (
              <div key={entry.id} className="history-item">
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                  <div className="history-meta">
                    <span className="badge-model">{entry.model}</span>
                    <span>{new Date(entry.created_at).toLocaleString(language)}</span>
                  </div>
                  <div className="history-actions">
                    <button className="btn btn-ghost btn" onClick={() => onRetranslate(entry)} title={t('history_panel.retranslate')}>
                      🔄 {t('history_panel.retranslate')}
                    </button>
                    <button className="btn btn-ghost btn" onClick={() => handleCopy(entry.translated_text)} title={t('history_panel.copy')}>
                      📋 {t('history_panel.copy')}
                    </button>
                    <button className="btn btn-danger btn" onClick={() => handleDelete(entry.id)} title="削除">
                      🗑
                    </button>
                  </div>
                </div>
                <div className="history-texts">
                  <div className="history-text-block">
                    <div className="history-text-label">{t('history_panel.source_label', { lang: entry.source_lang.toUpperCase() })}</div>
                    <div style={{ maxHeight: 120, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                      {entry.source_text}
                    </div>
                  </div>
                  <div className="history-text-block">
                    <div className="history-text-label">{t('history_panel.target_label', { lang: entry.target_lang.toUpperCase() })}</div>
                    <div style={{ maxHeight: 120, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                      {entry.translated_text}
                    </div>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>

        {hasMore && entries.length > 0 && (
          <div style={{ display: 'flex', justifyContent: 'center', marginTop: 24 }}>
            <button className="btn btn-secondary" onClick={() => loadEntries(false)} disabled={loading}>
              {loading ? t('history_panel.loading') : t('history_panel.load_more')}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
