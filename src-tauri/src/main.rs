#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use serde::Deserialize;
use std::process::Command;
use std::sync::Mutex;
use tauri::Emitter;
use tokio::process::Command as TokioCommand;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

// Track the gateway child process so we can kill the entire process group
static GATEWAY_PID: Mutex<Option<u32>> = Mutex::new(None);

// --- Types ---

#[derive(Serialize, Clone)]
struct EnvCheckResult {
    node: EnvStatus,
    git: EnvStatus,
    openclaw: EnvStatus,
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

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(non_snake_case)]
struct SkillInfo {
    name: String,
    emoji: String,
    description: String,
    eligible: bool,
    disabled: bool,
    source: String,
    missingBins: Vec<String>,
}

#[derive(Serialize, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelConfigResult {
    provider: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
    custom_base_url: Option<String>,
}

// --- Weixin Types ---

#[derive(Serialize, Clone)]
struct WeixinConfigResult {
    plugin_installed: bool,
    enabled: bool,
    connected: bool,
    account_id: Option<String>,
}

#[derive(Serialize, Clone)]
struct WeixinQrStartResult {
    qrcode: String,
    qrcode_img_content: String,
    message: String,
}

#[derive(Serialize, Clone, Deserialize)]
struct WeixinQrPollResult {
    status: String,
    bot_token: Option<String>,
    account_id: Option<String>,
    base_url: Option<String>,
    user_id: Option<String>,
}

const WEIXIN_PLUGIN_ID: &str = "openclaw-weixin";
const WEIXIN_CHANNEL_ID: &str = "openclaw-weixin";
const I_LINK_BASE_URL: &str = "https://ilinkai.weixin.qq.com";
const I_LINK_BOT_TYPE: &str = "3";

// --- Helpers ---

fn emit_log(app: &tauri::AppHandle, level: &str, message: &str) {
    let _ = app.emit(
        "deploy-log",
        LogMessage {
            level: level.to_string(),
            message: message.to_string(),
        },
    );
}

fn get_home_dir() -> Result<String, String> {
    if let Some(home) = dirs::home_dir() {
        return Ok(home.to_string_lossy().to_string());
    }
    if let Ok(home) = std::env::var("HOME") {
        return Ok(home);
    }
    Err("无法获取 HOME 目录".to_string())
}

fn get_openclaw_bin_path() -> Result<String, String> {
    let home = get_home_dir()?;
    Ok(format!("{}/.npm-global/bin/openclaw", home))
}

fn get_config_path() -> Result<String, String> {
    let home = get_home_dir()?;
    Ok(format!("{}/.openclaw/openclaw.json", home))
}

fn get_npm_prefix() -> Result<String, String> {
    let home = get_home_dir()?;
    Ok(format!("{}/.npm-global", home))
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

fn check_openclaw() -> EnvStatus {
    let bin = match get_openclaw_bin_path() {
        Ok(b) => b,
        Err(_) => return EnvStatus { ok: false, version: None },
    };
    match Command::new(&bin).arg("--version").output() {
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

fn setup_shell_path(app: &tauri::AppHandle) {
    let npm_config = "export NPM_CONFIG_PREFIX=\"$HOME/.npm-global\"";
    let path_export = "export PATH=\"$HOME/.npm-global/bin:$PATH\"";
    let shells = [".bashrc", ".zshrc"];
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };

    for file in &shells {
        let path = format!("{}/{}", home, file);
        if std::path::Path::new(&path).exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            if !content.contains(".npm-global") {
                let mut f = std::fs::OpenOptions::new().append(true).open(&path);
                if let Ok(ref mut f) = f {
                    use std::io::Write;
                    let _ = writeln!(f, "\n# OpenClaw");
                    let _ = writeln!(f, "{}", npm_config);
                    let _ = writeln!(f, "{}", path_export);
                    emit_log(app, "info", &format!("已添加 PATH 到 {}", file));
                }
            }
        }
    }
}

fn cleanup_shell_path(app: &tauri::AppHandle) {
    let shells = [".bashrc", ".zshrc"];
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };

    for file in &shells {
        let path = format!("{}/{}", home, file);
        if std::path::Path::new(&path).exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            let lines: Vec<&str> = content.lines().collect();
            let mut skip = false;
            let mut cleaned: Vec<String> = Vec::new();
            for line in lines {
                if line == "# OpenClaw" {
                    skip = true;
                    continue;
                } else if line.contains(".npm-global") {
                    skip = false;
                    continue;
                }
                if skip {
                    if line.is_empty() {
                        skip = false;
                        continue;
                    }
                    skip = false;
                }
                cleaned.push(line.to_string());
            }
            let _ = std::fs::write(&path, format!("{}\n", cleaned.join("\n")));
            emit_log(app, "info", &format!("已清理 {} 中的 PATH 配置", file));
        }
    }
}

