// ============================================================
// 애플리케이션 에러 — serde::Serialize 로 프론트 catch 에 문자열 전달
// ============================================================
use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("레지스트리 오류: {0}")]
    Registry(String),
    #[error("프로세스 오류: {0}")]
    Process(String),
    #[error("파일 오류: {0}")]
    Io(String),
    #[error("네트워크 오류: {0}")]
    Network(String),
    #[error("{0}")]
    Task(String),
    #[error("{0}")]
    Unknown(String),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}
