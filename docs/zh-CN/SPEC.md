[English](../../SPEC.md) | [日本語](../ja/SPEC.md) | [中文(简体)](SPEC.md) | [中文(繁體)](../zh-TW/SPEC.md) | [한국어](../ko/SPEC.md) | [Français](../fr/SPEC.md) | [Deutsch](../de/SPEC.md) | [Español](../es/SPEC.md)

# LLM Translator Desktop — 规格说明

## 1. 概述

### 1.1 名称
LLM Translator Desktop

### 1.2 目的
一款常驻桌面应用，类似于 DeepL Desktop，允许用户在任意应用中选中文本，按 Ctrl+C+C 将剪贴板文本发送到 LLM API，并在翻译窗口中显示翻译结果。

### 1.3 支持的操作系统
- Windows 10 / Windows 11
- macOS / Linux — 计划未来支持

---

## 2. 基本用户流程

```
在任意应用中选中文本
↓
按两次 Ctrl+C（或 Ctrl+Shift+C）
↓
应用读取剪贴板文本
↓
发送到翻译 API
↓
显示翻译窗口
↓
阅读 / 复制 / 重新翻译
```

---

## 3. 技术栈

| 层级 | 技术 |
|---------|------|
| 桌面框架 | Tauri v2 |
| 后端 | Rust |
| 前端 | React + TypeScript |
| 构建工具 | Vite |
| 主要 Tauri 插件 | global-shortcut, clipboard-manager, shell, store, log |
| HTTP | reqwest（Rust 端） |
| Windows API | SetWindowsHookEx（键盘钩子） |

---

## 4. 核心功能

### 4.1 Ctrl+C+C 翻译

使用 Windows 低级键盘钩子（`SetWindowsHookEx(WH_KEYBOARD_LL)`）在系统范围内检测 Ctrl+C 的双击。一旦检测到，从剪贴板读取文本并开始翻译。

- 检测：仅在同一 Ctrl 按住会话中第二次按下 C 时触发
- Ctrl 松开将重置会话（防止误触发）
- 排除 Shift/Alt/Win 修饰键（Ctrl+Shift+C 由 global_shortcut 处理）
- 过滤 C 键重复事件
- 阈值：默认 400ms（可在设置中配置）
- 可在设置中开启/关闭（需要重启应用）

### 4.2 剪贴板监控

在 v0.3.0 中已弃用。翻译仅由 Ctrl+C+C 和全局快捷键触发。

### 4.3 主翻译界面

- 源文本和翻译结果显示在两个并排窗格中
- 源文本窗格：直接输入、字符计数、清除
- 翻译窗格：重新翻译、复制
- 源语言选择（包括自动检测）和目标语言选择
- 语言交换
- 模型、语气和预设选择

### 4.4 设置界面

#### 通用设置
- 界面语言（11 种语言）
- 启动时最小化（默认关闭：首次启动时显示窗口）
- 始终置顶
- 聚焦翻译输入框
- 按 Esc 关闭
- 点击外部关闭
- Ctrl+C+C 快速翻译 开启/关闭
- 备用快捷键配置
- 打开翻译窗口快捷键
- 打开历史记录快捷键
- 翻译历史保存 开启/关闭
- 通知音效

#### API 设置
- 12 个服务商，以表格形式显示（服务商、状态）
- 展开的服务商详情：环境变量名称、API 密钥、Base URL、连接测试
- 基于角色的模型映射（默认、快速）
- 各模型行为模式（思考、普通）
- Ollama：本地模型列表获取和选择
- Google 翻译：Cloud API / Apps Script 支持
- DeepL：免费版 / 专业版支持
- 环境变量通过 `setx` + 注册表回退读取

#### 预设设置
- 10 个内置预设（新闻/学术/技术/邮件/字幕/自然/直译/正式/随意/友好）
- 搜索、名称/描述/系统提示编辑

#### 历史记录设置
- 最大历史条目数设置

### 4.5 历史记录界面
- 源文本和翻译结果并排显示
- 搜索
- 重新翻译、复制
- 加载更多
- 全部删除

### 4.6 系统托盘
- 常驻图标，带右键菜单
- 右键菜单：仅"退出"
- 左键单击显示主窗口
- X 按钮最小化到托盘（应用不会退出）
- 右键托盘图标 → "退出"即可完全退出
- 单实例：第二次启动会将已有实例置于前台

---

## 5. 支持的翻译服务商

