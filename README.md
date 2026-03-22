# TG Docs

Native desktop app for [docs.tomgreen.uk](https://docs.tomgreen.uk) built with [Tauri v2](https://v2.tauri.app).

Wraps the TG Docs site in a lightweight native window — no Electron, no bundled Chromium. Uses the OS webview (WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux).

## How it works

The app loads a single `index.html` that redirects to `https://docs.tomgreen.uk`. All content is served from the site — this is a thin native shell.

## Project structure

```
src/              HTML entrypoint (redirects to docs.tomgreen.uk)
src-tauri/
  tauri.conf.json Tauri config (window size, app ID, icons, bundling)
  icons/          App icons (all sizes + .icns/.ico)
  src/            Rust backend (minimal — just the Tauri bootstrap)
.github/workflows/build.yml  CI: builds for Linux, macOS, Windows
```

## Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js 22+](https://nodejs.org/)
- Platform-specific Tauri dependencies — see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

## Development

```bash
npm install
npm run dev
```

## Build

```bash
npm run build
```

Produces platform-native installers in `src-tauri/target/release/bundle/`:
- **macOS**: `.dmg`
- **Windows**: `.msi` / `.exe`
- **Linux**: `.deb` / `.AppImage`

## CI

GitHub Actions builds for all three platforms on push to `main`. Download artifacts from the **Actions** tab.

| Identifier | `uk.tomgreen.docs` |
|---|---|
| **Version** | `0.1.0` |
| **Window** | 1280x800 (min 800x600) |
