// ============================================================
// 브라우저(비-Tauri) 개발용 Mock
// 디자인(AI Setup Wizard.dc.html)의 시뮬레이션 로직을 그대로 재현한다.
// Tauri 환경에서는 실제 백엔드 커맨드가 대신 호출된다(commands.ts).
// ============================================================
import type {
  AppConfig,
  LogLine,
  NetworkDef,
  NetworkDetection,
  TaskOutput,
} from "./types";

const NETWORKS: NetworkDef[] = [
  {
    id: "sds",
    name: "SDS 망",
    proxy: "http://70.10.15.10:8080",
    cert: "SDS_SSL_CA.crt",
    noProxy: "localhost,127.0.0.1,*.samsung.com,*.samsungds.net",
    isDefault: false,
  },
  {
    id: "heavy",
    name: "중공업 망",
    proxy: "http://proxy.shi.internal:8080",
    cert: "SHI_Root_CA.crt",
    noProxy: "localhost,127.0.0.1,*.samsung.com,*.shi.samsung.co.kr",
    isDefault: false,
  },
  {
    id: "other",
    name: "그 외 (일반 망)",
    proxy: "http://proxy.corp.internal:8080",
    cert: "Corp_Root_CA.crt",
    noProxy: "localhost,127.0.0.1,*.samsung.com",
    isDefault: true,
  },
];

export const MOCK_CONFIG: AppConfig = {
  networks: NETWORKS,
  appVersion: "v0.3.0 (Prototype)",
  tasks: [
    { id: "env", name: "시스템 환경 변수 적용", desc: "http_proxy · https_proxy · no_proxy · NODE_EXTRA_CA_CERTS" },
    { id: "userenv", name: "사용자 환경 변수 충돌 검사", desc: "사용자(HKCU) 변수가 시스템 설정을 덮어쓰는지 확인" },
    { id: "cert", name: "인증서 파일 배치", desc: "망별 루트 인증서를 로컬 경로에 복사" },
    { id: "certstore", name: "인증서 신뢰 등록 (브라우저)", desc: "루트 인증서를 Windows 신뢰 저장소에 등록 — Edge·Chrome 인증서 오류 해결" },
    { id: "wininet", name: "브라우저 프록시 설정", desc: "인터넷 옵션(WinINET) 프록시·예외 구성 + WinHTTP 동기화" },
    { id: "node", name: "Node.js 확인 및 업데이트", desc: "설치 여부·버전 확인, 구버전이면 최신 LTS로 업데이트" },
    { id: "npm", name: "npm 확인", desc: "설치 여부·버전·프록시 설정 확인" },
    { id: "claude", name: "Claude Code CLI", desc: "설치 여부 확인, 미설치 시 설치" },
    { id: "codex", name: "Codex CLI", desc: "설치 여부·버전 확인, 구버전이면 업데이트" },
    { id: "wt", name: "Windows Terminal (Portable)", desc: "포터블 버전을 로컬 경로에 강제 복사" },
    { id: "browser", name: "브라우저 접속 진단 (Edge·Chrome)", desc: "인증서 신뢰·프록시 적용·정책 충돌·사내 시스템 접속 점검" },
  ],
  planned: [
    { name: "Git 프록시 · SSL 설정", desc: "http.proxy, http.sslCAInfo 자동 구성" },
    { name: "VS Code 확장 배포", desc: "사내 승인 확장 오프라인 설치" },
    { name: "사내 npm registry", desc: ".npmrc 사내 미러 레지스트리 등록" },
    { name: "PowerShell 실행 정책", desc: "ExecutionPolicy RemoteSigned 적용" },
  ],
};

const SIM_SPEED = 1;
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms / SIM_SPEED));

function netById(id: string): NetworkDef {
  return NETWORKS.find((n) => n.id === id) ?? NETWORKS[0];
}

type ScriptLine = [number, string, LogLine["kind"]];

// codex 실패-후-재시도 시뮬레이션용 시도 카운터
const attempts: Record<string, number> = {};

