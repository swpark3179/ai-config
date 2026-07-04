// ============================================================
// 커스텀 타이틀바 (40px) — 로고 · 제목 · 관리자 권한 배지 · 창 컨트롤
// ============================================================
import { isTauri } from "@tauri-apps/api/core";

async function win() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  return getCurrentWindow();
}

async function onMinimize() {
  if (isTauri()) (await win()).minimize();
}
async function onToggleMaximize() {
  if (isTauri()) (await win()).toggleMaximize();
}
async function onClose() {
  if (isTauri()) (await win()).close();
}

export default function TitleBar({ elevated }: { elevated: boolean }) {
  return (
    <div
      style={{
        height: 40,
        flex: "none",
        display: "flex",
        alignItems: "center",
        background: "#fff",
        borderBottom: "1px solid #e3e6ea",
        paddingLeft: 14,
        userSelect: "none",
      }}
    >
      {/* 드래그 가능 영역 */}
      <div
        data-tauri-drag-region
        style={{ display: "flex", alignItems: "center", gap: 8, flex: 1, height: "100%" }}
      >
        <div
          style={{
            width: 18,
            height: 18,
            borderRadius: 4,
            background: "linear-gradient(135deg,#0f6cbd,#4f9ee8)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: "#fff",
            fontSize: 10,
            fontWeight: 700,
          }}
        >
          AI
        </div>
        <span style={{ fontSize: 12, fontWeight: 600, color: "#3a3f45" }}>
          AI Ready — 사내 AI 환경 설정 도우미
        </span>
        {elevated && (
          <span
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: 4,
              fontSize: 10.5,
              fontWeight: 600,
              color: "#0f6cbd",
              background: "#eaf3fb",
              border: "1px solid #cfe4f7",
              borderRadius: 10,
              padding: "2px 8px",
            }}
          >
            <svg width="10" height="12" viewBox="0 0 10 12" fill="none">
              <path d="M5 0L10 2V5.5C10 8.5 8 11 5 12C2 11 0 8.5 0 5.5V2L5 0Z" fill="#0f6cbd" />
            </svg>
            관리자 권한
          </span>
        )}
      </div>

      {/* 창 컨트롤 */}
      <div style={{ display: "flex", height: 40 }}>
        <div className="win-btn" style={{ fontSize: 13 }} onClick={onMinimize}>
          –
        </div>
        <div className="win-btn" style={{ fontSize: 11 }} onClick={onToggleMaximize}>
          □
        </div>
        <div className="win-btn win-close" style={{ fontSize: 13 }} onClick={onClose}>
          ✕
        </div>
      </div>
    </div>
  );
}
