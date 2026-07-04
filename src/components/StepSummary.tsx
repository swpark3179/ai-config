// ============================================================
// STEP 3 — 완료 요약 (환경변수 표 · 설치 도구 · 경고 · 액션)
// ============================================================
import type { NetworkDef } from "../ipc/types";
import type { TaskState } from "../state/wizardStore";

interface EnvRow {
  key: string;
  val: string;
}
interface ToolRow {
  name: string;
  ver: string;
}

export default function StepSummary({
  network,
  tasks,
  onOpenTerminal,
  onSaveLog,
  onRestart,
}: {
  network: NetworkDef;
  tasks: TaskState[];
  onOpenTerminal: () => void;
  onSaveLog: () => void;
  onRestart: () => void;
}) {
  const done = tasks.filter((t) => t.status === "done").length;

  // 환경 변수 표 — 선택된 망 기준 (env 태스크가 실제 적용한 값이 있으면 우선)
  const envTask = tasks.find((t) => t.id === "env");
  const envRows: EnvRow[] = envTask?.output?.envApplied
    ? Object.entries(envTask.output.envApplied).map(([key, val]) => ({ key, val }))
    : [
        { key: "http_proxy", val: network.proxy },
        { key: "https_proxy", val: network.proxy },
        { key: "no_proxy", val: network.noProxy },
        { key: "NODE_EXTRA_CA_CERTS", val: `C:\\AISetup\\certs\\${network.cert}` },
      ];

  // 설치된 도구 — 태스크 결과에서 수집
  const toolRows: ToolRow[] = tasks
    .filter((t) => t.output?.toolName && t.output?.toolVer)
    .map((t) => ({ name: t.output!.toolName!, ver: t.output!.toolVer! }));

  return (
    <div
      className="fade-up"
      style={{
        maxWidth: 680,
        margin: "0 auto",
        width: "100%",
        padding: "52px 40px 60px",
        display: "flex",
        flexDirection: "column",
        gap: 22,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
        <div
          style={{
            width: 48,
            height: 48,
            flex: "none",
            borderRadius: "50%",
            background: "#f0f8f0",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <svg width="24" height="24" viewBox="0 0 20 20">
            <circle cx="10" cy="10" r="9" fill="#107c10" />
            <path
              d="M6 10.2L8.8 13L14 7.5"
              stroke="#fff"
              strokeWidth="2"
              fill="none"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 3 }}>
          <h2 style={{ margin: 0, fontSize: 21, fontWeight: 700 }}>설정이 완료되었습니다</h2>
          <p style={{ margin: 0, fontSize: 13, color: "#5c626b" }}>
            {network.name} 환경 기준 · {done}개 항목 적용 완료
          </p>
        </div>
      </div>

      {/* 환경 변수 */}
      <div
        style={{
          background: "#fff",
          border: "1px solid #e3e6ea",
          borderRadius: 12,
          padding: "18px 20px",
          display: "flex",
          flexDirection: "column",
          gap: 12,
        }}
      >
        <span style={{ fontSize: 12, fontWeight: 700, letterSpacing: "0.06em", color: "#8a919a" }}>
          적용된 시스템 환경 변수
        </span>
        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          {envRows.map((e) => (
            <div
              key={e.key}
              style={{
                display: "grid",
                gridTemplateColumns: "190px 1fr",
                gap: 12,
                fontSize: 12,
                padding: "6px 0",
                borderBottom: "1px solid #f0f2f4",
              }}
            >
              <span style={{ fontFamily: "Consolas,monospace", fontWeight: 600, color: "#0f6cbd" }}>
                {e.key}
              </span>
              <span style={{ fontFamily: "Consolas,monospace", color: "#3a3f45", wordBreak: "break-all" }}>
                {e.val}
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* 설치된 도구 */}
      <div
        style={{
          background: "#fff",
          border: "1px solid #e3e6ea",
          borderRadius: 12,
          padding: "18px 20px",
          display: "flex",
          flexDirection: "column",
          gap: 12,
        }}
      >
        <span style={{ fontSize: 12, fontWeight: 700, letterSpacing: "0.06em", color: "#8a919a" }}>
          설치된 도구
        </span>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
          {toolRows.map((tl) => (
            <div
              key={tl.name}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 10,
                padding: "10px 12px",
                background: "#f8f9fa",
                borderRadius: 8,
              }}
            >
              <svg width="14" height="14" viewBox="0 0 14 14" style={{ flex: "none" }}>
                <path
                  d="M2.5 7.5L5.5 10.5L11.5 4"
                  stroke="#107c10"
                  strokeWidth="2.2"
                  fill="none"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
              <span style={{ fontSize: 12.5, fontWeight: 600, flex: 1 }}>{tl.name}</span>
              <span style={{ fontSize: 11.5, fontFamily: "Consolas,monospace", color: "#5c626b" }}>
                {tl.ver}
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* 경고 */}
      <div
        style={{
          display: "flex",
          alignItems: "flex-start",
          gap: 10,
          padding: "13px 16px",
          background: "#fff8ec",
          border: "1px solid #f2ddb0",
          borderRadius: 10,
        }}
      >
        <span style={{ fontSize: 14 }}>⚠</span>
        <span style={{ fontSize: 12.5, color: "#7a5b12", lineHeight: 1.55 }}>
          환경 변수는 <b>새로 여는 터미널부터</b> 적용됩니다. 열려 있는 터미널·IDE는 재시작해 주세요.
        </span>
      </div>

      {/* 액션 */}
      <div style={{ display: "flex", gap: 10 }}>
        <button className="btn-primary" onClick={onOpenTerminal} style={{ fontSize: 13.5, padding: "11px 22px" }}>
          Windows Terminal 열기
        </button>
        <button className="btn-secondary" onClick={onSaveLog} style={{ fontSize: 13.5, padding: "11px 22px" }}>
          전체 로그 저장
        </button>
        <div style={{ flex: 1 }} />
        <button className="btn-restart" onClick={onRestart} style={{ fontSize: 13, padding: "11px 14px" }}>
          처음으로
        </button>
      </div>
    </div>
  );
}
