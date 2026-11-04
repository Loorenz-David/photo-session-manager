use psm_core::{
    Error,
    domain::{
        composition::{
            CompositionKind, CompositionState, CompositionValues, EditorGeometry, Flip, Frame,
            FrameSource, Orientation, Origin, Rect, ReviewReason, Shape,
        },
        photo::Classification,
    },
    events::AppEvent,
    mechanisms::arrival::path_key,
};
use psm_testkit::{TestApp, fixtures::synthetic_jpeg::SyntheticJpeg};
use rusqlite::Connection;
use std::{fs, path::Path};

fn session(a: &TestApp, name: &str) -> i64 {
    a.items()
        .create_local(name, name, "object", None)
        .unwrap()
        .1
        .session
        .id
}
fn add(a: &TestApp, s: i64, name: &str) -> psm_core::domain::photo::Photo {
    let photo = a.add_photo(s, name).unwrap();
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
    photo
}
fn issued(a: &TestApp) -> String {
    a.fake
        .state
        .lock()
        .unwrap()
        .issued
        .last()
        .unwrap()
        .0
        .clone()
}
fn preview(a: &TestApp, photo: &psm_core::domain::photo::Photo, w: u32, h: u32) {
    a.app.previews.request_initial(&photo.raw_path).unwrap();
    a.complete_render_with(&issued(a), &SyntheticJpeg::rgb(w, h, [200, 100, 50]))
        .unwrap();
}
fn rerender(a: &TestApp, photo: &psm_core::domain::photo::Photo, w: u32, h: u32) {
    a.app
        .previews
        .on_variant_adjusted(&format!("v{}", photo.id), &photo.raw_path)
        .unwrap();
    a.tick(20_000).unwrap();
    a.complete_render_with(&issued(a), &SyntheticJpeg::rgb(w, h, [200, 100, 50]))
        .unwrap();
}
fn square() -> Rect {
    Rect {
        x: (1.0 - 1707.0 / 2560.0) / 2.0,
        y: 0.0,
        w: 1707.0 / 2560.0,
        h: 1.0,
    }
}
fn values() -> CompositionValues {
    CompositionValues {
        kind: CompositionKind::Normal,
        shape: Shape::Square,
        orientation: Orientation::Turned,
        quarter_turns: 1,
        flip: Flip::Horizontal,
        straighten: 2.5,
        rect: Rect {
            x: 0.1,
            y: 0.3,
            w: 0.8,
            h: 1365.6 / 2560.0,
        },
    }
}
fn ready() -> (TestApp, i64, psm_core::domain::photo::Photo) {
    let a = TestApp::with_fake_document().unwrap();
    let s = session(&a, "S1");
    let p = add(&a, s, "a.ARW");
    preview(&a, &p, 2560, 1707);
    (a, s, p)
}
fn db(a: &TestApp) -> Connection {
    Connection::open(a.app.paths.db()).unwrap()
}
fn photo_events(a: &TestApp, id: i64) -> usize {
    a.events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter(|e| matches!(e,AppEvent::PhotoChanged{photo_id,..} if *photo_id==id))
        .count()
}

