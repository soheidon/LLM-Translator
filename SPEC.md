[English](SPEC.md) | [日本語](docs/ja/SPEC.md) | [中文(简体)](docs/zh-CN/SPEC.md) | [한국어](docs/ko/SPEC.md) | [Français](docs/fr/SPEC.md) | [Deutsch](docs/de/SPEC.md) | [Español](docs/es/SPEC.md)

# LLM Translator Desktop — Specification

## 1. Overview

### 1.1 Name
LLM Translator Desktop

### 1.2 Purpose
A resident desktop app, similar to DeepL Desktop, that lets users select text in any application and press Ctrl+C+C to send the clipboard text to an LLM API and display the translation in a translation window.

### 1.3 Supported OS
- Windows 10 / Windows 11
- macOS / Linux — planned for future release

---

## 2. Basic User Flow

```
Select text in any app
↓
Press Ctrl+C twice (or Ctrl+Shift+C)
↓
App reads clipboard text
↓
Sends to translation API
↓
Displays translation window
↓
Read / Copy / Re-translate
```

---

## 3. Tech Stack

| Layer | Technology |
|---------|------|
| Desktop Framework | Tauri v2 |
| Backend | Rust |
| Frontend | React + TypeScript |
| Build Tool | Vite |
| Main Tauri Plugins | global-shortcut, clipboard-manager, shell, store, log |
| HTTP | reqwest (Rust side) |
| Windows API | SetWindowsHookEx (keyboard hook) |

---

## 4. Core Features

### 4.1 Ctrl+C+C Translation

Uses a Windows low-level keyboard hook (`SetWindowsHookEx(WH_KEYBOARD_LL)`) to detect double-press of Ctrl+C system-wide. Once detected, reads text from the clipboard and starts translation.

- Detection: Only fires on the second C press within the same Ctrl-held session
- Ctrl keyup resets the session (prevents false triggers)
- Shift/Alt/Win modifiers excluded (Ctrl+Shift+C is handled by global_shortcut)
- C key repeat is filtered out
- Threshold: default 400ms (configurable in settings)
- Can be toggled ON/OFF in settings (requires app restart)

### 4.2 Clipboard Monitoring

Deprecated in v0.3.0. Translation is triggered only by Ctrl+C+C and global shortcut.

### 4.3 Main Translation Screen

- Source and translation shown in two side-by-side panes
- Source pane: direct input, character count, clear
- Translation pane: re-translate, copy
- Source language selection (including auto-detect) and target language selection
- Language swap
- Model, tone, and preset selection

### 4.4 Settings Screen

#### General Settings
- UI language (11 languages)
- Start minimized (default OFF: window is shown on first launch)
- Always on top
- Focus on translate
- Close on Esc
- Close on outside click
- Ctrl+C+C quick translate ON/OFF
- Alternative shortcut configuration
- Open translate window shortcut
- Open history shortcut
- Translation history save ON/OFF
- Notification sound

#### API Settings
- 12 providers in table format (Provider, Status)
- Expanded provider details: environment variable name, API key, Base URL, connection test
- Role-based model mapping (default, fast)
- Per-model behavior mode (thinking, normal)
- Ollama: local model list fetch and selection
- Google Translate: Cloud API / Apps Script support
- DeepL: Free / Pro support
- Environment variables read via `setx` + registry fallback

#### Preset Settings
- 10 built-in presets (News/Academic/Technical/Email/Subtitle/Natural/Literal/Formal/Casual/Friendly)
- Search, name/description/system prompt editing

#### History Settings
- Max history entries setting

### 4.5 History Screen
- Source and translation shown side-by-side
- Search
- Re-translate, copy
- Load More
- Delete all

### 4.6 System Tray
- Resident icon with right-click menu
- Right-click menu: "Exit" only
- Left-click shows the main window
- X button minimizes to tray (app does not exit)
- Right-click tray icon → "Exit" to fully quit
- Single instance: second launch brings the existing instance to the front

---

## 5. Supported Translation Providers

| Provider | API Type | Default Model |
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

## 6. Supported UI Languages

Japanese, English, 中文(简体), 中文(繁體), 한국어, Français, Deutsch, Español, Português, Русский, Italiano

---

## 7. Security

- API keys stored in OS environment variables, not in config files
- API keys never sent to the frontend (held on the Rust side)
- HTTPS for all communication (HTTP allowed only for local Ollama)
- History saving is user-toggleable ON/OFF

---

## 8. Data Storage

```
%APPDATA%/LLMTranslator/
├── settings.json    # Settings file
└── history.jsonl    # Translation history (JSON Lines)
```

---

## 9. File Structure

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

## 10. Changelog

### v0.3.4

