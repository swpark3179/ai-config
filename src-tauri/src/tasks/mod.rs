// ============================================================
// 태스크 러너 프레임워크
//  - TaskRunner: 모든 태스크가 구현하는 트레잇
//  - TaskCtx:    실행 컨텍스트(망 정의 + 로그 + dry-run 가드)
//  - runner_for: id → 구현 매핑 (definitions::tasks() 의 id 와 일치)
// ============================================================
pub mod browser;
pub mod cert;
pub mod certstore;
pub mod claude;
pub mod codex;
pub mod env;
pub mod node;
pub mod npm;
pub mod userenv;
pub mod wininet;
pub mod wt;

use crate::definitions::NetworkDef;
use crate::error::AppError;
use crate::logstream::LogSink;
use crate::proc;
use async_trait::async_trait;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskOutput {
    pub chip: String,
    pub meta: String,
    pub tool_name: Option<String>,
    pub tool_ver: Option<String>,
    pub env_applied: Option<HashMap<String, String>>,
}

impl TaskOutput {
    pub fn done(chip: &str, meta: impl Into<String>) -> Self {
        TaskOutput {
            chip: chip.into(),
            meta: meta.into(),
            tool_name: None,
            tool_ver: None,
            env_applied: None,
        }
    }
    pub fn with_tool(mut self, name: &str, ver: impl Into<String>) -> Self {
        self.tool_name = Some(name.into());
        self.tool_ver = Some(ver.into());
        self
    }
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env_applied = Some(env);
        self
    }
}

/// 태스크 실행 컨텍스트
pub struct TaskCtx {
    pub network: NetworkDef,
    pub sink: LogSink,
    pub dry_run: bool,
}

impl TaskCtx {
    /// 시스템을 변경하는 외부 명령을 실행한다. dry-run 이면 실제 실행하지 않는다.
    pub async fn guarded_exec(&self, cmdline: &str) -> Result<i32, AppError> {
        if self.dry_run {
            self.sink.cmd(cmdline.to_string());
            self.sink.dim("[DRY-RUN] 실제 실행 생략".to_string());
            return Ok(0);
        }
        proc::exec(&self.sink, cmdline).await
    }

    /// 시스템을 변경하는 임의 동작(레지스트리 쓰기/파일 복사 등)을 가드한다.
    pub fn guarded<F: FnOnce() -> Result<(), AppError>>(
        &self,
        note: &str,
        f: F,
    ) -> Result<(), AppError> {
        if self.dry_run {
            self.sink.dim(format!("[DRY-RUN] {} — 생략", note));
            Ok(())
        } else {
            f()
        }
    }
}

#[async_trait]
pub trait TaskRunner: Send + Sync {
    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutput, AppError>;
}

/// CLI --version 출력에서 첫 번째 버전처럼 보이는 토큰(예: "1.3.2")을 추출한다.
pub fn first_version(s: &str) -> String {
    s.split_whitespace()
        .find(|t| {
            t.contains('.') && t.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
        })
        .map(|t| t.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.').to_string())
        .unwrap_or_else(|| s.trim().to_string())
}

pub fn runner_for(id: &str) -> Option<Box<dyn TaskRunner>> {
    match id {
        "env" => Some(Box::new(env::EnvTask)),
        "userenv" => Some(Box::new(userenv::UserEnvTask)),
        "cert" => Some(Box::new(cert::CertTask)),
        "certstore" => Some(Box::new(certstore::CertStoreTask)),
        "wininet" => Some(Box::new(wininet::WinInetTask)),
        "browser" => Some(Box::new(browser::BrowserCheckTask)),
        "node" => Some(Box::new(node::NodeTask)),
        "npm" => Some(Box::new(npm::NpmTask)),
        "claude" => Some(Box::new(claude::ClaudeTask)),
        "codex" => Some(Box::new(codex::CodexTask)),
        "wt" => Some(Box::new(wt::WtTask)),
        _ => None,
    }
}
