# LLM Translator Desktop 仕様書

## 1. アプリ概要

### 1.1 アプリ名
LLM Translator Desktop

### 1.2 目的
DeepL Desktop と同様に、ユーザーが任意のアプリ上でテキストを選択し Ctrl+C+C を押すだけで、クリップボード上のテキストを LLM API に送信し、訳文を翻訳ウィンドウに表示する常駐型デスクトップアプリ。

### 1.3 対応 OS
- Windows 10 / Windows 11
- ※ macOS / Linux は将来対応

---

## 2. 基本ユーザーフロー

```
任意のアプリでテキストを選択
↓
Ctrl+C を2回押す（または Ctrl+Shift+C）
↓
アプリがクリップボードのテキストを取得
↓
翻訳 API へ送信
↓
翻訳ウィンドウを表示
↓
訳文を読む／コピーする／再翻訳する
```

---

## 3. 技術スタック

| レイヤー | 技術 |
|---------|------|
| デスクトップフレームワーク | Tauri v2 |
| バックエンド | Rust |
| フロントエンド | React + TypeScript |
| ビルドツール | Vite |
| 主なTauriプラグイン | global-shortcut, clipboard-manager, shell, store, log |
| HTTP通信 | reqwest（Rust側） |
| Windows API | SetWindowsHookEx（キーボードフック） |

---

## 4. 主要機能

### 4.1 Ctrl+C+C 翻訳

Windows の低レベルキーボードフック (`SetWindowsHookEx(WH_KEYBOARD_LL)`) を使用し、OS 全体で Ctrl+C の2回押しを検知する。
検知後、クリップボードからテキストを読み取り翻訳を開始する。

- 判定時間: デフォルト 800ms（設定で変更可能）
- 設定で ON/OFF 切替可能

### 4.2 クリップボード監視

400ms 間隔でクリップボードをポーリングし、変更があれば 600ms のデバウンス後に自動翻訳する。
Ctrl+C+C 有効時はポーリングを停止し、二重発火を防止する。

### 4.3 メイン翻訳画面

- 原文・訳文の左右2ペイン表示
- 原文ペイン: 直接入力、文字数表示、クリア
- 訳文ペイン: 再翻訳、コピー
- 翻訳元言語（自動検出含む）・翻訳先言語の選択
- 言語入れ替え
- モデル・文体・プリセット選択

### 4.4 設定画面

#### 一般設定
- UI言語（11言語）
- 起動時に最小化
- 常に前面表示
- 翻訳時に前面表示
- Esc で閉じる
- 外側クリックで閉じる
- Ctrl+C+C クイック翻訳 ON/OFF
- 代替ショートカット設定
- 翻訳ウィンドウを開くショートカット
- 履歴を開くショートカット
- 翻訳履歴保存 ON/OFF
- 通知音

#### API設定
- 12プロバイダの表形式管理（Provider, Status）
- プロバイダ展開時の詳細設定: 環境変数名, APIキー, Base URL, 接続テスト
- 機能役割別モデルマッピング（default, fast）
- モデルごとの動作モード（thinking, normal）
- Ollama: ローカルモデル一覧取得・選択
- Google Translate: Cloud API / Apps Script 両対応
- DeepL: Free / Pro 両対応
- 環境変数は `setx` + レジストリフォールバックで読み取り

#### プリセット設定
- 10種類の組み込みプリセット（ニュース/論文/技術文書/メール/字幕/自然訳/直訳/フォーマル/カジュアル/フレンドリー）
- 検索、名称・説明・システムプロンプト編集

#### 履歴設定
- 最大保存件数設定

### 4.5 履歴画面
- 原文・訳文の左右表示
- 検索
- 再翻訳、コピー
- Load More 追加読み込み
- 全削除

### 4.6 タスクトレイ
- 常駐アイコンと右クリックメニュー
- メインウィンドウ表示 / 翻訳 / 履歴 / 終了

---

## 5. 対応翻訳プロバイダ

