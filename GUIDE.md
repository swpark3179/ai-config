# AI Ready — 구현 요약 & 세부 기능 커스터마이징 가이드

이 문서는 (1) 현재까지 구현된 내용을 요약하고, (2) 세부 기능을 직접 추가/수정하려면
어디를 어떻게 손대야 하는지 실전 가이드를 제공합니다.

---

## 1. 무엇이 구현되었나 (요약)

Claude Design(`AI Setup Wizard.dc.html`)의 4단계 위저드 UI를 **Tauri 2.0 + React(TS/Vite)**
Windows 11 데스크톱 앱으로 이식하고, 각 카테고리의 기본 틀을 실제로 동작하도록 구현했습니다.

| 영역 | 내용 |
|------|------|
| UI | 커스텀 타이틀바 · 사이드바(4단계) · STEP0~3 · 8개 태스크 카드(실시간 터미널 로그) · 요약 |
| 권한 | 항상 관리자 실행 (`requireAdministrator` manifest 임베드) |
| 태스크 | env · userenv · cert · node · npm · claude · codex · wt (8종) |
| 동작 수준 | **감지/읽기 = 실제**, **쓰기/설치 = 실제 + 가드**(백업·멱등·검증) |
| 안전장치 | `DRY_RUN` 스위치 (기본 `true`: 읽기만 실제, 쓰기/설치는 로그만) |
| 로그 | 태스크별 `Channel<LogLine>` 실시간 스트리밍 |
| 감지 | 레지스트리 `AutoConfigURL` → proxy.pac fetch → 망 분류 |
| 설정 | 외부 파일 없이 `src-tauri/src/definitions.rs` 에 직접 정의 |

빌드 산출물: `src-tauri/target/release/bundle/{msi,nsis}/` 에 MSI · NSIS 설치 파일.

---

## 2. 파일 구조 & 역할

```
src/                              ── React 프론트엔드
  App.tsx                         위저드 셸 + 오케스트레이션(감지/실행/재시도)
  main.tsx                        Provider + global.css 마운트
  viewModel.ts                    색상·칩·스텝 매핑 (디자인 renderVals 이식)
  styles/global.css               keyframes · 스크롤바 · :hover/:active 클래스
  state/wizardStore.tsx           useReducer + Context 중앙 상태
  ipc/
    types.ts                      Rust 구조체 미러 TS 타입
    commands.ts                   invoke 래퍼 (Tauri↔mock 자동 분기)
    mock.ts                       브라우저 개발용 시뮬레이션
  components/
    TitleBar.tsx Sidebar.tsx
    StepWelcome/Network/Apply/Summary.tsx  TaskCard.tsx

src-tauri/src/                    ── Rust 백엔드
  lib.rs                          ★ 커맨드 등록 + 플러그인 + 진입점
  definitions.rs                  ★★ 세부 설정 편집 지점 (망/태스크/경로/DRY_RUN)
  error.rs                        AppError (프론트로 문자열 직렬화)
  logstream.rs                    LogLine · LogSink (Channel 래퍼)
  proc.rs                         외부 프로세스 실행/스트리밍 (probe/exec)
  detect.rs                       네트워크 감지
  win/registry.rs                 HKLM/HKCU 레지스트리 읽기/쓰기/삭제
  win/broadcast.rs                WM_SETTINGCHANGE 브로드캐스트
  win/elevation.rs                관리자 권한 확인
  tasks/mod.rs                    ★ TaskRunner 트레잇 · TaskCtx · runner_for(dispatch)
  tasks/<id>.rs                   ★ 각 태스크 실제 로직
src-tauri/manifest.xml           ★ 관리자 권한 manifest (⚠ ASCII만!)
src-tauri/build.rs               manifest 임베드
src-tauri/tauri.conf.json        창 크기/프레임리스/번들 설정
src-tauri/capabilities/default.json  창 제어 + 플러그인 권한
```

★ = 자주 손대는 파일, ★★ = 가장 먼저 보는 파일.

---

## 3. 프론트엔드 ↔ 백엔드 계약

`src/ipc/commands.ts` 가 아래 커맨드를 래핑합니다 (Rust `#[tauri::command]` ↔ JS camelCase 자동 변환).

| 커맨드 | Rust 시그니처 | 용도 |
|--------|--------------|------|
| `get_config` | `() -> AppConfig` | 망/태스크/추가예정/버전 반환 |
| `detect_network` | `() -> NetworkDetection` | proxy.pac 분석·망 분류 |
| `run_task` | `(id, network_id, on_log: Channel) -> TaskOutput` | 태스크 실행 + 로그 스트리밍 |
| `is_elevated` | `() -> bool` | 관리자 여부(배지) |
| `save_log` | `(path, contents) -> ()` | 로그 파일 저장 |
| `open_windows_terminal` | `() -> ()` | WT 실행 |

