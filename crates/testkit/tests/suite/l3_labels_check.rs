//! L3·U1 (`docs/implementation/plans/L3_generate_and_download.md` L3·C1–C3): the check, the
//! preview picture and the kept check (MC-L2.1–.5), through `App.labels` on
//! `TestApp::with_pricing()`, connected and `Allowed`, with the fake Pricing server and the fake
//! label files. "Pn" = a label product of the fake Pricing with product id `n`; the fake has the
//! default template `Label 70×100` (70 × 100 mm) unless a row says otherwise. The helpers here
//! are shared with the U2 and U3 stems.

use psm_core::{
    Error,
    domain::{
        conversation::ConnectionState,
        label::{CheckOutcome, LabelCheck, LabelSize, LabelStatus, LabelsError, PlanItem},
    },
    ports::{clock::Clock, conversations::ConversationStore},
    services::labels::LabelsFailure,
};
use psm_integrations::conversations::beyo_pricing::label_types::IntegrationLabelTemplate;
use psm_testkit::{TestApp, fake_pricing::LabelSpec, test_app::PRICING_KEY};
use serde_json::json;

/// 2026-11-04T09:00:00Z: when P4 was printed.
pub(crate) const T: i64 = 1_793_782_800_000;
pub(crate) const DEFAULT_TEMPLATE: &str = "tpl-70x100";
pub(crate) const SMALL_TEMPLATE: &str = "tpl-50x30";

pub(crate) fn app() -> TestApp {
    let a = TestApp::with_pricing().unwrap();
    a.clock.advance(1_000_000);
    a
}

pub(crate) fn connect(a: &TestApp) {
    let connection = a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    assert_eq!(connection.state, ConnectionState::Ready);
    let mut n = 0;
    while a.app.conversations.tick(a.clock.now_ms()).unwrap().more {
        n += 1;
        assert!(n < 50, "the check never completed");
    }
}

pub(crate) fn clear(a: &TestApp) {
    a.fake_pricing().clear_requests();
    a.events.0.lock().unwrap().clear();
}

pub(crate) fn add_template(
    a: &TestApp,
    id: &str,
    name: &str,
    size: (f64, f64),
    is_default: bool,
    types: &[&str],
) {
    a.fake_pricing().add_template(IntegrationLabelTemplate {
        id: id.into(),
        name: name.into(),
        width_mm: size.0,
        height_mm: size.1,
        is_default,
        product_types: types.iter().map(|t| (*t).to_owned()).collect(),
    });
}

/// A connected app whose fake has no template.
pub(crate) fn connected_bare() -> TestApp {
    let a = app();
    connect(&a);
    clear(&a);
    a
}

/// A connected app whose fake has the default template `Label 70×100` (product types `Tables`,
/// `Chairs`) and the 50 × 30 template `Small tag` (no product types, not default).
pub(crate) fn connected() -> TestApp {
    let a = connected_bare();
    add_template(
        &a,
        DEFAULT_TEMPLATE,
        "Label 70×100",
        (70.0, 100.0),
        true,
        &["Tables", "Chairs"],
    );
    add_template(&a, SMALL_TEMPLATE, "Small tag", (50.0, 30.0), false, &[]);
    a
}

/// A label product `id`: SKU `TEST-<id>`, title `Test product <id>`, priced unless `price` is
/// none, printed at `printed_at`.
pub(crate) fn product(a: &TestApp, id: &str, price: Option<i64>, printed_at: Option<i64>) {
    a.fake_pricing().add_label_product(LabelSpec {
        product_id: id.into(),
        sku: Some(format!("TEST-{id}")),
        title: format!("Test product {id}"),
        price_minor: price,
        printed_at,
        ..LabelSpec::default()
    });
}

