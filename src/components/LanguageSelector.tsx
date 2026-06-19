import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { LanguageInfo } from '../types/translation';
import { useT } from '../i18n/I18nContext';

interface Props {
  value: string;
  onChange: (val: string) => void;
  allowAuto?: boolean;
}

export function LanguageSelector({ value, onChange, allowAuto }: Props) {
  const [languages, setLanguages] = useState<LanguageInfo[]>([]);
  const { t } = useT();

  useEffect(() => {
    invoke<LanguageInfo[]>('get_languages').then(setLanguages).catch(console.error);
  }, []);

  const filtered = allowAuto
    ? languages
    : languages.filter(l => l.code !== 'auto');

  return (
    <select className="select language-select" value={value} onChange={e => onChange(e.target.value)}>
      {filtered.map(l => (
        <option key={l.code} value={l.code}>{t(`languages.${l.code}`)}</option>
      ))}
    </select>
  );
}
