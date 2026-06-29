[English](../../README.md) | [日本語](../ja/README.md) | [中文(简体)](README.md) | [中文(繁體)](../zh-TW/README.md) | [한국어](../ko/README.md) | [Français](../fr/README.md) | [Deutsch](../de/README.md) | [Español](../es/README.md)

# LLM Translator Desktop

一款 Windows 桌面快速翻译应用——类似于 DeepL Desktop，在任意应用中选中文本并按 Ctrl+C+C 即可即时翻译。包含 LLM 翻译、Google 翻译标签页和 ChatGPT 翻译标签页，让您根据需要切换翻译方式。

## 功能特点

### 通用功能

* **Ctrl+C+C 翻译** — 在任意应用中选中文本，按两次 Ctrl+C 即可将文本发送到当前标签页进行翻译。
* **全局快捷键** — 使用 Ctrl+Shift+C（或自定义快捷键）显式触发翻译。
* **系统托盘** — 关闭窗口会最小化到系统托盘而非退出。双击托盘图标恢复窗口，右键 → "退出"即可完全退出。窗口恢复功能在自动启动后也能可靠工作。
* **单实例** — 重复启动会复用已有实例，防止出现重复的托盘图标。
* **开机自启** — 可选择在登录 Windows 时自动启动 LLM Translator（默认：关闭）。
* **历史记录** — 保存、搜索和重新翻译您的翻译历史。
* **标签页记忆** — 记住上次选择的标签页，下次启动时自动恢复。
* **多语言界面** — 支持日语、英语、中文（简体/繁体）、韩语、法语、德语、西班牙语、葡萄牙语、俄语和意大利语。

### LLM 翻译

* **多个 LLM 服务商** — OpenAI、Claude、Gemini、DeepSeek、MiMo、Kimi、Qwen、MiniMAX、Ollama 等。
* **翻译预设** — 可选择新闻、学术、技术、邮件、字幕、自然、直译、正式、随意和友好等翻译风格。
* **语气选择** — 日语输出可选择自动、普通或礼貌语气。
* **API 密钥安全** — API 密钥存储在操作系统环境变量中，而非应用设置文件中。
* **模型显示** — 在状态栏中查看当前使用的服务商和模型。

### Google 翻译标签页

* **内嵌 Google 翻译** — 在应用内使用 Google 翻译，无需切换浏览器。
* **语言配置** — 在设置界面中设置源语言和目标语言。
* **智能导航栏** — 导航按钮（后退、前进、刷新、主页）仅在需要时显示（如登录页面、外部页面），在普通翻译页面中隐藏。
* **Ctrl+C+C 集成** — 将任意应用中选中的文本发送到 Google 翻译标签页。

### ChatGPT 翻译标签页

* **内嵌 ChatGPT** — 在应用内打开 ChatGPT 网页界面并发送翻译提示。
* **语言配置** — 在设置界面中设置源语言和目标语言。
* **LP 元素隐藏** — 可在设置中开关。隐藏 ChatGPT 页面中的营销导航和区块，让您专注于翻译表单。
* **多变体 DOM 支持** — 自动适配不同的 ChatGPT 页面结构（营销 LP 与 应用/登录 变体）。使用基于 `data-llm-chatgpt-container` 属性的选择器代替 `main#main`，在保留登录按钮的同时仅隐藏营销元素。
* **页面滚动抑制** — 消除变体 A 中出现的页面级滚动条，使翻译 UI 保持在视口范围内。
* **建议卡片隐藏** — 自动隐藏建议卡片（"让文字更商务化"、"像给 5 岁孩子解释一样"、"让文字听起来更自然"等）。处理 `button`、`a`、`[role="button"]` 和 `div` 类型的卡片。多次延迟重试和 MutationObserver 可吸收不同 PC 和浏览器环境下的 DOM 差异。
* **DOM 诊断 / HTML+CSS 诊断工具** — 用于检查 ChatGPT 的 DOM 结构和 CSS 状态的调试工具。可在设置中开关。
* **导航栏** — 不在首页时显示刷新和主页按钮。
* **Ctrl+C+C 集成** — 将任意应用中选中的文本发送到 ChatGPT 翻译标签页。

## 系统要求

- Windows 10 / Windows 11
- 需要使用的各 AI 服务商的 API 密钥

## 安装

从 [Releases](https://github.com/soheidon/LLM-Translator/releases) 下载最新的 `LLM-Translator-Desktop-Setup.exe` 并运行。安装程序启动时提供 11 种语言供选择（您的选择会被记住，供后续安装使用）。

## 使用方法

1. 启动应用——翻译窗口将会出现（如需启动时最小化到托盘，请在设置中开启"启动时最小化"）
2. 在 设置 → API 中配置您的 API 密钥
3. 在任意应用中选中文本，按两次 Ctrl+C 即可翻译
4. 也可以使用 Ctrl+Shift+C
5. 您还可以在主界面中手动输入或粘贴文本

## 开发

```bash
# 安装依赖
npm install

# 以开发模式启动
npm run tauri dev

# 构建
npm run tauri build
```

## 技术栈

- [Tauri v2](https://tauri.app/) — 桌面应用框架
- [Rust](https://www.rust-lang.org/) — 后端（API 通信、键盘钩子、配置管理）
- [React](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) — 前端 UI
- [Vite](https://vitejs.dev/) — 构建工具

## 版本

v0.4.0

## 许可证

MIT

---
