[English](../../SPEC.md) | [日本語](../ja/SPEC.md) | [中文(简体)](../zh-CN/SPEC.md) | [中文(繁體)](../zh-TW/SPEC.md) | [한국어](../ko/SPEC.md) | [Français](../fr/SPEC.md) | [Deutsch](SPEC.md) | [Español](../es/SPEC.md)

# LLM Translator Desktop — Spezifikation

## 1. Überblick

### 1.1 Name
LLM Translator Desktop

### 1.2 Zweck
Eine residente Desktop-App, ähnlich wie DeepL Desktop, mit der Benutzer Text in einer beliebigen Anwendung auswählen und Ctrl+C+C drücken können, um den Zwischenablage-Text an eine LLM-API zu senden und die Übersetzung in einem Übersetzungsfenster anzuzeigen.

### 1.3 Unterstützte Betriebssysteme
- Windows 10 / Windows 11
- macOS / Linux — für zukünftige Versionen geplant

---

## 2. Grundlegender Benutzerablauf

```
Text in einer beliebigen App auswählen
↓
Ctrl+C zweimal drücken (oder Ctrl+Shift+C)
↓
App liest Text aus der Zwischenablage
↓
Sendet an die Übersetzungs-API
↓
Zeigt das Übersetzungsfenster an
↓
Lesen / Kopieren / Erneut übersetzen
```

---

## 3. Tech-Stack

| Ebene | Technologie |
|---------|------|
| Desktop-Framework | Tauri v2 |
| Backend | Rust |
| Frontend | React + TypeScript |
| Build-Tool | Vite |
| Haupt-Tauri-Plugins | global-shortcut, clipboard-manager, shell, store, log |
| HTTP | reqwest (Rust-Seite) |
| Windows-API | SetWindowsHookEx (Keyboard-Hook) |

---

## 4. Kernfunktionen

### 4.1 Ctrl+C+C-Übersetzung

Verwendet einen systemweiten Low-Level-Keyboard-Hook (`SetWindowsHookEx(WH_KEYBOARD_LL)`) von Windows, um den Doppeldruck von Ctrl+C zu erkennen. Nach der Erkennung wird Text aus der Zwischenablage gelesen und die Übersetzung gestartet.

- Erkennung: Wird nur beim zweiten C-Druck innerhalb derselben Ctrl-Halte-Sitzung ausgelöst
- Ctrl-Taste loslassen setzt die Sitzung zurück (verhindert Fehlauslösungen)
- Umschalt-/Alt-/Win-Modifikatoren ausgeschlossen (Ctrl+Shift+C wird von global_shortcut behandelt)
- C-Tastenwiederholung wird herausgefiltert
- Schwellenwert: Standardmäßig 400 ms (in den Einstellungen konfigurierbar)
- Kann in den Einstellungen EIN/AUS geschaltet werden (erfordert App-Neustart)

### 4.2 Zwischenablage-Überwachung

In v0.3.0 veraltet. Die Übersetzung wird nur noch durch Ctrl+C+C und den globalen Shortcut ausgelöst.

### 4.3 Hauptübersetzungsbildschirm

- Quelle und Übersetzung in zwei nebeneinanderliegenden Bereichen
- Quellbereich: Direkteingabe, Zeichenzählung, Löschen
- Übersetzungsbereich: Erneut übersetzen, Kopieren
- Auswahl der Ausgangssprache (einschließlich automatischer Erkennung) und der Zielsprache
- Sprachtausch
- Modell-, Ton- und Preset-Auswahl

### 4.4 Einstellungsbildschirm

#### Allgemeine Einstellungen
- UI-Sprache (11 Sprachen)
- Minimiert starten (Standard AUS: Fenster wird beim ersten Start angezeigt)
- Immer im Vordergrund
- Fokus auf Übersetzen
- Schließen mit Esc
- Schließen bei Klick außerhalb
- Ctrl+C+C-Schnellübersetzung EIN/AUS
- Alternative Shortcut-Konfiguration
- Shortcut zum Öffnen des Übersetzungsfensters
- Shortcut zum Öffnen des Verlaufs
- Übersetzungsverlauf speichern EIN/AUS
- Benachrichtigungston

