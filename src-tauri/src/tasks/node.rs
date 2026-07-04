// ============================================================
// [node] Node.js 확인 및 업데이트
//  읽기: node --version(실제)  /  설치: 가드(winget, 구버전/미설치 시)
// ============================================================
use super::{TaskCtx, TaskOutput, TaskRunner};
use crate::definitions;
use crate::error::AppError;
use crate::proc;
use async_trait::async_trait;

pub struct NodeTask;

const WINGET_NODE: &str =
    "winget install OpenJS.NodeJS.LTS --silent --accept-package-agreements --accept-source-agreements";

fn parse_major(ver: &str) -> u32 {
    ver.trim()
        .trim_start_matches('v')
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0)
}

#[async_trait]
impl TaskRunner for NodeTask {
    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutput, AppError> {
        let probe = proc::probe(&ctx.sink, "node --version").await;

        match probe {
            Some((0, out)) if !out.trim().is_empty() => {
                let ver = out.trim().lines().next().unwrap_or("").trim().to_string();
                let major = parse_major(&ver);
                if major >= definitions::NODE_MIN_MAJOR {
                    ctx.sink.ok(format!("✓ Node.js {} (요구 조건 충족)", ver));
                    Ok(TaskOutput::done("확인됨", format!("{} (LTS)", ver)).with_tool("Node.js", ver))
                } else {
                    ctx.sink.warn(format!(
                        "→ {} (구버전, 최소 v{} 요구)",
                        ver,
                        definitions::NODE_MIN_MAJOR
                    ));
                    let code = ctx.guarded_exec(WINGET_NODE).await?;
                    if code != 0 && !ctx.dry_run {
                        return Err(AppError::Task(format!("Node.js 업데이트 실패 (exit {})", code)));
                    }
                    let newver = proc::probe(&ctx.sink, "node --version")
                        .await
                        .map(|(_, v)| v.trim().to_string())
                        .filter(|v| !v.is_empty())
                        .unwrap_or_else(|| "(재시작 후 확인)".into());
                    ctx.sink.ok(format!("✓ Node.js {} 업데이트 완료", newver));
                    Ok(TaskOutput::done("업데이트됨", format!("{} → {}", ver, newver))
                        .with_tool("Node.js", newver))
                }
            }
            _ => {
                ctx.sink.warn("→ Node.js 미설치".to_string());
                let code = ctx.guarded_exec(WINGET_NODE).await?;
                if code != 0 && !ctx.dry_run {
                    return Err(AppError::Task(format!("Node.js 설치 실패 (exit {})", code)));
                }
                let newver = proc::probe(&ctx.sink, "node --version")
                    .await
                    .map(|(_, v)| v.trim().to_string())
                    .filter(|v| !v.is_empty())
                    .unwrap_or_else(|| "(재시작 후 확인)".into());
                ctx.sink.ok(format!("✓ Node.js {} 설치 완료", newver));
                Ok(TaskOutput::done("설치됨", format!("신규 설치 → {}", newver))
                    .with_tool("Node.js", newver))
            }
        }
    }
}
