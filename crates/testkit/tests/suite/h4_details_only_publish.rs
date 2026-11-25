//! H4·U2 — the details-only publish and the rows of remote items (master plan archive §5
//! MC-H3.6–8, M-H1; book 10H §6 "Publish of details only"). `PublishService` (`preview`,
//! `confirm`, the run to its end), `SessionsService::list` and `StorageService` on
//! `TestApp::with_fake_shop()`. Outcomes: the preview, the persisted publish item, what
//! `FakeCommerce` recorded during the run, the shop's gallery and condition field afterwards, list
//! rows. "Archived item (3 + 2)" = 3 main and 2 condition photos, published, then cleared by the
//! real clearing (`h3_bring_back::archived_of`).

use crate::h3_bring_back::{Remote, adopted, archived_of};
use crate::p62_retention_after_publish::{clear_all, published, shop_app};
use psm_core::{
    domain::{
        attribute::{AttributeGroup, CoreType, Definition, Provenance},
        publish::{DefinitionLink, PublishItem, PublishPreviewItem},
        remote::{RemoteKind, RemoteState},
        session::{
            ListFilter, PublishStep, RowLabel, SessionList, SessionPage, SessionRow, SessionStatus,
        },
    },
    ports::{clock::Clock, commerce::CommerceAdapter},
    services::attributes::{AttributesService, PRODUCT_TITLE},
};
use psm_testkit::{ConfirmAsShown, TestApp};
use serde_json::json;

const DAY: i64 = 86_400_000;
const ARCHIVED: RemoteState = RemoteState::Remote(RemoteKind::Archived);
const SHOP_ONLY: RemoteState = RemoteState::Remote(RemoteKind::ShopOnly);
const CONDITION_KEY: &str = "custom.damage_report";
const TEXT_KEY: &str = "custom.material";
/// The mutations a details-only run never makes (MC-H3.6's invariant).
const MEDIA_OPS: [&str; 6] = [
    "staged_uploads_create",
    "upload",
    "product_create_media",
    "file_create",
    "product_delete_media",
    "product_reorder_media",
];

// ---------------------------------------------------------------------------------------------
// fixtures

/// `shop_app()` with a text field `custom.material` besides the condition-photos field.
fn app() -> TestApp {
    let a = shop_app();
    let text = Definition {
        integration: "shopify".into(),
        key: TEXT_KEY.into(),
        label: "Material".into(),
        core_type: CoreType::Text,
        choices: Vec::new(),
        validations: json!({}),
        group: AttributeGroup::Details,
        description: None,
        writable: true,
    };
    a.fake_shop().state.lock().unwrap().definitions.push((
        text,
        DefinitionLink {
            platform_type: "single_line_text_field".into(),
            platform_meta: json!({}),
        },
    ));
    AttributesService::new(a.store(), a.clock.as_ref(), Some("shopify"))
        .refresh_schema(a.fake_shop())
        .unwrap();
    a
}

fn drain(a: &TestApp) {
    for _ in 0..12 {
        a.tick(1100).unwrap();
        a.complete_all_renders().unwrap();
    }
}

fn archived(a: &TestApp, sku: &str) -> Remote {
    archived_of(a, sku, "Sideboard", 3, 2)
}

/// An adopted shop product with `gallery` and `condition` pictures whose first-open read ran.
fn shop_only(a: &TestApp, sku: &str, gallery: &[&str], condition: &[&str]) -> Remote {
    let r = adopted(a, sku, gallery, condition);
    assert_eq!(a.remote().ensure_record(r.session).unwrap(), SHOP_ONLY);
    r
}

fn reads(a: &TestApp, op: &str) -> usize {
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .reads
        .iter()
        .filter(|r| r.as_str() == op)
        .count()
}

fn preview_one(a: &TestApp, session: i64) -> PublishPreviewItem {
    let mut preview = a.app.publish.as_ref().unwrap().preview(&[session]).unwrap();
    assert_eq!(preview.items.len(), 1);
    preview.items.remove(0)
}

fn assert_details_only(p: &PublishPreviewItem) {
    assert_eq!(
        (
            p.main.clone(),
            p.condition.clone(),
            p.main_thumbnails.len(),
            p.condition_thumbnails.len(),
            p.announced_removals.clone(),
            p.details_only,
            p.create_item,
        ),
        (vec![], vec![], 0, 0, Vec::<String>::new(), true, false)
    );
}

