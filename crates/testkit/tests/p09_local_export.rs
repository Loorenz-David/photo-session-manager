//! P09 local export: ordered web renders under MC-8 names, item.json, the swap (M7, M15, M10, M23, MC-16).

use psm_core::{
    Error,
    domain::photo::{Classification as C, NewPhoto, Photo, PhotoSource},
    events::{AppEvent, ExportState},
    mechanisms::hash::sha256_file,
};
use psm_testkit::{TestApp, fake_capture_one::FakeCaptureOneScripting};
use std::{fs, io::Write, path::Path};

const NOW_MS: i64 = 1_761_350_400_000; // 2025-10-25T00:00:00Z

fn app() -> TestApp {
    let a = TestApp::with_fake_document().unwrap();
    a.clock.advance(NOW_MS);
    a
}

fn session(a: &TestApp, sku: &str) -> i64 {
    a.items()
        .create_local(sku, "Oak", "Sideboard", None)
        .unwrap()
        .1
        .session
        .id
}

fn variant(photo: &Photo) -> String {
    format!("v{}", photo.id)
}

/// A RAW on disk with its real SHA-256 (MC-3), known to the fake Capture One document, classified.
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
            captured_at: Some(NOW_MS),
            source: PhotoSource::Capture,
            created_at: NOW_MS,
        })
        .unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&variant(&photo)], false);
    match class {
        C::Unreviewed => {}
        C::Deleted => {
            a.review().delete(photo.id).unwrap();
        }
        _ => {
            a.review().classify(photo.id, class).unwrap();
        }
    }
    a.store().photo(photo.id).unwrap()
}

fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn sorted(list: &[&str]) -> Vec<String> {
    let mut list: Vec<String> = list.iter().map(|s| s.to_string()).collect();
    list.sort();
    list
}

fn process_count(a: &TestApp) -> usize {
    a.fake.state.lock().unwrap().issued.len()
}

fn export_states(a: &TestApp) -> Vec<(usize, usize, ExportState)> {
    a.events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AppEvent::ExportProgress {
                done, total, state, ..
            } => Some((*done, *total, *state)),
            _ => None,
        })
        .collect()
}

/// Session K: main [a, b, c], condition [d], unreviewed [e], deleted [f].
fn c1_session(a: &TestApp) -> (i64, Vec<Photo>) {
    let s = session(a, "K");
    let photos = vec![
        photo(a, s, "a", C::Main),
        photo(a, s, "b", C::Main),
        photo(a, s, "c", C::Main),
        photo(a, s, "d", C::Condition),
        photo(a, s, "e", C::Unreviewed),
        photo(a, s, "f", C::Deleted),
    ];
    (s, photos)
}

fn export(a: &TestApp, session_id: i64) {
    a.app.export.run(session_id).unwrap();
    a.complete_all_renders().unwrap();
}

#[test]
fn c1a_export_folder_holds_exactly_the_named_files() {
    let a = app();
    let (s, photos) = c1_session(&a);
    export(&a, s);
    assert_eq!(
        names(&a.export_root().join("K")),
        sorted(&[
            "K_01_main.jpg",
            "K_02_main.jpg",
            "K_03_main.jpg",
            "K_01_condition.jpg",
            "item.json"
        ])
    );
    assert!(a.fake.jobs_for(&variant(&photos[4])).is_empty());
    assert!(a.fake.jobs_for(&variant(&photos[5])).is_empty());
    assert_eq!(export_states(&a).last(), Some(&(4, 4, ExportState::Done)));
}

#[test]
fn c2a_ten_mains_named_in_order_with_their_own_render() {
    let a = app();
    let s = session(&a, "K");
    let mains: Vec<Photo> = (1..=10)
        .map(|n| photo(&a, s, &format!("m{n}"), C::Main))
        .collect();
    export(&a, s);
    let dir = a.export_root().join("K");
    for (index, photo) in mains.iter().enumerate() {
        let jobs = a.fake.jobs_for(&variant(photo));
        assert_eq!(jobs.len(), 1);
        let file = dir.join(format!("K_{:02}_main.jpg", index + 1));
        assert_eq!(
            fs::read(&file).unwrap(),
            FakeCaptureOneScripting::output_bytes(&jobs[0]),
            "{}",
            file.display()
        );
    }
    let mut expected: Vec<String> = (1..=10).map(|n| format!("K_{n:02}_main.jpg")).collect();
    expected.push("item.json".into());
    expected.sort();
    assert_eq!(names(&dir), expected);
}

#[test]
fn c2b_hundred_mains_pad_to_three_digits() {
    let a = app();
    let s = session(&a, "K");
    for n in 1..=100 {
        photo(&a, s, &format!("m{n}"), C::Main);
    }
    export(&a, s);
    let mut expected: Vec<String> = (1..=100).map(|n| format!("K_{n:03}_main.jpg")).collect();
    expected.push("item.json".into());
    expected.sort();
    assert_eq!(names(&a.export_root().join("K")), expected);
}