/// The products of L3·C1(a) and P4: P1 priced and not printed; P2 without a price; P3 failing
/// with `missing_code_value`; P4 printed at `T`.
pub(crate) fn standard(a: &TestApp) {
    product(a, "1", Some(890_000), None);
    product(a, "2", None, None);
    product(a, "3", Some(450_000), None);
    a.fake_pricing()
        .fail_label("3", "missing_code_value", "Barcode: no barcode.");
    product(a, "4", Some(120_000), Some(T));
}

pub(crate) fn ids(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

pub(crate) fn store_id(a: &TestApp) -> String {
    a.app.store.sync().unwrap().store.unwrap().id
}

pub(crate) fn label_requests(a: &TestApp) -> Vec<String> {
    a.fake_pricing()
        .request_lines()
        .into_iter()
        .filter(|line| line.contains("/labels/"))
        .collect()
}

/// The recorded requests whose path ends with `suffix`, as (line, body).
pub(crate) fn requests_ending(a: &TestApp, suffix: &str) -> Vec<(String, serde_json::Value)> {
    a.fake_pricing()
        .requests()
        .into_iter()
        .filter(|r| r.path.ends_with(suffix))
        .map(|r| (r.line(), r.body.clone().unwrap_or_default()))
        .collect()
}

/// The `productId`s of a plan or run body.
pub(crate) fn body_ids(body: &serde_json::Value) -> Vec<String> {
    body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["productId"].as_str().unwrap().to_owned())
        .collect()
}

pub(crate) fn check(a: &TestApp, list: &[&str]) -> LabelCheck {
    a.app.labels.check(&ids(list)).unwrap()
}

pub(crate) fn items(check: &LabelCheck) -> Vec<PlanItem> {
    check.groups.iter().flat_map(|g| g.items.clone()).collect()
}

fn item<'c>(check: &'c LabelCheck, id: &str) -> &'c PlanItem {
    check
        .groups
        .iter()
        .flat_map(|g| g.items.iter())
        .find(|i| i.product_id == id)
        .expect("item in the check")
}

fn previews_dir(a: &TestApp) -> std::path::PathBuf {
    a.app.paths.root.join("previews/labels")
}

#[test]
fn l3_c1a_one_plan_request_and_the_merged_check() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["1", "2", "3"]);
    let store = store_id(&a);
    let plans = requests_ending(&a, "/labels/plan");
    assert_eq!(plans.len(), 1, "{plans:?}");
    assert_eq!(plans[0].0, format!("POST /stores/{store}/labels/plan"));
    assert_eq!(body_ids(&plans[0].1), ids(&["1", "2", "3"]));
    assert_eq!(
        label_requests(&a)
            .iter()
            .filter(|l| **l == format!("GET /stores/{store}/labels/templates"))
            .count(),
        1
    );
    let s = c.summary;
    assert_eq!(
        (
            s.items,
            s.ready,
            s.failing,
            s.mark_printed,
            s.reprint,
            s.no_price
        ),
        (3, 2, 1, 1, 0, 1)
    );
    let outcomes: Vec<(String, CheckOutcome, Vec<String>)> = items(&c)
        .into_iter()
        .map(|i| (i.product_id, i.outcome, i.issues))
        .collect();
    assert_eq!(
        outcomes,
        vec![
            ("1".to_owned(), CheckOutcome::MarkPrinted, vec![]),
            ("2".to_owned(), CheckOutcome::NoPrice, vec![]),
            (
                "3".to_owned(),
                CheckOutcome::Fail,
                vec!["Barcode: no barcode.".to_owned()]
            ),
        ]
    );
    assert!(!c.id.is_empty());
}

#[test]
fn l3_c1b_repeated_products_are_checked_once() {
    let a = connected();
    standard(&a);
    check(&a, &["1", "1", "2"]);
    let plans = requests_ending(&a, "/labels/plan");
    assert_eq!(plans.len(), 1);
    assert_eq!(body_ids(&plans[0].1), ids(&["1", "2"]));
}

#[test]
fn l3_c1c_no_products_is_an_invalid_value_and_asks_nothing() {
    let a = connected();
    let error = a.app.labels.check(&[]).unwrap_err();
    assert!(
        matches!(error, LabelsFailure::Storage(Error::InvalidValue(_))),
        "{error:?}"
    );
    assert!(label_requests(&a).is_empty());
}

