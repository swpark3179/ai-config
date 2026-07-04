// ============================================================
// 위저드 중앙 상태 (useReducer + Context)
// 디자인(AI Setup Wizard.dc.html) Component.state 를 미러링
// ============================================================
import {
  createContext,
  useContext,
  useReducer,
  type Dispatch,
  type ReactNode,
} from "react";
import type {
  AppConfig,
  LogLine,
  NetworkDetection,
  TaskDef,
  TaskOutput,
  TaskStatus,
} from "../ipc/types";

export interface TaskState {
  id: string;
  status: TaskStatus;
  logs: LogLine[];
  expanded: boolean;
  attempt: number;
  output: TaskOutput | null;
  /** 에러 시 결과 메타 라인 */
  errorMeta: string | null;
}

export interface WizardState {
  config: AppConfig | null;
  elevated: boolean;
  step: number; // 0..3
  network: string; // 선택된 망 id
  detecting: boolean;
  detected: boolean;
  detection: NetworkDetection | null;
  tasks: TaskState[];
  running: boolean;
  applyDone: boolean;
}

function freshTasks(defs: TaskDef[]): TaskState[] {
  return defs.map((d) => ({
    id: d.id,
    status: "pending" as TaskStatus,
    logs: [],
    expanded: false,
    attempt: 0,
    output: null,
    errorMeta: null,
  }));
}

export const initialState: WizardState = {
  config: null,
  elevated: false,
  step: 0,
  network: "sds",
  detecting: false,
  detected: false,
  detection: null,
  tasks: [],
  running: false,
  applyDone: false,
};

export type Action =
  | { type: "SET_CONFIG"; config: AppConfig }
  | { type: "SET_ELEVATED"; elevated: boolean }
  | { type: "GO_STEP"; step: number }
  | { type: "SET_NETWORK"; network: string }
  | { type: "DETECT_START" }
  | { type: "DETECT_DONE"; detection: NetworkDetection }
  | { type: "TASKS_RESET" }
  | { type: "TASK_PATCH"; id: string; patch: Partial<TaskState> }
  | { type: "TASK_ADD_LOG"; id: string; line: LogLine }
  | { type: "TASK_TOGGLE"; id: string }
  | { type: "SET_RUNNING"; running: boolean }
  | { type: "SET_APPLY_DONE"; value: boolean }
  | { type: "RESTART" };

function patchTask(tasks: TaskState[], id: string, patch: Partial<TaskState>): TaskState[] {
  return tasks.map((t) => (t.id === id ? { ...t, ...patch } : t));
}

export function reducer(state: WizardState, action: Action): WizardState {
  switch (action.type) {
    case "SET_CONFIG":
      return {
        ...state,
        config: action.config,
        tasks: freshTasks(action.config.tasks),
      };
    case "SET_ELEVATED":
      return { ...state, elevated: action.elevated };
    case "GO_STEP":
      return { ...state, step: action.step };
    case "SET_NETWORK":
      return { ...state, network: action.network };
    case "DETECT_START":
      return { ...state, step: 1, detecting: true, detected: false, detection: null };
    case "DETECT_DONE":
      return {
        ...state,
        detecting: false,
        detected: true,
        detection: action.detection,
        network: action.detection.candidateId || state.network,
      };
    case "TASKS_RESET":
      return {
        ...state,
        applyDone: false,
        tasks: state.config ? freshTasks(state.config.tasks) : [],
      };
    case "TASK_PATCH":
      return { ...state, tasks: patchTask(state.tasks, action.id, action.patch) };
    case "TASK_ADD_LOG":
      return {
        ...state,
        tasks: state.tasks.map((t) =>
          t.id === action.id ? { ...t, logs: [...t.logs, action.line] } : t,
        ),
      };
    case "TASK_TOGGLE":
      return {
        ...state,
        tasks: state.tasks.map((t) =>
          t.id === action.id ? { ...t, expanded: !t.expanded } : t,
        ),
      };
    case "SET_RUNNING":
      return { ...state, running: action.running };
    case "SET_APPLY_DONE":
      return { ...state, applyDone: action.value };
    case "RESTART":
      return {
        ...state,
        step: 0,
        detecting: false,
        detected: false,
        detection: null,
        applyDone: false,
        running: false,
        tasks: state.config ? freshTasks(state.config.tasks) : [],
      };
    default:
      return state;
  }
}

const WizardCtx = createContext<{
  state: WizardState;
  dispatch: Dispatch<Action>;
} | null>(null);

export function WizardProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(reducer, initialState);
  return <WizardCtx.Provider value={{ state, dispatch }}>{children}</WizardCtx.Provider>;
}

export function useWizard() {
  const ctx = useContext(WizardCtx);
  if (!ctx) throw new Error("useWizard must be used within WizardProvider");
  return ctx;
}