| 服务商 | API 类型 | 默认模型 |
|-----------|---------|-------------|
| Google Translate / Cloud | Google Cloud Translation API | google-translate-v2 |
| DeepL / Free | DeepL API | deepl |
| DeepL / Pro | DeepL API | deepl |
| OpenAI / ChatGPT | OpenAI-compatible | gpt-5.5 |
| Gemini / Google | OpenAI-compatible | gemini-3.1-pro |
| Claude / Anthropic | Anthropic-compatible | claude-opus-4-8 |
| MiMo / 小米 | OpenAI-compatible | mimo-v2.5-pro |
| DeepSeek / DeepSeek | OpenAI-compatible | deepseek-v4-pro |
| Kimi / Moonshot | OpenAI-compatible | kimi-k2.7-code |
| Qwen / 阿里巴巴 | OpenAI-compatible | qwen3.7-max |
| MiniMAX / MiniMAX | OpenAI-compatible | MiniMax-M2.7 |
| Ollama / 本地 | OpenAI-compatible（本地） | - |
| Google Translate / Apps Script | Google Apps Script | google-apps-script |

---

## 6. 支持的界面语言

日语、English、中文(简体)、中文(繁體)、한국어、Français、Deutsch、Español、Português、Русский、Italiano

---

## 7. 安全性

- API 密钥存储在操作系统环境变量中，而非配置文件中
- API 密钥绝不会发送到前端（在 Rust 端持有）
- 所有通信使用 HTTPS（仅本地 Ollama 允许 HTTP）
- 历史记录保存可由用户开关控制

---

## 8. 数据存储

```
%APPDATA%/LLMTranslator/
├── settings.json    # 设置文件
└── history.jsonl    # 翻译历史（JSON Lines）
```

---

## 9. 文件结构

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
│   │   ├── googleTranslateLanguages.ts
│   │   └── chatgptTranslateLanguages.ts
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

## 10. 更新日志

### v0.3.4

- [x] Windows 10/11 支持
- [x] Tauri v2 + React + TypeScript + Rust
- [x] 系统托盘（X 按钮最小化到托盘，右键"退出"完全退出）
- [x] Ctrl+C+C 全局键盘钩子（Ctrl 按住会话方式，防误触发）
- [x] Ctrl+Shift+C 全局快捷键
- [x] 13 个服务商（OpenAI/Claude/Gemini/DeepSeek/MiMo/Kimi/Qwen/MiniMAX/Ollama/DeepL/Google Translate）
- [x] 标签页式翻译界面（LLM / Google 翻译 / ChatGPT 翻译）
- [x] 标签页记忆（上次选择的标签页保存到 localStorage，下次启动时恢复）
- [x] Google 翻译标签页：Tauri WebView 嵌入，源/目标语言设置，浏览器导航栏
- [x] ChatGPT 翻译标签页：WebView 嵌入，从设置界面配置语言
- [x] ChatGPT 翻译导航栏（刷新/主页，不在首页时显示）
- [x] 主翻译界面（双窗格）
- [x] 翻译复制
- [x] 设置界面（通用/API/预设/历史记录/Google 翻译/ChatGPT 翻译）
- [x] 11 种界面语言
- [x] 10 个翻译预设
- [x] 3 种语气选项（自动/普通/礼貌）
- [x] API 连接测试和状态显示
- [x] 历史记录保存/搜索/重新翻译
- [x] 基于环境变量的 API 密钥管理
- [x] 始终置顶
- [x] 启动时最小化
- [x] 版本显示（状态栏）
- [x] SVG 图标迁移（所有侧边栏/浏览器导航栏图标）
- [x] 应用图标显示（标题栏、侧边栏）
- [x] 设置界面顶栏清理（图标/标题文字移除），× 按钮移除
- [x] 设置按钮 UI 改进（黑色胶囊形），关闭设置按钮统一
- [x] LLM 标签页标签日语本地化（ja.json 中为"LLM翻訳"）
- [x] ChatGPT 建议卡片隐藏（含 DOM 诊断工具，可在设置中开关）
- [x] Ctrl+C+C 防误触发（keyboard_hook 重写：Ctrl 会话 + 修饰键排除 + 按键重复排除）
- [x] 剪贴板轮询移除（单次 Ctrl+C 不再触发翻译）
- [x] 关闭 = 托盘最小化（window.hide()），单实例防止
- [x] 系统托盘菜单简化（仅"退出"）
- [x] ChatGPT 翻译设置界面（源/目标语言选择）
- [x] 调试日志（触发来源识别）
- [x] 默认设置优化（double_copy_enabled 默认 true，阈值 400ms）
- [x] 状态栏显示简短的默认服务商模型名称
- [x] API 设置表格添加默认列（每行带"设为默认"按钮）
- [x] Google 翻译首页检测改进（基于 hostname + pathname，翻译后导航栏隐藏）
- [x] 设置 UI 改进（标题栏始终显示，标签栏/状态栏隐藏，← 图标移除）
- [x] Windows 开机自启（注册表 HKCU Run 键，开关控制，默认关闭，带引号路径）
- [x] 启动时最小化实现（`start_minimized` 设置强制执行，setup() 中 window.hide()）