#[test]
fn l3_c1d_a_product_pricing_does_not_hold_is_left_out() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["1", "99"]);
    assert_eq!(
        items(&c)
            .iter()
            .map(|i| i.product_id.as_str())
            .collect::<Vec<_>>(),
        vec!["1"]
    );
    assert_eq!(c.unresolved.len(), 1);
    assert_eq!(c.unresolved[0].product_id.as_deref(), Some("99"));
    assert_eq!(
        c.unresolved[0].message,
        "No product with id '99' in this store."
    );
    assert_eq!(c.summary.items, 1);
}

#[test]
fn l3_c1e_none_held_is_no_products() {
    let a = connected();
    standard(&a);
    let error = a.app.labels.check(&ids(&["98", "99"])).unwrap_err();
    assert!(
        matches!(error, LabelsFailure::Provider(LabelsError::NoProducts)),
        "{error:?}"
    );
}

#[test]
fn l3_c1f_a_printed_product_is_a_reprint_with_its_time() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["4"]);
    let i = item(&c, "4");
    assert_eq!(i.outcome, CheckOutcome::Reprint);
    assert_eq!(i.status, LabelStatus::Printed);
    assert_eq!(i.printed_at, Some(T));
}

#[test]
fn l3_c1g_a_check_changes_nothing() {
    let a = connected();
    standard(&a);
    let before: Vec<_> = ["1", "2", "3"]
        .iter()
        .map(|id| a.fake_pricing().label(id))
        .collect();
    check(&a, &["1", "2", "3"]);
    assert!(
        !a.fake_pricing()
            .request_lines()
            .iter()
            .any(|l| l.contains("/labels/runs")),
    );
    let after: Vec<_> = ["1", "2", "3"]
        .iter()
        .map(|id| a.fake_pricing().label(id))
        .collect();
    assert_eq!(before, after);
    assert!(a.fake_shop().mutation_ops().is_empty());
}

#[test]
fn l3_c1h_no_key_or_denied_asks_nothing() {
    let a = app();
    standard(&a);
    let error = a.app.labels.check(&ids(&["1"])).unwrap_err();
    assert!(
        matches!(error, LabelsFailure::Off(ConnectionState::NoKey)),
        "{error:?}"
    );
    assert!(label_requests(&a).is_empty());

    let d = app();
    standard(&d);
    d.fake_pricing().deny_labels();
    connect(&d);
    clear(&d);
    let error = d.app.labels.check(&ids(&["1"])).unwrap_err();
    assert!(matches!(error, LabelsFailure::Denied), "{error:?}");
    assert!(label_requests(&d).is_empty());
}

#[test]
fn l3_c1i_a_failed_check_keeps_nothing() {
    let a = connected();
    standard(&a);
    let earlier = check(&a, &["1"]);
    a.fake_pricing().fail_matching("/labels/plan", 1, 500);
    let error = a.app.labels.check(&ids(&["1"])).unwrap_err();
    assert!(
        matches!(error, LabelsFailure::Provider(LabelsError::Unreachable(_))),
        "{error:?}"
    );
    for id in [earlier.id.as_str(), "any-id"] {
        let error = a.app.labels.generate(id).unwrap_err();
        assert!(matches!(error, LabelsFailure::NotFound), "{error:?}");
    }
}

/// `n` priced, unprinted products (ids `77000<k>`); answers their ids.
pub(crate) fn many(a: &TestApp, n: usize) -> Vec<String> {
    a.fake_pricing().add_label_products(n)
}

