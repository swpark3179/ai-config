// ============================================================
// 터미널 로그 스트리밍 — Tauri Channel<LogLine> 래퍼
// 프론트 TaskCard 에 실시간으로 로그 라인을 전달한다.
// ============================================================
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::ipc::Channel;

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum LogStream {
    Stdout,
    Stderr,
    Meta,
}

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum LogKind {
    Cmd,
    Ok,
    Warn,
    Err,
    Dim,
}

#[derive(Serialize, Clone)]
pub struct LogLine {
    pub stream: LogStream,
    pub kind: LogKind,
    pub text: String,
    pub ts: u64,
}

/// 태스크 하나에 대응하는 로그 채널 래퍼. clone 하여 여러 비동기 태스크에서 공유 가능.
#[derive(Clone)]
pub struct LogSink {
    ch: Channel<LogLine>,
}

impl LogSink {
    pub fn new(ch: Channel<LogLine>) -> Self {
        Self { ch }
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    fn send(&self, stream: LogStream, kind: LogKind, text: String) {
        // 디자인 규칙: cmd 는 "> ", 나머지는 "  " prefix
        let prefix = match kind {
            LogKind::Cmd => "> ",
            _ => "  ",
        };
        let _ = self.ch.send(LogLine {
            stream,
            kind,
            text: format!("{}{}", prefix, text),
            ts: Self::now(),
        });
    }

    pub fn cmd(&self, t: impl Into<String>) {
        self.send(LogStream::Stdout, LogKind::Cmd, t.into());
    }
    pub fn out(&self, t: impl Into<String>) {
        self.send(LogStream::Stdout, LogKind::Dim, t.into());
    }
    pub fn ok(&self, t: impl Into<String>) {
        self.send(LogStream::Stdout, LogKind::Ok, t.into());
    }
    pub fn warn(&self, t: impl Into<String>) {
        self.send(LogStream::Stdout, LogKind::Warn, t.into());
    }
    pub fn err(&self, t: impl Into<String>) {
        self.send(LogStream::Stderr, LogKind::Err, t.into());
    }
    pub fn dim(&self, t: impl Into<String>) {
        self.send(LogStream::Stdout, LogKind::Dim, t.into());
    }
    #[allow(dead_code)]
    pub fn meta(&self, t: impl Into<String>) {
        self.send(LogStream::Meta, LogKind::Dim, t.into());
    }
}
