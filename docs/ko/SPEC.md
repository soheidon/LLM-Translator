[English](../../SPEC.md) | [日本語](../ja/SPEC.md) | [中文(简体)](../zh-CN/SPEC.md) | [中文(繁體)](../zh-TW/SPEC.md) | [한국어](SPEC.md) | [Français](../fr/SPEC.md) | [Deutsch](../de/SPEC.md) | [Español](../es/SPEC.md)

# LLM Translator Desktop — 사양

## 1. 개요

### 1.1 이름
LLM Translator Desktop

### 1.2 목적
DeepL Desktop과 유사한 상주형 데스크톱 앱으로, 사용자가 어떤 애플리케이션에서든 텍스트를 선택하고 Ctrl+C+C를 누르면 클립보드 텍스트를 LLM API로 전송하여 번역 결과를 번역 창에 표시합니다.

### 1.3 지원 OS
- Windows 10 / Windows 11
- macOS / Linux — 향후 출시 예정

---

## 2. 기본 사용자 흐름

```
어떤 앱에서든 텍스트 선택
↓
Ctrl+C 두 번 누르기 (또는 Ctrl+Shift+C)
↓
앱이 클립보드 텍스트 읽기
↓
번역 API로 전송
↓
번역 창 표시
↓
읽기 / 복사 / 재번역
```

---

## 3. 기술 스택

| 계층 | 기술 |
|---------|------|
| 데스크톱 프레임워크 | Tauri v2 |
| 백엔드 | Rust |
| 프론트엔드 | React + TypeScript |
| 빌드 도구 | Vite |
| 주요 Tauri 플러그인 | global-shortcut, clipboard-manager, shell, store, log |
| HTTP | reqwest (Rust 측) |
| Windows API | SetWindowsHookEx (키보드 후크) |

---

## 4. 핵심 기능

### 4.1 Ctrl+C+C 번역

Windows 저수준 키보드 후크(`SetWindowsHookEx(WH_KEYBOARD_LL)`)를 사용하여 시스템 전체에서 Ctrl+C 더블 프레스를 감지합니다. 감지되면 클립보드에서 텍스트를 읽고 번역을 시작합니다.

- 감지: 동일한 Ctrl 유지 세션 내에서 두 번째 C 누름에서만 발동
- Ctrl 키업은 세션을 리셋합니다 (오발동 방지)
- Shift/Alt/Win 수정자는 제외됩니다 (Ctrl+Shift+C는 global_shortcut으로 처리)
- C 키 반복은 필터링됩니다
- 임계값: 기본 400ms (설정에서 구성 가능)
- 설정에서 ON/OFF 전환 가능 (앱 재시작 필요)

### 4.2 클립보드 모니터링

v0.3.0에서 폐기됨. 번역은 Ctrl+C+C 및 전역 단축키로만 실행됩니다.

### 4.3 메인 번역 화면

- 원문과 번역이 두 개의 나란한 패널에 표시됨
- 원문 패널: 직접 입력, 글자 수, 지우기
- 번역 패널: 재번역, 복사
- 소스 언어 선택 (자동 감지 포함) 및 대상 언어 선택
- 언어 전환
- 모델, 톤, 프리셋 선택

### 4.4 설정 화면

#### 일반 설정
- UI 언어 (11개 언어)
- 최소화된 상태로 시작 (기본값 OFF: 최초 실행 시 창 표시)
- 항상 위에 표시
- 번역에 포커스
- Esc로 닫기
- 외부 클릭 시 닫기
- Ctrl+C+C 빠른 번역 ON/OFF
- 대체 단축키 구성
- 번역 창 열기 단축키
- 기록 열기 단축키
- 번역 기록 저장 ON/OFF
- 알림음

#### API 설정
- 12개 제공자를 테이블 형식으로 표시 (제공자, 상태)
- 확장된 제공자 세부 정보: 환경 변수 이름, API 키, Base URL, 연결 테스트
- 역할 기반 모델 매핑 (default, fast)
- 모델별 동작 모드 (thinking, normal)
- Ollama: 로컬 모델 목록 가져오기 및 선택
- Google Translate: Cloud API / Apps Script 지원
- DeepL: Free / Pro 지원
- 환경 변수는 `setx` + 레지스트리 폴백을 통해 읽음

#### 프리셋 설정
- 10개의 내장 프리셋 (뉴스/학술/기술/이메일/자막/자연스러운/직역/격식/캐주얼/친근함)
- 검색, 이름/설명/시스템 프롬프트 편집

#### 기록 설정
- 최대 기록 항목 수 설정

### 4.5 기록 화면
- 원문과 번역이 나란히 표시됨
- 검색
- 재번역, 복사
- 더 불러오기
- 전체 삭제