function scriptFor(id: string, net: NetworkDef): { script: ScriptLine[]; out: TaskOutput } {
  switch (id) {
    case "env":
      return {
        script: [
          [300, `setx /M http_proxy "${net.proxy}"`, "cmd"],
          [350, `setx /M https_proxy "${net.proxy}"`, "cmd"],
          [300, `setx /M no_proxy "${net.noProxy}"`, "cmd"],
          [350, `setx /M NODE_EXTRA_CA_CERTS "C:\\AISetup\\certs\\${net.cert}"`, "cmd"],
          [250, "✓ 시스템(HKLM) 환경 변수 4건 등록 완료", "ok"],
        ],
        out: {
          chip: "완료",
          meta: "시스템 변수 4건 등록됨",
          toolName: null,
          toolVer: null,
          envApplied: {
            http_proxy: net.proxy,
            https_proxy: net.proxy,
            no_proxy: net.noProxy,
            NODE_EXTRA_CA_CERTS: `C:\\AISetup\\certs\\${net.cert}`,
          },
        },
      };
    case "userenv":
      return {
        script: [
          [400, "reg query HKCU\\Environment ...", "cmd"],
          [500, "! 충돌 발견: HTTP_PROXY=http://old-proxy:3128 (사용자 변수)", "warn"],
          [300, "! 충돌 발견: NODE_EXTRA_CA_CERTS=D:\\old\\ca.pem (사용자 변수)", "warn"],
          [400, "reg delete HKCU\\Environment /v HTTP_PROXY /f", "cmd"],
          [300, "reg delete HKCU\\Environment /v NODE_EXTRA_CA_CERTS /f", "cmd"],
          [250, "✓ 충돌 변수 2건 제거 완료", "ok"],
        ],
        out: { chip: "정리됨", meta: "충돌 변수 2건 발견 → 제거", toolName: null, toolVer: null, envApplied: null },
      };
    case "cert":
      return {
        script: [
          [350, "mkdir C:\\AISetup\\certs", "cmd"],
          [450, `copy \\\\deploy\\certs\\${net.cert} C:\\AISetup\\certs\\`, "cmd"],
          [300, "certutil -verify 통과 (SHA256 지문 일치)", "dim"],
          [200, `✓ ${net.cert} 배치 완료`, "ok"],
        ],
        out: { chip: "완료", meta: `C:\\AISetup\\certs\\${net.cert}`, toolName: null, toolVer: null, envApplied: null },
      };
    case "certstore":
      return {
        script: [
          [350, `certutil -dump C:\\AISetup\\certs\\${net.cert}`, "cmd"],
          [300, `인증서 주체: CN=${net.cert.replace(".crt", "")}`, "dim"],
          [300, "신뢰 저장소에 미등록 — 신규 등록", "dim"],
          [500, `certutil -addstore -f Root C:\\AISetup\\certs\\${net.cert}`, "cmd"],
          [300, "CertUtil: -addstore 명령이 성공적으로 완료되었습니다.", "dim"],
          [250, `✓ ${net.cert} 신뢰 저장소(LocalMachine\\Root) 등록 완료`, "ok"],
          [200, "실행 중인 Edge·Chrome 은 완전히 종료 후 다시 시작해야 적용됩니다", "dim"],
        ],
        out: { chip: "완료", meta: "Windows 신뢰 저장소 등록", toolName: null, toolVer: null, envApplied: null },
      };
    case "wininet":
      return {
        script: [
          [350, 'reg query "HKCU\\...\\Internet Settings"', "cmd"],
          [300, "현재: 프록시 미사용 — 브라우저가 사내망 밖으로 직접 연결 시도 중", "warn"],
          [400, `reg add ... /v ProxyEnable=1 ProxyServer="${net.proxy.replace("http://", "")}"`, "cmd"],
          [300, "netsh winhttp import proxy source=ie", "cmd"],
          [250, `✓ 브라우저 프록시 설정 완료 — ${net.proxy.replace("http://", "")} · 예외 ${net.noProxy.split(",").length + 1}건`, "ok"],
          [200, "Edge·Chrome 은 자동 반영되며, 안 되면 브라우저 재시작", "dim"],
        ],
        out: {
          chip: "완료",
          meta: `수동 프록시 ${net.proxy.replace("http://", "")} · 예외 ${net.noProxy.split(",").length + 1}건`,
          toolName: null,
          toolVer: null,
          envApplied: null,
        },
      };
    case "browser":
      return {
        script: [
          [300, "── ① 인증서 신뢰 저장소 ──", "dim"],
          [400, `certutil -store Root "${net.cert.replace(".crt", "")}"`, "cmd"],
          [250, "✓ 루트 인증서가 신뢰 저장소에 등록됨", "ok"],
          [300, "── ② 브라우저 프록시 (WinINET) ──", "dim"],
          [300, `✓ 수동 프록시 적용됨: ${net.proxy.replace("http://", "")}`, "ok"],
          [300, "── ③ 브라우저 정책(GPO) ──", "dim"],
          [350, "✓ 프록시를 강제하는 브라우저 정책 없음", "ok"],
          [300, "── ④ 접속 점검 (프록시 경유) ──", "dim"],
          [700, "GET https://portal.samsung.net (사내 포털)", "cmd"],
          [300, "✓ 사내 포털 — HTTP 200", "ok"],
          [700, "GET https://www.google.com (외부 인터넷)", "cmd"],
          [300, "✓ 외부 인터넷 — HTTP 200", "ok"],
          [250, "✓ 브라우저 접속 진단 통과 — 문제가 계속되면 브라우저 완전 종료 후 재시작", "ok"],
        ],
        out: { chip: "정상", meta: "인증서·프록시·정책·접속 4항목 통과", toolName: null, toolVer: null, envApplied: null },
      };
    case "node":
      return {
        script: [
          [400, "node --version", "cmd"],
          [300, "→ v20.11.0 (설치됨, 최신 LTS 아님)", "warn"],
          [400, "최신 LTS 확인: v22.14.0", "dim"],
          [900, "node-v22.14.0-x64.msi 다운로드… (사내 미러)", "dim"],
          [800, "msiexec /i node-v22.14.0-x64.msi /qn", "cmd"],
          [300, "✓ Node.js v22.14.0 업데이트 완료", "ok"],
        ],
        out: { chip: "업데이트됨", meta: "v20.11.0 → v22.14.0 (LTS)", toolName: "Node.js", toolVer: "v22.14.0", envApplied: null },
      };
    case "npm":
      return {
        script: [
          [350, "npm --version", "cmd"],
          [250, "→ 10.9.2 (최신)", "dim"],
          [300, `npm config set proxy ${net.proxy}`, "cmd"],
          [250, `npm config set https-proxy ${net.proxy}`, "cmd"],
          [200, "✓ npm 10.9.2 정상 · 프록시 설정 완료", "ok"],
        ],
        out: { chip: "완료", meta: "v10.9.2 · 프록시 설정 적용", toolName: "npm", toolVer: "10.9.2", envApplied: null },
      };
    case "claude":
      return {
        script: [
          [400, "claude --version", "cmd"],
          [300, "→ 'claude' 명령을 찾을 수 없음 (미설치)", "warn"],
          [1100, "npm install -g @anthropic-ai/claude-code", "cmd"],
          [400, "added 1 package in 9s", "dim"],
          [250, "✓ Claude Code v2.1.8 설치 완료", "ok"],
        ],
        out: { chip: "설치됨", meta: "신규 설치 → v2.1.8", toolName: "Claude Code", toolVer: "v2.1.8", envApplied: null },
      };
    case "codex":
      return {
        script: [
          [400, "codex --version", "cmd"],
          [300, "→ 0.42.0 (구버전)", "warn"],
          [900, "npm install -g @openai/codex@latest", "cmd"],
          [300, "✓ Codex CLI 0.42.0 → 1.3.2 업데이트 완료", "ok"],
        ],
        out: { chip: "업데이트됨", meta: "v0.42.0 → v1.3.2", toolName: "Codex CLI", toolVer: "v1.3.2", envApplied: null },
      };
    case "wt":
      return {
        script: [
          [400, "기존 설치 확인: Microsoft Store 버전 없음", "dim"],
          [900, "robocopy \\\\deploy\\wt-portable C:\\AISetup\\WindowsTerminal /MIR", "cmd"],
          [350, "128 files copied (42.3 MB)", "dim"],
          [300, "시작 메뉴 바로가기 생성", "dim"],
          [200, "✓ Windows Terminal 1.22 (Portable) 배치 완료", "ok"],
        ],
        out: {
          chip: "완료",
          meta: "C:\\AISetup\\WindowsTerminal (v1.22)",
          toolName: "Windows Terminal",
          toolVer: "v1.22 (Portable)",
          envApplied: null,
        },
      };
    default:
      return { script: [[200, "알 수 없는 태스크", "err"]], out: { chip: "실패", meta: "unknown", toolName: null, toolVer: null, envApplied: null } };
  }
}

