use psm_infra::paths::AppPaths;
use std::{
    collections::HashMap,
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};
pub mod manifest;
pub mod sony;

static TEST_MANIFESTS: OnceLock<Mutex<HashMap<PathBuf, manifest::Manifest>>> = OnceLock::new();
static TEST_BUNDLED: OnceLock<Mutex<HashMap<PathBuf, PathBuf>>> = OnceLock::new();
pub fn with_test_bundled(paths: &AppPaths, bundled: PathBuf) {
    TEST_BUNDLED
        .get_or_init(Default::default)
        .lock()
        .unwrap()
        .insert(paths.root.clone(), bundled);
}
pub fn with_test_manifest(paths: &AppPaths, spec: manifest::Manifest) {
    TEST_MANIFESTS
        .get_or_init(Default::default)
        .lock()
        .unwrap()
        .insert(paths.root.clone(), spec);
}
fn manifest_for(paths: &AppPaths) -> manifest::Manifest {
    TEST_MANIFESTS
        .get()
        .and_then(|m| m.lock().unwrap().get(&paths.root).cloned())
        .unwrap_or_else(manifest::sony)
}

/// An installed camera support package (10E §Camera support packages; master plan §6.11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledSupport {
    pub make: &'static str,
    pub version: String,
    pub sidecar_path: PathBuf,
    pub folder: PathBuf,
}

/// The newest `camera/sony/<version>/` folder holding an executable `camera-sidecar` (P46; the
/// manifest check is P50's). `None` → the Capture One camera adapter is used.
pub fn installed(paths: &AppPaths) -> Option<InstalledSupport> {
    let root = paths.root.join("camera/sony");
    let spec = manifest_for(paths);
    let mut versions: Vec<_> = fs::read_dir(root)
        .ok()?
        .flatten()
        .filter(|e| e.path().is_dir())
        .collect();
    versions.sort_by_key(|e| e.file_name());
    versions.into_iter().rev().find_map(|entry| {
        let folder = entry.path();
        let sidecar_path = folder.join("camera-sidecar");
        fs::metadata(&sidecar_path)
            .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            .then_some(())
            .filter(|_| entry.file_name().to_string_lossy() == spec.version)
            .filter(|_| spec.files.iter().all(|f| folder.join(&f.path).is_file()))
            .map(|_| InstalledSupport {
                make: "sony",
                version: entry.file_name().to_string_lossy().into_owned(),
                sidecar_path,
                folder,
            })
    })
}

pub fn installed_sidecar(paths: &AppPaths) -> Option<PathBuf> {
    installed(paths).map(|support| support.sidecar_path)
}
pub fn damaged(paths: &AppPaths) -> bool {
    let root = paths.root.join("camera/sony");
    let present = fs::read_dir(root).ok().is_some_and(|mut entries| {
        entries.any(|e| {
            e.ok().is_some_and(|e| {
                e.path().is_dir() && !e.file_name().to_string_lossy().starts_with('.')
            })
        })
    });
    present && installed(paths).is_none()
}
pub fn bundled_sidecar_path() -> Option<PathBuf> {
    let beside = std::env::current_exe()
        .ok()?
        .parent()?
        .join("camera-sidecar");
    if beside.is_file() {
        return Some(beside);
    }
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin");
    dev.is_file().then_some(dev)
}
pub fn bundled_for(paths: &AppPaths) -> Option<PathBuf> {
    TEST_BUNDLED
        .get()
        .and_then(|m| m.lock().unwrap().get(&paths.root).cloned())
        .or_else(bundled_sidecar_path)
}
