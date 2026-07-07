# AI Ready — 사내 AI 환경 설정 도우미

신규 PC의 AI 개발 환경(프록시·인증서·시스템 환경 변수·AI CLI 도구)을 4단계 위저드로
자동 구성하는 Tauri 2.0 + React 데스크톱 앱 (Windows 11 전용).

## 구조

```
src/                 React 프론트엔드 (디자인 이식)
  components/         TitleBar · Sidebar · Step0~3 · TaskCard
  ipc/               commands.ts(invoke 래퍼) · mock.ts(브라우저 폴백) · types.ts
  state/             wizardStore(useReducer+Context)
src-tauri/src/
  definitions.rs     ★ 세부 설정 편집 지점 (망/태스크/경로/DRY_RUN)
  tasks/             11개 태스크 (env·userenv·cert·certstore·wininet·node·npm·claude·codex·wt·browser)
  win/               registry · broadcast(WM_SETTINGCHANGE) · elevation · wininet(설정 변경 통지)
  proc.rs            외부 프로세스 실행/스트리밍
  detect.rs          proxy.pac 네트워크 감지
  logstream.rs       Channel<LogLine> 로그 스트리밍
```

## 개발 실행

관리자 권한이 필요하므로 **관리자 권한 터미널**에서 실행:

```bash
npm install
npm run tauri dev      # 실행 시 UAC 프롬프트 1회
```

브라우저만으로 UI 흐름을 보려면 `npm run dev` → mock 데이터로 전체 위저드 동작.

## 빌드

```bash
npm run tauri build    # 관리자 manifest가 임베드된 설치 파일(NSIS/MSI) 생성
```

## ⚠️ 안전 스위치 (DRY_RUN)

`src-tauri/src/definitions.rs` 최상단:

```rust
pub const DRY_RUN: bool = true;   // 기본값: 안전 모드
```

- `true`  — 감지/읽기(버전 확인·레지스트리 조회·충돌 검사)는 **실제** 수행하지만,
  시스템을 변경하는 쓰기/설치는 실행하지 않고 `[DRY-RUN]` 로그만 남긴다.
- `false` — 시스템 환경 변수 수정·인증서 배치·CLI 설치 등 **실제 변경**을 수행한다.

검증이 끝난 뒤 실제 배포에 사용할 때 `false` 로 바꾼다.

## 세부 설정 수정

`src-tauri/src/definitions.rs` 한 곳에서 수정한다.

- 망 정의: `networks()` — SDS / 중공업 / 그 외 (id·이름·프록시·인증서·no_proxy)
- 경로: `CERT_DEST_DIR`, `CERT_DEPLOY_SHARE`, `WT_DEPLOY_SHARE`, `WT_DEST_DIR`
- Node 최소 버전: `NODE_MIN_MAJOR`
- 브라우저 접속 진단 대상 URL: `PROBE_URLS`
- 태스크 목록/설명: `tasks()`, 추가 예정 항목: `planned()`

각 태스크의 실제 명령/로직은 `src-tauri/src/tasks/<id>.rs` 에서 수정한다.

## 브라우저(Edge·Chrome) 접속 이슈 대응

CLI 환경 변수(`http_proxy` 등)는 브라우저에 적용되지 않는다. 브라우저에서 사내 시스템
접속 시 발생하는 문제는 아래 3개 태스크가 담당한다.

| 태스크 | 해결하는 증상 |
|--------|--------------|
| `certstore` — 인증서 신뢰 등록 | `NET::ERR_CERT_AUTHORITY_INVALID` 등 인증서 경고. 루트 CA 를 Windows 신뢰 저장소(`LocalMachine\Root`)에 등록 — Edge·Chrome 은 이 저장소를 사용 |
| `wininet` — 브라우저 프록시 설정 | 페이지가 아예 안 열리는 문제. 인터넷 옵션(WinINET)에 프록시·예외 목록 구성 (PAC 이 있으면 유지) + WinHTTP 동기화 |
| `browser` — 접속 진단 | 인증서 신뢰·프록시 적용·GPO 정책 강제 여부·프록시 경유 실제 접속(407 인증/타임아웃/TLS 오류)을 점검하고 처방 안내 |