fn confirm(a: &TestApp, session: i64) {
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm_as_shown(&[session], "Publish details")
        .unwrap();
}

fn latest_item(a: &TestApp, session: i64) -> PublishItem {
    let latest = a.store().latest_publish(session).unwrap().unwrap();
    a.store().publish_item(latest.item_id).unwrap()
}

fn product(a: &TestApp, item: i64) -> String {
    a.store()
        .item_link(item, "shopify")
        .unwrap()
        .unwrap()
        .external_id
}

/// The shop's gallery media ids and condition-field ids of the item's product.
fn shop_pictures(a: &TestApp, item: i64) -> (Vec<String>, Vec<String>) {
    let readback = a.fake_shop().media_readback(&product(a, item)).unwrap();
    (
        readback.gallery.into_iter().map(|m| m.id).collect(),
        readback.condition.into_iter().map(|m| m.id).collect(),
    )
}

fn shop_value(a: &TestApp, item: i64, key: &str) -> Option<serde_json::Value> {
    let product = product(a, item);
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .values
        .get(&product)
        .and_then(|rows| rows.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone()))
}

fn shop_title(a: &TestApp, item: i64) -> String {
    let product = product(a, item);
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .products
        .iter()
        .find(|p| p.external_id == product)
        .unwrap()
        .title
        .clone()
}

fn media_mutations(ops: &[String]) -> Vec<String> {
    ops.iter()
        .filter(|op| MEDIA_OPS.contains(&op.as_str()))
        .cloned()
        .collect()
}

fn set_text(a: &TestApp, item: i64, value: &str) {
    a.attributes(Some("shopify"))
        .set_value(item, TEXT_KEY, json!(value), Provenance::Human)
        .unwrap();
}

fn row(a: &TestApp, session: i64) -> SessionRow {
    a.sessions()
        .with_remote(a.remote())
        .list(ListFilter::All, None)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .find(|r| r.id == session)
        .unwrap()
}

fn counts(r: &SessionRow) -> (i64, i64, i64) {
    (r.main_count, r.condition_count, r.unreviewed_count)
}

// ---------------------------------------------------------------------------------------------
// H4·C5 — the preview of a remote item

#[test]
fn h4_c5a_an_archived_item_is_previewed_details_only() {
    let a = app();
    let r = archived(&a, "DO-A");
    let item_gets = reads(&a, "item_get");
    let p = preview_one(&a, r.session);
    assert_details_only(&p);
    assert_eq!(reads(&a, "item_get"), item_gets);
}

#[test]
fn h4_c5b_a_local_published_session_is_previewed_as_today() {
    let a = app();
    let it = published(&a, "DO-B", 2, 0);
    let p = preview_one(&a, it.session);
    assert!(!p.details_only);
    assert_eq!(p.main, vec![it.photos[0].id, it.photos[1].id]);
}

#[test]
fn h4_c5c_a_shop_only_item_is_previewed_details_only() {
    let a = app();
    let r = shop_only(&a, "DO-C", &["doc-m1", "doc-m2"], &[]);
    let item_gets = reads(&a, "item_get");
    let p = preview_one(&a, r.session);
    assert_details_only(&p);
    assert_eq!(reads(&a, "item_get"), item_gets);
}

#[test]
fn h4_c5d_an_unknown_adopted_item_is_read_then_previewed_details_only() {
    let a = app();
    let r = adopted(&a, "DO-D", &["dod-m1", "dod-m2"], &[]);
    assert_eq!(a.remote().state(r.session).unwrap(), RemoteState::Unknown);
    let (readbacks, item_gets) = (reads(&a, "media_readback"), reads(&a, "item_get"));
    let p = preview_one(&a, r.session);
    assert_eq!(reads(&a, "media_readback") - readbacks, 1);
    assert_details_only(&p);
    // The fake's `media_readback` reads its product through `item_get` once; the preview itself
    // makes no `item_get` read.
    assert_eq!(reads(&a, "item_get") - item_gets, 1);
    assert_eq!(a.remote().state(r.session).unwrap(), SHOP_ONLY);
}

// ---------------------------------------------------------------------------------------------
// H4·C6 — the details-only run