- [x] Windows 10/11 support
- [x] Tauri v2 + React + TypeScript + Rust
- [x] System tray (X minimizes to tray, right-click "Exit" to fully quit)
- [x] Ctrl+C+C global keyboard hook (Ctrl-held session method, false-trigger prevention)
- [x] Ctrl+Shift+C global shortcut
- [x] 13 providers (OpenAI/Claude/Gemini/DeepSeek/MiMo/Kimi/Qwen/MiniMAX/Ollama/DeepL/Google Translate)
- [x] Tabbed translation screen (LLM / Google Translate / ChatGPT Translate)
- [x] Tab memory (last tab saved to localStorage, restored on next launch)
- [x] Google Translate tab: Tauri WebView embedding, source/target language settings, browser nav bar
- [x] ChatGPT Translate tab: WebView embedding, language settings from settings screen
- [x] ChatGPT Translate nav bar (reload/home, shown outside top page)
- [x] Main translation screen (2-pane)
- [x] Translation copy
- [x] Settings screen (General/API/Presets/History/Google Translate/ChatGPT Translate)
- [x] 11-language UI
- [x] 10 translation presets
- [x] 3 tone options (Auto/Plain/Polite)
- [x] API connection test and status display
- [x] History save/search/re-translate
- [x] Environment variable-based API key management
- [x] Always on top
- [x] Start minimized
- [x] Version display (status bar)
- [x] SVG icon migration (all sidebar/browser nav bar icons)
- [x] App icon display (title bar, sidebar)
- [x] Settings top bar cleanup (icon/title text removed), × button removed
- [x] Settings button UI improvement (black capsule), close-settings button unified
- [x] LLM tab label localized for Japanese ("LLM翻訳" in ja.json)
- [x] ChatGPT suggestion card hiding (with DOM diagnostic tool, toggle ON/OFF in settings)
- [x] Ctrl+C+C false trigger prevention (keyboard_hook rewrite: Ctrl session + modifier exclusion + key repeat exclusion)
- [x] Clipboard polling removed (single Ctrl+C no longer triggers translation)
- [x] Close = tray minimize (window.hide()), single instance prevention
- [x] System tray menu simplified ("Exit" only)
- [x] ChatGPT Translate settings screen (source/target language selection)
- [x] Debug logging (trigger source identification)
- [x] Default settings optimization (double_copy_enabled default true, threshold 400ms)
- [x] Status bar shows short default provider model name
- [x] Default column added to API settings table (with Set as Default button per row)
- [x] Google Translate top-page detection improved (hostname + pathname based, nav bar hidden after translation)
- [x] Settings UI improvements (title bar always shown, tab bar/status bar hidden, ← icon removed)
- [x] Windows auto-start (Registry HKCU Run key, toggle ON/OFF, default OFF, quoted path)
- [x] Start minimized implementation (`start_minimized` setting enforced, window.hide() in setup())

### v0.3.5

- [x] ChatGPT suggestion card hiding stabilized (delay schedule extended to 8s, `.prompt-card` CSS always hidden)
- [x] Suggestion card cleanup debug logging (outputs only newly hidden card count, data attribute prevents double counting)
- [x] `--auto-start` flag to separate auto-start from manual launch
- [x] Tray icon double-click restoration fix (`SetWindowPos` + `SetForegroundWindow` bring-to-front, Click fallback, 700ms cooldown)
- [x] Tray icon event logging added (all event types logged)

### v0.3.6

- [x] ChatGPT suggestion card hiding extended to div-type cards (scans `div` in addition to `button`/`a`/`[role="button"]`, with AND-condition parent container exclusion guard)
- [x] DOM diagnostic command: child element dump added (outputs one level of direct children for elements containing suggestion text)

### v0.3.7

- [x] NSIS installer language selection dialog added (`displayLanguageSelector: true`, 11 languages, selection saved to registry)
- [x] `start_minimized` default changed to `false` (fixes window not appearing on first launch)
- [x] "Start minimized" label and description improved in settings (minimizes to tray on normal launch)

### v0.3.8

- [x] ChatGPT Translate tab: LP element hiding toggle (header nav, marketing sections)
- [x] ChatGPT Translate tab: DOM diagnostic tool (StatusBar button to list interactive element candidates, toggle ON/OFF in settings)
- [x] ChatGPT Translate tab: HTML+CSS diagnostic tool (inspect CSS state of headers/footers/LP elements, toggle ON/OFF in settings)
- [x] Diagnostic log scope narrowed (DOM: exclude giant page wrappers, HTML+CSS: exclude translation form body)
- [x] System tray restoration reliability improved (tray icon lifetime extended to app exit, WebviewWindow preserved after hide for guaranteed double-click restoration)

### v0.4.0

- [x] ChatGPT Translate tab: Multi-variant DOM support — translation form displays correctly on both marketing LP (variant A) and app/login (variant B)
- [x] CSS selectors changed from `main#main` to `[data-llm-chatgpt-container="true"]` attribute-based, fixing full-height display on variant A (no `id="main"`)
- [x] Login button preservation — `#contentful-header` reshaped to a 44px slim bar instead of being hidden entirely, keeping the login button while hiding marketing elements
- [x] Enhanced CTA hiding — "ChatGPT で翻訳を開始する", "Try now", and similar CTAs detected and hidden on variant A
- [x] Page scroll suppression — `html, body { overflow: hidden }` eliminates page-level scrollbar
- [x] Scroll position reset — scroll position reset after layout markers to prevent translate form from shifting off-screen
- [x] LP section hiding variant support — resolved conflict between `#contentful-header` and `[class*="h-mkt-header-height"]` to safely hide only LP elements

### v0.5
- macOS support (planned)
- Auto-update
- Export/import settings
- OCR translation
- Glossaries
- Markdown preservation
- Multi-engine comparison translation

