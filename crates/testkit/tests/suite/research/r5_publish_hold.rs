//! R5·U2 (rows A09·C1–C4, C6–C8): withheld values, conflicts and the held item in a publish
//! (08B MC-A10, MC-A12; 07A MC-10, MC-11, MC-13). Suggestions and conflicts are made through the
//! research write path (`psm_research::writes`).

use std::{collections::BTreeSet, fs, sync::Arc};

use psm_core::{
    Error,
    domain::{
        attribute::{
            AiConfidence, AiStatus, AttributeGroup, CoreType, Definition, Preset, PresetField,
            Provenance, Value,
        },
        photo::{Classification as C, NewPhoto, PhotoSource},
        publish::{DefinitionLink, PublishItem, ShownSuggestions, StockChoice, StockLocation},
        session::PublishStep,
    },
    mechanisms::hash::sha256_file,
    ports::{clock::Clock, commerce::CommerceAdapter},
    services::{attributes::AttributesService, history::StepKind, publish::PublishService},
};
use psm_research::writes::WriteOutcome;
use psm_testkit::{RESEARCH_INTEGRATION, TestApp};
use serde_json::{Value as Json, json};

const NOW_MS: i64 = 1_762_840_000_000;

fn shop_app(keys: &[&str]) -> TestApp {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(NOW_MS);
    a.research_fields(keys).unwrap();
    a
}
/// A new item with one session and one main photo: `(session, item)`.
fn session(a: &TestApp, sku: &str) -> (i64, i64) {
    let (item, session) = a
        .items()
        .create_local(sku, "Oak", "Sideboard", None)
        .unwrap();
    photo(a, session.session.id, &format!("{sku}.ARW"));
    (session.session.id, item.id)
}
fn photo(a: &TestApp, s: i64, name: &str) {
    let dir = a.dir.path().join("Capture/raws");
    fs::create_dir_all(&dir).unwrap();
    let raw = dir.join(name);
    fs::write(&raw, format!("raw bytes of {name}")).unwrap();
    let photo = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id: s,
            raw_path: raw.to_string_lossy().into_owned(),
            raw_hash: Some(sha256_file(&raw).unwrap()),
            captured_at: Some(NOW_MS),
            source: PhotoSource::Capture,
            created_at: NOW_MS,
        })
        .unwrap();
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.review().classify(photo.id, C::Main).unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
}
fn service(a: &TestApp) -> Arc<PublishService> {
    a.app.publish.clone().unwrap()
}
fn attrs(a: &TestApp) -> AttributesService<'_> {
    a.attributes(Some(RESEARCH_INTEGRATION))
}
fn drain(a: &TestApp) {
    for _ in 0..15 {
        a.tick(1_100).unwrap();
        a.complete_all_renders().unwrap();
    }
}
fn latest(a: &TestApp, s: i64) -> PublishItem {
    a.store()
        .publish_item(a.store().latest_publish(s).unwrap().unwrap().item_id)
        .unwrap()
}
fn text(s: &str) -> String {
    Json::String(s.into()).to_string()
}
/// A row stored directly (no undo step, no research write).
fn put(a: &TestApp, item: i64, key: &str, value: &str, provenance: Provenance, valid: bool) {
    a.store()
        .transaction(&mut |tx| {
            tx.put_attribute_value(&Value {
                item_id: item,
                key: key.into(),
                value: text(value),
                valid,
                provenance,
                ai_confidence: None,
                ai_status: None,
                ai_reason: None,
                ai_sources: None,
                ai_prior: None,
                updated_at: NOW_MS,
            })
        })
        .unwrap();
}
fn suggest(a: &TestApp, item: i64, key: &str, value: &str, confidence: AiConfidence) {
    assert!(matches!(
        a.research_suggest(item, key, value, confidence).unwrap(),
        WriteOutcome::Written(_)
    ));
}
fn conflict(a: &TestApp, item: i64, key: &str, prior: &str, suggestion: &str) {
    a.research_conflict(item, key, prior, suggestion, AiConfidence::High)
        .unwrap();
}
fn row(a: &TestApp, item: i64, key: &str) -> Option<Value> {
    let mut out = None;
    a.store()
        .transaction(&mut |tx| {
            out = tx
                .attribute_values(item)?
                .into_iter()
                .find(|v| v.key == key);
            Ok(())
        })
        .unwrap();
    out
}
/// `publish_step` job rows for the publish item (any status).
fn jobs(a: &TestApp, publish_item: i64) -> i64 {
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE kind='publish_step' AND target=?1",
            [publish_item.to_string()],
            |r| r.get(0),
        )
        .unwrap()
}
fn queued_jobs(a: &TestApp, publish_item: i64) -> i64 {
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE kind='publish_step' AND target=?1 AND status='queued'",
            [publish_item.to_string()],
            |r| r.get(0),
        )
        .unwrap()
}
fn runs(a: &TestApp) -> i64 {
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row("SELECT COUNT(*) FROM publish_runs", [], |r| r.get(0))
        .unwrap()
}
/// Every key sent in a recorded `attributes_write`, with its last value.
fn sent(a: &TestApp) -> Vec<(String, Json)> {
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .attribute_writes
        .iter()
        .flatten()
        .map(|w| (w.key.clone(), w.value.clone()))
        .collect()
}
fn sent_value(a: &TestApp, key: &str) -> Option<Json> {
    sent(a)
        .into_iter()
        .rev()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
}
fn local_attributes(a: &TestApp, sku: &str) -> Json {
    let text = fs::read_to_string(a.export_root().join(sku).join("item.json")).unwrap();
    serde_json::from_str::<Json>(&text).unwrap()["attributes"].clone()
}
fn mutations(a: &TestApp) -> usize {
    a.fake_shop().state.lock().unwrap().mutations.len()
}
fn last_token(a: &TestApp, s: i64) -> String {
    let store = a.app.history.store();
    let all = store.lock().unwrap();
    match &all.get(&s).unwrap().undo.back().unwrap().kind {
        StepKind::Attribute { token, .. } => token.clone(),
        other => panic!("not an attribute step: {other:?}"),
    }
}
fn undo_depth(a: &TestApp, s: i64) -> usize {
    let store = a.app.history.store();
    let all = store.lock().unwrap();
    all.get(&s).map_or(0, |l| l.undo.len())
}
fn entry_token(item: &PublishItem, key: &str) -> Option<String> {
    item.resolved_attributes
        .get(key)
        .and_then(|e| e["step_token"].as_str())
        .map(str::to_owned)
}
fn keys<T>(map: &std::collections::BTreeMap<String, T>) -> Vec<String> {
    map.keys().cloned().collect()
}
/// A confirmed item waiting on `k1` (Oak → Walnut) and `k2` (Pine → Teak).
fn held_two(sku: &str) -> (TestApp, i64, i64) {
    let a = shop_app(&["k1", "k2"]);
    let (s, item) = session(&a, sku);
    conflict(&a, item, "k1", "Oak", "Walnut");
    conflict(&a, item, "k2", "Pine", "Teak");
    a.confirm_as_shown(&[s], "Publish").unwrap();
    assert_eq!(latest(&a, s).step, PublishStep::Held);
    (a, s, item)
}

