// ============================================================
// 태스크 카드 — 상태 아이콘 · 이름/메타 · 칩 · 펼침 터미널 로그 · 재시도
// ============================================================
import type { TaskDef } from "../ipc/types";
import type { TaskState } from "../state/wizardStore";
import { chipFor, logColor, taskStyle } from "../viewModel";

function StatusIcon({ status }: { status: TaskState["status"] }) {
  if (status === "running") {
    return (
      <div
        className="spin"
        style={{
          width: 14,
          height: 14,
          border: "2.5px solid #dbe7f3",
          borderTopColor: "#0f6cbd",
          borderRadius: "50%",
        }}
      />
    );
  }
  if (status === "done") {
    return (
      <svg width="13" height="13" viewBox="0 0 14 14">
        <path
          d="M2.5 7.5L5.5 10.5L11.5 4"
          stroke="#107c10"
          strokeWidth="2.2"
          fill="none"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    );
  }
  if (status === "error") {
    return (
      <svg width="12" height="12" viewBox="0 0 12 12">
        <path d="M2 2L10 10M10 2L2 10" stroke="#c42b1c" strokeWidth="2.2" strokeLinecap="round" />
      </svg>
    );
  }
  return <div style={{ width: 8, height: 8, borderRadius: "50%", background: "#c8cdd4" }} />;
}

export default function TaskCard({
  task,
  def,
  onToggle,
  onRetry,
}: {
  task: TaskState;
  def: TaskDef;
  onToggle: () => void;
  onRetry: () => void;
}) {
  const st = taskStyle(task.status);
  const chip = chipFor(task);
  const meta = task.output?.meta ?? task.errorMeta ?? def.desc;

  return (
    <div style={{ background: "#fff", border: `1px solid ${st.border}`, borderRadius: 11, overflow: "hidden" }}>
      <div
        className="task-header"
        onClick={onToggle}
        style={{ display: "flex", alignItems: "center", gap: 13, padding: "13px 16px" }}
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
            background: st.iconBg,
          }}
        >
          <StatusIcon status={task.status} />
        </div>
        <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 1 }}>
          <span style={{ fontSize: 13.5, fontWeight: 600, color: st.nameColor }}>{def.name}</span>
          <span style={{ fontSize: 11.5, color: "#8a919a" }}>{meta}</span>
        </div>

        {task.status === "error" && (
          <button
            className="btn-retry"
            onClick={(e) => {
              e.stopPropagation();
              onRetry();
            }}
            style={{ fontSize: 11.5, padding: "5px 12px" }}
          >
            재시도
          </button>
        )}

        <span
          style={{
            flex: "none",
            fontSize: 11,
            fontWeight: 700,
            padding: "3px 10px",
            borderRadius: 10,
            background: chip.bg,
            color: chip.fg,
          }}
        >
          {chip.label}
        </span>

        <svg
          width="12"
          height="12"
          viewBox="0 0 12 12"
          style={{
            flex: "none",
            transition: "transform 0.2s",
            transform: `rotate(${task.expanded ? "180deg" : "0deg"})`,
          }}
        >
          <path
            d="M2.5 4.5L6 8L9.5 4.5"
            stroke="#8a919a"
            strokeWidth="1.8"
            fill="none"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      </div>

      {task.expanded && (
        <div
          style={{
            background: "#1b1f24",
            padding: "13px 16px",
            display: "flex",
            flexDirection: "column",
            gap: 3,
            maxHeight: 220,
            overflowY: "auto",
          }}
        >
          {task.logs.length === 0 ? (
            <div style={{ fontFamily: "Consolas,monospace", fontSize: 11.5, color: "#5c626b" }}>
              아직 로그가 없습니다.
            </div>
          ) : (
            task.logs.map((ln, i) => (
              <div
                key={i}
                style={{
                  fontFamily: "Consolas,'Cascadia Mono',monospace",
                  fontSize: 11.5,
                  lineHeight: 1.55,
                  color: logColor(ln.kind),
                  whiteSpace: "pre-wrap",
                }}
              >
                {ln.text}
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}
