//! W2·U1: exactly what the writer sees (`master_plan_writer.md` §5 WC2 "Visible keys" and
//! "Input"; WC4.1 "Requestable texts"). The input is stored as the run's `input_snapshot` and
//! sent byte for byte besides the instructions.

use psm_core::{
    Result,
    domain::attribute::{AttributeGroup, CoreType},
    mechanisms::values::canonical_json,
    ports::store::StoreTx,
};
use serde_json::{Map, Value as Json, json};

use crate::{
    domain::Verdict,
    service::{FormSnapshot, ResearchService, form_snapshot},
    stages::display,
    store,
    writer::{
        config::WriterSettings,
        note::{Decision, decision},
    },
};

/// The built-in description key.
pub const PRODUCT_DESCRIPTION: &str = "product.description";
/// W5's search title and search description keys (not defined before W5, so never requestable).
pub const PRODUCT_SEO_TITLE: &str = "product.seo_title";
pub const PRODUCT_SEO_DESCRIPTION: &str = "product.seo_description";
/// The most matching-listing titles the writer receives (WC2).
pub const LISTINGS_MAX: usize = 8;

/// The writer's five texts as `(output name, field key)`, in the order of 08C §9.
pub fn text_keys(settings: &WriterSettings) -> Vec<(&'static str, String)> {
    vec![
        ("title", crate::writer::note::PRODUCT_TITLE.to_owned()),
        ("description", PRODUCT_DESCRIPTION.to_owned()),
        ("condition_report", settings.condition_report_key.clone()),
        ("seo_title", PRODUCT_SEO_TITLE.to_owned()),
        ("seo_description", PRODUCT_SEO_DESCRIPTION.to_owned()),
    ]
}

/// WC4.1: `title` always; each other text key when it is in the layout with an active, writable
/// definition of core type `text`, `textarea` or `rich_text`. `(output name, field key)` in the
/// order of [`text_keys`].
pub fn requestable_texts(form: &FormSnapshot, settings: &WriterSettings) -> Vec<(String, String)> {
    text_keys(settings)
        .into_iter()
        .filter(|(name, key)| {
            *name == "title"
                || form.fields.iter().any(|f| {
                    &f.definition.key == key
                        && f.definition.writable
                        && matches!(
                            f.definition.core_type,
                            CoreType::Text | CoreType::Textarea | CoreType::RichText
                        )
                })
        })
        .map(|(name, key)| (name.to_owned(), key))
        .collect()
}

/// One layout field as [`visible_keys`] reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutField {
    pub key: String,
    pub step: AttributeGroup,
    pub core_type: CoreType,
}

/// WC2 "Visible keys": `saved` (when set) ∩ the layout, in layout order; otherwise every layout
/// key of the Details and Condition steps whose core type is not `money`, `media_list` or
/// `reference`. The writer's own text keys (`own`) are never visible.
pub fn visible_keys(
    layout: &[LayoutField],
    saved: Option<&[String]>,
    own: &[String],
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for field in layout {
        if own.contains(&field.key) || out.contains(&field.key) {
            continue;
        }
        let shown = match saved {
            Some(saved) => saved.contains(&field.key),
            None => {
                matches!(
                    field.step,
                    AttributeGroup::Details | AttributeGroup::Condition
                ) && !matches!(
                    field.core_type,
                    CoreType::Money | CoreType::MediaList | CoreType::Reference
                )
            }
        };
        if shown {
            out.push(field.key.clone());
        }
    }
    out
}

/// The layout of a form snapshot as [`visible_keys`] reads it.
pub fn layout_of(form: &FormSnapshot) -> Vec<LayoutField> {
    form.fields
        .iter()
        .map(|f| LayoutField {
            key: f.definition.key.clone(),
            step: f.step,
            core_type: f.definition.core_type,
        })
        .collect()
}

/// The visible keys of an item's form under the writer settings.
pub fn visible_of(form: &FormSnapshot, settings: &WriterSettings) -> Vec<String> {
    let own: Vec<String> = text_keys(settings).into_iter().map(|(_, k)| k).collect();
    visible_keys(&layout_of(form), settings.visible_keys.as_deref(), &own)
}

/// `fact` of a decision (WC2): a value the writer may state.
pub fn is_fact(decision: Decision) -> bool {
    !matches!(
        decision,
        Decision::Empty
            | Decision::Dismissed
            | Decision::Unchecked(Some(psm_core::domain::attribute::AiConfidence::Low))
    )
}