#[test]
fn a09_c1a_every_confirmed_kind_of_row_is_frozen() {
    let a = shop_app(&["h", "d", "i", "acc", "sh", "sm"]);
    let (s, item) = session(&a, "R5-C1A");
    put(&a, item, "h", "Human", Provenance::Human, true);
    put(&a, item, "d", "Default", Provenance::Default, true);
    put(&a, item, "i", "Shop", Provenance::Integration, true);
    suggest(&a, item, "acc", "Accepted", AiConfidence::Medium);
    attrs(&a).accept(item, "acc").unwrap();
    suggest(&a, item, "sh", "High", AiConfidence::High);
    suggest(&a, item, "sm", "Medium", AiConfidence::Medium);
    a.confirm_as_shown(&[s], "Publish").unwrap();
    let p = latest(&a, s);
    assert_eq!(
        keys(&p.frozen_attributes),
        ["acc", "d", "h", "i", "sh", "sm"]
    );
    assert_eq!(p.frozen_attributes["sh"], json!("High"));
    assert_eq!(p.step, PublishStep::Prepare);
}

#[test]
fn a09_c1b_low_suggestion_without_prior_is_withheld() {
    let a = shop_app(&["h", "low"]);
    let (s, item) = session(&a, "R5-C1B");
    put(&a, item, "h", "Human", Provenance::Human, true);
    suggest(&a, item, "low", "Guess", AiConfidence::Low);
    let preview = service(&a).preview(&[s]).unwrap();
    assert_eq!(preview.items[0].n_withheld, 1);
    a.confirm_as_shown(&[s], "Publish").unwrap();
    let p = latest(&a, s);
    assert_eq!(keys(&p.frozen_attributes), ["h"]);
    assert_eq!(p.frozen_attribute_origin["low"]["class"], "withheld");
    assert_eq!(p.frozen_attribute_origin["low"]["confidence"], "low");
    assert_eq!(p.frozen_attribute_origin["low"]["status"], "suggested");
}