export async function mockGetConfig(): Promise<AppConfig> {
  await sleep(80);
  return MOCK_CONFIG;
}

export async function mockDetectNetwork(): Promise<NetworkDetection> {
  await sleep(900);
  return { detected: true, candidateId: "sds", pacUrl: "http://proxy.pac.internal/proxy.pac", matchedProxy: "70.10.15.10:8080" };
}

export async function mockRunTask(
  id: string,
  networkId: string,
  onLog: (line: LogLine) => void,
): Promise<TaskOutput> {
  const net = netById(networkId);
  const attempt = attempts[id] ?? 0;
  attempts[id] = attempt + 1;

  // codex는 첫 시도 실패, 재시도 성공 (디자인의 failFirst 재현)
  const codexFail = id === "codex" && attempt === 0;
  const { script, out } = scriptFor(id, net);

  const runScript: ScriptLine[] = codexFail
    ? [
        [400, "codex --version", "cmd"],
        [300, "→ 0.42.0 (구버전)", "warn"],
        [900, "npm install -g @openai/codex@latest", "cmd"],
        [400, "✗ ETIMEDOUT: registry.npmjs.org 연결 실패 (프록시 응답 없음)", "err"],
      ]
    : script;

  if (attempt > 0) {
    onLog({ stream: "meta", kind: "dim", text: "--- 재시도 ---", ts: Date.now() });
  }

  for (const [ms, text, kind] of runScript) {
    await sleep(ms);
    onLog({
      stream: kind === "err" ? "stderr" : "stdout",
      kind,
      text: (kind === "cmd" ? "> " : "  ") + text,
      ts: Date.now(),
    });
  }

  if (codexFail) {
    throw { chip: "실패", meta: "네트워크 오류 — 재시도 필요" };
  }
  return out;
}
