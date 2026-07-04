// ============================================================
// [env] 시스템(HKLM) 환경 변수 적용
//  읽기: 기존 값 조회(백업 로그)  /  쓰기: 가드(기존값 백업·멱등·검증)
// ============================================================
use super::{TaskCtx, TaskOutput, TaskRunner};
use crate::definitions;
use crate::error::AppError;
use crate::win::{broadcast, registry};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct EnvTask;

#[async_trait]
impl TaskRunner for EnvTask {
    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutput, AppError> {
        let net = &ctx.network;
        let cert_path = format!("{}\\{}", definitions::CERT_DEST_DIR, net.cert);
        let vars: [(&str, String); 4] = [
            ("http_proxy", net.proxy.clone()),
            ("https_proxy", net.proxy.clone()),
            ("no_proxy", net.no_proxy.clone()),
            ("NODE_EXTRA_CA_CERTS", cert_path.clone()),
        ];

        let mut applied: HashMap<String, String> = HashMap::new();
        let mut changed = 0;

        for (name, value) in &vars {
            let existing = registry::read_hklm_env(name);
            match &existing {
                Some(v) if v == value => {
                    ctx.sink.dim(format!("{} 이미 최신 값 — 건너뜀", name));
                }
                Some(v) => {
                    ctx.sink.warn(format!("기존값 백업: {}={}", name, v));
                }
                None => {
                    ctx.sink.dim(format!("{} 없음 — 신규 등록", name));
                }
            }

            if existing.as_deref() != Some(value.as_str()) {
                ctx.sink.cmd(format!("setx /M {} \"{}\"", name, value));
                let (n, val) = (*name, value.clone());
                ctx.guarded(&format!("HKLM {} 쓰기", name), || {
                    registry::write_hklm_env(n, &val)
                })?;
                changed += 1;
            }
            applied.insert(name.to_string(), value.clone());
        }

        // 새 프로세스가 인식하도록 변경 통지
        ctx.guarded("WM_SETTINGCHANGE 브로드캐스트", || {
            broadcast::broadcast_env_change();
            Ok(())
        })?;

        // 검증 (실제 적용된 경우에만)
        if !ctx.dry_run {
            for (name, value) in &vars {
                match registry::read_hklm_env(name) {
                    Some(v) if &v == value => {}
                    _ => ctx
                        .sink
                        .warn(format!("검증 경고: {} 값이 예상과 다릅니다", name)),
                }
            }
        }

        ctx.sink
            .ok(format!("✓ 시스템(HKLM) 환경 변수 {}건 적용 완료", vars.len()));

        let meta = if changed == 0 {
            format!("시스템 변수 {}건 (변경 없음)", vars.len())
        } else {
            format!("시스템 변수 {}건 등록됨", changed)
        };
        Ok(TaskOutput::done("완료", meta).with_env(applied))
    }
}
