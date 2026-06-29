[English](../../SPEC.md) | [日本語](SPEC.md) | [中文(简体)](../zh-CN/SPEC.md) | [한국어](../ko/SPEC.md) | [Français](../fr/SPEC.md) | [Deutsch](../de/SPEC.md) | [Español](../es/SPEC.md)

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

- 判定方式: Ctrl を押しっぱなしの同一セッション内で C を2回押した場合のみ発火
- Ctrl keyup でセッションリセット（誤発火防止）
- Shift/Alt/Win 同時押しは判定対象外（Ctrl+Shift+C は global_shortcut 側で処理）
- Cキー押しっぱなしのリピートは除外
- 判定時間: デフォルト 400ms（設定で変更可能）
- 設定で ON/OFF 切替可能（変更後はアプリ再起動が必要）

### 4.2 クリップボード監視

※ v0.3.0 で廃止。Ctrl+C+C 翻訳とグローバルショートカットのみで翻訳を開始する。

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
- 起動時に最小化（デフォルトOFF: 初回起動時はウィンドウを表示）
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
- 右クリックメニューは「終了」のみ
- 左クリックでメインウィンドウを表示
- Xボタンでウィンドウを閉じるとトレーに最小化（アプリは終了しない）
- トレー右クリック → 「終了」でアプリ完全終了
- 多重起動防止: 2回目の起動時は既存インスタンスを前面表示

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
│   │   ├── TabBar.tsx
│   │   └── ToneSelector.tsx
│   ├── data/
│   │   └── googleTranslateLanguages.ts
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
│   │   ├── default.json
│   │   └── google-translate.json
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

## 10. 変更履歴

### v0.3.4

- [x] Windows 10/11 対応
- [x] Tauri v2 + React + TypeScript + Rust
- [x] タスクトレイ常駐（Xボタンでトレー最小化、右クリック「終了」で完全終了）
- [x] Ctrl+C+C グローバルキーボードフック（Ctrl押しっぱなしセッション方式、誤発火防止）
- [x] Ctrl+Shift+C グローバルショートカット
- [x] 13プロバイダ対応（OpenAI/Claude/Gemini/DeepSeek/MiMo/Kimi/Qwen/MiniMAX/Ollama/DeepL/Google Translate）
- [x] タブ式翻訳画面（LLM / Google翻訳 / chatGPT翻訳）
- [x] タブ選択メモリ（最後のタブを localStorage に保存し次回起動時に復元）
- [x] Google翻訳タブ：Tauri WebView埋め込み、ソース/ターゲット言語設定、ブラウザ操作バー
- [x] chatGPT翻訳タブ：ChatGPT Translate WebView埋め込み、翻訳元・翻訳先言語の設定画面からの指定
- [x] chatGPT翻訳ブラウザ操作バー（再読み込み/ホーム、トップページ以外で表示）
- [x] メイン翻訳画面（2ペイン）
- [x] 訳文コピー
- [x] 設定画面（一般/API/プリセット/履歴/Google翻訳/chatGPT翻訳）
- [x] 11言語UI（日本語・English・中文(簡体/繁体)・한국어・Français・Deutsch・Español・Português・Русский・Italiano）
- [x] 10種類の翻訳プリセット
- [x] 3種の文体（自動/常体/敬体）
- [x] API接続テストと状態表示
- [x] 履歴保存・検索・再翻訳
- [x] 環境変数ベースのAPIキー管理
- [x] 常に前面表示
- [x] 起動時最小化
- [x] バージョン表示（ステータスバー）
- [x] SVGアイコン移行（全サイドバーアイコン・ブラウザ操作バーアイコンをSVGに統一）
- [x] アプリアイコン表示（タイトルバー・サイドバーにアプリアイコンを表示）
- [x] 設定画面トップバーの整理（アイコン・タイトルテキスト削除）、右上×廃止
- [x] 設定ボタンUI改善（黒いカプセル型）、設定を閉じるボタンも統一
- [x] LLMタブラベル日本語対応（ja.json で "LLM翻訳" に）
- [x] chatGPT翻訳サジェストカード非表示（DOM診断ツール搭載、設定でON/OFF切替）
- [x] Ctrl+C+C 誤発火防止（keyboard_hook 全面書き換え：Ctrlセッション方式 + 他修飾キー除外 + Cキーリピート除外）
- [x] クリップボードポーリング廃止（Ctrl+C 1回での翻訳発火を完全排除）
- [x] 閉じる＝トレー最小化（window.hide()）、多重起動防止（single instance plugin）
- [x] システムトレーメニュー簡略化（「終了」のみ）
- [x] ChatGPT Translate 設定画面（翻訳元・翻訳先言語選択）
- [x] 各種デバッグログ（発火元特定用）
- [x] 設定のデフォルト値を最適化（double_copy_enabled 初期 true、threshold 400ms）
- [x] ステータスバーにデフォルトプロバイダのモデル名を短縮表示
- [x] API設定テーブルに Default 列を追加（行内で Set as Default ボタンも配置）
- [x] Google翻訳のトップページ判定を hostname + pathname ベースに改善（翻訳後もナビゲーションバー非表示）
- [x] 設定画面UI改善（タイトルバー常時表示、タブバー/ステータスバー非表示、←アイコン除去）
- [x] Windows自動起動（Registry HKCU Run キー、設定トグルでON/OFF、初期値OFF、引用符付きパス）
- [x] 起動時最小化の実装（`start_minimized` 設定を実際に反映、setup() で window.hide()）

