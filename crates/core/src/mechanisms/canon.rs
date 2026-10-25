use crate::{Error, Result};
use std::path::Path;
use unicode_normalization::UnicodeNormalization;

pub fn canon(path: &Path) -> Result<String> {
    if !path.is_absolute() {
        return Err(Error::RelativePath);
    }
    let name = path
        .file_name()
        .ok_or_else(|| Error::InvalidValue("missing file name".into()))?;
    let parent = path
        .parent()
        .ok_or_else(|| Error::InvalidValue("missing parent".into()))?;
    let parent = std::fs::canonicalize(parent)?;
    let wanted = name
        .to_str()
        .ok_or_else(|| Error::InvalidValue("non-UTF-8 path".into()))?;
    let wanted_nfc: String = wanted.nfc().collect();
    let mut disk_name = None;
    for entry in std::fs::read_dir(&parent)? {
        let entry = entry?;
        let entry_name = entry.file_name();
        let Some(entry_name) = entry_name.to_str() else {
            continue;
        };
        let normalized: String = entry_name.nfc().collect();
        if normalized == wanted_nfc {
            disk_name = Some(normalized);
            break;
        }
        if normalized.to_lowercase() == wanted_nfc.to_lowercase() && disk_name.is_none() {
            disk_name = Some(normalized);
        }
    }
    let name = disk_name.unwrap_or(wanted_nfc);
    let full = parent.join(name);
    let normalized: String = full
        .to_str()
        .ok_or_else(|| Error::InvalidValue("non-UTF-8 path".into()))?
        .nfc()
        .collect();
    Ok(normalized.trim_end_matches('/').to_owned())
}
