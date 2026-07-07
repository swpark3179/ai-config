// ============================================================
// [certstore] 루트 인증서를 Windows 신뢰 저장소에 등록 (Edge·Chrome 대응)
//  Edge·Chrome 은 NODE_EXTRA_CA_CERTS 를 읽지 않고 Windows 인증서 저장소를
//  사용하므로, 파일 배치(cert)와 별도로 LocalMachine\Root 등록이 있어야
//  SSL 검사 프록시 망에서 NET::ERR_CERT_AUTHORITY_INVALID 가 사라진다.
//  읽기: 파일 존재·주체 CN·기존 등록 확인  /  쓰기: 가드(certutil -addstore)
// ============================================================
use super::{TaskCtx, TaskOutput, TaskRunner};
use crate::definitions;
use crate::error::AppError;
use crate::proc;
use async_trait::async_trait;
use std::path::Path;

pub struct CertStoreTask;

/// certutil -dump 출력에서 주체 CN 을 추출한다.
/// 출력이 현지화되어도 "CN=" 토큰은 유지된다. 발급자 CN 이 먼저 나오므로
/// 마지막 매칭(주체)을 사용한다 — 루트 CA 는 자체 서명이라 둘이 같다.
pub fn extract_cn(dump: &str) -> Option<String> {
    dump.lines()
        .filter_map(|l| {
            let idx = l.find("CN=")?;
            let rest = &l[idx + 3..];
            let cn = rest.split(',').next()?.trim();
            (!cn.is_empty()).then(|| cn.to_string())
        })
        .last()
}

#[async_trait]
impl TaskRunner for CertStoreTask {
    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutput, AppError> {
        let net = &ctx.network;
        let cert_path = format!("{}\\{}", definitions::CERT_DEST_DIR, net.cert);

        // 배치된 인증서 파일 확인 (cert 태스크 선행 필요)
        if !Path::new(&cert_path).exists() {
            if ctx.dry_run {
                ctx.sink.warn(format!(
                    "인증서 파일 없음(dry-run): {} — 실제 모드에서는 [cert] 태스크가 먼저 배치",
                    cert_path
                ));
            } else {
                ctx.sink.err(format!("✗ 인증서 파일을 찾을 수 없음: {}", cert_path));
                return Err(AppError::Task(format!(
                    "인증서 파일 없음: {} — [인증서 파일 배치] 태스크를 먼저 실행하세요",
                    cert_path
                )));
            }
        }

        // 주체 CN 추출 (등록 확인·검증에 사용)
        let mut subject: Option<String> = None;
        if Path::new(&cert_path).exists() {
            if let Some((0, dump)) =
                proc::probe_quiet(&ctx.sink, &format!("certutil -dump \"{}\"", cert_path)).await
            {
                subject = extract_cn(&dump);
            }
            match &subject {
                Some(cn) => ctx.sink.dim(format!("인증서 주체: CN={}", cn)),
                None => ctx.sink.warn("인증서 주체(CN)를 확인하지 못함 — 등록 후 검증 생략".to_string()),
            }
        }

        // 기존 등록 여부 (멱등 — 이미 있어도 -f 로 갱신 등록)
        if let Some(cn) = &subject {
            let found = proc::probe_quiet(&ctx.sink, &format!("certutil -store Root \"{}\"", cn))
                .await
                .map(|(code, _)| code == 0)
                .unwrap_or(false);
            if found {
                ctx.sink.dim("이미 신뢰 저장소에 등록됨 — 갱신 등록 수행".to_string());
            } else {
                ctx.sink.dim("신뢰 저장소에 미등록 — 신규 등록".to_string());
            }
        }

        // LocalMachine\Root 등록 (관리자 권한 필요)
        let code = ctx
            .guarded_exec(&format!("certutil -addstore -f Root \"{}\"", cert_path))
            .await?;
        if code != 0 && !ctx.dry_run {
            ctx.sink.err("✗ certutil -addstore 실패".to_string());
            return Err(AppError::Task(format!(
                "신뢰 저장소 등록 실패 (exit {}) — 관리자 권한·인증서 형식(.crt/DER·PEM)을 확인하세요",
                code
            )));
        }

        // 검증
        if !ctx.dry_run {
            if let Some(cn) = &subject {
                let ok = proc::probe_quiet(&ctx.sink, &format!("certutil -store Root \"{}\"", cn))
                    .await
                    .map(|(c, _)| c == 0)
                    .unwrap_or(false);
                if !ok {
                    ctx.sink.warn("검증 경고: 등록 후에도 저장소에서 찾지 못했습니다".to_string());
                }
            }
        }

        ctx.sink.ok(format!(
            "✓ {} 신뢰 저장소(LocalMachine\\Root) 등록 완료",
            net.cert
        ));
        ctx.sink.dim("실행 중인 Edge·Chrome 은 완전히 종료 후 다시 시작해야 적용됩니다".to_string());
        Ok(TaskOutput::done("완료", "Windows 신뢰 저장소 등록"))
    }
}

#[cfg(test)]
mod tests {
    use super::extract_cn;

    #[test]
    fn extracts_last_cn() {
        let dump = "발급자:\n    CN=SDS Root CA, O=Samsung\n주체:\n    CN=SDS SSL CA, O=Samsung SDS\n";
        assert_eq!(extract_cn(dump).as_deref(), Some("SDS SSL CA"));
    }

    #[test]
    fn none_when_missing() {
        assert_eq!(extract_cn("no subject here"), None);
    }
}
