use psm_infra::paths::AppPaths;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

pub fn installed_sidecar(paths: &AppPaths) -> Option<PathBuf> {
    let root = paths.root.join("camera/sony");
    let mut versions: Vec<_> = fs::read_dir(root)
        .ok()?
        .flatten()
        .filter(|e| e.path().is_dir())
        .collect();
    versions.sort_by_key(|e| e.file_name());
    versions
        .into_iter()
        .rev()
        .map(|entry| entry.path().join("camera-sidecar"))
        .find(|path| {
            fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
}
