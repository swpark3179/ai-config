// ============================================================
// AI Ready — Tauri 백엔드 진입점
// ============================================================
mod definitions;
mod detect;
mod error;
mod logstream;
mod proc;
mod tasks;
mod win;

use error::AppError;
use logstream::{LogLine, LogSink};
use tasks::{TaskCtx, TaskOutput};
use tauri::ipc::Channel;

/// definitions.rs 의 망/태스크 정의를 프론트에 제공한다.
#[tauri::command]
fn get_config() -> definitions::AppConfig {
    definitions::app_config()
}

/// proxy.pac 분석으로 소속 망을 감지한다.
#[tauri::command]
async fn detect_network() -> detect::NetworkDetection {
    detect::detect_network().await
}

/// 단일 태스크를 실행하고 로그를 on_log 채널로 스트리밍한다.
#[tauri::command]
async fn run_task(
    id: String,
    network_id: String,
    on_log: Channel<LogLine>,
) -> Result<TaskOutput, AppError> {
    let network = definitions::network_by_id(&network_id);
    let sink = LogSink::new(on_log);
    let runner = tasks::runner_for(&id)
        .ok_or_else(|| AppError::Unknown(format!("알 수 없는 태스크: {}", id)))?;
    let ctx = TaskCtx {
        network,
        sink,
        dry_run: definitions::DRY_RUN,
    };
    runner.run(&ctx).await
}

/// 관리자 권한(상승)으로 실행 중인지 확인.
#[tauri::command]
fn is_elevated() -> bool {
    win::elevation::is_elevated()
}

/// 전체 로그를 지정 경로에 저장.
#[tauri::command]
fn save_log(path: String, contents: String) -> Result<(), AppError> {
    std::fs::write(&path, contents).map_err(AppError::from)
}

/// Windows Terminal 실행 (포터블 경로 우선, 없으면 PATH 의 wt).
#[tauri::command]
fn open_windows_terminal() -> Result<(), AppError> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let portable = format!("{}\\wt.exe", definitions::WT_DEST_DIR);
    let mut cmd = if std::path::Path::new(&portable).exists() {
        let mut c = std::process::Command::new(&portable);
        c.creation_flags(CREATE_NO_WINDOW);
        c
    } else {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", "", "wt"]);
        c.creation_flags(CREATE_NO_WINDOW);
        c
    };
    cmd.spawn()
        .map_err(|e| AppError::Process(format!("Windows Terminal 실행 실패: {}", e)))?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_config,
            detect_network,
            run_task,
            is_elevated,
            save_log,
            open_windows_terminal
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