#[test]
fn a09_c1c_conflicts_at_any_confidence_are_pending_not_withheld() {
    let a = shop_app(&["c1", "c2"]);
    let (s, item) = session(&a, "R5-C1C");
    a.research_conflict(item, "c1", "Oak", "Walnut", AiConfidence::Low)
        .unwrap();
    a.research_conflict(item, "c2", "Pine", "Teak", AiConfidence::High)
        .unwrap();
    let preview = service(&a).preview(&[s]).unwrap();
    assert_eq!(preview.items[0].n_withheld, 0);
    assert_eq!(preview.items[0].conflicts.len(), 2);
    a.confirm_as_shown(&[s], "Publish").unwrap();
    let p = latest(&a, s);
    assert!(p.frozen_attributes.is_empty());
    assert_eq!(keys(&p.pending_conflicts), ["c1", "c2"]);
    let c1 = &p.pending_conflicts["c1"];
    assert_eq!(c1["prior_value"], json!(text("Oak")));
    assert_eq!(c1["prior_provenance"], "default");
    assert_eq!(c1["suggestion"], json!(text("Walnut")));
    assert_eq!(c1["confidence"], "low");
    assert_eq!(p.pending_conflicts["c2"]["confidence"], "high");
    assert_eq!(p.frozen_attribute_origin["c1"]["class"], "pending");
    assert_eq!(p.step, PublishStep::Held);
}

#[test]
fn a09_c1d_low_suggestion_same_as_its_prior_is_frozen() {
    let a = shop_app(&["k"]);
    let (s, item) = session(&a, "R5-C1D");
    conflict(&a, item, "k", "Oak", "Walnut");
    suggest(&a, item, "k", "oak", AiConfidence::Low);
    let r = row(&a, item, "k").unwrap();
    assert_eq!(r.ai_prior.unwrap().value, text("Oak"));
    a.confirm_as_shown(&[s], "Publish").unwrap();
    let p = latest(&a, s);
    assert_eq!(p.frozen_attributes["k"], json!("oak"));
    assert!(p.pending_conflicts.is_empty());
    assert_eq!(p.step, PublishStep::Prepare);
}

#[test]
fn a09_c1e_cleared_and_invalid_rows_are_not_frozen() {
    let a = shop_app(&["gone", "bad", "ok"]);
    let (s, item) = session(&a, "R5-C1E");
    put(&a, item, "gone", "Old", Provenance::Default, true);
    attrs(&a).clear(item, "gone").unwrap();
    put(&a, item, "bad", "x", Provenance::Human, false);
    put(&a, item, "ok", "Fine", Provenance::Human, true);
    a.confirm_as_shown(&[s], "Publish").unwrap();
    assert_eq!(keys(&latest(&a, s).frozen_attributes), ["ok"]);
}

#[test]
fn a09_c1f_conflict_outside_the_layout_is_left_out() {
    let a = shop_app(&["k", "other"]);
    let (s, item) = session(&a, "R5-C1F");
    conflict(&a, item, "k", "Oak", "Walnut");
    attrs(&a)
        .presets_set(Preset {
            item_type: "Sideboard".into(),
            group: AttributeGroup::Details,
            layout: vec![PresetField {
                key: "other".into(),
                default: None,
            }],
        })
        .unwrap();
    let preview = service(&a).preview(&[s]).unwrap();
    assert_eq!(preview.items[0].n_withheld, 0);
    assert!(preview.items[0].conflicts.is_empty());
    a.confirm_as_shown(&[s], "Publish").unwrap();
    let p = latest(&a, s);
    assert!(!p.frozen_attributes.contains_key("k"));
    assert!(p.pending_conflicts.is_empty());
    assert_eq!(p.step, PublishStep::Prepare);
}

