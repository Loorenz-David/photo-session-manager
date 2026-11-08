use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Seek, SeekFrom};
use zip::ZipArchive;

pub const SONY_JSON: &str =
    include_str!("../../../../../native/camera-sidecar/manifest/sony-2.02.00.json");
pub const SUPPORTED_SDK_VERSIONS: &[&str] = &["2.02.00"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub files: Vec<ManifestFile>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFile {
    pub path: String,
    pub sha256: String,
}
#[derive(Debug)]
pub struct Located {
    pub files: Vec<(String, Vec<u8>)>,
}

pub fn sony() -> Manifest {
    serde_json::from_str(SONY_JSON).expect("compiled Sony manifest")
}

fn checked_name(name: &str) -> Result<String, String> {
    let normalized = name.replace('\\', "/");
    if normalized.starts_with('/') || normalized.split('/').any(|part| part == "..") {
        return Err(format!("invalid {name}"));
    }
    Ok(normalized)
}

fn matching<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    path: &str,
) -> Result<Option<Vec<u8>>, String> {
    let mut result = None;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).map_err(|_| format!("invalid {path}"))?;
        let name = checked_name(entry.name())?;
        if result.is_none() && (name == path || name.ends_with(&format!("/{path}"))) {
            let mut bytes = Vec::new();
            entry
                .read_to_end(&mut bytes)
                .map_err(|_| format!("invalid {path}"))?;
            result = Some(bytes);
        }
    }
    Ok(result)
}

/// Reads only manifest files. A nested zip is copied as a compressed archive to a temporary
/// seekable file; its other members are never extracted.
pub fn verify<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    manifest: &Manifest,
) -> Result<Located, String> {
    if manifest.files.is_empty() {
        return Err("invalid manifest".into());
    }
    let mut nested_archives = Vec::new();
    for index in 0..zip.len() {
        let mut entry = zip
            .by_index(index)
            .map_err(|_| "invalid archive".to_string())?;
        let name = checked_name(entry.name())?;
        if !name.to_ascii_lowercase().ends_with(".zip") {
            continue;
        }
        let mut nested_file = tempfile::tempfile().map_err(|_| format!("invalid {name}"))?;
        std::io::copy(&mut entry, &mut nested_file).map_err(|_| format!("invalid {name}"))?;
        nested_file
            .seek(SeekFrom::Start(0))
            .map_err(|_| format!("invalid {name}"))?;
        nested_archives.push(ZipArchive::new(nested_file).map_err(|_| format!("invalid {name}"))?);
    }
    let mut files = Vec::new();
    for file in &manifest.files {
        let mut bytes = matching(zip, &file.path)?;
        if bytes.is_none() {
            for nested in &mut nested_archives {
                bytes = matching(nested, &file.path)?;
                if bytes.is_some() {
                    break;
                }
            }
        }
        let bytes = bytes.ok_or_else(|| format!("missing {}", file.path))?;
        let actual = Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        if actual != file.sha256.to_lowercase() {
            return Err(format!("changed {}", file.path));
        }
        files.push((file.path.clone(), bytes));
    }
    Ok(Located { files })
}
