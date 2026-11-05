use psm_core::domain::composition::{
    CompositionKind, CompositionValues, Flip, Orientation, Rect, Shape,
};
use psm_core::domain::photo::{Classification, NewPhoto, PhotoSource};
use psm_core::mechanisms::hash::sha256_file;
use psm_testkit::fixtures::synthetic_master::SyntheticMaster;
use psm_testkit::{TestApp, fixtures::synthetic_jpeg::SyntheticJpeg};
use std::io::Write;
use std::sync::{Arc, Mutex};

#[test]
fn c3c_batch_applies_valid_photos_and_keeps_a_refusal() {
    let a = TestApp::with_fake_document().unwrap();
    let session = a
        .items()
        .create_local("Batch", "BATCH", "object", None)
        .unwrap()
        .1
        .session
        .id;
    let mut photos = Vec::new();
    for name in ["a.ARW", "b.ARW", "c.ARW"] {
        let photo = a.add_photo(session, name).unwrap();
        a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
        a.fake
            .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
        a.app.previews.request_initial(&photo.raw_path).unwrap();
        let issued = a
            .fake
            .state
            .lock()
            .unwrap()
            .issued
            .last()
            .unwrap()
            .0
            .clone();
        a.complete_render_with(&issued, &SyntheticJpeg::rgb(2560, 1707, [80, 80, 80]))
            .unwrap();
        photos.push(photo);
    }
    let valid = CompositionValues {
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
    };
    let invalid = CompositionValues {
        rect: Rect {
            x: 0.2,
            y: 0.0,
            w: 0.5,
            h: 1.0,
        },
        ..valid.clone()
    };
    let seq = a.store().session(session).unwrap().session.change_seq;
    let result = a.app.compositions.apply(vec![
        (photos[0].id, valid.clone()),
        (photos[1].id, invalid),
        (photos[2].id, valid),
    ]);
    assert_eq!(result.applied, vec![photos[0].id, photos[2].id]);
    assert_eq!(result.refused.len(), 1);
    assert_eq!(result.refused[0].photo_id, photos[1].id);
    assert_eq!(result.refused[0].reason, "off_shape");
    assert_eq!(
        photos
            .iter()
            .map(|p| a.composition(p.id).unwrap().revision)
            .collect::<Vec<_>>(),
        vec![1, 0, 1]
    );
    assert_eq!(
        a.store().session(session).unwrap().session.change_seq,
        seq + 2
    );
}

#[derive(Clone)]
struct LogBytes(Arc<Mutex<Vec<u8>>>);
impl Write for LogBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn c7b_master_and_output_log_photo_timings() {
    let a = TestApp::with_fake_document().unwrap();
    let session = a
        .items()
        .create_local("Timed", "TIMED", "object", None)
        .unwrap()
        .1
        .session
        .id;
    let path = a.dir.path().join("timed.ARW");
    std::fs::write(&path, b"timed raw").unwrap();
    let photo = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id: session,
            raw_path: path.to_string_lossy().into_owned(),
            raw_hash: Some(sha256_file(&path).unwrap()),
            captured_at: Some(0),
            source: PhotoSource::Capture,
            created_at: 0,
        })
        .unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.app.previews.request_initial(&photo.raw_path).unwrap();
    let preview_job = a
        .fake
        .state
        .lock()
        .unwrap()
        .issued
        .last()
        .unwrap()
        .0
        .clone();
    a.complete_render_with(&preview_job, &SyntheticJpeg::rgb(2560, 1707, [80, 80, 80]))
        .unwrap();
    a.review().classify(photo.id, Classification::Main).unwrap();
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let output = LogBytes(buffer.clone());
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || output.clone())
        .without_time()
        .with_ansi(false)
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        a.app.export.run(session).unwrap();
        let master_job = a
            .fake
            .state
            .lock()
            .unwrap()
            .issued
            .last()
            .unwrap()
            .0
            .clone();
        a.complete_master_render(
            &master_job,
            &SyntheticMaster::solid(2560, 1707, [80, 80, 80]),
        )
        .unwrap();
        a.tick(0).unwrap();
    });
    let log = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
    let master = log
        .lines()
        .find(|line| line.contains("master rendered"))
        .unwrap();
    assert!(master.contains("raw=") && master.contains("adjusted_gen=") && master.contains("ms="));
    assert!(
        master
            .split("ms=")
            .nth(1)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<u64>()
            .is_ok()
    );
    let output = log
        .lines()
        .find(|line| line.contains("render_output done"))
        .unwrap();
    assert!(
        output.contains(&format!("photo_id={}", photo.id))
            && output.contains("slot=")
            && output.contains("ms=")
    );
    assert!(
        output
            .split("ms=")
            .nth(1)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<u64>()
            .is_ok()
    );
}
