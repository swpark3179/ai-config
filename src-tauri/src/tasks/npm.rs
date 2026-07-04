// ============================================================
// [npm] npm 확인 및 프록시 설정
//  읽기: npm --version(실제)  /  쓰기: 가드(npm config set proxy)
// ============================================================
use super::{TaskCtx, TaskOutput, TaskRunner};
use crate::error::AppError;
use crate::proc;
use async_trait::async_trait;

pub struct NpmTask;

#[async_trait]
impl TaskRunner for NpmTask {
    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutput, AppError> {
        let net = &ctx.network;

        let ver = match proc::probe(&ctx.sink, "npm --version").await {
            Some((0, out)) if !out.trim().is_empty() => {
                out.trim().lines().next().unwrap_or("").trim().to_string()
            }
            _ => {
                return Err(AppError::Task(
                    "npm 을 찾을 수 없습니다 (Node.js 설치가 필요합니다)".into(),
                ));
            }
        };
        ctx.sink.dim(format!("→ npm {}", ver));

        ctx.guarded_exec(&format!("npm config set proxy {}", net.proxy))
            .await?;
        ctx.guarded_exec(&format!("npm config set https-proxy {}", net.proxy))
            .await?;

        ctx.sink
            .ok(format!("✓ npm {} 정상 · 프록시 설정 완료", ver));
        Ok(TaskOutput::done("완료", format!("v{} · 프록시 설정 적용", ver)).with_tool("npm", ver))
    }
}
