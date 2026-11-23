//! H3·U3 — publish after bring back sends only changes; retention (master plan archive §5
//! MC-H4.7, MC-H4.8, M-H3; book 10H §7.5, §7.7). `PublishService` and `StorageService` on
//! `TestApp::with_fake_shop()`; outcomes: the mutations `FakeCommerce` recorded during the publish,
//! its gallery and condition field afterwards, stored links, files on disk. "Brought-back item
//! (3 + 2)" = the archived item of U2 after `bring_back`, gallery `[m1, m2, m3]` and condition
//! `[c1, c2]` on the fake shop. No publish code was changed for these rows (MC-H4.7).

use crate::h3_bring_back::{Remote, adopted, archived_of, register_pictures};
use crate::p62_retention_after_publish::{clear_all, shop_app};
use psm_core::{
    domain::{
        composition::{
            CompositionKind, CompositionValues, Flip, Orientation, OutputSlot, Rect, Shape,
        },
        photo::{Classification as C, Photo},
        session::PublishStep,
    },
    mechanisms::composition::{default_rect, turned_frame},
    ports::clock::Clock,
};
use psm_testkit::{ConfirmAsShown, TestApp, fixtures::synthetic_raw::SyntheticRaw};
use std::{
    fs,
    path::Path,
    time::{Duration, UNIX_EPOCH},
};

const DAY: i64 = 86_400_000;
const CONDITION_KEY: &str = "custom.damage_report";

// ---------------------------------------------------------------------------------------------
// helpers

fn drain(a: &TestApp) {
    for _ in 0..12 {
        a.tick(1100).unwrap();
        a.complete_all_renders().unwrap();
    }
}

/// Registers each photo of the session with the fake Capture One (the `p58` pattern) and lets
/// their jobs and renders run.
fn settle(a: &TestApp, session: i64) {
    for photo in a.store().photos_in_session(session).unwrap() {
        if photo.classification != C::Deleted {
            a.fake
                .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
        }
    }
    drain(a);
}

/// The archived item (`main` + `condition`) of U2 brought back, its files registered with the
/// fake Capture One, its jobs and renders done.
fn brought_back(a: &TestApp, sku: &str, main: usize, condition: usize) -> Remote {
    let r = archived_of(a, sku, "Sideboard", main, condition);
    a.remote().bring_back(r.session).unwrap();
    settle(a, r.session);
    r
}

fn publish(a: &TestApp, session: i64) {
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm_as_shown(&[session], "Publish images")
        .unwrap();
    drain(a);
    assert_eq!(
        a.store().latest_publish(session).unwrap().map(|l| l.step),
        Some(PublishStep::Done),
        "{:?}",
        a.store().latest_publish(session).unwrap()
    );
}

/// The mutation ops recorded since `mark` (a length of `mutation_ops()`).
fn ops_since(a: &TestApp, mark: usize) -> Vec<String> {
    a.fake_shop().mutation_ops()[mark..].to_vec()
}

fn uploads_since(a: &TestApp, mark: usize) -> usize {
    ops_since(a, mark)
        .iter()
        .filter(|op| *op == "upload")
        .count()
}

fn count(ops: &[String], op: &str) -> usize {
    ops.iter().filter(|o| *o == op).count()
}

fn product(a: &TestApp, r: &Remote) -> String {
    a.store()
        .item_link(r.item, "shopify")
        .unwrap()
        .unwrap()
        .external_id
}

fn gallery(a: &TestApp, r: &Remote) -> Vec<String> {
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .products
        .iter()
        .find(|p| p.external_id == product(a, r))
        .unwrap()
        .media
        .iter()
        .map(|m| m.id.clone())
        .collect()
}

