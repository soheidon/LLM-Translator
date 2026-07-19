[English](../../README.md) | [日本語](../ja/README.md) | [中文(简体)](../zh-CN/README.md) | [中文(繁體)](../zh-TW/README.md) | [한국어](../ko/README.md) | [Français](README.md) | [Deutsch](../de/README.md) | [Español](../es/README.md)

# LLM Translator Desktop

Une application de bureau Windows pour la traduction rapide — comme DeepL Desktop, sélectionnez du texte dans n'importe quelle application et appuyez sur Ctrl+C+C pour traduire instantanément. Inclut la traduction par LLM, un onglet Google Translate et un onglet ChatGPT Translate, vous permettant de changer de méthode de traduction selon vos besoins.

## Fonctionnalités

### Communes

* **Traduction Ctrl+C+C** — Sélectionnez du texte dans n'importe quelle application et appuyez deux fois sur Ctrl+C pour l'envoyer à l'onglet actuel pour traduction.
* **Raccourci global** — Utilisez Ctrl+Shift+C (ou un raccourci personnalisé) pour déclencher explicitement la traduction.
* **Zone de notification** — Fermer la fenêtre la réduit dans la zone de notification au lieu de quitter. Double-cliquez sur l'icône de la zone de notification pour restaurer la fenêtre, clic droit → « Exit » pour quitter complètement. La restauration de la fenêtre fonctionne de manière fiable même après le démarrage automatique.
* **Instance unique** — Réutilise l'instance existante pour éviter les icônes en double dans la zone de notification.
* **Démarrage automatique avec Windows** — Lancez optionnellement LLM Translator lorsque vous vous connectez à Windows (par défaut : OFF).
* **Historique** — Enregistrez, recherchez et retraduisez votre historique de traduction.
* **Mémoire d'onglet** — Mémorise le dernier onglet sélectionné et le restaure au prochain lancement.
* **Interface multilingue** — Prend en charge le japonais, l'anglais, le chinois (simplifié/traditionnel), le coréen, le français, l'allemand, l'espagnol, le portugais, le russe et l'italien.

### Traduction LLM

* **Plusieurs fournisseurs LLM** — OpenAI, Claude, Gemini, DeepSeek, MiMo, Kimi, Qwen, MiniMAX, Ollama et plus.
* **Préréglages de traduction** — Choisissez parmi les styles de traduction Actualité, Académique, Technique, Email, Sous-titres, Naturel, Littéral, Formel, Décontracté et Amical.
* **Sélection du ton** — Ton Auto, Simple ou Poli pour la sortie en japonais.
* **Sécurité des clés API** — Les clés API sont stockées dans les variables d'environnement du système, jamais dans les fichiers de paramètres de l'application.
* **Affichage du modèle** — Consultez le fournisseur et le modèle actuels dans la barre d'état.

### Onglet Google Translate

* **Google Translate intégré** — Utilisez Google Translate dans l'application sans passer par un navigateur.
* **Configuration des langues** — Définissez les langues source et cible depuis l'écran des paramètres.
* **Barre de navigation intelligente** — Les boutons de navigation (retour, avant, actualiser, accueil) n'apparaissent que lorsque nécessaire (par ex., pages de connexion, pages externes), masqués sur la page de traduction normale.
* **Intégration Ctrl+C+C** — Envoyez le texte sélectionné depuis n'importe quelle application vers l'onglet Google Translate.

### Onglet ChatGPT Translate

* **ChatGPT intégré** — Ouvrez l'interface web de ChatGPT dans l'application et envoyez des invites de traduction.
* **Configuration des langues** — Définissez les langues source et cible depuis l'écran des paramètres.
* **Masquage des éléments LP** — Activez/Désactivez dans les paramètres. Masquez la navigation marketing et les sections de la page ChatGPT pour vous concentrer sur le formulaire de traduction.
* **Support DOM multi-variantes** — S'adapte automatiquement aux différentes structures de page ChatGPT (LP marketing vs. variantes app/connexion). Utilise des sélecteurs basés sur l'attribut `data-llm-chatgpt-container` au lieu de `main#main`, préservant le bouton de connexion tout en masquant uniquement les éléments marketing.
* **Suppression du défilement de page** — Élimine la barre de défilement au niveau de la page qui apparaissait sur la variante A, gardant l'interface de traduction contenue dans la fenêtre.
* **Masquage des cartes de suggestion** — Masque automatiquement les cartes de suggestion (« Make it more business-like », « Explain like I'm 5 », « Make it sound more natural », etc.). Gère les cartes de type `button`, `a`, `[role="button"]` et `div`. Plusieurs tentatives différées et un MutationObserver absorbent les différences DOM entre les PC et les environnements de navigateur.
* **Outils de diagnostic DOM / HTML+CSS** — Outils de débogage pour inspecter la structure DOM et l'état CSS de ChatGPT. Activez/Désactivez dans les paramètres.
* **Barre de navigation** — Les boutons Actualiser et Accueil apparaissent lorsque vous n'êtes pas sur la page d'accueil.
* **Intégration Ctrl+C+C** — Envoyez le texte sélectionné depuis n'importe quelle application vers l'onglet ChatGPT Translate.

## Prérequis

- Windows 10 / Windows 11
- Une clé API pour chaque fournisseur d'IA que vous souhaitez utiliser

## Installation

Téléchargez le dernier `LLM-Translator-Desktop-Setup.exe` depuis [Releases](https://github.com/soheidon/LLM-Translator/releases) et exécutez-le. L'installateur propose 11 langues au choix au lancement (votre sélection est mémorisée pour les installations futures).

## Utilisation

1. Lancez l'application — la fenêtre de traduction apparaît (activez « Start minimized » dans les paramètres pour démarrer dans la zone de notification)
2. Configurez vos clés API dans Paramètres → API
3. Sélectionnez du texte dans n'importe quelle application et appuyez deux fois sur Ctrl+C pour traduire
4. Vous pouvez également utiliser Ctrl+Shift+C
5. Vous pouvez aussi taper ou coller du texte manuellement dans l'écran principal

## Développement

```bash
# Installer les dépendances
npm install

# Démarrer en mode développement
npm run tauri dev

# Compiler
npm run tauri build
```

## Stack technique

- [Tauri v2](https://tauri.app/) — Framework d'application de bureau
- [Rust](https://www.rust-lang.org/) — Backend (communication API, hook clavier, gestion de la configuration)
- [React](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) — Interface utilisateur frontend
- [Vite](https://vitejs.dev/) — Outil de build

## Licence

[MIT](../../LICENSE)
