// ============================================================
// App — 위저드 셸 + 스텝 라우팅 + 오케스트레이션
// (디자인 Component 의 startDetect / confirmNetwork / runAll / retryTask 이식)
// ============================================================
import { useEffect, useRef } from "react";
import {
  detectNetwork,
  getConfig,
  isElevated,
  openWindowsTerminal,
  runTask,
  saveFullLog,
} from "./ipc/commands";
import type { LogLine, TaskDef } from "./ipc/types";
import { useWizard, type TaskState } from "./state/wizardStore";
import TitleBar from "./components/TitleBar";
import Sidebar from "./components/Sidebar";
import StepWelcome from "./components/StepWelcome";
import StepNetwork from "./components/StepNetwork";
import StepApply from "./components/StepApply";
import StepSummary from "./components/StepSummary";

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export default function App() {
  const { state, dispatch } = useWizard();
  // 비동기 루프에서 항상 최신 상태를 읽기 위한 ref (stale closure 방지)
  const stateRef = useRef(state);
  stateRef.current = state;
  const bootstrapped = useRef(false);

  // 최초 1회: 설정 로드 + 관리자 권한 확인
  useEffect(() => {
    if (bootstrapped.current) return;
    bootstrapped.current = true;
    (async () => {
      const [cfg, elevated] = await Promise.all([getConfig(), isElevated()]);
      dispatch({ type: "SET_CONFIG", config: cfg });
      dispatch({ type: "SET_ELEVATED", elevated });
    })();
  }, [dispatch]);

  // ---- 오케스트레이션 ----
  const startDetect = async () => {
    dispatch({ type: "DETECT_START" });
    // 감지 자체는 빠를 수 있으므로 스피너/로그 애니메이션을 위해 최소 시간 확보
    const [det] = await Promise.all([detectNetwork(), sleep(1800)]);
    dispatch({ type: "DETECT_DONE", detection: det });
  };

  const runOne = async (def: TaskDef): Promise<boolean> => {
    const cur = stateRef.current.tasks.find((t) => t.id === def.id) as TaskState;
    const isRetry = cur.attempt > 0;
    const retryMarker: LogLine = { stream: "meta", kind: "dim", text: "--- 재시도 ---", ts: Date.now() };
    dispatch({
      type: "TASK_PATCH",
      id: def.id,
      patch: { status: "running", errorMeta: null, logs: isRetry ? [...cur.logs, retryMarker] : [] },
    });
    try {
      const out = await runTask(def.id, stateRef.current.network, (line) =>
        dispatch({ type: "TASK_ADD_LOG", id: def.id, line }),
      );
      dispatch({ type: "TASK_PATCH", id: def.id, patch: { status: "done", attempt: cur.attempt + 1, output: out } });
      return true;
    } catch (e: unknown) {
      const meta = extractErrorMeta(e);
      dispatch({
        type: "TASK_PATCH",
        id: def.id,
        patch: { status: "error", attempt: cur.attempt + 1, errorMeta: meta, expanded: true },
      });
      // 에러도 로그에 남긴다 (실제 백엔드 에러가 스트림에 없을 수 있으므로)
      dispatch({
        type: "TASK_ADD_LOG",
        id: def.id,
        line: { stream: "stderr", kind: "err", text: `  ✗ ${meta}`, ts: Date.now() },
      });
      return false;
    }
  };

  const runAll = async () => {
    if (stateRef.current.running) return;
    dispatch({ type: "SET_RUNNING", running: true });
    const defs = stateRef.current.config?.tasks ?? [];
    for (const def of defs) {
      const cur = stateRef.current.tasks.find((t) => t.id === def.id);
      if (cur && cur.status === "done") continue;
      await runOne(def);
    }
    dispatch({ type: "SET_RUNNING", running: false });
    dispatch({ type: "SET_APPLY_DONE", value: true });
  };

  const confirmNetwork = () => {
    dispatch({ type: "GO_STEP", step: 2 });
    dispatch({ type: "TASKS_RESET" });
    // 상태 반영 후 실행
    setTimeout(runAll, 0);
  };

  const retryTask = async (id: string) => {
    const def = stateRef.current.config?.tasks.find((d) => d.id === id);
    if (!def) return;
    await runOne(def);
    const anyBad = stateRef.current.tasks.some((t) => t.status !== "done");
    if (!anyBad) dispatch({ type: "SET_APPLY_DONE", value: true });
  };

  const handleSaveLog = async () => {
    await saveFullLog(buildFullLog(stateRef.current.tasks, stateRef.current.config?.tasks ?? []));
  };

  // ---- 렌더 ----
  const cfg = state.config;
  if (!cfg) {
    return (
      <Shell elevated={state.elevated}>
        <div style={{ margin: "auto", display: "flex", alignItems: "center", gap: 12, color: "#8a919a" }}>
          <div
            className="spin"
            style={{ width: 22, height: 22, border: "3px solid #dbe7f3", borderTopColor: "#0f6cbd", borderRadius: "50%" }}
          />
          설정을 불러오는 중…
        </div>
      </Shell>
    );
  }

  const selectedNet = cfg.networks.find((n) => n.id === state.network) ?? cfg.networks[0];

  return (
    <Shell elevated={state.elevated} step={state.step} taskTotal={cfg.tasks.length} appVersion={cfg.appVersion}>
      {state.step === 0 && <StepWelcome onStart={startDetect} />}
      {state.step === 1 && (
        <StepNetwork
          detecting={state.detecting}
          detected={state.detected}
          detection={state.detection}
          networks={cfg.networks}
          selected={state.network}
          onSelect={(id) => dispatch({ type: "SET_NETWORK", network: id })}
          onConfirm={confirmNetwork}
          onRedetect={startDetect}
        />
      )}
      {state.step === 2 && (
        <StepApply
          tasks={state.tasks}
          defs={cfg.tasks}
          planned={cfg.planned}
          networkName={selectedNet.name}
          applyDone={state.applyDone}
          onToggle={(id) => dispatch({ type: "TASK_TOGGLE", id })}
          onRetry={retryTask}
          onGoSummary={() => dispatch({ type: "GO_STEP", step: 3 })}
        />
      )}
      {state.step === 3 && (
        <StepSummary
          network={selectedNet}
          tasks={state.tasks}
          onOpenTerminal={openWindowsTerminal}
          onSaveLog={handleSaveLog}
          onRestart={() => dispatch({ type: "RESTART" })}
        />
      )}
    </Shell>
  );
}

