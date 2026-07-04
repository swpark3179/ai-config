// ============================================================
// 환경 변수 변경을 시스템에 알림 (WM_SETTINGCHANGE 브로드캐스트)
// setx 와 달리 레지스트리 직접 쓰기는 자동 통지되지 않으므로 명시적으로 브로드캐스트한다.
// 이후 새로 시작하는 프로세스가 변경된 환경 변수를 인식한다.
// ============================================================
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};

pub fn broadcast_env_change() {
    let mut param: Vec<u16> = "Environment".encode_utf16().collect();
    param.push(0);
    unsafe {
        let _ = SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(param.as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            5000,
            None,
        );
    }
}
