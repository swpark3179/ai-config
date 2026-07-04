// ============================================================
// [userenv] 사용자(HKCU) 환경 변수 충돌 검사
//  읽기: 사용자 변수 조회(실제)  /  삭제: 가드(백업 로그 후 제거)
//  시스템 설정을 덮어쓰는 사용자 변수를 찾아 제거한다.
// ============================================================
use super::{TaskCtx, TaskOutput, TaskRunner};
use crate::error::AppError;
use crate::win::{broadcast, registry};
use async_trait::async_trait;

// 시스템 프록시/인증서 설정을 덮어쓸 수 있는 사용자 변수 후보
const CONFLICT_VARS: &[&str] = &[
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "http_proxy",
    "https_proxy",
    "NO_PROXY",
    "NODE_EXTRA_CA_CERTS",
];

pub struct UserEnvTask;

#[async_trait]
impl TaskRunner for UserEnvTask {
    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutput, AppError> {
        ctx.sink.cmd("reg query HKCU\\Environment".to_string());

        let mut removed = 0;
        for name in CONFLICT_VARS {
            if let Some(val) = registry::read_hkcu_env(name) {
                ctx.sink
                    .warn(format!("! 충돌 발견: {}={} (사용자 변수)", name, val));
                ctx.sink
                    .cmd(format!("reg delete HKCU\\Environment /v {} /f", name));
                let n = *name;
                ctx.guarded(&format!("HKCU {} 삭제", name), || {
                    registry::delete_hkcu_env(n)
                })?;
                removed += 1;
            }
        }

        if removed > 0 {
            ctx.guarded("WM_SETTINGCHANGE 브로드캐스트", || {
                broadcast::broadcast_env_change();
                Ok(())
            })?;
            ctx.sink.ok(format!("✓ 충돌 변수 {}건 제거 완료", removed));
            Ok(TaskOutput::done(
                "정리됨",
                format!("충돌 변수 {}건 발견 → 제거", removed),
            ))
        } else {
            ctx.sink.ok("✓ 충돌하는 사용자 변수 없음".to_string());
            Ok(TaskOutput::done("완료", "충돌 없음"))
        }
    }
}