#[test]
fn a09_c1g_conflict_whose_definition_was_dropped_is_left_out() {
    let a = shop_app(&["k", "kept"]);
    let (s, item) = session(&a, "R5-C1G");
    conflict(&a, item, "k", "Oak", "Walnut");
    put(&a, item, "kept", "Fine", Provenance::Human, true);
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .definitions
        .retain(|(d, _)| d.key != "k");
    attrs(&a).refresh_schema(a.fake_shop()).unwrap();
    a.confirm_as_shown(&[s], "Publish").unwrap();
    let p = latest(&a, s);
    assert!(!p.frozen_attributes.contains_key("k"));
    assert!(p.pending_conflicts.is_empty());
    assert_eq!(p.step, PublishStep::Prepare);
    drain(&a);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    let local = local_attributes(&a, "R5-C1G");
    assert_eq!(local["kept"], json!("Fine"));
    assert!(local.get("k").is_none(), "{local}");
}

#[test]
fn a09_c2a_withheld_count_changed_refuses_the_confirmation() {
    let a = shop_app(&["l1", "l2"]);
    let (s, item) = session(&a, "R5-C2A");
    suggest(&a, item, "l1", "One", AiConfidence::Low);
    let preview = service(&a).preview(&[s]).unwrap();
    assert_eq!(preview.items[0].n_withheld, 1);
    let shown = vec![ShownSuggestions {
        session_id: s,
        n_withheld: 1,
        conflict_keys: vec![],
    }];
    suggest(&a, item, "l2", "Two", AiConfidence::Low);
    let refused = service(&a).confirm(&[s], &preview.summary, &shown);
    assert!(
        matches!(refused, Err(Error::SuggestionsChanged)),
        "{refused:?}"
    );
    assert_eq!(
        Error::SuggestionsChanged.to_string(),
        "AI suggestions changed while you were confirming"
    );
    assert_eq!(runs(&a), 0);
    // Separately: an empty shown set for an item with one withheld value.
    let b = shop_app(&["l1"]);
    let (t, other) = session(&b, "R5-C2A-B");
    suggest(&b, other, "l1", "One", AiConfidence::Low);
    let refused = service(&b).confirm(&[t], "Publish", &[]);
    assert!(
        matches!(refused, Err(Error::SuggestionsChanged)),
        "{refused:?}"
    );
    assert_eq!(runs(&b), 0);
}

#[test]
fn a09_c2b_conflict_resolved_before_confirm_refuses_the_confirmation() {
    let a = shop_app(&["k1"]);
    let (s, item) = session(&a, "R5-C2B");
    conflict(&a, item, "k1", "Oak", "Walnut");
    let preview = service(&a).preview(&[s]).unwrap();
    let shown = vec![ShownSuggestions {
        session_id: s,
        n_withheld: preview.items[0].n_withheld,
        conflict_keys: preview.items[0]
            .conflicts
            .iter()
            .map(|c| c.key.clone())
            .collect(),
    }];
    assert_eq!(shown[0].conflict_keys, ["k1"]);
    attrs(&a).keep_previous(item, "k1").unwrap();
    let refused = service(&a).confirm(&[s], &preview.summary, &shown);
    assert!(
        matches!(refused, Err(Error::SuggestionsChanged)),
        "{refused:?}"
    );
    assert_eq!(runs(&a), 0);
}

/// Two sessions confirmed together; the first waits on one conflict.
fn held_and_running() -> (TestApp, i64, i64) {
    let a = shop_app(&["k"]);
    let (s1, first) = session(&a, "R5-HELD");
    conflict(&a, first, "k", "Oak", "Walnut");
    let (s2, second) = session(&a, "R5-RUNS");
    put(&a, second, "k", "Pine", Provenance::Default, true);
    a.confirm_as_shown(&[s1, s2], "Publish").unwrap();
    (a, s1, s2)
}