#[test]
fn c2a_capture_arrival_has_frameless_default() {
    let a = TestApp::with_fake_document().unwrap();
    let s = session(&a, "K1");
    a.sessions().activate(s).unwrap();
    let path = a.dir.path().join("Capture One Session/Capture/K1/a.ARW");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"raw").unwrap();
    a.fake.add_image(path.to_str().unwrap(), &["7"], false);
    a.deliver_hook(&serde_json::json!({"hook":"capture_done","argv":[path]}).to_string())
        .unwrap();
    let p = a.store().photos_in_session(s).unwrap().pop().unwrap();
    let c = a.composition(p.id).unwrap();
    assert_eq!(
        (
            c.kind,
            c.shape,
            c.orientation,
            c.quarter_turns,
            c.flip,
            c.straighten,
            c.rect,
            c.fingerprint,
            c.origin,
            c.revision,
            c.state,
            c.review_reason
        ),
        (
            CompositionKind::Normal,
            Shape::Square,
            Orientation::Named,
            0,
            Flip::None,
            0.0,
            None,
            None,
            Origin::Default,
            0,
            CompositionState::Ok,
            None
        )
    );
    assert_eq!(p.thumbnail_path, None);
    assert!(
        !a.fake
            .state
            .lock()
            .unwrap()
            .ops
            .iter()
            .any(|op| op.contains("crop"))
    );
}
#[test]
fn c2b_first_preview_materialises_default_and_thumbnail() {
    let (a, _, p) = ready();
    let c = a.composition(p.id).unwrap();
    assert_eq!(
        c.fingerprint.as_ref().unwrap().frame,
        Frame { w: 2560, h: 1707 }
    );
    assert_eq!(c.fingerprint.as_ref().unwrap().source, FrameSource::Preview);
    assert_eq!(
        c.fingerprint.as_ref().unwrap().geometry,
        EditorGeometry::default()
    );
    assert_eq!(c.rect, Some(square()));
    assert_eq!(
        (c.revision, c.origin, c.state),
        (0, Origin::Default, CompositionState::Ok)
    );
    let stored = a.store().photo(p.id).unwrap();
    let thumb = stored.thumbnail_path.unwrap();
    let key = path_key(&p.raw_path);
    let expected = a
        .app
        .paths
        .root
        .join("previews")
        .join(&key[..2])
        .join(key)
        .join("thumb-1.jpg");
    assert_eq!(Path::new(&thumb), expected);
    assert_eq!(image::image_dimensions(&thumb).unwrap(), (640, 427));
    assert!(Path::new(&stored.adjusted_preview_path.unwrap()).exists());
    assert_eq!(photo_events(&a, p.id), 1);
}
#[test]
fn c2c_original_default_uses_whole_frame() {
    let mut a = TestApp::with_fake_document().unwrap();
    a.settings_set(&serde_json::json!({"default_shape":"original"}))
        .unwrap();
    let s = session(&a, "S1");
    let p = add(&a, s, "a.ARW");
    preview(&a, &p, 2560, 1707);
    let c = a.composition(p.id).unwrap();
    assert_eq!(c.shape, Shape::Original);
    assert_eq!(
        c.rect,
        Some(Rect {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0
        })
    );
}
#[test]
fn c2d_copy_preserves_default_and_user_composition() {
    let (mut a, _, p) = ready();
    a.settings_set(&serde_json::json!({"default_shape":"4:5"}))
        .unwrap();
    let s2 = session(&a, "S2");
    a.review().copy_photos(&[p.id], s2).unwrap();
    let copied = a.store().photos_in_session(s2).unwrap().pop().unwrap();
    let mut c = a.composition(copied.id).unwrap();
    c.photo_id = p.id;
    assert_eq!(c, a.composition(p.id).unwrap());
    let s3 = session(&a, "S3");
    a.app.compositions.set(p.id, values()).unwrap();
    a.review().copy_photos(&[p.id], s3).unwrap();
    let copied = a.store().photos_in_session(s3).unwrap().pop().unwrap();
    let mut c = a.composition(copied.id).unwrap();
    c.photo_id = p.id;
    assert_eq!(c, a.composition(p.id).unwrap());
}
#[test]
fn c2e_undecodable_preview_keeps_composition_frameless() {
    let a = TestApp::with_fake_document().unwrap();
    let s = session(&a, "S1");
    let p = add(&a, s, "a.ARW");
    a.app.previews.request_initial(&p.raw_path).unwrap();
    a.complete_render(&issued(&a)).unwrap();
    let stored = a.store().photo(p.id).unwrap();
    assert!(stored.adjusted_preview_path.is_some());
    assert!(stored.thumbnail_path.is_none());
    let c = a.composition(p.id).unwrap();
    assert_eq!((c.rect, c.fingerprint), (None, None));
    assert_eq!(photo_events(&a, p.id), 1);
}
#[test]
fn c3a_valid_save_survives_restart() {
    let (mut a, _, p) = ready();
    let input = values();
    let saved = a.app.compositions.set(p.id, input.clone()).unwrap();
    assert_eq!(saved.values(), Some(input));
    assert_eq!(
        (saved.origin, saved.revision, saved.state),
        (Origin::User, 1, CompositionState::Ok)
    );
    assert_eq!(photo_events(&a, p.id), 2);
    a.kill_and_restart().unwrap();
    assert_eq!(a.composition(p.id).unwrap(), saved);
}
#[test]
fn c3b_invalid_saves_leave_row_unchanged() {
    let (a, _, p) = ready();
    let before = a.composition(p.id).unwrap();
    let events = photo_events(&a, p.id);
    let invalid = [
        (
            Rect {
                x: 0.2,
                y: 0.0,
                w: 0.666,
                h: 1.0,
            },
            0.0,
            CompositionKind::Normal,
            "off_shape",
        ),
        (
            Rect {
                x: 0.4,
                y: 0.0,
                w: square().w,
                h: 1.0,
            },
            0.0,
            CompositionKind::Normal,
            "rect_outside",
        ),
        (square(), 46.0, CompositionKind::Normal, "straighten_range"),
        (
            Rect {
                x: 0.0,
                y: 0.125,
                w: 1.0,
                h: 0.74,
            },
            0.0,
            CompositionKind::Split,
            "split_ratio",
        ),
    ];
    for (rect, straighten, kind, reason) in invalid {
        let mut v = CompositionValues {
            kind,
            shape: Shape::Square,
            orientation: Orientation::Named,
            quarter_turns: 0,
            flip: Flip::None,
            straighten,
            rect,
        };
        if kind == CompositionKind::Split {
            v.shape = Shape::Original;
        }
        assert!(
            matches!(a.app.compositions.set(p.id,v),Err(Error::CompositionInvalid{reason:r}) if r==reason)
        );
        assert_eq!(a.composition(p.id).unwrap(), before);
    }
    assert_eq!(photo_events(&a, p.id), events);
}
#[test]
fn c3c_identical_save_is_noop() {
    let (a, s, p) = ready();
    let v = values();
    let first = a.app.compositions.set(p.id, v.clone()).unwrap();
    let seq = a.store().session(s).unwrap().session.change_seq;
    let events = photo_events(&a, p.id);
    let second = a.app.compositions.set(p.id, v).unwrap();
    assert_eq!(second, first);
    assert_eq!(a.store().session(s).unwrap().session.change_seq, seq);
    assert_eq!(photo_events(&a, p.id), events);
}
#[test]
fn c3d_published_photo_composition_change_dirties_session() {
    let (a, s, p) = ready();
    a.review().classify(p.id, Classification::Main).unwrap();
    a.mark_published(s).unwrap();
    let before = a.store().session(s).unwrap();
    a.events.0.lock().unwrap().clear();
    a.app.compositions.set(p.id, values()).unwrap();
    let after = a.store().session(s).unwrap();
    assert!(after.dirty);
    assert_eq!(after.session.change_seq, before.session.published_seq + 1);
    assert_eq!(a.composition(p.id).unwrap().revision, 1);
    assert!(
        a.events
            .0
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e,AppEvent::SessionChanged{session_id} if *session_id==s))
    );
}
#[test]
fn c3e_deleted_photo_change_does_not_dirty() {
    let (a, s, p) = ready();
    a.review().delete(p.id).unwrap();
    a.mark_published(s).unwrap();
    let seq = a.store().session(s).unwrap().session.change_seq;
    a.app.compositions.set(p.id, values()).unwrap();
    assert_eq!(a.composition(p.id).unwrap().revision, 1);
    let after = a.store().session(s).unwrap();
    assert_eq!(after.session.change_seq, seq);
    assert!(!after.dirty);
}
#[test]
fn c3f_frameless_save_is_refused() {
    let a = TestApp::with_fake_document().unwrap();
    let s = session(&a, "S1");
    let p = add(&a, s, "a.ARW");
    assert!(matches!(
        a.app.compositions.set(p.id, values()),
        Err(Error::FrameUnknown)
    ));
    let c = a.composition(p.id).unwrap();
    assert!(c.rect.is_none() && c.fingerprint.is_none());
}
#[test]
fn c4a_reset_uses_current_default_and_is_idempotent() {
    let (mut a, _, p) = ready();
    a.app.compositions.set(p.id, values()).unwrap();
    a.settings_set(&serde_json::json!({"default_shape":"4:5"}))
        .unwrap();
    a.events.0.lock().unwrap().clear();
    let reset = a.app.compositions.reset(p.id).unwrap();
    assert_eq!(
        (
            reset.kind,
            reset.shape,
            reset.orientation,
            reset.quarter_turns,
            reset.flip,
            reset.straighten,
            reset.origin,
            reset.revision,
            reset.state
        ),
        (
            CompositionKind::Normal,
            Shape::FourFive,
            Orientation::Named,
            0,
            Flip::None,
            0.0,
            Origin::Default,
            2,
            CompositionState::Ok
        )
    );
    assert_eq!(
        reset.rect,
        Some(Rect {
            x: 0.23328125,
            y: 0.0,
            w: 0.5334375,
            h: 1.0
        })
    );
    assert_eq!(photo_events(&a, p.id), 1);
    assert_eq!(a.app.compositions.reset(p.id).unwrap(), reset);
    assert_eq!(photo_events(&a, p.id), 1);
}
#[test]
fn c5a_geometry_change_flags_review_and_replaces_fingerprint() {
    let (a, _, p) = ready();
    let old = a.composition(p.id).unwrap();
    let variant = format!("v{}", p.id);
    a.fake.set_variant_geometry(
        &variant,
        EditorGeometry {
            rotation: 3.0,
            ..Default::default()
        },
    );
    rerender(&a, &p, 2560, 1707);
    let c = a.composition(p.id).unwrap();
    assert_eq!(
        (c.state, c.review_reason, c.revision, c.rect),
        (
            CompositionState::NeedsReview,
            Some(ReviewReason::Geometry),
            0,
            old.rect
        )
    );
    assert_eq!(c.fingerprint.unwrap().geometry.rotation, 3.0);
    assert!(
        a.store()
            .photo(p.id)
            .unwrap()
            .adjusted_preview_path
            .unwrap()
            .ends_with("adjusted-2.jpg")
    );
    assert_eq!(photo_events(&a, p.id), 2);
}
#[test]
fn c5b_compatible_preview_sizes_keep_composition() {
    for (w, h) in [(1920, 1280), (2560, 1700)] {
        let (a, _, p) = ready();
        let old = a.composition(p.id).unwrap();
        rerender(&a, &p, w, h);
        let c = a.composition(p.id).unwrap();
        assert_eq!((c.state, c.rect), (CompositionState::Ok, old.rect));
    }
}
#[test]
fn c5c_save_confirms_needs_review() {
    let (a, _, p) = ready();
    a.fake.set_variant_geometry(
        &format!("v{}", p.id),
        EditorGeometry {
            rotation: 3.0,
            ..Default::default()
        },
    );
    rerender(&a, &p, 2560, 1707);
    let c = a
        .app
        .compositions
        .set(
            p.id,
            CompositionValues {
                kind: CompositionKind::Normal,
                shape: Shape::Square,
                orientation: Orientation::Named,
                quarter_turns: 0,
                flip: Flip::None,
                straighten: 0.0,
                rect: square(),
            },
        )
        .unwrap();
    assert_eq!(
        (c.state, c.review_reason, c.revision),
        (CompositionState::Ok, None, 1)
    );
    assert_eq!(c.values().unwrap().rect, square());
}
#[test]
fn c5d_shared_raw_marks_both_rows_for_review() {
    let (a, _, p) = ready();
    let s2 = session(&a, "S2");
    a.review().copy_photos(&[p.id], s2).unwrap();
    let copy = a.store().photos_in_session(s2).unwrap().pop().unwrap();
    a.fake.set_variant_geometry(
        &format!("v{}", p.id),
        EditorGeometry {
            rotation: 3.0,
            ..Default::default()
        },
    );
    rerender(&a, &p, 2560, 1707);
    for id in [p.id, copy.id] {
        let c = a.composition(id).unwrap();
        assert_eq!(
            (c.state, c.review_reason),
            (CompositionState::NeedsReview, Some(ReviewReason::Geometry))
        );
    }
}
#[test]
fn c5e_orientation_change_flags_frame_review() {
    let (a, _, p) = ready();
    let old = a.composition(p.id).unwrap();
    rerender(&a, &p, 1707, 2560);
    let c = a.composition(p.id).unwrap();
    assert_eq!(
        (c.state, c.review_reason, c.rect),
        (
            CompositionState::NeedsReview,
            Some(ReviewReason::Frame),
            old.rect
        )
    );
    assert_eq!(c.fingerprint.unwrap().frame, Frame { w: 1707, h: 2560 });
}
#[test]
fn c6a_restart_backfills_missing_composition_rows_without_dirty() {
    let mut a = TestApp::with_fake_document().unwrap();
    let s = session(&a, "S1");
    let ids = ["a.ARW", "b.ARW", "c.ARW"].map(|name| add(&a, s, name).id);
    let seq = a.store().session(s).unwrap().session.change_seq;
    db(&a).execute("DELETE FROM compositions", []).unwrap();
    a.kill_and_restart().unwrap();
    for id in ids {
        let c = a.composition(id).unwrap();
        assert_eq!(
            (c.shape, c.revision, c.origin, c.rect, c.fingerprint),
            (Shape::Square, 0, Origin::Default, None, None)
        );
    }
    let after = a.store().session(s).unwrap();
    assert_eq!(after.session.change_seq, seq);
    assert!(!after.dirty);
}
#[test]
fn c6b_real_start_rerenders_old_previews_once_without_retouch() {
    let mut a = TestApp::with_fake_document().unwrap();
    let s = session(&a, "S1");
    let old = add(&a, s, "a.ARW");
    let fresh = add(&a, s, "b.ARW");
    preview(&a, &old, 2560, 1707);
    db(&a)
        .execute(
            "DELETE FROM settings WHERE key='rerender_after_composition'",
            [],
        )
        .unwrap();
    let before = a.store().preview_state(&old.raw_path).unwrap().unwrap();
    let seq = a.store().session(s).unwrap().session.change_seq;
    a.restart_with_startup().unwrap();
    let after = a.store().preview_state(&old.raw_path).unwrap().unwrap();
    let untouched = a.store().preview_state(&fresh.raw_path).unwrap().unwrap();
    assert_eq!(
        (after.adjusted_gen, after.retouch_gen),
        (before.adjusted_gen + 1, before.retouch_gen)
    );
    assert_eq!(untouched.adjusted_gen, 0);
    assert_eq!(a.store().session(s).unwrap().session.change_seq, seq);
    assert_eq!(
        db(&a)
            .query_row(
                "SELECT value FROM settings WHERE key='rerender_after_composition'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "done"
    );
    a.tick(20_000).unwrap();
    assert_eq!(
        a.fake
            .state
            .lock()
            .unwrap()
            .ops
            .iter()
            .filter(|op| *op == &format!("process:v{}:PSM Preview", old.id))
            .count(),
        2
    );
    a.complete_render_with(&issued(&a), &SyntheticJpeg::rgb(2560, 1707, [200, 100, 50]))
        .unwrap();
    a.restart_with_startup().unwrap();
    assert_eq!(
        a.store()
            .preview_state(&old.raw_path)
            .unwrap()
            .unwrap()
            .adjusted_gen,
        after.adjusted_gen
    );
}

#[test]
fn c7a_session_view_aligns_compositions_and_thumbnails() {
    let (a, s, p) = ready();
    let second = add(&a, s, "b.ARW");
    let view = a.sessions().view(s).unwrap();
    assert_eq!(view.photos.len(), view.compositions.len());
    for (photo, composition) in view.photos.iter().zip(&view.compositions) {
        assert_eq!(photo.id, composition.photo_id);
        assert_eq!(*composition, a.composition(photo.id).unwrap());
        assert_eq!(
            photo.thumbnail_path,
            a.store().photo(photo.id).unwrap().thumbnail_path
        );
    }
    assert!(view.photos.iter().any(|photo| photo.id == p.id));
    assert!(view.photos.iter().any(|photo| photo.id == second.id));
}
