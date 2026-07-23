[English](../../SPEC.md) | [日本語](../ja/SPEC.md) | [中文(简体)](../zh-CN/SPEC.md) | [中文(繁體)](../zh-TW/SPEC.md) | [한국어](../ko/SPEC.md) | [Français](../fr/SPEC.md) | [Deutsch](../de/SPEC.md) | [Español](SPEC.md)

# LLM Translator Desktop — Especificación

## 1. Descripción general

### 1.1 Nombre
LLM Translator Desktop

### 1.2 Propósito
Una aplicación de escritorio residente, similar a DeepL Desktop, que permite a los usuarios seleccionar texto en cualquier aplicación y presionar Ctrl+C+C para enviar el texto del portapapeles a una API de LLM y mostrar la traducción en una ventana de traducción.

### 1.3 SO compatible
- Windows 10 / Windows 11
- macOS / Linux — planeado para una versión futura

---

## 2. Flujo básico del usuario

```
Seleccionar texto en cualquier aplicación
↓
Presionar Ctrl+C dos veces (o Ctrl+Shift+C)
↓
La aplicación lee el texto del portapapeles
↓
Lo envía a la API de traducción
↓
Muestra la ventana de traducción
↓
Leer / Copiar / Retraducir
```

---

## 3. Stack tecnológico

| Capa | Tecnología |
|---------|------|
| Framework de escritorio | Tauri v2 |
| Backend | Rust |
| Frontend | React + TypeScript |
| Herramienta de compilación | Vite |
| Plugins principales de Tauri | global-shortcut, clipboard-manager, shell, store, log |
| HTTP | reqwest (lado Rust) |
| API de Windows | SetWindowsHookEx (gancho de teclado) |

---

## 4. Funcionalidades principales

### 4.1 Traducción con Ctrl+C+C

Utiliza un gancho de teclado de bajo nivel de Windows (`SetWindowsHookEx(WH_KEYBOARD_LL)`) para detectar la doble pulsación de Ctrl+C en todo el sistema. Una vez detectada, lee el texto del portapapeles e inicia la traducción.

- Detección: Solo se activa en la segunda pulsación de C dentro de la misma sesión de Ctrl mantenida
- Soltar Ctrl restablece la sesión (evita activaciones falsas)
- Modificadores Shift/Alt/Win excluidos (Ctrl+Shift+C se maneja mediante global_shortcut)
- La repetición de la tecla C se filtra
- Umbral: 400ms por defecto (configurable en ajustes)
- Se puede activar/desactivar en ajustes (requiere reiniciar la aplicación)

### 4.2 Monitoreo del portapapeles

Obsoleto en v0.3.0. La traducción solo se activa mediante Ctrl+C+C y el atajo global.

### 4.3 Pantalla principal de traducción

- Original y traducción mostrados en dos paneles lado a lado
- Panel de origen: entrada directa, conteo de caracteres, borrar
- Panel de traducción: retraducir, copiar
- Selección de idioma de origen (incluyendo detección automática) e idioma de destino
- Intercambio de idiomas
- Selección de modelo, tono y preajuste

### 4.4 Pantalla de ajustes

#### Ajustes generales
- Idioma de la interfaz (11 idiomas)
- Iniciar minimizado (por defecto OFF: la ventana se muestra en el primer inicio)
- Siempre visible
- Enfocar al traducir
- Cerrar con Esc
- Cerrar al hacer clic fuera
- Traducción rápida Ctrl+C+C activar/desactivar
- Configuración de atajo alternativo
- Atajo para abrir ventana de traducción
- Atajo para abrir historial
- Guardar historial de traducción activar/desactivar
- Sonido de notificación

#### Ajustes de API
- 12 proveedores en formato de tabla (Proveedor, Estado)
- Detalles expandidos del proveedor: nombre de variable de entorno, clave API, URL base, prueba de conexión
- Mapeo de modelos basado en roles (default, fast)
- Modo de comportamiento por modelo (thinking, normal)
- Ollama: obtención y selección de lista de modelos locales
- Google Translate: soporte para Cloud API / Apps Script
- DeepL: soporte Free / Pro
- Variables de entorno leídas mediante `setx` + respaldo del registro

