# LLM Translator Desktop

DeepL Desktop のように、どのアプリでも Ctrl+C を2回押すだけでクリップボードのテキストを AI 翻訳する Windows デスクトップアプリです。

## 主な機能

- **Ctrl+C+C 翻訳** — 任意のアプリでテキストを選択し Ctrl+C を2回押すだけで翻訳
- **クリップボード監視** — コピーしたテキストを自動検出して翻訳
- **グローバルショートカット** — Ctrl+Shift+C で明示的に翻訳起動
- **多様な AI プロバイダ対応** — OpenAI, Claude, Gemini, DeepSeek, MiMo, Kimi, Qwen, MiniMAX, Ollama, DeepL, Google Translate ほか
- **11言語 UI** — 日本語・English・中文(簡体/繁体)・한국어・Français・Deutsch・Español・Português・Русский・Italiano
- **翻訳プリセット** — ニュース/論文/技術文書/メール/字幕/自然訳/直訳/フォーマル/カジュアル/フレンドリー
- **文体選択** — 自動/常体/敬体
- **履歴管理** — 翻訳履歴の保存・検索・再翻訳
- **タスクトレイ常駐** — 最小化でトレイに常駐、すぐに呼び出し可能
- **常に前面表示** — 翻訳ウィンドウを常に手前に表示
- **APIキー安全管理** — OS の環境変数を使用、設定ファイルにキーは保存しない
- **Google翻訳タブ** — Google翻訳をアプリ内WebViewに埋め込み、ソース/ターゲット言語を設定可能
- **chatGPT翻訳タブ** — ChatGPT Translateをアプリ内WebViewに埋め込み、翻訳元・翻訳先言語を設定画面から選択可能
- **ブラウザ操作バー** — Google翻訳・chatGPT翻訳タブで戻る/進む/再読み込み/ホームボタンを表示（トップページ以外で自動表示）
- **タブメモリ** — 最後に選択していたタブ（LLM/Google翻訳/chatGPT翻訳）を記憶し次回起動時に復元
- **SVGアイコン** — 全サイドバーアイコンをSVGに統一、アプリアイコンをタイトルバー・サイドバーに表示
- **Ctrl+C+C 誤発火防止** — Ctrl押しっぱなしセッション方式 + 他修飾キー除外 + Cキーリピート除外
- **閉じる＝トレー最小化** — Xボタンでアプリ終了せずシステムトレーに常駐、右クリック「終了」で完全終了
- **多重起動防止** — 既存インスタンスを前面表示、トレーアイコン重複防止
- **トレーメニュー簡略化** — 右クリックメニューは「終了」のみ、左クリックでウィンドウ表示
- **chatGPT翻訳サジェストカード非表示** — ChatGPT WebView内の不要な提案カードを自動非表示
- **chatGPT翻訳タブツールバー** — 再読み込み/ホームボタンをトップページ以外で表示
- **設定ボタンUI改善** — 黒いカプセル型ボタンに変更、設定画面の閉じるボタンも統一
- **設定画面の右上×を廃止** — サイドバーの「設定を閉じる」のみで遷移
- **DOM診断ツール** — chatGPT翻訳タブ用のDOM診断機能（設定画面でON/OFF切替）

## 必要環境

- Windows 10 / Windows 11
- 各 AI プロバイダの API キー

## インストール

[Releases](https://github.com/soheidon/LLM-Translator/releases) から最新の `LLM-Translator-Desktop-Setup.exe` をダウンロードして実行してください。

## 使い方

1. アプリを起動するとタスクトレイに常駐します
2. API 設定で使用したいプロバイダの API キーを設定してください（設定 → API）
3. 任意のアプリで翻訳したいテキストを選択し、Ctrl+C を2回押すと翻訳されます
4. または Ctrl+Shift+C でも翻訳できます
5. メイン画面では手動でテキストを入力・貼り付けして翻訳することもできます

## 開発

```bash
# 依存関係のインストール
npm install

# 開発モードで起動
npm run tauri dev

# ビルド
npm run tauri build
```

## 技術スタック

- [Tauri v2](https://tauri.app/) — デスクトップアプリケーションフレームワーク
- [Rust](https://www.rust-lang.org/) — バックエンド（API通信、キーボードフック、設定管理）
- [React](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) — フロントエンド UI
- [Vite](https://vitejs.dev/) — ビルドツール

## バージョン

v0.3.0

## ライセンス

MIT
