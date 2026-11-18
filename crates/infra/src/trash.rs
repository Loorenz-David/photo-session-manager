//! The macOS Trash (owner correction A17): `NSFileManager trashItemAtURL:resultingItemURL:error:`,
//! the same move Finder makes, so the owner can put a file back from the Trash. No Finder
//! scripting, no other app involved (H5).

use psm_core::{Error, Result, ports::trash::Trash};
use std::path::Path;

pub struct MacTrash;

impl Trash for MacTrash {
    fn trash(&self, path: &Path) -> Result<bool> {
        if std::fs::symlink_metadata(path).is_err() {
            return Ok(false);
        }
        move_to_trash(path)?;
        Ok(true)
    }
}

#[cfg(target_os = "macos")]
fn move_to_trash(path: &Path) -> Result<()> {
    use objc2_foundation::{NSFileManager, NSString, NSURL};
    let text = path
        .to_str()
        .ok_or_else(|| Error::InvalidData(format!("{} is not UTF-8", path.display())))?;
    let url = NSURL::fileURLWithPath(&NSString::from_str(text));
    let manager = NSFileManager::defaultManager();
    manager
        .trashItemAtURL_resultingItemURL_error(&url, None)
        .map_err(|error| {
            Error::InvalidData(format!(
                "{} could not be moved to the Trash: {}",
                path.display(),
                error.localizedDescription()
            ))
        })
}

#[cfg(not(target_os = "macos"))]
fn move_to_trash(path: &Path) -> Result<()> {
    Err(Error::InvalidData(format!(
        "{}: the Trash is only available on macOS",
        path.display()
    )))
}
