# OpenClaw Manager Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Tauri + React desktop app for Ubuntu that provides one-click deploy/uninstall of OpenClaw with gateway status control, no terminal windows.

**Architecture:** Tauri v2 shell with Rust backend handling system commands (hidden from user) and React frontend displaying environment checks, action buttons, gateway status, and real-time logs via event emission.

**Tech Stack:** Tauri v2, Rust, React 18, TypeScript, Vite

---

### Task 1: Scaffold Tauri + React Project

**Files:**
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `vite.config.ts`
- Create: `index.html`
- Create: `src/main.tsx`
- Create: `src/App.tsx`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/icons/` (placeholder icons)

- [ ] **Step 1: Create frontend package.json**

```json
{
  "name": "openclaw-manager",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "react": "^18",
    "react-dom": "^18"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "@types/react": "^18",
    "@types/react-dom": "^18",
    "@vitejs/plugin-react": "^4",
    "typescript": "^5",
    "vite": "^5"
  }
}
```

- [ ] **Step 2: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"]
}
```

- [ ] **Step 3: Create vite.config.ts**

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
```

- [ ] **Step 4: Create index.html**

```html
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>OpenClaw Manager</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

- [ ] **Step 5: Create src/main.tsx**

```typescript
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

- [ ] **Step 6: Create initial src/App.tsx (minimal placeholder)**

```typescript
function App() {
  return (
    <div style={{ padding: 20 }}>
      <h1>OpenClaw Manager</h1>
      <p>Loading...</p>
    </div>
  );
}

export default App;
```

- [ ] **Step 7: Create src-tauri/Cargo.toml**

```toml
[package]
name = "openclaw-manager"
version = "0.1.0"
description = "One-click OpenClaw deploy manager"
edition = "2021"

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

- [ ] **Step 8: Create src-tauri/tauri.conf.json**

```json
{
  "productName": "OpenClaw Manager",
  "version": "0.1.0",
  "identifier": "com.openclaw.manager",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [
      {
        "title": "OpenClaw Manager",
        "width": 720,
        "height": 560,
        "resizable": true,
        "center": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": ["deb"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.png"
    ]
  }
}
```

- [ ] **Step 9: Create src-tauri/build.rs**

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 10: Create src-tauri/src/main.rs (initial placeholder)**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 11: Install dependencies**

Run: `npm install`
Expected: All packages installed successfully

Run: `cd src-tauri && cargo check`
Expected: Rust compiles (may download crates on first run)

- [ ] **Step 12: Create placeholder icons directory**

```bash
mkdir -p src-tauri/icons
```

Create a minimal 32x32 PNG placeholder (any small PNG will work for scaffolding; real icons will be added later). For now, copy a simple colored square or use:

```bash
# Generate a simple 32x32 PNG placeholder using Python
python3 -c "
import struct, zlib
def create_png(w, h, r, g, b):
    def make_chunk(chunk_type, data):
        c = chunk_type + data
        return struct.pack('>I', len(data)) + c + struct.pack('>I', zlib.crc32(c) & 0xffffffff)
    header = b'\\x89PNG\\r\\n\\x1a\\n'
    ihdr = make_chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0))
    raw = b''
    for y in range(h):
        raw += b'\\x00' + bytes([r, g, b]) * w
    idat = make_chunk(b'IDAT', zlib.compress(raw))
    iend = make_chunk(b'IEND', b'')
    return header + ihdr + idat + iend
open('src-tauri/icons/32x32.png', 'wb').write(create_png(32, 32, 233, 84, 32))
open('src-tauri/icons/128x128.png', 'wb').write(create_png(128, 128, 233, 84, 32))
open('src-tauri/icons/icon.png', 'wb').write(create_png(256, 256, 233, 84, 32))
"
```

- [ ] **Step 13: Commit**

```bash
git add package.json tsconfig.json vite.config.ts index.html
git add src/
git add src-tauri/
git commit -m "feat: scaffold Tauri v2 + React project"
```

---

### Task 2: Rust Environment Check Command

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Add environment check command to main.rs**

Replace the entire `src-tauri/src/main.rs` with:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::process::Command;
use tauri::Manager;

