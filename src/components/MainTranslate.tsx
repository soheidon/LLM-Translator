import { useState, useEffect, useCallback, useRef } from 'react';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { LanguageSelector } from './LanguageSelector';
import { useT } from '../i18n/I18nContext';
import type { AppConfig } from '../types/settings';
import type { TranslationState } from '../types/translation';
import type { ModelRole } from '../types/model';

interface Props {
  config: AppConfig;
  translation: TranslationState & {
    setSourceText: (text: string) => void;
    translate: (params: any) => Promise<any>;
    translateFromClipboard: (params: any) => Promise<any>;
    clear: () => void;
  };
  mode: string;
  tone: string;
  providerId: string | null;
  modelRole: ModelRole;
}

export function MainTranslate({ config, translation, mode, tone, providerId, modelRole }: Props) {
  const { t } = useT();
  const { sourceText, translatedText, isTranslating, error, setSourceText, translate, clear } = translation;

  const [sourceLang, setSourceLang] = useState(config.translation.source_lang);
  const [targetLang, setTargetLang] = useState(config.translation.target_lang);
  const [copyFeedback, setCopyFeedback] = useState(false);

  useEffect(() => {
    setSourceLang(config.translation.source_lang);
    setTargetLang(config.translation.target_lang);
  }, [config.translation]);

  // Debounced auto-translate on text change
  const translateRef = useRef(translate);
  translateRef.current = translate;
  const paramsRef = useRef({ sourceLang, targetLang, mode, tone, providerId, modelRole });
  paramsRef.current = { sourceLang, targetLang, mode, tone, providerId, modelRole };
  const isTranslatingRef = useRef(isTranslating);
  isTranslatingRef.current = isTranslating;

  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  useEffect(() => {
    if (!sourceText.trim() || isTranslatingRef.current) return;
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      if (isTranslatingRef.current) return;
      const p = paramsRef.current;
      translateRef.current({
        text: sourceText,
        source_lang: p.sourceLang,
        target_lang: p.targetLang,
        mode: p.mode,
        tone: p.tone,
        provider_id: p.providerId || undefined,
        model_role: p.modelRole,
      });
    }, 600);
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [sourceText]);

  const handleTranslate = useCallback(async () => {
    if (!sourceText.trim()) return;
    await translate({
      text: sourceText,
      source_lang: sourceLang,
      target_lang: targetLang,
      mode,
      tone,
      provider_id: providerId || undefined,
      model_role: modelRole,
    });
  }, [sourceText, sourceLang, targetLang, mode, tone, providerId, modelRole, translate]);

  const handleSwapLanguages = useCallback(() => {
    if (sourceLang === 'auto') return;
    const newSource = targetLang;
    const newTarget = sourceLang;
    setSourceLang(newSource);
    setTargetLang(newTarget);
    if (translatedText) {
      setSourceText(translatedText);
    }
  }, [sourceLang, targetLang, translatedText, setSourceText]);

  const handleCopyTranslation = useCallback(async () => {
    if (!translatedText) return;
    try {
      await writeText(translatedText);
      setCopyFeedback(true);
      setTimeout(() => setCopyFeedback(false), 1500);
    } catch (e) {
      console.error('Copy failed:', e);
    }
  }, [translatedText]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      handleTranslate();
    }
  }, [handleTranslate]);

  return (
    <div className="translate-container">
      {/* ── Language Bar (左=自動検出, 中央=⇄, 右=日本語) ── */}
      <div className="language-bar">
        <div className="language-bar-left">
          <LanguageSelector value={sourceLang} onChange={setSourceLang} allowAuto />
        </div>
        <div className="language-bar-center">
          <button className="toolbar-swap" onClick={handleSwapLanguages} title={t('main.swap_languages')}>⇄</button>
        </div>
        <div className="language-bar-right">
          <LanguageSelector value={targetLang} onChange={setTargetLang} />
        </div>
      </div>

      {/* ── Error ── */}
      {error && (
        <div className="error-message">
          <span style={{ fontSize: '16px' }}>⚠</span>
          {t(error) !== error ? t(error) : error}
        </div>
      )}

      {/* ── 2-Pane Translation Area ── */}
      <div className="translation-area">
        <div className="source-pane">
          <textarea
            className="pane-textarea"
            value={sourceText}
            onChange={e => setSourceText(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder=""
            spellCheck={false}
          />
          {!sourceText && (
            <div className="pane-hint">
              {t('main.placeholder_hint')}
            </div>
          )}
        </div>
        <div className="target-pane">
          {isTranslating ? (
            <div className="loading-overlay">
              <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 12 }}>
                <div className="loading-spinner" />
                <span className="loading-label">{t('main.translating')}</span>
              </div>
            </div>
          ) : translatedText ? (
            <div className="pane-output">{translatedText}</div>
          ) : (
            <div className="pane-output pane-output-empty">
              {t('main.result_placeholder')}
            </div>
          )}
        </div>
      </div>

      {/* ── Pane Footers ── */}
      <div className="pane-footer">
        <div className="footer-left">
          <span className="pane-char-count">{t('main.char_count', { count: sourceText.length })}</span>
          <div className="pane-actions">
            <button className="btn btn-secondary" onClick={clear}>{t('main.button_clear')}</button>
          </div>
        </div>
        <div className="footer-right">
          <span className="pane-char-count" />
          <div className="pane-actions">
            <button className="btn btn-secondary" onClick={handleTranslate} disabled={isTranslating || !sourceText.trim()}>
              {t('main.button_retranslate')}
            </button>
            <button
              className={`btn ${copyFeedback ? 'btn-copy-done' : 'btn-copy'}`}
              onClick={handleCopyTranslation}
              disabled={!translatedText}
            >
              {copyFeedback ? t('main.copied') : t('main.button_copy')}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
