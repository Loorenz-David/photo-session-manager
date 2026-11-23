//! H3·U1 — JPEG photos in the photo pipeline (master plan archive MC-H4.6; book 10H §7.3–7.4).
//! A JPEG enters as a photo through the existing import into an active session; the rows observe
//! the stored photo and composition after `tick`, the files written and what the fake Capture One
//! received.
use psm_core::{
    domain::{
        composition::{
            CompositionKind, CompositionValues, Flip, Frame, FrameSource, Orientation, Rect, Shape,
        },
        photo::{Classification, Photo},
    },
    mechanisms::canon::canon,
    services::photos::PhotosService,
};
use psm_testkit::{
    TestApp,
    fixtures::{synthetic_jpeg::SyntheticJpeg, synthetic_raw::SyntheticRaw},
};
use std::{fs, path::PathBuf};

const SKU: &str = "H3J";

/// An app with a fake document and an active session for item `H3J`.
fn app() -> (TestApp, i64) {
    let a = TestApp::with_fake_document().unwrap();
    let s = a
        .items()
        .create_local(SKU, "Oak", "Chair", None)
        .unwrap()
        .1
        .session
        .id;
    a.store().set_active_session_id(Some(s)).unwrap();
    (a, s)
}

/// Imports `bytes` named `name` (written outside the capture folder) through `PhotosService::import`
/// and answers the new photo.
fn import(a: &TestApp, name: &str, bytes: &[u8]) -> Photo {
    let outside = a.dir.path().join("outside");
    fs::create_dir_all(&outside).unwrap();
    let source = outside.join(name);
    fs::write(&source, bytes).unwrap();
    PhotosService {
        arrival: a.app.arrival.as_ref(),
        editing: a.app.editing.as_ref(),
        store: a.store(),
        clock: a.clock.as_ref(),
        history: a.app.history.as_ref(),
    }
    .import(&[source.to_string_lossy().into_owned()])
    .unwrap();
    let target = canon(&a.capture_root().join(SKU).join(name)).unwrap();
    a.store()
        .photos_for_raw(&target)
        .unwrap()
        .into_iter()
        .next()
        .expect("imported photo")
}

fn photo(a: &TestApp, id: i64) -> Photo {
    a.store().photo(id).unwrap()
}

fn job_status(a: &TestApp, kind: &str, target: &str) -> String {
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            "SELECT status FROM jobs WHERE kind=?1 AND target=?2",
            [kind, target],
            |r| r.get(0),
        )
        .unwrap()
}

fn image_size(path: &str) -> (u32, u32) {
    image::image_dimensions(path).unwrap()
}

fn issued_for(a: &TestApp, variant: &str) -> String {
    a.fake
        .jobs_for(variant)
        .last()
        .cloned()
        .expect("issued job")
}

#[test]
fn h3_c1a_large_jpeg_preview_scaled_frame_is_pixel_size() {
    let (a, _) = app();
    let p = import(&a, "a.jpg", &SyntheticJpeg::rgb(3000, 2000, [120, 90, 60]));
    a.tick(0).unwrap();
    let p = photo(&a, p.id);
    let preview = p.original_preview_path.clone().expect("original preview");
    assert!(PathBuf::from(&preview).is_file());
    assert_eq!(image_size(&preview), (1600, 1067));
    assert_eq!(p.orientation, None);
    let c = a.composition(p.id).unwrap();
    assert_eq!(c.frame(), Some(Frame { w: 3000, h: 2000 }));
    assert_eq!(c.fingerprint.unwrap().source, FrameSource::Camera);
}

#[test]
fn h3_c1b_exif_orientation_is_stored_and_turns_the_frame() {
    let (a, _) = app();
    let bytes = SyntheticJpeg::with_orientation(3000, 2000, [120, 90, 60], 6);
    // The fixture carries the tag where a JPEG reader finds it.
    let read = exif::Reader::new()
        .read_from_container(&mut std::io::Cursor::new(&bytes))
        .unwrap();
    assert_eq!(
        read.get_field(exif::Tag::Orientation, exif::In::PRIMARY)
            .unwrap()
            .value
            .get_uint(0),
        Some(6)
    );
    let p = import(&a, "b.jpg", &bytes);
    a.tick(0).unwrap();
    let p = photo(&a, p.id);
    assert_eq!(p.orientation, Some(6));
    assert_eq!(
        a.composition(p.id).unwrap().frame(),
        Some(Frame { w: 2000, h: 3000 })
    );
}

#[test]
fn h3_c1c_small_jpeg_is_its_own_preview_never_enlarged() {
    let (a, _) = app();
    let bytes = SyntheticJpeg::rgb(800, 600, [10, 200, 30]);
    let p = import(&a, "c.jpg", &bytes);
    a.tick(0).unwrap();
    let p = photo(&a, p.id);
    let preview = p.original_preview_path.clone().expect("original preview");
    assert_eq!(fs::read(&preview).unwrap(), bytes);
    assert_eq!(
        a.composition(p.id).unwrap().frame(),
        Some(Frame { w: 800, h: 600 })
    );
}

#[test]
fn h3_c1d_undecodable_jpeg_finishes_without_preview_or_frame() {
    let (a, _) = app();
    let p = import(&a, "x.jpg", b"not a jpeg");
    a.tick(0).unwrap();
    a.tick(60_000).unwrap();
    let stored = photo(&a, p.id);
    assert_eq!(stored.original_preview_path, None);
    assert_eq!(a.composition(p.id).unwrap().frame(), None);
    assert_eq!(job_status(&a, "original_preview", &p.raw_path), "done");
}

