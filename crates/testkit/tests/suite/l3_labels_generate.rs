//! L3·U2 (`docs/implementation/plans/L3_generate_and_download.md` L3·C5–C9): generating (one
//! key per chunk, retries with the same key, Try again, the double press) and the result
//! (MC-L2.6–.10), through `App.labels` on `TestApp::with_pricing()`, connected and `Allowed`.
//! "The run requests" = the recorded requests whose path ends `/labels/runs`. Setup helpers:
//! `l3_labels_check`.

use crate::l3_labels_check::{
    SMALL_TEMPLATE, T, body_ids, check, clear, connected, ids, many, product, requests_ending,
    standard,
};
use psm_core::{
    domain::label::{
        LabelArtifact, LabelFailure, LabelRunResult, LabelStatus, LabelsAccess, LabelsError,
    },
    events::AppEvent,
    services::{items::ItemsService, labels::LabelsFailure},
};
use psm_testkit::TestApp;
use rusqlite::Connection;
use serde_json::Value;
use std::sync::Arc;

fn runs(a: &TestApp) -> Vec<Value> {
    requests_ending(a, "/labels/runs")
        .into_iter()
        .map(|(_, body)| body)
        .collect()
}

fn key(body: &Value) -> String {
    body["idempotencyKey"].as_str().unwrap().to_owned()
}

fn failure(id: &str, sku: Option<&str>, title: Option<&str>, message: &str) -> LabelFailure {
    LabelFailure {
        product_id: Some(id.into()),
        sku: sku.map(Into::into),
        title: title.map(Into::into),
        message: message.into(),
    }
}

/// A linked session for `Pn`: the shop product `gid://shopify/Product/<n>` adopted and given a
/// session; answers the session.
fn linked(a: &TestApp, n: &str) -> i64 {
    let external = a.fake_shop().add_product_with_id(
        &format!("gid://shopify/Product/{n}"),
        &format!("TEST-{n}"),
        &format!("TEST-{n} shop title"),
        None,
    );
    let item = ItemsService::new(a.store(), a.clock.as_ref())
        .with_commerce(Some(a.fake_shop()))
        .adopt(&external)
        .unwrap()
        .id;
    a.sessions().create(item).unwrap().session.id
}

fn labels_changed(a: &TestApp) -> Vec<(Vec<String>, bool)> {
    a.events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AppEvent::LabelsChanged {
                product_ids,
                access,
            } => Some((product_ids.clone(), *access)),
            _ => None,
        })
        .collect()
}

fn result_counts(r: &LabelRunResult) -> (i64, i64, i64, i64, i64) {
    (
        r.total,
        r.rendered,
        r.marked_printed,
        r.reprinted,
        r.no_price,
    )
}

fn generate(a: &TestApp, list: &[&str]) -> (String, LabelRunResult) {
    let c = check(a, list);
    let result = a.app.labels.generate(&c.id).unwrap();
    (c.id, result)
}

#[test]
fn l3_c5a_one_run_of_the_ready_products_with_a_new_key() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["1", "2", "3"]);
    clear(&a);
    let result = a.app.labels.generate(&c.id).unwrap();
    let bodies = runs(&a);
    assert_eq!(bodies.len(), 1);
    assert_eq!(body_ids(&bodies[0]), ids(&["1", "2"]));
    let k = key(&bodies[0]);
    assert!(k.starts_with("sm-"), "{k}");
    assert_eq!(k.chars().count(), 39);
    assert_eq!(result_counts(&result), (3, 2, 1, 0, 1));
    assert_eq!(
        result.failures,
        vec![failure(
            "3",
            Some("TEST-3"),
            Some("Test product 3"),
            "Barcode: no barcode."
        )]
    );
    assert!(result.not_run.is_empty());
    assert_eq!(result.artifacts.len(), 1);
    let art = &result.artifacts[0];
    assert_eq!(
        (art.width_mm, art.height_mm, art.page_count),
        (70.0, 100.0, 2)
    );
    assert_eq!(result.failed_product_ids, ids(&["3"]));
}

