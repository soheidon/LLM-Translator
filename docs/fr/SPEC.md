[English](SPEC.md) | [日本語](docs/ja/SPEC.md) | [中文(简体)](docs/zh-CN/SPEC.md) | [한국어](docs/ko/SPEC.md) | [Français](docs/fr/SPEC.md) | [Deutsch](docs/de/SPEC.md) | [Español](docs/es/SPEC.md)

# LLM Translator Desktop — Spécification

## 1. Aperçu

### 1.1 Nom
LLM Translator Desktop

### 1.2 Objectif
Une application de bureau résidente, similaire à DeepL Desktop, qui permet aux utilisateurs de sélectionner du texte dans n'importe quelle application et d'appuyer sur Ctrl+C+C pour envoyer le texte du presse-papiers à une API LLM et afficher la traduction dans une fenêtre de traduction.

### 1.3 OS pris en charge
- Windows 10 / Windows 11
- macOS / Linux — prévu pour une version future

---

## 2. Flux utilisateur de base

```
Sélectionner du texte dans n'importe quelle application
↓
Appuyer deux fois sur Ctrl+C (ou Ctrl+Shift+C)
↓
L'application lit le texte du presse-papiers
↓
Envoie à l'API de traduction
↓
Affiche la fenêtre de traduction
↓
Lire / Copier / Retraduire
```

---

## 3. Stack technique

| Couche | Technologie |
|---------|------|
| Framework Bureau | Tauri v2 |
| Backend | Rust |
| Frontend | React + TypeScript |
| Outil de Build | Vite |
| Plugins Tauri principaux | global-shortcut, clipboard-manager, shell, store, log |
| HTTP | reqwest (côté Rust) |
| API Windows | SetWindowsHookEx (hook clavier) |

---

## 4. Fonctionnalités principales

### 4.1 Traduction Ctrl+C+C

Utilise un hook clavier bas niveau Windows (`SetWindowsHookEx(WH_KEYBOARD_LL)`) pour détecter la double pression de Ctrl+C à l'échelle du système. Une fois détectée, lit le texte du presse-papiers et démarre la traduction.

