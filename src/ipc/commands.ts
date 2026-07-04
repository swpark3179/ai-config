// ============================================================
// IPC 래퍼 — Tauri 환경이면 실제 #[tauri::command] 호출,
// 브라우저(개발)면 mock.ts 로 폴백.
// ============================================================
import { invoke, Channel, isTauri } from "@tauri-apps/api/core";
import type { AppConfig, LogLine, NetworkDetection, TaskOutput } from "./types";
import { mockDetectNetwork, mockGetConfig, mockRunTask } from "./mock";

function inTauri(): boolean {
  try {
    return isTauri();
  } catch {
    return false;
  }
}

/** definitions.rs 의 망/태스크 정의를 가져온다. */
export async function getConfig(): Promise<AppConfig> {
  if (!inTauri()) return mockGetConfig();
  return invoke<AppConfig>("get_config");
}

/** proxy.pac 분석으로 소속 망을 감지한다. */
export async function detectNetwork(): Promise<NetworkDetection> {
  if (!inTauri()) return mockDetectNetwork();
  return invoke<NetworkDetection>("detect_network");
}

/**
 * 단일 태스크를 실행하고 로그를 실시간 스트리밍한다.
 * @param onLog 로그 한 줄이 도착할 때마다 호출된다.
 */
export async function runTask(
  id: string,
  networkId: string,
  onLog: (line: LogLine) => void,
): Promise<TaskOutput> {
  if (!inTauri()) return mockRunTask(id, networkId, onLog);
  const onLogChannel = new Channel<LogLine>();
  onLogChannel.onmessage = onLog;
  return invoke<TaskOutput>("run_task", { id, networkId, onLog: onLogChannel });
}

/** 관리자 권한으로 실행 중인지 확인. */
export async function isElevated(): Promise<boolean> {
  if (!inTauri()) return true;
  try {
    return await invoke<boolean>("is_elevated");
  } catch {
    return false;
  }
}

/** 전체 로그를 파일로 저장 (Rust에서 파일 쓰기). */
export async function saveLog(path: string, contents: string): Promise<void> {
  await invoke("save_log", { path, contents });
}

/** Windows Terminal 실행. */
export async function openWindowsTerminal(): Promise<void> {
  if (!inTauri()) {
    alert("(mock) Windows Terminal 열기");
    return;
  }
  await invoke("open_windows_terminal");
}

/**
 * 저장 다이얼로그를 띄워 전체 로그를 파일로 저장한다.
 * 브라우저에서는 blob 다운로드로 폴백.
 */
export async function saveFullLog(contents: string): Promise<void> {
  const fileName = "ai-ready-setup-log.txt";
  if (!inTauri()) {
    const blob = new Blob([contents], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = fileName;
    a.click();
    URL.revokeObjectURL(url);
    return;
  }
  const { save } = await import("@tauri-apps/plugin-dialog");
  const path = await save({
    defaultPath: fileName,
    filters: [{ name: "Text", extensions: ["txt", "log"] }],
  });
  if (path) await saveLog(path, contents);
}
