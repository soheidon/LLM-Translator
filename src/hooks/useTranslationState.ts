import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import type { TranslationResponse, TranslationState } from '../types/translation';

export function useTranslationState() {
  const [state, setState] = useState<TranslationState>({
    sourceText: '',
    translatedText: '',
    isTranslating: false,
    error: null,
    lastResponse: null,
  });

  const setSourceText = useCallback((text: string) => {
    setState(s => ({ ...s, sourceText: text, error: null }));
  }, []);

  const translate = useCallback(async (params: {
    text?: string;
    source_lang?: string;
    target_lang: string;
    mode: string;
    tone: string;
    preset_id?: string;
    provider_id?: string;
    model?: string;
    system_prompt?: string;
  }) => {
    const text = params.text ?? state.sourceText;
    if (!text.trim()) {
      setState(s => ({ ...s, error: 'errors.empty_text' }));
      return;
    }

    setState(s => ({ ...s, isTranslating: true, error: null }));

    try {
      const response = await invoke<TranslationResponse>('translate', {
        text: text.trim(),
        sourceLang: params.source_lang === 'auto' ? null : params.source_lang,
        targetLang: params.target_lang,
        mode: params.mode,
        tone: params.tone,
        presetId: params.preset_id || null,
        providerId: params.provider_id || null,
        model: params.model || null,
        systemPrompt: params.system_prompt || null,
      });

      setState(s => ({
        ...s,
        sourceText: text,
        translatedText: response.translated_text,
        isTranslating: false,
        lastResponse: response,
        error: null,
      }));

      return response;
    } catch (e: any) {
      const msg = typeof e === 'string' ? e : e?.message || 'errors.translation_error';
      setState(s => ({ ...s, isTranslating: false, error: msg }));
      throw e;
    }
  }, [state.sourceText]);

  const translateFromClipboard = useCallback(async (params: {
    source_lang?: string;
    target_lang: string;
    mode: string;
    tone: string;
    preset_id?: string;
    provider_id?: string;
    model?: string;
  }) => {
    try {
      const text = await readText();
      if (!text || !text.trim()) {
        setState(s => ({ ...s, error: 'errors.empty_clipboard' }));
        return;
      }
      setState(s => ({ ...s, sourceText: text }));
      return await translate({ ...params, text });
    } catch (e: any) {
      setState(s => ({ ...s, error: 'errors.clipboard_read_failed' }));
    }
  }, [translate]);

  const clear = useCallback(() => {
    setState({
      sourceText: '',
      translatedText: '',
      isTranslating: false,
      error: null,
      lastResponse: null,
    });
  }, []);

  return {
    ...state,
    setSourceText,
    translate,
    translateFromClipboard,
    clear,
  };
}