- Détection : Se déclenche uniquement sur la deuxième pression de C dans la même session Ctrl maintenue
- Le relâchement de Ctrl réinitialise la session (empêche les faux déclenchements)
- Les modificateurs Shift/Alt/Win sont exclus (Ctrl+Shift+C est géré par global_shortcut)
- La répétition de la touche C est filtrée
- Seuil : 400ms par défaut (configurable dans les paramètres)
- Peut être activé/désactivé dans les paramètres (nécessite un redémarrage de l'application)

### 4.2 Surveillance du presse-papiers

Obsolète depuis v0.3.0. La traduction est déclenchée uniquement par Ctrl+C+C et le raccourci global.

### 4.3 Écran principal de traduction

- Source et traduction affichées dans deux volets côte à côte
- Volet source : saisie directe, compteur de caractères, effacer
- Volet traduction : retraduire, copier
- Sélection de la langue source (incluant la détection automatique) et de la langue cible
- Permutation des langues
- Sélection du modèle, du ton et du préréglage

### 4.4 Écran des paramètres

#### Paramètres généraux
- Langue de l'interface (11 langues)
- Démarrage réduit (par défaut OFF : la fenêtre s'affiche au premier lancement)
- Toujours au premier plan
- Focus sur la traduction
- Fermer avec Échap
- Fermer au clic extérieur
- Traduction rapide Ctrl+C+C ON/OFF
- Configuration de raccourci alternatif
- Raccourci pour ouvrir la fenêtre de traduction
- Raccourci pour ouvrir l'historique
- Sauvegarde de l'historique de traduction ON/OFF
- Son de notification

#### Paramètres API
- 12 fournisseurs en format tableau (Fournisseur, Statut)
- Détails étendus du fournisseur : nom de la variable d'environnement, clé API, URL de base, test de connexion
- Mappage de modèles basé sur les rôles (par défaut, rapide)
- Mode de comportement par modèle (thinking, normal)
- Ollama : récupération et sélection de la liste des modèles locaux
- Google Translate : support API Cloud / Apps Script
- DeepL : support Free / Pro
- Variables d'environnement lues via `setx` + fallback registre

#### Paramètres des préréglages
- 10 préréglages intégrés (Actualité/Académique/Technique/Email/Sous-titres/Naturel/Littéral/Formel/Décontracté/Amical)
- Recherche, édition du nom/description/prompt système

#### Paramètres d'historique
- Réglage du nombre maximal d'entrées d'historique

### 4.5 Écran d'historique
- Source et traduction affichées côte à côte
- Recherche
- Retraduire, copier
- Charger plus
- Tout supprimer

### 4.6 Zone de notification
- Icône résidente avec menu contextuel
- Menu contextuel : « Exit » uniquement
- Clic gauche affiche la fenêtre principale
- Le bouton X réduit dans la zone de notification (l'application ne quitte pas)
- Clic droit sur l'icône de la zone de notification → « Exit » pour quitter complètement
- Instance unique : un deuxième lancement amène l'instance existante au premier plan

---

## 5. Fournisseurs de traduction pris en charge

| Fournisseur | Type d'API | Modèle par défaut |
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

## 6. Langues d'interface prises en charge

Japanese, English, 中文(简体), 中文(繁體), 한국어, Français, Deutsch, Español, Português, Русский, Italiano

---

## 7. Sécurité

- Clés API stockées dans les variables d'environnement du système, pas dans les fichiers de configuration
- Clés API jamais envoyées au frontend (conservées côté Rust)
- HTTPS pour toutes les communications (HTTP autorisé uniquement pour Ollama local)
- Sauvegarde de l'historique activable/désactivable par l'utilisateur ON/OFF

---

## 8. Stockage des données

```
%APPDATA%/LLMTranslator/
├── settings.json    # Fichier de paramètres
└── history.jsonl    # Historique de traduction (JSON Lines)
```

---

## 9. Structure des fichiers

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

## 10. Journal des modifications

### v0.3.4

- [x] Support Windows 10/11
- [x] Tauri v2 + React + TypeScript + Rust
- [x] Zone de notification (X réduit dans la zone de notification, clic droit « Exit » pour quitter complètement)
- [x] Hook clavier global Ctrl+C+C (méthode de session Ctrl maintenue, prévention des faux déclenchements)
- [x] Raccourci global Ctrl+Shift+C
- [x] 13 fournisseurs (OpenAI/Claude/Gemini/DeepSeek/MiMo/Kimi/Qwen/MiniMAX/Ollama/DeepL/Google Translate)
- [x] Écran de traduction par onglets (LLM / Google Translate / ChatGPT Translate)
- [x] Mémoire d'onglet (dernier onglet sauvegardé dans localStorage, restauré au prochain lancement)
- [x] Onglet Google Translate : intégration WebView Tauri, paramètres de langue source/cible, barre de navigation du navigateur
- [x] Onglet ChatGPT Translate : intégration WebView, paramètres de langue depuis l'écran des paramètres
- [x] Barre de navigation ChatGPT Translate (actualiser/accueil, affichée hors de la page d'accueil)
- [x] Écran principal de traduction (2 volets)
- [x] Copie de traduction
- [x] Écran des paramètres (Général/API/Préréglages/Historique/Google Translate/ChatGPT Translate)
- [x] Interface en 11 langues
- [x] 10 préréglages de traduction
- [x] 3 options de ton (Auto/Simple/Poli)
- [x] Test de connexion API et affichage du statut
- [x] Sauvegarde/recherche/retraduction de l'historique
- [x] Gestion des clés API basée sur les variables d'environnement
- [x] Toujours au premier plan
- [x] Démarrage réduit
- [x] Affichage de la version (barre d'état)
- [x] Migration des icônes SVG (toutes les icônes de la barre latérale/barre de navigation du navigateur)
- [x] Affichage de l'icône de l'application (barre de titre, barre latérale)
- [x] Nettoyage de la barre supérieure des paramètres (icône/texte du titre supprimés), bouton × supprimé
- [x] Amélioration de l'interface du bouton Paramètres (capsule noire), bouton de fermeture des paramètres unifié
- [x] Libellé de l'onglet LLM localisé pour le japonais (« LLM翻訳 » dans ja.json)
- [x] Masquage des cartes de suggestion ChatGPT (avec outil de diagnostic DOM, activation/désactivation dans les paramètres)
- [x] Prévention des faux déclenchements Ctrl+C+C (réécriture de keyboard_hook : session Ctrl + exclusion des modificateurs + exclusion de la répétition de touche)
- [x] Surveillance du presse-papiers supprimée (un seul Ctrl+C ne déclenche plus la traduction)
- [x] Fermer = réduire dans la zone de notification (window.hide()), prévention d'instance unique
- [x] Menu de la zone de notification simplifié (« Exit » uniquement)
- [x] Écran des paramètres ChatGPT Translate (sélection de la langue source/cible)
- [x] Journalisation de débogage (identification de la source du déclenchement)
- [x] Optimisation des paramètres par défaut (double_copy_enabled par défaut true, seuil 400ms)
- [x] La barre d'état affiche le nom court du modèle du fournisseur par défaut
- [x] Colonne Défaut ajoutée au tableau des paramètres API (avec bouton Définir par défaut par ligne)
- [x] Détection de la page d'accueil Google Translate améliorée (basée sur hostname + pathname, barre de navigation masquée après la traduction)
- [x] Améliorations de l'interface des paramètres (barre de titre toujours affichée, barre d'onglets/barre d'état masquées, icône ← supprimée)
- [x] Démarrage automatique Windows (clé Registre HKCU Run, activation/désactivation, par défaut OFF, chemin entre guillemets)
- [x] Implémentation du démarrage réduit (paramètre `start_minimized` appliqué, window.hide() dans setup())

### v0.3.5

- [x] Stabilisation du masquage des cartes de suggestion ChatGPT (programme de délai étendu à 8s, `.prompt-card` toujours masqué en CSS)
- [x] Journalisation de débogage du nettoyage des cartes de suggestion (affiche uniquement le nombre de cartes nouvellement masquées, l'attribut data empêche le double comptage)
- [x] Option `--auto-start` pour séparer le démarrage automatique du lancement manuel
- [x] Correction de la restauration par double-clic sur l'icône de la zone de notification (`SetWindowPos` + `SetForegroundWindow` pour le premier plan, fallback Click, délai de récupération de 700ms)
- [x] Journalisation des événements de l'icône de la zone de notification ajoutée (tous les types d'événements journalisés)

### v0.3.6

- [x] Masquage des cartes de suggestion ChatGPT étendu aux cartes de type div (analyse les `div` en plus de `button`/`a`/`[role="button"]`, avec garde d'exclusion du conteneur parent par condition AND)
- [x] Commande de diagnostic DOM : vidage des éléments enfants ajouté (affiche un niveau d'enfants directs pour les éléments contenant du texte de suggestion)

### v0.3.7

- [x] Dialogue de sélection de langue de l'installateur NSIS ajouté (`displayLanguageSelector: true`, 11 langues, sélection sauvegardée dans le registre)
- [x] Valeur par défaut de `start_minimized` changée à `false` (corrige la fenêtre qui n'apparaissait pas au premier lancement)
- [x] Libellé et description de « Start minimized » améliorés dans les paramètres (réduit dans la zone de notification lors d'un lancement normal)

### v0.3.8

- [x] Onglet ChatGPT Translate : bascule de masquage des éléments LP (navigation d'en-tête, sections marketing)
- [x] Onglet ChatGPT Translate : outil de diagnostic DOM (bouton dans la barre d'état pour lister les éléments interactifs candidats, activation/désactivation dans les paramètres)
- [x] Onglet ChatGPT Translate : outil de diagnostic HTML+CSS (inspecter l'état CSS des en-têtes/pieds de page/éléments LP, activation/désactivation dans les paramètres)
- [x] Portée du journal de diagnostic réduite (DOM : exclure les grands wrappers de page, HTML+CSS : exclure le corps du formulaire de traduction)
- [x] Fiabilité de restauration de la zone de notification améliorée (durée de vie de l'icône de la zone de notification étendue jusqu'à la sortie de l'application, WebviewWindow préservée après masquage pour une restauration garantie par double-clic)

### v0.4.0

- [x] Onglet ChatGPT Translate : support DOM multi-variantes — le formulaire de traduction s'affiche correctement sur la LP marketing (variante A) et app/connexion (variante B)
- [x] Sélecteurs CSS changés de `main#main` à `[data-llm-chatgpt-container="true"]` basés sur attribut, corrigeant l'affichage pleine hauteur sur la variante A (pas de `id="main"`)
- [x] Préservation du bouton de connexion — `#contentful-header` redimensionné en une barre fine de 44px au lieu d'être entièrement masqué, conservant le bouton de connexion tout en masquant les éléments marketing
- [x] Masquage CTA amélioré — « ChatGPT で翻訳を開始する », « Try now » et CTA similaires détectés et masqués sur la variante A
- [x] Suppression du défilement de page — `html, body { overflow: hidden }` élimine la barre de défilement au niveau de la page
- [x] Réinitialisation de la position de défilement — réinitialisation de la position de défilement après les marqueurs de mise en page pour empêcher le formulaire de traduction de sortir de l'écran
- [x] Support des variantes de masquage des sections LP — résolution du conflit entre `#contentful-header` et `[class*="h-mkt-header-height"]` pour masquer en toute sécurité uniquement les éléments LP

### v0.5
- Support macOS (prévu)
- Mise à jour automatique
- Export/import des paramètres
- Traduction OCR
- Glossaires
- Préservation du Markdown
- Traduction comparative multi-moteurs
