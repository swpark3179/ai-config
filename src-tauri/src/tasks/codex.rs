// ============================================================
// [codex] Codex CLI
//  읽기: codex --version(실제)  /  설치·업데이트: 가드(npm -g @latest)
// ============================================================
use super::{first_version, TaskCtx, TaskOutput, TaskRunner};
use crate::error::AppError;
use crate::proc;
use async_trait::async_trait;

pub struct CodexTask;

const INSTALL: &str = "npm install -g @openai/codex@latest";

#[async_trait]
impl TaskRunner for CodexTask {
    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutput, AppError> {
        let before = proc::probe(&ctx.sink, "codex --version")
            .await
            .filter(|(c, o)| *c == 0 && !o.trim().is_empty())
            .map(|(_, o)| first_version(&o));

        match &before {
            Some(v) => ctx.sink.dim(format!("→ {} (설치됨) — 최신으로 업데이트", v)),
            None => ctx.sink.warn("→ 'codex' 명령을 찾을 수 없음 (미설치)".to_string()),
        }

        let code = ctx.guarded_exec(INSTALL).await?;
        if code != 0 && !ctx.dry_run {
            return Err(AppError::Task(format!(
                "Codex CLI 설치/업데이트 실패 (exit {}) — 네트워크/프록시 확인 필요",
                code
            )));
        }

        let after = proc::probe(&ctx.sink, "codex --version")
            .await
            .map(|(_, o)| first_version(&o))
            .unwrap_or_else(|| "(설치됨)".into());

        match before {
            Some(v) if v != after => {
                ctx.sink.ok(format!("✓ Codex CLI {} → {} 업데이트 완료", v, after));
                Ok(TaskOutput::done("업데이트됨", format!("v{} → v{}", v, after))
                    .with_tool("Codex CLI", format!("v{}", after)))
            }
            Some(v) => {
                ctx.sink.ok(format!("✓ Codex CLI v{} (최신)", v));
                Ok(TaskOutput::done("확인됨", format!("v{} (최신)", v))
                    .with_tool("Codex CLI", format!("v{}", v)))
            }
            None => {
                ctx.sink.ok(format!("✓ Codex CLI v{} 설치 완료", after));
                Ok(TaskOutput::done("설치됨", format!("신규 설치 → v{}", after))
                    .with_tool("Codex CLI", format!("v{}", after)))
            }
        }
    }
}