#[test]
fn a09_c3a_held_item_sends_nothing_while_the_other_publishes() {
    let (a, s1, s2) = held_and_running();
    let held = latest(&a, s1);
    assert_eq!(held.step, PublishStep::Held);
    assert_eq!(jobs(&a, held.id), 0);
    drain(&a);
    assert_eq!(latest(&a, s1).step, PublishStep::Held);
    assert_eq!(latest(&a, s2).step, PublishStep::Done);
    let shop = a.fake_shop().state.lock().unwrap();
    assert_eq!(shop.products.len(), 1);
    assert_eq!(shop.products[0].sku.as_deref(), Some("R5-RUNS"));
    let product = shop.products[0].external_id.clone();
    assert_eq!(
        shop.values.keys().cloned().collect::<Vec<_>>(),
        vec![product.clone()]
    );
    assert!(shop.values[&product].contains(&("k".to_string(), json!("Pine"))));
    drop(shop);
    assert!(!a.fake_shop().item_get(&product).unwrap().media.is_empty());
}

#[test]
fn a09_c3b_every_mutation_follows_the_confirmation() {
    let (a, _, _) = held_and_running();
    drain(&a);
    let shop = a.fake_shop().state.lock().unwrap();
    assert!(!shop.mutations.is_empty());
    for m in &shop.mutations {
        assert!(a.store().confirmed_run(m.run_id).unwrap().unwrap() <= m.at);
    }
}

#[test]
fn a09_c4a_keep_previous_records_the_prior_and_still_waits() {
    let (a, s, item) = held_two("R5-C4A");
    a.clock.advance(5_000);
    attrs(&a).keep_previous(item, "k1").unwrap();
    let p = latest(&a, s);
    let entry = &p.resolved_attributes["k1"];
    assert_eq!(entry["value"], json!("Oak"));
    assert_eq!(entry["action"], "kept_previous");
    assert_eq!(entry["at"], json!(a.clock.now_ms()));
    assert_eq!(p.step, PublishStep::Held);
    drain(&a);
    assert_eq!(mutations(&a), 0);
}

fn l1() -> StockChoice {
    StockChoice {
        location_id: "gid://shopify/Location/111".into(),
        location_name: "Warehouse".into(),
        quantity: 1,
    }
}

/// C4(b)'s run: `k1` kept, a later suggestion on it, `k2` used; a withheld `kw` and a human `kh`
/// beside them; the item is new and its form chose a stock.
fn released_with_stock() -> (TestApp, i64, i64) {
    let a = shop_app(&["k1", "k2", "kw", "kh"]);
    a.fake_shop().state.lock().unwrap().locations = vec![StockLocation {
        id: "gid://shopify/Location/111".into(),
        name: "Warehouse".into(),
    }];
    let (s, item) = session(&a, "R5-C4B");
    conflict(&a, item, "k1", "Oak", "Walnut");
    conflict(&a, item, "k2", "Pine", "Teak");
    suggest(&a, item, "kw", "Guess", AiConfidence::Low);
    put(&a, item, "kh", "Mine", Provenance::Human, true);
    service(&a).set_item_stock(item, Some(&l1())).unwrap();
    a.confirm_as_shown(&[s], "Publish").unwrap();
    attrs(&a).keep_previous(item, "k1").unwrap();
    suggest(&a, item, "k1", "Cherry", AiConfidence::High);
    attrs(&a).accept(item, "k2").unwrap();
    (a, s, item)
}

#[test]
fn a09_c4b_last_resolution_releases_with_resolved_values_and_stock() {
    let (a, s, _) = released_with_stock();
    let p = latest(&a, s);
    assert_eq!(p.step, PublishStep::Prepare);
    assert!(p.released_at.is_some());
    assert_eq!(queued_jobs(&a, p.id), 1);
    drain(&a);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    assert_eq!(sent_value(&a, "k1"), Some(json!("Oak")));
    assert_eq!(sent_value(&a, "k2"), Some(json!("Teak")));
    assert!(!sent(&a).iter().any(|(_, v)| v == &json!("Cherry")));
    let local = local_attributes(&a, "R5-C4B");
    assert_eq!(local["k1"], json!("Oak"));
    assert_eq!(local["k2"], json!("Teak"));
    let shop = a.fake_shop().state.lock().unwrap();
    assert_eq!(
        shop.stock_requests,
        vec![("item_create_stocked".to_string(), l1())]
    );
    let product = shop.products[0].external_id.clone();
    assert_eq!(shop.stock.get(&product), Some(&l1()));
}

