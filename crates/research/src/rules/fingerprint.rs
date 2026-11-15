//! MC-A7: canonical JSON of a unit's input identity and its fingerprint `F(unit)`.

use serde_json::Value as Json;
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

/// A change to any stage's identity field list bumps this.
pub const RESEARCH_FINGERPRINT_VERSION: u32 = 2;

/// Canonical JSON cannot hold a float (decimals are MC-9 strings).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloatInIdentity(pub String);

impl std::fmt::Display for FloatInIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a unit identity holds the float {}", self.0)
    }
}

impl std::error::Error for FloatInIdentity {}

fn string(text: &str, out: &mut String) {
    let nfc: String = text.nfc().collect();
    out.push_str(&Json::String(nfc).to_string());
}

fn write(value: &Json, out: &mut String) -> Result<(), FloatInIdentity> {
    match value {
        Json::Null | Json::Bool(_) => out.push_str(&value.to_string()),
        Json::Number(n) => {
            if n.is_i64() || n.is_u64() {
                out.push_str(&n.to_string());
            } else {
                return Err(FloatInIdentity(n.to_string()));
            }
        }
        Json::String(s) => string(s, out),
        Json::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write(item, out)?;
            }
            out.push(']');
        }
        Json::Object(map) => {
            let mut entries: Vec<(String, &Json)> =
                map.iter().map(|(k, v)| (k.nfc().collect(), v)).collect();
            entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            out.push('{');
            for (i, (key, item)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                string(key, out);
                out.push(':');
                write(item, out)?;
            }
            out.push('}');
        }
    }
    Ok(())
}

/// Canonical JSON: keys sorted by UTF-8 bytes, no insignificant white space, strings NFC,
/// integers as integers, arrays in order; a float is an error.
pub fn canonical(value: &Json) -> Result<String, FloatInIdentity> {
    let mut out = String::new();
    write(value, &mut out)?;
    Ok(out)
}

/// `F(unit)` = lowercase hex SHA-256 of the canonical form of `{"stage": n, "unit": …, "v": V}`
/// with `V` = [`RESEARCH_FINGERPRINT_VERSION`].
pub fn fingerprint(stage: u8, unit: &Json) -> Result<String, FloatInIdentity> {
    let text = canonical(&serde_json::json!({
        "stage": stage,
        "unit": unit,
        "v": RESEARCH_FINGERPRINT_VERSION,
    }))?;
    Ok(sha256_hex(text.as_bytes()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a04_c7a_fingerprint_bytes() {
        assert_eq!(
            fingerprint(2, &json!({"rev": 3, "cover": 7})).unwrap(),
            sha256_hex(br#"{"stage":2,"unit":{"cover":7,"rev":3},"v":2}"#)
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn a04_c7b_nfd_and_nfc_equal() {
        assert_eq!(
            fingerprint(1, &json!({"maker": "Børge Mo\u{0308}gensen"})).unwrap(),
            fingerprint(1, &json!({"maker": "Børge M\u{00f6}gensen"})).unwrap()
        );
    }

    #[test]
    fn a04_c7c_array_order_matters() {
        assert_ne!(
            fingerprint(6, &json!({"ids": [1, 2]})).unwrap(),
            fingerprint(6, &json!({"ids": [2, 1]})).unwrap()
        );
    }

    #[test]
    fn a04_c7d_float_is_an_error() {
        assert!(canonical(&json!({"long_edge": 1.5})).is_err());
    }
}
