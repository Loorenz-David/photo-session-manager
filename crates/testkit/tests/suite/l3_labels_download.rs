//! L3·U3 (`docs/implementation/plans/L3_generate_and_download.md` L3·C10): the download of a
//! label file into the labels folder and opening it (MC-L2.11, .13), through `App.labels` on
//! `TestApp::with_pricing()`, connected and `Allowed`, with `FakeLabelFiles` (files under the
//! test's own home; nothing is opened). "The labels folder" = `<test home>/Documents/Session
//! Manager Labels`. Setup helpers: `l3_labels_check`.

use crate::l3_labels_check::{check, clear, connected, label_requests, standard, store_id};
use psm_core::{
    domain::label::{LabelArtifact, LabelsError},
    services::{items::ItemsService, labels::LabelsFailure},
};
use psm_testkit::TestApp;
use std::{fs, path::PathBuf};

fn folder(a: &TestApp) -> PathBuf {
    a.dir.path().join("Documents/Session Manager Labels")
}

fn files_in(dir: &std::path::Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return vec![];
    };
    let mut names: Vec<String> = entries
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// L3·C5(a)'s generation: answers the check id and its one artifact.
fn generated(a: &TestApp) -> (String, LabelArtifact) {
    standard(a);
    let c = check(a, &["1", "2", "3"]);
    let result = a.app.labels.generate(&c.id).unwrap();
    clear(a);
    (c.id, result.artifacts[0].clone())
}

fn run_of(artifact: &LabelArtifact) -> String {
    artifact.download_url.split('/').nth(7).unwrap().to_owned()
}

fn bytes_of(artifact: &LabelArtifact) -> Vec<u8> {
    format!("%PDF-1.4 fake labels {} {}", run_of(artifact), artifact.id).into_bytes()
}

#[test]
fn l3_c10a_download_saves_the_file_and_opens_it() {
    let a = connected();
    let (id, art) = generated(&a);
    let saved = a.app.labels.download(&id, &art.id).unwrap();
    assert_eq!(
        label_requests(&a),
        vec![format!(
            "GET /stores/{}/labels/runs/{}/artifacts/{}",
            store_id(&a),
            run_of(&art),
            art.id
        )]
    );
    let path = folder(&a).join(&art.filename);
    assert_eq!(fs::read(&path).unwrap(), bytes_of(&art));
    assert_eq!(a.label_files().opened(), vec![path.clone()]);
    assert_eq!(saved.path, path);
    assert!(saved.opened);
}

#[test]
fn l3_c10b_a_second_download_is_numbered() {
    let a = connected();
    let (id, art) = generated(&a);
    let first = a.app.labels.download(&id, &art.id).unwrap();
    let second = a.app.labels.download(&id, &art.id).unwrap();
    let stem = art.filename.trim_end_matches(".pdf");
    assert_eq!(second.path, folder(&a).join(format!("{stem} (2).pdf")));
    assert_eq!(fs::read(&second.path).unwrap(), bytes_of(&art));
    assert_eq!(fs::read(&first.path).unwrap(), bytes_of(&art));
}

#[test]
fn l3_c10c_an_expired_file_saves_nothing() {
    let a = connected();
    let (id, art) = generated(&a);
    a.fake_pricing().expire_artifacts();
    let error = a.app.labels.download(&id, &art.id).unwrap_err();
    assert!(
        matches!(error, LabelsFailure::Provider(LabelsError::Expired)),
        "{error:?}"
    );
    assert!(files_in(&folder(&a)).is_empty());
}

#[test]
fn l3_c10d_a_failed_save_can_be_tried_again() {
    let a = connected();
    let (id, art) = generated(&a);
    a.label_files().fail_save_next("disk full");
    let error = a.app.labels.download(&id, &art.id).unwrap_err();
    assert!(
        matches!(&error, LabelsFailure::SaveFailed(reason) if reason == "disk full"),
        "{error:?}"
    );
    assert!(files_in(&folder(&a)).is_empty());
    let saved = a.app.labels.download(&id, &art.id).unwrap();
    assert_eq!(fs::read(&saved.path).unwrap(), bytes_of(&art));
}

#[test]
fn l3_c10e_a_file_preview_cannot_open_is_still_saved() {
    let a = connected();
    let (id, art) = generated(&a);
    a.label_files().fail_open_next();
    let saved = a.app.labels.download(&id, &art.id).unwrap();
    assert!(!saved.opened);
    assert_eq!(saved.path, folder(&a).join(&art.filename));
    assert_eq!(fs::read(&saved.path).unwrap(), bytes_of(&art));
}

#[test]
fn l3_c10f_only_a_file_of_the_kept_result_is_downloaded() {
    let a = connected();
    let (id, art) = generated(&a);
    for (check_id, artifact_id) in [
        (id.as_str(), "no-such-artifact"),
        ("no-such-check", art.id.as_str()),
    ] {
        let error = a.app.labels.download(check_id, artifact_id).unwrap_err();
        assert!(matches!(error, LabelsFailure::NotFound), "{error:?}");
    }
    let fresh = check(&a, &["1"]);
    clear(&a);
    let error = a.app.labels.download(&fresh.id, &art.id).unwrap_err();
    assert!(matches!(error, LabelsFailure::NotFound), "{error:?}");
    assert!(label_requests(&a).is_empty());
}

#[test]
fn l3_c10g_labels_never_touch_the_shop() {
    let a = connected();
    standard(&a);
    let external = a.fake_shop().add_product_with_id(
        "gid://shopify/Product/1",
        "TEST-1",
        "TEST-1 shop title",
        None,
    );
    let item = ItemsService::new(a.store(), a.clock.as_ref())
        .with_commerce(Some(a.fake_shop()))
        .adopt(&external)
        .unwrap()
        .id;
    let s1 = a.sessions().create(item).unwrap().session.id;
    a.fake_shop().state.lock().unwrap().reads.clear();
    let c = check(&a, &["1", "2", "3"]);
    a.app.labels.preview(&c.id, "1").unwrap();
    let result = a.app.labels.generate(&c.id).unwrap();
    a.app
        .labels
        .download(&c.id, &result.artifacts[0].id)
        .unwrap();
    assert!(a.fake_shop().mutation_ops().is_empty());
    assert!(a.fake_shop().state.lock().unwrap().reads.is_empty());
    clear(&a);
    a.app.storage.delete_session(s1, "test").unwrap();
    assert!(label_requests(&a).is_empty());
}
