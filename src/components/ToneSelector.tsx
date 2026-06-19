import { useT } from '../i18n/I18nContext';

interface Props {
  value: string;
  onChange: (val: string) => void;
}

const TONE_IDS = ['auto', 'plain', 'polite'] as const;

export function ToneSelector({ value, onChange }: Props) {
  const { t } = useT();
  return (
    <select className="select tone-select" value={value} onChange={e => onChange(e.target.value)}>
      {TONE_IDS.map(tid => (
        <option key={tid} value={tid}>{t(`tones.${tid}`)}</option>
      ))}
    </select>
  );
}
