[English](SPEC.md) | [日本語](docs/ja/SPEC.md) | [中文(简体)](docs/zh-CN/SPEC.md) | [中文(繁體)](docs/zh-TW/SPEC.md) | [한국어](docs/ko/SPEC.md) | [Français](docs/fr/SPEC.md) | [Deutsch](docs/de/SPEC.md) | [Español](docs/es/SPEC.md)

# LLM Translator Desktop — 規格說明書

## 1. 概述

### 1.1 名稱
LLM Translator Desktop

### 1.2 目的
一款常駐桌面應用程式，類似 DeepL Desktop，讓使用者在任何應用程式中選取文字並按下 Ctrl+C+C，將剪貼簿文字傳送至 LLM API 並在翻譯視窗中顯示翻譯結果。

### 1.3 支援的作業系統
- Windows 10 / Windows 11
- macOS / Linux — 計劃於未來版本支援

---

## 2. 基本使用者流程

```
在任何應用程式中選取文字
↓
按兩次 Ctrl+C（或 Ctrl+Shift+C）
↓
應用程式讀取剪貼簿文字
↓
傳送至翻譯 API
↓
顯示翻譯視窗
↓
閱讀 / 複製 / 重新翻譯
```

---

## 3. 技術棧

| 層級 | 技術 |
|---------|------|
| 桌面框架 | Tauri v2 |
| 後端 | Rust |
| 前端 | React + TypeScript |
| 建置工具 | Vite |
| 主要 Tauri 外掛 | global-shortcut、clipboard-manager、shell、store、log |
| HTTP | reqwest（Rust 端） |
| Windows API | SetWindowsHookEx（鍵盤勾子） |

---

## 4. 核心功能

### 4.1 Ctrl+C+C 翻譯

使用 Windows 低階鍵盤勾子（`SetWindowsHookEx(WH_KEYBOARD_LL)`）在全系統範圍內偵測 Ctrl+C 的雙擊。一旦偵測到，即從剪貼簿讀取文字並開始翻譯。

- 偵測：僅在相同 Ctrl 按住期間內第二次按下 C 時觸發
- Ctrl 放開時重置按鍵階段（防止誤觸發）
- 排除 Shift/Alt/Win 修飾鍵（Ctrl+Shift+C 由 global_shortcut 處理）
- 過濾 C 鍵重複事件
- 閾值：預設 400ms（可在設定中調整）
- 可在設定中切換開/關（需重新啟動應用程式）

### 4.2 剪貼簿監控

於 v0.3.0 已淘汰。翻譯僅由 Ctrl+C+C 與全域快速鍵觸發。

### 4.3 主翻譯畫面

- 原文與翻譯結果並排顯示於兩個窗格
- 原文窗格：直接輸入、字元計數、清除
- 翻譯窗格：重新翻譯、複製
- 來源語言選擇（包含自動偵測）與目標語言選擇
- 語言交換
- 模型、語氣與預設風格選擇

### 4.4 設定畫面

#### 一般設定
- 介面語言（11 種語言）
- 最小化啟動（預設關閉：首次啟動時顯示視窗）
- 最上層顯示
- 翻譯時聚焦
- 按 Esc 關閉
- 點擊外部關閉
- Ctrl+C+C 快速翻譯 開/關
- 備用快速鍵設定
- 開啟翻譯視窗快速鍵
- 開啟歷史記錄快速鍵
- 翻譯歷史儲存 開/關
- 通知音效

#### API 設定
- 12 個供應商以表格格式顯示（供應商、狀態）
- 展開的供應商詳細資訊：環境變數名稱、API 金鑰、基礎 URL、連線測試
- 角色型模型對應（default、fast）
- 各模型行為模式（thinking、normal）
- Ollama：本機模型清單擷取與選擇
- Google 翻譯：雲端 API / Apps Script 支援
- DeepL：免費版 / 專業版支援
- 環境變數透過 `setx` + 登錄檔備援讀取

#### 預設風格設定
- 10 個內建預設風格（新聞/學術/技術/電子郵件/字幕/自然/直譯/正式/隨意/友善）
- 搜尋、名稱/描述/系統提示編輯

#### 歷史記錄設定
- 歷史記錄數量上限設定

### 4.5 歷史記錄畫面
- 原文與翻譯並排顯示
- 搜尋
- 重新翻譯、複製
- 載入更多
- 全部刪除

### 4.6 系統匣
- 常駐圖示，附右鍵選單
- 右鍵選單：僅「Exit」
- 左鍵點擊顯示主視窗
- X 按鈕最小化至系統匣（應用程式不結束）
- 右鍵點擊系統匣圖示 →「Exit」完全結束
- 單一執行個體：第二次啟動時將現有執行個體帶至前景

