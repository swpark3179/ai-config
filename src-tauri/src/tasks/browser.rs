// ============================================================
// [browser] 브라우저 접속 진단 (Edge·Chrome)
//  사내 시스템 접속 시 브라우저에서 발생하는 대표 원인 4가지를 점검한다.
//   ① 루트 인증서가 Windows 신뢰 저장소에 없음 (NET::ERR_CERT_AUTHORITY_INVALID)
//   ② WinINET 프록시 미적용 (환경 변수만으로는 브라우저에 반영 안 됨)
//   ③ GPO 정책이 프록시를 강제 (위저드 설정보다 우선 적용)
//   ④ 프록시 경유 실제 접속 실패 (인증 407·타임아웃·TLS 오류)
//  전 항목 읽기 전용 — DRY_RUN 여부와 무관하게 실제 점검한다.
// ============================================================
use super::{certstore, TaskCtx, TaskOutput, TaskRunner};
use crate::definitions;
use crate::error::AppError;
use crate::proc;
use crate::win::registry;
use async_trait::async_trait;
use std::path::Path;
use std::time::Duration;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

pub struct BrowserCheckTask;

const POLICY_PATHS: &[(&str, &str)] = &[
    ("Edge", "SOFTWARE\\Policies\\Microsoft\\Edge"),
    ("Chrome", "SOFTWARE\\Policies\\Google\\Chrome"),
];
const POLICY_PROXY_VALUES: &[&str] = &["ProxyMode", "ProxySettings", "ProxyServer", "ProxyPacUrl"];

/// 프록시를 경유해 URL 에 접속해 본다. (블로킹 reqwest → blocking 스레드)
/// native-tls(schannel)가 Windows 인증서 저장소로 검증하므로 브라우저와 같은 조건이다.
async fn probe_url(proxy: &str, url: &str) -> Result<u16, String> {
    let (proxy, url) = (proxy.to_string(), url.to_string());
    tokio::task::spawn_blocking(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(8))
            .proxy(reqwest::Proxy::all(&proxy).map_err(|e| e.to_string())?)
            .build()
            .map_err(|e| e.to_string())?;
        client
            .get(&url)
            .send()
            .map(|r| r.status().as_u16())
            .map_err(|e| {
                if e.is_timeout() {
                    "timeout".to_string()
                } else {
                    format!("{:?}", e)
                }
            })
    })
    .await
    .map_err(|e| e.to_string())?
}

fn classify_probe_error(err: &str) -> &'static str {
    let low = err.to_lowercase();
    if low.contains("timeout") {
        "프록시 응답 없음 — 망 선택이 맞는지, 방화벽 승인 여부를 확인하세요"
    } else if low.contains("certificate") || low.contains("cert") || low.contains("tls") || low.contains("ssl") {
        "TLS 인증서 검증 실패 — [인증서 신뢰 등록] 태스크 완료 여부를 확인하세요"
    } else if low.contains("dns") || low.contains("resolve") {
        "주소를 찾을 수 없음 — DNS/프록시 주소를 확인하세요"
    } else {
        "연결 실패 — 프록시 주소·네트워크 상태를 확인하세요"
    }
}