#### API-Einstellungen
- 12 Anbieter in Tabellenformat (Anbieter, Status)
- Erweiterte Anbieterdetails: Name der Umgebungsvariable, API-Key, Basis-URL, Verbindungstest
- Rollenbasiertes Modell-Mapping (Standard, Schnell)
- Verhaltensmodus pro Modell (Thinking, Normal)
- Ollama: Abrufen und Auswahl lokaler Modellliste
- Google Translate: Cloud API / Apps Script-Unterstützung
- DeepL: Free / Pro-Unterstützung
- Umgebungsvariablen werden über `setx` + Registry-Fallback gelesen

#### Preset-Einstellungen
- 10 integrierte Presets (News/Akademisch/Technisch/E-Mail/Untertitel/Natürlich/Wörtlich/Formell/Leger/Freundlich)
- Suche, Bearbeitung von Name/Beschreibung/System-Prompt

#### Verlaufseinstellungen
- Einstellung für maximale Verlaufseinträge

### 4.5 Verlaufsbildschirm
- Quelle und Übersetzung nebeneinander
- Suche
- Erneut übersetzen, Kopieren
- Mehr laden
- Alle löschen

### 4.6 System-Tray
- Residentes Symbol mit Rechtsklick-Menü
- Rechtsklick-Menü: Nur „Beenden"
- Linksklick zeigt das Hauptfenster
- X-Schaltfläche minimiert in den Tray (App wird nicht beendet)
- Rechtsklick auf Tray-Symbol → „Beenden" zum vollständigen Schließen
- Einzelinstanz: Ein zweiter Start bringt die bestehende Instanz in den Vordergrund

---

## 5. Unterstützte Übersetzungsanbieter

| Anbieter | API-Typ | Standardmodell |
|-----------|---------|-------------|
| Google Translate / Cloud | Google Cloud Translation API | google-translate-v2 |
| DeepL / Free | DeepL API | deepl |
| DeepL / Pro | DeepL API | deepl |
| OpenAI / ChatGPT | OpenAI-kompatibel | gpt-5.5 |
| Gemini / Google | OpenAI-kompatibel | gemini-3.1-pro |
| Claude / Anthropic | Anthropic-kompatibel | claude-opus-4-8 |
| MiMo / Xiaomi | OpenAI-kompatibel | mimo-v2.5-pro |
| DeepSeek / DeepSeek | OpenAI-kompatibel | deepseek-v4-pro |
| Kimi / Moonshot | OpenAI-kompatibel | kimi-k2.7-code |
| Qwen / Alibaba | OpenAI-kompatibel | qwen3.7-max |
| MiniMAX / MiniMAX | OpenAI-kompatibel | MiniMax-M2.7 |
| Ollama / Lokal | OpenAI-kompatibel (lokal) | - |
| Google Translate / Apps Script | Google Apps Script | google-apps-script |

---

## 6. Unterstützte UI-Sprachen

Japanisch, Englisch, 中文(简体), 中文(繁體), 한국어, Français, Deutsch, Español, Português, Русский, Italiano

---

## 7. Sicherheit

- API-Keys werden in den Umgebungsvariablen des Betriebssystems gespeichert, nicht in Konfigurationsdateien
- API-Keys werden niemals an das Frontend gesendet (verbleiben auf der Rust-Seite)
- HTTPS für sämtliche Kommunikation (HTTP nur für lokales Ollama erlaubt)
- Verlaufsspeicherung kann vom Benutzer EIN/AUS geschaltet werden

---

## 8. Datenspeicherung

```
%APPDATA%/LLMTranslator/
├── settings.json    # Einstellungsdatei
└── history.jsonl    # Übersetzungsverlauf (JSON Lines)
```

---

## 9. Dateistruktur

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

