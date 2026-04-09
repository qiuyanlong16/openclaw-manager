# OpenClaw Manager Design Spec

**Date:** 2026-04-09
**Topic:** Ubuntu 桌面应用 — 一键部署/卸载 OpenClaw GUI
**技术栈:** Tauri (Rust 后端) + React (前端)

---

## 1. 概述

面向零命令行 Ubuntu 用户的桌面应用，提供图形化的一键部署和卸载 OpenClaw 服务。部署全程无终端弹窗，所有操作通过 GUI 完成。

**当前阶段:** 仅实现 UI 框架 + 基础交互逻辑。Qwen 认证流程暂不实现。

---

## 2. 架构

### 2.1 前端 (React)

三个主要 UI 区域：

| 区域 | 功能 |
|------|------|
| 环境检测区 | 启动时自动检测 Node.js >= 22 和 Git，显示绿色/红色状态 |
| 操作区 | "一键部署" / "一键卸载" 按钮 |
| 网关状态区 | 显示 Gateway 运行状态（运行中/已停止），支持启停控制 |
| 日志区 | 底部只读实时日志，展示部署/卸载进度 |

### 2.2 后端 (Rust / Tauri Commands)

| Command | 输入 | 输出 |
|---------|------|------|
| `check_environment` | 无 | `{ node: { ok, version }, git: { ok, version } }` |
| `deploy_openclaw` | 无 | 事件流（日志），最终返回 `{ success, error }` |
| `uninstall_openclaw` | 无 | 事件流（日志），最终返回 `{ success, error }` |
| `get_gateway_status` | 无 | `{ running: bool }` |
| `start_gateway` | 无 | `{ success, error }` |
| `stop_gateway` | 无 | `{ success, error }` |

### 2.3 进程管理

所有子进程通过 `std::process::Command` 启动，stdout/stderr 重定向，Linux 下不弹出终端窗口。通过 `app.emit()` 将实时日志推送给前端。

---

## 3. 部署流程 (后端)

```
check_environment()
  ├── Node.js >= 22? 如否 → 下载安装 Node.js 22 LTS (tarball)
  ├── Git 存在? 如否 → 下载官方绿色版解压到 /opt/git
  └── npm install -g openclaw@latest
      └── openclaw gateway start (隐藏窗口)
```

## 4. 卸载流程 (后端)

```
stop_gateway()
  ├── npm uninstall -g openclaw
  ├── 清理 ~/.openclaw 配置目录
  └── 清理 ~/openclaw/workspace 工作目录
```

---

## 5. 项目结构

```
openclaw-manager/
├── src-tauri/
│   ├── src/
│   │   └── main.rs          # Tauri 入口 + commands
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── main.tsx             # React 入口
│   ├── App.tsx              # 主组件
│   ├── components/
│   │   ├── EnvironmentCheck.tsx   # 环境检测区
│   │   ├── ActionButtons.tsx      # 部署/卸载按钮
│   │   ├── GatewayStatus.tsx      # 网关状态
│   │   └── LogViewer.tsx          # 日志区
│   └── hooks/
│       └── useLogListener.ts      # 日志事件监听
├── package.json
└── vite.config.ts
```

---

## 6. UI 设计

简洁风格，Ubuntu 原生色调（#E95420 橙色 + 白色/灰色）。

```
┌──────────────────────────────────────┐
│  OpenClaw Manager              - □ × │
├──────────────────────────────────────┤
│                                      │
│  环境检测                            │
│  ┌────────────────────────────────┐  │
│  │  Node.js   ✓ v22.x            │  │
│  │  Git       ✓ v2.40.x          │  │
│  └────────────────────────────────┘  │
│                                      │
│  操作                                │
│  ┌──────────┐  ┌──────────┐         │
│  │ 一键部署  │  │ 一键卸载  │         │
│  └──────────┘  └──────────┘         │
│                                      │
│  网关状态                            │
│  ┌────────────────────────────────┐  │
│  │  ● 运行中                      │  │
│  │  [ 启动 ]  [ 停止 ]            │  │
│  └────────────────────────────────┘  │
│                                      │
├──────────────────────────────────────┤
│  日志                                │
│  ✓ 检测到 Node.js 22.14.0           │
│  ✓ 检测到 Git 2.43.0                │
│  ✓ openclaw 安装完成                 │
│  ✓ 网关已启动                        │
└──────────────────────────────────────┘
```

---

## 7. 当前阶段范围 (Phase 1)

- [x] Tauri + React 项目脚手架
- [x] 环境检测 UI（显示 Node.js 和 Git 状态）
- [x] 一键部署/卸载按钮 UI
- [x] 网关状态展示 UI
- [x] 日志实时显示 UI
- [ ] Qwen 认证流程（后续阶段）

**Qwen 认证暂不实现** — 预留位置，后续用户指定方案后再加入。

---

## 8. 错误处理

- 部署失败：日志区显示红色错误信息，按钮恢复可点击状态
- 卸载失败：同上，防止部分卸载导致状态不一致
- 网络异常：下载安装 Node.js 时检测网络，提示用户

---

## 9. 打包与发布

### 9.1 目标格式

- **`.deb` 包** — Ubuntu 原生格式，支持 `dpkg -i` 安装
- **AppImage**（可选）— 免安装运行，方便测试验证

### 9.2 Tauri 打包配置 (`tauri.conf.json`)

```json
{
  "bundle": {
    "active": true,
    "targets": ["deb", "appimage"],
    "identifier": "com.openclaw.manager",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.ico", "icons/icon.png"],
    "linux": {
      "deb": {
        "depends": ["libwebkit2gtk-4.1", "libgtk-3-0"],
        "section": "utility",
        "priority": "optional",
        "dependsData": []
      }
    }
  }
}
```

**注意：** 不将 `nodejs`、`git`、`openclaw` 声明为 deb 依赖，因为本应用的核心价值就是自动管理这些依赖的安装。

### 9.3 构建命令

```bash
# 本地开发
npm run tauri dev

# 构建 deb 包（产出在 src-tauri/target/release/bundle/deb/）
npm run tauri build

# 构建 AppImage（产出在 src-tauri/target/release/bundle/appimage/）
npm run tauri build -- --target x86_64-unknown-linux-gnu --bundles appimage
```

### 9.4 构建环境要求

- Ubuntu 22.04+ 或兼容的 Debian 系统
- Rust 1.70+（推荐通过 rustup 安装）
- Node.js 18+（开发依赖，非运行时依赖）
- `build-essential`, `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libappindicator3-dev`
- `patchelf`, `desktop-file-utils`（AppImage 需要）

### 9.5 打包验证清单

每次打包后需验证：

- [ ] `dpkg -i` 安装成功，无依赖冲突
- [ ] 应用能从系统菜单/桌面启动
- [ ] 应用图标正确显示
- [ ] 卸载后无残留（`dpkg --purge` 清理干净）
- [ ] AppImage 直接 `chmod +x && ./OpenClawManager.AppImage` 可运行

---

## 10. 待确认事项

- Qwen 认证交互流程（用户后续提供）
- 是否需要安装/卸载状态持久化（重启后恢复状态）
- 是否需要开机自启选项
