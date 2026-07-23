export type ChatgptLanguage = {
  code: string;
  nameJa: string;
  nameEn: string;
};

// 47 languages supported by ChatGPT Translate.
// Keep in sync with aliasesForLabel / findOption in commands.rs IIFE.
export const CHATGPT_LANGUAGES: ChatgptLanguage[] = [
  { code: 'ar', nameJa: 'アラビア語', nameEn: 'Arabic' },
  { code: 'bn', nameJa: 'ベンガル語', nameEn: 'Bengali' },
  { code: 'my', nameJa: 'ビルマ語', nameEn: 'Burmese' },
  { code: 'zh-CN', nameJa: '中国語（簡体字）', nameEn: 'Chinese (Simplified)' },
  { code: 'zh-HK', nameJa: '中国語（繁体字、香港）', nameEn: 'Chinese (Traditional, Hong Kong)' },
  { code: 'zh-TW', nameJa: '中国語（繁体字、台湾）', nameEn: 'Chinese (Traditional, Taiwan)' },
  { code: 'cs', nameJa: 'チェコ語', nameEn: 'Czech' },
  { code: 'nl', nameJa: 'オランダ語', nameEn: 'Dutch' },
  { code: 'en', nameJa: '英語', nameEn: 'English' },
  { code: 'fi', nameJa: 'フィンランド語', nameEn: 'Finnish' },
  { code: 'fr', nameJa: 'フランス語', nameEn: 'French' },
  { code: 'de', nameJa: 'ドイツ語', nameEn: 'German' },
  { code: 'gu', nameJa: 'グジャラート語', nameEn: 'Gujarati' },
  { code: 'ha', nameJa: 'ハウサ語', nameEn: 'Hausa' },
  { code: 'hi', nameJa: 'ヒンディー語', nameEn: 'Hindi' },
  { code: 'hu', nameJa: 'ハンガリー語', nameEn: 'Hungarian' },
  { code: 'ig', nameJa: 'イボ語', nameEn: 'Igbo' },
  { code: 'id', nameJa: 'インドネシア語', nameEn: 'Indonesian' },
  { code: 'it', nameJa: 'イタリア語', nameEn: 'Italian' },
  { code: 'ja', nameJa: '日本語', nameEn: 'Japanese' },
  { code: 'jv', nameJa: 'ジャワ語', nameEn: 'Javanese' },
  { code: 'kn', nameJa: 'カンナダ語', nameEn: 'Kannada' },
  { code: 'ko', nameJa: '韓国語', nameEn: 'Korean' },
  { code: 'la', nameJa: 'ラテン語', nameEn: 'Latin' },
  { code: 'ms', nameJa: 'マレー語', nameEn: 'Malay' },
  { code: 'ml', nameJa: 'マラヤーラム語', nameEn: 'Malayalam' },
  { code: 'mr', nameJa: 'マラーティー語', nameEn: 'Marathi' },
  { code: 'or', nameJa: 'オリヤー語', nameEn: 'Odia' },
  { code: 'pl', nameJa: 'ポーランド語', nameEn: 'Polish' },
  { code: 'pt-BR', nameJa: 'ポルトガル語（ブラジル）', nameEn: 'Portuguese (Brazil)' },
  { code: 'pt-PT', nameJa: 'ポルトガル語（ポルトガル）', nameEn: 'Portuguese (Portugal)' },
  { code: 'pa', nameJa: 'パンジャブ語', nameEn: 'Punjabi' },
  { code: 'ro', nameJa: 'ルーマニア語', nameEn: 'Romanian' },
  { code: 'ru', nameJa: 'ロシア語', nameEn: 'Russian' },
  { code: 'si', nameJa: 'シンハラ語', nameEn: 'Sinhala' },
  { code: 'sk', nameJa: 'スロバキア語', nameEn: 'Slovak' },
  { code: 'es', nameJa: 'スペイン語', nameEn: 'Spanish' },
  { code: 'sv', nameJa: 'スウェーデン語', nameEn: 'Swedish' },
  { code: 'tl', nameJa: 'タガログ語', nameEn: 'Tagalog' },
  { code: 'ta', nameJa: 'タミル語', nameEn: 'Tamil' },
  { code: 'te', nameJa: 'テルグ語', nameEn: 'Telugu' },
  { code: 'th', nameJa: 'タイ語', nameEn: 'Thai' },
  { code: 'tr', nameJa: 'トルコ語', nameEn: 'Turkish' },
  { code: 'ur', nameJa: 'ウルドゥー語', nameEn: 'Urdu' },
  { code: 'vi', nameJa: 'ベトナム語', nameEn: 'Vietnamese' },
  { code: 'yo', nameJa: 'ヨルバ語', nameEn: 'Yoruba' },
  { code: 'zu', nameJa: 'ズールー語', nameEn: 'Zulu' },
];

export const CHATGPT_AUTO_LANGUAGE: ChatgptLanguage = {
  code: 'auto',
  nameJa: '言語を検出する',
  nameEn: 'Detect language',
} as const;

export function getChatgptLanguage(code: string): ChatgptLanguage | undefined {
  if (code === 'auto') return CHATGPT_AUTO_LANGUAGE;
  return CHATGPT_LANGUAGES.find(l => l.code === code);
}

export function resolveChatgptSourceLanguage(savedCode: string | null): string {
  if (savedCode === 'auto') return 'auto';
  if (savedCode && CHATGPT_LANGUAGES.some(l => l.code === savedCode)) return savedCode;
  return 'auto';
}

export function resolveChatgptTargetLanguage(savedCode: string | null): string {
  if (savedCode && CHATGPT_LANGUAGES.some(l => l.code === savedCode)) return savedCode;
  return 'ja';
}