### 4.6 시스템 트레이
- 우클릭 메뉴가 있는 상주 아이콘
- 우클릭 메뉴: "Exit"만 표시
- 좌클릭 시 메인 창 표시
- X 버튼은 트레이로 최소화 (앱이 종료되지 않음)
- 트레이 아이콘 우클릭 → "Exit"로 완전히 종료
- 단일 인스턴스: 두 번째 실행 시 기존 인스턴스를 앞으로 가져옴

---

## 5. 지원 번역 제공자

| 제공자 | API 유형 | 기본 모델 |
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

## 6. 지원 UI 언어

Japanese, English, 中文(简体), 中文(繁體), 한국어, Français, Deutsch, Español, Português, Русский, Italiano

---

## 7. 보안

- API 키는 설정 파일이 아닌 OS 환경 변수에 저장됨
- API 키는 프론트엔드로 전송되지 않음 (Rust 측에서 보유)
- 모든 통신에 HTTPS 사용 (로컬 Ollama에만 HTTP 허용)
- 기록 저장은 사용자가 ON/OFF 전환 가능

---

## 8. 데이터 저장

```
%APPDATA%/LLMTranslator/
├── settings.json    # 설정 파일
└── history.jsonl    # 번역 기록 (JSON Lines)
```

---

## 9. 파일 구조

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

## 10. 변경 로그

### v0.3.4

- [x] Windows 10/11 지원
- [x] Tauri v2 + React + TypeScript + Rust
- [x] 시스템 트레이 (X는 트레이로 최소화, 우클릭 "Exit"로 완전 종료)
- [x] Ctrl+C+C 전역 키보드 후크 (Ctrl 유지 세션 방식, 오발동 방지)
- [x] Ctrl+Shift+C 전역 단축키
- [x] 13개 제공자 (OpenAI/Claude/Gemini/DeepSeek/MiMo/Kimi/Qwen/MiniMAX/Ollama/DeepL/Google Translate)
- [x] 탭형 번역 화면 (LLM / Google Translate / ChatGPT Translate)
- [x] 탭 기억 (마지막 탭을 localStorage에 저장, 다음 실행 시 복원)
- [x] Google Translate 탭: Tauri WebView 임베딩, 소스/대상 언어 설정, 브라우저 내비게이션 바
- [x] ChatGPT Translate 탭: WebView 임베딩, 설정 화면에서 언어 설정
- [x] ChatGPT Translate 내비게이션 바 (새로고침/홈, 최상위 페이지 외에서 표시)
- [x] 메인 번역 화면 (2-패널)
- [x] 번역 복사
- [x] 설정 화면 (일반/API/프리셋/기록/Google Translate/ChatGPT Translate)
- [x] 11개 언어 UI
- [x] 10개 번역 프리셋
- [x] 3가지 톤 옵션 (자동/평범/정중)
- [x] API 연결 테스트 및 상태 표시
- [x] 기록 저장/검색/재번역
- [x] 환경 변수 기반 API 키 관리
- [x] 항상 위에 표시
- [x] 최소화된 상태로 시작
- [x] 버전 표시 (상태 표시줄)
- [x] SVG 아이콘 마이그레이션 (모든 사이드바/브라우저 내비게이션 바 아이콘)
- [x] 앱 아이콘 표시 (타이틀 바, 사이드바)
- [x] 설정 상단 바 정리 (아이콘/타이틀 텍스트 제거), × 버튼 제거
- [x] 설정 버튼 UI 개선 (검은색 캡슐), 설정 닫기 버튼 통일
- [x] LLM 탭 레이블 일본어 현지화 (ja.json에 "LLM翻訳")
- [x] ChatGPT 제안 카드 숨기기 (DOM 진단 도구 포함, 설정에서 ON/OFF 전환)
- [x] Ctrl+C+C 오발동 방지 (keyboard_hook 재작성: Ctrl 세션 + 수정자 제외 + 키 반복 제외)
- [x] 클립보드 폴링 제거 (단일 Ctrl+C로는 번역 실행 안 함)
- [x] 닫기 = 트레이 최소화 (window.hide()), 단일 인스턴스 방지
- [x] 시스템 트레이 메뉴 간소화 ("Exit"만 표시)
- [x] ChatGPT Translate 설정 화면 (소스/대상 언어 선택)
- [x] 디버그 로깅 (트리거 소스 식별)
- [x] 기본 설정 최적화 (double_copy_enabled 기본 true, 임계값 400ms)
- [x] 상태 표시줄에 짧은 기본 제공자 모델명 표시
- [x] API 설정 테이블에 Default 열 추가 (행별 Set as Default 버튼 포함)
- [x] Google Translate 최상위 페이지 감지 개선 (hostname + pathname 기반, 번역 후 내비게이션 바 숨김)
- [x] 설정 UI 개선 (타이틀 바 항상 표시, 탭 바/상태 표시줄 숨김, ← 아이콘 제거)
- [x] Windows 자동 시작 (레지스트리 HKCU Run 키, ON/OFF 전환, 기본 OFF, 인용 경로)
- [x] 최소화된 상태로 시작 구현 (`start_minimized` 설정 적용, setup()에서 window.hide())