/// H4·C6(a)'s run: the archived item's title changed and its text field edited, confirmed and
/// run to its end. Returns the item and the mutation ops of the run.
fn details_published(a: &TestApp) -> (Remote, Vec<String>) {
    let r = archived(a, "DO-RUN");
    a.attributes(Some("shopify"))
        .set_value(
            r.item,
            PRODUCT_TITLE,
            json!("Oak sideboard, restored"),
            Provenance::Human,
        )
        .unwrap();
    set_text(a, r.item, "Solid oak");
    let mark = a.fake_shop().mutation_ops().len();
    confirm(a, r.session);
    drain(a);
    let ops = a.fake_shop().mutation_ops()[mark..].to_vec();
    (r, ops)
}

#[test]
fn h4_c6a_an_archived_item_publishes_its_details_and_no_media() {
    let a = app();
    let (r, ops) = details_published(&a);
    assert_eq!(latest_item(&a, r.session).step, PublishStep::Done);
    assert_eq!(shop_title(&a, r.item), "Oak sideboard, restored");
    assert_eq!(shop_value(&a, r.item, TEXT_KEY), Some(json!("Solid oak")));
    assert_eq!(media_mutations(&ops), Vec::<String>::new());
    assert_eq!(
        shop_pictures(&a, r.item),
        (r.media[..3].to_vec(), r.media[3..].to_vec())
    );
}

#[test]
fn h4_c6b_details_only_is_stored_with_empty_photo_lists() {
    let mut a = app();
    let (r, _) = details_published(&a);
    a.kill_and_restart().unwrap();
    let item = latest_item(&a, r.session);
    assert_eq!(
        (item.details_only, item.frozen_main, item.frozen_condition),
        (true, vec![], vec![])
    );
}

#[test]
fn h4_c6c_a_run_killed_after_the_attribute_write_resumes_details_only() {
    let mut a = app();
    let r = archived(&a, "DO-KILL");
    set_text(&a, r.item, "Walnut");
    confirm(&a, r.session);
    a.fake_shop().fail_after_effect("attributes_write");
    let killed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drain(&a)));
    assert!(killed.is_err(), "the run was not interrupted");
    a.kill_and_restart().unwrap();
    drain(&a);
    assert_eq!(latest_item(&a, r.session).step, PublishStep::Done);
    assert_eq!(
        shop_pictures(&a, r.item),
        (r.media[..3].to_vec(), r.media[3..].to_vec())
    );
    // The whole history: only the first publish (before clearing) touched media.
    let item = latest_item(&a, r.session);
    let runs: Vec<String> = a
        .fake_shop()
        .state
        .lock()
        .unwrap()
        .mutations
        .iter()
        .filter(|m| m.run_id == item.run_id)
        .map(|m| m.op.clone())
        .collect();
    assert_eq!(media_mutations(&runs), Vec::<String>::new());
}

#[test]
fn h4_c6d_a_shop_only_item_publishes_its_details() {
    let a = app();
    let r = shop_only(&a, "DO-SO", &["dso-m1", "dso-m2"], &["dso-c1"]);
    set_text(&a, r.item, "Pine");
    let mark = a.fake_shop().mutation_ops().len();
    confirm(&a, r.session);
    drain(&a);
    assert_eq!(latest_item(&a, r.session).step, PublishStep::Done);
    assert_eq!(shop_value(&a, r.item, TEXT_KEY), Some(json!("Pine")));
    assert_eq!(
        media_mutations(&a.fake_shop().mutation_ops()[mark..]),
        Vec::<String>::new()
    );
    assert!(
        a.store()
            .session(r.session)
            .unwrap()
            .session
            .published_at
            .is_some()
    );
    assert_eq!(a.remote().state(r.session).unwrap(), SHOP_ONLY);
}

#[test]
fn h4_c6e_the_item_stays_archived_after_its_details_publish() {
    let a = app();
    let r = archived(&a, "DO-STAY");
    let cleared = a.store().item_retention(r.item).unwrap().cleared_at;
    assert!(cleared.is_some());
    set_text(&a, r.item, "Teak");
    confirm(&a, r.session);
    drain(&a);
    assert_eq!(latest_item(&a, r.session).step, PublishStep::Done);
    assert_eq!(
        a.store().item_retention(r.item).unwrap().cleared_at,
        cleared
    );
    assert_eq!(a.remote().state(r.session).unwrap(), ARCHIVED);
}

