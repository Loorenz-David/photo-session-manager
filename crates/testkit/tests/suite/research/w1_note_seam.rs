//! W1·U3 (rows W1·C13(a), C13(b)): the product note and Look again through the wired app. The
//! boundary is the wired `TestApp` (the real `App` start-up path) and the writer handle the
//! commands `writer_note` and `writer_look_again` call (`AppHandles::writer`).

use std::fs;

use psm_core::{
    domain::{
        attribute::{AiConfidence, AiStatus, Provenance, Value},
        composition::{CompositionValues, Origin, Rect},
        photo::{Classification as C, NewPhoto, Photo, PhotoSource},
    },
    mechanisms::hash::sha256_file,
    ports::clock::Clock,
};
use psm_research::{
    domain::{RunState, RunTrigger, Stage},
    stages::OBSERVE_UNIT,
    store as rstore,
    writer::{api::WriterApi, note::LookBlock},
};
use psm_testkit::{TestApp, fixtures::synthetic_jpeg::SyntheticJpeg};
use serde_json::json;

const NOW_MS: i64 = 1_763_500_000_000;
const DESIGNER: &str = "custom.designer";

/// A RAW on disk known to the fake Capture One document, classified, with its adjusted preview.
fn photo(a: &TestApp, session_id: i64, name: &str, class: C) -> Photo {
    let dir = a.dir.path().join("Capture/raws");
    fs::create_dir_all(&dir).unwrap();
    let raw = dir.join(format!("{name}.ARW"));
    fs::write(&raw, format!("raw bytes of {name}")).unwrap();
    let photo = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id,
            raw_path: raw.to_string_lossy().into_owned(),
            raw_hash: Some(sha256_file(&raw).unwrap()),
            captured_at: Some(a.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: a.clock.now_ms(),
        })
        .unwrap();
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
    a.review().classify(photo.id, class).unwrap();
    a.store().adjusted_event(&photo.raw_path, false).unwrap();
    let generation = a.store().adjusted_gen(&photo.raw_path).unwrap();
    let path = a
        .dir
        .path()
        .join(format!("adjusted-{}-g{generation}.jpg", photo.id));
    fs::write(&path, SyntheticJpeg::rgb(1600, 1200, [180, 90, 40])).unwrap();
    a.store()
        .complete_preview(
            &photo.raw_path,
            generation,
            path.to_str().unwrap(),
            None,
            None,
        )
        .unwrap();
    a.store().photo(photo.id).unwrap()
}

fn writer(a: &TestApp) -> std::sync::Arc<dyn WriterApi> {
    a.app
        .handles()
        .writer
        .expect("the wired app hands the writer")
}

/// A researched session (fakes): one Main photo, a finished arranged run whose note has one
/// observation tagged with the designer field, which the owner accepted. Returns
/// `(session, photo)`.
fn researched(a: &TestApp) -> (i64, i64) {
    a.clock.advance(NOW_MS);
    let (item, session) = a
        .items()
        .create_local("W1-SEAM", "Teak chair", "Chair", None)
        .unwrap();
    let s = session.session.id;
    let main = photo(a, s, "w1-main", C::Main);
    a.research_fakes().model.reply(
        Stage::Observe,
        OBSERVE_UNIT,
        json!({"reads": {}, "marking_photos": [], "note": [
            {"text": "Designed by Kai Kristiansen", "keys": [DESIGNER]},
        ]}),
    );
    assert!(a.app.research.as_ref().unwrap().arranged(s).unwrap());
    a.tick(1_100).unwrap();
    let runs = a
        .app
        .store
        .research_transaction(|tx| rstore::runs(tx.conn(), Some(s)))
        .unwrap();
    assert_eq!(runs.last().unwrap().state, RunState::Finished);
    a.store()
        .transaction(&mut |tx| {
            tx.put_attribute_value(&Value {
                item_id: item.id,
                key: DESIGNER.into(),
                value: json!("Hans Wegner").to_string(),
                valid: true,
                provenance: Provenance::Ai,
                ai_confidence: Some(AiConfidence::Medium),
                ai_status: Some(AiStatus::Accepted),
                ai_reason: None,
                ai_sources: None,
                ai_prior: None,
                updated_at: NOW_MS,
            })
        })
        .unwrap();
    (s, main.id)
}

#[test]
fn w1_c13a_the_note_shows_the_owners_decision() {
    let a = TestApp::with_research().unwrap();
    let (s, _) = researched(&a);
    let note = writer(&a).note(s).unwrap();
    assert_eq!(note.observations.len(), 1);
    assert!(note.observations[0].owner);
    assert_eq!(
        note.observations[0].text,
        format!("{DESIGNER}: Hans Wegner (owner's decision)")
    );
    assert!(!note.stale_main);
}

#[test]
fn w1_c13b_look_again_after_a_recompose_queues_one_look_run() {
    let a = TestApp::with_research().unwrap();
    let (s, main) = researched(&a);
    let current = a.composition(main).unwrap();
    a.store()
        .save_composition(
            main,
            &CompositionValues {
                kind: current.kind,
                shape: current.shape,
                orientation: current.orientation,
                quarter_turns: current.quarter_turns,
                flip: current.flip,
                straighten: current.straighten,
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 0.5,
                    h: 1.0,
                },
            },
            Origin::User,
            a.clock.now_ms(),
        )
        .unwrap();
    let first = writer(&a).look_again(s).unwrap();
    assert!(first.queued);
    assert_eq!(first.reason, None);
    let looks = a
        .app
        .store
        .research_transaction(|tx| rstore::all_runs(tx.conn(), Some(s)))
        .unwrap()
        .into_iter()
        .filter(|r| r.trigger == RunTrigger::Look)
        .count();
    assert_eq!(looks, 1);
    let second = writer(&a).look_again(s).unwrap();
    assert!(!second.queued);
    assert_eq!(second.reason, Some(LookBlock::Busy.reason()));
}
