use std::{env, path::PathBuf, process::Command};

/// `bundle.externalBin` names `binaries/camera-sidecar`, and tauri-build checks at compile time that
/// `binaries/camera-sidecar-<target triple>` exists. On a machine that has never built the sidecar
/// (a clean checkout, CI) the stub sidecar is built first, so `cargo test` and `tauri build` work
/// without the Sony SDK. An existing file is never rebuilt here: after a C++ edit run
/// `tools/sidecar/build.sh` (deleting the binary also triggers this build again).
fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let target = env::var("TARGET").unwrap_or_else(|_| "aarch64-apple-darwin".into());
    let sidecar = manifest_dir.join(format!("binaries/camera-sidecar-{target}"));
    if !sidecar.exists() {
        if Command::new("cmake").arg("--version").output().is_err() {
            panic!(
                "cmake is required to build the camera sidecar stub ({}). Install it (for example: brew install cmake) and build again.",
                sidecar.display()
            );
        }
        let script = manifest_dir.join("../../../tools/sidecar/build.sh");
        let status = Command::new("bash")
            .arg(&script)
            .arg("--stub")
            .status()
            .unwrap_or_else(|e| panic!("could not run {}: {e}", script.display()));
        if !status.success() {
            panic!(
                "building the camera sidecar stub failed ({}): run tools/sidecar/build.sh --stub to see why",
                script.display()
            );
        }
    }
    tauri_build::build()
}
