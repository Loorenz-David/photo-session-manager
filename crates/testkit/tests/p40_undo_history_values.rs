use psm_core::{
    Error,
    domain::{
        attribute::{AiConfidence, AiStatus, Provenance, Value},
        composition::{
            CompositionKind, CompositionState, CompositionValues, Flip, Orientation, Rect, Shape,
        },
        photo::{Classification, Photo},
    },
    events::AppEvent,
    services::{attributes::PRODUCT_TITLE, history::HistoryState},
};
use psm_testkit::{
    TestApp,
    fixtures::{
        definitions::{INTEGRATION, shop_definitions},
        synthetic_jpeg::SyntheticJpeg,
    },
};
use serde_json::json;

const K1: &str = "custom.color_furniture";
const K2: &str = "custom.condition_report";

fn attr() -> (TestApp, i64, i64) {
    let a = TestApp::with_fake_document().unwrap();
    a.attributes(Some(INTEGRATION))
        .import_definitions(INTEGRATION, shop_definitions())
        .unwrap();
    let (item, session) = a
        .items()
        .create_local("K", "Oak", "Sideboard", None)
        .unwrap();
    (a, item.id, session.session.id)
}
fn photo(a: &TestApp, s: i64, name: &str) -> Photo {
    let p = a.add_photo(s, name).unwrap();
    a.store().upsert_raw_state(&p.raw_path, None).unwrap();
    a.fake
        .add_image(&p.raw_path, &[&format!("v{}", p.id)], false);
    a.app.previews.request_initial(&p.raw_path).unwrap();
    let job = a
        .fake
        .state
        .lock()
        .unwrap()
        .issued
        .last()
        .unwrap()
        .0
        .clone();
    a.complete_render_with(&job, &SyntheticJpeg::rgb(2560, 1707, [80, 80, 80]))
        .unwrap();
    p
}
fn v1() -> CompositionValues {
    CompositionValues {
        kind: CompositionKind::Normal,
        shape: Shape::Square,
        orientation: Orientation::Named,
        quarter_turns: 0,
        flip: Flip::None,
        straighten: 0.0,
        rect: Rect {
            x: 0.25,
            y: (1.0 - 1280.0 / 1707.0) / 2.0,
            w: 0.5,
            h: 1280.0 / 1707.0,
        },
    }
}
fn row(a: &TestApp, item: i64, key: &str) -> Option<Value> {
    let mut result = None;
    a.store()
        .transaction(&mut |tx| {
            result = tx
                .attribute_values(item)?
                .into_iter()
                .find(|v| v.key == key);
            Ok(())
        })
        .unwrap();
    result
}
fn hs(a: &TestApp, s: i64) -> HistoryState {
    a.history().state(s)
}
fn seq(a: &TestApp, s: i64) -> i64 {
    a.store().session(s).unwrap().session.change_seq
}
fn set(a: &TestApp, item: i64, key: &str, text: &str) {
    a.attributes(Some(INTEGRATION))
        .set_value(item, key, json!(text), Provenance::Human)
        .unwrap();
}
fn err(error: Error, direction: &str, label: &str, reason: &str) {
    match error {
        Error::StepCannotApply {
            direction: d,
            label: l,
            reason: r,
        } => assert_eq!(
            (d.as_str(), l.as_str(), r.as_str()),
            (direction, label, reason)
        ),
        e => panic!("{e:?}"),
    }
}

