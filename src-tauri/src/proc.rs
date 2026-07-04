// ============================================================
// 외부 프로세스 실행 헬퍼
// - Windows 의 .cmd shim(npm/claude/codex) 을 위해 `cmd /C` 로 실행
// - stdout/stderr 를 라인 단위로 LogSink 에 스트리밍
// - probe(): 버전 확인처럼 출력 캡처가 필요한 읽기 작업
// - exec():  설치처럼 실시간 스트리밍이 필요한 작업
// ============================================================
use crate::error::AppError;
use crate::logstream::LogSink;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn shell(cmdline: &str) -> Command {
    let mut c = Command::new("cmd");
    c.args(["/C", cmdline]);
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);
    c
}

/// 명령을 실행하고 (exit_code, stdout) 을 캡처한다.
/// 프로그램을 찾을 수 없거나 실행 실패 시 None 을 반환하고 경고 로그를 남긴다.
pub async fn probe(sink: &LogSink, cmdline: &str) -> Option<(i32, String)> {
    sink.cmd(cmdline.to_string());
    match shell(cmdline).output().await {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            for l in stdout.lines() {
                sink.out(l.to_string());
            }
            for l in stderr.lines() {
                sink.err(l.to_string());
            }
            Some((out.status.code().unwrap_or(-1), stdout))
        }
        Err(e) => {
            sink.warn(format!("→ 실행할 수 없음: {}", e));
            None
        }
    }
}

/// 명령을 실행하고 출력을 실시간 스트리밍한다. exit code 를 반환.
pub async fn exec(sink: &LogSink, cmdline: &str) -> Result<i32, AppError> {
    sink.cmd(cmdline.to_string());
    let mut child = shell(cmdline)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Process(format!("'{}' 실행 실패: {}", cmdline, e)))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let (so, se) = (sink.clone(), sink.clone());
    let h_out = tokio::spawn(async move {
        if let Some(o) = stdout {
            pump(o, so, false).await;
        }
    });
    let h_err = tokio::spawn(async move {
        if let Some(e) = stderr {
            pump(e, se, true).await;
        }
    });

    let status = child
        .wait()
        .await
        .map_err(|e| AppError::Process(e.to_string()))?;
    let _ = h_out.await;
    let _ = h_err.await;
    Ok(status.code().unwrap_or(-1))
}

/// 리더를 라인 단위로 읽어 LogSink 로 흘려보낸다.
/// UTF-8 이 아닌 바이트도 스트림이 끊기지 않도록 lossy 변환한다.
async fn pump<R: AsyncRead + Unpin>(reader: R, sink: LogSink, is_err: bool) {
    let mut buf = BufReader::new(reader);
    let mut line: Vec<u8> = Vec::new();
    loop {
        line.clear();
        match buf.read_until(b'\n', &mut line).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let text = String::from_utf8_lossy(&line);
                let text = text.trim_end_matches(['\r', '\n']);
                if text.is_empty() {
                    continue;
                }
                if is_err {
                    sink.err(text.to_string());
                } else {
                    sink.out(text.to_string());
                }
            }
        }
    }
}
