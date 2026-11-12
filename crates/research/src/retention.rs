//! MC-A11 retention: a call record, and every image file only expired records reference, is
//! deleted once `now ≥ started_at + record_days × 86 400 000` ms (elapsed time, as 10F's
//! retention mechanism). Runs, steps, feedback, outbound rows and stage outputs are kept
//! (R-IA26). The app runs it at launch and hourly (R7·U2).

use std::collections::BTreeSet;

use psm_core::Result;
use serde_json::Value as Json;

use crate::{service::ResearchService, store};

/// One day in ms.
pub const DAY_MS: i64 = 86_400_000;

/// Every `image_sha256` value anywhere in a recorded request or response text.
pub fn image_refs(text: &str) -> Vec<String> {
    fn walk(value: &Json, out: &mut Vec<String>) {
        match value {
            Json::Object(map) => {
                for (key, inner) in map {
                    match (key.as_str(), inner) {
                        ("image_sha256", Json::String(sha)) => out.push(sha.clone()),
                        _ => walk(inner, out),
                    }
                }
            }
            Json::Array(items) => items.iter().for_each(|item| walk(item, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    if let Ok(value) = serde_json::from_str::<Json>(text) {
        walk(&value, &mut out);
    }
    out
}

fn call_refs(call: &store::CallRow) -> Vec<String> {
    let mut refs = image_refs(&call.request);
    if let Some(response) = &call.response {
        refs.extend(image_refs(response));
    }
    refs
}

impl ResearchService {
    /// The call records older than `record_days` at `now_ms`, and the image files only they
    /// referenced, are deleted. Files a remaining call references stay.
    pub fn purge_expired(&self, now_ms: i64) -> Result<()> {
        let days = self.settings().record_days.max(2);
        let cutoff = now_ms.saturating_sub(days.saturating_mul(DAY_MS));
        let orphaned: BTreeSet<String> = self.store.research_transaction(|tx| {
            let conn = tx.conn();
            let expired = store::calls_started_by(conn, cutoff)?;
            if expired.is_empty() {
                return Ok(BTreeSet::new());
            }
            let mut candidates: BTreeSet<String> = BTreeSet::new();
            for call in &expired {
                candidates.extend(call_refs(call));
                store::delete_call(conn, call.id)?;
            }
            for call in store::calls(conn, None)? {
                for sha in call_refs(&call) {
                    candidates.remove(&sha);
                }
            }
            Ok(candidates)
        })?;
        for sha in orphaned {
            self.files.remove(&sha)?;
        }
        Ok(())
    }
}
