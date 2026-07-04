fn main() {
    // requireAdministrator manifest를 임베드한다.
    // 주의: embed_resource 등으로 manifest를 이중 임베드하면 "duplicate resource"
    // 빌드 오류(tauri-apps/tauri#10154)가 발생하므로 이 경로 하나만 사용한다.
    let windows = tauri_build::WindowsAttributes::new().app_manifest(include_str!("manifest.xml"));
    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
        .expect("failed to run tauri-build");
}