#### Ajustes de preajustes
- 10 preajustes integrados (Noticias/Académico/Técnico/Correo electrónico/Subtítulos/Natural/Literal/Formal/Informal/Amigable)
- Búsqueda, edición de nombre/descripción/system prompt

#### Ajustes de historial
- Configuración de entradas máximas de historial

### 4.5 Pantalla de historial
- Original y traducción mostrados lado a lado
- Búsqueda
- Retraducir, copiar
- Cargar más
- Eliminar todo

### 4.6 Bandeja del sistema
- Icono residente con menú de clic derecho
- Menú de clic derecho: solo "Salir"
- Clic izquierdo muestra la ventana principal
- El botón X minimiza a la bandeja (la aplicación no se cierra)
- Clic derecho en el icono de la bandeja → "Salir" para cerrar completamente
- Instancia única: un segundo inicio trae la instancia existente al frente

---

## 5. Proveedores de traducción compatibles

| Proveedor | Tipo de API | Modelo predeterminado |
|-----------|---------|-------------|
| Google Translate / Cloud | Google Cloud Translation API | google-translate-v2 |
| DeepL / Free | DeepL API | deepl |
| DeepL / Pro | DeepL API | deepl |
| OpenAI / ChatGPT | Compatible con OpenAI | gpt-5.5 |
| Gemini / Google | Compatible con OpenAI | gemini-3.1-pro |
| Claude / Anthropic | Compatible con Anthropic | claude-opus-4-8 |
| MiMo / Xiaomi | Compatible con OpenAI | mimo-v2.5-pro |
| DeepSeek / DeepSeek | Compatible con OpenAI | deepseek-v4-pro |
| Kimi / Moonshot | Compatible con OpenAI | kimi-k2.7-code |
| Qwen / Alibaba | Compatible con OpenAI | qwen3.7-max |
| MiniMAX / MiniMAX | Compatible con OpenAI | MiniMax-M2.7 |
| Ollama / Local | Compatible con OpenAI (local) | - |
| Google Translate / Apps Script | Google Apps Script | google-apps-script |

---

## 6. Idiomas de interfaz compatibles

Japanese, English, 中文(简体), 中文(繁體), 한국어, Français, Deutsch, Español, Português, Русский, Italiano

---

## 7. Seguridad

- Claves API almacenadas en variables de entorno del sistema operativo, no en archivos de configuración
- Las claves API nunca se envían al frontend (se mantienen en el lado Rust)
- HTTPS para toda la comunicación (HTTP permitido solo para Ollama local)
- El guardado del historial es activable/desactivable por el usuario

---

## 8. Almacenamiento de datos

```
%APPDATA%/LLMTranslator/
├── settings.json    # Archivo de configuración
└── history.jsonl    # Historial de traducciones (JSON Lines)
```

---

## 9. Estructura de archivos

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

## 10. Registro de cambios

### v0.3.4

