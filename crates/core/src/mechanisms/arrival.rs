use sha2::{Digest, Sha256};
use std::path::Path;

pub fn path_key(canonical: &str) -> String {
    Sha256::digest(canonical.as_bytes())
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn is_candidate(path: &Path, capture_root: &Path, raw_extensions: &[&str]) -> bool {
    if !path.is_file() {
        return false;
    }
    let Ok(relative) = path.strip_prefix(capture_root) else {
        return false;
    };
    let components: Vec<_> = relative.components().collect();
    if components.is_empty() || components.len() > 2 {
        return false;
    }
    if components.iter().any(|part| {
        let name = part.as_os_str().to_string_lossy();
        name.starts_with('.') || name == "CaptureOne"
    }) {
        return false;
    }
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            raw_extensions
                .iter()
                .any(|accepted| ext.eq_ignore_ascii_case(accepted))
        })
}

pub fn attribute(
    parent_dir: &Path,
    capture_root: &Path,
    sessions_by_folder_key: &[(String, i64)],
    active_session: Option<i64>,
) -> i64 {
    if let Ok(relative) = parent_dir.strip_prefix(capture_root)
        && relative.components().count() == 1
    {
        let key = relative.to_string_lossy();
        if key == "_Unassigned" {
            return 1;
        }
        let matches: Vec<_> = sessions_by_folder_key
            .iter()
            .filter(|(folder, _)| folder == &key)
            .collect();
        if matches.len() == 1 {
            return matches[0].1;
        }
    }
    active_session.unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mc2_attribution_order() {
        let root = Path::new("/Capture");
        let folders = [("K1".into(), 2)];
        assert_eq!(
            attribute(Path::new("/Capture/_Unassigned"), root, &folders, Some(3)),
            1
        );
        assert_eq!(
            attribute(Path::new("/Capture/K1"), root, &folders, Some(3)),
            2
        );
        assert_eq!(
            attribute(Path::new("/Capture/Other"), root, &folders, Some(3)),
            3
        );
        assert_eq!(attribute(root, root, &folders, None), 1);
    }
}
