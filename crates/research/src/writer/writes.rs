//! W2·U4: the writer's one door to the form (`master_plan_writer.md` §5 WC4 items 2–5; HW1;
//! 08B MC-A1's write table unchanged). It writes only the run's requestable text keys (fail
//! closed: research's `never_written` still refuses all of them, so research and the writer own
//! disjoint keys), reads the stored state in the writing transaction, and never writes
//! `items.title`.

use psm_core::{
    Result,
    domain::attribute::{AiConfidence, AiStatus, Definition, Provenance, Value},
    mechanisms::values::canonicalize,
    ports::store::StoreTx,
    services::attributes::builtin_definitions,
};
use serde_json::Value as Json;

use crate::{
    rules::write_table::{WriteDecision, decide_write},
    writer::note::PRODUCT_TITLE,
    writes::{WriteOutcome, cleared_set, same_content},
};

/// The integration whose title base makes an item's title the shop's (the rule `form_get` uses).
use psm_core::services::attributes::LINK_INTEGRATION as TITLE_LINK_INTEGRATION;

/// WC4.3: the stored state of a text key. For any key but `product.title`: its row. For
/// `product.title`: its attribute row if one exists; else none when the item's title is blank;
/// else a live row holding the title, from the shop (`integration`) when the item's link title
/// base equals the title, otherwise typed (`human`).
pub fn stored_state(
    row: Option<&Value>,
    item_id: i64,
    key: &str,
    item_title: &str,
    title_base: Option<&str>,
) -> Option<Value> {
    if key != PRODUCT_TITLE || row.is_some() {
        return row.cloned();
    }
    if item_title.trim().is_empty() {
        return None;
    }
    Some(Value {
        item_id,
        key: key.to_owned(),
        value: Json::String(item_title.to_owned()).to_string(),
        valid: true,
        provenance: if title_base == Some(item_title) {
            Provenance::Integration
        } else {
            Provenance::Human
        },
        ai_confidence: None,
        ai_status: None,
        ai_reason: None,
        ai_sources: None,
        ai_prior: None,
        updated_at: 0,
    })
}

/// How the writer's trace words an outcome (HW1 refusals in the table's words).
pub fn outcome_text(outcome: &WriteOutcome) -> String {
    match outcome {
        WriteOutcome::MatchesCleared => "matches a text you dismissed".to_owned(),
        WriteOutcome::AlreadyHolds => "already holds this text".to_owned(),
        WriteOutcome::NotResearched => "not requested".to_owned(),
        other => other.text(),
    }
}

fn definitions(tx: &dyn StoreTx, integration: Option<&str>) -> Result<Vec<Definition>> {
    let mut all = builtin_definitions();
    if let Some(integration) = integration {
        all.extend(tx.attribute_definitions(integration)?);
    }
    Ok(all)
}

/// Writes one text through MC-A1's table inside the caller's transaction (WC4.4, WC4.5).
/// `allowed` is the run's requestable text keys; any other key is not written (fail closed).
#[allow(clippy::too_many_arguments)]
pub(crate) fn write_text(
    tx: &mut dyn StoreTx,
    now: i64,
    integration: Option<&str>,
    item_id: i64,
    key: &str,
    value: &Json,
    confidence: AiConfidence,
    reason: &str,
    allowed: &[String],
) -> Result<WriteOutcome> {
    if !allowed.iter().any(|k| k == key) {
        return Ok(WriteOutcome::NotResearched);
    }
    let all = definitions(tx, integration)?;
    let Some(definition) = all.iter().find(|d| d.key == key) else {
        return Ok(WriteOutcome::NotResearched);
    };
    let item = tx.item(item_id)?;
    let row = tx
        .attribute_values(item_id)?
        .into_iter()
        .find(|v| v.key == key);
    let title_base = if key == PRODUCT_TITLE && row.is_none() {
        tx.item_title_base(item_id, TITLE_LINK_INTEGRATION)?
    } else {
        None
    };
    let stored = stored_state(
        row.as_ref(),
        item_id,
        key,
        &item.title,
        title_base.as_deref(),
    );
    let cleared = cleared_set(tx, stored.as_ref(), item_id, key)?;
    let ai_prior = match decide_write(definition, stored.as_ref(), value, &cleared) {
        WriteDecision::Write { ai_prior } => ai_prior,
        WriteDecision::MatchesCleared => return Ok(WriteOutcome::MatchesCleared),
        WriteDecision::AlreadyHolds => return Ok(WriteOutcome::AlreadyHolds),
        WriteDecision::Refused(why) => return Ok(WriteOutcome::Refused(why.to_owned())),
    };
    let Ok(canonical) = canonicalize(definition.core_type, definition, value.clone()) else {
        return Ok(WriteOutcome::Refused(
            "the text is not valid for this field".to_owned(),
        ));
    };
    let written = Value {
        item_id,
        key: key.to_owned(),
        value: canonical.0,
        valid: true,
        provenance: Provenance::Ai,
        ai_confidence: Some(confidence),
        ai_status: Some(AiStatus::Suggested),
        ai_reason: Some(reason.to_owned()),
        ai_sources: Some(Vec::new()),
        ai_prior,
        updated_at: now,
    };
    if row.as_ref().is_some_and(|s| same_content(s, &written)) {
        return Ok(WriteOutcome::Unchanged);
    }
    tx.put_attribute_value(&written)?;
    tx.set_item_activity(item_id, now)?;
    Ok(WriteOutcome::Written(written))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(key: &str, provenance: Provenance) -> Value {
        Value {
            item_id: 1,
            key: key.into(),
            value: "\"Stored\"".into(),
            valid: true,
            provenance,
            ai_confidence: None,
            ai_status: None,
            ai_reason: None,
            ai_sources: None,
            ai_prior: None,
            updated_at: 5,
        }
    }

    #[test]
    fn wc4_3_a_text_key_other_than_the_title_is_its_row() {
        let stored = row("product.description", Provenance::Integration);
        assert_eq!(
            stored_state(
                Some(&stored),
                1,
                "product.description",
                "Chair",
                Some("Chair")
            ),
            Some(stored)
        );
        assert_eq!(
            stored_state(None, 1, "product.description", "Chair", None),
            None
        );
    }

    #[test]
    fn wc4_3_a_title_row_wins_over_the_items_title() {
        let stored = row(PRODUCT_TITLE, Provenance::Ai);
        assert_eq!(
            stored_state(Some(&stored), 1, PRODUCT_TITLE, "Chair", None),
            Some(stored)
        );
    }

    #[test]
    fn wc4_3_a_blank_title_is_no_stored_state() {
        assert_eq!(stored_state(None, 1, PRODUCT_TITLE, "  ", Some("  ")), None);
    }

    #[test]
    fn wc4_3_the_title_is_the_shops_when_it_equals_the_link_title_base_else_typed() {
        let from_shop =
            stored_state(None, 1, PRODUCT_TITLE, "Shop chair", Some("Shop chair")).unwrap();
        assert_eq!(from_shop.provenance, Provenance::Integration);
        assert_eq!(from_shop.value, "\"Shop chair\"");
        assert!(from_shop.valid && from_shop.is_live());
        let typed = stored_state(None, 1, PRODUCT_TITLE, "Chair", Some("Shop chair")).unwrap();
        assert_eq!(typed.provenance, Provenance::Human);
        let unlinked = stored_state(None, 1, PRODUCT_TITLE, "Chair", None).unwrap();
        assert_eq!(unlinked.provenance, Provenance::Human);
    }
}