async fn run_command_with_env(
    app: &tauri::AppHandle,
    cmd: &str,
    args: &[&str],
    label: &str,
    extra_env: Option<(&str, &str)>,
) -> Result<String, String> {
    emit_log(app, "info", &format!("正在执行: {}", label));
    let mut command = TokioCommand::new(cmd);
    command.args(args);
    if let Some((key, value)) = extra_env {
        command.env(key, value);
    }
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

async fn run_openclaw_cmd(
    app: &tauri::AppHandle,
    args: &[&str],
    label: &str,
) -> Result<String, String> {
    let bin = get_openclaw_bin_path()?;
    run_command_async(app, &bin, args, label).await
}

async fn run_command_async(
    app: &tauri::AppHandle,
    cmd: &str,
    args: &[&str],
    label: &str,
) -> Result<String, String> {
    run_command_with_env(app, cmd, args, label, None).await
}

async fn spawn_gateway(app: &tauri::AppHandle) -> Result<(), String> {
    let bin = get_openclaw_bin_path()?;
    emit_log(app, "info", "启动 gateway run...");

    // First kill any existing gateway process group
    kill_gateway_group();

    // Also try killing any existing openclaw gateway process on the port
    let _ = run_command_async(app, &bin, &["gateway", "stop"], "清理旧网关").await;

    // Spawn `openclaw gateway run` in its own process group
    let mut cmd = std::process::Command::new(&bin);
    cmd.args(["gateway", "run"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    #[cfg(unix)]
    unsafe {
        cmd.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("启动网关失败: {}", e))?;

    let pid = child.id();

    // Give it a moment to start up
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Verify it's still running
    if child.try_wait().map_or(true, |s| s.is_some()) {
        return Err("网关启动后立即退出".to_string());
    }

    // Store the PID for process group killing
    if let Ok(mut guard) = GATEWAY_PID.lock() {
        *guard = Some(pid);
    }

    Ok(())
}

// Map provider to openclaw onboard auth choice flag
fn auth_choice_for_provider(provider: &str) -> &'static str {
    match provider {
        "anthropic" => "anthropic-cli",
        "openai" => "openai-api-key",
        "google" => "gemini-api-key",
        "kimi" => "moonshot-api-key",
        "deepseek" => "deepseek-api-key",
        _ => "custom-api-key",
    }
}

fn api_key_arg_for_provider(provider: &str) -> &'static str {
    match provider {
        "anthropic" => "--anthropic-api-key",
        "openai" => "--openai-api-key",
        "google" => "--gemini-api-key",
        "kimi" => "--moonshot-api-key",
        "deepseek" => "--deepseek-api-key",
        _ => "--custom-api-key",
    }
}

fn get_default_onboard_args() -> Option<Vec<String>> {
    // If user has saved a model config, use it
    if let Ok(cfg) = get_model_config_inner() {
        if let (Some(provider), Some(api_key), Some(model)) =
            (&cfg.provider, &cfg.api_key, &cfg.model)
        {
            if !api_key.is_empty() && !model.is_empty() {
                let mut args: Vec<String> = vec![
                    "onboard".into(),
                    "--non-interactive".into(),
                    "--auth-choice".into(),
                    auth_choice_for_provider(provider).into(),
                    "--accept-risk".into(),
                    "--skip-channels".into(),
                    "--skip-skills".into(),
                    "--skip-search".into(),
                    "--skip-health".into(),
                ];
                args.push(api_key_arg_for_provider(provider).into());
                args.push(api_key.clone());
                args.push("--custom-model-id".into());
                args.push(model.clone());

                if provider == "custom" {
                    if let Some(base_url) = &cfg.custom_base_url {
                        if !base_url.is_empty() {
                            args.push("--custom-base-url".into());
                            args.push(base_url.clone());
                            args.push("--custom-compatibility".into());
                            args.push("openai".into());
                        }
                    }
                }
                return Some(args);
            }
        }
    }

    // No model configured: use skip auth to create base config + gateway only
    Some(vec![
        "onboard".into(),
        "--non-interactive".into(),
        "--auth-choice".into(),
        "skip".into(),
        "--accept-risk".into(),
        "--skip-channels".into(),
        "--skip-skills".into(),
        "--skip-search".into(),
        "--skip-health".into(),
    ])
}

// --- Tauri Commands ---

#[tauri::command]
fn check_environment() -> EnvCheckResult {
    EnvCheckResult {
        node: check_version("node", "--version"),
        git: check_version("git", "--version"),
        openclaw: check_openclaw(),
    }
}

#[tauri::command]
fn is_openclaw_installed() -> bool {
    check_openclaw().ok
}

#[tauri::command(async)]
async fn deploy_openclaw(app: tauri::AppHandle) -> Result<DeployResult, String> {
    emit_log(&app, "info", "=== 开始部署 OpenClaw ===");

    // 1. Check Node.js
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

    // 2. Check Git
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

    // 3. Setup npm prefix
    setup_shell_path(&app);
    let npm_prefix = get_npm_prefix()?;
    std::fs::create_dir_all(&npm_prefix)
        .map_err(|e| format!("创建安装目录失败: {}", e))?;

    // 4. Install openclaw (skip if already installed)
    let openclaw_bin = get_openclaw_bin_path()?;
    if std::path::Path::new(&openclaw_bin).exists() {
        emit_log(&app, "info", "OpenClaw 已安装，跳过重复安装");
    } else {
        // Set Huawei npm mirror
        emit_log(&app, "info", "配置 npm 镜像源（华为云）...");
        run_command_with_env(
            &app,
            "npm",
            &["config", "set", "registry", "https://mirrors.huaweicloud.com/repository/npm/"],
            "npm config set registry",
            Some(("NPM_CONFIG_PREFIX", &npm_prefix)),
        ).await?;

        emit_log(&app, "info", "正在安装 OpenClaw...");
        run_command_with_env(
            &app,
            "npm",
            &["install", "-g", "--prefix", &npm_prefix, "openclaw@latest"],
            "npm install -g",
            Some(("NPM_CONFIG_PREFIX", &npm_prefix)),
        ).await?;
        emit_log(&app, "info", "OpenClaw 安装完成");
    }

    let openclaw_bin = get_openclaw_bin_path()?;

    // 5. Always run onboard: creates .openclaw/openclaw.json with gateway + base config
    //    If model is configured in Settings, includes API key; otherwise skips auth
    emit_log(&app, "info", "正在初始化 OpenClaw 配置...");

    // Ensure .openclaw directory exists
    let config_dir = format!("{}/.openclaw", get_home_dir()?);
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("创建配置目录失败: {}", e))?;

    let onboard_args = get_default_onboard_args().unwrap();

    let mut cmd = TokioCommand::new(&openclaw_bin);
    cmd.args(&onboard_args);
    let output = cmd.output().await.map_err(|e| format!("初始化配置失败: {}", e))?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        emit_log(&app, "info", "OpenClaw 配置初始化完成");
    } else {
        let err_msg = stderr.trim();
        emit_log(&app, "warn", &format!("初始化警告: {}", err_msg));
        emit_log(&app, "info", "继续执行...");
    }

    // 7. Start gateway (spawn as background process)
    emit_log(&app, "info", "正在启动 OpenClaw 网关...");
    spawn_gateway(&app).await?;
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

    let npm_prefix = get_npm_prefix()?;

    // Stop gateway
    emit_log(&app, "info", "正在停止网关...");
    let _ = run_openclaw_cmd(&app, &["gateway", "stop"], "停止网关").await;

    // Uninstall
    emit_log(&app, "info", "正在卸载 OpenClaw...");
    run_command_with_env(
        &app,
        "npm",
        &["uninstall", "-g", "--prefix", &npm_prefix, "openclaw"],
        "npm uninstall -g",
        Some(("NPM_CONFIG_PREFIX", &npm_prefix)),
    ).await?;
    emit_log(&app, "info", "OpenClaw 已卸载");

    // Delete config folder
    let config_dir = format!("{}/.openclaw", get_home_dir()?);
    if std::path::Path::new(&config_dir).exists() {
        let _ = std::fs::remove_dir_all(&config_dir);
        emit_log(&app, "info", "已删除 ~/.openclaw 配置目录");
    }

    cleanup_shell_path(&app);

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
    // First check if our tracked gateway process group is still running
    if let Ok(guard) = GATEWAY_PID.lock() {
        if let Some(pid) = *guard {
            #[cfg(unix)]
            {
                // Check if the process group leader is alive
                let alive = unsafe { libc::kill(pid as i32, 0) == 0 };
                if alive {
                    return GatewayStatusResult { running: true };
                } else {
                    // Process is gone, clear the PID
                    drop(guard);
                    if let Ok(mut g) = GATEWAY_PID.lock() {
                        *g = None;
                    }
                }
            }
            #[cfg(not(unix))]
            {
                let _ = pid;
            }
        }
    }

    // Fallback: check if something is listening on the gateway port
    if std::net::TcpStream::connect_timeout(
        &"127.0.0.1:18789".parse().unwrap(),
        std::time::Duration::from_secs(1),
    )
    .is_ok()
    {
        return GatewayStatusResult { running: true };
    }

    // Fallback: try openclaw gateway status command
    let openclaw_bin = match get_openclaw_bin_path() {
        Ok(p) => p,
        Err(_) => return GatewayStatusResult { running: false },
    };
    let output = Command::new(&openclaw_bin)
        .args(["gateway", "status"])
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}{}", stdout, stderr);
            let running = !combined.contains("stopped") && !combined.contains("failed");
            GatewayStatusResult { running }
        }
        Err(_) => GatewayStatusResult { running: false },
    }
}

