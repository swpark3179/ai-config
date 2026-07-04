// ============================================================
// 사이드바 — 4단계 진행 표시기 + 버전 푸터
// ============================================================
import { buildSteps } from "../viewModel";

export default function Sidebar({
  step,
  taskTotal,
  appVersion,
}: {
  step: number;
  taskTotal: number;
  appVersion: string;
}) {
  const steps = buildSteps(step, taskTotal);

  return (
    <div
      style={{
        width: 230,
        flex: "none",
        background: "#fff",
        borderRight: "1px solid #e3e6ea",
        padding: "28px 18px",
        display: "flex",
        flexDirection: "column",
        gap: 4,
      }}
    >
      <div
        style={{
          fontSize: 11,
          fontWeight: 700,
          letterSpacing: "0.08em",
          color: "#8a919a",
          padding: "0 8px 12px",
        }}
      >
        설정 단계
      </div>

      {steps.map((s, i) => (
        <div key={i}>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 11,
              padding: "9px 8px",
              borderRadius: 8,
              background: s.rowBg,
            }}
          >
            <div
              style={{
                width: 26,
                height: 26,
                flex: "none",
                borderRadius: "50%",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                fontSize: 12,
                fontWeight: 700,
                background: s.circleBg,
                color: s.circleFg,
                border: `1.5px solid ${s.circleBorder}`,
              }}
            >
              {s.glyph}
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 1 }}>
              <span style={{ fontSize: 13, fontWeight: s.weight, color: s.labelColor }}>
                {s.label}
              </span>
              <span style={{ fontSize: 11, color: "#9aa1aa" }}>{s.sub}</span>
            </div>
          </div>
          {s.showConnector && (
            <div
              style={{
                width: 2,
                height: 14,
                background: s.connectorColor,
                marginLeft: 20,
                borderRadius: 1,
              }}
            />
          )}
        </div>
      ))}

      <div style={{ flex: 1 }} />

      <div
        style={{
          padding: 12,
          borderRadius: 8,
          background: "#f6f8fa",
          border: "1px solid #eceff2",
          fontSize: 11,
          lineHeight: 1.5,
          color: "#6b7280",
        }}
      >
        {appVersion}
        <br />
        Tauri 2.0 · Windows 11
      </div>
    </div>
  );
}
