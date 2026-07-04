// ============================================================
// STEP 0 — Welcome
// ============================================================

const WELCOME_ITEMS = [
  { glyph: "⇄", title: "프록시·환경 변수", desc: "망 자동 감지 후 시스템 변수 적용, 사용자 변수 충돌 정리" },
  { glyph: "⛨", title: "인증서 배치", desc: "망별 루트 인증서를 표준 경로에 복사" },
  { glyph: "❯", title: "AI CLI 도구", desc: "Node.js · npm · Claude Code · Codex 설치 및 업데이트" },
  { glyph: "⌸", title: "Windows Terminal", desc: "포터블 버전 강제 복사 배치" },
];

export default function StepWelcome({ onStart }: { onStart: () => void }) {
  return (
    <div
      className="fade-up"
      style={{
        maxWidth: 640,
        margin: "0 auto",
        padding: "64px 40px",
        display: "flex",
        flexDirection: "column",
        gap: 24,
      }}
    >
      <div
        style={{
          width: 56,
          height: 56,
          borderRadius: 14,
          background: "linear-gradient(135deg,#0f6cbd,#4f9ee8)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none">
          <path
            d="M12 2L20 5V11C20 16.5 16.5 20.7 12 22C7.5 20.7 4 16.5 4 11V5L12 2Z"
            stroke="#fff"
            strokeWidth="1.8"
            fill="none"
          />
          <path
            d="M8.5 12L11 14.5L15.5 9.5"
            stroke="#fff"
            strokeWidth="1.8"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
        <h1 style={{ margin: 0, fontSize: 26, fontWeight: 700, letterSpacing: "-0.01em" }}>
          사내 AI 개발 환경을
          <br />
          자동으로 설정합니다
        </h1>
        <p style={{ margin: 0, fontSize: 14, lineHeight: 1.65, color: "#5c626b" }}>
          현재 PC의 네트워크 환경(proxy.pac)을 감지하여 프록시·인증서·환경 변수를 구성하고, AI
          CLI 도구 설치 상태를 점검합니다. 전 과정은 자동으로 진행되며 각 단계의 상세 로그를 확인할
          수 있습니다.
        </p>
      </div>

      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10 }}>
        {WELCOME_ITEMS.map((w, i) => (
          <div
            key={i}
            style={{
              display: "flex",
              alignItems: "flex-start",
              gap: 10,
              padding: 14,
              background: "#fff",
              border: "1px solid #e3e6ea",
              borderRadius: 10,
            }}
          >
            <div
              style={{
                width: 30,
                height: 30,
                flex: "none",
                borderRadius: 8,
                background: "#eaf3fb",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                fontSize: 14,
                color: "#0f6cbd",
                fontWeight: 700,
              }}
            >
              {w.glyph}
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
              <span style={{ fontSize: 13, fontWeight: 600 }}>{w.title}</span>
              <span style={{ fontSize: 11.5, color: "#8a919a", lineHeight: 1.45 }}>{w.desc}</span>
            </div>
          </div>
        ))}
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: 14, marginTop: 8 }}>
        <button
          className="btn-primary"
          onClick={onStart}
          style={{ fontSize: 14, padding: "11px 28px", boxShadow: "0 1px 2px rgba(15,108,189,0.3)" }}
        >
          시작하기
        </button>
        <span style={{ fontSize: 12, color: "#8a919a" }}>약 2~5분 소요 · 관리자 권한으로 실행 중</span>
      </div>
    </div>
  );
}