#[test]
fn l3_c5b_the_markers_follow_the_run() {
    let a = connected();
    standard(&a);
    let s1 = linked(&a, "1");
    let c = check(&a, &["1", "2", "3"]);
    clear(&a);
    a.app.labels.generate(&c.id).unwrap();
    let fake = a.fake_pricing();
    assert!(fake.label("1").unwrap().printed_at.is_some());
    assert_eq!(fake.label("2").unwrap().printed_at, None);
    let run_at = fake.runs()[0].created_at;
    let before = fake.request_lines().len();
    let labels = a.app.labels.session_labels(&[s1], false).unwrap();
    let state = labels[0].state.unwrap();
    assert_eq!(state.status, LabelStatus::Printed);
    assert_eq!(state.printed_at, Some(run_at));
    assert_eq!(fake.request_lines().len(), before);
    assert_eq!(labels_changed(&a), vec![(ids(&["1", "2"]), false)]);
}

#[test]
fn l3_c5c_a_reprint_keeps_its_time_and_logs_no_print() {
    let a = connected();
    standard(&a);
    let s4 = linked(&a, "4");
    let (_, result) = generate(&a, &["4"]);
    assert_eq!(
        (result.reprinted, result.marked_printed, result.rendered),
        (1, 0, 1)
    );
    assert_eq!(a.fake_pricing().printed_events("4"), 0);
    let state = a.app.labels.session_labels(&[s4], false).unwrap()[0]
        .state
        .unwrap();
    assert_eq!(state.status, LabelStatus::Printed);
    assert_eq!(state.printed_at, Some(T));
}

#[test]
fn l3_c5d_nothing_ready_is_nothing_to_generate() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["3"]);
    let error = a.app.labels.generate(&c.id).unwrap_err();
    assert!(
        matches!(error, LabelsFailure::NothingToGenerate),
        "{error:?}"
    );
    assert!(runs(&a).is_empty());
}

#[test]
fn l3_c5e_only_the_kept_check_generates() {
    let a = connected();
    standard(&a);
    let error = a.app.labels.generate("no-such-check").unwrap_err();
    assert!(matches!(error, LabelsFailure::NotFound), "{error:?}");
    let first = check(&a, &["1"]);
    check(&a, &["2"]);
    let error = a.app.labels.generate(&first.id).unwrap_err();
    assert!(matches!(error, LabelsFailure::NotFound), "{error:?}");
    assert!(runs(&a).is_empty());
}

#[test]
fn l3_c6a_two_chunks_two_runs_two_keys() {
    let a = connected();
    let list = many(&a, 301);
    let c = a.app.labels.check(&list).unwrap();
    a.app.labels.generate(&c.id).unwrap();
    let bodies = runs(&a);
    assert_eq!(bodies.len(), 2);
    assert_eq!(body_ids(&bodies[0]), list[..300].to_vec());
    assert_eq!(body_ids(&bodies[1]), list[300..].to_vec());
    assert_ne!(key(&bodies[0]), key(&bodies[1]));
    assert_eq!(a.fake_pricing().runs().len(), 2);
}

#[test]
fn l3_c6b_the_result_sums_the_runs() {
    let a = connected();
    let list = many(&a, 301);
    let c = a.app.labels.check(&list).unwrap();
    let result = a.app.labels.generate(&c.id).unwrap();
    assert_eq!(
        (result.total, result.rendered, result.marked_printed),
        (301, 301, 301)
    );
    let run_ids: Vec<String> = a
        .fake_pricing()
        .runs()
        .iter()
        .map(|r| r.id.clone())
        .collect();
    let from: Vec<String> = result
        .artifacts
        .iter()
        .map(|art| art.download_url.split('/').nth(7).unwrap().to_owned())
        .collect();
    assert_eq!(from, run_ids);
}

