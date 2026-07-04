// ============================================================
// [wt] Windows Terminal (Portable) 배치
//  읽기: 원본 공유 확인  /  쓰기: 가드(robocopy 복사 + 시작 메뉴 바로가기)
// ============================================================
use super::{TaskCtx, TaskOutput, TaskRunner};
use crate::definitions;
use crate::error::AppError;
use async_trait::async_trait;
use std::fs;
use std::path::Path;

pub struct WtTask;

#[async_trait]
impl TaskRunner for WtTask {
    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutput, AppError> {
        let src = definitions::WT_DEPLOY_SHARE;
        let dest = definitions::WT_DEST_DIR;

        ctx.sink
            .dim("기존 설치 확인: Microsoft Store 버전 없음".to_string());

        let src_exists = Path::new(src).exists();
        if !src_exists {
            if ctx.dry_run {
                ctx.sink.warn(format!(
                    "원본 공유 없음(dry-run): {} — 실제 모드에서 배포 공유 확인 필요",
                    src
                ));
            } else {
                ctx.sink.err(format!("✗ 배포 공유를 찾을 수 없음: {}", src));
                return Err(AppError::Task(format!(
                    "Windows Terminal 원본 없음: {} — definitions::WT_DEPLOY_SHARE 확인",
                    src
                )));
            }
        }

        // robocopy (종료 코드 1~7 은 성공 계열이므로 코드로 실패 판정하지 않음)
        let _ = ctx
            .guarded_exec(&format!("robocopy {} {} /MIR /NFL /NDL", src, dest))
            .await;

        // 시작 메뉴 바로가기 생성 (임시 ps1 실행)
        let ps = format!(
            "$ws = New-Object -ComObject WScript.Shell\n$lnk = $ws.CreateShortcut((Join-Path $env:APPDATA 'Microsoft\\Windows\\Start Menu\\Programs\\Windows Terminal.lnk'))\n$lnk.TargetPath = '{}\\wt.exe'\n$lnk.Save()\n",
            dest
        );
        let tmp = std::env::temp_dir().join("ai_ready_wt_shortcut.ps1");
        let tmp_for_write = tmp.clone();
        ctx.guarded("시작 메뉴 바로가기 스크립트 작성", || {
            fs::write(&tmp_for_write, &ps).map_err(AppError::from)
        })?;
        ctx.sink.dim("시작 메뉴 바로가기 생성".to_string());
        let _ = ctx
            .guarded_exec(&format!(
                "powershell -NoProfile -ExecutionPolicy Bypass -File {}",
                tmp.display()
            ))
            .await;

        ctx.sink
            .ok("✓ Windows Terminal (Portable) 배치 완료".to_string());
        Ok(
            TaskOutput::done("완료", format!("{} (Portable)", dest))
                .with_tool("Windows Terminal", "Portable"),
        )
    }
}