- [x] Soporte para Windows 10/11
- [x] Tauri v2 + React + TypeScript + Rust
- [x] Bandeja del sistema (X minimiza a la bandeja, clic derecho "Salir" para cerrar completamente)
- [x] Gancho de teclado global Ctrl+C+C (método de sesión con Ctrl mantenida, prevención de activaciones falsas)
- [x] Atajo global Ctrl+Shift+C
- [x] 13 proveedores (OpenAI/Claude/Gemini/DeepSeek/MiMo/Kimi/Qwen/MiniMAX/Ollama/DeepL/Google Translate)
- [x] Pantalla de traducción con pestañas (LLM / Google Translate / ChatGPT Translate)
- [x] Memoria de pestañas (última pestaña guardada en localStorage, restaurada en el siguiente inicio)
- [x] Pestaña Google Translate: incrustación WebView de Tauri, ajustes de idioma origen/destino, barra de navegación del navegador
- [x] Pestaña ChatGPT Translate: incrustación WebView, ajustes de idioma desde la pantalla de configuración
- [x] Barra de navegación de ChatGPT Translate (recargar/inicio, mostrada fuera de la página principal)
- [x] Pantalla principal de traducción (2 paneles)
- [x] Copiar traducción
- [x] Pantalla de ajustes (General/API/Preajustes/Historial/Google Translate/ChatGPT Translate)
- [x] Interfaz en 11 idiomas
- [x] 10 preajustes de traducción
- [x] 3 opciones de tono (Automático/Neutro/Cortés)
- [x] Prueba de conexión API y visualización de estado
- [x] Guardar/buscar/retraducir historial
- [x] Gestión de claves API basada en variables de entorno
- [x] Siempre visible
- [x] Iniciar minimizado
- [x] Visualización de versión (barra de estado)
- [x] Migración a iconos SVG (todos los iconos de la barra lateral y barra de navegación del navegador)
- [x] Visualización del icono de la aplicación (barra de título, barra lateral)
- [x] Limpieza de la barra superior de ajustes (eliminados icono/texto del título), eliminado botón ×
- [x] Mejora de la interfaz del botón de ajustes (cápsula negra), botón de cerrar ajustes unificado
- [x] Etiqueta de pestaña LLM localizada para japonés ("LLM翻訳" en ja.json)
- [x] Ocultación de tarjetas de sugerencias de ChatGPT (con herramienta de diagnóstico DOM, activar/desactivar en ajustes)
- [x] Prevención de activaciones falsas de Ctrl+C+C (reescritura de keyboard_hook: sesión Ctrl + exclusión de modificadores + exclusión de repetición de tecla)
- [x] Sondeo del portapapeles eliminado (Ctrl+C simple ya no activa la traducción)
- [x] Cerrar = minimizar a bandeja (window.hide()), prevención de instancia única
- [x] Menú de bandeja del sistema simplificado (solo "Salir")
- [x] Pantalla de ajustes de ChatGPT Translate (selección de idioma origen/destino)
- [x] Registro de depuración (identificación de fuente de activación)
- [x] Optimización de ajustes predeterminados (double_copy_enabled por defecto true, umbral 400ms)
- [x] La barra de estado muestra el nombre corto del modelo del proveedor predeterminado
- [x] Columna Default añadida a la tabla de ajustes de API (con botón Establecer como predeterminado por fila)
- [x] Detección mejorada de página principal de Google Translate (basada en hostname + pathname, barra de navegación oculta tras la traducción)
- [x] Mejoras de interfaz de ajustes (barra de título siempre visible, barra de pestañas/estado ocultas, eliminado icono ←)
- [x] Inicio automático con Windows (clave Registry HKCU Run, activar/desactivar, por defecto OFF, ruta entrecomillada)
- [x] Implementación de iniciar minimizado (configuración `start_minimized` aplicada, window.hide() en setup())

### v0.3.5

- [x] Estabilización de ocultación de tarjetas de sugerencias de ChatGPT (programación de retardo extendida a 8s, `.prompt-card` CSS siempre oculto)
- [x] Registro de depuración de limpieza de tarjetas de sugerencias (solo muestra el recuento de tarjetas recién ocultadas, atributo data evita el doble recuento)
- [x] Bandera `--auto-start` para separar el inicio automático del inicio manual
- [x] Corrección de restauración por doble clic en icono de bandeja (`SetWindowPos` + `SetForegroundWindow` para traer al frente, respaldo Click, enfriamiento de 700ms)
- [x] Registro de eventos del icono de bandeja añadido (todos los tipos de eventos registrados)

### v0.3.6

- [x] Ocultación de tarjetas de sugerencias de ChatGPT extendida a tarjetas de tipo div (escanea `div` además de `button`/`a`/`[role="button"]`, con protección de exclusión por contenedor padre mediante condición AND)
- [x] Comando de diagnóstico DOM: volcado de elementos hijos añadido (muestra un nivel de hijos directos para elementos que contienen texto de sugerencia)

