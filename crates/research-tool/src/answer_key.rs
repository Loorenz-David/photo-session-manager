//! The owner's test pieces and answer key, and the R1 scoring definitions (08A §12A):
//! own listing, naming result, the corrected-key bucket. Host matching here is a temporary
//! private helper of the tool; R9·U1 replaces it with `psm_research::rules::sites`.

use serde_json::Value as Json;
use std::path::Path;

pub const TEST_FILE: &str = "docs/test-items/beyo_test_samples_1.json";
pub const CORRECTIONS_FILE: &str = "docs/test-items/answer_key_corrections.json";

/// The attribution metafields of the test file (08A §6: the shop's designer and manufacturer keys).
pub const ATTRIBUTION_FIELDS: [&str; 2] = ["designer", "manufacturer"];

/// `answer_key_corrections.json` `scoring_rules.hedged_attribution`: pieces whose attribution is
/// hedged ("most likely by", "in the style of").
pub const HEDGED_PIECES: [u32; 4] = [2, 26, 37, 38];

/// The hedged name of a piece whose metafields are empty: piece 26, "most likely by Holger
/// Christensens Møbelmagasiner" (its description and `data_notes`).
pub const HEDGED_FROM_TEXT: [(u32, &str, &str); 1] =
    [(26, "manufacturer", "Holger Christensens Møbelmagasiner")];

#[derive(Debug, Clone)]
pub struct Piece {
    pub n: u32,
    pub test_group: String,
    pub attribution: String,
    pub handle: String,
    pub title: String,
    pub description: String,
    pub product_type: String,
    pub published: bool,
    pub url: String,
    pub images: Vec<String>,
    pub condition_images: Vec<String>,
    pub metafields: Json,
}

impl Piece {
    /// "with attribution" in the file's own terms (`attribution` ≠ `none`).
    pub fn has_attribution(&self) -> bool {
        self.attribution != "none"
    }
}

pub fn load_pieces(path: &Path) -> Result<Vec<Piece>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let json: Json = serde_json::from_str(&text).map_err(|e| format!("test file: {e}"))?;
    let s = |v: &Json| v.as_str().unwrap_or("").to_owned();
    let list = |v: &Json| {
        v.as_array()
            .into_iter()
            .flatten()
            .filter_map(|x| x.as_str().map(str::to_owned))
            .collect::<Vec<_>>()
    };
    Ok(json
        .as_array()
        .into_iter()
        .flatten()
        .map(|p| Piece {
            n: p["n"].as_u64().unwrap_or(0) as u32,
            test_group: s(&p["test_group"]),
            attribution: s(&p["attribution"]),
            handle: s(&p["handle"]),
            title: s(&p["title"]),
            description: s(&p["description"]),
            product_type: s(&p["product_type"]),
            published: p["published_online"].as_bool().unwrap_or(false),
            url: s(&p["url"]),
            images: list(&p["images"]),
            condition_images: list(&p["condition_report_images"]),
            metafields: p["metafields"].clone(),
        })
        .collect())
}

#[derive(Debug, Clone, PartialEq)]
pub struct Correction {
    pub n: u32,
    pub field: String,
    pub key_value: Json,
    pub corrected_value: Json,
}

pub fn load_corrections(path: &Path) -> Result<Vec<Correction>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let json: Json = serde_json::from_str(&text).map_err(|e| format!("corrections file: {e}"))?;
    Ok(json["corrections"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|c| Correction {
            n: c["n"].as_u64().unwrap_or(0) as u32,
            field: c["field"].as_str().unwrap_or("").to_owned(),
            key_value: c["key_value"].clone(),
            corrected_value: c["corrected_value"].clone(),
        })
        .collect())
}

/// Which bucket a key value is scored in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Bucket {
    /// The test file's own metafield value.
    OwnerVerified,
    /// An entry of `answer_key_corrections.json` (scored against the corrected value).
    Corrected,
    /// A hedged name taken from the piece's text because its metafields are empty (piece 26).
    FromText,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyEntry {
    pub n: u32,
    pub field: String,
    pub value: Json,
    pub bucket: Bucket,
    pub hedged: bool,
}