---

## 5. 支援的翻譯供應商

| 供應商 | API 類型 | 預設模型 |
|-----------|---------|-------------|
| Google Translate / Cloud | Google Cloud Translation API | google-translate-v2 |
| DeepL / Free | DeepL API | deepl |
| DeepL / Pro | DeepL API | deepl |
| OpenAI / ChatGPT | OpenAI-compatible | gpt-5.5 |
| Gemini / Google | OpenAI-compatible | gemini-3.1-pro |
| Claude / Anthropic | Anthropic-compatible | claude-opus-4-8 |
| MiMo / Xiaomi | OpenAI-compatible | mimo-v2.5-pro |
| DeepSeek / DeepSeek | OpenAI-compatible | deepseek-v4-pro |
| Kimi / Moonshot | OpenAI-compatible | kimi-k2.7-code |
| Qwen / Alibaba | OpenAI-compatible | qwen3.7-max |
| MiniMAX / MiniMAX | OpenAI-compatible | MiniMax-M2.7 |
| Ollama / Local | OpenAI-compatible (local) | - |
| Google Translate / Apps Script | Google Apps Script | google-apps-script |

---

## 6. 支援的介面語言

日本語、English、中文(简体)、中文(繁體)、한국어、Français、Deutsch、Español、Português、Русский、Italiano

---

## 7. 安全性

- API 金鑰儲存在作業系統環境變數中，而非設定檔內
- API 金鑰絕不傳送至前端（保留在 Rust 端）
- 所有通訊使用 HTTPS（僅本機 Ollama 允許 HTTP）
- 歷史記錄儲存可由使用者切換開/關

---

## 8. 資料儲存

```
%APPDATA%/LLMTranslator/
├── settings.json    # 設定檔
└── history.jsonl    # 翻譯歷史記錄（JSON Lines）
```

---

## 9. 檔案結構

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
│   │   ├── en.json、ja.json、zh-CN.json、zh-TW.json、
│   │   │   ko.json、fr.json、de.json、es.json、
│   │   │   pt.json、ru.json、it.json
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

## 10. 變更記錄

### v0.3.4

- [x] Windows 10/11 支援
- [x] Tauri v2 + React + TypeScript + Rust
- [x] 系統匣（X 最小化至系統匣，右鍵「Exit」完全結束）
- [x] Ctrl+C+C 全域鍵盤勾子（Ctrl 按住階段方式，防止誤觸發）
- [x] Ctrl+Shift+C 全域快速鍵
- [x] 13 個供應商（OpenAI/Claude/Gemini/DeepSeek/MiMo/Kimi/Qwen/MiniMAX/Ollama/DeepL/Google Translate）
- [x] 分頁式翻譯畫面（LLM / Google 翻譯 / ChatGPT 翻譯）
- [x] 分頁記憶（最後選取的分頁儲存至 localStorage，下次啟動時還原）
- [x] Google 翻譯分頁：Tauri WebView 內嵌、來源/目標語言設定、瀏覽器導覽列
- [x] ChatGPT 翻譯分頁：WebView 內嵌、從設定畫面進行語言設定
- [x] ChatGPT 翻譯導覽列（重新整理/首頁，在非首頁時顯示）
- [x] 主翻譯畫面（雙窗格）
- [x] 翻譯複製
- [x] 設定畫面（一般/API/預設風格/歷史記錄/Google 翻譯/ChatGPT 翻譯）
- [x] 11 種語言介面
- [x] 10 種翻譯預設風格
- [x] 3 種語氣選項（自動/普通/禮貌）
- [x] API 連線測試與狀態顯示
- [x] 歷史記錄儲存/搜尋/重新翻譯
- [x] 基於環境變數的 API 金鑰管理
- [x] 最上層顯示
- [x] 最小化啟動
- [x] 版本顯示（狀態列）
- [x] SVG 圖示遷移（所有側邊欄/瀏覽器導覽列圖示）
- [x] 應用程式圖示顯示（標題列、側邊欄）
- [x] 設定頂欄清理（移除圖示/標題文字）、移除 × 按鈕
- [x] 設定按鈕 UI 改進（黑色膠囊形狀）、關閉設定按鈕統一
- [x] LLM 分頁標籤日文在地化（ja.json 中的「LLM翻訳」）
- [x] ChatGPT 建議卡片隱藏（附 DOM 診斷工具，於設定中切換開/關）
- [x] Ctrl+C+C 誤觸發防止（keyboard_hook 重寫：Ctrl 階段 + 修飾鍵排除 + 按鍵重複排除）
- [x] 移除剪貼簿輪詢（單次 Ctrl+C 不再觸發翻譯）
- [x] 關閉 = 系統匣最小化（window.hide()）、單一執行個體防止
- [x] 系統匣選單簡化（僅「Exit」）
- [x] ChatGPT 翻譯設定畫面（來源/目標語言選擇）
- [x] 除錯記錄（觸發來源識別）
- [x] 預設設定最佳化（double_copy_enabled 預設 true、閾值 400ms）
- [x] 狀態列顯示簡短預設供應商模型名稱
- [x] API 設定表格新增預設欄位（每列附「設為預設」按鈕）
- [x] Google 翻譯首頁偵測改進（基於 hostname + pathname，翻譯後隱藏導覽列）
- [x] 設定 UI 改進（標題列始終顯示、分頁列/狀態列隱藏、移除 ← 圖示）
- [x] Windows 自動啟動（登錄檔 HKCU Run 機碼、切換開/關、預設關閉、帶引號路徑）
- [x] 最小化啟動實作（強制執行 `start_minimized` 設定、setup() 中呼叫 window.hide()）