로그 한 줄: `LogLine { stream, kind, text, ts }` — `kind`(cmd/ok/warn/err/dim)가 색상을 결정.

> Rust 타입을 바꾸면 `src/ipc/types.ts` 도 함께 맞춰야 합니다(수동 동기화).

---

## 4. 세부 기능 커스터마이징

### 4-1. ⚠️ 안전 스위치 (가장 먼저 이해할 것)

`src-tauri/src/definitions.rs`:

```rust
pub const DRY_RUN: bool = true;
```

- `true` (기본): 버전 확인·레지스트리 조회·충돌 검사 등 **읽기는 실제** 수행,
  환경 변수 수정·설치·복사 등 **시스템 변경은 실행하지 않고 `[DRY-RUN]` 로그만** 남김.
- `false`: 실제 시스템 변경 수행.

**실제로 시스템을 바꾸려면 반드시 `false` 로 변경**하고 다시 빌드/실행하세요.

### 4-2. 망(네트워크) 추가/수정

`definitions.rs` 의 `networks()` 를 편집합니다. `net(id, name, proxy, cert, no_proxy, is_default)`:

```rust
pub fn networks() -> Vec<NetworkDef> {
    vec![
        net("sds", "SDS 망", "http://70.10.15.10:8080", "SDS_SSL_CA.crt",
            "localhost,127.0.0.1,*.samsung.com,*.samsungds.net", false),
        // ↓ 새 망 추가 예시
        net("rnd", "연구소 망", "http://proxy.rnd.internal:8080", "RND_CA.crt",
            "localhost,127.0.0.1,*.samsung.com", false),
        net("other", "그 외 (일반 망)", "http://proxy.corp.internal:8080", "Corp_Root_CA.crt",
            "localhost,127.0.0.1,*.samsung.com", true), // is_default=true 는 하나만
    ]
}
```

- `id` 는 내부 식별자(감지 매칭·선택에 사용), `is_default=true` 는 감지 실패 시 폴백.
- 프론트는 `get_config` 로 이 목록을 받아 자동 렌더 → **UI 수정 불필요**.

### 4-3. 경로·파라미터 상수

`definitions.rs` 상단 상수를 실제 환경에 맞게 수정:

```rust
pub const CERT_DEST_DIR: &str    = "C:\\AISetup\\certs";     // 인증서 배치 위치
pub const CERT_DEPLOY_SHARE: &str = "\\\\deploy\\certs";      // 인증서 원본 공유
pub const WT_DEPLOY_SHARE: &str  = "\\\\deploy\\wt-portable"; // WT 원본 공유
pub const WT_DEST_DIR: &str      = "C:\\AISetup\\WindowsTerminal";
pub const NODE_MIN_MAJOR: u32    = 22;                        // 요구 Node LTS
```

### 4-4. 기존 태스크 로직 수정

각 태스크는 `src-tauri/src/tasks/<id>.rs` 에 독립적으로 있습니다. 예: `npm` 의 프록시 설정을
바꾸려면 `tasks/npm.rs` 에서 `guarded_exec` 호출을 수정.

**태스크 안에서 쓸 수 있는 도구:**

```rust
// 로그 (색상별)
ctx.sink.cmd("명령 표시");   // > 파랑
ctx.sink.out("일반 출력");   //   회색
ctx.sink.ok("✓ 성공");       //   초록
ctx.sink.warn("! 경고");     //   노랑
ctx.sink.err("✗ 오류");      //   빨강 (stderr)
ctx.sink.dim("부가 설명");   //   회색

// 외부 명령 (cmd /C 로 실행 → npm/claude 등 .cmd 도 OK)
let out: Option<(i32, String)> = proc::probe(&ctx.sink, "node --version").await; // 읽기(캡처)
let code: i32 = ctx.guarded_exec("npm install -g pkg").await?;  // 쓰기(DRY_RUN 가드+스트리밍)

// 시스템 변경 동작 가드 (레지스트리 쓰기, 파일 복사 등)
ctx.guarded("HKLM foo 쓰기", || registry::write_hklm_env("foo", "bar"))?;

// 레지스트리 헬퍼 (win/registry.rs)
registry::read_hklm_env("http_proxy");
registry::write_hklm_env("http_proxy", &net.proxy);   // 관리자 필요
registry::read_hkcu_env("HTTP_PROXY");
registry::delete_hkcu_env("HTTP_PROXY");
crate::win::broadcast::broadcast_env_change();         // 변경 통지

// 결과 반환
Ok(TaskOutput::done("완료", "메타 설명")
    .with_tool("Node.js", "v22.14.0")   // 요약 화면 "설치된 도구" 목록에 표시
    .with_env(map))                     // 요약 화면 환경 변수 표에 표시
```

