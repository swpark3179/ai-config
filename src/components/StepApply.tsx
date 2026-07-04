// ============================================================
// STEP 2 — 설정 적용 (진행률 + 8개 태스크 카드 + 추가 예정 항목)
// ============================================================
import type { PlannedItem, TaskDef } from "../ipc/types";
import type { TaskState } from "../state/wizardStore";
import TaskCard from "./TaskCard";

export default function StepApply({
  tasks,
  defs,
  planned,
  networkName,
  applyDone,
  onToggle,
  onRetry,
  onGoSummary,
}: {
  tasks: TaskState[];
  defs: TaskDef[];
  planned: PlannedItem[];
  networkName: string;
  applyDone: boolean;
  onToggle: (id: string) => void;
  onRetry: (id: string) => void;
  onGoSummary: () => void;
}) {
  const total = tasks.length;
  const done = tasks.filter((t) => t.status === "done").length;
  const pct = total > 0 ? Math.round((done / total) * 100) : 0;
  const defById = (id: string) => defs.find((d) => d.id === id)!;

  return (
    <div
      className="fade-up"
      style={{
        maxWidth: 760,
        margin: "0 auto",
        width: "100%",
        padding: "44px 40px 60px",
        display: "flex",
        flexDirection: "column",
        gap: 20,
      }}
    >
      <div style={{ display: "flex", alignItems: "flex-end", justifyContent: "space-between", gap: 20 }}>
        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          <h2 style={{ margin: 0, fontSize: 21, fontWeight: 700 }}>설정 적용</h2>
          <p style={{ margin: 0, fontSize: 13, color: "#5c626b" }}>
            {networkName} 기준으로 {total}개 항목을 자동 적용합니다
          </p>
        </div>
        <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end", gap: 6, flex: "none" }}>
          <span style={{ fontSize: 12, fontWeight: 600, color: "#5c626b" }}>
            {done} / {total} 완료
          </span>
          <div style={{ width: 200, height: 6, background: "#e3e6ea", borderRadius: 3, overflow: "hidden" }}>
            <div
              style={{
                height: "100%",
                background: "#0f6cbd",
                borderRadius: 3,
                transition: "width 0.4s ease",
                width: `${pct}%`,
              }}
            />
          </div>
        </div>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
        {tasks.map((t) => (
          <TaskCard
            key={t.id}
            task={t}
            def={defById(t.id)}
            onToggle={() => onToggle(t.id)}
            onRetry={() => onRetry(t.id)}
          />
        ))}
      </div>

      {/* 추가 예정 항목 */}
      <div style={{ display: "flex", flexDirection: "column", gap: 8, marginTop: 6 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8, padding: "0 2px" }}>
          <span style={{ fontSize: 11, fontWeight: 700, letterSpacing: "0.08em", color: "#8a919a" }}>
            추가 예정 항목
          </span>
          <div style={{ flex: 1, height: 1, background: "#e3e6ea" }} />
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
          {planned.map((p, i) => (
            <div
              key={i}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 11,
                padding: "12px 14px",
                background: "#f8f9fa",
                border: "1px dashed #d5dae0",
                borderRadius: 10,
                opacity: 0.75,
              }}
            >
              <div
                style={{
                  width: 24,
                  height: 24,
                  flex: "none",
                  borderRadius: "50%",
                  background: "#eceff2",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                }}
              >
                <div style={{ width: 7, height: 7, borderRadius: "50%", background: "#c8cdd4" }} />
              </div>
              <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: 1 }}>
                <span style={{ fontSize: 12.5, fontWeight: 600, color: "#6b7280" }}>{p.name}</span>
                <span style={{ fontSize: 11, color: "#9aa1aa" }}>{p.desc}</span>
              </div>
              <span
                style={{
                  fontSize: 10,
                  fontWeight: 700,
                  color: "#9aa1aa",
                  background: "#eceff2",
                  borderRadius: 8,
                  padding: "2px 8px",
                }}
              >
                예정
              </span>
            </div>
          ))}
        </div>
      </div>

      {applyDone && (
        <div style={{ display: "flex", justifyContent: "flex-end", marginTop: 4 }}>
          <button className="btn-primary" onClick={onGoSummary} style={{ fontSize: 14, padding: "11px 26px" }}>
            결과 보기
          </button>
        </div>
      )}
    </div>
  );
}