#[derive(Serialize)]
struct EnvCheckResult {
    node: EnvStatus,
    git: EnvStatus,
}

#[derive(Serialize)]
struct EnvStatus {
    ok: bool,
    version: Option<String>,
}

fn check_version(cmd: &str, arg: &str) -> EnvStatus {
    match Command::new(cmd).arg(arg).output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            EnvStatus {
                ok: true,
                version: Some(version),
            }
        }
        _ => EnvStatus {
            ok: false,
            version: None,
        },
    }
}

#[tauri::command]
fn check_environment() -> EnvCheckResult {
    EnvCheckResult {
        node: check_version("node", "--version"),
        git: check_version("git", "--version"),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![check_environment])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Verify Rust compiles**

Run: `cd src-tauri && cargo check`
Expected: `Finished dev [unoptimized + debuginfo] target(s)`

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat: add environment check Rust command"
```

---

### Task 3: Environment Check React Component + App Layout

**Files:**
- Create: `src/components/EnvironmentCheck.tsx`
- Create: `src/App.css`
- Modify: `src/App.tsx`

- [ ] **Step 1: Create EnvironmentCheck component**

Create `src/components/EnvironmentCheck.tsx`:

```typescript
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface EnvStatus {
  ok: boolean;
  version: string | null;
}

interface EnvCheckData {
  node: EnvStatus;
  git: EnvStatus;
}

export default function EnvironmentCheck() {
  const [env, setEnv] = useState<EnvCheckData | null>(null);

  useEffect(() => {
    invoke<EnvCheckData>("check_environment").then(setEnv).catch(console.error);
  }, []);

  if (!env) return null;

  return (
    <div className="card">
      <h2>环境检测</h2>
      <div className="env-list">
        <div className="env-item">
          <span className={`status-dot ${env.node.ok ? "ok" : "error"}`} />
          <span>
            Node.js {env.node.ok ? `${env.node.version}` : "未安装 (需要 >= 22)"}
          </span>
        </div>
        <div className="env-item">
          <span className={`status-dot ${env.git.ok ? "ok" : "error"}`} />
          <span>
            Git {env.git.ok ? `${env.git.version}` : "未安装"}
          </span>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Create App.css with Ubuntu-themed styles**

Create `src/App.css`:

```css
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: "Ubuntu", "Segoe UI", system-ui, -apple-system, sans-serif;
  background: #f5f5f5;
  color: #333;
}

.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  padding: 16px;
  gap: 16px;
}

.app-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.app-header h1 {
  font-size: 1.25rem;
  font-weight: 600;
  color: #333;
}

.app-header .logo {
  width: 24px;
  height: 24px;
  background: #e95420;
  border-radius: 4px;
}

.content {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card {
  background: #fff;
  border-radius: 8px;
  padding: 16px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

.card h2 {
  font-size: 0.9rem;
  font-weight: 600;
  color: #666;
  margin-bottom: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.env-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.env-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.95rem;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot.ok {
  background: #4caf50;
}

.status-dot.error {
  background: #f44336;
}

.action-buttons {
  display: flex;
  gap: 12px;
}

.btn {
  flex: 1;
  padding: 12px 20px;
  border: none;
  border-radius: 6px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s, opacity 0.2s;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: #e95420;
  color: #fff;
}

.btn-primary:hover:not(:disabled) {
  background: #c7431b;
}

.btn-danger {
  background: #f44336;
  color: #fff;
}

.btn-danger:hover:not(:disabled) {
  background: #d32f2f;
}

.btn-small {
  padding: 6px 16px;
  font-size: 0.85rem;
  flex: none;
}

.gateway-status {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.gateway-info {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.95rem;
}

.gateway-actions {
  display: flex;
  gap: 8px;
}

.log-viewer {
  background: #2d2d2d;
  color: #e0e0e0;
  border-radius: 6px;
  padding: 12px;
  font-family: "Ubuntu Mono", "Consolas", monospace;
  font-size: 0.8rem;
  max-height: 160px;
  overflow-y: auto;
  min-height: 80px;
}

.log-viewer h2 {
  color: #999;
  margin-bottom: 8px;
}

.log-entry {
  padding: 2px 0;
  line-height: 1.4;
}

.log-entry.info {
  color: #8bc34a;
}

.log-entry.error {
  color: #f44336;
}

.log-entry.warn {
  color: #ff9800;
}
```

- [ ] **Step 3: Update App.tsx to use EnvironmentCheck**

Replace `src/App.tsx`:

```typescript
import EnvironmentCheck from "./components/EnvironmentCheck";
import "./App.css";

function App() {
  return (
    <div className="app">
      <header className="app-header">
        <div className="logo" />
        <h1>OpenClaw Manager</h1>
      </header>
      <div className="content">
        <EnvironmentCheck />
      </div>
    </div>
  );
}

export default App;
```

- [ ] **Step 4: Verify dev mode**

Run: `npm run tauri dev`
Expected: Window opens showing "环境检测" section with Node.js and Git status dots

- [ ] **Step 5: Commit**

```bash
git add src/components/EnvironmentCheck.tsx src/App.css src/App.tsx
git commit -m "feat: add environment check UI component with Ubuntu theme"
```

---

### Task 4: ActionButtons + LogViewer Components

**Files:**
- Create: `src/components/ActionButtons.tsx`
- Create: `src/components/LogViewer.tsx`
- Create: `src/hooks/useLogListener.ts`
- Modify: `src/App.tsx`

- [ ] **Step 1: Create log listener hook**

Create `src/hooks/useLogListener.ts`:

```typescript
import { useEffect, useRef, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";

export interface LogEntry {
  level: "info" | "warn" | "error";
  message: string;
  timestamp: string;
}

export function useLogListener() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [isDeploying, setIsDeploying] = useState(false);
  const [isUninstalling, setIsUninstalling] = useState(false);

  useEffect(() => {
    const unlisten = listen<{
      level: string;
      message: string;
    }>("deploy-log", (event) => {
      const entry: LogEntry = {
        level: event.payload.level as "info" | "warn" | "error",
        message: event.payload.message,
        timestamp: new Date().toLocaleTimeString(),
      };
      setLogs((prev) => [...prev, entry]);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const clearLogs = useCallback(() => setLogs([]), []);

  return { logs, clearLogs, isDeploying, setIsDeploying, isUninstalling, setIsUninstalling };
}
```

- [ ] **Step 2: Create LogViewer component**

Create `src/components/LogViewer.tsx`:

```typescript
import { useRef, useEffect } from "react";
import { LogEntry } from "../hooks/useLogListener";

interface LogViewerProps {
  logs: LogEntry[];
}

export default function LogViewer({ logs }: LogViewerProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [logs]);

  return (
    <div className="card log-card">
      <h2>日志</h2>
      <div className="log-viewer" ref={containerRef}>
        {logs.length === 0 ? (
          <div className="log-entry info">等待操作...</div>
        ) : (
          logs.map((entry, i) => (
            <div key={i} className={`log-entry ${entry.level}`}>
              [{entry.timestamp}] {entry.message}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Create ActionButtons component (UI only, commands wired in Task 9)**

Create `src/components/ActionButtons.tsx`:

```typescript
import { invoke } from "@tauri-apps/api/core";

interface ActionButtonsProps {
  isDeploying: boolean;
  isUninstalling: boolean;
  onDeployStart: () => void;
  onDeployEnd: (success: boolean) => void;
  onUninstallStart: () => void;
  onUninstallEnd: (success: boolean) => void;
}

export default function ActionButtons({
  isDeploying,
  isUninstalling,
  onDeployStart,
  onDeployEnd,
  onUninstallStart,
  onUninstallEnd,
}: ActionButtonsProps) {
  async function handleDeploy() {
    onDeployStart();
    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        "deploy_openclaw"
      );
      onDeployEnd(result.success);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error("Deploy failed:", msg);
      onDeployEnd(false);
    }
  }

  async function handleUninstall() {
    onUninstallStart();
    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        "uninstall_openclaw"
      );
      onUninstallEnd(result.success);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error("Uninstall failed:", msg);
      onUninstallEnd(false);
    }
  }

  return (
    <div className="card">
      <h2>操作</h2>
      <div className="action-buttons">
        <button
          className="btn btn-primary"
          onClick={handleDeploy}
          disabled={isDeploying || isUninstalling}
        >
          {isDeploying ? "部署中..." : "一键部署"}
        </button>
        <button
          className="btn btn-danger"
          onClick={handleUninstall}
          disabled={isDeploying || isUninstalling}
        >
          {isUninstalling ? "卸载中..." : "一键卸载"}
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Update App.tsx to include all components**

Replace `src/App.tsx`:

```typescript
import EnvironmentCheck from "./components/EnvironmentCheck";
import ActionButtons from "./components/ActionButtons";
import GatewayStatus from "./components/GatewayStatus";
import LogViewer from "./components/LogViewer";
import { useLogListener } from "./hooks/useLogListener";
import "./App.css";

function App() {
  const { logs, isDeploying, setIsDeploying, isUninstalling, setIsUninstalling } =
    useLogListener();

  return (
    <div className="app">
      <header className="app-header">
        <div className="logo" />
        <h1>OpenClaw Manager</h1>
      </header>
      <div className="content">
        <EnvironmentCheck />
        <ActionButtons
          isDeploying={isDeploying}
          isUninstalling={isUninstalling}
          onDeployStart={() => setIsDeploying(true)}
          onDeployEnd={(success) => {
            setIsDeploying(false);
          }}
          onUninstallStart={() => setIsUninstalling(true)}
          onUninstallEnd={(success) => {
            setIsUninstalling(false);
          }}
        />
        <GatewayStatus />
        <LogViewer logs={logs} />
      </div>
    </div>
  );
}

export default App;
```

- [ ] **Step 5: Verify frontend compiles**

Run: `npx tsc --noEmit`
Expected: No errors (GatewayStatus import will error since it doesn't exist yet — this is expected, we create it in Task 5)

- [ ] **Step 6: Commit**

```bash
git add src/hooks/useLogListener.ts src/components/LogViewer.tsx src/components/ActionButtons.tsx src/App.tsx
git commit -m "feat: add action buttons, log viewer, and log listener hook"
```

---

### Task 5: GatewayStatus Component

**Files:**
- Create: `src/components/GatewayStatus.tsx`

- [ ] **Step 1: Create GatewayStatus component (commands wired in Task 10)**

Create `src/components/GatewayStatus.tsx`:

```typescript
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function GatewayStatus() {
  const [running, setRunning] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    fetchStatus();
  }, []);

  async function fetchStatus() {
    try {
      const result = await invoke<{ running: boolean }>("get_gateway_status");
      setRunning(result.running);
    } catch {
      setRunning(false);
    }
  }

  async function handleStart() {
    setLoading(true);
    try {
      await invoke("start_gateway");
      setRunning(true);
    } catch (e) {
      console.error("Failed to start gateway:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleStop() {
    setLoading(true);
    try {
      await invoke("stop_gateway");
      setRunning(false);
    } catch (e) {
      console.error("Failed to stop gateway:", e);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="card">
      <h2>网关状态</h2>
      <div className="gateway-status">
        <div className="gateway-info">
          <span
            className={`status-dot ${
              running === null ? "" : running ? "ok" : "error"
            }`}
          />
          <span>
            {running === null
              ? "检测中..."
              : running
                ? "运行中"
                : "已停止"}
          </span>
        </div>
        <div className="gateway-actions">
          <button
            className="btn btn-small btn-primary"
            onClick={handleStart}
            disabled={loading || running === true}
          >
            启动
          </button>
          <button
            className="btn btn-small btn-danger"
            onClick={handleStop}
            disabled={loading || running !== true}
          >
            停止
          </button>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Add gateway CSS class to App.css**

Add to `src/App.css` after the existing `.gateway-actions` block:

```css
.gateway-status {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.gateway-info {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.95rem;
}
```

(Note: these may already exist from Task 3's App.css — if so, skip this step)

- [ ] **Step 3: Commit**

```bash
git add src/components/GatewayStatus.tsx
git commit -m "feat: add gateway status component with start/stop controls"
```

---

### Task 6: Rust Deploy Command

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Add deploy_openclaw command**

Add the deploy command and log emission to `src-tauri/src/main.rs`. Replace the file with:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::process::Command;
use tauri::{Emitter, Manager};

#[derive(Serialize, Clone)]
struct EnvCheckResult {
    node: EnvStatus,
    git: EnvStatus,
}

#[derive(Serialize, Clone)]
struct EnvStatus {
    ok: bool,
    version: Option<String>,
}

#[derive(Serialize, Clone)]
struct DeployResult {
    success: bool,
    error: Option<String>,
}

#[derive(Serialize, Clone)]
struct LogMessage {
    level: String,
    message: String,
}

fn check_version(cmd: &str, arg: &str) -> EnvStatus {
    match Command::new(cmd).arg(arg).output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            EnvStatus {
                ok: true,
                version: Some(version),
            }
        }
        _ => EnvStatus {
            ok: false,
            version: None,
        },
    }
}

fn emit_log(app: &tauri::AppHandle, level: &str, message: &str) {
    let _ = app.emit(
        "deploy-log",
        LogMessage {
            level: level.to_string(),
            message: message.to_string(),
        },
    );
}

fn run_command(app: &tauri::AppHandle, cmd: &str, args: &[&str], label: &str) -> Result<String, String> {
    emit_log(app, "info", &format!("正在执行: {}", label));
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("{} 失败: {}", label, e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            emit_log(app, "info", stdout.trim());
        }
        Ok(stdout.trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = format!("{} 失败: {}", label, stderr.trim());
        emit_log(app, "error", &msg);
        Err(msg)
    }
}

#[tauri::command]
fn check_environment() -> EnvCheckResult {
    EnvCheckResult {
        node: check_version("node", "--version"),
        git: check_version("git", "--version"),
    }
}

#[tauri::command]
fn deploy_openclaw(app: tauri::AppHandle) -> Result<DeployResult, String> {
    emit_log(&app, "info", "=== 开始部署 OpenClaw ===");

    // Check Node.js
    emit_log(&app, "info", "检查 Node.js 环境...");
    let node = check_version("node", "--version");
    if node.ok {
        let version = node.version.as_deref().unwrap_or("");
        let major = version.trim_start_matches('v').split('.').next().unwrap_or("0");
        let major: u32 = major.parse().unwrap_or(0);
        if major < 22 {
            emit_log(&app, "error", &format!("Node.js 版本 {} 过低，需要 >= 22", version));
            return Ok(DeployResult {
                success: false,
                error: Some(format!("Node.js {} 过低，需要 >= 22", version)),
            });
        }
        emit_log(&app, "info", &format!("Node.js {} 已安装", version));
    } else {
        emit_log(&app, "error", "未检测到 Node.js，请先安装 Node.js >= 22");
        return Ok(DeployResult {
            success: false,
            error: Some("未检测到 Node.js".to_string()),
        });
    }

    // Check Git
    emit_log(&app, "info", "检查 Git 环境...");
    let git = check_version("git", "--version");
    if git.ok {
        emit_log(&app, "info", &format!("Git {} 已安装", git.version.as_deref().unwrap_or("")));
    } else {
        emit_log(&app, "error", "未检测到 Git，请先安装 Git");
        return Ok(DeployResult {
            success: false,
            error: Some("未检测到 Git".to_string()),
        });
    }

    // Install OpenClaw via npm
    emit_log(&app, "info", "正在安装 OpenClaw...");
    run_command(&app, "npm", &["install", "-g", "openclaw@latest"], "npm install")?;
    emit_log(&app, "info", "OpenClaw 安装完成");

    // Start gateway
    emit_log(&app, "info", "正在启动 OpenClaw 网关...");
    run_command(&app, "openclaw", &["gateway", "start"], "启动网关")?;
    emit_log(&app, "info", "网关已启动");

    emit_log(&app, "info", "=== OpenClaw 部署完成 ===");
    Ok(DeployResult {
        success: true,
        error: None,
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            check_environment,
            deploy_openclaw,
        ])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Verify Rust compiles**

Run: `cd src-tauri && cargo check`
Expected: `Finished dev [unoptimized + debuginfo] target(s)`

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat: add deploy_openclaw command with env check and npm install"
```

---

### Task 7: Rust Uninstall Command

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Add uninstall_openclaw command**

Add to the invoke_handler in `main.rs`:

```rust
#[tauri::command]
fn uninstall_openclaw(app: tauri::AppHandle) -> Result<DeployResult, String> {
    emit_log(&app, "info", "=== 开始卸载 OpenClaw ===");

    // Stop gateway
    emit_log(&app, "info", "正在停止网关...");
    let _ = run_command(&app, "openclaw", &["gateway", "stop"], "停止网关");

    // Uninstall npm package
    emit_log(&app, "info", "正在卸载 OpenClaw npm 包...");
    run_command(&app, "npm", &["uninstall", "-g", "openclaw"], "npm uninstall")?;
    emit_log(&app, "info", "OpenClaw npm 包已卸载");

    // Clean config directory
    emit_log(&app, "info", "正在清理配置目录...");
    let home = std::env::var("HOME").map_err(|_| "无法获取 HOME 目录".to_string())?;
    let config_dir = format!("{}/.openclaw", home);
    let workspace_dir = format!("{}/openclaw", home);

    if std::path::Path::new(&config_dir).exists() {
        std::fs::remove_dir_all(&config_dir)
            .map_err(|e| format!("清理配置目录失败: {}", e))?;
        emit_log(&app, "info", &format!("已清理 {}", config_dir));
    }

    if std::path::Path::new(&workspace_dir).exists() {
        std::fs::remove_dir_all(&workspace_dir)
            .map_err(|e| format!("清理工作目录失败: {}", e))?;
        emit_log(&app, "info", &format!("已清理 {}", workspace_dir));
    }

    emit_log(&app, "info", "=== OpenClaw 卸载完成 ===");
    Ok(DeployResult {
        success: true,
        error: None,
    })
}
```

Add `uninstall_openclaw` to the `generate_handler!` macro:

```rust
.invoke_handler(tauri::generate_handler![
    check_environment,
    deploy_openclaw,
    uninstall_openclaw,
])
```

- [ ] **Step 2: Verify Rust compiles**

Run: `cd src-tauri && cargo check`
Expected: `Finished dev [unoptimized + debuginfo] target(s)`

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat: add uninstall_openclaw command with config cleanup"
```

---

### Task 8: Rust Gateway Commands

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Add gateway status/start/stop commands**

Add to `main.rs` before the `main()` function:

```rust
#[derive(Serialize, Clone)]
struct GatewayStatusResult {
    running: bool,
}

#[tauri::command]
fn get_gateway_status() -> GatewayStatusResult {
    let output = Command::new("openclaw")
        .args(["gateway", "status"])
        .output();

    match output {
        Ok(output) => GatewayStatusResult {
            running: output.status.success(),
        },
        Err(_) => GatewayStatusResult { running: false },
    }
}

#[tauri::command]
fn start_gateway(app: tauri::AppHandle) -> Result<DeployResult, String> {
    emit_log(&app, "info", "正在启动网关...");
    run_command(&app, "openclaw", &["gateway", "start"], "启动网关")?;
    emit_log(&app, "info", "网关已启动");
    Ok(DeployResult {
        success: true,
        error: None,
    })
}

#[tauri::command]
fn stop_gateway(app: tauri::AppHandle) -> Result<DeployResult, String> {
    emit_log(&app, "info", "正在停止网关...");
    run_command(&app, "openclaw", &["gateway", "stop"], "停止网关")?;
    emit_log(&app, "info", "网关已停止");
    Ok(DeployResult {
        success: true,
        error: None,
    })
}
```

Add all three to the `generate_handler!` macro:

```rust
.invoke_handler(tauri::generate_handler![
    check_environment,
    deploy_openclaw,
    uninstall_openclaw,
    get_gateway_status,
    start_gateway,
    stop_gateway,
])
```

- [ ] **Step 2: Verify Rust compiles**

Run: `cd src-tauri && cargo check`
Expected: `Finished dev [unoptimized + debuginfo] target(s)`

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat: add gateway status/start/stop commands"
```

---

### Task 9: Wire Deploy/Uninstall to Frontend with Real Commands

**Files:**
- Modify: `src/components/ActionButtons.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: Update ActionButtons to show success/error feedback**

Replace `src/components/ActionButtons.tsx`:

```typescript
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ActionButtonsProps {
  isDeploying: boolean;
  isUninstalling: boolean;
  onDeployStart: () => void;
  onDeployEnd: (success: boolean) => void;
  onUninstallStart: () => void;
  onUninstallEnd: (success: boolean) => void;
}

export default function ActionButtons({
  isDeploying,
  isUninstalling,
  onDeployStart,
  onDeployEnd,
  onUninstallStart,
  onUninstallEnd,
}: ActionButtonsProps) {
  const [lastResult, setLastResult] = useState<{
    type: "deploy" | "uninstall";
    success: boolean;
  } | null>(null);

  async function handleDeploy() {
    setLastResult(null);
    onDeployStart();
    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        "deploy_openclaw"
      );
      setLastResult({ type: "deploy", success: result.success });
      onDeployEnd(result.success);
    } catch {
      setLastResult({ type: "deploy", success: false });
      onDeployEnd(false);
    }
  }

  async function handleUninstall() {
    setLastResult(null);
    onUninstallStart();
    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        "uninstall_openclaw"
      );
      setLastResult({ type: "uninstall", success: result.success });
      onUninstallEnd(result.success);
    } catch {
      setLastResult({ type: "uninstall", success: false });
      onUninstallEnd(false);
    }
  }

  return (
    <div className="card">
      <h2>操作</h2>
      <div className="action-buttons">
        <button
          className="btn btn-primary"
          onClick={handleDeploy}
          disabled={isDeploying || isUninstalling}
        >
          {isDeploying ? "部署中..." : "一键部署"}
        </button>
        <button
          className="btn btn-danger"
          onClick={handleUninstall}
          disabled={isDeploying || isUninstalling}
        >
          {isUninstalling ? "卸载中..." : "一键卸载"}
        </button>
      </div>
      {lastResult && (
        <p
          style={{
            marginTop: 8,
            color: lastResult.success ? "#4caf50" : "#f44336",
            fontSize: "0.85rem",
          }}
        >
          {lastResult.type === "deploy"
            ? lastResult.success
              ? "部署成功！"
              : "部署失败，请查看日志"
            : lastResult.success
              ? "卸载完成！"
              : "卸载失败，请查看日志"}
        </p>
      )}
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/ActionButtons.tsx src/App.tsx
git commit -m "feat: wire deploy/uninstall to frontend with success/error feedback"
```

---

### Task 10: Wire Gateway to Frontend + Auto-refresh

**Files:**
- Modify: `src/components/GatewayStatus.tsx`

- [ ] **Step 1: Add auto-refresh to GatewayStatus**

Replace `src/components/GatewayStatus.tsx`:

```typescript
import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function GatewayStatus() {
  const [running, setRunning] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchStatus = useCallback(async () => {
    try {
      const result = await invoke<{ running: boolean }>("get_gateway_status");
      setRunning(result.running);
    } catch {
      setRunning(false);
    }
  }, []);

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 5000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  async function handleStart() {
    setLoading(true);
    try {
      await invoke("start_gateway");
      setRunning(true);
    } catch (e) {
      console.error("Failed to start gateway:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleStop() {
    setLoading(true);
    try {
      await invoke("stop_gateway");
      setRunning(false);
    } catch (e) {
      console.error("Failed to stop gateway:", e);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="card">
      <h2>网关状态</h2>
      <div className="gateway-status">
        <div className="gateway-info">
          <span
            className={`status-dot ${
              running === null ? "" : running ? "ok" : "error"
            }`}
          />
          <span>
            {running === null
              ? "检测中..."
              : running
                ? "运行中"
                : "已停止"}
          </span>
        </div>
        <div className="gateway-actions">
          <button
            className="btn btn-small btn-primary"
            onClick={handleStart}
            disabled={loading || running === true}
          >
            启动
          </button>
          <button
            className="btn btn-small btn-danger"
            onClick={handleStop}
            disabled={loading || running !== true}
          >
            停止
          </button>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/GatewayStatus.tsx
git commit -m "feat: add gateway auto-refresh every 5s and wire to Rust commands"
```

---

### Task 11: Packaging Configuration + Build

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Update tauri.conf.json with Linux bundle config**

Replace `src-tauri/tauri.conf.json` bundle section:

```json
{
  "productName": "OpenClaw Manager",
  "version": "0.1.0",
  "identifier": "com.openclaw.manager",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [
      {
        "title": "OpenClaw Manager",
        "width": 720,
        "height": 560,
        "resizable": true,
        "center": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": ["deb", "appimage"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.png"
    ],
    "linux": {
      "deb": {
        "depends": [
          "libwebkit2gtk-4.1-0",
          "libgtk-3-0"
        ],
        "section": "utility",
        "priority": "optional"
      }
    }
  }
}
```

- [ ] **Step 2: Add appindicator feature to Cargo.toml if needed for system tray**

For now, keep Cargo.toml as-is (no system tray in Phase 1). If the deb build requires it, add to `src-tauri/Cargo.toml`:

```toml
[dependencies]
tauri = { version = "2", features = [] }
```

No changes needed for Phase 1.

- [ ] **Step 3: Build on Ubuntu**

> **Note:** The `npm run tauri build` command must run on Ubuntu Linux (not Windows). Build commands:

```bash
# On Ubuntu:
npm install
npm run tauri build
```

Expected output: `.deb` file in `src-tauri/target/release/bundle/deb/`

- [ ] **Step 4: Verify .deb package**

```bash
# Install the package
sudo dpkg -i src-tauri/target/release/bundle/deb/openclaw-manager_0.1.0_amd64.deb

# Verify it's installed
dpkg -l | grep openclaw

# Launch from terminal
openclaw-manager

# Uninstall for cleanup
sudo dpkg --purge openclaw-manager
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "feat: add deb/appimage packaging config for Ubuntu"
```

---

## Self-Review

### 1. Spec Coverage

| Spec Requirement | Task |
|-----------------|------|
| Environment check (Node.js + Git) | Task 2 (Rust), Task 3 (UI) |
| One-click deploy button | Task 4 (UI), Task 6 (Rust), Task 9 (wire) |
| One-click uninstall button | Task 4 (UI), Task 7 (Rust), Task 9 (wire) |
| Gateway status display | Task 5 (UI), Task 8 (Rust), Task 10 (wire) |
| Gateway start/stop | Task 8 (Rust), Task 10 (wire) |
| Real-time log display | Task 4 (hook + LogViewer) |
| No terminal windows | All commands use `Command::output()` with piped stdout |
| Qwen auth deferred | Noted in spec, not implemented |
| .deb packaging | Task 11 |
| Ubuntu theme (#E95420) | Task 3 (App.css) |
| Window size 720x560 | Task 1 (tauri.conf.json) |

### 2. Placeholder Scan
- No TBD/TODO in any step
- All code blocks are complete
- No "similar to Task N" references
- All type signatures match across tasks (`DeployResult`, `LogMessage`, `EnvCheckResult` are consistent)

### 3. Type Consistency
- `DeployResult { success: bool, error: Option<String> }` — used in Tasks 6, 7, 8, 9, 10 consistently
- `LogMessage { level: String, message: String }` — used in Tasks 4 (hook) and 6-8 (Rust emit) consistently
- `GatewayStatusResult { running: bool }` — used in Tasks 8 (Rust) and 10 (React) consistently
- Event name `"deploy-log"` — consistent across Rust emit (Tasks 6-8) and React listen (Task 4)

### 4. Important Notes

- **Development environment:** This project can be scaffolded on Windows, but `npm run tauri dev` and `npm run tauri build` for Linux targets require running on Ubuntu Linux. The user should transfer the code to Ubuntu for testing.
- **Dependencies on Ubuntu:** Before running `tauri dev` on Ubuntu, ensure `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `build-essential` are installed.
- **OpenClaw must be available** on the target machine for the gateway commands to work. The deploy command installs it via npm first.
