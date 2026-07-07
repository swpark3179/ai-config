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

/// WinINET(인터넷 옵션) 프록시 상태 — Edge·Chrome 이 실제로 사용하는 설정
pub struct WinInetProxy {
    pub auto_config_url: Option<String>,
    pub proxy_enable: bool,
    pub proxy_server: Option<String>,
    pub proxy_override: Option<String>,
}

/// 현재 사용자(HKCU)의 WinINET 프록시 설정을 읽는다.
pub fn read_wininet_proxy() -> WinInetProxy {
    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(INTERNET_SETTINGS)
        .ok();
    let get_s = |name: &str| -> Option<String> {
        key.as_ref()
            .and_then(|k| k.get_value::<String, _>(name).ok())
            .filter(|s| !s.trim().is_empty())
    };
    let enable: u32 = key
        .as_ref()
        .and_then(|k| k.get_value::<u32, _>("ProxyEnable").ok())
        .unwrap_or(0);
    WinInetProxy {
        auto_config_url: get_s("AutoConfigURL"),
        proxy_enable: enable == 1,
        proxy_server: get_s("ProxyServer"),
        proxy_override: get_s("ProxyOverride"),
    }
}

/// WinINET 수동 프록시를 설정한다 (ProxyEnable=1 + 서버 + 예외 목록).
pub fn write_wininet_proxy(server: &str, override_list: &str) -> Result<(), AppError> {
    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(INTERNET_SETTINGS, KEY_READ | KEY_SET_VALUE)
        .map_err(|e| AppError::Registry(format!("Internet Settings 열기 실패: {}", e)))?;
    key.set_value("ProxyEnable", &1u32)
        .map_err(|e| AppError::Registry(format!("ProxyEnable 쓰기 실패: {}", e)))?;
    key.set_value("ProxyServer", &server.to_string())
        .map_err(|e| AppError::Registry(format!("ProxyServer 쓰기 실패: {}", e)))?;
    key.set_value("ProxyOverride", &override_list.to_string())
        .map_err(|e| AppError::Registry(format!("ProxyOverride 쓰기 실패: {}", e)))
}

/// 임의 경로의 문자열 값 읽기 (브라우저 정책 검사 등)
pub fn read_string_at(root: winreg::HKEY, path: &str, name: &str) -> Option<String> {
    let key = RegKey::predef(root).open_subkey(path).ok()?;
    key.get_value::<String, _>(name)
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// 임의 경로의 DWORD 값 읽기 (브라우저 정책 검사 등)
pub fn read_dword_at(root: winreg::HKEY, path: &str, name: &str) -> Option<u32> {
    let key = RegKey::predef(root).open_subkey(path).ok()?;
    key.get_value::<u32, _>(name).ok()
}
