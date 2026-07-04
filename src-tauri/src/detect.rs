// ============================================================
// 네트워크 감지 — AutoConfigURL 읽기 → proxy.pac fetch → 망 분류
// ============================================================
use crate::definitions;
use crate::error::AppError;
use crate::win::registry;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDetection {
    pub detected: bool,
    pub candidate_id: String,
    pub pac_url: Option<String>,
    pub matched_proxy: Option<String>,
}

pub async fn detect_network() -> NetworkDetection {
    let pac_url = registry::read_autoconfig_url();
    let mut candidate = definitions::default_network_id();
    let mut matched_proxy = None;

    if let Some(url) = &pac_url {
        if let Ok(body) = fetch_pac(url).await {
            matched_proxy = extract_first_proxy(&body);
            if let Some(mp) = &matched_proxy {
                if let Some(n) = definitions::networks()
                    .iter()
                    .find(|n| proxy_matches(&n.proxy, mp))
                {
                    candidate = n.id.clone();
                }
            }
        }
    }

    NetworkDetection {
        detected: pac_url.is_some(),
        candidate_id: candidate,
        pac_url,
        matched_proxy,
    }
}

/// PAC 파일을 가져온다. (reqwest blocking 을 blocking 스레드에서 실행)
async fn fetch_pac(url: &str) -> Result<String, AppError> {
    let url = url.to_string();
    tokio::task::spawn_blocking(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(6))
            .build()
            .map_err(|e| AppError::Network(e.to_string()))?;
        let resp = client
            .get(&url)
            .send()
            .map_err(|e| AppError::Network(e.to_string()))?;
        resp.text().map_err(|e| AppError::Network(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Network(e.to_string()))?
}

/// PAC 본문에서 첫 번째 `PROXY host:port` 를 추출한다.
fn extract_first_proxy(pac: &str) -> Option<String> {
    for token in pac.split(['"', '\'', ';', ',']) {
        let t = token.trim();
        if let Some(rest) = t.strip_prefix("PROXY ") {
            if let Some(hostport) = rest.trim().split_whitespace().next() {
                if !hostport.is_empty() {
                    return Some(hostport.to_string());
                }
            }
        }
    }
    None
}

/// 망 정의의 프록시와 PAC 에서 추출한 프록시가 같은 호스트인지 비교한다.
fn proxy_matches(net_proxy: &str, matched: &str) -> bool {
    let np = net_proxy
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    np == matched || np.split(':').next() == matched.split(':').next()
}
