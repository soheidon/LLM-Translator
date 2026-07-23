[English](README.md) | [日本語](docs/ja/README.md) | [中文(简体)](docs/zh-CN/README.md) | [中文(繁體)](docs/zh-TW/README.md) | [한국어](docs/ko/README.md) | [Français](docs/fr/README.md) | [Deutsch](docs/de/README.md) | [Español](docs/es/README.md)

# LLM Translator Desktop

A Windows desktop app for quick translation — just like DeepL Desktop, select text in any app and press Ctrl+C+C to translate instantly. Includes LLM translation, a Google Translate tab, and a ChatGPT Translate tab, letting you switch translation methods depending on your needs.

## Features

### Common

* **Ctrl+C+C Translation** — Select text in any app and press Ctrl+C twice to send it to the current tab for translation.
* **Global Shortcut** — Use Ctrl+Shift+C (or a custom shortcut) to explicitly trigger translation.
* **System Tray** — Closing the window minimizes to the system tray instead of quitting. Double-click the tray icon to restore the window, right-click → "Exit" to fully quit. Window restoration works reliably even after auto-start.
* **Single Instance** — Reuses the existing instance to prevent duplicate tray icons.
* **Auto-start with Windows** — Optionally launch LLM Translator when you log into Windows (default: OFF).
* **History** — Save, search, and re-translate your translation history.
* **Tab Memory** — Remembers the last selected tab and restores it on next launch.
* **Multilingual UI** — Supports Japanese, English, Chinese (Simplified/Traditional), Korean, French, German, Spanish, Portuguese, Russian, and Italian.

### LLM Translation

* **Multiple LLM Providers** — OpenAI, Claude, Gemini, DeepSeek, MiMo, Kimi, Qwen, MiniMAX, Ollama, and more.
* **Translation Presets** — Choose from News, Academic, Technical, Email, Subtitle, Natural, Literal, Formal, Casual, and Friendly translation styles.
* **Tone Selection** — Auto, Plain, or Polite tone for Japanese output.
* **API Key Security** — API keys are stored in OS environment variables, never in app settings files.
* **Model Display** — See the current provider and model in the status bar.
* **Quality / Speed Model Selection** — Switch between high-quality and fast models per provider from the status bar. The model ID and thinking/normal mode are resolved automatically from each provider's model mapping configuration.

### Google Translate Tab

* **Embedded Google Translate** — Use Google Translate inside the app without switching to a browser.
* **Language Configuration** — Set source and target languages from the settings screen.
* **Smart Navigation Bar** — Navigation buttons (back, forward, reload, home) appear only when needed (e.g., login pages, external pages), hidden on the normal translate page.
* **Ctrl+C+C Integration** — Send selected text from any app to the Google Translate tab.

### ChatGPT Translate Tab

* **Embedded ChatGPT** — Open ChatGPT's web interface inside the app and send translation prompts.
* **47-Language Support** — Choose from 47 languages including Chinese (Simplified/Traditional/Hong Kong) and Portuguese (Brazil/Portugal). Language names are matched in both Japanese and English.
* **Auto Language Apply** — Saved source and target languages are automatically applied to the ChatGPT page on launch.
* **Language / Console Debug Logs** — Built-in diagnostic logs for language selection and console output, copyable from the status bar. Toggle ON/OFF in settings.
* **LP Element Hiding** — Toggle ON/OFF in settings. Hide marketing navigation and sections from the ChatGPT page so you can focus on the translation form.
* **Multi-Variant DOM Support** — Automatically adapts to different ChatGPT page structures (marketing LP vs. app/login variants). Uses `data-llm-chatgpt-container` attribute-based selectors instead of `main#main`, preserving the login button while hiding only marketing elements.
* **Page Scroll Suppression** — Eliminates the page-level scrollbar that appeared on variant A, keeping the translation UI contained within the viewport.
* **Suggestion Card Hiding** — Automatically hides suggestion cards ("Make it more business-like", "Explain like I'm 5", "Make it sound more natural", etc.). Handles `button`, `a`, `[role="button"]`, and `div`-type cards. Multiple delayed retries and a MutationObserver absorb DOM differences across PCs and browser environments.
* **DOM Diagnostic / HTML+CSS Diagnostic Tools** — Debug tools for inspecting ChatGPT's DOM structure and CSS state. Toggle ON/OFF in settings.
* **Navigation Bar** — Reload and home buttons appear when not on the top page.
* **Ctrl+C+C Integration** — Send selected text from any app to the ChatGPT Translate tab.

## Requirements

- Windows 10 / Windows 11
- An API key for each AI provider you want to use

## Installation

Download the latest `LLM-Translator-Desktop-Setup.exe` from [Releases](https://github.com/soheidon/LLM-Translator/releases) and run it. The installer offers 11 languages to choose from on launch (your selection is remembered for future installs).

## Usage

1. Launch the app — the translation window appears (enable "Start minimized" in settings to start in the tray instead)
2. Configure your API keys in Settings → API
3. Select text in any app and press Ctrl+C twice to translate
4. Alternatively, use Ctrl+Shift+C
5. You can also type or paste text manually in the main screen

## Development

```bash
# Install dependencies
npm install

# Start in development mode
npm run tauri dev

# Build
npm run tauri build
```

## Tech Stack

- [Tauri v2](https://tauri.app/) — Desktop application framework
- [Rust](https://www.rust-lang.org/) — Backend (API communication, keyboard hook, config management)
- [React](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) — Frontend UI
- [Vite](https://vitejs.dev/) — Build tool

## License

[MIT](LICENSE)