#[test]
fn a09_c4c_edit_sends_the_edit_and_clear_sends_nothing() {
    let a = shop_app(&["ka", "kb"]);
    let (sa, ia) = session(&a, "R5-C4C-A");
    conflict(&a, ia, "ka", "Oak", "Walnut");
    let (sb, ib) = session(&a, "R5-C4C-B");
    conflict(&a, ib, "kb", "Pine", "Teak");
    a.confirm_as_shown(&[sa], "Publish").unwrap();
    a.confirm_as_shown(&[sb], "Publish").unwrap();
    attrs(&a)
        .set_value(ia, "ka", json!("Hand"), Provenance::Human)
        .unwrap();
    attrs(&a).clear(ib, "kb").unwrap();
    assert_eq!(latest(&a, sa).step, PublishStep::Prepare);
    assert_eq!(latest(&a, sb).step, PublishStep::Prepare);
    drain(&a);
    assert_eq!(latest(&a, sa).step, PublishStep::Done);
    assert_eq!(latest(&a, sb).step, PublishStep::Done);
    assert_eq!(sent_value(&a, "ka"), Some(json!("Hand")));
    assert!(sent(&a).iter().all(|(k, _)| k != "kb"));
}

#[test]
fn a09_c4d_undo_of_a_resolution_makes_the_key_wait_again() {
    let (a, s, item) = held_two("R5-C4D");
    attrs(&a).keep_previous(item, "k1").unwrap();
    assert!(latest(&a, s).resolved_attributes.contains_key("k1"));
    a.history().undo(s).unwrap();
    let p = latest(&a, s);
    assert!(!p.resolved_attributes.contains_key("k1"));
    assert_eq!(p.step, PublishStep::Held);
    assert_eq!(p.unresolved_conflicts(), 2);
}

#[test]
fn a09_c4e_later_edit_and_its_undo_leave_the_entry() {
    let (a, s, item) = held_two("R5-C4E");
    attrs(&a).keep_previous(item, "k1").unwrap();
    let entry = latest(&a, s).resolved_attributes["k1"].clone();
    a.clock.advance(1_000);
    attrs(&a)
        .set_value(item, "k1", json!("Elm"), Provenance::Human)
        .unwrap();
    assert_eq!(latest(&a, s).resolved_attributes["k1"], entry);
    a.history().undo(s).unwrap();
    let p = latest(&a, s);
    assert_eq!(p.resolved_attributes["k1"], entry);
    assert_eq!(p.step, PublishStep::Held);
    assert_eq!(p.unresolved_conflicts(), 1);
    assert!(!p.resolved_attributes.contains_key("k2"));
}

#[test]
fn a09_c4f_redo_records_the_resolution_again() {
    let (a, s, item) = held_two("R5-C4F");
    attrs(&a).keep_previous(item, "k1").unwrap();
    let token = last_token(&a, s);
    a.history().undo(s).unwrap();
    a.history().redo(s).unwrap();
    let p = latest(&a, s);
    assert_eq!(entry_token(&p, "k1"), Some(token));
    assert_eq!(p.resolved_attributes["k1"]["value"], json!("Oak"));
    assert_eq!(p.step, PublishStep::Held);
    attrs(&a).accept(item, "k2").unwrap();
    let p = latest(&a, s);
    assert_eq!(p.step, PublishStep::Prepare);
    assert_eq!(queued_jobs(&a, p.id), 1);
}

#[test]
fn a09_c4g_typing_on_a_held_item_resolves_at_the_first_save() {
    let a = shop_app(&["k"]);
    let (s, item) = session(&a, "R5-C4G");
    conflict(&a, item, "k", "Oak", "Walnut");
    a.confirm_as_shown(&[s], "Publish").unwrap();
    let depth = undo_depth(&a, s);
    attrs(&a)
        .set_value(item, "k", json!("Wa"), Provenance::Human)
        .unwrap();
    let first = last_token(&a, s);
    assert_eq!(latest(&a, s).step, PublishStep::Prepare);
    attrs(&a)
        .set_value(item, "k", json!("Walnut"), Provenance::Human)
        .unwrap();
    assert_eq!(undo_depth(&a, s), depth + 2);
    let p = latest(&a, s);
    assert_eq!(p.resolved_attributes["k"]["value"], json!("Wa"));
    assert_eq!(entry_token(&p, "k"), Some(first));
}

