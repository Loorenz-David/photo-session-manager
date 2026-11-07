use psm_infra::paths::AppPaths;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

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
            .then(|| InstalledSupport {
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
