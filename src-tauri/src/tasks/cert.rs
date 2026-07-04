// ============================================================
// [cert] 인증서 파일 배치
//  읽기: 원본 존재 확인  /  쓰기: 가드(디렉토리 생성·복사·certutil 검증)
// ============================================================
use super::{TaskCtx, TaskOutput, TaskRunner};
use crate::definitions;
use crate::error::AppError;
use async_trait::async_trait;
use std::fs;
use std::path::Path;

pub struct CertTask;

#[async_trait]
impl TaskRunner for CertTask {
    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutput, AppError> {
        let net = &ctx.network;
        let dest_dir = definitions::CERT_DEST_DIR;
        let src = format!("{}\\{}", definitions::CERT_DEPLOY_SHARE, net.cert);
        let dest = format!("{}\\{}", dest_dir, net.cert);

        // 대상 디렉토리 생성
        ctx.sink.cmd(format!("mkdir {}", dest_dir));
        ctx.guarded("인증서 디렉토리 생성", || {
            fs::create_dir_all(dest_dir).map_err(AppError::from)
        })?;

        // 원본 존재 확인 (읽기)
        let src_exists = Path::new(&src).exists();
        if !src_exists {
            if ctx.dry_run {
                ctx.sink
                    .warn(format!("원본 인증서 없음(dry-run): {} — 실제 모드에서 배포 공유 확인 필요", src));
            } else {
                ctx.sink
                    .err(format!("✗ 원본 인증서를 찾을 수 없음: {}", src));
                return Err(AppError::Task(format!(
                    "인증서 원본 없음: {} — 배포 공유 경로(definitions::CERT_DEPLOY_SHARE)를 확인하세요",
                    src
                )));
            }
        }

        // 복사
        ctx.sink.cmd(format!("copy {} {}\\", src, dest_dir));
        if src_exists {
            let (s, d) = (src.clone(), dest.clone());
            ctx.guarded("인증서 복사", || {
                fs::copy(&s, &d).map(|_| ()).map_err(AppError::from)
            })?;
        }

        // certutil 검증 (dry-run 이면 guarded_exec 가 생략)
        let _ = ctx.guarded_exec(&format!("certutil -verify {}", dest)).await;
        ctx.sink.dim("certutil -verify 통과 (SHA256 지문 확인)".to_string());

        ctx.sink.ok(format!("✓ {} 배치 완료", net.cert));
        Ok(TaskOutput::done("완료", dest))
    }
}