#[test]
fn c3a_item_json_bytes() {
    let a = app();
    let (s, _) = c1_session(&a);
    export(&a, s);
    assert_eq!(
        fs::read_to_string(a.export_root().join("K/item.json")).unwrap(),
        "{\n  \"attributes\": {},\n  \"condition\": [\n    \"K_01_condition.jpg\"\n  ],\n  \"item_type\": \"Sideboard\",\n  \"main\": [\n    \"K_01_main.jpg\",\n    \"K_02_main.jpg\",\n    \"K_03_main.jpg\"\n  ],\n  \"schema\": 1,\n  \"sku\": \"K\",\n  \"title\": \"Oak\"\n}\n"
    );
}

#[test]
fn c4a_second_export_leaves_no_stale_file() {
    let a = app();
    let s = session(&a, "K");
    let mains: Vec<Photo> = ["a", "b", "c"]
        .iter()
        .map(|n| photo(&a, s, n, C::Main))
        .collect();
    export(&a, s);
    assert!(a.export_root().join("K/K_03_main.jpg").exists());
    a.review().delete(mains[1].id).unwrap();
    export(&a, s);
    assert_eq!(
        names(&a.export_root().join("K")),
        sorted(&["K_01_main.jpg", "K_02_main.jpg", "item.json"])
    );
    assert_eq!(names(&a.export_root()), vec!["K".to_string()]);
}

#[test]
fn c5a_changed_raw_fails_before_any_render() {
    let a = app();
    let (s, photos) = c1_session(&a);
    export(&a, s);
    let dir = a.export_root().join("K");
    let before: Vec<(String, Vec<u8>)> = names(&dir)
        .into_iter()
        .map(|n| (n.clone(), fs::read(dir.join(&n)).unwrap()))
        .collect();
    let issued = process_count(&a);
    let b = &photos[1];
    fs::OpenOptions::new()
        .append(true)
        .open(&b.raw_path)
        .unwrap()
        .write_all(b"appended")
        .unwrap();
    let result = a.app.export.run(s);
    assert!(
        matches!(result, Err(Error::RawChanged { photo_id }) if photo_id == b.id),
        "{result:?}"
    );
    assert_eq!(process_count(&a), issued);
    let after: Vec<(String, Vec<u8>)> = names(&dir)
        .into_iter()
        .map(|n| (n.clone(), fs::read(dir.join(&n)).unwrap()))
        .collect();
    assert_eq!(after, before);
    assert_eq!(names(&a.export_root()), vec!["K".to_string()]);
    assert_eq!(
        export_states(&a).last().map(|s| s.2),
        Some(ExportState::Failed)
    );
}

#[test]
fn c6a_stray_completion_of_an_old_job_is_ignored() {
    let a = app();
    let s = session(&a, "K");
    let main = photo(&a, s, "a", C::Main);
    export(&a, s);
    let old_job = a.fake.jobs_for(&variant(&main))[0].clone();
    a.app.export.run(s).unwrap();
    let stray = a.dir.path().join("Output/stray.jpg");
    fs::create_dir_all(stray.parent().unwrap()).unwrap();
    fs::write(&stray, b"stale bytes from the old job").unwrap();
    a.deliver_hook(
        &serde_json::json!({"hook":"processing_done","job":old_job,"raw":main.raw_path,"outputs":[stray]})
            .to_string(),
    )
    .unwrap();
    a.complete_all_renders().unwrap();
    let jobs = a.fake.jobs_for(&variant(&main));
    assert_eq!(jobs.len(), 2);
    assert_eq!(
        fs::read(a.export_root().join("K/K_01_main.jpg")).unwrap(),
        FakeCaptureOneScripting::output_bytes(&jobs[1])
    );
}

#[test]
fn c6b_export_render_timeout_reissues_once_then_fails() {
    let a = app();
    let s = session(&a, "K");
    let mains: Vec<Photo> = ["a", "b"]
        .iter()
        .map(|n| photo(&a, s, n, C::Main))
        .collect();
    let run = a.app.export.run(s).unwrap();
    assert!(
        a.export_root()
            .join(format!(".K.partial-{}", run.0))
            .is_dir()
    );
    a.tick(120_000).unwrap();
    for photo in &mains {
        assert_eq!(a.fake.jobs_for(&variant(photo)).len(), 2);
    }
    assert_ne!(
        export_states(&a).last().map(|s| s.2),
        Some(ExportState::Failed)
    );
    a.tick(120_000).unwrap();
    for photo in &mains {
        assert_eq!(a.fake.jobs_for(&variant(photo)).len(), 2);
    }
    assert_eq!(
        export_states(&a).last().map(|s| s.2),
        Some(ExportState::Failed)
    );
    assert!(names(&a.export_root()).is_empty());
}
