[English](README.md) | [日本語](docs/ja/README.md) | [中文(简体)](docs/zh-CN/README.md) | [中文(繁體)](docs/zh-TW/README.md) | [한국어](docs/ko/README.md) | [Français](docs/fr/README.md) | [Deutsch](docs/de/README.md) | [Español](docs/es/README.md)

# LLM Translator Desktop

一款 Windows 桌面快速翻譯工具 — 如同 DeepL Desktop，在任何應用程式中選取文字並按下 Ctrl+C+C 即可即時翻譯。內含 LLM 翻譯、Google 翻譯分頁及 ChatGPT 翻譯分頁，可依需求切換翻譯方式。

## 功能特色

### 共通功能

* **Ctrl+C+C 翻譯** — 在任何應用程式中選取文字，按兩次 Ctrl+C 即可將文字傳送至目前分頁進行翻譯。
* **全域快速鍵** — 使用 Ctrl+Shift+C（或自訂快速鍵）明確觸發翻譯。
* **系統匣** — 關閉視窗時最小化至系統匣而非結束程式。按兩下系統匣圖示可還原視窗，按右鍵 →「Exit」可完全結束。即使自動啟動後，視窗還原也能穩定運作。
* **單一執行個體** — 重複使用現有執行個體，避免重複的系統匣圖示。
* **隨 Windows 啟動** — 可選擇在登入 Windows 時自動啟動 LLM Translator（預設：關閉）。
* **歷史記錄** — 儲存、搜尋並重新翻譯您的翻譯歷史。
* **分頁記憶** — 記住上次選取的分頁，並在下次啟動時還原。
* **多語言介面** — 支援日文、英文、簡體中文、繁體中文、韓文、法文、德文、西班牙文、葡萄牙文、俄文及義大利文。

### LLM 翻譯

* **多種 LLM 供應商** — OpenAI、Claude、Gemini、DeepSeek、MiMo、Kimi、Qwen、MiniMAX、Ollama 等。
* **翻譯預設風格** — 可選擇新聞、學術、技術、電子郵件、字幕、自然、直譯、正式、隨意、友善等翻譯風格。
* **語氣選擇** — 日文輸出可使用自動、普通或禮貌語氣。
* **API 金鑰安全性** — API 金鑰儲存在作業系統環境變數中，絕不存放於應用程式設定檔內。
* **模型顯示** — 在狀態列中查看目前使用的供應商與模型。

### Google 翻譯分頁

* **內嵌 Google 翻譯** — 在應用程式內使用 Google 翻譯，無需切換至瀏覽器。
* **語言設定** — 從設定畫面設定來源語言與目標語言。
* **智慧導覽列** — 導覽按鈕（上一頁、下一頁、重新整理、首頁）僅在需要時顯示（例如登入頁面、外部頁面），在一般翻譯頁面則隱藏。
* **Ctrl+C+C 整合** — 將任何應用程式中選取的文字傳送至 Google 翻譯分頁。

### ChatGPT 翻譯分頁

* **內嵌 ChatGPT** — 在應用程式內開啟 ChatGPT 的網頁介面並傳送翻譯提示。
* **語言設定** — 從設定畫面設定來源語言與目標語言。
* **LP 元素隱藏** — 於設定中切換開/關。隱藏 ChatGPT 頁面的行銷導覽與區塊，讓您專注於翻譯表單。
* **多變體 DOM 支援** — 自動適應不同的 ChatGPT 頁面結構（行銷 LP 與應用程式/登入變體）。使用 `data-llm-chatgpt-container` 屬性型選擇器取代 `main#main`，保留登入按鈕同時僅隱藏行銷元素。
* **頁面捲軸抑制** — 消除變體 A 上出現的頁面層級捲軸，將翻譯介面限制在可視區域內。
* **建議卡片隱藏** — 自動隱藏建議卡片（「讓它更像商業用語」、「像對五歲小孩解釋一樣」、「讓它聽起來更自然」等）。處理 `button`、`a`、`[role="button"]` 及 `div` 類型卡片。多次延遲重試與 MutationObserver 可吸收不同電腦與瀏覽器環境間的 DOM 差異。
* **DOM 診斷 / HTML+CSS 診斷工具** — 用於檢查 ChatGPT 的 DOM 結構與 CSS 狀態的除錯工具。於設定中切換開/關。
* **導覽列** — 不在首頁時顯示重新整理與首頁按鈕。
* **Ctrl+C+C 整合** — 將任何應用程式中選取的文字傳送至 ChatGPT 翻譯分頁。

## 系統需求

- Windows 10 / Windows 11
- 您欲使用的每個 AI 供應商的 API 金鑰

## 安裝

從 [Releases](https://github.com/soheidon/LLM-Translator/releases) 下載最新的 `LLM-Translator-Desktop-Setup.exe` 並執行。安裝程式啟動時提供 11 種語言供選擇（您的選擇會記住以供日後安裝使用）。

## 使用方式

1. 啟動應用程式 — 翻譯視窗會顯示出來（在設定中啟用「最小化啟動」可改為啟動時收納至系統匣）
2. 在「設定 → API」中設定您的 API 金鑰
3. 在任何應用程式中選取文字，按兩次 Ctrl+C 即可翻譯
4. 或者，使用 Ctrl+Shift+C
5. 您也可以在主畫面手動輸入或貼上文字

## 開發

```bash
# 安裝依賴
npm install

# 以開發模式啟動
npm run tauri dev

# 建置
npm run tauri build
```

## 技術棧

- [Tauri v2](https://tauri.app/) — 桌面應用程式框架
- [Rust](https://www.rust-lang.org/) — 後端（API 通訊、鍵盤勾子、設定管理）
- [React](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) — 前端 UI
- [Vite](https://vitejs.dev/) — 建置工具

## 授權條款

[MIT](../../LICENSE)
