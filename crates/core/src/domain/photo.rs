use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Photo {
    pub id: i64,
    pub uid: String,
    pub session_id: i64,
    pub raw_path: String,
    pub raw_hash: Option<String>,
    pub integrity: Integrity,
    pub provider_ref: Option<String>,
    pub original_preview_path: Option<String>,
    pub adjusted_preview_path: Option<String>,
    pub orientation: Option<i64>,
    pub classification: Classification,
    pub position: Option<i64>,
    pub captured_at: Option<i64>,
    pub source: PhotoSource,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Integrity {
    Ok,
    RawChanged,
    RawMissing,
}

impl Integrity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::RawChanged => "raw_changed",
            Self::RawMissing => "raw_missing",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    Unreviewed,
    Main,
    Condition,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhotoSource {
    Capture,
    Import,
}

impl PhotoSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Capture => "capture",
            Self::Import => "import",
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewPhoto {
    pub session_id: i64,
    pub raw_path: String,
    pub raw_hash: Option<String>,
    pub captured_at: Option<i64>,
    pub source: PhotoSource,
    pub created_at: i64,
}
