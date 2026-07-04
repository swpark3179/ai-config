// ============================================================
// 프론트엔드 ↔ 백엔드 공유 타입 (Rust 구조체 미러)
// ============================================================

/** 터미널 로그 한 줄. kind는 디자인의 색상 매핑에 대응. */
export type LogStream = "stdout" | "stderr" | "meta";
export type LogKind = "cmd" | "ok" | "warn" | "err" | "dim";

export interface LogLine {
  stream: LogStream;
  kind: LogKind;
  text: string;
  ts: number;
}

/** 망 정의 (Rust definitions.rs 의 NetworkDef) */
export interface NetworkDef {
  id: string;
  name: string;
  proxy: string;
  cert: string;
  /** no_proxy 목록 (쉼표 구분) */
  noProxy: string;
  /** "그 외(일반 망)"처럼 기본값 망 여부 */
  isDefault: boolean;
}

/** 태스크 정의 (Rust definitions.rs 의 TaskDef) */
export interface TaskDef {
  id: string;
  name: string;
  desc: string;
}

/** 추가 예정(미구현) 항목 */
export interface PlannedItem {
  name: string;
  desc: string;
}

/** get_config 반환값 */
export interface AppConfig {
  networks: NetworkDef[];
  tasks: TaskDef[];
  planned: PlannedItem[];
  /** 사이드바 버전 표기 */
  appVersion: string;
}

/** detect_network 반환값 */
export interface NetworkDetection {
  detected: boolean;
  candidateId: string;
  pacUrl: string | null;
  matchedProxy: string | null;
}

/** run_task 반환값 (구조화 결과) */
export interface TaskOutput {
  /** 완료 칩 라벨 (예: "완료", "업데이트됨", "설치됨", "정리됨") */
  chip: string;
  /** 결과 메타 라인 */
  meta: string;
  /** 요약 화면 "설치된 도구" 목록에 넣을 이름 (없으면 null) */
  toolName: string | null;
  /** 요약 화면 도구 버전 */
  toolVer: string | null;
  /** env 태스크가 실제 적용한 환경 변수 (요약 표에 사용) */
  envApplied: Record<string, string> | null;
}

/** 프론트 태스크 런타임 상태 */
export type TaskStatus = "pending" | "running" | "done" | "error";
