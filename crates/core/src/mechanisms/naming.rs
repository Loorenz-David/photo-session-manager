use crate::{Error, Result};
use unicode_normalization::UnicodeNormalization;

pub fn folder_key(sku: &str) -> Result<String> {
    let mut key: String = sku
        .nfc()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if key.starts_with(['.', '_']) {
        key.replace_range(..1, "-");
    }
    if key.is_empty() || key.len() > 100 {
        return Err(Error::InvalidValue(
            "sku folder key must be 1–100 characters".into(),
        ));
    }
    Ok(key)
}