#[test]
fn l3_c6c_two_sizes_are_two_artifacts_of_one_run() {
    let a = connected();
    standard(&a);
    a.fake_pricing().set_label("2", |l| {
        l.template = Some(SMALL_TEMPLATE.into());
        l.price_minor = Some(100);
    });
    let (_, result) = generate(&a, &["1", "2"]);
    assert_eq!(runs(&a).len(), 1);
    let sizes: Vec<(f64, f64, i64)> = result
        .artifacts
        .iter()
        .map(|art: &LabelArtifact| (art.width_mm, art.height_mm, art.page_count))
        .collect();
    assert_eq!(sizes, vec![(70.0, 100.0, 1), (50.0, 30.0, 1)]);
}

#[test]
fn l3_c7a_a_lost_answer_is_sent_again_with_the_same_body() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["1"]);
    a.fake_pricing().drop_answer_next(1);
    let result = a.app.labels.generate(&c.id).unwrap();
    assert_eq!(result.marked_printed, 1);
    let bodies = runs(&a);
    assert_eq!(bodies.len(), 2);
    assert_eq!(bodies[0], bodies[1]);
    assert_eq!(a.fake_pricing().runs().len(), 1);
    assert_eq!(a.fake_pricing().printed_events("1"), 1);
}

#[test]
fn l3_c7b_two_server_errors_then_success() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["1"]);
    a.fake_pricing().fail_matching("/labels/runs", 2, 500);
    assert!(a.app.labels.generate(&c.id).is_ok());
    let bodies = runs(&a);
    assert_eq!(bodies.len(), 3);
    assert!(bodies.iter().all(|b| key(b) == key(&bodies[0])));
}

#[test]
fn l3_c7c_try_again_reuses_the_key() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["1"]);
    a.fake_pricing().fail_matching("/labels/runs", 3, 500);
    let error = a.app.labels.generate(&c.id).unwrap_err();
    assert!(
        matches!(error, LabelsFailure::Provider(LabelsError::Unreachable(_))),
        "{error:?}"
    );
    assert_eq!(runs(&a).len(), 3);
    assert!(a.app.labels.generate(&c.id).is_ok());
    let bodies = runs(&a);
    assert_eq!(bodies.len(), 4);
    assert!(bodies.iter().all(|b| key(b) == key(&bodies[0])));
}

#[test]
fn l3_c7d_a_busy_renderer_marks_nothing() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["1"]);
    a.fake_pricing().fail_matching_with(
        "/labels/runs",
        3,
        503,
        "renderer_unavailable",
        "The renderer is busy.",
    );
    let error = a.app.labels.generate(&c.id).unwrap_err();
    assert!(
        matches!(
            error,
            LabelsFailure::Provider(LabelsError::RendererUnavailable)
        ),
        "{error:?}"
    );
    let bodies = runs(&a);
    assert_eq!(bodies.len(), 3);
    assert!(bodies.iter().all(|b| key(b) == key(&bodies[0])));
    assert_eq!(a.fake_pricing().label("1").unwrap().printed_at, None);
}

/// L3·C7(e): 301 products whose second chunk fails three times.
fn second_chunk_failed(a: &TestApp) -> (Vec<String>, String) {
    let list = many(a, 301);
    let c = a.app.labels.check(&list).unwrap();
    a.fake_pricing()
        .fail_matching_after("/labels/runs", 1, 3, 500);
    let error = a.app.labels.generate(&c.id).unwrap_err();
    assert!(
        matches!(error, LabelsFailure::Provider(LabelsError::Unreachable(_))),
        "{error:?}"
    );
    (list, c.id)
}

