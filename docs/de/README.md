[English](../../README.md) | [日本語](../ja/README.md) | [中文(简体)](../zh-CN/README.md) | [中文(繁體)](../zh-TW/README.md) | [한국어](../ko/README.md) | [Français](../fr/README.md) | [Deutsch](README.md) | [Español](../es/README.md)

# LLM Translator Desktop

Eine Windows-Desktop-App für schnelle Übersetzungen — genau wie DeepL Desktop: Text in einer beliebigen App auswählen und Ctrl+C+C drücken, um sofort zu übersetzen. Enthält LLM-Übersetzung, einen Google Translate-Tab und einen ChatGPT Translate-Tab, sodass Sie je nach Bedarf zwischen den Übersetzungsmethoden wechseln können.

## Funktionen

### Allgemein

* **Ctrl+C+C-Übersetzung** — Wählen Sie Text in einer beliebigen App aus und drücken Sie Ctrl+C zweimal, um ihn zur Übersetzung an den aktuellen Tab zu senden.
* **Globaler Shortcut** — Verwenden Sie Ctrl+Shift+C (oder einen benutzerdefinierten Shortcut), um die Übersetzung explizit auszulösen.
* **System-Tray** — Beim Schließen des Fensters wird die App in den System-Tray minimiert, anstatt beendet zu werden. Doppelklicken Sie auf das Tray-Symbol, um das Fenster wiederherzustellen, Rechtsklick → „Beenden", um die App vollständig zu schließen. Die Fensterwiederherstellung funktioniert auch nach dem Autostart zuverlässig.
* **Einzelinstanz** — Verwendet die bestehende Instanz wieder, um doppelte Tray-Symbole zu vermeiden.
* **Autostart mit Windows** — Starten Sie LLM Translator optional bei der Windows-Anmeldung (Standard: AUS).
* **Verlauf** — Speichern, durchsuchen und erneut übersetzen Sie Ihren Übersetzungsverlauf.
* **Tab-Speicher** — Merkt sich den zuletzt ausgewählten Tab und stellt ihn beim nächsten Start wieder her.
* **Mehrsprachige UI** — Unterstützt Japanisch, Englisch, Chinesisch (Vereinfacht/Traditionell), Koreanisch, Französisch, Deutsch, Spanisch, Portugiesisch, Russisch und Italienisch.

### LLM-Übersetzung

* **Mehrere LLM-Anbieter** — OpenAI, Claude, Gemini, DeepSeek, MiMo, Kimi, Qwen, MiniMAX, Ollama und mehr.
* **Übersetzungs-Presets** — Wählen Sie aus News, Akademisch, Technisch, E-Mail, Untertitel, Natürlich, Wörtlich, Formell, Leger und Freundlich.
* **Tonauswahl** — Auto, Schlicht oder Höflich für die japanische Ausgabe.
* **API-Key-Sicherheit** — API-Keys werden in den Umgebungsvariablen des Betriebssystems gespeichert, niemals in den App-Einstellungsdateien.
* **Modellanzeige** — Sehen Sie den aktuellen Anbieter und das Modell in der Statusleiste.

### Google Translate-Tab

* **Eingebetteter Google Translate** — Verwenden Sie Google Translate innerhalb der App, ohne zum Browser wechseln zu müssen.
* **Sprachkonfiguration** — Legen Sie Quell- und Zielsprache im Einstellungsbildschirm fest.
* **Intelligente Navigationsleiste** — Navigationsschaltflächen (Zurück, Vorwärts, Neu laden, Startseite) erscheinen nur bei Bedarf (z. B. bei Login-Seiten, externen Seiten) und sind auf der normalen Übersetzungsseite ausgeblendet.
* **Ctrl+C+C-Integration** — Senden Sie ausgewählten Text aus einer beliebigen App an den Google Translate-Tab.

### ChatGPT Translate-Tab

* **Eingebetteter ChatGPT** — Öffnen Sie die ChatGPT-Weboberfläche innerhalb der App und senden Sie Übersetzungs-Prompts.
* **Sprachkonfiguration** — Legen Sie Quell- und Zielsprache im Einstellungsbildschirm fest.
* **LP-Elemente ausblenden** — Ein/Aus in den Einstellungen. Blendet Marketing-Navigation und -Abschnitte von der ChatGPT-Seite aus, damit Sie sich auf das Übersetzungsformular konzentrieren können.
* **Multi-Varianten-DOM-Unterstützung** — Passt sich automatisch an verschiedene ChatGPT-Seitenstrukturen an (Marketing-LP vs. App/Login-Varianten). Verwendet `data-llm-chatgpt-container`-attributbasierte Selektoren anstelle von `main#main`, sodass der Login-Button erhalten bleibt und nur Marketing-Elemente ausgeblendet werden.
* **Seiten-Scroll-Unterdrückung** — Entfernt die Seiten-Scrollleiste, die bei Variante A auftrat, und hält die Übersetzungs-UI innerhalb des Viewports.
* **Vorschlagskarten ausblenden** — Blendet Vorschlagskarten automatisch aus („Mach es geschäftlicher", „Erklär's mir, als wäre ich 5", „Lass es natürlicher klingen" usw.). Behandelt `button`-, `a`-, `[role="button"]`- und `div`-Karten. Mehrfache verzögerte Wiederholungen und ein MutationObserver gleichen DOM-Unterschiede zwischen PCs und Browser-Umgebungen aus.
* **DOM-Diagnose / HTML+CSS-Diagnosetools** — Debug-Tools zur Untersuchung der DOM-Struktur und des CSS-Zustands von ChatGPT. Ein/Aus in den Einstellungen.
* **Navigationsleiste** — Neu-laden- und Startseite-Schaltflächen erscheinen, wenn Sie sich nicht auf der Startseite befinden.
* **Ctrl+C+C-Integration** — Senden Sie ausgewählten Text aus einer beliebigen App an den ChatGPT Translate-Tab.

## Voraussetzungen

- Windows 10 / Windows 11
- Ein API-Key für jeden KI-Anbieter, den Sie verwenden möchten

## Installation

Laden Sie die neueste `LLM-Translator-Desktop-Setup.exe` von [Releases](https://github.com/soheidon/LLM-Translator/releases) herunter und führen Sie sie aus. Das Installationsprogramm bietet beim Start 11 Sprachen zur Auswahl (Ihre Auswahl wird für zukünftige Installationen gespeichert).

## Verwendung

1. Starten Sie die App — das Übersetzungsfenster erscheint (aktivieren Sie „Minimiert starten" in den Einstellungen, um stattdessen im Tray zu starten)
2. Konfigurieren Sie Ihre API-Keys unter Einstellungen → API
3. Wählen Sie Text in einer beliebigen App aus und drücken Sie Ctrl+C zweimal zum Übersetzen
4. Alternativ können Sie Ctrl+Shift+C verwenden
5. Sie können Text auch manuell auf dem Hauptbildschirm eingeben oder einfügen

## Entwicklung

```bash
# Abhängigkeiten installieren
npm install

# Im Entwicklungsmodus starten
npm run tauri dev

# Build
npm run tauri build
```

## Tech-Stack

- [Tauri v2](https://tauri.app/) — Desktop-Anwendungs-Framework
- [Rust](https://www.rust-lang.org/) — Backend (API-Kommunikation, Keyboard-Hook, Konfigurationsverwaltung)
- [React](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) — Frontend-UI
- [Vite](https://vitejs.dev/) — Build-Tool

## Lizenz

[MIT](../../LICENSE)