| プロバイダ | API種別 | デフォルトモデル |
|-----------|---------|-------------|
| Google Translate / Cloud | Google Cloud Translation API | google-translate-v2 |
| DeepL / Free | DeepL API | deepl |
| DeepL / Pro | DeepL API | deepl |
| OpenAI / ChatGPT | OpenAI互換 | gpt-5.5 |
| Gemini / Google | OpenAI互換 | gemini-3.1-pro |
| Claude / Anthropic | Anthropic互換 | claude-opus-4-8 |
| MiMo / Xiaomi | OpenAI互換 | mimo-v2.5-pro |
| DeepSeek / DeepSeek | OpenAI互換 | deepseek-v4-pro |
| Kimi / Moonshot | OpenAI互換 | kimi-k2.7-code |
| Qwen / Alibaba | OpenAI互換 | qwen3.7-max |
| MiniMAX / MiniMAX | OpenAI互換 | MiniMax-M2.7 |
| Ollama / Local | OpenAI互換（ローカル） | - |
| Google Translate / Apps Script | Google Apps Script | google-apps-script |

---

## 6. 対応 UI 言語

日本語, English, 中文(简体), 中文(繁體), 한국어, Français, Deutsch, Español, Português, Русский, Italiano

---

## 7. セキュリティ

- API キーは OS 環境変数に保存、設定ファイルに含めない
- フロントエンドに API キーを渡さない（Rust 側で保持）
- HTTPS 通信（ローカル Ollama のみ HTTP 許可）
- 履歴保存はユーザーが ON/OFF 可能

---

## 8. データ保存

```
%APPDATA%/LLMTranslator/
├── settings.json    # 設定ファイル
└── history.jsonl    # 翻訳履歴（JSON Lines）
```

---

## 9. ファイル構成

```
├── index.html
├── package.json
├── README.md
├── SPEC.md
├── tsconfig.json
├── vite.config.ts
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/
│   │   ├── MainTranslate.tsx
│   │   ├── SettingsPanel.tsx
│   │   ├── StatusBar.tsx
│   │   ├── HistoryPanel.tsx
│   │   ├── LanguageSelector.tsx
│   │   └── ToneSelector.tsx
│   ├── hooks/
│   │   ├── useSettings.ts
│   │   └── useTranslationState.ts
│   ├── i18n/
│   │   └── I18nContext.tsx
│   ├── lang/
│   │   ├── index.ts
│   │   ├── en.json, ja.json, zh-CN.json, zh-TW.json,
│   │   │   ko.json, fr.json, de.json, es.json,
│   │   │   pt.json, ru.json, it.json
│   ├── types/
│   │   ├── settings.ts
│   │   ├── translation.ts
│   │   └── provider.ts
│   └── styles/
│       └── app.css
├── src-tauri/
│   ├── tauri.conf.json
│   ├── Cargo.toml
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── config.rs
│       ├── commands.rs
│       ├── history.rs
│       ├── translator.rs
│       ├── keyboard_hook.rs
│       ├── tray.rs
│       └── providers/
│           ├── mod.rs
│           ├── openai_compat.rs
│           ├── anthropic_compat.rs
│           ├── deepl.rs
│           └── google_translate.rs
```

---

## 10. v0.1.2 実装済み機能

- [x] Windows 10/11 対応
- [x] Tauri v2 + React + TypeScript + Rust
- [x] タスクトレイ常駐
- [x] Ctrl+C+C グローバルキーボードフック（DeepL方式）
- [x] Ctrl+Shift+C グローバルショートカット
- [x] クリップボード監視
- [x] 13プロバイダ対応（OpenAI/Claude/Gemini/DeepSeek/MiMo/Kimi/Qwen/MiniMAX/Ollama/DeepL/Google Translate）
- [x] メイン翻訳画面（2ペイン）
- [x] 訳文コピー
- [x] 設定画面（一般/API/プリセット/履歴）
- [x] 11言語UI
- [x] 10種類の翻訳プリセット
- [x] 3種の文体（自動/常体/敬体）
- [x] API接続テストと状態表示
- [x] 履歴保存・検索・再翻訳
- [x] 環境変数ベースのAPIキー管理
- [x] 常に前面表示
- [x] 起動時最小化

---

## 11. 将来計画

### v0.2
- macOS 対応検討
- 自動アップデート
- エクスポート/インポート設定

### v0.3
- OCR翻訳
- 用語集
- Markdown保持
- 複数エンジン比較翻訳