### v0.3.5

- [x] ChatGPT 제안 카드 숨기기 안정화 (지연 스케줄 8초로 확장, `.prompt-card` CSS 항상 숨김)
- [x] 제안 카드 정리 디버그 로깅 (새로 숨겨진 카드 수만 출력, data 속성으로 이중 집계 방지)
- [x] 자동 시작과 수동 실행을 구분하는 `--auto-start` 플래그
- [x] 트레이 아이콘 더블 클릭 복원 수정 (`SetWindowPos` + `SetForegroundWindow` 앞으로 가져오기, Click 폴백, 700ms 쿨다운)
- [x] 트레이 아이콘 이벤트 로깅 추가 (모든 이벤트 유형 로깅)

### v0.3.6

- [x] ChatGPT 제안 카드 숨기기가 div 유형 카드로 확장 (`button`/`a`/`[role="button"]` 외에 `div`도 스캔, AND 조건 부모 컨테이너 제외 가드 포함)
- [x] DOM 진단 명령: 자식 요소 덤프 추가 (제안 텍스트를 포함하는 요소의 직계 자식 한 레벨 출력)

### v0.3.7

- [x] NSIS 설치 프로그램 언어 선택 다이얼로그 추가 (`displayLanguageSelector: true`, 11개 언어, 선택한 언어 레지스트리에 저장)
- [x] `start_minimized` 기본값을 `false`로 변경 (최초 실행 시 창이 표시되지 않는 문제 수정)
- [x] 설정에서 "최소화된 상태로 시작" 레이블 및 설명 개선 (일반 실행 시 트레이로 최소화됨을 명확히 함)

### v0.3.8

- [x] ChatGPT Translate 탭: LP 요소 숨기기 전환 (헤더 내비게이션, 마케팅 섹션)
- [x] ChatGPT Translate 탭: DOM 진단 도구 (StatusBar 버튼으로 대화형 요소 후보 나열, 설정에서 ON/OFF 전환)
- [x] ChatGPT Translate 탭: HTML+CSS 진단 도구 (헤더/푸터/LP 요소의 CSS 상태 검사, 설정에서 ON/OFF 전환)
- [x] 진단 로그 범위 축소 (DOM: 거대한 페이지 래퍼 제외, HTML+CSS: 번역 폼 본문 제외)
- [x] 시스템 트레이 복원 신뢰성 개선 (트레이 아이콘 수명을 앱 종료까지 연장, WebViewWindow를 hide 후에도 보존하여 더블 클릭 복원 보장)

### v0.4.0

- [x] ChatGPT Translate 탭: 다중 변형 DOM 지원 — 마케팅 LP(변형 A)와 앱/로그인(변형 B) 모두에서 번역 폼이 올바르게 표시됨
- [x] CSS 선택자를 `main#main`에서 `[data-llm-chatgpt-container="true"]` 속성 기반으로 변경, 변형 A(`id="main"` 없음)에서 전체 높이 표시 문제 수정
- [x] 로그인 버튼 보존 — `#contentful-header`를 완전히 숨기는 대신 44px 슬림 바로 재구성하여 마케팅 요소는 숨기고 로그인 버튼은 유지
- [x] 강화된 CTA 숨기기 — 변형 A에서 "ChatGPT で翻訳を開始する", "Try now" 및 유사한 CTA 감지 및 숨김
- [x] 페이지 스크롤 억제 — `html, body { overflow: hidden }`으로 페이지 수준 스크롤바 제거
- [x] 스크롤 위치 리셋 — 레이아웃 마커 후 스크롤 위치를 리셋하여 번역 폼이 화면 밖으로 밀리는 것 방지
- [x] LP 섹션 숨기기 변형 지원 — `#contentful-header`와 `[class*="h-mkt-header-height"]` 간 충돌을 해결하여 LP 요소만 안전하게 숨김

### v0.5
- macOS 지원 (계획)
- 자동 업데이트
- 설정 내보내기/가져오기
- OCR 번역
- 용어집
- 마크다운 보존
- 다중 엔진 비교 번역

---