#[test]
fn a09_c4j_redo_after_confirmation_is_the_resolution() {
    let a = shop_app(&["k"]);
    let (s, item) = session(&a, "R5-C4J");
    conflict(&a, item, "k", "Oak", "Walnut");
    attrs(&a).clear(item, "k").unwrap();
    let token = last_token(&a, s);
    a.history().undo(s).unwrap();
    a.confirm_as_shown(&[s], "Publish").unwrap();
    assert_eq!(latest(&a, s).step, PublishStep::Held);
    a.history().redo(s).unwrap();
    let p = latest(&a, s);
    assert_eq!(p.resolved_attributes["k"]["value"], Json::Null);
    assert_eq!(entry_token(&p, "k"), Some(token));
    assert_eq!(p.step, PublishStep::Prepare);
    assert_eq!(queued_jobs(&a, p.id), 1);
    drain(&a);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    assert!(sent(&a).iter().all(|(k, _)| k != "k"));
}

#[test]
fn a09_c4h_accept_all_resolves_a_key_that_is_no_longer_a_conflict() {
    let (a, s, item) = held_two("R5-C4H");
    suggest(&a, item, "k1", "oak", AiConfidence::Medium);
    attrs(&a).accept_all(item).unwrap();
    let p = latest(&a, s);
    assert_eq!(p.resolved_attributes["k1"]["action"], "accepted");
    assert!(!p.resolved_attributes.contains_key("k2"));
    assert_eq!(p.unresolved_conflicts(), 1);
    assert_eq!(p.step, PublishStep::Held);
}

#[test]
fn a09_c4i_keep_previous_of_an_invalid_shop_value_sends_nothing() {
    let a = shop_app(&[]);
    let count = Definition {
        integration: RESEARCH_INTEGRATION.into(),
        key: "custom.count".into(),
        label: "Count".into(),
        core_type: CoreType::Integer,
        choices: vec![],
        validations: json!({}),
        group: AttributeGroup::Details,
        description: None,
        writable: true,
    };
    a.fake_shop().state.lock().unwrap().definitions.push((
        count,
        DefinitionLink {
            platform_type: "number_integer".into(),
            platform_meta: json!({}),
        },
    ));
    attrs(&a).refresh_schema(a.fake_shop()).unwrap();
    let product = a.fake_shop().add_product("R5-C4I", "Oak", None);
    a.fake_shop().state.lock().unwrap().values.insert(
        product.clone(),
        vec![("custom.count".into(), json!("many"))],
    );
    let item = a
        .items()
        .with_commerce(Some(a.fake_shop()))
        .adopt(&product)
        .unwrap()
        .id;
    let s = a.sessions().create(item).unwrap().session.id;
    photo(&a, s, "R5-C4I.ARW");
    attrs(&a)
        .read_integration_values(Some(a.fake_shop()), item)
        .unwrap();
    let shop_row = row(&a, item, "custom.count").unwrap();
    assert_eq!(
        (shop_row.provenance, shop_row.valid),
        (Provenance::Integration, false)
    );
    let written = psm_research::writes::suggest(
        a.store(),
        a.clock.as_ref(),
        Some(RESEARCH_INTEGRATION),
        item,
        "custom.count",
        &json!(4),
        AiConfidence::High,
        Some("fixture reason"),
        &[],
    )
    .unwrap();
    assert!(matches!(written, WriteOutcome::Written(_)), "{written:?}");
    a.confirm_as_shown(&[s], "Publish").unwrap();
    assert_eq!(latest(&a, s).step, PublishStep::Held);
    attrs(&a).keep_previous(item, "custom.count").unwrap();
    let p = latest(&a, s);
    assert_eq!(p.resolved_attributes["custom.count"]["value"], Json::Null);
    assert_eq!(p.step, PublishStep::Prepare);
    drain(&a);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    assert!(sent(&a).iter().all(|(k, _)| k != "custom.count"));
}