fn condition_field(a: &TestApp, r: &Remote) -> Vec<String> {
    let product = product(a, r);
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .values
        .get(&product)
        .and_then(|rows| rows.iter().find(|(k, _)| k == CONDITION_KEY).cloned())
        .and_then(|(_, v)| v.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect()
}

/// The brought-back photos: main in order, then condition in order.
fn photos(a: &TestApp, session: i64) -> (Vec<Photo>, Vec<Photo>) {
    let all = a.sessions().view(session).unwrap().photos;
    (
        all.iter()
            .filter(|p| p.classification == C::Main)
            .cloned()
            .collect(),
        all.iter()
            .filter(|p| p.classification == C::Condition)
            .cloned()
            .collect(),
    )
}

/// A tighter crop of the whole frame (shape free).
fn tighter(a: &TestApp, photo: &Photo) {
    let current = a.composition(photo.id).unwrap().values().unwrap();
    a.app
        .compositions
        .set(
            photo.id,
            CompositionValues {
                shape: Shape::Free,
                rect: Rect {
                    x: 0.1,
                    y: 0.1,
                    w: 0.8,
                    h: 0.8,
                },
                ..current
            },
        )
        .unwrap();
}

fn link(
    a: &TestApp,
    r: &Remote,
    photo: &Photo,
    role: &str,
) -> psm_core::domain::publish::PhotoLink {
    a.store()
        .photo_link(photo.id, r.item, role, OutputSlot::Single)
        .unwrap()
        .unwrap()
}

/// Media ids that are not among `known`.
fn new_ids(ids: &[String], known: &[String]) -> Vec<String> {
    ids.iter()
        .filter(|id| !known.contains(id))
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------------------------
// H3·C10 — nothing changed, nothing uploaded

#[test]
fn h3_c10a_an_untouched_bring_back_publishes_without_uploading() {
    let a = shop_app();
    let r = brought_back(&a, "BB-10A", 3, 2);
    let mark = a.fake_shop().mutation_ops().len();
    publish(&a, r.session);
    let ops = ops_since(&a, mark);
    for op in [
        "staged_uploads_create",
        "upload",
        "product_create_media",
        "file_create",
        "product_delete_media",
    ] {
        assert_eq!(count(&ops, op), 0, "{op} in {ops:?}");
    }
    assert_eq!(gallery(&a, &r), r.media[..3].to_vec());
    assert_eq!(condition_field(&a, &r), r.media[3..].to_vec());
}

#[test]
fn h3_c10b_a_first_settings_file_is_not_an_edit() {
    let a = shop_app();
    let r = brought_back(&a, "BB-10B", 3, 2);
    let (main, _) = photos(&a, r.session);
    tighter(&a, &main[1]);
    a.fake
        .rewrite_settings(
            &main[0].raw_path,
            UNIX_EPOCH + Duration::from_millis(a.clock.now_ms() as u64 + 1_000),
        )
        .unwrap();
    a.focus().unwrap();
    let mark = a.fake_shop().mutation_ops().len();
    publish(&a, r.session);
    assert_eq!(uploads_since(&a, mark), 1);
    let first = link(&a, &r, &main[0], "gallery");
    assert_eq!(first.upload_state, "ready");
    assert_eq!(first.external_id.as_deref(), Some(r.media[0].as_str()));
    assert_eq!(first.revision, 0);
    let after = gallery(&a, &r);
    assert_eq!(after.len(), 3);
    assert_eq!((&after[0], &after[2]), (&r.media[0], &r.media[2]));
    assert_eq!(new_ids(&after, &r.media).len(), 1);
}

#[test]
fn h3_c10c_preview_announces_no_removal() {
    let a = shop_app();
    let r = brought_back(&a, "BB-10C", 3, 2);
    let preview = a
        .app
        .publish
        .as_ref()
        .unwrap()
        .preview(&[r.session])
        .unwrap();
    assert!(preview.items[0].announced_removals.is_empty());
}

// ---------------------------------------------------------------------------------------------
// H3·C11 — a changed photo replaces its media

#[test]
fn h3_c11a_a_tighter_crop_replaces_that_media_only() {
    let a = shop_app();
    let r = brought_back(&a, "BB-11A", 3, 2);
    let (main, _) = photos(&a, r.session);
    tighter(&a, &main[1]);
    let mark = a.fake_shop().mutation_ops().len();
    publish(&a, r.session);
    assert_eq!(uploads_since(&a, mark), 1);
    let after = gallery(&a, &r);
    let n = new_ids(&after, &r.media);
    assert_eq!(n.len(), 1);
    assert_eq!(
        after,
        vec![r.media[0].clone(), n[0].clone(), r.media[2].clone()]
    );
    let second = link(&a, &r, &main[1], "gallery");
    assert_eq!(second.upload_state, "ready");
    assert_eq!(second.external_id.as_deref(), Some(n[0].as_str()));
    assert_eq!(second.revision, 1);
    assert!(second.superseded_ids.contains(&r.media[1]));
}

#[test]
fn h3_c11b_a_quarter_turn_replaces_that_media_only() {
    let a = shop_app();
    let r = brought_back(&a, "BB-11B", 3, 2);
    let (main, _) = photos(&a, r.session);
    let c = a.composition(main[2].id).unwrap();
    let frame = c.frame().unwrap();
    let values = CompositionValues {
        kind: CompositionKind::Normal,
        shape: Shape::Original,
        orientation: Orientation::Named,
        quarter_turns: 1,
        flip: Flip::None,
        straighten: 0.0,
        rect: default_rect(
            CompositionKind::Normal,
            Shape::Original,
            Orientation::Named,
            turned_frame(frame, 1),
        ),
    };
    a.app.compositions.set(main[2].id, values).unwrap();
    let mark = a.fake_shop().mutation_ops().len();
    publish(&a, r.session);
    assert_eq!(uploads_since(&a, mark), 1);
    let after = gallery(&a, &r);
    let n = new_ids(&after, &r.media);
    assert_eq!(
        after,
        vec![r.media[0].clone(), r.media[1].clone(), n[0].clone()]
    );
}

#[test]
fn h3_c11c_a_capture_one_edit_replaces_that_media_only() {
    let a = shop_app();
    let r = brought_back(&a, "BB-11C", 3, 2);
    let (main, _) = photos(&a, r.session);
    a.store().adjusted_event(&main[0].raw_path, true).unwrap();
    let mark = a.fake_shop().mutation_ops().len();
    publish(&a, r.session);
    assert_eq!(uploads_since(&a, mark), 1);
    let after = gallery(&a, &r);
    let n = new_ids(&after, &r.media);
    assert_eq!(
        after,
        vec![n[0].clone(), r.media[1].clone(), r.media[2].clone()]
    );
}

#[test]
fn h3_c11d_a_split_replaces_one_media_with_two() {
    let a = shop_app();
    let r = brought_back(&a, "BB-11D", 3, 2);
    let (main, _) = photos(&a, r.session);
    let frame = a.composition(main[0].id).unwrap().frame().unwrap();
    let rect = default_rect(
        CompositionKind::Split,
        Shape::Square,
        Orientation::Named,
        turned_frame(frame, 0),
    );
    a.app
        .compositions
        .set(
            main[0].id,
            CompositionValues {
                kind: CompositionKind::Split,
                shape: Shape::Square,
                orientation: Orientation::Named,
                quarter_turns: 0,
                flip: Flip::None,
                straighten: 0.0,
                rect,
            },
        )
        .unwrap();
    let mark = a.fake_shop().mutation_ops().len();
    publish(&a, r.session);
    assert_eq!(uploads_since(&a, mark), 2);
    let after = gallery(&a, &r);
    let n = new_ids(&after, &r.media);
    assert_eq!(n.len(), 2);
    assert_eq!(
        after,
        vec![
            n[0].clone(),
            n[1].clone(),
            r.media[1].clone(),
            r.media[2].clone()
        ]
    );
    let left = a
        .store()
        .photo_link(main[0].id, r.item, "gallery", OutputSlot::Left)
        .unwrap()
        .unwrap();
    assert_eq!(left.external_id.as_deref(), Some(n[0].as_str()));
    assert!(!after.contains(&r.media[0]));
}

// ---------------------------------------------------------------------------------------------
// H3·C12 — order, hiding, condition, new photos

#[test]
fn h3_c12a_a_new_order_reorders_without_uploading() {
    let a = shop_app();
    let r = brought_back(&a, "BB-12A", 3, 2);
    let (main, _) = photos(&a, r.session);
    a.review().reorder(main[2].id, 1).unwrap();
    let mark = a.fake_shop().mutation_ops().len();
    publish(&a, r.session);
    let ops = ops_since(&a, mark);
    assert_eq!(count(&ops, "upload"), 0, "{ops:?}");
    assert_eq!(count(&ops, "product_reorder_media"), 1, "{ops:?}");
    assert_eq!(
        gallery(&a, &r),
        vec![r.media[2].clone(), r.media[0].clone(), r.media[1].clone()]
    );
}

#[test]
fn h3_c12b_a_hidden_photo_removes_its_media() {
    let a = shop_app();
    let r = brought_back(&a, "BB-12B", 3, 2);
    let (main, _) = photos(&a, r.session);
    a.review().delete(main[1].id).unwrap();
    let mark = a.fake_shop().mutation_ops().len();
    publish(&a, r.session);
    assert_eq!(uploads_since(&a, mark), 0);
    assert_eq!(
        gallery(&a, &r),
        vec![r.media[0].clone(), r.media[2].clone()]
    );
}

#[test]
fn h3_c12c_a_changed_condition_photo_replaces_its_file() {
    let a = shop_app();
    let r = brought_back(&a, "BB-12C", 3, 2);
    let (_, condition) = photos(&a, r.session);
    tighter(&a, &condition[0]);
    let mark = a.fake_shop().mutation_ops().len();
    publish(&a, r.session);
    let ops = ops_since(&a, mark);
    assert_eq!(count(&ops, "upload"), 1, "{ops:?}");
    assert_eq!(count(&ops, "file_create"), 1, "{ops:?}");
    let field = condition_field(&a, &r);
    assert_eq!(field.len(), 2);
    assert_ne!(field[0], r.media[3]);
    assert_eq!(field[1], r.media[4]);
    assert_eq!(gallery(&a, &r), r.media[..3].to_vec());
}

#[test]
fn h3_c12d_a_new_photo_is_added_after_the_others() {
    let a = shop_app();
    let r = brought_back(&a, "BB-12D", 3, 2);
    a.clock.advance(60_000);
    let (bytes, _) =
        SyntheticRaw::build_with_frame(&[(1600, 1067, true)], 1, None, Some((6000, 4000)));
    let raw = r.folder.join("DSC07001.ARW");
    fs::write(&raw, bytes).unwrap();
    a.watcher_event(&raw).unwrap();
    a.tick(1_000).unwrap();
    let canon = fs::canonicalize(&raw).unwrap();
    let arrived = a
        .store()
        .photos_in_session(r.session)
        .unwrap()
        .into_iter()
        .find(|p| Path::new(&p.raw_path) == canon)
        .expect("arrival filed into the session");
    assert_eq!(
        (arrived.classification, arrived.position),
        (C::Main, Some(4))
    );
    settle(&a, r.session);
    let mark = a.fake_shop().mutation_ops().len();
    publish(&a, r.session);
    assert_eq!(uploads_since(&a, mark), 1);
    let after = gallery(&a, &r);
    let n = new_ids(&after, &r.media);
    assert_eq!(
        after,
        vec![
            r.media[0].clone(),
            r.media[1].clone(),
            r.media[2].clone(),
            n[0].clone()
        ]
    );
}

// ---------------------------------------------------------------------------------------------
// H3·C13 — a shop-only item's media count as the app's own

fn shop_only_brought_back(a: &TestApp, sku: &str) -> Remote {
    let r = adopted(a, sku, &["m1", "m2"], &[]);
    a.remote().ensure_record(r.session).unwrap();
    a.remote().bring_back(r.session).unwrap();
    settle(a, r.session);
    r
}

#[test]
fn h3_c13a_reordering_shop_only_media_is_not_foreign() {
    let a = shop_app();
    let r = shop_only_brought_back(&a, "SO-13A");
    let (main, _) = photos(&a, r.session);
    a.review().reorder(main[1].id, 1).unwrap();
    let mark = a.fake_shop().mutation_ops().len();
    publish(&a, r.session);
    assert_eq!(uploads_since(&a, mark), 0);
    assert_eq!(gallery(&a, &r), vec!["m2".to_string(), "m1".to_string()]);
}

#[test]
fn h3_c13b_replacing_shop_only_media() {
    let a = shop_app();
    let r = shop_only_brought_back(&a, "SO-13B");
    let (main, _) = photos(&a, r.session);
    tighter(&a, &main[0]);
    let mark = a.fake_shop().mutation_ops().len();
    publish(&a, r.session);
    assert_eq!(uploads_since(&a, mark), 1);
    let after = gallery(&a, &r);
    let n = new_ids(&after, &r.media);
    assert_eq!(after, vec![n[0].clone(), "m2".to_string()]);
    assert!(!after.contains(&"m1".to_string()));
}

#[test]
fn h3_c13c_preview_of_a_shop_only_bring_back() {
    let a = shop_app();
    let r = adopted(&a, "SO-13C", &["m1", "m2"], &[]);
    a.remote().ensure_record(r.session).unwrap();
    a.remote().bring_back(r.session).unwrap();
    let preview = a
        .app
        .publish
        .as_ref()
        .unwrap()
        .preview(&[r.session])
        .unwrap();
    assert!(preview.items[0].announced_removals.is_empty());
    assert!(!preview.items[0].create_item);
}

// ---------------------------------------------------------------------------------------------
// H3·C14 — retention after bring back

fn files_exist(a: &TestApp, r: &Remote) -> Vec<bool> {
    a.store()
        .photos_in_session(r.session)
        .unwrap()
        .iter()
        .filter(|p| p.classification != C::Deleted)
        .map(|p| Path::new(&p.raw_path).is_file())
        .collect()
}

fn readbacks(a: &TestApp) -> usize {
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .reads
        .iter()
        .filter(|r| r.as_str() == "media_readback")
        .count()
}

/// The brought-back item untouched 30 days, scanned: not cleared.
fn untouched_month(a: &TestApp) -> Remote {
    let r = brought_back(a, "BB-14", 3, 2);
    let reads = readbacks(a);
    a.clock.advance(30 * DAY);
    clear_all(a);
    assert_eq!(readbacks(a), reads, "no readback for an item not due");
    r
}

/// After `untouched_month`: photo 2 re-framed and published; the clock past the retention days.
fn changed_and_published(a: &TestApp, r: &Remote) -> String {
    let (main, _) = photos(a, r.session);
    tighter(a, &main[1]);
    publish(a, r.session);
    let n = new_ids(&gallery(a, r), &r.media);
    assert_eq!(n.len(), 1);
    a.clock.advance(4 * DAY + 60_000);
    n[0].clone()
}

#[test]
fn h3_c14a_an_untouched_bring_back_is_not_cleared() {
    let a = shop_app();
    let r = untouched_month(&a);
    assert_eq!(files_exist(&a, &r), vec![true; 5]);
    assert_eq!(a.store().item_retention(r.item).unwrap().cleared_at, None);
}

#[test]
fn h3_c14b_the_next_publish_makes_it_eligible_again() {
    let a = shop_app();
    let r = untouched_month(&a);
    let paths: Vec<String> = photos(&a, r.session)
        .0
        .iter()
        .chain(photos(&a, r.session).1.iter())
        .map(|p| p.raw_path.clone())
        .collect();
    let n = changed_and_published(&a, &r);
    clear_all(&a);
    assert!(
        a.store()
            .item_retention(r.item)
            .unwrap()
            .cleared_at
            .is_some()
    );
    for path in &paths {
        assert!(!Path::new(path).exists(), "{path}");
    }
    let record: Vec<(String, String)> = a
        .store()
        .archived_images(r.item)
        .unwrap()
        .into_iter()
        .map(|i| (i.role, i.media_id))
        .collect();
    assert_eq!(
        record,
        vec![
            ("gallery".into(), r.media[0].clone()),
            ("gallery".into(), n),
            ("gallery".into(), r.media[2].clone()),
            ("condition".into(), r.media[3].clone()),
            ("condition".into(), r.media[4].clone()),
        ]
    );
}

#[test]
fn h3_c14c_keep_files_holds_it() {
    let a = shop_app();
    let r = untouched_month(&a);
    changed_and_published(&a, &r);
    a.store().set_item_keep_files(r.item, true).unwrap();
    clear_all(&a);
    assert_eq!(a.store().item_retention(r.item).unwrap().cleared_at, None);
    assert_eq!(files_exist(&a, &r), vec![true; 5]);
}

#[test]
fn h3_c14d_a_second_bring_back_takes_new_names() {
    let a = shop_app();
    let r = untouched_month(&a);
    changed_and_published(&a, &r);
    clear_all(&a);
    assert!(
        a.store()
            .item_retention(r.item)
            .unwrap()
            .cleared_at
            .is_some()
    );
    register_pictures(&a, r.item);
    a.remote().bring_back(r.session).unwrap();
    let (main, condition) = photos(&a, r.session);
    assert_eq!((main.len(), condition.len()), (3, 2));
    let key = r.folder.file_name().unwrap().to_string_lossy().into_owned();
    let names: Vec<String> = main
        .iter()
        .chain(condition.iter())
        .map(|p| {
            Path::new(&p.raw_path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(
        names,
        vec![
            format!("{key}-main-01-b2.jpg"),
            format!("{key}-main-02-b2.jpg"),
            format!("{key}-main-03-b2.jpg"),
            format!("{key}-condition-01-b2.jpg"),
            format!("{key}-condition-02-b2.jpg"),
        ]
    );
}
