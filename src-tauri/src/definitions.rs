// ============================================================
// ★★★ 세부 설정을 편집하는 단일 지점 ★★★
// 망 정의 · 태스크 목록 · 경로/파라미터 상수를 여기서 수정한다.
// (계획에 따라 외부 config 파일 대신 Rust 코드에 직접 정의)
// ============================================================
use serde::Serialize;

// ------------------------------------------------------------
// 안전 스위치
// ------------------------------------------------------------
// true  = 감지/읽기(버전 확인·레지스트리 조회·충돌 검사)는 실제 수행하지만,
//         시스템을 변경하는 쓰기/설치 작업은 실행하지 않고 "[DRY-RUN]" 로그만 남긴다.
// false = 환경 변수 수정·인증서 배치·설치 등 실제 시스템 변경을 수행한다.
//
// 검증이 끝나고 실제 배포에 사용할 때 false 로 바꾼다.
pub const DRY_RUN: bool = true;

// ------------------------------------------------------------
// 경로 · 파라미터 (환경에 맞게 수정)
// ------------------------------------------------------------
/// 인증서를 배치할 로컬 표준 경로
pub const CERT_DEST_DIR: &str = "C:\\AISetup\\certs";
/// 망별 루트 인증서가 있는 배포 공유
pub const CERT_DEPLOY_SHARE: &str = "\\\\deploy\\certs";
/// 포터블 Windows Terminal 배포 공유
pub const WT_DEPLOY_SHARE: &str = "\\\\deploy\\wt-portable";
/// Windows Terminal 을 배치할 로컬 경로
pub const WT_DEST_DIR: &str = "C:\\AISetup\\WindowsTerminal";
/// 요구되는 Node.js 최소 LTS major 버전 (미만이면 업데이트 시도)
pub const NODE_MIN_MAJOR: u32 = 22;
/// [browser] 진단 태스크가 프록시 경유 접속을 점검할 대상 (이름, URL)
/// 실제 배포 시 자주 쓰는 사내 시스템 주소로 교체한다.
pub const PROBE_URLS: &[(&str, &str)] = &[
    ("사내 포털", "https://portal.samsung.net"),
    ("외부 인터넷", "https://www.google.com"),
];

// ------------------------------------------------------------
// 데이터 구조 (프론트 types.ts 와 대응)
// ------------------------------------------------------------
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDef {
    pub id: String,
    pub name: String,
    pub proxy: String,
    pub cert: String,
    pub no_proxy: String,
    pub is_default: bool,
}

#[derive(Clone, Serialize)]
pub struct TaskDef {
    pub id: String,
    pub name: String,
    pub desc: String,
}

#[derive(Clone, Serialize)]
pub struct PlannedItem {
    pub name: String,
    pub desc: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub networks: Vec<NetworkDef>,
    pub tasks: Vec<TaskDef>,
    pub planned: Vec<PlannedItem>,
    pub app_version: String,
}

fn net(id: &str, name: &str, proxy: &str, cert: &str, no_proxy: &str, is_default: bool) -> NetworkDef {
    NetworkDef {
        id: id.into(),
        name: name.into(),
        proxy: proxy.into(),
        cert: cert.into(),
        no_proxy: no_proxy.into(),
        is_default,
    }
}

// ------------------------------------------------------------
// 망 정의 (여기서 추가/수정)
// ------------------------------------------------------------
pub fn networks() -> Vec<NetworkDef> {
    vec![
        net(
            "sds",
            "SDS 망",
            "http://70.10.15.10:8080",
            "SDS_SSL_CA.crt",
            "localhost,127.0.0.1,*.samsung.com,*.samsungds.net",
            false,
        ),
        net(
            "heavy",
            "중공업 망",
            "http://proxy.shi.internal:8080",
            "SHI_Root_CA.crt",
            "localhost,127.0.0.1,*.samsung.com,*.shi.samsung.co.kr",
            false,
        ),
        net(
            "other",
            "그 외 (일반 망)",
            "http://proxy.corp.internal:8080",
            "Corp_Root_CA.crt",
            "localhost,127.0.0.1,*.samsung.com",
            true,
        ),
    ]
}

pub fn network_by_id(id: &str) -> NetworkDef {
    let list = networks();
    list.iter()
        .find(|n| n.id == id)
        .cloned()
        .or_else(|| list.iter().find(|n| n.is_default).cloned())
        .unwrap_or_else(|| list[0].clone())
}

pub fn default_network_id() -> String {
    networks()
        .into_iter()
        .find(|n| n.is_default)
        .map(|n| n.id)
        .unwrap_or_else(|| "other".into())
}

// ------------------------------------------------------------
// 태스크 목록 (id 는 tasks/mod.rs 의 runner_for 와 일치해야 함)
// ------------------------------------------------------------
fn task(id: &str, name: &str, desc: &str) -> TaskDef {
    TaskDef { id: id.into(), name: name.into(), desc: desc.into() }
}

pub fn tasks() -> Vec<TaskDef> {
    vec![
        task("env", "시스템 환경 변수 적용", "http_proxy · https_proxy · no_proxy · NODE_EXTRA_CA_CERTS"),
        task("userenv", "사용자 환경 변수 충돌 검사", "사용자(HKCU) 변수가 시스템 설정을 덮어쓰는지 확인"),
        task("cert", "인증서 파일 배치", "망별 루트 인증서를 로컬 경로에 복사"),
        task("certstore", "인증서 신뢰 등록 (브라우저)", "루트 인증서를 Windows 신뢰 저장소에 등록 — Edge·Chrome 인증서 오류 해결"),
        task("wininet", "브라우저 프록시 설정", "인터넷 옵션(WinINET) 프록시·예외 구성 + WinHTTP 동기화"),
        task("node", "Node.js 확인 및 업데이트", "설치 여부·버전 확인, 구버전이면 최신 LTS로 업데이트"),
        task("npm", "npm 확인", "설치 여부·버전·프록시 설정 확인"),
        task("claude", "Claude Code CLI", "설치 여부 확인, 미설치 시 설치"),
        task("codex", "Codex CLI", "설치 여부·버전 확인, 구버전이면 업데이트"),
        task("wt", "Windows Terminal (Portable)", "포터블 버전을 로컬 경로에 강제 복사"),
        task("browser", "브라우저 접속 진단 (Edge·Chrome)", "인증서 신뢰·프록시 적용·정책 충돌·사내 시스템 접속 점검"),
    ]
}

pub fn planned() -> Vec<PlannedItem> {
    vec![
        PlannedItem { name: "Git 프록시 · SSL 설정".into(), desc: "http.proxy, http.sslCAInfo 자동 구성".into() },
        PlannedItem { name: "VS Code 확장 배포".into(), desc: "사내 승인 확장 오프라인 설치".into() },
        PlannedItem { name: "사내 npm registry".into(), desc: ".npmrc 사내 미러 레지스트리 등록".into() },
        PlannedItem { name: "PowerShell 실행 정책".into(), desc: "ExecutionPolicy RemoteSigned 적용".into() },
    ]
}

pub fn app_config() -> AppConfig {
    AppConfig {
        networks: networks(),
        tasks: tasks(),
        planned: planned(),
        app_version: format!("v{} (Prototype)", env!("CARGO_PKG_VERSION")),
    }
}
