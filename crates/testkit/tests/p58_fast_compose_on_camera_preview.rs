//! P58 — compose on the camera's own preview; render what matters first (book changelog 39).
use psm_core::{
    domain::{
        composition::{CompositionState, CompositionValues, Frame, FrameSource, Rect, Shape},
        editing::RenderPurpose,
        photo::Classification,
    },
    mechanisms::composition::validate,
    ports::clock::Clock,
};
use psm_testkit::{
    TestApp,
    fixtures::{
        synthetic_jpeg::SyntheticJpeg, synthetic_master::SyntheticMaster,
        synthetic_raw::SyntheticRaw,
    },
};
use std::{fs, time::Duration};

/// One capture whose RAW embeds a `jpeg`-sized preview and declares a `sensor` frame, delivered
/// through the capture hook; the original-preview job and the first preview request have run.
fn camera_photo(jpeg: (u16, u16), sensor: (u32, u32), orientation: u8) -> (TestApp, i64) {
    let a = TestApp::with_fake_document().unwrap();
    let raw = a.dir.path().join("Capture One Session/Capture/camera.ARW");
    fs::create_dir_all(raw.parent().unwrap()).unwrap();
    let (bytes, _) =
        SyntheticRaw::build_with_frame(&[(jpeg.0, jpeg.1, true)], orientation, None, Some(sensor));
    fs::write(&raw, bytes).unwrap();
    a.deliver_hook(&serde_json::json!({"hook":"capture_done","argv":[raw]}).to_string())
        .unwrap();
    let photo = a.store().photos_in_session(1).unwrap().remove(0);
    a.fake.add_image(&photo.raw_path, &["v1"], false);
    a.tick(0).unwrap();
    (a, photo.id)
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

/// Saves `rect` with `shape` as the photographer's composition.
fn compose(
    a: &TestApp,
    id: i64,
    shape: Shape,
    rect: Rect,
) -> psm_core::domain::composition::Composition {
    let values = CompositionValues {
        shape,
        rect,
        ..a.composition(id).unwrap().values().unwrap()
    };
    a.app.compositions.set(id, values).unwrap()
}

/// The row's rect (0.1, 0.1, ·, 0.6) in 4:5 on a 3:2 frame: w = 0.8 · 0.6 · 4160 / 6240 = 0.32.
const FOUR_FIVE: Rect = Rect {
    x: 0.1,
    y: 0.1,
    w: 0.32,
    h: 0.6,
};

#[test]
fn c1a_full_camera_preview_materializes_composable_frame() {
    for (orientation, frame) in [
        (1, Frame { w: 6240, h: 4160 }),
        (6, Frame { w: 4160, h: 6240 }),
    ] {
        let (a, id) = camera_photo((6240, 4160), (6240, 4160), orientation);
        let p = a.store().photo(id).unwrap();
        let c = a.composition(id).unwrap();
        assert!(p.original_preview_path.is_some());
        assert_eq!(p.orientation, Some(orientation as i64));
        assert_eq!(p.adjusted_preview_path, None);
        assert_eq!(c.frame(), Some(frame), "orientation {orientation}");
        assert_eq!(c.fingerprint.unwrap().source, FrameSource::Camera);
        assert!(c.rect.is_some());
    }
}

#[test]
fn c1b_cropped_or_small_camera_preview_keeps_compose_waiting() {
    for jpeg in [(1088, 1080), (1500, 1000)] {
        let (a, id) = camera_photo(jpeg, (6000, 4000), 1);
        let p = a.store().photo(id).unwrap();
        assert!(p.original_preview_path.is_some());
        assert_eq!(p.adjusted_preview_path, None);
        let c = a.composition(id).unwrap();
        assert_eq!((c.frame(), c.rect), (None, None), "{jpeg:?}");
    }
}

#[test]
fn c1c_adjusted_preview_replaces_provisional_frame_without_changing_saved_rect() {
    let (a, id) = camera_photo((6240, 4160), (6240, 4160), 1);
    let original = a.store().photo(id).unwrap().original_preview_path;
    let before = compose(&a, id, Shape::FourFive, FOUR_FIVE);
    a.complete_render_with(&issued(&a), &SyntheticJpeg::rgb(300, 200, [90, 120, 150]))
        .unwrap();
    let after = a.composition(id).unwrap();
    let photo = a.store().photo(id).unwrap();
    assert!(photo.adjusted_preview_path.is_some());
    assert_eq!(photo.original_preview_path, original);
    assert_eq!(
        (after.shape, after.rect, after.revision, after.state),
        (
            Shape::FourFive,
            Some(FOUR_FIVE),
            before.revision,
            CompositionState::Ok
        )
    );
    assert_eq!(after.frame(), Some(Frame { w: 300, h: 200 }));
    assert_eq!(after.fingerprint.unwrap().source, FrameSource::Preview);

    // A composition that would no longer fit Capture One's frame is pulled back inside it.
    let (a, id) = camera_photo((6240, 4160), (6240, 4160), 1);
    let whole = Rect {
        x: 0.0,
        y: 0.0,
        w: 1.0,
        h: 1.0,
    };
    compose(&a, id, Shape::ThreeTwo, whole);
    a.complete_render_with(&issued(&a), &SyntheticJpeg::rgb(300, 203, [90, 120, 150]))
        .unwrap();
    let after = a.composition(id).unwrap();
    let rect = after.rect.unwrap();
    assert_ne!(rect, whole);
    assert!(validate(&after.values().unwrap(), Frame { w: 300, h: 203 }).is_ok());
    assert_eq!(after.state, CompositionState::Ok);
}

#[test]
fn c2a_export_requested_on_original_waits_for_capture_one_master() {
    let (a, id) = camera_photo((6240, 4160), (6240, 4160), 1);
    let sid = a
        .items()
        .create_local("P58", "Oak", "Chair", None)
        .unwrap()
        .1
        .session
        .id;
    a.review().move_photos(&[id], sid).unwrap();
    a.review().classify(id, Classification::Main).unwrap();
    compose(&a, id, Shape::FourFive, FOUR_FIVE);
    assert_eq!(
        a.composition(id).unwrap().fingerprint.unwrap().source,
        FrameSource::Camera
    );
    let _run = a.app.export.run(sid).unwrap();
    a.tick(0).unwrap();
    let output = a.export_root().join("P58/P58_01_main.jpg");
    assert!(!output.exists());
    // The preview already sent to Capture One finishes first; then the master is requested.
    a.complete_render_with(&issued(&a), &SyntheticJpeg::rgb(300, 200, [90, 120, 150]))
        .unwrap();
    let master_job = issued(&a);
    assert!(a.is_master_job(&master_job));
    assert!(!output.exists());
    a.complete_master_render(
        &master_job,
        &SyntheticMaster::solid(600, 400, [0xC800, 0x6400, 0x3200]),
    )
    .unwrap();
    a.tick(0).unwrap();
    let image = a.decode(&output).unwrap();
    // The 4:5 rect cut from Capture One's 600 × 400 master, in the master's colour.
    assert_eq!(image.dimensions(), (192, 240));
    let px = image.get_pixel(96, 120).0;
    assert!(
        px.iter()
            .zip([200u8, 100, 50])
            .all(|(got, want)| got.abs_diff(want) <= 3),
        "{px:?}"
    );
}

fn queued(a: &TestApp, name: &str, fresh: bool) -> (i64, String) {
    let photo = a.add_photo(1, name).unwrap();
    a.fake.add_image(&photo.raw_path, &[name], false);
    a.app
        .editing
        .request_render(
            &photo.raw_path,
            RenderPurpose::Preview,
            1,
            1,
            a.clock.mono(),
            a.clock.now_ms(),
            fresh,
        )
        .unwrap();
    (photo.id, name.to_owned())
}

/// Capture One finishes the render it is working on; returns the variant it is sent next.
fn finish(a: &TestApp) -> String {
    let job = issued(a);
    a.app
        .editing
        .finish_render(
            &job,
            a.clock.mono() + Duration::from_secs(1),
            a.clock.now_ms(),
        )
        .unwrap();
    a.fake
        .state
        .lock()
        .unwrap()
        .issued
        .last()
        .unwrap()
        .1
        .clone()
}

#[test]
fn c3a_focused_photo_is_next_after_current_render() {
    let a = TestApp::with_fake_document().unwrap();
    let photos: Vec<_> = (1..=20)
        .map(|n| queued(&a, &format!("photo-{n}.ARW"), false))
        .collect();
    a.app.previews.set_focus(Some(photos[14].0)).unwrap();
    assert_eq!(finish(&a), photos[14].1);
    a.app.previews.set_focus(None).unwrap();
    assert_eq!(finish(&a), photos[1].1);
}

#[test]
fn c3b_newest_arrivals_precede_oldest_backlog() {
    let a = TestApp::with_fake_document().unwrap();
    let old: Vec<_> = (1..=10)
        .map(|n| queued(&a, &format!("old-{n}.ARW"), false))
        .collect();
    let new1 = queued(&a, "new-1.ARW", true);
    let new2 = queued(&a, "new-2.ARW", true);
    assert_eq!(finish(&a), new2.1);
    assert_eq!(finish(&a), new1.1);
    // old-1 was already with Capture One; the backlog continues oldest-first.
    assert_eq!(finish(&a), old[1].1);
    assert_eq!(finish(&a), old[2].1);
}
