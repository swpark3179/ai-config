// ============================================================
// [claude] Claude Code CLI
//  읽기: claude --version(실제)  /  설치: 가드(미설치 시 npm -g)
// ============================================================
use super::{first_version, TaskCtx, TaskOutput, TaskRunner};
use crate::error::AppError;
use crate::proc;
use async_trait::async_trait;

pub struct ClaudeTask;

const INSTALL: &str = "npm install -g @anthropic-ai/claude-code";

#[async_trait]
impl TaskRunner for ClaudeTask {
    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutput, AppError> {
        match proc::probe(&ctx.sink, "claude --version").await {
            Some((0, out)) if !out.trim().is_empty() => {
                let ver = first_version(&out);
                ctx.sink.ok(format!("✓ Claude Code v{} (설치됨)", ver));
                Ok(TaskOutput::done("확인됨", format!("v{} (설치됨)", ver))
                    .with_tool("Claude Code", format!("v{}", ver)))
            }
            _ => {
                ctx.sink.warn("→ 'claude' 명령을 찾을 수 없음 (미설치)".to_string());
                let code = ctx.guarded_exec(INSTALL).await?;
                if code != 0 && !ctx.dry_run {
                    return Err(AppError::Task(format!("Claude Code 설치 실패 (exit {})", code)));
                }
                let ver = proc::probe(&ctx.sink, "claude --version")
                    .await
                    .map(|(_, v)| first_version(&v))
                    .unwrap_or_else(|| "(설치됨)".into());
                ctx.sink.ok(format!("✓ Claude Code v{} 설치 완료", ver));
                Ok(TaskOutput::done("설치됨", format!("신규 설치 → v{}", ver))
                    .with_tool("Claude Code", format!("v{}", ver)))
            }
        }
    }
}