### v0.3.5

- [x] ChatGPT 建議卡片隱藏穩定化（延遲排程擴展至 8 秒、`.prompt-card` CSS 始終隱藏）
- [x] 建議卡片清除除錯記錄（僅輸出新隱藏的卡片數量、data 屬性防止重複計算）
- [x] `--auto-start` 旗標以區分自動啟動與手動啟動
- [x] 系統匣圖示按兩下還原修正（`SetWindowPos` + `SetForegroundWindow` 帶至前景、Click 備援、700ms 冷卻時間）
- [x] 新增系統匣圖示事件記錄（所有事件類型均記錄）

### v0.3.6

- [x] ChatGPT 建議卡片隱藏擴展至 div 類型卡片（除 `button`/`a`/`[role="button"]` 外，亦掃描 `div`，並以 AND 條件父容器排除防護）
- [x] DOM 診斷指令：新增子元素傾印（輸出一層直接子元素，針對包含建議文字的元素）

### v0.3.7

- [x] 新增 NSIS 安裝程式語言選擇對話框（`displayLanguageSelector: true`、11 種語言、選擇儲存至登錄檔）
- [x] `start_minimized` 預設值改為 `false`（修正首次啟動時視窗未顯示的問題）
- [x] 設定中的「最小化啟動」標籤與說明改進（一般啟動時最小化至系統匣）

### v0.3.8

- [x] ChatGPT 翻譯分頁：LP 元素隱藏切換（標題導覽、行銷區塊）
- [x] ChatGPT 翻譯分頁：DOM 診斷工具（狀態列按鈕列出互動元素候選、於設定中切換開/關）
- [x] ChatGPT 翻譯分頁：HTML+CSS 診斷工具（檢查標頭/頁尾/LP 元素的 CSS 狀態、於設定中切換開/關）
- [x] 診斷記錄範圍縮小（DOM：排除巨型頁面包裹器、HTML+CSS：排除翻譯表單主體）
- [x] 系統匣還原可靠性改進（系統匣圖示生命週期延長至應用程式結束、WebviewWindow 在隱藏後保留以確保按兩下還原）

### v0.4.0

- [x] ChatGPT 翻譯分頁：多變體 DOM 支援 — 翻譯表單在行銷 LP（變體 A）與應用程式/登入（變體 B）上均正確顯示
- [x] CSS 選擇器從 `main#main` 改為 `[data-llm-chatgpt-container="true"]` 屬性型選擇器，修正變體 A 上的全高顯示問題（無 `id="main"`）
- [x] 登入按鈕保留 — `#contentful-header` 改為 44px 細條而非完全隱藏，保留登入按鈕同時隱藏行銷元素
- [x] 增強 CTA 隱藏 — 在變體 A 上偵測並隱藏「ChatGPT で翻訳を開始する」、「Try now」及類似的 CTA
- [x] 頁面捲軸抑制 — `html, body { overflow: hidden }` 消除頁面層級捲軸
- [x] 捲軸位置重置 — 版面標記後重置捲軸位置，防止翻譯表單移出畫面
- [x] LP 區塊隱藏變體支援 — 解決 `#contentful-header` 與 `[class*="h-mkt-header-height"]` 的衝突，安全地僅隱藏 LP 元素

### v0.5
- macOS 支援（計劃中）
- 自動更新
- 匯出/匯入設定
- OCR 翻譯
- 詞彙表
- Markdown 保留
- 多引擎對照翻譯