// 공통 레이아웃 셸 (타이틀바 + 사이드바 + 콘텐츠)
function Shell({
  children,
  elevated,
  step = 0,
  taskTotal = 8,
  appVersion = "",
}: {
  children: React.ReactNode;
  elevated: boolean;
  step?: number;
  taskTotal?: number;
  appVersion?: string;
}) {
  return (
    <div
      style={{
        width: "100%",
        height: "100vh",
        minHeight: 680,
        display: "flex",
        flexDirection: "column",
        background: "#f3f5f7",
        color: "#1a1d21",
        overflow: "hidden",
      }}
    >
      <TitleBar elevated={elevated} />
      <div style={{ flex: 1, display: "flex", minHeight: 0 }}>
        <Sidebar step={step} taskTotal={taskTotal} appVersion={appVersion} />
        <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", overflowY: "auto" }}>
          {children}
        </div>
      </div>
    </div>
  );
}

function extractErrorMeta(e: unknown): string {
  if (typeof e === "string") return e;
  if (e && typeof e === "object") {
    const o = e as Record<string, unknown>;
    if (typeof o.meta === "string") return o.meta;
    if (typeof o.message === "string") return o.message;
  }
  return "실행 중 오류가 발생했습니다";
}

function buildFullLog(tasks: TaskState[], defs: TaskDef[]): string {
  const lines: string[] = ["AI Ready — 설정 로그", `생성 시각: ${new Date().toLocaleString()}`, ""];
  for (const t of tasks) {
    const def = defs.find((d) => d.id === t.id);
    lines.push(`==== [${def?.name ?? t.id}] (${t.status}) ====`);
    for (const ln of t.logs) lines.push(ln.text);
    lines.push("");
  }
  return lines.join("\r\n");
}