### v0.3.7

- [x] Diálogo de selección de idioma del instalador NSIS añadido (`displayLanguageSelector: true`, 11 idiomas, selección guardada en el registro)
- [x] Valor predeterminado de `start_minimized` cambiado a `false` (corrige que la ventana no apareciera en el primer inicio)
- [x] Etiqueta y descripción de "Iniciar minimizado" mejoradas en ajustes (minimiza a la bandeja en el inicio normal)

### v0.3.8

- [x] Pestaña ChatGPT Translate: opción de ocultar elementos LP (navegación de cabecera, secciones de marketing)
- [x] Pestaña ChatGPT Translate: herramienta de diagnóstico DOM (botón en barra de estado para listar candidatos de elementos interactivos, activar/desactivar en ajustes)
- [x] Pestaña ChatGPT Translate: herramienta de diagnóstico HTML+CSS (inspeccionar estado CSS de cabeceras/pies/elementos LP, activar/desactivar en ajustes)
- [x] Alcance del registro de diagnóstico reducido (DOM: excluye envoltorios de página gigantes, HTML+CSS: excluye el cuerpo del formulario de traducción)
- [x] Fiabilidad de restauración de la bandeja del sistema mejorada (vida útil del icono de bandeja extendida hasta el cierre de la aplicación, WebviewWindow preservada después de ocultar para garantizar la restauración por doble clic)

### v0.4.0

- [x] Pestaña ChatGPT Translate: Soporte DOM multi-variante — el formulario de traducción se muestra correctamente tanto en LP de marketing (variante A) como en app/inicio de sesión (variante B)
- [x] Selectores CSS cambiados de `main#main` a basados en atributo `[data-llm-chatgpt-container="true"]`, corrigiendo la visualización a pantalla completa en la variante A (sin `id="main"`)
- [x] Preservación del botón de inicio de sesión — `#contentful-header` rediseñado como una barra delgada de 44px en lugar de ocultarse por completo, manteniendo el botón de inicio de sesión mientras oculta los elementos de marketing
- [x] Ocultación mejorada de CTA — "ChatGPT で翻訳を開始する", "Try now" y CTA similares detectados y ocultados en la variante A
- [x] Supresión de desplazamiento de página — `html, body { overflow: hidden }` elimina la barra de desplazamiento a nivel de página
- [x] Restablecimiento de posición de desplazamiento — posición de desplazamiento restablecida después de los marcadores de diseño para evitar que el formulario de traducción se desplace fuera de la pantalla
- [x] Soporte de variantes para ocultación de secciones LP — conflicto resuelto entre `#contentful-header` y `[class*="h-mkt-header-height"]` para ocultar de forma segura solo los elementos LP

### v0.4.3

- [x] ChatGPT Translate: soporte para 47 idiomas (incluye división en 3 del chino y en 2 del portugués)
- [x] ChatGPT Translate: coincidencia de alias bilingüe (nombres de idiomas en japonés + inglés)
- [x] ChatGPT Translate: aplicación automática de idiomas al iniciar desde ajustes guardados
- [x] ChatGPT Translate: selector de idiomas en la pantalla de ajustes ampliado a 47 idiomas
- [x] ChatGPT Translate: registro de consola y registro de depuración de idiomas (búfer circular sessionStorage, copia desde barra de estado)
- [x] Portapapeles: migración de `navigator.clipboard` al plugin Tauri v2 `clipboard-manager`
- [x] Botones de diagnóstico DOM/HTML+CSS cambiados a comportamiento de alternancia
- [x] Eliminados comandos de borrado innecesarios
- [x] Corregida doble coma en 9 archivos JSON de idiomas

### v0.5
- Soporte para macOS (planeado)
- Actualización automática
- Exportar/importar ajustes
- Traducción OCR
- Glosarios
- Preservación de Markdown
- Comparación de traducción multi-motor

---
