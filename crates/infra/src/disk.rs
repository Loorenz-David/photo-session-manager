//! Free space on the disk holding a folder (P56 C3(d): no full-size master render below 5 GB).

use std::{ffi::CStr, io, os::unix::ffi::OsStrExt, path::Path};

/// Space left for this user on the disk holding a folder, and a name for that disk the
/// photographer recognises.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeSpace {
    pub available_bytes: u64,
    /// "the startup disk" for the system volume, else the volume's name (its mount folder name).
    pub disk: String,
}

/// `statfs` of the nearest existing ancestor of `path`.
pub fn free_space(path: &Path) -> io::Result<FreeSpace> {
    let mut probe = path;
    while !probe.exists() {
        probe = probe
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no existing folder"))?;
    }
    let mut c_path = probe.as_os_str().as_bytes().to_vec();
    c_path.push(0);
    let mut stat: libc::statfs = unsafe { std::mem::zeroed() };
    // SAFETY: `c_path` is NUL-terminated and `stat` is a valid out-pointer for the call.
    if unsafe { libc::statfs(c_path.as_ptr().cast(), &mut stat) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: the kernel fills `f_mntonname` with a NUL-terminated mount point.
    let mount = unsafe { CStr::from_ptr(stat.f_mntonname.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    Ok(FreeSpace {
        available_bytes: stat.f_bavail.saturating_mul(u64::from(stat.f_bsize)),
        disk: disk_name(&mount),
    })
}

fn disk_name(mount: &str) -> String {
    if mount == "/" || mount == "/System/Volumes/Data" {
        return "the startup disk".into();
    }
    let name = Path::new(mount)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| mount.to_string());
    format!("the disk \u{201c}{name}\u{201d}")
}
