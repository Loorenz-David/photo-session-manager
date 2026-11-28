//! Where the phone app's files are served from (`master_plan_live.md` §4.5): `PSM_PHONE_WEB` when
//! set; else `<Resources>/phone-web` of the packaged app when it exists; else, in a debug build,
//! `apps/desktop/dist`.
use std::{ffi::OsString, path::PathBuf};

/// The web root, from the packaged app's resource folder, the `PSM_PHONE_WEB` value and the debug
/// default (`None` in a release build). `None` when nothing applies.
pub fn resolve(
    packaged_resources: Option<PathBuf>,
    env: Option<OsString>,
    debug_default: Option<PathBuf>,
) -> Option<PathBuf> {
    if let Some(dir) = env.filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(dir));
    }
    if let Some(dir) = packaged_resources
        .map(|r| r.join("phone-web"))
        .filter(|d| d.is_dir())
    {
        return Some(dir);
    }
    debug_default
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_environment_wins() {
        let resources = tempfile::tempdir().unwrap();
        std::fs::create_dir(resources.path().join("phone-web")).unwrap();
        assert_eq!(
            resolve(
                Some(resources.path().into()),
                Some("/tmp/web".into()),
                Some("/dev/dist".into())
            ),
            Some(PathBuf::from("/tmp/web"))
        );
    }

    #[test]
    fn the_packaged_folder_when_present() {
        let resources = tempfile::tempdir().unwrap();
        std::fs::create_dir(resources.path().join("phone-web")).unwrap();
        assert_eq!(
            resolve(
                Some(resources.path().into()),
                None,
                Some("/dev/dist".into())
            ),
            Some(resources.path().join("phone-web"))
        );
        assert_eq!(
            resolve(Some(resources.path().into()), Some(OsString::new()), None),
            Some(resources.path().join("phone-web"))
        );
    }

    #[test]
    fn else_the_debug_default_or_nothing() {
        let resources = tempfile::tempdir().unwrap();
        assert_eq!(
            resolve(
                Some(resources.path().into()),
                None,
                Some("/dev/dist".into())
            ),
            Some(PathBuf::from("/dev/dist"))
        );
        assert_eq!(resolve(None, None, None), None);
    }
}
