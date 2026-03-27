# Publicidad

Aplicación de escritorio **Tauri 2** + **React 19** + **Vite 7** que muestra un **ticker de noticias** en una franja horizontal sobre el resto de ventanas.

## Qué hace

- Ventana de **200 px** de alto, **ancho del monitor de trabajo**, **siempre visible** (`alwaysOnTop`) y sin barra de título decorativa.
- Titulares con **animación de izquierda a derecha**; al hacer clic se abre el artículo en el navegador predeterminado.
- Las noticias se cargan desde el **RSS mundial de la BBC** (`feeds.bbci.co.uk/news/world/rss.xml`), obtenido y parseado en **Rust** (comando Tauri). Las imágenes se extraen del HTML del feed cuando están disponibles.
- Actualización periódica del feed desde el frontend (intervalo en el orden de minutos).

## Requisitos

- [Node.js](https://nodejs.org/) (incluye `npm`)
- [Rust](https://www.rust-lang.org/tools/install) y el target de escritorio de tu SO
- **Windows 10/11:** WebView2 suele venir preinstalado

## Estructura del repo

| Ruta | Rol |
|------|-----|
| `src/` | Frontend React (UI del ticker) |
| `src-tauri/` | Backend Rust (Tauri, fetch RSS) |

## Scripts

| Comando | Uso |
|---------|-----|
| `npm install` | Instala dependencias de Node |
| `npm run tauri dev` | Desarrollo: Vite + ventana Tauri con recarga |
| `npm run dev` | Solo Vite en `http://localhost:1420` (sin shell nativa) |
| `npm run tauri build` | Compila el instalador / binario de la app |

## Compilación de producción

```bash
npm run tauri build
```

Los artefactos quedan bajo `src-tauri/target/release/` (y salida de empaquetado según el target configurado en Tauri).

## IDE recomendado

- [Visual Studio Code](https://code.visualstudio.com/)
- Extensiones: [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode), [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Notas

- El contenido del feed es propiedad de la BBC; esta app solo lo consume como lector RSS público.
- Identificador de app Tauri: `com.publicidad.ticker` · nombre del producto: **Publicidad**.
