use crate::Result;
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path};

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut chunk = [0u8; 64 * 1024];
    loop {
        let size = file.read(&mut chunk)?;
        if size == 0 {
            break;
        }
        hash.update(&chunk[..size]);
    }
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