#[test]
fn l3_c7e_try_again_continues_at_the_failed_chunk() {
    let a = connected();
    let (list, id) = second_chunk_failed(&a);
    let partial = a.app.labels.partial(&id).unwrap();
    assert_eq!((partial.total, partial.rendered), (301, 300));
    assert_eq!(partial.not_run, vec![list[300].clone()]);
    assert_eq!(partial.failed_product_ids, vec![list[300].clone()]);
    let first_run = &a.fake_pricing().runs()[0];
    assert!(partial.artifacts.iter().all(|art| {
        art.download_url
            .contains(&format!("/runs/{}/", first_run.id))
    }));
    assert_eq!(partial.artifacts.len(), 1);
    let before = runs(&a);
    let chunk_keys: Vec<String> = before.iter().map(key).collect();
    let result = a.app.labels.generate(&id).unwrap();
    let after = runs(&a);
    assert_eq!(after.len(), before.len() + 1);
    let last = after.last().unwrap();
    assert_eq!(body_ids(last), vec![list[300].clone()]);
    assert_eq!(key(last), chunk_keys[1]);
    assert_ne!(key(last), chunk_keys[0]);
    assert_eq!(result.rendered, 301);
}

#[test]
fn l3_c7f_a_refused_run_denies_labels() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["1"]);
    a.fake_pricing().fail_matching_with(
        "/labels/runs",
        1,
        403,
        "insufficient_scope",
        "This key lacks labels:write.",
    );
    let error = a.app.labels.generate(&c.id).unwrap_err();
    assert!(
        matches!(error, LabelsFailure::Provider(LabelsError::NotAllowed)),
        "{error:?}"
    );
    assert_eq!(runs(&a).len(), 1);
    assert_eq!(a.app.labels.access().unwrap(), LabelsAccess::Denied);
}

#[test]
fn l3_c7g_a_run_that_finds_no_product_fails_them_all() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["1"]);
    a.fake_pricing().remove_label_product("1");
    let result = a.app.labels.generate(&c.id).unwrap();
    assert_eq!(result.rendered, 0);
    assert_eq!(
        result.failures,
        vec![failure(
            "1",
            Some("TEST-1"),
            Some("Test product 1"),
            "None of these products were found in Pricing."
        )]
    );
    assert_eq!(runs(&a).len(), 1);
}

#[test]
fn l3_c7h_refusals_and_odd_answers_are_not_retried() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["1"]);
    a.fake_pricing()
        .fail_matching_with("/labels/runs", 1, 422, "validation_error", "M");
    let error = a.app.labels.generate(&c.id).unwrap_err();
    assert!(
        matches!(
            &error,
            LabelsFailure::Provider(LabelsError::Rejected { message, .. }) if message == "M"
        ),
        "{error:?}"
    );
    assert_eq!(runs(&a).len(), 1);
    let other = check(&a, &["2"]);
    clear(&a);
    a.fake_pricing().fail_matching("/labels/runs", 1, 201);
    let error = a.app.labels.generate(&other.id).unwrap_err();
    assert!(
        matches!(error, LabelsFailure::Provider(LabelsError::BadReply(_))),
        "{error:?}"
    );
    assert_eq!(runs(&a).len(), 1);
}

#[test]
fn l3_c8a_a_double_press_makes_no_second_run() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["1"]);
    let fake = a.pricing.clone().unwrap();
    fake.hold();
    let labels = Arc::clone(&a.app.labels);
    let id = c.id.clone();
    let first = std::thread::spawn(move || labels.generate(&id));
    assert!(fake.wait_until_held());
    let error = a.app.labels.generate(&c.id).unwrap_err();
    assert!(matches!(error, LabelsFailure::Busy), "{error:?}");
    let error = a.app.labels.check(&ids(&["2"])).unwrap_err();
    assert!(matches!(error, LabelsFailure::Busy), "{error:?}");
    fake.release();
    assert!(first.join().unwrap().is_ok());
    assert_eq!(runs(&a).len(), 1);
}

#[test]
fn l3_c8b_a_finished_generation_answers_again_without_a_request() {
    let a = connected();
    standard(&a);
    let (id, result) = generate(&a, &["1", "2", "3"]);
    let before = a.fake_pricing().request_lines().len();
    assert_eq!(a.app.labels.generate(&id).unwrap(), result);
    assert_eq!(a.fake_pricing().request_lines().len(), before);
}

