import { useT } from '../i18n/I18nContext';

export type TranslateTab = 'llm' | 'google' | 'chatgpt';

interface Props {
  activeTab: TranslateTab;
  onChangeTab: (tab: TranslateTab) => void;
}

const TABS: { key: TranslateTab; labelKey: string }[] = [
  { key: 'llm', labelKey: 'tabs.llm' },
  { key: 'google', labelKey: 'tabs.google' },
  { key: 'chatgpt', labelKey: 'tabs.chatgpt' },
];

export function TabBar({ activeTab, onChangeTab }: Props) {
  const { t } = useT();

  return (
    <div className="tab-bar">
      {TABS.map(tab => (
        <button
          key={tab.key}
          className={`tab-item${activeTab === tab.key ? ' active' : ''}`}
          onClick={() => onChangeTab(tab.key)}
        >
          {t(tab.labelKey)}
        </button>
      ))}
    </div>
  );
}