- [x] Windows 10/11-Unterstützung
- [x] Tauri v2 + React + TypeScript + Rust
- [x] System-Tray (X minimiert in Tray, Rechtsklick „Beenden" zum vollständigen Schließen)
- [x] Ctrl+C+C globaler Keyboard-Hook (Ctrl-Halte-Sitzungs-Methode, Schutz vor Fehlauslösungen)
- [x] Ctrl+Shift+C globaler Shortcut
- [x] 13 Anbieter (OpenAI/Claude/Gemini/DeepSeek/MiMo/Kimi/Qwen/MiniMAX/Ollama/DeepL/Google Translate)
- [x] Übersetzungsbildschirm mit Tabs (LLM / Google Translate / ChatGPT Translate)
- [x] Tab-Speicher (letzter Tab in localStorage gespeichert, beim nächsten Start wiederhergestellt)
- [x] Google Translate-Tab: Tauri WebView-Einbettung, Quell-/Zielsprachen-Einstellungen, Browser-Navigationsleiste
- [x] ChatGPT Translate-Tab: WebView-Einbettung, Spracheinstellungen vom Einstellungsbildschirm
- [x] ChatGPT Translate-Navigationsleiste (Neu laden/Startseite, außerhalb der Startseite angezeigt)
- [x] Hauptübersetzungsbildschirm (2 Bereiche)
- [x] Übersetzung kopieren
- [x] Einstellungsbildschirm (Allgemein/API/Presets/Verlauf/Google Translate/ChatGPT Translate)
- [x] 11-sprachige UI
- [x] 10 Übersetzungs-Presets
- [x] 3 Tonoptionen (Auto/Schlicht/Höflich)
- [x] API-Verbindungstest und Statusanzeige
- [x] Verlauf speichern/suchen/erneut übersetzen
- [x] Umgebungsvariablen-basierte API-Key-Verwaltung
- [x] Immer im Vordergrund
- [x] Minimiert starten
- [x] Versionsanzeige (Statusleiste)
- [x] SVG-Icon-Migration (alle Sidebar-/Browser-Navigationsleisten-Icons)
- [x] App-Icon-Anzeige (Titelleiste, Sidebar)
- [x] Einstellungs-Titelleiste bereinigt (Icon/Titeltext entfernt), ×-Schaltfläche entfernt
- [x] Einstellungen-Schaltfläche UI verbessert (schwarze Kapsel), Einstellungen-schließen-Schaltfläche vereinheitlicht
- [x] LLM-Tab-Label für Japanisch lokalisiert ("LLM翻訳" in ja.json)
- [x] ChatGPT-Vorschlagskarten ausblenden (mit DOM-Diagnosetool, EIN/AUS in den Einstellungen)
- [x] Ctrl+C+C-Schutz vor Fehlauslösungen (keyboard_hook-Neuschreibung: Ctrl-Sitzung + Modifikator-Ausschluss + Tastenwiederholungs-Ausschluss)
- [x] Zwischenablage-Polling entfernt (einmaliges Ctrl+C löst keine Übersetzung mehr aus)
- [x] Schließen = Tray-Minimierung (window.hide()), Einzelinstanz-Verhinderung
- [x] System-Tray-Menü vereinfacht (nur „Beenden")
- [x] ChatGPT Translate-Einstellungsbildschirm (Quell-/Zielsprachenauswahl)
- [x] Debug-Protokollierung (Identifikation der Auslösequelle)
- [x] Standardeinstellungen optimiert (double_copy_enabled Standard true, Schwellenwert 400 ms)
- [x] Statusleiste zeigt kurzen Standardanbieter-Modellnamen
- [x] Standard-Spalte zur API-Einstellungstabelle hinzugefügt (mit „Als Standard festlegen"-Schaltfläche pro Zeile)
- [x] Google Translate-Startseitenerkennung verbessert (hostname + pathname-basiert, Navigationsleiste nach Übersetzung ausgeblendet)
- [x] Einstellungs-UI verbessert (Titelleiste immer angezeigt, Tab-Leiste/Statusleiste ausgeblendet, ←-Icon entfernt)
- [x] Windows-Autostart (Registry HKCU Run-Schlüssel, EIN/AUS, Standard AUS, Pfad in Anführungszeichen)
- [x] Minimiert starten implementiert (`start_minimized`-Einstellung durchgesetzt, window.hide() in setup())

### v0.3.5

- [x] ChatGPT-Vorschlagskarten-Ausblendung stabilisiert (Verzögerungsplan auf 8 s verlängert, `.prompt-card` CSS immer ausgeblendet)
- [x] Debug-Protokollierung für Vorschlagskarten-Bereinigung (gibt nur Anzahl neu ausgeblendeter Karten aus, Datenattribut verhindert Doppelzählung)
- [x] `--auto-start`-Flag zur Trennung von Autostart und manuellem Start
- [x] Tray-Icon-Doppelklick-Wiederherstellung behoben (`SetWindowPos` + `SetForegroundWindow` In-den-Vordergrund-bringen, Click-Fallback, 700 ms Abklingzeit)
- [x] Tray-Icon-Ereignisprotokollierung hinzugefügt (alle Ereignistypen protokolliert)

### v0.3.6

- [x] ChatGPT-Vorschlagskarten-Ausblendung auf div-Karten erweitert (scannt `div` zusätzlich zu `button`/`a`/`[role="button"]`, mit UND-Bedingungs-Elterncontainer-Ausschlussschutz)
- [x] DOM-Diagnosebefehl: Kindelement-Dump hinzugefügt (gibt eine Ebene direkter Kinder für Elemente aus, die Vorschlagstext enthalten)

### v0.3.7

- [x] NSIS-Installationsprogramm-Sprachauswahldialog hinzugefügt (`displayLanguageSelector: true`, 11 Sprachen, Auswahl in Registry gespeichert)
- [x] `start_minimized`-Standard auf `false` geändert (behebt, dass das Fenster beim ersten Start nicht angezeigt wurde)
- [x] „Minimiert starten"-Label und -Beschreibung in den Einstellungen verbessert (minimiert beim normalen Start in den Tray)

### v0.3.8

- [x] ChatGPT Translate-Tab: LP-Elemente ausblenden umschaltbar (Header-Navigation, Marketing-Abschnitte)
- [x] ChatGPT Translate-Tab: DOM-Diagnosetool (StatusBar-Schaltfläche zum Auflisten interaktiver Elementkandidaten, EIN/AUS in den Einstellungen)
- [x] ChatGPT Translate-Tab: HTML+CSS-Diagnosetool (CSS-Zustand von Headern/Footern/LP-Elementen untersuchen, EIN/AUS in den Einstellungen)
- [x] Diagnoseprotokoll-Umfang eingeschränkt (DOM: riesige Seiten-Wrapper ausschließen, HTML+CSS: Übersetzungsformular-Body ausschließen)
- [x] System-Tray-Wiederherstellungszuverlässigkeit verbessert (Tray-Icon-Lebensdauer bis zum App-Ende verlängert, WebviewWindow nach hide erhalten für garantierte Doppelklick-Wiederherstellung)

### v0.4.0

- [x] ChatGPT Translate-Tab: Multi-Varianten-DOM-Unterstützung — Übersetzungsformular wird sowohl auf Marketing-LP (Variante A) als auch auf App/Login (Variante B) korrekt angezeigt
- [x] CSS-Selektoren von `main#main` auf `[data-llm-chatgpt-container="true"]` geändert (attributbasiert), behebt die Vollbildanzeige bei Variante A (kein `id="main"`)
- [x] Login-Button-Erhaltung — `#contentful-header` wurde in eine schmale 44px-Leiste umgeformt, anstatt vollständig ausgeblendet zu werden, sodass der Login-Button erhalten bleibt und nur Marketing-Elemente ausgeblendet werden
- [x] Erweitertes CTA-Ausblenden — „ChatGPT で翻訳を開始する", „Jetzt ausprobieren" und ähnliche CTAs werden bei Variante A erkannt und ausgeblendet
- [x] Seiten-Scroll-Unterdrückung — `html, body { overflow: hidden }` entfernt die Seiten-Scrollleiste
- [x] Scroll-Position zurücksetzen — Scroll-Position wird nach Layout-Markern zurückgesetzt, um zu verhindern, dass das Übersetzungsformular aus dem Bildschirm verschoben wird
- [x] LP-Abschnitt-Ausblendungs-Variantenunterstützung — Konflikt zwischen `#contentful-header` und `[class*="h-mkt-header-height"]` gelöst, um nur LP-Elemente sicher auszublenden

### v0.5
- macOS-Unterstützung (geplant)
- Automatische Updates
- Einstellungen exportieren/importieren
- OCR-Übersetzung
- Glossare
- Markdown-Erhaltung
- Multi-Engine-Vergleichsübersetzung