#[test]
fn c1a_composition_undo_saves_new_revision() {
    let (a, _, s) = attr();
    let p = photo(&a, s, "a.ARW");
    a.review().classify(p.id, Classification::Main).unwrap();
    let c0 = a.composition(p.id).unwrap();
    a.app.compositions.set(p.id, v1()).unwrap();
    let n = seq(&a, s);
    let event_count = a.events.0.lock().unwrap().len();
    let out = a.history().undo(s).unwrap();
    let c = a.composition(p.id).unwrap();
    assert_eq!(out.label, "Compose 01");
    assert_eq!(out.active_photo_id, None);
    assert_eq!(
        out.sessions
            .iter()
            .map(|x| x.session.id)
            .collect::<Vec<_>>(),
        vec![s]
    );
    assert_eq!(c.values(), c0.values());
    assert_eq!(c.origin, c0.origin);
    assert_eq!(c.revision, 2);
    assert_eq!(c.state, CompositionState::Ok);
    assert_eq!(seq(&a, s), n + 1);
    let events = a.events.0.lock().unwrap();
    assert!(events[event_count..].iter().any(|e| matches!(e,AppEvent::PhotoChanged{photo_id,session_id} if *photo_id==p.id && *session_id==s)));
    assert!(
        events[event_count..]
            .iter()
            .any(|e| matches!(e,AppEvent::SessionChanged{session_id} if *session_id==s))
    );
}
#[test]
fn c1b_composition_redo_saves_new_revision() {
    let (a, _, s) = attr();
    let p = photo(&a, s, "a.ARW");
    a.review().classify(p.id, Classification::Main).unwrap();
    a.app.compositions.set(p.id, v1()).unwrap();
    a.history().undo(s).unwrap();
    a.history().redo(s).unwrap();
    let c = a.composition(p.id).unwrap();
    assert_eq!(c.values(), Some(v1()));
    assert_eq!(c.origin, psm_core::domain::composition::Origin::User);
    assert_eq!(c.revision, 3);
    assert_eq!(
        hs(&a, s),
        HistoryState {
            undo: Some("Compose 01".into()),
            redo: None
        }
    );
}
#[test]
fn c1c_reset_is_step() {
    let (a, _, s) = attr();
    let p = photo(&a, s, "a.ARW");
    a.app.compositions.set(p.id, v1()).unwrap();
    a.app.compositions.reset(p.id).unwrap();
    a.history().undo(s).unwrap();
    let c = a.composition(p.id).unwrap();
    assert_eq!(c.values(), Some(v1()));
    assert_eq!(c.origin, psm_core::domain::composition::Origin::User);
    assert_eq!(c.revision, 3);
    assert_eq!(
        hs(&a, s),
        HistoryState {
            undo: Some("Compose photo".into()),
            redo: Some("Compose photo".into())
        }
    );
}
#[test]
fn c1d_same_composition_is_one_step() {
    let (a, _, s) = attr();
    let p = photo(&a, s, "a.ARW");
    let c0 = a.composition(p.id).unwrap();
    a.app.compositions.set(p.id, v1()).unwrap();
    a.app.compositions.set(p.id, v1()).unwrap();
    assert_eq!(a.composition(p.id).unwrap().revision, 1);
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Compose photo"));
    a.history().undo(s).unwrap();
    assert_eq!(a.composition(p.id).unwrap().values(), c0.values());
    assert_eq!(hs(&a, s).undo, None);
}
fn applied() -> (TestApp, i64, [Photo; 3]) {
    let (a, _, s) = attr();
    let p = photo(&a, s, "a.ARW");
    let q = photo(&a, s, "b.ARW");
    let r = photo(&a, s, "c.ARW");
    let mut v2 = v1();
    v2.rect.x = 0.3;
    let mut invalid = v1();
    invalid.rect = Rect {
        x: 0.2,
        y: 0.0,
        w: 0.5,
        h: 1.0,
    };
    let result = a
        .app
        .compositions
        .apply(vec![(q.id, v1()), (r.id, v2), (p.id, invalid)]);
    assert_eq!(result.applied, vec![q.id, r.id]);
    assert_eq!(
        result
            .refused
            .iter()
            .map(|v| v.reason.as_str())
            .collect::<Vec<_>>(),
        vec!["off_shape"]
    );
    (a, s, [p, q, r])
}
#[test]
fn c2a_apply_one_step_with_partial_refusal() {
    let (a, s, [p, q, r]) = applied();
    let default = a.composition(p.id).unwrap().values();
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Apply framing to 2 photos"));
    let out = a.history().undo(s).unwrap();
    assert_eq!(out.label, "Apply framing to 2 photos");
    assert_eq!(a.composition(q.id).unwrap().revision, 2);
    assert_eq!(a.composition(r.id).unwrap().revision, 2);
    assert_eq!(a.composition(p.id).unwrap().revision, 0);
    assert_eq!(a.composition(q.id).unwrap().values(), default);
    assert_eq!(a.composition(r.id).unwrap().values(), default);
    assert_eq!(
        hs(&a, s),
        HistoryState {
            undo: None,
            redo: Some("Apply framing to 2 photos".into())
        }
    );
}
#[test]
fn c2b_apply_redo() {
    let (a, s, [_p, q, r]) = applied();
    a.history().undo(s).unwrap();
    a.history().redo(s).unwrap();
    let mut v2 = v1();
    v2.rect.x = 0.3;
    assert_eq!(a.composition(q.id).unwrap().values(), Some(v1()));
    assert_eq!(a.composition(r.id).unwrap().values(), Some(v2));
    assert_eq!(
        (
            a.composition(q.id).unwrap().revision,
            a.composition(r.id).unwrap().revision
        ),
        (3, 3)
    );
}
#[test]
fn c3a_needs_review_drops_composition_step() {
    use psm_core::domain::composition::EditorGeometry;
    let (a, _, s) = attr();
    let p = photo(&a, s, "a.ARW");
    a.review().classify(p.id, Classification::Main).unwrap();
    a.app.compositions.set(p.id, v1()).unwrap();
    a.fake.set_variant_geometry(
        &format!("v{}", p.id),
        EditorGeometry {
            rotation: 3.0,
            ..Default::default()
        },
    );
    a.app
        .previews
        .on_variant_adjusted(&format!("v{}", p.id), &p.raw_path)
        .unwrap();
    a.tick(15_000).unwrap();
    let job = a
        .fake
        .state
        .lock()
        .unwrap()
        .issued
        .last()
        .unwrap()
        .0
        .clone();
    a.complete_render_with(&job, &SyntheticJpeg::rgb(2560, 1707, [80, 80, 80]))
        .unwrap();
    let rev = a.composition(p.id).unwrap().revision;
    err(
        a.history().undo(s).unwrap_err(),
        "undo",
        "Compose 01",
        "that photo changed in Capture One — re-frame it first",
    );
    let c = a.composition(p.id).unwrap();
    assert_eq!(c.state, CompositionState::NeedsReview);
    assert_eq!(c.revision, rev);
    assert_eq!(c.values(), Some(v1()));
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Approve 01"));
}
#[test]
fn c4a_ai_row_restored_byte_for_byte_without_dirty() {
    let (a, item, s) = attr();
    let attrs = a.attributes(Some(INTEGRATION));
    attrs
        .ai_suggest(
            item,
            K1,
            json!("Teak"),
            AiConfidence::High,
            Some("label"),
            vec!["https://x".into()],
        )
        .unwrap();
    let ai = row(&a, item, K1);
    a.clock.advance(1_000);
    set(&a, item, K1, "Oak");
    let last = a.store().session(s).unwrap().session.last_activity_at;
    let n = seq(&a, s);
    a.clock.advance(1_000);
    let out = a.history().undo(s).unwrap();
    assert_eq!(out.label, "Edit Color");
    assert_eq!(out.active_photo_id, None);
    assert_eq!(row(&a, item, K1), ai);
    assert!(a.store().session(s).unwrap().session.last_activity_at > last);
    assert_eq!(seq(&a, s), n);
}
#[test]
fn c4b_first_value_undo_removes_row() {
    let (a, item, s) = attr();
    set(&a, item, K2, "Scuffed");
    a.history().undo(s).unwrap();
    assert_eq!(row(&a, item, K2), None);
    assert!(
        !a.attributes(Some(INTEGRATION))
            .values(item)
            .unwrap()
            .iter()
            .any(|v| v.key == K2)
    );
    assert_eq!(
        hs(&a, s),
        HistoryState {
            undo: None,
            redo: Some("Edit Condition report".into())
        }
    );
}
#[test]
fn c4c_redo_reinstates_human_row() {
    let (a, item, s) = attr();
    a.attributes(Some(INTEGRATION))
        .ai_suggest(
            item,
            K1,
            json!("Teak"),
            AiConfidence::High,
            Some("label"),
            vec!["https://x".into()],
        )
        .unwrap();
    set(&a, item, K1, "Oak");
    a.history().undo(s).unwrap();
    a.history().redo(s).unwrap();
    let r = row(&a, item, K1).unwrap();
    assert_eq!(r.value, "\"Oak\"");
    assert_eq!(r.provenance, Provenance::Human);
    assert_eq!(r.ai_status, None);
    assert_eq!(r.ai_confidence, None);
}
#[test]
fn c4d_title_undo_redo() {
    let (a, item, s) = attr();
    let n = seq(&a, s);
    set(&a, item, PRODUCT_TITLE, "Oak table");
    a.history().undo(s).unwrap();
    assert_eq!(a.store().item(item).unwrap().title, "Oak");
    assert_eq!(hs(&a, s).redo.as_deref(), Some("Edit Title"));
    a.history().redo(s).unwrap();
    assert_eq!(a.store().item(item).unwrap().title, "Oak table");
    assert_eq!(seq(&a, s), n);
}
#[test]
fn c4e_typing_coalesces() {
    let (a, item, s) = attr();
    for text in ["O", "Oa", "Oak"] {
        set(&a, item, K1, text)
    }
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Edit Color"));
    a.history().undo(s).unwrap();
    assert_eq!(row(&a, item, K1), None);
    assert_eq!(
        hs(&a, s),
        HistoryState {
            undo: None,
            redo: Some("Edit Color".into())
        }
    );
}
#[test]
fn c4f_other_field_breaks_coalescing() {
    let (a, item, s) = attr();
    set(&a, item, K1, "Oak");
    set(&a, item, K2, "Scuffed");
    set(&a, item, K1, "Teak");
    a.history().undo(s).unwrap();
    assert_eq!(row(&a, item, K1).unwrap().value, "\"Oak\"");
    assert_eq!(row(&a, item, K2).unwrap().value, "\"Scuffed\"");
    a.history().undo(s).unwrap();
    assert_eq!(row(&a, item, K2), None);
    a.history().undo(s).unwrap();
    assert_eq!(row(&a, item, K1), None);
    assert_eq!(hs(&a, s).undo, None);
}
#[test]
fn c5a_accept_all_is_one_step() {
    let (a, item, s) = attr();
    let attrs = a.attributes(Some(INTEGRATION));
    for (key, confidence) in [
        (K1, AiConfidence::High),
        (K2, AiConfidence::Medium),
        ("custom.condition_cracks", AiConfidence::Low),
    ] {
        attrs
            .ai_suggest(item, key, json!("Minor"), confidence, None, vec![])
            .unwrap();
    }
    assert_eq!(attrs.accept_all(item).unwrap(), 2);
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Accept all suggestions"));
    a.history().undo(s).unwrap();
    for key in [K1, K2, "custom.condition_cracks"] {
        assert_eq!(
            row(&a, item, key).unwrap().ai_status,
            Some(AiStatus::Suggested)
        );
    }
    assert_eq!(
        hs(&a, s),
        HistoryState {
            undo: None,
            redo: Some("Accept all suggestions".into())
        }
    );
}
#[test]
fn c5b_clear_is_separate_step() {
    let (a, item, s) = attr();
    set(&a, item, K1, "Oak");
    a.attributes(Some(INTEGRATION)).clear(item, K1).unwrap();
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Clear Color"));
    a.history().undo(s).unwrap();
    let r = row(&a, item, K1).unwrap();
    assert_eq!(r.ai_status, None);
    assert!(r.is_live());
    assert_eq!(r.value, "\"Oak\"");
    assert_eq!(r.provenance, Provenance::Human);
    assert!(
        a.attributes(Some(INTEGRATION))
            .values(item)
            .unwrap()
            .iter()
            .any(|v| v.key == K1)
    );
    assert_eq!(
        hs(&a, s),
        HistoryState {
            undo: Some("Edit Color".into()),
            redo: Some("Clear Color".into())
        }
    );
    a.attributes(Some(INTEGRATION)).clear(item, K2).unwrap();
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Edit Color"));
}
#[test]
fn c6a_one_history_across_kinds() {
    let (a, item, s) = attr();
    let p = photo(&a, s, "a.ARW");
    let c0 = a.composition(p.id).unwrap();
    let u = a.add_photo(s, "u.ARW").unwrap();
    a.review().classify(p.id, Classification::Main).unwrap();
    a.review().classify(u.id, Classification::Main).unwrap();
    set(&a, item, K1, "Oak");
    a.app.compositions.set(p.id, v1()).unwrap();
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Compose 01"));
    a.history().undo(s).unwrap();
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Edit Color"));
    a.history().undo(s).unwrap();
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Approve 02"));
    a.history().undo(s).unwrap();
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Approve 01"));
    assert_eq!(a.composition(p.id).unwrap().values(), c0.values());
    assert_eq!(row(&a, item, K1), None);
    assert_eq!(a.store().photo(p.id).unwrap().position, Some(1));
    assert_eq!(
        a.store().photo(u.id).unwrap().classification,
        Classification::Unreviewed
    );
}
#[test]
fn c7a_attribute_events_only_history() {
    let (a, item, s) = attr();
    set(&a, item, K1, "Oak");
    a.history().undo(s).unwrap();
    let events = a.events.0.lock().unwrap();
    let history: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let AppEvent::HistoryChanged {
                session_id,
                undo,
                redo,
            } = e
            {
                Some((*session_id, undo.clone(), redo.clone()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        history,
        vec![
            (s, Some("Edit Color".into()), None),
            (s, None, Some("Edit Color".into()))
        ]
    );
    assert!(!events.iter().any(|e| matches!(
        e,
        AppEvent::PhotoChanged { .. } | AppEvent::SessionChanged { .. }
    )));
}
#[test]
fn c8a_copy_redo_missing_source_drops_step() {
    let a = TestApp::new().unwrap();
    let mut sessions = Vec::new();
    for name in ["r", "s", "t"] {
        sessions.push(
            a.items()
                .create_local(name, name, "object", None)
                .unwrap()
                .1
                .session
                .id,
        )
    }
    let [r, s, t]: [i64; 3] = sessions.try_into().unwrap();
    let ra = a.add_photo(r, "ra.ARW").unwrap();
    a.review().classify(ra.id, Classification::Main).unwrap();
    a.review().copy_photos(&[ra.id], s).unwrap();
    let cs = a.store().photos_in_session(s).unwrap()[0].id;
    a.review().copy_photos(&[cs], t).unwrap();
    a.history().undo(s).unwrap();
    a.history().undo(r).unwrap();
    err(
        a.history().redo(s).unwrap_err(),
        "redo",
        "Copy to t",
        "that photo is gone",
    );
    assert!(a.store().photos_in_session(t).unwrap().is_empty());
    assert_eq!(hs(&a, s), HistoryState::default());
    assert!(matches!(a.history().redo(s), Err(Error::NothingToRedo)));
}