#[test]
fn l3_c8c_nothing_of_a_check_survives_a_restart() {
    let mut a = connected();
    standard(&a);
    let (id, _) = generate(&a, &["1"]);
    a.kill_and_restart().unwrap();
    let error = a.app.labels.generate(&id).unwrap_err();
    assert!(matches!(error, LabelsFailure::NotFound), "{error:?}");
    let conn = Connection::open(a.app.paths.db()).unwrap();
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert!(
        !tables.iter().any(|t| t.to_lowercase().contains("label")),
        "{tables:?}"
    );
    let mut columns = Vec::new();
    for table in &tables {
        let names: Vec<String> = conn
            .prepare(&format!("SELECT name FROM pragma_table_info('{table}')"))
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        for name in names {
            if name.to_lowercase().contains("label") {
                columns.push(format!("{table}.{name}"));
            }
        }
    }
    columns.sort();
    assert_eq!(
        columns,
        [
            "attribute_definitions.label",
            "conversation_sync.labels_access"
        ]
    );
}

#[test]
fn l3_c9a_a_product_failing_in_the_run_is_listed() {
    let a = connected();
    standard(&a);
    a.fake_pricing()
        .set_label("2", |l| l.price_minor = Some(100));
    a.fake_pricing()
        .fail_in_run("2", "missing_code_value", "Barcode: no barcode.");
    let (_, result) = generate(&a, &["1", "2"]);
    assert_eq!((result.total, result.rendered), (2, 1));
    assert_eq!(
        result.failures,
        vec![failure(
            "2",
            Some("TEST-2"),
            Some("Test product 2"),
            "Barcode: no barcode."
        )]
    );
    assert_eq!(result.failed_product_ids, ids(&["2"]));
}

#[test]
fn l3_c9b_a_product_the_run_did_not_find_is_listed() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["1", "2"]);
    a.fake_pricing().remove_label_product("2");
    let result = a.app.labels.generate(&c.id).unwrap();
    assert_eq!(
        result.failures,
        vec![failure(
            "2",
            Some("TEST-2"),
            Some("Test product 2"),
            "No product with id '2' in this store."
        )]
    );
    assert_eq!((result.rendered, result.total), (1, 2));
}

#[test]
fn l3_c9c_check_failures_come_before_run_failures() {
    let a = connected();
    standard(&a);
    a.fake_pricing()
        .set_label("2", |l| l.price_minor = Some(100));
    a.fake_pricing()
        .fail_in_run("2", "missing_code_value", "Barcode: no barcode.");
    let (_, result) = generate(&a, &["1", "2", "3"]);
    let order: Vec<Option<String>> = result
        .failures
        .iter()
        .map(|f| f.product_id.clone())
        .collect();
    assert_eq!(order, vec![Some("3".to_owned()), Some("2".to_owned())]);
    assert_eq!((result.rendered, result.total), (1, 3));
}

#[test]
fn l3_c9d_everything_generated_lists_nothing() {
    let a = connected();
    product(&a, "1", Some(100), None);
    product(&a, "2", Some(200), None);
    let (_, result) = generate(&a, &["1", "2"]);
    assert!(result.failures.is_empty());
    assert!(result.not_run.is_empty());
    assert_eq!((result.rendered, result.total), (2, 2));
}

#[test]
fn l3_c9e_a_partial_result_is_never_fully_generated() {
    let a = connected();
    let (list, id) = second_chunk_failed(&a);
    let partial = a.app.labels.partial(&id).unwrap();
    assert!(partial.rendered < partial.total);
    assert_eq!((partial.rendered, partial.total), (300, 301));
    assert!(partial.not_run.contains(&list[300]));
    assert!(partial.failed_product_ids.contains(&list[300]));
}