#[test]
fn l3_c2a_301_products_are_two_requests_shown_as_one() {
    let a = connected();
    let list = many(&a, 301);
    let c = a.app.labels.check(&list).unwrap();
    let plans = requests_ending(&a, "/labels/plan");
    assert_eq!(plans.len(), 2);
    assert_eq!(body_ids(&plans[0].1), list[..300].to_vec());
    assert_eq!(body_ids(&plans[1].1), list[300..].to_vec());
    let all = items(&c);
    assert_eq!(
        all.iter()
            .map(|i| (i.position, i.product_id.clone()))
            .collect::<Vec<_>>(),
        list.iter().cloned().enumerate().collect::<Vec<_>>()
    );
    assert_eq!(c.summary.items, 301);
    assert_eq!(c.summary.mark_printed, 301);
}

#[test]
fn l3_c2b_sizes_are_merged_across_chunks() {
    let a = connected();
    let list = many(&a, 301);
    for id in &list[200..300] {
        a.fake_pricing()
            .set_label(id, |l| l.template = Some(SMALL_TEMPLATE.into()));
    }
    let c = a.app.labels.check(&list).unwrap();
    assert_eq!(
        c.sizes,
        vec![
            LabelSize {
                width_mm: 70.0,
                height_mm: 100.0,
                count: 201
            },
            LabelSize {
                width_mm: 50.0,
                height_mm: 30.0,
                count: 100
            },
        ]
    );
}

#[test]
fn l3_c2c_a_chunk_pricing_holds_none_of_is_not_found() {
    let a = connected();
    let mut list = many(&a, 300);
    list.push("99999".into());
    let c = a.app.labels.check(&list).unwrap();
    assert_eq!(items(&c).len(), 300);
    assert_eq!(c.unresolved.len(), 1);
    assert_eq!(c.unresolved[0].product_id.as_deref(), Some("99999"));
    assert_eq!(c.unresolved[0].message, "Not found");
}

#[test]
fn l3_c2d_groups_by_template_with_the_template_less_last() {
    let a = connected_bare();
    add_template(
        &a,
        "tpl-a",
        "Label 70×100",
        (70.0, 100.0),
        false,
        &["Tables", "Chairs"],
    );
    add_template(&a, "tpl-b", "Small tag", (50.0, 30.0), false, &[]);
    product(&a, "1", Some(100), None);
    a.fake_pricing()
        .set_label("1", |l| l.product_type = Some("Tables".into()));
    product(&a, "2", Some(100), None);
    a.fake_pricing()
        .set_label("2", |l| l.template = Some("tpl-b".into()));
    product(&a, "3", Some(100), None);
    let c = check(&a, &["1", "2", "3"]);
    let shape: Vec<(Option<String>, Vec<String>, Vec<String>)> = c
        .groups
        .iter()
        .map(|g| {
            (
                g.template.as_ref().map(|t| t.id.clone()),
                g.product_types.clone(),
                g.items.iter().map(|i| i.product_id.clone()).collect(),
            )
        })
        .collect();
    assert_eq!(
        shape,
        vec![
            (
                Some("tpl-a".to_owned()),
                ids(&["Tables", "Chairs"]),
                ids(&["1"])
            ),
            (Some("tpl-b".to_owned()), vec![], ids(&["2"])),
            (None, vec![], ids(&["3"])),
        ]
    );
}

#[test]
fn l3_c2e_a_failed_templates_read_leaves_the_product_types_empty() {
    let a = connected();
    standard(&a);
    a.fake_pricing().fail_matching("/labels/templates", 1, 500);
    let c = check(&a, &["1"]);
    assert_eq!(c.groups.len(), 1);
    assert!(c.groups[0].product_types.is_empty());
    assert_eq!(
        c.groups[0].template.as_ref().map(|t| t.id.as_str()),
        Some(DEFAULT_TEMPLATE)
    );
}

/// L3·C3(a): a check of P1 and P2 with P1's picture made.
fn previewed(a: &TestApp) -> (LabelCheck, std::path::PathBuf) {
    standard(a);
    let c = check(a, &["1", "2"]);
    clear(a);
    let path = a.app.labels.preview(&c.id, "1").unwrap();
    (c, path)
}