#[tauri::command(async)]
async fn start_gateway(app: tauri::AppHandle) -> Result<DeployResult, String> {
    emit_log(&app, "info", "正在启动网关...");
    spawn_gateway(&app).await?;
    emit_log(&app, "info", "网关已启动");
    Ok(DeployResult {
        success: true,
        error: None,
    })
}

#[tauri::command(async)]
async fn stop_gateway(app: tauri::AppHandle) -> Result<DeployResult, String> {
    emit_log(&app, "info", "正在停止网关...");

    kill_gateway_group();

    // Also try the CLI stop command for cleanup
    let _ = run_openclaw_cmd(&app, &["gateway", "stop"], "清理网关").await;

    emit_log(&app, "info", "网关已停止");
    Ok(DeployResult {
        success: true,
        error: None,
    })
}

/// Kill the entire gateway process group (parent + all children)
fn kill_gateway_group() {
    if let Ok(guard) = GATEWAY_PID.lock() {
        if let Some(pid) = *guard {
            #[cfg(unix)]
            {
                // Send SIGTERM to the process group (negative PID = process group)
                unsafe {
                    libc::killpg(pid as i32, libc::SIGTERM);
                }
                // Brief wait for graceful shutdown
                std::thread::sleep(std::time::Duration::from_millis(500));
                // Force kill if still alive
                unsafe {
                    libc::killpg(pid as i32, libc::SIGKILL);
                }
            }
            #[cfg(not(unix))]
            {
                // Fallback for non-Unix: try to kill via PID
                if let Ok(mut g) = GATEWAY_PID.lock() {
                    if let Some(p) = *g {
                        let _ = std::process::Command::new("taskkill")
                            .args(["/PID", &p.to_string(), "/F", "/T"])
                            .output();
                    }
                    *g = None;
                }
            }
        }
    }
    // Clear the stored PID
    if let Ok(mut g) = GATEWAY_PID.lock() {
        *g = None;
    }
}