// ---------------------------------------------------------------------------------------------
// H4·C7 — the condition-photos value is never written by a details-only run

#[test]
fn h4_c7a_a_stored_condition_value_is_left_out_of_the_writes() {
    let a = app();
    // The condition-photos value as an earlier publish left it: set, published, then cleared.
    let it = published(&a, "DO-COND", 3, 2);
    a.attributes(Some("shopify"))
        .set_value(
            it.item,
            CONDITION_KEY,
            json!({"excluded": []}),
            Provenance::Human,
        )
        .unwrap();
    confirm(&a, it.session);
    drain(&a);
    assert_eq!(latest_item(&a, it.session).step, PublishStep::Done);
    assert!(!a.store().session(it.session).unwrap().dirty);
    a.clock.advance(4 * DAY + 60_000);
    clear_all(&a);
    assert!(
        a.store()
            .item_retention(it.item)
            .unwrap()
            .cleared_at
            .is_some()
    );
    let (_, condition_before) = shop_pictures(&a, it.item);
    assert_eq!(condition_before.len(), 2);
    assert!(
        latest_item(&a, it.session)
            .frozen_attributes
            .contains_key(CONDITION_KEY)
    );
    set_text(&a, it.item, "Elm");
    let writes_before = a.fake_shop().state.lock().unwrap().attribute_writes.len();
    confirm(&a, it.session);
    drain(&a);
    assert_eq!(latest_item(&a, it.session).step, PublishStep::Done);
    let written: Vec<String> = a.fake_shop().state.lock().unwrap().attribute_writes
        [writes_before..]
        .iter()
        .flatten()
        .map(|w| w.key.clone())
        .collect();
    assert_eq!(written, vec![TEXT_KEY.to_string()]);
    assert_eq!(shop_pictures(&a, it.item).1, condition_before);
}

// ---------------------------------------------------------------------------------------------
// H4·C8 — retention keeps a published shop-only item

#[test]
fn h4_c8a_a_published_shop_only_item_is_never_cleared() {
    let a = app();
    let r = shop_only(&a, "DO-KEEP", &["dk-m1", "dk-m2"], &["dk-c1"]);
    set_text(&a, r.item, "Ash");
    confirm(&a, r.session);
    drain(&a);
    assert_eq!(latest_item(&a, r.session).step, PublishStep::Done);
    let record = a.store().archived_images(r.item).unwrap();
    let readbacks = reads(&a, "media_readback");
    a.clock.advance(30 * DAY);
    clear_all(&a);
    assert_eq!(a.store().item_retention(r.item).unwrap().cleared_at, None);
    assert_eq!(a.store().archived_images(r.item).unwrap(), record);
    assert_eq!(reads(&a, "media_readback"), readbacks);
}

// ---------------------------------------------------------------------------------------------
// H4·C9 — the rows of remote items

#[test]
fn h4_c9b_a_clean_archived_row() {
    let a = app();
    let r = archived(&a, "ROW-CLEAN");
    let row = row(&a, r.session);
    assert_eq!(
        (row.status, row.label, row.main_count, row.condition_count),
        (SessionStatus::Published, RowLabel::Archived, 3, 2)
    );
}

#[test]
fn h4_c9c_a_dirty_archived_row() {
    let a = app();
    let r = archived(&a, "ROW-DIRTY");
    set_text(&a, r.item, "Beech");
    // An attribute edit never moves `change_seq` (attributes MC-6), so the session is made dirty
    // through the store, as a change after its publish leaves it.
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .execute(
            "UPDATE sessions SET change_seq=change_seq+1 WHERE id=?1",
            [r.session],
        )
        .unwrap();
    assert!(a.store().session(r.session).unwrap().dirty);
    let row = row(&a, r.session);
    assert_eq!(
        (row.status, row.label),
        (SessionStatus::Ready, RowLabel::Archived)
    );
}

/// An archived item whose details-only publish was confirmed and then put in `step` (through the
/// store, as a running or ended publish leaves it).
fn archived_in_step(a: &TestApp, sku: &str, step: PublishStep, failed: Option<PublishStep>) -> i64 {
    let r = archived(a, sku);
    set_text(a, r.item, "Maple");
    confirm(a, r.session);
    let item = latest_item(a, r.session);
    assert!(item.details_only);
    a.store()
        .set_publish_step(
            item.id,
            step,
            failed,
            failed.map(|_| "the shop could not be reached"),
            a.clock.now_ms(),
        )
        .unwrap();
    r.session
}

