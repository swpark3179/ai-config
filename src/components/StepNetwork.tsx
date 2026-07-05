// ============================================================
// STEP 1 — 네트워크 환경 확인 (proxy.pac 감지 + 망 선택)
// ============================================================
import { useEffect, useState } from "react";
import type { NetworkDef, NetworkDetection } from "../ipc/types";

const DETECT_MESSAGES = [
  'reg query "HKCU\\...\\Internet Settings" /v AutoConfigURL',
  "GET http://proxy.pac.internal/proxy.pac … 200 OK",
  "PAC 규칙 분석: PROXY 매칭 확인 중…",
];

export default function StepNetwork({
  detecting,
  detected,
  detection,
  networks,
  selected,
  onConfirm,
  onRedetect,
}: {
  detecting: boolean;
  detected: boolean;
  detection: NetworkDetection | null;
  networks: NetworkDef[];
  selected: string;
  onConfirm: () => void;
  onRedetect: () => void;
}) {
  const [logIdx, setLogIdx] = useState(0);

  useEffect(() => {
    if (!detecting) return;
    setLogIdx(0);
    const t = setInterval(() => setLogIdx((i) => Math.min(i + 1, DETECT_MESSAGES.length - 1)), 650);
    return () => clearInterval(t);
  }, [detecting]);

  const candidateId = detection?.candidateId ?? selected;
  const detectedNet = networks.find((n) => n.id === candidateId) ?? null;
  const detectedName = detectedNet?.name ?? "";

  return (
    <div
      className="fade-up"
      style={{
        maxWidth: 680,
        margin: "0 auto",
        width: "100%",
        padding: "52px 40px",
        display: "flex",
        flexDirection: "column",
        gap: 22,
      }}
    >
      <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
        <h2 style={{ margin: 0, fontSize: 21, fontWeight: 700 }}>네트워크 환경 확인</h2>
        <p style={{ margin: 0, fontSize: 13.5, color: "#5c626b", lineHeight: 1.6 }}>
          브라우저의 자동 구성 스크립트(proxy.pac)를 분석해 소속 망을 판별합니다.
        </p>
      </div>

      {detecting && (
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 14,
            padding: 22,
            background: "#fff",
            border: "1px solid #e3e6ea",
            borderRadius: 12,
          }}
        >
          <div
            className="spin"
            style={{
              width: 26,
              height: 26,
              flex: "none",
              border: "3px solid #dbe7f3",
              borderTopColor: "#0f6cbd",
              borderRadius: "50%",
            }}
          />
          <div style={{ display: "flex", flexDirection: "column", gap: 3 }}>
            <span style={{ fontSize: 13.5, fontWeight: 600 }}>proxy.pac 분석 중…</span>
            <span style={{ fontSize: 12, color: "#8a919a", fontFamily: "Consolas,monospace" }}>
              {DETECT_MESSAGES[logIdx]}
            </span>
          </div>
        </div>
      )}

      {detected && (
        <>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 10,
              padding: "13px 16px",
              background: "#f0f8f0",
              border: "1px solid #cde8cd",
              borderRadius: 10,
            }}
          >
            <svg width="18" height="18" viewBox="0 0 20 20">
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
            <span style={{ fontSize: 13, color: "#0e5c0e" }}>
              <b>{detectedName}</b> 환경이 감지되었습니다. 확인 후 계속하세요.
            </span>
          </div>

          {detectedNet && (
            <div
              style={{
                display: "flex",
                alignItems: "flex-start",
                gap: 14,
                padding: "16px 18px",
                background: "#fff",
                border: "1.5px solid #0f6cbd",
                borderRadius: 12,
                boxShadow: "0 0 0 3px rgba(15,108,189,0.12)",
              }}
            >
              {/* 자동 판별된 망은 고정 — 사용자가 변경할 수 없으므로 라디오 대신 잠금 표시 */}
              <svg width="18" height="18" viewBox="0 0 20 20" style={{ flex: "none", marginTop: 2 }}>
                <rect x="4" y="9" width="12" height="8" rx="1.5" fill="#0f6cbd" />
                <path
                  d="M6.5 9V6.8a3.5 3.5 0 0 1 7 0V9"
                  stroke="#0f6cbd"
                  strokeWidth="1.8"
                  fill="none"
                  strokeLinecap="round"
                />
                <circle cx="10" cy="12.5" r="1.4" fill="#fff" />
              </svg>
              <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: 6 }}>
                <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                  <span style={{ fontSize: 14, fontWeight: 700 }}>{detectedNet.name}</span>
                  <span
                    style={{
                      fontSize: 10.5,
                      fontWeight: 700,
                      color: "#0f6cbd",
                      background: "#eaf3fb",
                      borderRadius: 9,
                      padding: "2px 8px",
                    }}
                  >
                    자동 감지됨
                  </span>
                </div>
                <div
                  style={{
                    display: "grid",
                    gridTemplateColumns: "88px 1fr",
                    gap: "3px 10px",
                    fontSize: 12,
                  }}
                >
                  <span style={{ color: "#8a919a" }}>프록시</span>
                  <span style={{ fontFamily: "Consolas,monospace", color: "#3a3f45" }}>
                    {detectedNet.isDefault ? `${detectedNet.proxy}  (기본값)` : detectedNet.proxy}
                  </span>
                  <span style={{ color: "#8a919a" }}>인증서</span>
                  <span style={{ fontFamily: "Consolas,monospace", color: "#3a3f45" }}>{detectedNet.cert}</span>
                </div>
              </div>
            </div>
          )}

          <p style={{ margin: 0, fontSize: 12, color: "#8a919a", lineHeight: 1.6 }}>
            소속 망은 프록시 설정을 기준으로 자동 판별되며, 임의로 변경할 수 없습니다.
          </p>

          <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
            <button className="btn-primary" onClick={onConfirm} style={{ fontSize: 14, padding: "11px 26px" }}>
              이 설정으로 계속
            </button>
            <button className="btn-ghost" onClick={onRedetect} style={{ fontSize: 13, padding: "11px 10px" }}>
              다시 감지
            </button>
          </div>
        </>
      )}
    </div>
  );
}