### v0.3.5

- [x] ChatGPT 建议卡片隐藏稳定化（延迟计划延长至 8 秒，`.prompt-card` CSS 始终隐藏）
- [x] 建议卡片清理调试日志（仅输出新隐藏的卡片数量，data 属性防止重复计数）
- [x] `--auto-start` 标志以区分自动启动和手动启动
- [x] 托盘图标双击恢复修复（`SetWindowPos` + `SetForegroundWindow` 置前，Click 回退，700ms 冷却时间）
- [x] 托盘图标事件日志添加（记录所有事件类型）

### v0.3.6

- [x] ChatGPT 建议卡片隐藏扩展到 div 类型卡片（除了 `button`/`a`/`[role="button"]` 外还扫描 `div`，附带 AND 条件父容器排除保护）
- [x] DOM 诊断命令：添加子元素导出（输出包含建议文本的元素的一级直接子元素）

### v0.3.7

- [x] NSIS 安装程序语言选择对话框添加（`displayLanguageSelector: true`，11 种语言，选择保存到注册表）
- [x] `start_minimized` 默认值改为 `false`（修复首次启动时窗口不出现的问题）
- [x] 设置中"启动时最小化"标签和描述改进（正常启动时最小化到托盘）

### v0.3.8

- [x] ChatGPT 翻译标签页：LP 元素隐藏开关（头部导航、营销区块）
- [x] ChatGPT 翻译标签页：DOM 诊断工具（状态栏按钮列出可交互元素候选，设置中开关）
- [x] ChatGPT 翻译标签页：HTML+CSS 诊断工具（检查页头/页脚/LP 元素的 CSS 状态，设置中开关）
- [x] 诊断日志范围收窄（DOM：排除巨型页面包装器，HTML+CSS：排除翻译表单主体）
- [x] 系统托盘恢复可靠性改进（托盘图标生命周期延长到应用退出，hide 后保留 WebviewWindow 以确保双击恢复可靠）

### v0.4.0

- [x] ChatGPT 翻译标签页：多变体 DOM 支持——翻译表单在营销 LP（变体 A）和 应用/登录（变体 B）上均正确显示
- [x] CSS 选择器从 `main#main` 改为基于 `[data-llm-chatgpt-container="true"]` 属性，修复变体 A 上（无 `id="main"`）的全高显示问题
- [x] 登录按钮保留——`#contentful-header` 重新塑形为 44px 细条而非完全隐藏，保留登录按钮同时隐藏营销元素
- [x] 增强 CTA 隐藏——变体 A 上的"ChatGPT で翻訳を開始する"、"Try now"以及类似 CTA 被检测并隐藏
- [x] 页面滚动抑制——`html, body { overflow: hidden }` 消除页面级滚动条
- [x] 滚动位置重置——布局标记后重置滚动位置，防止翻译表单偏移出屏幕
- [x] LP 区块隐藏变体支持——解决 `#contentful-header` 与 `[class*="h-mkt-header-height"]` 之间的冲突，安全地仅隐藏 LP 元素

### v0.4.3

- [x] ChatGPT 翻译：47 种语言支持（含中文 3 路拆分、葡萄牙语 2 路拆分）
- [x] ChatGPT 翻译：双语别名匹配（日语 + 英语语言名称）
- [x] ChatGPT 翻译：启动时从已保存设置自动应用语言
- [x] ChatGPT 翻译：设置界面语言选择器扩展至 47 种语言
- [x] ChatGPT 翻译：控制台日志和语言调试日志（sessionStorage 环形缓冲区，状态栏复制）
- [x] 剪贴板：从 `navigator.clipboard` 迁移至 Tauri v2 `clipboard-manager` 插件
- [x] DOM/HTML+CSS 诊断按钮改为切换行为
- [x] 移除不必要的清空命令
- [x] 修复 9 个语言 JSON 文件中的双逗号问题

### v0.5
- macOS 支持（计划中）
- 自动更新
- 导出/导入设置
- OCR 翻译
- 术语表
- Markdown 保留
- 多引擎对比翻译

---
