# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**OpenClaw Manager** is a one-click desktop GUI application for deploying and uninstalling [OpenClaw](https://github.com/openclaw/openclaw) on Ubuntu and Windows. Built with **Tauri v2 + React 18 + Rust**.

All UI text is in Chinese (zh-CN). The app theme follows Ubuntu branding (accent color `#E95420`).

## Quick Start

### Prerequisites

- Node.js 18+
- Rust stable (`rustup`)
- **Ubuntu**: `sudo apt install build-essential libwebkit2gtk-4.1-dev libgtk-3-dev`
- **Windows**: Visual Studio Build Tools with C++ workload

### Common Commands

```bash
npm install              # Install JS dependencies
npm run tauri dev        # Full Tauri dev mode (Rust + React hot reload)
npm run build            # TypeScript check + Vite build (produces dist/)
npm run build:linux      # Tauri build -> DEB bundle
npm run build:windows    # Tauri build -> NSIS installer
npm run build:all        # Tauri build for current platform
```

Build output: `src-tauri/target/release/bundle/`

There are **no lint or test scripts** configured. TypeScript strict mode in `tsconfig.json` provides compile-time checking (`noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`).

### Dev Server

- Vite runs on port **1420** (hardcoded in `vite.config.ts`)
- Tauri dev mode auto-starts Vite via `beforeDevCommand`

## Architecture

### Rust Backend (`src-tauri/src/main.rs`, ~1460 lines)

All 15 Tauri commands are in a single file:

| Command | Purpose |
|---|---|
| `check_environment` | Checks Node.js >= 22, Git, OpenClaw versions |
| `is_openclaw_installed` | Boolean check for openclaw npm package |
| `deploy_openclaw` | npm install + onboard + gateway start |
| `uninstall_openclaw` | Stop gateway + npm uninstall + config cleanup |
| `get_gateway_status` | PID + port check + CLI fallback |
| `start_gateway` / `stop_gateway` | Gateway lifecycle control |
| `open_url` | Opens browser with gateway token |
| `get_model_config` / `set_model_config` | Read/write LLM provider config |
| `list_skills` / `set_skill_enabled` | Skills management |
| `install_weixin_plugin` | WeChat plugin install |
| `get_weixin_config` / `start_weixin_qr_login` / `poll_weixin_qr_status` / `save_weixin_login_result` / `disconnect_weixin` | WeChat QR login flow |

Key patterns:
- Long-running commands use `tokio::process::Command` (async) to avoid freezing UI
- Gateway is spawned in its own process group (`setsid()` on Unix), tracked via `Mutex<Option<u32>>` for clean `killpg()`
- Log events pushed to frontend via `app.emit("deploy-log", ...)`
- WeChat iLink API calls use `reqwest::blocking` (HTTPS)

### React Frontend (`src/`)

| File | Purpose |
|---|---|
| `App.tsx` | Main layout: header, 4 cards, settings overlay |
| `components/EnvironmentCheck.tsx` | Node.js / Git / OpenClaw detection card |
| `components/ActionButtons.tsx` | Deploy / Uninstall buttons with loading states |
| `components/GatewayStatus.tsx` | Gateway status card with start/stop/open browser |
| `components/LogViewer.tsx` | Real-time log display (auto-scroll) |
| `components/Settings.tsx` | Full-screen settings panel (Model/Skills/Channels tabs) |
| `components/Skills.tsx` | Skills list with search/filter, enable/disable toggles |
| `components/Channels.tsx` | WeChat (Weixin) channel management card |
| `components/WeixinQrModal.tsx` | QR code modal for WeChat login (`qrcode.react`) |
| `hooks/useLogListener.ts` | Subscribes to `"deploy-log"` Tauri events |

### Communication

- Frontend -> Backend: `invoke()` from `@tauri-apps/api/core`
- Backend -> Frontend: `listen()` on `"deploy-log"` events from `@tauri-apps/api/event`

## Key Configuration

- `tsconfig.json`: ES2020 target, strict mode, `bundler` module resolution, `noEmit` (Vite handles compilation)
- `vite.config.ts`: Port 1420, watches all except `src-tauri/**`
- `src-tauri/tauri.conf.json`: Window 720x560, bundles DEB/NSIS/AppImage, `withGlobalTauri: false`
- `src-tauri/capabilities/default.json`: Permissions for event listening and scoped URL opener (restricted to `http://127.0.0.1:18789/` and `http://localhost:18789/`)
- Gateway default port: **18789** (referenced in URL opener scope)
