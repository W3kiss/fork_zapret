fn main() {
    // Release: manifest forces UAC elevation (WinDivert needs admin rights).
    // Debug (`tauri dev`): asInvoker, so cargo can launch the exe from a normal
    // terminal; the app then re-launches itself elevated (see lib.rs).
    let release = std::env::var("PROFILE").map(|p| p == "release").unwrap_or(false);
    let manifest = if release {
        include_str!("app.manifest")
    } else {
        include_str!("app.dev.manifest")
    };
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=app.dev.manifest");
    let windows = tauri_build::WindowsAttributes::new().app_manifest(manifest);
    let attrs = tauri_build::Attributes::new().windows_attributes(windows);
    tauri_build::try_build(attrs).expect("failed to run tauri-build");
}