#[test]
fn h4_c9d_a_running_details_publish_shows_through() {
    let a = app();
    let s = archived_in_step(&a, "ROW-RUN", PublishStep::Update, None);
    assert_eq!(row(&a, s).label, RowLabel::Publishing(75));
}

#[test]
fn h4_c9e_a_failed_details_publish_shows_through() {
    let a = app();
    let s = archived_in_step(
        &a,
        "ROW-FAIL",
        PublishStep::Failed,
        Some(PublishStep::Update),
    );
    assert_eq!(row(&a, s).label, RowLabel::PublishFailed);
}

#[test]
fn h4_c9f_a_cancelled_details_publish_shows_through() {
    let a = app();
    let s = archived_in_step(&a, "ROW-CANCEL", PublishStep::Cancelled, None);
    assert_eq!(row(&a, s).label, RowLabel::PublishCancelled);
}

#[test]
fn h4_c9g_after_bring_back_the_row_is_an_ordinary_published_row() {
    let a = app();
    let r = archived(&a, "ROW-BACK");
    a.remote().bring_back(r.session).unwrap();
    let row = row(&a, r.session);
    assert_eq!(
        (row.status, row.label, row.main_count, row.condition_count),
        (SessionStatus::Published, RowLabel::Published, 3, 2)
    );
}

// ---------------------------------------------------------------------------------------------
// L4·C1 — products from the other Mac count as Published (09C §14.3, MC-H3.7A)

fn listed(a: &TestApp, filter: ListFilter) -> SessionList {
    a.sessions()
        .with_remote(a.remote())
        .list(filter, None)
        .unwrap()
}

fn holds(list: &SessionList, session: i64) -> usize {
    list.groups
        .iter()
        .flat_map(|g| &g.rows)
        .filter(|r| r.id == session)
        .count()
}

fn page_holds(page: &SessionPage, session: i64) -> Option<SessionRow> {
    page.groups
        .iter()
        .flat_map(|g| &g.rows)
        .find(|r| r.id == session)
        .cloned()
}

/// A shop-only item whose details-only publish was confirmed and then put in `step` (through the
/// store, as `archived_in_step` does for an archived item).
fn shop_only_in_step(
    a: &TestApp,
    sku: &str,
    step: PublishStep,
    failed: Option<PublishStep>,
) -> i64 {
    let r = shop_only(a, sku, &["st-m1", "st-m2", "st-m3"], &["st-c1"]);
    set_text(a, r.item, "Maple");
    confirm(a, r.session);
    let item = latest_item(a, r.session);
    assert!(item.details_only);
    a.store()
        .set_publish_step(
            item.id,
            step,
            failed,
            failed.map(|_| "the shop could not be reached"),
            a.clock.now_ms(),
        )
        .unwrap();
    r.session
}

#[test]
fn l4_c1a_a_shop_only_row_is_published_and_counts_its_record() {
    let a = app();
    let r = shop_only(&a, "TEST-1802", &["so-m1", "so-m2", "so-m3"], &["so-c1"]);
    let row = row(&a, r.session);
    assert_eq!(counts(&row), (3, 1, 0));
    assert_eq!(
        (row.status, row.label),
        (SessionStatus::Published, RowLabel::Archived)
    );
}

#[test]
fn l4_c1b_a_shop_only_item_raises_published_not_open() {
    let a = app();
    let before = listed(&a, ListFilter::All).counts;
    shop_only(&a, "TEST-1803", &["sb-m1", "sb-m2", "sb-m3"], &["sb-c1"]);
    let after = listed(&a, ListFilter::All).counts;
    assert_eq!(
        (after.published, after.open, after.all),
        (before.published + 1, before.open, before.all + 1)
    );
}

#[test]
fn l4_c1c_a_shop_only_item_filters_as_published() {
    let a = app();
    let r = shop_only(&a, "TEST-1804", &["sc-m1", "sc-m2", "sc-m3"], &["sc-c1"]);
    assert_eq!(holds(&listed(&a, ListFilter::Open), r.session), 0);
    assert_eq!(holds(&listed(&a, ListFilter::Published), r.session), 1);
}