### v0.3.5

- [x] ChatGPT翻訳サジェストカード非表示の安定化（遅延スケジュール8秒まで延長、`.prompt-card` CSS 常時非表示）
- [x] サジェストカード cleanup のデバッグログ追加（新規非表示カード数のみ出力、data属性で重複カウント防止）
- [x] `--auto-start` フラグによる自動起動と手動起動の分離
- [x] トレーアイコンダブルクリック復帰の修正（`SetWindowPos` + `SetForegroundWindow` 前面化、Click フォールバック、700ms クールダウン）
- [x] トレーアイコンイベントログ追加（全イベント種別をログ出力）

### v0.3.6

- [x] ChatGPT翻訳サジェストカード非表示を div 型カードにも対応（`button`/`a`/`[role="button"]` に加えて `div` もスキャン、AND条件の親コンテナ除外ガード付き）
- [x] DOM診断コマンドに子要素ダンプ機能追加（サジェスト文言を含む要素の直下 children を1階層出力）

### v0.3.7

- [x] NSIS インストーラーに言語選択ダイアログを追加（`displayLanguageSelector: true`、11言語対応、選択はレジストリに保存）
- [x] `start_minimized` デフォルト値を `false` に変更（初回起動時にウィンドウが表示されない問題を修正）
- [x] 設定画面の「起動時に最小化」ラベルと説明文を改善（通常起動時にトレーへ最小化）

### v0.3.8

- [x] ChatGPT翻訳タブ：不要なLP要素（ヘッダーナビ・マーケティングセクション）を非表示にする設定トグル追加
- [x] ChatGPT翻訳タブ：DOM診断ツール（StatusBarボタンで対話要素の候補を列挙、設定でON/OFF切替）
- [x] ChatGPT翻訳タブ：HTML+CSS診断ツール（ヘッダー/フッター/LP要素のCSS状態を調査、設定でON/OFF切替）
- [x] 診断ログの取得範囲を絞り込み（DOM診断：巨大ページラッパーを除外、HTML+CSS診断：翻訳フォーム本体を除外）
- [x] システムトレイ復帰の信頼性向上（トレイアイコンの生存期間をアプリ終了まで延長、hide後のWebviewWindowを保持してダブルクリック復帰を保証）

### v0.4.0

- [x] ChatGPT翻訳タブ：複数DOMバリアント対応 — マーケティングLP（variant A）とアプリ/ログイン版（variant B）の両方で翻訳フォームを適切に表示
- [x] CSSセレクタを `main#main` から `[data-llm-chatgpt-container="true"]` 属性ベースに変更し、variant A（`id="main"` 無し）でも翻訳UIが画面全体に表示されるよう修正
- [x] ログインボタン保持 — `#contentful-header` を丸ごと非表示にせず、マーケティング要素のみ非表示にしてログインボタンを残す44pxバーに変形
- [x] CTA非表示の強化 — 「ChatGPT で翻訳を開始する」「今すぐ試す」等のCTA要素をvariant Aでも検出・非表示化
- [x] ページ全体のスクロール抑制 — `html, body { overflow: hidden }` によりページレベルのスクロールバーを排除
- [x] スクロール位置リセット — layout markers設定後にスクロール位置をリセットし、翻訳フォームの画面外への飛び出しを修正
- [x] LPセクション非表示のバリアント対応 — `#contentful-header` と `[class*="h-mkt-header-height"]` の競合を解消し、安全にLP要素のみ非表示化

### v0.5
- macOS 対応検討
- 自動アップデート
- エクスポート/インポート設定
- OCR翻訳
- 用語集
- Markdown保持
- 複数エンジン比較翻訳
