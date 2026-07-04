// ============================================================
// 디자인의 renderVals() 스타일 매핑 이식 (색상 · 칩 · 스텝)
// ============================================================
import type { LogKind, TaskStatus } from "./ipc/types";
import type { TaskState } from "./state/wizardStore";

/** 터미널 로그 색상 (디자인 매핑) */
export function logColor(kind: LogKind): string {
  switch (kind) {
    case "cmd": return "#8ab4f8";
    case "ok": return "#7ee787";
    case "warn": return "#f5c542";
    case "err": return "#ff7b72";
    case "dim":
    default: return "#9aa1aa";
  }
}

export interface Chip {
  label: string;
  bg: string;
  fg: string;
}

const CHIP_BY_STATUS: Record<TaskStatus, Chip> = {
  pending: { label: "대기", bg: "#eceff2", fg: "#8a919a" },
  running: { label: "진행 중", bg: "#eaf3fb", fg: "#0f6cbd" },
  done: { label: "완료", bg: "#f0f8f0", fg: "#107c10" },
  error: { label: "실패", bg: "#fdf3f2", fg: "#c42b1c" },
};

/** 태스크 상태 → 칩. done이고 커스텀 라벨이면 초록 칩으로 라벨 교체. */
export function chipFor(t: TaskState): Chip {
  if (t.status === "done" && t.output && t.output.chip !== "완료") {
    return { label: t.output.chip, bg: "#eef6ee", fg: "#107c10" };
  }
  return CHIP_BY_STATUS[t.status];
}

/** 태스크 카드 스타일 파생값 */
export function taskStyle(status: TaskStatus) {
  return {
    border: status === "running" ? "#9fc5e8" : status === "error" ? "#e8b7b1" : "#e3e6ea",
    iconBg:
      status === "done" ? "#f0f8f0" :
      status === "error" ? "#fdf3f2" :
      status === "running" ? "#eaf3fb" : "#f3f5f7",
    nameColor: status === "pending" ? "#8a919a" : "#1a1d21",
  };
}

export type StepPhase = "done" | "current" | "todo";

export interface StepVm {
  label: string;
  sub: string;
  glyph: string;
  circleBg: string;
  circleFg: string;
  circleBorder: string;
  labelColor: string;
  weight: number;
  rowBg: string;
  showConnector: boolean;
  connectorColor: string;
}

/** 사이드바 4단계 뷰모델 */
export function buildSteps(step: number, taskTotal: number): StepVm[] {
  const meta = [
    { label: "시작", sub: "권한 확인" },
    { label: "네트워크 확인", sub: "proxy.pac 분석" },
    { label: "설정 적용", sub: `${taskTotal}개 항목 자동 실행` },
    { label: "완료", sub: "결과 요약" },
  ];
  return meta.map((m, i) => {
    const phase: StepPhase = i < step ? "done" : i === step ? "current" : "todo";
    return {
      label: m.label,
      sub: m.sub,
      glyph: phase === "done" ? "✓" : String(i + 1),
      circleBg: phase === "done" ? "#107c10" : phase === "current" ? "#0f6cbd" : "#fff",
      circleFg: phase === "todo" ? "#8a919a" : "#fff",
      circleBorder: phase === "done" ? "#107c10" : phase === "current" ? "#0f6cbd" : "#c8cdd4",
      labelColor: phase === "todo" ? "#8a919a" : "#1a1d21",
      weight: phase === "current" ? 700 : 600,
      rowBg: phase === "current" ? "#eaf3fb" : "transparent",
      showConnector: i < 3,
      connectorColor: i < step ? "#107c10" : "#e3e6ea",
    };
  });
}
