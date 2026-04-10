#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use serde::Deserialize;
use std::process::Command;
use std::sync::Mutex;
use tauri::Emitter;
use tokio::process::Command as TokioCommand;

// Track the gateway child process so we can stop it
static GATEWAY_CHILD: Mutex<Option<std::process::Child>> = Mutex::new(None);

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

    // First stop any existing gateway child
    if let Ok(mut guard) = GATEWAY_CHILD.lock() {
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
        }
    }

    // Also try killing any existing openclaw gateway process on the port
    let _ = run_command_async(app, &bin, &["gateway", "stop"], "清理旧网关").await;

    // Spawn `openclaw gateway run` as a background process
    let mut child = std::process::Command::new(&bin)
        .args(["gateway", "run"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("启动网关失败: {}", e))?;

    // Give it a moment to start up
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Verify it's still running
    if child.try_wait().map_or(true, |s| s.is_some()) {
        return Err("网关启动后立即退出".to_string());
    }

    // Store the child process handle
    if let Ok(mut guard) = GATEWAY_CHILD.lock() {
        *guard = Some(child);
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
    // First check if our tracked child process is still running
    if let Ok(mut guard) = GATEWAY_CHILD.lock() {
        if let Some(child) = guard.as_mut() {
            if child.try_wait().map_or(false, |s| s.is_none()) {
                return GatewayStatusResult { running: true };
            } else {
                guard.take(); // child exited, clear it
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

    // Kill the tracked child process
    if let Ok(mut guard) = GATEWAY_CHILD.lock() {
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            emit_log(&app, "info", "已终止网关进程");
        }
    }

    // Also try the CLI stop command for cleanup
    let _ = run_openclaw_cmd(&app, &["gateway", "stop"], "清理网关").await;

    emit_log(&app, "info", "网关已停止");
    Ok(DeployResult {
        success: true,
        error: None,
    })
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
        ])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
