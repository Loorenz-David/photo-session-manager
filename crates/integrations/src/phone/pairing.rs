//! PH1: one pairing per phone. A pairing's token is 32 random bytes as unpadded base64url (43
//! characters), returned once by [`PhonePairings::create`]; only its lowercase hex SHA-256 is
//! stored (`phone_pairings.token_sha256`), so it can never be read again. Pairings are independent.
use psm_core::{
    Error, Result,
    domain::live::PhonePairing,
    ports::{clock::Clock, store::Store},
};
use sha2::{Digest, Sha256};

/// A pairing just made: its token is shown once (in the QR code and the URL).
#[derive(Debug, Clone)]
pub struct CreatedPairing {
    pub id: i64,
    pub name: String,
    pub token: String,
}

/// The pairings of the phones allowed to use the phone server.
pub struct PhonePairings<'a> {
    store: &'a dyn Store,
    clock: &'a dyn Clock,
}

/// The lowercase hex SHA-256 of a token, as `phone_pairings.token_sha256` holds it.
pub fn token_sha256(token: &str) -> String {
    Sha256::digest(token.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Unpadded base64url (RFC 4648 §5).
fn base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, b)| n | (u32::from(*b) << (16 - 8 * i)));
        let chars = chunk.len() + 1;
        for i in 0..chars {
            out.push(char::from(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize]));
        }
    }
    out
}

impl<'a> PhonePairings<'a> {
    pub fn new(store: &'a dyn Store, clock: &'a dyn Clock) -> Self {
        Self { store, clock }
    }

    /// A new pairing named `name` (trimmed), or "Phone <n>" (n = pairings ever made + 1).
    pub fn create(&self, name: Option<&str>) -> Result<CreatedPairing> {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).map_err(|e| Error::InvalidData(e.to_string()))?;
        let token = base64url(&bytes);
        let name = match name.map(str::trim).filter(|n| !n.is_empty()) {
            Some(name) => name.to_owned(),
            None => format!("Phone {}", self.store.phone_pairings_ever()? + 1),
        };
        let id =
            self.store
                .phone_pairing_insert(&name, &token_sha256(&token), self.clock.now_ms())?;
        Ok(CreatedPairing { id, name, token })
    }

    /// The live pairing of `token` (its `last_seen_at` now), or `None` (unknown or revoked).
    pub fn verify(&self, token: &str) -> Result<Option<PhonePairing>> {
        if token.is_empty() {
            return Ok(None);
        }
        self.store
            .phone_pairing_seen(&token_sha256(token), self.clock.now_ms())
    }

    /// The pairing's token stops working; the phone server closes its sockets (PH1).
    pub fn revoke(&self, id: i64) -> Result<()> {
        self.store.phone_pairing_revoke(id, self.clock.now_ms())
    }

    pub fn rename(&self, id: i64, name: &str) -> Result<()> {
        let name = name.trim();
        if name.is_empty() {
            return Err(Error::InvalidValue("name".into()));
        }
        self.store.phone_pairing_rename(id, name)
    }

    /// The pairings not revoked, oldest first (no token: it is never stored).
    pub fn list(&self) -> Result<Vec<PhonePairing>> {
        self.store.phone_pairings()
    }
}