#[test]
fn l3_c3a_the_preview_is_one_request_and_one_picture() {
    let a = connected();
    let (c, path) = previewed(&a);
    let previews = requests_ending(&a, "/labels/preview");
    assert_eq!(previews.len(), 1);
    assert_eq!(previews[0].1, json!({"productId": "1"}));
    assert_eq!(
        a.label_files().rasters(),
        vec![b"%PDF-1.4 fake preview 1".to_vec()]
    );
    assert_eq!(path, previews_dir(&a).join(format!("{}-1.png", c.id)));
    assert!(path.is_file());
}

#[test]
fn l3_c3b_a_second_ask_answers_the_same_picture_without_a_request() {
    let a = connected();
    let (c, path) = previewed(&a);
    let again = a.app.labels.preview(&c.id, "1").unwrap();
    assert_eq!(again, path);
    assert_eq!(requests_ending(&a, "/labels/preview").len(), 1);
    assert_eq!(a.label_files().rasters().len(), 1);
}

#[test]
fn l3_c3c_a_product_or_check_not_kept_is_not_found() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["1", "2"]);
    clear(&a);
    for (check_id, product_id) in [(c.id.as_str(), "9"), ("another-id", "1")] {
        let error = a.app.labels.preview(check_id, product_id).unwrap_err();
        assert!(matches!(error, LabelsFailure::NotFound), "{error:?}");
    }
    assert!(label_requests(&a).is_empty());
}

#[test]
fn l3_c3d_provider_refusals_of_a_preview() {
    let a = connected_bare();
    add_template(&a, "tpl-a", "Label 70×100", (70.0, 100.0), false, &[]);
    product(&a, "1", Some(100), None);
    a.fake_pricing()
        .set_label("1", |l| l.template = Some("tpl-a".into()));
    product(&a, "3", Some(100), None);
    let c = check(&a, &["1", "3"]);
    let error = a.app.labels.preview(&c.id, "3").unwrap_err();
    assert!(
        matches!(
            &error,
            LabelsFailure::Provider(LabelsError::Rejected { message, .. })
                if message == "No template applies."
        ),
        "{error:?}"
    );
    a.fake_pricing().fail_matching_with(
        "/labels/preview",
        1,
        503,
        "renderer_unavailable",
        "The renderer is busy.",
    );
    let error = a.app.labels.preview(&c.id, "1").unwrap_err();
    assert!(
        matches!(
            error,
            LabelsFailure::Provider(LabelsError::RendererUnavailable)
        ),
        "{error:?}"
    );
}

#[test]
fn l3_c3e_a_picture_that_cannot_be_made_never_blocks_generate() {
    let a = connected();
    standard(&a);
    let c = check(&a, &["1"]);
    a.label_files().fail_raster_next();
    let error = a.app.labels.preview(&c.id, "1").unwrap_err();
    assert!(
        matches!(error, LabelsFailure::PreviewFailed(_)),
        "{error:?}"
    );
    assert!(a.app.labels.generate(&c.id).is_ok());
}

#[test]
fn l3_c3f_a_new_check_drops_the_earlier_pictures() {
    let a = connected();
    let (first, path) = previewed(&a);
    check(&a, &["2"]);
    assert!(!path.exists());
    let error = a.app.labels.preview(&first.id, "1").unwrap_err();
    assert!(matches!(error, LabelsFailure::NotFound), "{error:?}");
}

#[test]
fn l3_c3g_check_and_preview_never_run() {
    let a = connected();
    let (c, _) = previewed(&a);
    a.app.labels.preview(&c.id, "1").unwrap();
    let _ = a.app.labels.preview(&c.id, "9");
    a.label_files().fail_raster_next();
    let _ = a.app.labels.preview(&c.id, "2");
    check(&a, &["2"]);
    let _ = a.app.labels.preview(&c.id, "1");
    assert!(
        !a.fake_pricing()
            .request_lines()
            .iter()
            .any(|l| l.contains("/labels/runs")),
    );
}
