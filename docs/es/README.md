[English](../../README.md) | [日本語](../ja/README.md) | [中文(简体)](../zh-CN/README.md) | [中文(繁體)](../zh-TW/README.md) | [한국어](../ko/README.md) | [Français](../fr/README.md) | [Deutsch](../de/README.md) | [Español](README.md)

# LLM Translator Desktop

Una aplicación de escritorio para Windows para traducción rápida — al igual que DeepL Desktop, selecciona texto en cualquier aplicación y presiona Ctrl+C+C para traducir al instante. Incluye traducción por LLM, una pestaña de Google Translate y una pestaña de ChatGPT Translate, permitiéndote cambiar el método de traducción según tus necesidades.

## Características

### Comunes

* **Traducción con Ctrl+C+C** — Selecciona texto en cualquier aplicación y presiona Ctrl+C dos veces para enviarlo a la pestaña actual para su traducción.
* **Atajo global** — Usa Ctrl+Shift+C (o un atajo personalizado) para activar explícitamente la traducción.
* **Bandeja del sistema** — Cerrar la ventana la minimiza a la bandeja del sistema en lugar de salir. Haz doble clic en el icono de la bandeja para restaurar la ventana, clic derecho → "Salir" para cerrar completamente. La restauración de la ventana funciona de forma fiable incluso después del inicio automático.
* **Instancia única** — Reutiliza la instancia existente para evitar iconos duplicados en la bandeja.
* **Inicio automático con Windows** — Opcionalmente, inicia LLM Translator al iniciar sesión en Windows (por defecto: OFF).
* **Historial** — Guarda, busca y retraduce tu historial de traducciones.
* **Memoria de pestañas** — Recuerda la última pestaña seleccionada y la restaura en el siguiente inicio.
* **Interfaz multilingüe** — Compatible con japonés, inglés, chino (simplificado/tradicional), coreano, francés, alemán, español, portugués, ruso e italiano.

### Traducción por LLM

* **Múltiples proveedores de LLM** — OpenAI, Claude, Gemini, DeepSeek, MiMo, Kimi, Qwen, MiniMAX, Ollama y más.
* **Preajustes de traducción** — Elige entre estilos de traducción: Noticias, Académico, Técnico, Correo electrónico, Subtítulos, Natural, Literal, Formal, Informal y Amigable.
* **Selección de tono** — Tono Automático, Neutro o Cortés para la salida en japonés.
* **Seguridad de claves API** — Las claves API se almacenan en variables de entorno del sistema operativo, nunca en archivos de configuración de la aplicación.
* **Visualización del modelo** — Consulta el proveedor y modelo actual en la barra de estado.

### Pestaña Google Translate

* **Google Translate integrado** — Usa Google Translate dentro de la aplicación sin cambiar al navegador.
* **Configuración de idiomas** — Configura los idiomas de origen y destino desde la pantalla de ajustes.
* **Barra de navegación inteligente** — Los botones de navegación (atrás, adelante, recargar, inicio) aparecen solo cuando son necesarios (por ejemplo, páginas de inicio de sesión, páginas externas), ocultos en la página normal de traducción.
* **Integración con Ctrl+C+C** — Envía el texto seleccionado desde cualquier aplicación a la pestaña de Google Translate.

### Pestaña ChatGPT Translate

* **ChatGPT integrado** — Abre la interfaz web de ChatGPT dentro de la aplicación y envía instrucciones de traducción.
* **Soporte para 47 idiomas** — Elige entre 47 idiomas, incluyendo chino (simplificado/tradicional/Hong Kong) y portugués (Brasil/Portugal). Los nombres de idiomas se reconocen tanto en japonés como en inglés.
* **Aplicación automática de idiomas** — Los idiomas de origen y destino guardados se aplican automáticamente a la página de ChatGPT al iniciar.
* **Registros de depuración de idioma / consola** — Registros de diagnóstico integrados para la selección de idioma y la salida de consola, copiables desde la barra de estado. Activar/desactivar en ajustes.
* **Ocultación de elementos LP** — Activar/desactivar en ajustes. Oculta la navegación de marketing y las secciones de la página de ChatGPT para que puedas concentrarte en el formulario de traducción.
* **Soporte DOM multi-variante** — Se adapta automáticamente a diferentes estructuras de página de ChatGPT (LP de marketing vs. variantes de app/inicio de sesión). Utiliza selectores basados en el atributo `data-llm-chatgpt-container` en lugar de `main#main`, preservando el botón de inicio de sesión mientras oculta solo los elementos de marketing.
* **Supresión de desplazamiento de página** — Elimina la barra de desplazamiento a nivel de página que aparecía en la variante A, manteniendo la interfaz de traducción contenida dentro de la ventana.
* **Ocultación de tarjetas de sugerencias** — Oculta automáticamente las tarjetas de sugerencias ("Hazlo más profesional", "Explícamelo como si tuviera 5 años", "Haz que suene más natural", etc.). Maneja tarjetas de tipo `button`, `a`, `[role="button"]` y `div`. Múltiples reintentos con retardo y un MutationObserver absorben las diferencias del DOM entre distintos PCs y entornos de navegador.
* **Herramientas de diagnóstico DOM / HTML+CSS** — Herramientas de depuración para inspeccionar la estructura DOM y el estado CSS de ChatGPT. Activar/desactivar en ajustes.
* **Barra de navegación** — Los botones de recargar e inicio aparecen cuando no se está en la página principal.
* **Integración con Ctrl+C+C** — Envía el texto seleccionado desde cualquier aplicación a la pestaña de ChatGPT Translate.

## Requisitos

- Windows 10 / Windows 11
- Una clave API para cada proveedor de IA que desees usar

## Instalación

Descarga el último `LLM-Translator-Desktop-Setup.exe` desde [Releases](https://github.com/soheidon/LLM-Translator/releases) y ejecútalo. El instalador ofrece 11 idiomas para elegir al iniciarse (tu selección se recuerda para futuras instalaciones).

## Uso

1. Inicia la aplicación — aparece la ventana de traducción (activa "Iniciar minimizado" en ajustes para iniciar en la bandeja)
2. Configura tus claves API en Ajustes → API
3. Selecciona texto en cualquier aplicación y presiona Ctrl+C dos veces para traducir
4. Alternativamente, usa Ctrl+Shift+C
5. También puedes escribir o pegar texto manualmente en la pantalla principal

## Desarrollo

```bash
# Instalar dependencias
npm install

# Iniciar en modo desarrollo
npm run tauri dev

# Compilar
npm run tauri build
```

## Stack tecnológico

- [Tauri v2](https://tauri.app/) — Framework de aplicaciones de escritorio
- [Rust](https://www.rust-lang.org/) — Backend (comunicación API, gancho de teclado, gestión de configuración)
- [React](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) — Interfaz de usuario frontend
- [Vite](https://vitejs.dev/) — Herramienta de compilación

## Licencia

[MIT](../../LICENSE)

---