fn get_gateway_token() -> Option<String> {
    let config_path = get_config_path().ok()?;
    if !std::path::Path::new(&config_path).exists() {
        return None;
    }
    let content = std::fs::read_to_string(&config_path).ok()?;
    let config: serde_json::Value = serde_json::from_str(&content).ok()?;
    config
        .get("gateway")
        .and_then(|g| g.get("auth"))
        .and_then(|a| a.get("token"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    // Append gateway token as query param if available
    let final_url = if let Some(token) = get_gateway_token() {
        if url.contains('?') {
            format!("{}&token={}", url, token)
        } else {
            format!("{}?token={}", url, token)
        }
    } else {
        url
    };
    open::that(&final_url).map_err(|e| format!("无法打开: {}", e))
}

// --- Skills Management ---

fn get_openclaw_bin() -> Result<String, String> {
    get_openclaw_bin_path()
}

fn list_skills_inner() -> Result<Vec<SkillInfo>, String> {
    let bin = get_openclaw_bin()?;
    let output = Command::new(&bin)
        .args(["skills", "list", "--json"])
        .output()
        .map_err(|e| format!("获取技能列表失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("解析技能列表失败: {}", e))?;

    let skills = json.get("skills").and_then(|s| s.as_array()).cloned().unwrap_or_default();
    let mut result = Vec::new();
    for skill in &skills {
        let name = skill.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let emoji = skill.get("emoji").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let description = skill.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let eligible = skill.get("eligible").and_then(|v| v.as_bool()).unwrap_or(false);
        let disabled = skill.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false);
        let source = skill.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let missing_bins = skill.get("missing")
            .and_then(|m| m.get("bins"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_default();
        result.push(SkillInfo { name, emoji, description, eligible, disabled, source, missingBins: missing_bins });
    }
    Ok(result)
}

#[tauri::command]
fn list_skills() -> Vec<SkillInfo> {
    list_skills_inner().unwrap_or_default()
}

fn get_blocked_skills() -> Result<Vec<String>, String> {
    let bin = get_openclaw_bin()?;
    let output = Command::new(&bin)
        .args(["config", "get", "skills.blockBundled"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() || val == "null" {
                return Ok(Vec::new());
            }
            // Parse JSON array string like ["skill1","skill2"]
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(&val) {
                return Ok(arr);
            }
            Ok(Vec::new())
        }
        _ => Ok(Vec::new()),
    }
}

#[tauri::command]
fn set_skill_enabled(name: String, enabled: bool) -> Result<(), String> {
    let bin = get_openclaw_bin()?;

    let mut blocked = get_blocked_skills()?;
    if enabled {
        blocked.retain(|s| s != &name);
    } else {
        if !blocked.contains(&name) {
            blocked.push(name);
        }
    }

    let blocked_json = serde_json::to_string(&blocked).map_err(|e| format!("序列化失败: {}", e))?;
    let output = Command::new(&bin)
        .args(["config", "set", "skills.blockBundled", &blocked_json, "--strict-json"])
        .output()
        .map_err(|e| format!("设置技能状态失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("设置技能状态失败: {}", stderr.trim()));
    }

    Ok(())
}

// --- Model Configuration ---

fn get_model_config_inner() -> Result<ModelConfigResult, String> {
    let config_path = get_config_path()?;

    if !std::path::Path::new(&config_path).exists() {
        return Ok(ModelConfigResult {
            provider: None, api_key: None, model: None, custom_base_url: None,
        });
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置失败: {}", e))?;

    let config: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析配置失败: {}", e))?;

    // Read from models.providers.<name> - find the first provider with an apiKey
    let providers = config.get("models").and_then(|m| m.get("providers"));
    if let Some(providers_obj) = providers.and_then(|p| p.as_object()) {
        for (name, provider_val) in providers_obj {
            if let Some(api_key) = provider_val.get("apiKey").and_then(|v| v.as_str()) {
                if !api_key.is_empty() {
                    let base_url = provider_val.get("baseUrl").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let model = provider_val.get("models")
                        .and_then(|m| m.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|m| m.get("id"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let api = provider_val.get("api").and_then(|v| v.as_str()).unwrap_or("");
                    let frontend_provider = match (name.as_str(), api, base_url.as_deref()) {
                        ("anthropic", _, _) => "anthropic",
                        ("openai", _, _) => "openai",
                        ("google", _, _) => "google",
                        ("moonshot", _, _) => "kimi",
                        ("custom", _, _) => "custom",
                        _ => {
                            match base_url.as_deref() {
                                Some("https://api.anthropic.com") => "anthropic",
                                Some("https://api.openai.com/v1") => "openai",
                                Some("https://generativelanguage.googleapis.com") => "google",
                                Some("https://api.moonshot.cn/v1") => "kimi",
                                _ => "custom",
                            }
                        }
                    };
                    return Ok(ModelConfigResult {
                        provider: Some(frontend_provider.to_string()),
                        api_key: Some(api_key.to_string()),
                        model,
                        custom_base_url: base_url,
                    });
                }
            }
        }
    }

    Ok(ModelConfigResult {
        provider: None, api_key: None, model: None, custom_base_url: None,
    })
}

#[tauri::command]
fn get_model_config() -> ModelConfigResult {
    get_model_config_inner().unwrap_or(ModelConfigResult {
        provider: None, api_key: None, model: None, custom_base_url: None,
    })
}

#[tauri::command(async)]
async fn set_model_config(
    app: tauri::AppHandle,
    provider: String,
    api_key: String,
    model: String,
    custom_base_url: Option<String>,
) -> Result<(), String> {
    let openclaw_bin = get_openclaw_bin_path()?;

    let (auth_choice, key_arg): (&str, &str) = match provider.as_str() {
        "anthropic" => ("anthropic-cli", "--anthropic-api-key"),
        "openai" => ("openai-api-key", "--openai-api-key"),
        "google" => ("gemini-api-key", "--gemini-api-key"),
        "kimi" => ("moonshot-api-key", "--moonshot-api-key"),
        "deepseek" => ("deepseek-api-key", "--deepseek-api-key"),
        _ => ("custom-api-key", "--custom-api-key"),
    };

    let mut args: Vec<String> = vec![
        "onboard".into(),
        "--non-interactive".into(),
        "--auth-choice".into(),
        auth_choice.into(),
        key_arg.into(),
        api_key.clone(),
        "--custom-model-id".into(),
        model.clone(),
        "--accept-risk".into(),
        "--skip-channels".into(),
        "--skip-skills".into(),
        "--skip-search".into(),
        "--skip-health".into(),
    ];

    if provider == "custom" {
        if let Some(url) = &custom_base_url {
            if !url.is_empty() {
                args.push("--custom-base-url".into());
                args.push(url.clone());
                args.push("--custom-compatibility".into());
                args.push("openai".into());
            }
        }
    }

    emit_log(&app, "info", "正在更新模型配置...");

    let mut cmd = TokioCommand::new(&openclaw_bin);
    cmd.args(&args);
    let output = cmd.output().await.map_err(|e| format!("设置配置失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let err_msg = stderr.trim().lines().next().unwrap_or("").to_string();
        emit_log(&app, "error", &format!("配置更新失败: {}", err_msg));
        return Err(format!("配置更新失败: {}", err_msg));
    }

    emit_log(&app, "info", &format!("已保存配置: {} ({})", model, provider));

    // Restart gateway to apply new config
    emit_log(&app, "info", "正在重启网关以应用新配置...");
    let _ = TokioCommand::new(&openclaw_bin)
        .args(&["gateway", "stop"])
        .output()
        .await;

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    match TokioCommand::new(&openclaw_bin)
        .args(&["gateway", "start"])
        .output()
        .await
    {
        Ok(out) if out.status.success() => {
            emit_log(&app, "info", "网关已重启，新配置已生效");
        }
        _ => {
            emit_log(&app, "warn", "网关启动失败，请手动启动");
        }
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            check_environment,
            is_openclaw_installed,
            deploy_openclaw,
            uninstall_openclaw,
            get_gateway_status,
            start_gateway,
            stop_gateway,
            open_url,
            get_model_config,
            set_model_config,
            list_skills,
            set_skill_enabled,
            install_weixin_plugin,
            get_weixin_config,
            start_weixin_qr_login,
            poll_weixin_qr_status,
            save_weixin_login_result,
            disconnect_weixin,
        ])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// --- Weixin Helpers ---

fn is_weixin_plugin_installed() -> bool {
    let bin = match get_openclaw_bin_path() {
        Ok(b) => b,
        Err(_) => return false,
    };
    let output = Command::new(&bin)
        .args(["plugins", "list", "--json"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains(WEIXIN_PLUGIN_ID)
        }
        _ => false,
    }
}

fn is_weixin_channel_enabled() -> bool {
    let config_path = match get_config_path() {
        Ok(p) => p,
        Err(_) => return false,
    };
    if !std::path::Path::new(&config_path).exists() {
        return false;
    }
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let config: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return false,
    };

    // Check plugins.entries.openclaw-weixin.enabled
    let plugin_enabled = config
        .get("plugins")
        .and_then(|p| p.get("entries"))
        .and_then(|e| e.get(WEIXIN_PLUGIN_ID))
        .and_then(|e| e.get("enabled"))
        .and_then(|e| e.as_bool())
        .unwrap_or(false);

    let channel_enabled = config
        .get("channels")
        .and_then(|c| c.get(WEIXIN_CHANNEL_ID))
        .and_then(|c| c.get("enabled"))
        .and_then(|c| c.as_bool())
        .unwrap_or(false);

    plugin_enabled || channel_enabled
}

fn is_weixin_connected() -> Option<String> {
    let index_path = format!("{}/.openclaw-weixin/state/accounts.json", get_home_dir().ok()?);

    if !std::path::Path::new(&index_path).exists() {
        return None;
    }

    let content = std::fs::read_to_string(&index_path).ok()?;
    let ids: Vec<String> = serde_json::from_str(&content).ok()?;
    if ids.is_empty() {
        return None;
    }

    // Return the first account ID
    Some(ids[0].clone())
}

fn https_get(url: &str, headers: Option<&[(&str, &str)]>, timeout_secs: u64) -> Result<(u16, String), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let mut req = client.get(url);
    if let Some(hdrs) = headers {
        for (k, v) in hdrs {
            req = req.header(*k, *v);
        }
    }

    let resp = req.send().map_err(|e| format!("请求失败: {}", e))?;
    let status = resp.status().as_u16();
    let body = resp.text().map_err(|e| format!("读取响应失败: {}", e))?;

    Ok((status, body))
}

fn update_openclaw_config(updater: impl FnOnce(&mut serde_json::Value)) -> Result<(), String> {
    let config_path = get_config_path()?;

    // Ensure .openclaw directory exists
    let config_dir = format!("{}/.openclaw", get_home_dir()?);
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("创建配置目录失败: {}", e))?;

    let config: serde_json::Value = if std::path::Path::new(&config_path).exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("读取配置失败: {}", e))?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let mut config = config;
    updater(&mut config);

    let updated = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&config_path, updated)
        .map_err(|e| format!("写入配置失败: {}", e))?;

    Ok(())
}

fn normalize_account_id(value: &str) -> String {
    let trimmed = value.trim().to_lowercase();
    if trimmed.is_empty() {
        return "default".to_string();
    }
    let normalized: String = trimmed
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    let trimmed = normalized.trim_start_matches('-').trim_end_matches('-');
    if trimmed.is_empty() {
        "default".to_string()
    } else {
        trimmed.chars().take(64).collect()
    }
}

// --- Weixin Tauri Commands ---

fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            chars.next(); // skip '['
            while let Some(&cc) = chars.peek() {
                chars.next();
                if cc.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[tauri::command(async)]
async fn install_weixin_plugin(app: tauri::AppHandle) -> Result<DeployResult, String> {
    let app_clone = app.clone();
    emit_log(&app_clone, "info", "正在安装微信插件 @tencent-weixin/openclaw-weixin...");

    let bin = get_openclaw_bin_path()?;
    let result = tokio::task::spawn_blocking(move || {
        Command::new(&bin)
            .args(["plugins", "install", "@tencent-weixin/openclaw-weixin"])
            .output()
            .map_err(|e| format!("安装微信插件失败: {}", e))
    })
    .await
    .map_err(|e| format!("安装微信插件失败: {}", e))??;

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    // Check if the plugin was actually installed (exit 0 or plugin directory exists)
    let installed = result.status.success() || is_weixin_plugin_installed();

    if installed {
        emit_log(&app, "info", &format!("微信插件安装成功: {}", stdout.trim()));
        Ok(DeployResult {
            success: true,
            error: None,
        })
    } else {
        let err_msg = strip_ansi(stderr.trim().lines().next().unwrap_or("未知错误"));
        emit_log(&app, "error", &format!("安装失败: {}", err_msg));
        Err(format!("安装失败: {}", err_msg))
    }
}

#[tauri::command(async)]
async fn get_weixin_config() -> WeixinConfigResult {
    let plugin_installed = tokio::task::spawn_blocking(is_weixin_plugin_installed)
        .await
        .unwrap_or(false);
    let enabled = is_weixin_channel_enabled();
    let account_id = is_weixin_connected();
    let connected = account_id.is_some();

    WeixinConfigResult {
        plugin_installed,
        enabled,
        connected,
        account_id,
    }
}

#[tauri::command(async)]
async fn start_weixin_qr_login() -> Result<WeixinQrStartResult, String> {
    tokio::task::spawn_blocking(|| {
        let base = I_LINK_BASE_URL;
        let url = format!("{}/ilink/bot/get_bot_qrcode?bot_type={}", base, I_LINK_BOT_TYPE);

        let (status, body) = https_get(&url, None, 35)?;
        if status != 200 {
            return Err(format!("获取二维码失败: HTTP {}", status));
        }

        let data: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| format!("解析响应失败: {}", e))?;

        let qrcode = data.get("qrcode")
            .and_then(|v| v.as_str())
            .ok_or("未获取到 qrcode")?
            .to_string();

        let qrcode_img_content = data.get("qrcode_img_content")
            .and_then(|v| v.as_str())
            .ok_or("未获取到 qrcode_img_content")?
            .to_string();

        Ok(WeixinQrStartResult {
            qrcode,
            qrcode_img_content,
            message: "使用微信扫描以下二维码，以完成连接。".to_string(),
        })
    })
    .await
    .map_err(|e| format!("获取二维码失败: {}", e))?
}

#[tauri::command(async)]
async fn poll_weixin_qr_status(qrcode: String) -> Result<WeixinQrPollResult, String> {
    tokio::task::spawn_blocking(move || {
        let base = I_LINK_BASE_URL;
        let url = format!("{}/ilink/bot/get_qrcode_status?qrcode={}", base, qrcode);

        let headers = vec![("iLink-App-ClientVersion", "1")];
        let (status, body) = match https_get(&url, Some(&headers), 35) {
            Ok(v) => v,
            Err(_) => {
                return Ok(WeixinQrPollResult {
                    status: "wait".to_string(),
                    bot_token: None,
                    account_id: None,
                    base_url: None,
                    user_id: None,
                });
            }
        };

        if status != 200 {
            return Ok(WeixinQrPollResult {
                status: "wait".to_string(),
                bot_token: None,
                account_id: None,
                base_url: None,
                user_id: None,
            });
        }

        let data: serde_json::Value = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(_) => {
                return Ok(WeixinQrPollResult {
                    status: "wait".to_string(),
                    bot_token: None,
                    account_id: None,
                    base_url: None,
                    user_id: None,
                });
            }
        };

        Ok(WeixinQrPollResult {
            status: data.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("wait")
                .to_string(),
            bot_token: data.get("bot_token").and_then(|v| v.as_str()).map(|s| s.to_string()),
            account_id: data.get("ilink_bot_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
            base_url: data.get("baseurl").and_then(|v| v.as_str()).map(|s| s.to_string()),
            user_id: data.get("ilink_user_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
        })
    })
    .await
    .map_err(|e| format!("轮询失败: {}", e))?
}

#[tauri::command(async)]
async fn save_weixin_login_result(result: WeixinQrPollResult) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        do_save_weixin_login_result(result)
    })
    .await
    .map_err(|e| format!("保存凭据失败: {}", e))?
}

fn do_save_weixin_login_result(result: WeixinQrPollResult) -> Result<String, String> {
    let account_id = result.account_id
        .as_ref()
        .ok_or("缺少 accountId")?;
    let bot_token = result.bot_token
        .as_ref()
        .ok_or("缺少 botToken")?;

    let normalized_id = normalize_account_id(account_id);
    let home = get_home_dir()?;

    // Create state directories
    let accounts_dir = format!("{}/.openclaw-weixin/state/accounts", home);
    std::fs::create_dir_all(&accounts_dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    // Write account data
    let mut account_data = serde_json::json!({
        "token": bot_token,
        "savedAt": chrono_lite_timestamp()
    });
    if let Some(ref base_url) = result.base_url {
        account_data["baseUrl"] = serde_json::json!(base_url);
    }
    if let Some(ref user_id) = result.user_id {
        account_data["userId"] = serde_json::json!(user_id);
    }

    let account_path = format!("{}/{}.json", accounts_dir, normalized_id);
    std::fs::write(&account_path, serde_json::to_string_pretty(&account_data).unwrap())
        .map_err(|e| format!("写入账号数据失败: {}", e))?;

    // Try to set file permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&account_path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o600);
            let _ = std::fs::set_permissions(&account_path, perms);
        }
    }

    // Write account index
    let index_path = format!("{}/.openclaw-weixin/state/accounts.json", home);
    let existing_ids = if std::path::Path::new(&index_path).exists() {
        let content = std::fs::read_to_string(&index_path).unwrap_or_default();
        serde_json::from_str::<Vec<String>>(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    if !existing_ids.contains(&normalized_id) {
        let mut ids = existing_ids;
        ids.push(normalized_id.clone());
        std::fs::write(&index_path, serde_json::to_string_pretty(&ids).unwrap())
            .map_err(|e| format!("写入账号索引失败: {}", e))?;
    }

    // Update openclaw.json to enable the channel
    update_openclaw_config(|config| {
        // Enable plugin
        config["plugins"]["entries"][WEIXIN_PLUGIN_ID]["enabled"] = serde_json::json!(true);
        // Enable channel
        config["channels"][WEIXIN_CHANNEL_ID]["enabled"] = serde_json::json!(true);
    })?;

    Ok(normalized_id)
}

fn chrono_lite_timestamp() -> String {
    // Simple ISO 8601 timestamp without external crate dependency
    let secs_since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Format as ISO 8601 (approximate, good enough for savedAt field)
    let days = secs_since_epoch / 86400;
    let remaining = secs_since_epoch % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    // Days from epoch to year
    let mut year = 1970;
    let mut d = days;
    loop {
        let year_days = if is_leap_year(year) { 366 } else { 365 };
        if d < year_days { break; }
        d -= year_days;
        year += 1;
    }
    let month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 0;
    for (i, &md) in month_days.iter().enumerate() {
        let adjusted = if i == 1 && is_leap_year(year) { 29 } else { md };
        if d < adjusted { break; }
        d -= adjusted;
        month += 1;
    }
    let day = d + 1;

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month + 1, day, hours, minutes, seconds)
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[tauri::command(async)]
async fn disconnect_weixin(app: tauri::AppHandle) -> Result<DeployResult, String> {
    emit_log(&app, "info", "正在断开微信连接...");

    tokio::task::spawn_blocking(do_disconnect_weixin)
        .await
        .map_err(|e| format!("断开微信失败: {}", e))??;

    emit_log(&app, "info", "微信连接已断开");
    Ok(DeployResult {
        success: true,
        error: None,
    })
}

fn do_disconnect_weixin() -> Result<(), String> {
    let home = get_home_dir()?;
    let accounts_dir = format!("{}/.openclaw-weixin/state/accounts", home);
    let index_path = format!("{}/.openclaw-weixin/state/accounts.json", home);

    // Delete account data files
    if std::path::Path::new(&accounts_dir).exists() {
        if let Ok(entries) = std::fs::read_dir(&accounts_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }

    // Clear account index
    let _ = std::fs::write(&index_path, "[]");

    // Update openclaw.json to disable the channel
    update_openclaw_config(|config| {
        if let Some(plugin) = config.get_mut("plugins").and_then(|p| p.get_mut("entries")).and_then(|e| e.get_mut(WEIXIN_PLUGIN_ID)) {
            plugin["enabled"] = serde_json::json!(false);
        }
        if let Some(channel) = config.get_mut("channels").and_then(|c| c.get_mut(WEIXIN_CHANNEL_ID)) {
            channel["enabled"] = serde_json::json!(false);
        }
    })
}
