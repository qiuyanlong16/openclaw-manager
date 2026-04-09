# OpenClaw Manager

One-click desktop GUI for deploying and uninstalling [OpenClaw](https://github.com/openclaw/openclaw) on Ubuntu / Windows.

Built with [Tauri v2](https://v2.tauri.app/) + React 18 + Rust.

## Features

- **Environment detection** — automatically checks Node.js >= 22 and Git
- **One-click deploy** — installs OpenClaw via npm, starts gateway, no terminal windows
- **One-click uninstall** — stops gateway, removes npm package, cleans config directories
- **Gateway control** — real-time status monitoring with start/stop buttons (5s auto-refresh)
- **Live log viewer** — shows deployment progress in real-time
- **Cross-platform** — Windows (NSIS) and Linux (DEB / AppImage)

## Screenshots

```
┌──────────────────────────────────────┐
│  OpenClaw Manager              - □ × │
├──────────────────────────────────────┤
│  环境检测                            │
│  ┌────────────────────────────────┐  │
│  │  Node.js   ✓ v22.x            │  │
│  │  Git       ✓ v2.40.x          │  │
│  └────────────────────────────────┘  │
│  操作                                │
│  ┌──────────┐  ┌──────────┐         │
│  │ 一键部署  │  │ 一键卸载  │         │
│  └──────────┘  └──────────┘         │
│  网关状态                            │
│  ┌────────────────────────────────┐  │
│  │  ● 运行中  [ 启动 ]  [ 停止 ]  │  │
│  └────────────────────────────────┘  │
├──────────────────────────────────────┤
│  日志                                │
│  ✓ 检测到 Node.js 22.14.0           │
│  ✓ openclaw 安装完成                 │
│  ✓ 网关已启动                        │
└──────────────────────────────────────┘
```

## Downloads

| Platform | Format | File |
|----------|--------|------|
| Windows | NSIS Installer | `OpenClaw Manager_0.1.0_x64-setup.exe` |
| Ubuntu 22.04+ | DEB Package | `openclaw-manager_0.1.0_amd64.deb` |
| Ubuntu (portable) | AppImage | `openclaw-manager_0.1.0_amd64.AppImage` |

## Installation

### Ubuntu / Debian

```bash
# Install dependencies
sudo apt install -y libwebkit2gtk-4.1-0 libgtk-3-0

# Install
sudo dpkg -i openclaw-manager_0.1.0_amd64.deb

# Or run AppImage directly (no install needed)
chmod +x openclaw-manager_0.1.0_amd64.AppImage
./openclaw-manager_0.1.0_amd64.AppImage
```

### Windows

Double-click `OpenClaw Manager_0.1.0_x64-setup.exe` and follow the installer.

## Build from Source

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) stable
- **Ubuntu**: `sudo apt install build-essential libwebkit2gtk-4.1-dev libgtk-3-dev`
- **Windows**: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with C++ workload

### Steps

```bash
git clone https://github.com/qiuyanlong16/openclaw-manager.git
cd openclaw-manager
npm install
```

### Development

```bash
npm run tauri dev
```

### Production Build

```bash
# Current platform (auto-detects)
npm run build:all

# Ubuntu only
npm run build:linux

# Windows only
npm run build:windows
```

Build output: `src-tauri/target/release/bundle/`

## Project Structure

```
├── src/
│   ├── main.tsx                          # React entry
│   ├── App.tsx                           # Main layout + state
│   ├── App.css                           # Ubuntu-themed styles (#E95420)
│   ├── components/
│   │   ├── EnvironmentCheck.tsx          # Node.js / Git detection
│   │   ├── ActionButtons.tsx             # Deploy / Uninstall buttons
│   │   ├── GatewayStatus.tsx             # Gateway status + controls
│   │   └── LogViewer.tsx                 # Real-time log display
│   └── hooks/
│       └── useLogListener.ts             # Tauri event listener
├── src-tauri/
│   ├── src/main.rs                       # Rust backend (6 Tauri commands)
│   ├── Cargo.toml
│   └── tauri.conf.json                   # Build + bundle config
├── package.json
└── tsconfig.json
```

## Architecture

- **Rust backend** — runs system commands (hidden, no terminal popup), emits logs via Tauri events
- **React frontend** — subscribes to events, displays status, controls UI state
- All long-running commands (`npm install`, `openclaw gateway start/stop`) run async to avoid freezing the UI

## Roadmap

- [ ] Qwen model one-click authentication
- [ ] Automatic Node.js 22+ installation when missing
- [ ] Automatic Git installation when missing
- [ ] Auto-start on boot option
- [ ] Dark mode support

## License

MIT