#[test]
fn a09_c6a_cancel_of_a_held_item_is_immediate() {
    let a = shop_app(&["k"]);
    let (s, item) = session(&a, "R5-C6A");
    conflict(&a, item, "k", "Oak", "Walnut");
    a.confirm_as_shown(&[s], "Publish").unwrap();
    let activity = service(&a).activity(s).unwrap().unwrap();
    assert_eq!(activity.step, PublishStep::Held);
    assert!(activity.can_cancel);
    service(&a).cancel(activity.item_id).unwrap();
    assert_eq!(latest(&a, s).step, PublishStep::Cancelled);
    drain(&a);
    assert_eq!(mutations(&a), 0);
    let form = attrs(&a).form_get(item).unwrap();
    let value = form
        .groups
        .iter()
        .flat_map(|g| g.fields.iter())
        .find(|f| f.definition.key == "k")
        .and_then(|f| f.value.clone())
        .unwrap();
    assert_eq!(value.ai_status, Some(AiStatus::Suggested));
    assert_eq!(value.value, text("Walnut"));
    assert_eq!(value.ai_prior.unwrap().value, text("Oak"));
}

#[test]
fn a09_c6b_held_item_stays_held_across_a_restart() {
    let (mut a, s, _) = held_two("R5-C6B");
    a.kill_and_restart().unwrap();
    drain(&a);
    let p = latest(&a, s);
    assert_eq!(p.step, PublishStep::Held);
    assert_eq!(jobs(&a, p.id), 0);
}

#[test]
fn a09_c6c_release_survives_a_restart_and_runs_once() {
    let mut a = shop_app(&["k"]);
    let (s, item) = session(&a, "R5-C6C");
    conflict(&a, item, "k", "Oak", "Walnut");
    a.confirm_as_shown(&[s], "Publish").unwrap();
    attrs(&a).accept(item, "k").unwrap();
    a.kill_and_restart().unwrap();
    let p = latest(&a, s);
    assert_eq!(p.step, PublishStep::Prepare);
    assert_eq!(queued_jobs(&a, p.id), 1);
    drain(&a);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    let shop = a.fake_shop().state.lock().unwrap();
    assert_eq!(shop.products.len(), 1);
    let product = shop.products[0].external_id.clone();
    drop(shop);
    assert_eq!(a.fake_shop().item_get(&product).unwrap().media.len(), 1);
    assert_eq!(sent_value(&a, "k"), Some(json!("Walnut")));
}

#[test]
fn a09_c7a_conflict_after_confirmation_does_not_hold() {
    let a = shop_app(&["k"]);
    let (s, item) = session(&a, "R5-C7A");
    put(&a, item, "k", "Oak", Provenance::Default, true);
    a.confirm_as_shown(&[s], "Publish").unwrap();
    assert_eq!(latest(&a, s).step, PublishStep::Prepare);
    suggest(&a, item, "k", "Walnut", AiConfidence::High);
    drain(&a);
    let p = latest(&a, s);
    assert_eq!(p.step, PublishStep::Done);
    assert!(p.pending_conflicts.is_empty());
    assert_eq!(sent_value(&a, "k"), Some(json!("Oak")));
}

#[test]
fn a09_c8a_origin_and_resolutions_account_for_every_sent_key() {
    let (a, s, _) = released_with_stock();
    drain(&a);
    let p = latest(&a, s);
    assert_eq!(p.step, PublishStep::Done);
    let sent_keys: BTreeSet<String> = sent(&a).into_iter().map(|(k, _)| k).collect();
    assert!(!sent_keys.is_empty());
    for key in &sent_keys {
        let frozen = p
            .frozen_attribute_origin
            .get(key)
            .is_some_and(|o| o["class"] == "frozen");
        assert!(
            frozen || p.resolved_attributes.contains_key(key),
            "{key} sent without origin or resolution"
        );
        assert_ne!(p.frozen_attribute_origin[key]["class"], "withheld", "{key}");
    }
    assert_eq!(p.frozen_attribute_origin["kw"]["class"], "withheld");
    assert!(!sent_keys.contains("kw"));
}
