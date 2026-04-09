#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::process::Command;
use tauri::Emitter;
use tokio::process::Command as TokioCommand;

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

async fn run_command_async(app: &tauri::AppHandle, cmd: &str, args: &[&str], label: &str) -> Result<String, String> {
    emit_log(app, "info", &format!("正在执行: {}", label));
    let mut command = TokioCommand::new(cmd);
    command.args(args);
    let output = command
        .output()
        .await
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

#[tauri::command(async)]
async fn deploy_openclaw(app: tauri::AppHandle) -> Result<DeployResult, String> {
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
    run_command_async(&app, "npm", &["install", "-g", "openclaw@latest"], "npm install").await?;
    emit_log(&app, "info", "OpenClaw 安装完成");

    // Start gateway
    emit_log(&app, "info", "正在启动 OpenClaw 网关...");
    run_command_async(&app, "openclaw", &["gateway", "start"], "启动网关").await?;
    emit_log(&app, "info", "网关已启动");

    emit_log(&app, "info", "=== OpenClaw 部署完成 ===");
    Ok(DeployResult {
        success: true,
        error: None,
    })
}

#[tauri::command(async)]
async fn uninstall_openclaw(app: tauri::AppHandle) -> Result<DeployResult, String> {
    emit_log(&app, "info", "=== 开始卸载 OpenClaw ===");

    // Stop gateway
    emit_log(&app, "info", "正在停止网关...");
    let _ = run_command_async(&app, "openclaw", &["gateway", "stop"], "停止网关").await;

    // Uninstall npm package
    emit_log(&app, "info", "正在卸载 OpenClaw npm 包...");
    run_command_async(&app, "npm", &["uninstall", "-g", "openclaw"], "npm uninstall").await?;
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

#[tauri::command(async)]
async fn start_gateway(app: tauri::AppHandle) -> Result<DeployResult, String> {
    emit_log(&app, "info", "正在启动网关...");
    run_command_async(&app, "openclaw", &["gateway", "start"], "启动网关").await?;
    emit_log(&app, "info", "网关已启动");
    Ok(DeployResult {
        success: true,
        error: None,
    })
}

#[tauri::command(async)]
async fn stop_gateway(app: tauri::AppHandle) -> Result<DeployResult, String> {
    emit_log(&app, "info", "正在停止网关...");
    run_command_async(&app, "openclaw", &["gateway", "stop"], "停止网关").await?;
    emit_log(&app, "info", "网关已停止");
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
            uninstall_openclaw,
            get_gateway_status,
            start_gateway,
            stop_gateway,
        ])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
