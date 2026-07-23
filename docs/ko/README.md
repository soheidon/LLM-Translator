[English](../../README.md) | [日本語](../ja/README.md) | [中文(简体)](../zh-CN/README.md) | [中文(繁體)](../zh-TW/README.md) | [한국어](README.md) | [Français](../fr/README.md) | [Deutsch](../de/README.md) | [Español](../es/README.md)

# LLM Translator Desktop

Windows 데스크톱 번역 앱 — DeepL Desktop처럼 어떤 앱에서든 텍스트를 선택하고 Ctrl+C+C를 누르면 즉시 번역됩니다. LLM 번역, Google Translate 탭, ChatGPT Translate 탭을 포함하여 필요에 따라 번역 방식을 전환할 수 있습니다.

## 기능

### 공통

* **Ctrl+C+C 번역** — 어떤 앱에서든 텍스트를 선택하고 Ctrl+C를 두 번 누르면 현재 탭으로 전송되어 번역됩니다.
* **전역 단축키** — Ctrl+Shift+C (또는 사용자 지정 단축키)를 사용하여 명시적으로 번역을 실행할 수 있습니다.
* **시스템 트레이** — 창을 닫으면 종료되지 않고 시스템 트레이로 최소화됩니다. 트레이 아이콘을 더블 클릭하면 창이 복원되고, 우클릭 → "Exit"로 완전히 종료할 수 있습니다. 자동 시작 후에도 창 복원이 안정적으로 작동합니다.
* **단일 인스턴스** — 기존 인스턴스를 재사용하여 중복 트레이 아이콘을 방지합니다.
* **Windows 시작 시 자동 실행** — Windows 로그인 시 LLM Translator를 자동으로 실행할 수 있습니다 (기본값: OFF).
* **번역 기록** — 번역 기록을 저장, 검색 및 재번역할 수 있습니다.
* **탭 기억** — 마지막으로 선택한 탭을 기억하여 다음 실행 시 복원합니다.
* **다국어 UI** — 일본어, 영어, 중국어(간체/번체), 한국어, 프랑스어, 독일어, 스페인어, 포르투갈어, 러시아어, 이탈리아어를 지원합니다.

### LLM 번역

* **다양한 LLM 제공자** — OpenAI, Claude, Gemini, DeepSeek, MiMo, Kimi, Qwen, MiniMAX, Ollama 등.
* **번역 프리셋** — 뉴스, 학술, 기술, 이메일, 자막, 자연스러운, 직역, 격식, 캐주얼, 친근함 번역 스타일 중에서 선택할 수 있습니다.
* **톤 선택** — 일본어 출력에 대해 자동, 평범, 정중 톤을 선택할 수 있습니다.
* **API 키 보안** — API 키는 앱 설정 파일이 아닌 OS 환경 변수에 저장됩니다.
* **모델 표시** — 상태 표시줄에서 현재 제공자와 모델을 확인할 수 있습니다.

### Google Translate 탭

* **내장 Google Translate** — 브라우저로 전환하지 않고 앱 내에서 Google Translate를 사용할 수 있습니다.
* **언어 설정** — 설정 화면에서 소스 및 대상 언어를 설정할 수 있습니다.
* **스마트 내비게이션 바** — 내비게이션 버튼(뒤로, 앞으로, 새로고침, 홈)은 필요할 때만(로그인 페이지, 외부 페이지 등) 표시되며, 일반 번역 페이지에서는 숨겨집니다.
* **Ctrl+C+C 연동** — 어떤 앱에서든 선택한 텍스트를 Google Translate 탭으로 보낼 수 있습니다.

### ChatGPT Translate 탭

* **내장 ChatGPT** — 앱 내에서 ChatGPT 웹 인터페이스를 열고 번역 프롬프트를 보낼 수 있습니다.
* **47개 언어 지원** — 중국어(간체/번체/홍콩)와 포르투갈어(브라질/포르투갈)를 포함한 47개 언어 중 선택 가능. 언어 이름은 일본어와 영어 모두에서 매칭됩니다.
* **언어 자동 적용** — 저장된 소스 및 대상 언어가 실행 시 ChatGPT 페이지에 자동으로 적용됩니다.
* **언어 / 콘솔 디버그 로그** — 언어 선택 및 콘솔 출력을 위한 진단 로그 내장. 상태 표시줄에서 복사 가능. 설정에서 ON/OFF 전환.
* **LP 요소 숨기기** — 설정에서 ON/OFF 전환. ChatGPT 페이지에서 마케팅 내비게이션과 섹션을 숨겨 번역 폼에 집중할 수 있습니다.
* **다중 변형 DOM 지원** — 다양한 ChatGPT 페이지 구조(마케팅 LP vs. 앱/로그인 변형)에 자동으로 적응합니다. `main#main` 대신 `data-llm-chatgpt-container` 속성 기반 선택자를 사용하여 마케팅 요소만 숨기고 로그인 버튼은 보존합니다.
* **페이지 스크롤 억제** — 변형 A에서 발생하던 페이지 수준 스크롤바를 제거하여 번역 UI를 뷰포트 내에 유지합니다.
* **제안 카드 숨기기** — 제안 카드("더 비즈니스답게", "5살 아이에게 설명하듯", "더 자연스럽게" 등)를 자동으로 숨깁니다. `button`, `a`, `[role="button"]`, `div` 유형의 카드를 처리합니다. 여러 차례의 지연 재시도와 MutationObserver가 PC 및 브라우저 환경 간의 DOM 차이를 흡수합니다.
* **DOM 진단 / HTML+CSS 진단 도구** — ChatGPT의 DOM 구조와 CSS 상태를 검사하기 위한 디버그 도구. 설정에서 ON/OFF 전환.
* **내비게이션 바** — 최상위 페이지가 아닐 때 새로고침 및 홈 버튼이 표시됩니다.
* **Ctrl+C+C 연동** — 어떤 앱에서든 선택한 텍스트를 ChatGPT Translate 탭으로 보낼 수 있습니다.

## 요구 사항

- Windows 10 / Windows 11
- 사용하려는 각 AI 제공자의 API 키

## 설치

[Releases](https://github.com/soheidon/LLM-Translator/releases)에서 최신 `LLM-Translator-Desktop-Setup.exe`를 다운로드하여 실행하세요. 설치 프로그램은 실행 시 11개 언어를 선택할 수 있으며 (선택한 언어는 이후 설치 시 기억됩니다).

## 사용법

1. 앱을 실행하면 번역 창이 나타납니다 (설정에서 "최소화된 상태로 시작"을 활성화하면 대신 트레이에서 시작됩니다)
2. 설정 → API에서 API 키를 구성하세요
3. 어떤 앱에서든 텍스트를 선택하고 Ctrl+C를 두 번 눌러 번역하세요
4. 또는 Ctrl+Shift+C를 사용할 수도 있습니다
5. 메인 화면에서 직접 텍스트를 입력하거나 붙여넣을 수도 있습니다

## 개발

```bash
# 의존성 설치
npm install

# 개발 모드로 시작
npm run tauri dev

# 빌드
npm run tauri build
```

## 기술 스택

- [Tauri v2](https://tauri.app/) — 데스크톱 애플리케이션 프레임워크
- [Rust](https://www.rust-lang.org/) — 백엔드 (API 통신, 키보드 후크, 설정 관리)
- [React](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) — 프론트엔드 UI
- [Vite](https://vitejs.dev/) — 빌드 도구

## 라이선스

[MIT](../../LICENSE)

---