`ctx.network` 로 선택된 망(proxy/cert/no_proxy)에 접근합니다.

### 4-5. 🆕 새 태스크 추가 (전체 절차)

예로 `git` 프록시 설정 태스크를 추가한다고 하면:

**① 태스크 목록에 등록** — `definitions.rs`:
```rust
pub fn tasks() -> Vec<TaskDef> {
    vec![
        // ...기존 8개...
        task("git", "Git 프록시 · SSL 설정", "http.proxy, http.sslCAInfo 자동 구성"),
    ]
}
```

**② 태스크 구현 파일 작성** — `src-tauri/src/tasks/git.rs`:
```rust
use super::{TaskCtx, TaskOutput, TaskRunner};
use crate::error::AppError;
use async_trait::async_trait;

pub struct GitTask;

#[async_trait]
impl TaskRunner for GitTask {
    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutput, AppError> {
        let net = &ctx.network;
        ctx.guarded_exec(&format!("git config --global http.proxy {}", net.proxy)).await?;
        ctx.guarded_exec(&format!(
            "git config --global http.sslCAInfo {}\\{}",
            crate::definitions::CERT_DEST_DIR, net.cert
        )).await?;
        ctx.sink.ok("✓ Git 프록시/SSL 설정 완료");
        Ok(TaskOutput::done("완료", "http.proxy · sslCAInfo 적용"))
    }
}
```

**③ 모듈 + dispatch 등록** — `src-tauri/src/tasks/mod.rs`:
```rust
pub mod git;                       // 상단 모듈 선언에 추가

pub fn runner_for(id: &str) -> Option<Box<dyn TaskRunner>> {
    match id {
        // ...기존...
        "git" => Some(Box::new(git::GitTask)),
        _ => None,
    }
}
```

끝입니다. 프론트는 `get_config` 로 새 태스크를 자동 렌더하고 순차 실행합니다. **UI 수정 불필요.**

> `id`(definitions ↔ runner_for)는 반드시 일치해야 합니다.

### 4-6. "추가 예정 항목" 관리 / 실제 태스크로 승격

- 표시만: `definitions.rs` 의 `planned()` 수정.
- 실제 기능화: 4-5 절차대로 `tasks()` 로 옮기고 러너를 구현한 뒤, `planned()` 에서 제거.

### 4-7. 네트워크 감지 로직

`src-tauri/src/detect.rs`:
- `extract_first_proxy()` — PAC 본문에서 프록시 추출 규칙
- `proxy_matches()` — 추출된 프록시 ↔ 망 정의 매칭 규칙
사내 PAC 형식에 맞게 이 두 함수를 조정하면 자동 감지 정확도를 높일 수 있습니다.

### 4-8. UI(디자인) 수정

- 색상/칩/애니메이션: `src/viewModel.ts`, `src/styles/global.css`
- Welcome 카드 문구: `src/components/StepWelcome.tsx` 의 `WELCOME_ITEMS`
- 각 스텝 레이아웃: `src/components/Step*.tsx` (inline style이 디자인과 1:1)
- 창 크기/타이틀: `src-tauri/tauri.conf.json`, 사이드바 버전 문구: `definitions.rs` 의 `app_version`

---

## 5. 빌드 · 실행 · 트러블슈팅

```bash
npm install
npm run dev            # 브라우저에서 mock 흐름 미리보기 (권한 불필요)
npm run tauri dev      # 실제 앱 (⚠ 관리자 권한 터미널에서, UAC 프롬프트 1회)
npm run tauri build    # MSI/NSIS 설치 파일 생성
```

**트러블슈팅**

- **"side-by-side 구성이 잘못되었습니다" (SXS)**: `manifest.xml` 에 **비-ASCII 문자(한글 주석 등)** 가
  들어가면 임베드 시 손상되어 발생합니다. → manifest는 **영문/ASCII만** 사용하세요. (이미 수정됨)
- **관리자 권한 필요**: `requireAdministrator` 때문에 dev 실행은 **관리자 터미널**에서 해야 합니다.
  일반 터미널에서는 실행 실패(740)합니다.
- **환경 변수가 반영 안 됨**: 새로 여는 터미널부터 적용됩니다(기존 터미널/IDE 재시작 필요).
- **DRY_RUN**: 실제 변경이 안 일어난다면 `definitions.rs` 의 `DRY_RUN` 이 `true` 인지 확인하세요.
```