/// The `answered` writer findings of an item, oldest first (by when they were answered).
pub(crate) fn answered(conn: &rusqlite::Connection, item_id: i64) -> Result<Vec<Json>> {
    let mut stmt = conn
        .prepare(
            "SELECT type, keys, text, answer FROM writer_findings WHERE item_id=?1 \
             AND state='answered' ORDER BY COALESCE(resolved_at, created_at), id",
        )
        .map_err(store::db)?;
    stmt.query_map([item_id], |r| {
        let keys: String = r.get(1)?;
        Ok(json!({
            "type": r.get::<_, String>(0)?,
            "keys": serde_json::from_str::<Json>(&keys).unwrap_or(json!([])),
            "question": r.get::<_, String>(2)?,
            "answer": r.get::<_, Option<String>>(3)?.unwrap_or_default(),
        }))
    })
    .map_err(store::db)?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(store::db)
}

/// WC2 `listings`: the titles of the stage 3 pages of the session's latest research run with a
/// stage 4 verdict `match` or `similar`, `match` first, then in stage 3 order, at most 8.
fn listings(conn: &rusqlite::Connection, session_id: i64) -> Result<Vec<Json>> {
    let Some(run) = store::runs(conn, Some(session_id))?.pop() else {
        return Ok(Vec::new());
    };
    let outputs = store::outputs(conn, run.id)?;
    let parsed = |o: &store::OutputRow| serde_json::from_str::<Json>(&o.output).ok();
    let verdicts: Vec<(String, Verdict)> = outputs
        .iter()
        .filter(|o| o.stage == crate::domain::Stage::Compare.as_str())
        .filter_map(|o| {
            let output = parsed(o)?;
            let verdict: Verdict = serde_json::from_value(output["verdict"].clone()).ok()?;
            Some((o.unit_key.clone(), verdict))
        })
        .collect();
    let mut matched: Vec<Json> = Vec::new();
    let mut similar: Vec<Json> = Vec::new();
    for page in outputs
        .iter()
        .filter(|o| o.stage == crate::domain::Stage::Read.as_str())
    {
        let Some(output) = parsed(page) else {
            continue;
        };
        if output["obtained"].as_bool() != Some(true) {
            continue;
        }
        let Some((_, verdict)) = verdicts.iter().find(|(unit, _)| *unit == page.unit_key) else {
            continue;
        };
        let title = output["title"].as_str().unwrap_or_default().to_owned();
        match verdict {
            Verdict::Match => matched.push(json!({ "title": title, "verdict": "match" })),
            Verdict::Similar => similar.push(json!({ "title": title, "verdict": "similar" })),
            Verdict::Different => {}
        }
    }
    matched.extend(similar);
    matched.truncate(LISTINGS_MAX);
    Ok(matched)
}

/// `writer::input::build` (WC2 "Input"), as a free function.
pub fn build(svc: &ResearchService, session_id: i64, settings: &WriterSettings) -> Result<Json> {
    svc.writer_input(session_id, settings)
}

