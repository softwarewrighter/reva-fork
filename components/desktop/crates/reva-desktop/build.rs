//! The app's version is the release version (the repository's VERSION),
//! passed to Tauri on every build, so the app, its About menu and its
//! installers name the release, never tauri.conf.json's placeholder.
//! Tauri reads the override from `TAURI_CONFIG`: here, in tauri-build, and
//! in `generate_context!` (through `rustc-env`). A change to VERSION or to
//! the page in `dist` (which `generate_context!` embeds) rebuilds the app.

fn main() {
    let version = include_str!("../../../../VERSION").trim_ascii();
    println!("cargo:rerun-if-changed=../../../../VERSION");
    println!("cargo:rerun-if-changed=../../dist");
    let mut config: serde_json::Value = std::env::var("TAURI_CONFIG")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    config["version"] = version.into();
    let config = config.to_string();
    // SAFETY: a build script is single-threaded until tauri-build runs.
    unsafe { std::env::set_var("TAURI_CONFIG", &config) };
    println!("cargo:rustc-env=TAURI_CONFIG={config}");
    tauri_build::build()
}
