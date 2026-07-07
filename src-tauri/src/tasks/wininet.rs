// ============================================================
// [wininet] 브라우저 프록시 설정 (인터넷 옵션)
//  Edge·Chrome 은 http_proxy 환경 변수가 아니라 Windows 인터넷 옵션(WinINET)을
//  사용한다. PAC(AutoConfigURL)이 있으면 사내 표준이므로 유지하고,
//  없으면 수동 프록시 + 예외 목록(ProxyOverride)을 구성한다.
//  마지막에 WinHTTP(시스템 서비스용)도 동기화한다.
//  읽기: 현재 WinINET 상태  /  쓰기: 가드(레지스트리 + 변경 통지 + netsh)
// ============================================================
use super::{TaskCtx, TaskOutput, TaskRunner};
use crate::error::AppError;
use crate::win::{registry, wininet};
use async_trait::async_trait;

pub struct WinInetTask;

/// no_proxy(쉼표 구분)를 WinINET ProxyOverride(세미콜론 구분) 형식으로 변환한다.
/// `<local>` 은 점 없는 호스트명(사내 단축 주소) 전체를 예외 처리한다.
pub fn to_proxy_override(no_proxy: &str) -> String {
    let mut items: Vec<String> = no_proxy
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "<local>")
        .collect();
    items.push("<local>".to_string());
    items.join(";")
}

/// 망 정의의 프록시 URL 에서 스킴을 제거해 WinINET ProxyServer 형식(host:port)으로 만든다.
fn to_proxy_server(proxy: &str) -> String {
    proxy
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/')
        .to_string()
}

#[async_trait]
impl TaskRunner for WinInetTask {
    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutput, AppError> {
        let net = &ctx.network;
        let cur = registry::read_wininet_proxy();

        // 현재 상태 로그
        ctx.sink.cmd("reg query \"HKCU\\...\\Internet Settings\"".to_string());
        match (&cur.auto_config_url, cur.proxy_enable, &cur.proxy_server) {
            (Some(pac), _, _) => ctx.sink.out(format!("현재: PAC 자동 구성 ({})", pac)),
            (None, true, Some(s)) => ctx.sink.out(format!("현재: 수동 프록시 ({})", s)),
            _ => ctx.sink.warn("현재: 프록시 미사용 — 브라우저가 사내망 밖으로 직접 연결 시도 중".to_string()),
        }

        // PAC 이 있으면 사내 배포 표준이므로 건드리지 않는다.
        if let Some(pac) = &cur.auto_config_url {
            ctx.sink.ok("✓ PAC 자동 구성 사용 중 — 수동 프록시 설정 불필요".to_string());
            ctx.sink.dim("PAC 이 망별 프록시·예외를 이미 처리합니다 (변경 없음)".to_string());
            return Ok(TaskOutput::done("유지", format!("PAC: {}", pac)));
        }

        let server = to_proxy_server(&net.proxy);
        let override_list = to_proxy_override(&net.no_proxy);
        let n_override = override_list.split(';').count();

        // 기존값 백업 로그
        if let Some(old) = &cur.proxy_server {
            if old != &server {
                ctx.sink.warn(format!("기존값 백업: ProxyServer={}", old));
            }
        }
        if let Some(old) = &cur.proxy_override {
            if old != &override_list {
                ctx.sink.warn(format!("기존값 백업: ProxyOverride={}", old));
            }
        }

        // WinINET 수동 프록시 설정
        ctx.sink.cmd(format!(
            "reg add ... /v ProxyEnable=1 ProxyServer=\"{}\" ProxyOverride=\"{}\"",
            server, override_list
        ));
        {
            let (s, o) = (server.clone(), override_list.clone());
            ctx.guarded("WinINET 프록시 쓰기", || {
                registry::write_wininet_proxy(&s, &o)
            })?;
        }

        // 실행 중인 앱(브라우저 포함)에 변경 통지
        ctx.guarded("InternetSetOption 변경 통지", || {
            wininet::refresh_settings();
            Ok(())
        })?;

        // WinHTTP(시스템 서비스·일부 백그라운드 앱)도 동일하게 동기화
        let _ = ctx.guarded_exec("netsh winhttp import proxy source=ie").await;

        // 검증
        if !ctx.dry_run {
            let after = registry::read_wininet_proxy();
            if !after.proxy_enable || after.proxy_server.as_deref() != Some(server.as_str()) {
                ctx.sink.warn("검증 경고: WinINET 설정이 예상과 다릅니다".to_string());
            }
        }

        ctx.sink.ok(format!(
            "✓ 브라우저 프록시 설정 완료 — {} · 예외 {}건",
            server, n_override
        ));
        ctx.sink.dim("Edge·Chrome 은 자동 반영되며, 안 되면 브라우저 재시작".to_string());
        Ok(TaskOutput::done(
            "완료",
            format!("수동 프록시 {} · 예외 {}건", server, n_override),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{to_proxy_override, to_proxy_server};

    #[test]
    fn converts_no_proxy_to_override() {
        assert_eq!(
            to_proxy_override("localhost,127.0.0.1, *.samsung.com"),
            "localhost;127.0.0.1;*.samsung.com;<local>"
        );
    }

    #[test]
    fn does_not_duplicate_local() {
        assert_eq!(to_proxy_override("localhost,<local>"), "localhost;<local>");
    }

    #[test]
    fn strips_scheme_from_proxy() {
        assert_eq!(to_proxy_server("http://70.10.15.10:8080"), "70.10.15.10:8080");
        assert_eq!(to_proxy_server("proxy.corp:8080"), "proxy.corp:8080");
    }
}
