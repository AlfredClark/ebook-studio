# tauri-desktop-template

English · [简体中文](docs/README_zh-CN.md)

A Tauri 2 based desktop application template with a SvelteKit 5 + TypeScript frontend
and a Rust backend. It integrates common desktop capabilities — system tray, global
shortcuts, auto-start, single instance, auto-update, dialogs, file system, system
information, clipboard, notifications, window state and more — ready to use as the
starting point for a new desktop application.

## Screenshots

|             Home              |               Settings                |              About              |
| :---------------------------: | :-----------------------------------: | :-----------------------------: |
| ![Home](docs/images/home.png) | ![Settings](docs/images/settings.png) | ![About](docs/images/about.png) |

## Tech Stack

- **Frontend**: SvelteKit 5 / Svelte 5 / TypeScript / Vite / Tailwind CSS v4 / shadcn-svelte (bun)
- **Backend**: Tauri 2 / Rust (edition 2024)
- **Internationalization**: Paraglide (frontend) / rust-i18n (backend)
- **Quality**: ESLint / Stylelint / Prettier / Clippy / rustfmt / Husky + lint-staged

## Plugins

- **System**: tray-icon, autostart, global-shortcut, single-instance, window-state, menu (macOS only)
- **Desktop**: updater, notification, dialog, fs, os, clipboard-manager, opener, process, system-fonts, log, store

## Development Guide

Full development conventions — architecture layers, code style, validation and Git workflow — are documented in [AGENTS.md](AGENTS.md).

## Getting Started

```bash
bun install
bun run i18n:compile  # paraglide artifacts are not committed; compile after install
bun run tauri:dev     # development
bun run tauri:build   # production build
```

## Template Initialization

Before shipping your own app, rename the template identity (identifier / productName /
updater pubkey & endpoints) and remove the demo module — see [AGENTS.md](AGENTS.md).

## License

[GPL-3.0-only](LICENSE)
