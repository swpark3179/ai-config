// ============================================================
// 레지스트리 읽기/쓰기/삭제 (winreg)
//  - HKLM 시스템 환경 변수: 관리자 권한 필요
//  - HKCU 사용자 환경 변수
//  - HKCU AutoConfigURL (proxy.pac 위치)
// ============================================================
use crate::error::AppError;
use winreg::enums::*;
use winreg::RegKey;

const HKLM_ENV: &str = "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment";
const HKCU_ENV: &str = "Environment";
const INTERNET_SETTINGS: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings";

/// 시스템(HKLM) 환경 변수 읽기
pub fn read_hklm_env(name: &str) -> Option<String> {
    let key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(HKLM_ENV).ok()?;
    key.get_value::<String, _>(name).ok()
}

/// 시스템(HKLM) 환경 변수 쓰기 (REG_SZ)
pub fn write_hklm_env(name: &str, value: &str) -> Result<(), AppError> {
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(HKLM_ENV, KEY_READ | KEY_SET_VALUE)
        .map_err(|e| AppError::Registry(format!("HKLM Environment 열기 실패: {}", e)))?;
    key.set_value(name, &value.to_string())
        .map_err(|e| AppError::Registry(format!("{} 쓰기 실패: {}", name, e)))
}

/// 사용자(HKCU) 환경 변수 읽기
pub fn read_hkcu_env(name: &str) -> Option<String> {
    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey(HKCU_ENV).ok()?;
    key.get_value::<String, _>(name).ok()
}

/// 사용자(HKCU) 환경 변수 삭제
pub fn delete_hkcu_env(name: &str) -> Result<(), AppError> {
    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(HKCU_ENV, KEY_READ | KEY_SET_VALUE)
        .map_err(|e| AppError::Registry(format!("HKCU Environment 열기 실패: {}", e)))?;
    key.delete_value(name)
        .map_err(|e| AppError::Registry(format!("{} 삭제 실패: {}", name, e)))
}

/// 브라우저 자동 구성 스크립트(proxy.pac) 위치를 읽는다.
pub fn read_autoconfig_url() -> Option<String> {
    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(INTERNET_SETTINGS)
        .ok()?;
    key.get_value::<String, _>("AutoConfigURL")
        .ok()
        .filter(|s| !s.trim().is_empty())
}