impl ResearchService {
    /// `writer::input::build` (WC2 "Input"): what a writer run of the session receives, built from
    /// the item's stored rows and feedback, the note as the owner's decisions show it, the latest
    /// research run's listings and the answered findings.
    pub fn writer_input(&self, session_id: i64, settings: &WriterSettings) -> Result<Json> {
        let Some(item_id) = self.session_item(session_id)? else {
            return Ok(Json::Null);
        };
        let form = form_snapshot(
            &self.store,
            self.clock.as_ref(),
            self.integration.as_deref(),
            item_id,
        )?;
        let note = self.assemble(session_id)?;
        let visible = visible_of(&form, settings);
        let texts = requestable_texts(&form, settings);
        let label = |key: &str| -> String {
            form.fields
                .iter()
                .find(|f| f.definition.key == key)
                .map(|f| f.definition.label.clone())
                .unwrap_or_else(|| key.to_owned())
        };
        let (fields, rejected, listings, answers) = self.store.research_transaction(|tx| {
            let rows = tx.attribute_values(item_id)?;
            let mut fields: Vec<Json> = Vec::new();
            for key in &visible {
                let row = rows.iter().find(|r| &r.key == key);
                let feedback = tx.feedback_by_key(item_id, key)?;
                let decided = decision(row, &feedback);
                let value = match decided {
                    Decision::Empty | Decision::Dismissed => Json::Null,
                    _ => row
                        .and_then(|r| r.json())
                        .map(|v| Json::String(display(&v)))
                        .unwrap_or(Json::Null),
                };
                fields.push(json!({
                    "key": key,
                    "label": label(key),
                    "value": value,
                    "decision": decided.code(),
                    "fact": is_fact(decided),
                }));
            }
            let mut rejected: Vec<Json> = Vec::new();
            for (name, key) in &texts {
                let row = rows.iter().find(|r| &r.key == key);
                let mut seen: Vec<String> = Vec::new();
                for value in crate::writes::cleared_set(tx, row, item_id, key)? {
                    let canonical = canonical_json(&value);
                    if seen.contains(&canonical) {
                        continue;
                    }
                    seen.push(canonical);
                    rejected.push(json!({ "text": name, "value": display(&value) }));
                }
            }
            Ok((
                fields,
                rejected,
                listings(tx.conn(), session_id)?,
                answered(tx.conn(), item_id)?,
            ))
        })?;
        let mut research_saw: Vec<Json> = Vec::new();
        for saw in &note.research_saw {
            research_saw.push(json!({ "key": saw.key, "text": saw.text }));
        }
        let observations: Vec<Json> = note
            .observations
            .iter()
            .filter(|o| o.in_writer_input)
            .map(|o| {
                json!({
                    "ref": o.reference,
                    "text": o.text,
                    "keys": o.keys,
                    "photo": o.photo,
                    "owner": o.owner,
                })
            })
            .collect();
        Ok(assemble(
            InputParts {
                fields,
                note: observations,
                research_saw,
                listings,
                answers,
                rejected,
                texts: texts.into_iter().map(|(n, _)| n).collect(),
            },
            settings,
        ))
    }
}

/// The parts of a writer input read from an item (or, for the preview, from the built-in
/// example item).
#[derive(Debug, Clone, Default)]
pub struct InputParts {
    pub fields: Vec<Json>,
    pub note: Vec<Json>,
    pub research_saw: Vec<Json>,
    pub listings: Vec<Json>,
    pub answers: Vec<Json>,
    pub rejected: Vec<Json>,
    /// The requested output names.
    pub texts: Vec<String>,
}

/// The writer input (WC2 "Input") from its parts and the settings: a run's input and the
/// preview's are both assembled here.
pub fn assemble(parts: InputParts, settings: &WriterSettings) -> Json {
    let mut input = Map::new();
    input.insert("fields".into(), Json::Array(parts.fields));
    input.insert("note".into(), Json::Array(parts.note));
    input.insert("research_saw".into(), Json::Array(parts.research_saw));
    input.insert("listings".into(), Json::Array(parts.listings));
    input.insert("answers".into(), Json::Array(parts.answers));
    input.insert("rejected".into(), Json::Array(parts.rejected));
    input.insert(
        "texts".into(),
        Json::Array(parts.texts.iter().map(|n| json!(n)).collect()),
    );
    input.insert("language".into(), json!(settings.language));
    input.insert(
        "lengths".into(),
        serde_json::to_value(&settings.lengths).unwrap_or(Json::Null),
    );
    if !settings.shop_name.trim().is_empty() {
        input.insert("shop_name".into(), json!(settings.shop_name));
    }
    Json::Object(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(key: &str, step: AttributeGroup, core_type: CoreType) -> LayoutField {
        LayoutField {
            key: key.into(),
            step,
            core_type,
        }
    }

    fn layout() -> Vec<LayoutField> {
        use AttributeGroup::{Condition, Details, Product};
        vec![
            field("custom.designer", Details, CoreType::Text),
            field("custom.price", Details, CoreType::Money),
            field("custom.condition_scratch", Condition, CoreType::Choice),
            field("custom.damage_report", Condition, CoreType::MediaList),
            field("product.description", Product, CoreType::RichText),
        ]
    }

    fn own() -> Vec<String> {
        text_keys(&WriterSettings::default())
            .into_iter()
            .map(|(_, k)| k)
            .collect()
    }

    #[test]
    fn w2_c3a_by_default_the_details_and_condition_fields_without_prices_and_media() {
        assert_eq!(
            visible_keys(&layout(), None, &own()),
            ["custom.designer", "custom.condition_scratch"]
        );
    }

    #[test]
    fn w2_c3b_a_saved_list_in_layout_order_without_unknown_keys() {
        let saved: Vec<String> = ["custom.price", "custom.gone", "custom.designer"]
            .iter()
            .map(|k| (*k).to_owned())
            .collect();
        assert_eq!(
            visible_keys(&layout(), Some(&saved), &own()),
            ["custom.designer", "custom.price"]
        );
    }
}
