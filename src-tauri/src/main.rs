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