#[test]
fn h3_c1e_raw_keeps_the_embedded_jpeg_path() {
    let (a, _) = app();
    let (bytes, streams) =
        SyntheticRaw::build_with_frame(&[(6240, 4160, true)], 1, None, Some((6240, 4160)));
    let p = import(&a, "a.ARW", &bytes);
    a.tick(0).unwrap();
    let p = photo(&a, p.id);
    let preview = p.original_preview_path.clone().expect("original preview");
    assert_eq!(fs::read(&preview).unwrap(), streams[0]);
    let c = a.composition(p.id).unwrap();
    assert_eq!(c.frame(), Some(Frame { w: 6240, h: 4160 }));
    assert_eq!(c.fingerprint.unwrap().source, FrameSource::Camera);
}

#[test]
fn h3_c2a_composition_set_on_a_jpeg_photo() {
    let (a, _) = app();
    let p = import(&a, "a.jpg", &SyntheticJpeg::rgb(3000, 2000, [120, 90, 60]));
    a.tick(0).unwrap();
    let saved = a
        .app
        .compositions
        .set(
            p.id,
            CompositionValues {
                kind: CompositionKind::Normal,
                shape: Shape::Free,
                orientation: Orientation::Named,
                quarter_turns: 1,
                flip: Flip::None,
                straighten: 0.0,
                rect: Rect {
                    x: 0.1,
                    y: 0.1,
                    w: 0.5,
                    h: 0.5,
                },
            },
        )
        .expect("saved, no frame_unknown");
    assert_eq!(saved.revision, 1);
}

#[test]
fn h3_c2b_classify_and_reorder_jpeg_photos() {
    let (a, s) = app();
    let first = import(&a, "a.jpg", &SyntheticJpeg::rgb(1200, 800, [1, 2, 3]));
    let second = import(&a, "b.jpg", &SyntheticJpeg::rgb(1200, 800, [4, 5, 6]));
    a.tick(0).unwrap();
    a.review()
        .classify(second.id, Classification::Condition)
        .unwrap();
    a.review()
        .classify(second.id, Classification::Main)
        .unwrap();
    a.review().reorder(second.id, 1).unwrap();
    let view = a.sessions().view(s).unwrap();
    let main: Vec<(i64, Option<i64>)> = view
        .photos
        .iter()
        .filter(|p| p.classification == Classification::Main)
        .map(|p| (p.id, p.position))
        .collect();
    assert_eq!(main, vec![(second.id, Some(1)), (first.id, Some(2))]);
}

#[test]
fn h3_c2c_capture_one_preview_replaces_the_camera_frame() {
    let (a, _) = app();
    let p = import(&a, "a.jpg", &SyntheticJpeg::rgb(3000, 2000, [120, 90, 60]));
    a.tick(0).unwrap();
    a.fake.add_image(&p.raw_path, &["v1"], false);
    for _ in 0..5 {
        a.tick(1_000).unwrap();
    }
    let ops = a.fake.state.lock().unwrap().ops.clone();
    assert!(
        ops.iter().any(|op| op.starts_with("process:v1:")),
        "{ops:?}"
    );
    a.complete_render_with(
        &issued_for(&a, "v1"),
        &SyntheticJpeg::rgb(1500, 1000, [120, 90, 60]),
    )
    .unwrap();
    assert!(photo(&a, p.id).adjusted_preview_path.is_some());
    let c = a.composition(p.id).unwrap();
    assert_eq!(c.frame(), Some(Frame { w: 1500, h: 1000 }));
    assert_eq!(c.fingerprint.unwrap().source, FrameSource::Preview);
}

fn render_output_open(a: &TestApp) -> i64 {
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE kind='render_output' AND status IN ('queued','running')",
            [],
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn h3_c2d_export_holds_one_output_for_a_jpeg_photo() {
    let (a, s) = app();
    let p = import(&a, "a.jpg", &SyntheticJpeg::rgb(3000, 2000, [120, 90, 60]));
    a.fake.add_image(&p.raw_path, &["v1"], false);
    a.tick(0).unwrap();
    // An import is approved on arrival (P66): the photo is main.
    assert_eq!(photo(&a, p.id).classification, Classification::Main);
    a.app.export.run(s).unwrap();
    let mut finished = false;
    for _ in 0..10 {
        a.complete_all_renders().unwrap();
        a.tick(0).unwrap();
        if render_output_open(&a) == 0 && a.complete_all_renders().unwrap().is_empty() {
            finished = true;
            break;
        }
    }
    assert!(finished, "export did not finish");
    let dir = a.export_root().join(SKU);
    let outputs: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".jpg"))
        .collect();
    assert_eq!(outputs, vec![format!("{SKU}_01_main.jpg")]);
}

#[test]
fn h3_c2e_a_jpeg_written_into_the_capture_folder_is_not_an_arrival() {
    let (a, s) = app();
    let late = a.capture_root().join(SKU).join("late.jpg");
    fs::create_dir_all(late.parent().unwrap()).unwrap();
    fs::write(&late, SyntheticJpeg::rgb(800, 600, [1, 2, 3])).unwrap();
    a.watcher_event(&late).unwrap();
    a.tick(1_000).unwrap();
    a.tick(1_000).unwrap();
    assert!(a.store().photos_in_session(s).unwrap().is_empty());
    let all: i64 = rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row("SELECT COUNT(*) FROM photos", [], |r| r.get(0))
        .unwrap();
    assert_eq!(all, 0);
}