#[test]
fn l4_c1d_the_paged_list_agrees() {
    let a = app();
    let r = shop_only(&a, "TEST-1805", &["sd-m1", "sd-m2", "sd-m3"], &["sd-c1"]);
    let all = listed(&a, ListFilter::All).counts;
    let sessions = a.sessions();
    let sessions = sessions.with_remote(a.remote());
    let published = sessions
        .page(ListFilter::Published, None, None, None, 1)
        .unwrap();
    let open = sessions
        .page(ListFilter::Open, None, None, None, 1)
        .unwrap();
    assert_eq!(
        page_holds(&published, r.session).map(|row| row.status),
        Some(SessionStatus::Published)
    );
    assert!(page_holds(&open, r.session).is_none());
    assert_eq!(published.counts, all);
    assert_eq!(open.counts, all);
}

#[test]
fn l4_c1e_a_running_publish_of_a_shop_only_item_is_open() {
    let a = app();
    let s = shop_only_in_step(&a, "TEST-1806", PublishStep::Update, None);
    let row = row(&a, s);
    assert_eq!(
        (row.status, row.label),
        (SessionStatus::Publishing, RowLabel::Publishing(75))
    );
    assert_eq!(holds(&listed(&a, ListFilter::Open), s), 1);
}

#[test]
fn l4_c1f_a_held_publish_of_a_shop_only_item_is_publishing() {
    let a = app();
    let s = shop_only_in_step(&a, "TEST-1807", PublishStep::Held, None);
    let row = row(&a, s);
    assert_eq!(
        (row.status, row.label),
        (SessionStatus::Publishing, RowLabel::WaitingForYou(0))
    );
}

#[test]
fn l4_c1g_a_failed_publish_of_a_shop_only_item_is_failed() {
    let a = app();
    let s = shop_only_in_step(
        &a,
        "TEST-1808",
        PublishStep::Failed,
        Some(PublishStep::Update),
    );
    let row = row(&a, s);
    assert_eq!(
        (row.status, row.label),
        (SessionStatus::Failed, RowLabel::PublishFailed)
    );
    assert_eq!(holds(&listed(&a, ListFilter::Published), s), 0);
}

#[test]
fn l4_c1h_a_cancelled_publish_of_a_shop_only_item_is_ready() {
    let a = app();
    let s = shop_only_in_step(&a, "TEST-1809", PublishStep::Cancelled, None);
    let row = row(&a, s);
    assert_eq!(
        (row.status, row.label),
        (SessionStatus::Ready, RowLabel::PublishCancelled)
    );
    assert_eq!(holds(&listed(&a, ListFilter::Published), s), 0);
}

#[test]
fn l4_c1i_after_its_details_publish_a_shop_only_item_is_published() {
    let a = app();
    let r = shop_only(&a, "TEST-1810", &["si-m1", "si-m2", "si-m3"], &["si-c1"]);
    set_text(&a, r.item, "Ash");
    confirm(&a, r.session);
    drain(&a);
    assert_eq!(latest_item(&a, r.session).step, PublishStep::Done);
    let row = row(&a, r.session);
    assert_eq!(
        (row.status, row.label),
        (SessionStatus::Published, RowLabel::Archived)
    );
}

#[test]
fn l4_c1j_an_unread_adopted_item_stays_open() {
    let a = app();
    let r = adopted(&a, "TEST-1811", &["sj-m1", "sj-m2", "sj-m3"], &["sj-c1"]);
    assert_eq!(a.remote().state(r.session).unwrap(), RemoteState::Unknown);
    let row = row(&a, r.session);
    assert_eq!(
        (row.status, row.label),
        (SessionStatus::InProgress, RowLabel::InProgress(0))
    );
    assert_eq!(holds(&listed(&a, ListFilter::Open), r.session), 1);
}

#[test]
fn l4_c1k_an_adopted_item_without_pictures_stays_open() {
    let a = app();
    let r = adopted(&a, "TEST-1812", &[], &[]);
    assert_eq!(
        a.remote().ensure_record(r.session).unwrap(),
        RemoteState::Local
    );
    let row = row(&a, r.session);
    assert_eq!(
        (row.status, row.label),
        (SessionStatus::InProgress, RowLabel::InProgress(0))
    );
}