#[async_trait]
impl TaskRunner for BrowserCheckTask {
    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutput, AppError> {
        let net = &ctx.network;
        let mut issues: u32 = 0;

        // ① 루트 인증서 신뢰 저장소 등록 확인
        ctx.sink.dim("── ① 인증서 신뢰 저장소 ──".to_string());
        let cert_path = format!("{}\\{}", definitions::CERT_DEST_DIR, net.cert);
        if Path::new(&cert_path).exists() {
            let mut registered = false;
            if let Some((0, dump)) =
                proc::probe_quiet(&ctx.sink, &format!("certutil -dump \"{}\"", cert_path)).await
            {
                if let Some(cn) = certstore::extract_cn(&dump) {
                    registered =
                        proc::probe_quiet(&ctx.sink, &format!("certutil -store Root \"{}\"", cn))
                            .await
                            .map(|(c, _)| c == 0)
                            .unwrap_or(false);
                }
            }
            if registered {
                ctx.sink.ok("✓ 루트 인증서가 신뢰 저장소에 등록됨".to_string());
            } else {
                issues += 1;
                ctx.sink.warn(
                    "! 신뢰 저장소 미등록 — 브라우저에서 인증서 오류 발생. [인증서 신뢰 등록] 실행 필요".to_string(),
                );
            }
        } else {
            issues += 1;
            ctx.sink.warn(format!(
                "! 인증서 파일 없음: {} — [인증서 파일 배치]부터 실행 필요",
                cert_path
            ));
        }

        // ② WinINET(브라우저) 프록시 적용 상태
        ctx.sink.dim("── ② 브라우저 프록시 (WinINET) ──".to_string());
        let w = registry::read_wininet_proxy();
        match (&w.auto_config_url, w.proxy_enable, &w.proxy_server) {
            (Some(pac), _, _) => ctx.sink.ok(format!("✓ PAC 자동 구성 사용 중: {}", pac)),
            (None, true, Some(s)) => {
                ctx.sink.ok(format!("✓ 수동 프록시 적용됨: {}", s));
                match &w.proxy_override {
                    Some(o) => ctx.sink.dim(format!("예외(ProxyOverride): {}", o)),
                    None => ctx.sink.warn(
                        "! 프록시 예외 목록이 비어 있음 — 사내 주소도 프록시를 타서 느려질 수 있음".to_string(),
                    ),
                }
            }
            _ => {
                issues += 1;
                ctx.sink.warn(
                    "! 브라우저 프록시 미설정 — Edge·Chrome 이 사내망 밖으로 직접 연결을 시도합니다. [브라우저 프록시 설정] 실행 필요".to_string(),
                );
            }
        }

        // WinHTTP (시스템 서비스용) 상태
        let _ = proc::probe(&ctx.sink, "netsh winhttp show proxy").await;

        // ③ GPO 정책이 브라우저 프록시를 강제하는지
        ctx.sink.dim("── ③ 브라우저 정책(GPO) ──".to_string());
        let mut forced = false;
        for (browser, path) in POLICY_PATHS {
            for (hive_name, hive) in [("HKLM", HKEY_LOCAL_MACHINE), ("HKCU", HKEY_CURRENT_USER)] {
                for value in POLICY_PROXY_VALUES {
                    if let Some(v) = registry::read_string_at(hive, path, value) {
                        forced = true;
                        ctx.sink.warn(format!(
                            "! {} 정책 감지: {}\\{}\\{} = {} — 이 설정이 인터넷 옵션보다 우선합니다",
                            browser, hive_name, path, value, v
                        ));
                    } else if let Some(v) = registry::read_dword_at(hive, path, value) {
                        forced = true;
                        ctx.sink.warn(format!(
                            "! {} 정책 감지: {}\\{}\\{} = {} — 이 설정이 인터넷 옵션보다 우선합니다",
                            browser, hive_name, path, value, v
                        ));
                    }
                }
            }
        }
        if forced {
            issues += 1;
            ctx.sink.dim("정책은 IT 관리 부서가 배포한 것으로, 임의 변경 대신 관리자 문의가 필요합니다".to_string());
        } else {
            ctx.sink.ok("✓ 프록시를 강제하는 브라우저 정책 없음".to_string());
        }

        // ④ 프록시 경유 실제 접속 점검
        ctx.sink.dim("── ④ 접속 점검 (프록시 경유) ──".to_string());
        for (label, url) in definitions::PROBE_URLS {
            ctx.sink.cmd(format!("GET {} ({})", url, label));
            match probe_url(&net.proxy, url).await {
                Ok(status) if (200..400).contains(&status) => {
                    ctx.sink.ok(format!("✓ {} — HTTP {}", label, status));
                }
                Ok(407) => {
                    issues += 1;
                    ctx.sink.warn(format!(
                        "! {} — HTTP 407 프록시 인증 필요. 브라우저 최초 접속 시 사번/비밀번호 인증 창을 확인하세요",
                        label
                    ));
                }
                Ok(status) => {
                    issues += 1;
                    ctx.sink.warn(format!(
                        "! {} — HTTP {} 응답. 프록시 차단 정책 또는 접근 권한을 확인하세요",
                        label, status
                    ));
                }
                Err(e) => {
                    issues += 1;
                    ctx.sink.warn(format!("! {} — {}", label, classify_probe_error(&e)));
                }
            }
        }

        // 마무리 안내
        if issues == 0 {
            ctx.sink.ok("✓ 브라우저 접속 진단 통과 — 문제가 계속되면 브라우저 완전 종료 후 재시작".to_string());
            Ok(TaskOutput::done("정상", "인증서·프록시·정책·접속 4항목 통과"))
        } else {
            ctx.sink.warn(format!("! 조치가 필요한 항목 {}건", issues));
            ctx.sink.dim(
                "설정 변경 후에도 오류가 남으면: 브라우저 완전 종료 → 재시작, edge://net-internals/#hsts 에서 도메인 HSTS 삭제, 캐시 삭제(Ctrl+Shift+Del)".to_string(),
            );
            Ok(TaskOutput::done("점검 필요", format!("조치 필요 {}건", issues)))
        }
    }
}
