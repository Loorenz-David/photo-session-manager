use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

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

/// The first archive entry whose suffix is a manifest path wins. Names are checked before any
/// write, so a hostile zip cannot escape the temporary install folder.
pub fn verify(
    archive_entries: &[(String, Vec<u8>)],
    manifest: &Manifest,
) -> Result<Located, String> {
    if manifest.files.is_empty() {
        return Err("manifest".into());
    }
    for (name, _) in archive_entries {
        let normalized = name.replace('\\', "/");
        if normalized.starts_with('/') || normalized.split('/').any(|part| part == "..") {
            return Err(name.clone());
        }
    }
    let mut found = HashMap::new();
    for file in &manifest.files {
        let candidate = archive_entries.iter().find(|(name, _)| {
            let name = name.replace('\\', "/");
            name == file.path || name.ends_with(&format!("/{}", file.path))
        });
        let (_, bytes) = candidate.ok_or_else(|| file.path.clone())?;
        let actual = Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        if actual != file.sha256.to_lowercase() {
            return Err(file.path.clone());
        }
        found.insert(file.path.clone(), bytes.clone());
    }
    Ok(Located {
        files: manifest
            .files
            .iter()
            .map(|file| (file.path.clone(), found.remove(&file.path).unwrap()))
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn c2a_missing_and_c2b_hash_mismatch() {
        let manifest = Manifest {
            version: "2.02.00".into(),
            files: vec![ManifestFile {
                path: "CrAdapter/libssh2.dylib".into(),
                sha256: Sha256::digest(b"ok")
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect(),
            }],
        };
        assert_eq!(
            verify(&[], &manifest).unwrap_err(),
            "CrAdapter/libssh2.dylib"
        );
        assert_eq!(
            verify(
                &[("sdk/CrAdapter/libssh2.dylib".into(), b"wrong".to_vec())],
                &manifest
            )
            .unwrap_err(),
            "CrAdapter/libssh2.dylib"
        );
    }
}
