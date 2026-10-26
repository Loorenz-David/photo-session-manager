use crate::paths::AppPaths;
use psm_core::{Error, Result};
use rusqlite::Connection;
use std::{fs, path::Path};

pub fn create(paths: &AppPaths) -> Result<()> {
    let dir = paths.root.join("backup");
    fs::create_dir_all(&dir)?;
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let mut target = dir.join(format!("session_manager-{stamp}.db"));
    let mut suffix = 1;
    while target.exists() {
        target = dir.join(format!("session_manager-{stamp}~{suffix:04}.db"));
        suffix += 1;
    }
    let source = Connection::open(paths.db()).map_err(db)?;
    source.backup("main", &target, None).map_err(db)?;
    prune(&dir, 30)?;
    Ok(())
}

pub fn prune(dir: &Path, keep: usize) -> Result<()> {
    let mut entries = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("session_manager-")
                && entry.path().extension().is_some_and(|ext| ext == "db")
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries.iter().take(entries.len().saturating_sub(keep)) {
        fs::remove_file(entry.path())?;
    }
    Ok(())
}

fn db(error: rusqlite::Error) -> Error {
    Error::InvalidData(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prune_keeps_newest_names() {
        let dir = tempfile::tempdir().unwrap();
        for n in 0..31 {
            fs::write(
                dir.path()
                    .join(format!("session_manager-20261026-{:06}.db", n)),
                b"db",
            )
            .unwrap();
        }
        prune(dir.path(), 30).unwrap();
        assert!(
            !dir.path()
                .join("session_manager-20261026-000000.db")
                .exists()
        );
        assert!(
            dir.path()
                .join("session_manager-20261026-000030.db")
                .exists()
        );
    }
}