/// Every scored key value of every piece: each non-empty metafield, with a correction replacing
/// the metafield's value (or supplying a value the file leaves empty) in the corrected bucket.
pub fn key_entries(pieces: &[Piece], corrections: &[Correction]) -> Vec<KeyEntry> {
    let mut out = Vec::new();
    for piece in pieces {
        let hedged = HEDGED_PIECES.contains(&piece.n);
        let corrected = |field: &str| {
            corrections
                .iter()
                .find(|c| c.n == piece.n && c.field == field)
        };
        for (field, value) in piece.metafields.as_object().into_iter().flatten() {
            if let Some(c) = corrected(field) {
                out.push(KeyEntry {
                    n: piece.n,
                    field: field.clone(),
                    value: c.corrected_value.clone(),
                    bucket: Bucket::Corrected,
                    hedged,
                });
            } else if !value.is_null() {
                out.push(KeyEntry {
                    n: piece.n,
                    field: field.clone(),
                    value: value.clone(),
                    bucket: Bucket::OwnerVerified,
                    hedged,
                });
            }
        }
        for c in corrections.iter().filter(|c| c.n == piece.n) {
            if piece.metafields.get(&c.field).is_none() {
                out.push(KeyEntry {
                    n: piece.n,
                    field: c.field.clone(),
                    value: c.corrected_value.clone(),
                    bucket: Bucket::Corrected,
                    hedged,
                });
            }
        }
        for (n, field, name) in HEDGED_FROM_TEXT {
            if n == piece.n && !out.iter().any(|e| e.n == n && e.field == field) {
                out.push(KeyEntry {
                    n,
                    field: field.to_owned(),
                    value: Json::String(name.to_owned()),
                    bucket: Bucket::FromText,
                    hedged: true,
                });
            }
        }
    }
    out
}

/// The attribution names of a piece (designer, manufacturer) with the bucket each is scored in.
pub fn key_names(n: u32, entries: &[KeyEntry]) -> Vec<(String, Bucket)> {
    entries
        .iter()
        .filter(|e| e.n == n && ATTRIBUTION_FIELDS.contains(&e.field.as_str()))
        .filter_map(|e| e.value.as_str().map(|s| (s.to_owned(), e.bucket)))
        .filter(|(s, _)| !s.trim().is_empty())
        .collect()
}

/// Lowercase, every character that is not a letter or a digit removed (punctuation and spaces).
pub fn fold(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// A naming result: `text` (a title or a page's text) contains one of `names`, compared
/// case-insensitively with punctuation ignored.
pub fn is_naming(text: &str, names: &[String]) -> bool {
    let hay = fold(text);
    names.iter().any(|name| {
        let needle = fold(name);
        !needle.is_empty() && hay.contains(&needle)
    })
}

/// `H(url)` without IDNA: lowercased host, trailing dot and port dropped; `None` for a non-http URL.
pub fn host(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return None;
    }
    let host = parsed
        .host_str()?
        .trim_end_matches('.')
        .to_ascii_lowercase();
    Some(host)
}

/// `h` equals `entry` or is one of its subdomains.
pub fn host_matches(h: &str, entry: &str) -> bool {
    h == entry || h.ends_with(&format!(".{entry}"))
}

/// The piece's own listing: a URL on a shop host (or a subdomain) whose path ends in
/// `/products/<handle>`, with or without a locale prefix (MC-A3, R1 scoring definitions).
pub fn is_own_listing(url: &str, handle: &str, shop_hosts: &[String]) -> bool {
    let Some(h) = host(url) else {
        return false;
    };
    if !shop_hosts.iter().any(|s| host_matches(&h, s)) {
        return false;
    }
    let Ok(parsed) = url::Url::parse(url) else {
        return false;
    };
    let path = parsed.path().trim_end_matches('/');
    path.ends_with(&format!("/products/{handle}"))
}

/// The site class of a URL against the shop hosts and the trusted list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SiteClass {
    Shop,
    Trusted(String),
    Other(String),
}

pub fn site_of(url: &str, shop_hosts: &[String], trusted: &[String]) -> SiteClass {
    let h = host(url).unwrap_or_default();
    if shop_hosts.iter().any(|s| host_matches(&h, s)) {
        return SiteClass::Shop;
    }
    if let Some(e) = trusted.iter().find(|e| host_matches(&h, e)) {
        return SiteClass::Trusted(e.clone());
    }
    SiteClass::Other(h.strip_prefix("www.").unwrap_or(&h).to_owned())
}

/// The owner's nine trusted sites in priority order (evidence doc, owner 2026-11-12).
pub const TRUSTED_SITES: [&str; 9] = [
    "etsy.com",
    "pamono.com",
    "bukowskis.com",
    "midcenturymobler.com",
    "vntg.com",
    "maleforsimports.com",
    "auktion.sweakronan.se",
    "vgliving.co",
    "auctionet.com",
];

pub fn trusted_sites() -> Vec<String> {
    TRUSTED_SITES.iter().map(|s| (*s).to_owned()).collect()
}
