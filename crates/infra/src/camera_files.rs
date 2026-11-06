//! 10E MC-33 R-I27: the capture ledger's file operations. The rename never replaces an existing
//! file (`renamex_np(RENAME_EXCL)`); `std::fs::rename` silently replaces on macOS, so it is never
//! used for camera files, and there is no fallback (copy, link) on any other error.
use psm_core::{
    Result,
    ports::camera::{DeliveryFs, RenameOutcome},
};
use std::{ffi::CString, os::unix::ffi::OsStrExt, path::Path};

fn c_path(path: &Path) -> Result<CString> {
    CString::new(path.as_os_str().as_bytes())
        .map_err(|_| psm_core::Error::InvalidValue("path contains a NUL byte".into()))
}

/// Moves `from` to `to` unless `to` exists: `Exists` on `EEXIST`, every other failure
/// (`ENOTSUP` off APFS, `EXDEV`, `ENOENT`, …) is an error.
pub fn rename_exclusive(from: &Path, to: &Path) -> Result<RenameOutcome> {
    let (from_c, to_c) = (c_path(from)?, c_path(to)?);
    // SAFETY: both arguments are valid NUL-terminated paths that outlive the call.
    let status = unsafe { libc::renamex_np(from_c.as_ptr(), to_c.as_ptr(), libc::RENAME_EXCL) };
    if status == 0 {
        return Ok(RenameOutcome::Moved);
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::EEXIST) {
        Ok(RenameOutcome::Exists)
    } else {
        Err(error.into())
    }
}

/// The production `DeliveryFs`.
#[derive(Debug, Default, Clone, Copy)]
pub struct CameraFiles;

impl DeliveryFs for CameraFiles {
    fn rename_exclusive(&self, from: &Path, to: &Path) -> Result<RenameOutcome> {
        rename_exclusive(from, to)
    }
    fn create_dir_all(&self, dir: &Path) -> Result<()> {
        Ok(std::fs::create_dir_all(dir)?)
    }
    fn file_size(&self, path: &Path) -> Result<u64> {
        Ok(std::fs::metadata(path)?.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mc33_rename_exclusive_never_replaces() {
        let dir = tempfile::tempdir().unwrap();
        let (a, b, c) = (
            dir.path().join("a"),
            dir.path().join("b"),
            dir.path().join("c"),
        );
        std::fs::write(&a, b"first").unwrap();
        std::fs::write(&b, b"second").unwrap();
        assert_eq!(rename_exclusive(&a, &b).unwrap(), RenameOutcome::Exists);
        assert_eq!(std::fs::read(&a).unwrap(), b"first");
        assert_eq!(std::fs::read(&b).unwrap(), b"second");
        assert_eq!(rename_exclusive(&a, &c).unwrap(), RenameOutcome::Moved);
        assert!(!a.exists());
        assert_eq!(std::fs::read(&c).unwrap(), b"first");
    }
}
