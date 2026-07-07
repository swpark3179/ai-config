// ============================================================
// WinINET 설정 변경 통지
// 레지스트리로 프록시를 바꿔도 실행 중인 앱(Edge·Chrome 포함)은 모르므로,
// InternetSetOption 으로 "설정 변경됨 + 다시 읽어라"를 명시적으로 알린다.
// ============================================================

const INTERNET_OPTION_REFRESH: u32 = 37;
const INTERNET_OPTION_SETTINGS_CHANGED: u32 = 39;

#[link(name = "wininet")]
extern "system" {
    fn InternetSetOptionW(
        hinternet: *mut core::ffi::c_void,
        dwoption: u32,
        lpbuffer: *mut core::ffi::c_void,
        dwbufferlength: u32,
    ) -> i32;
}

pub fn refresh_settings() {
    unsafe {
        InternetSetOptionW(
            std::ptr::null_mut(),
            INTERNET_OPTION_SETTINGS_CHANGED,
            std::ptr::null_mut(),
            0,
        );
        InternetSetOptionW(
            std::ptr::null_mut(),
            INTERNET_OPTION_REFRESH,
            std::ptr::null_mut(),
            0,
        );
    }
}
