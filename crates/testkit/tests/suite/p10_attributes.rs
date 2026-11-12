//! P10 attributes: canonical values (MC-9, M16), provenance (ch08), damage-report selection
//! (MC-14, M21), presets (ch08), activity without dirtying (MC-4, MC-6), item.json (MC-8, M7).

use psm_core::{
    Error,
    domain::{
        attribute::{
            AiConfidence, AttributeGroup, Form, FormField, Preset, PresetField, Provenance,
        },
        photo::Classification,
    },
    services::attributes::{AttributesService, PRODUCT_DESCRIPTION, PRODUCT_TAGS, PRODUCT_TITLE},
};
use psm_testkit::{
    TestApp,
    fixtures::definitions::{DAMAGE_REPORT, INTEGRATION, shop_definitions},
};
use rusqlite::OptionalExtension;
use serde_json::{Value as Json, json};

const T0: i64 = 1_761_350_400_000; // 2025-10-25T00:00:00Z

fn app_with(document: bool) -> TestApp {
    let a = if document {
        TestApp::with_fake_document().unwrap()
    } else {
        TestApp::new().unwrap()
    };
    a.clock.advance(T0);
    attrs(&a)
        .import_definitions(INTEGRATION, shop_definitions())
        .unwrap();
    a
}

fn app() -> TestApp {
    app_with(false)
}

fn attrs(a: &TestApp) -> AttributesService<'_> {
    AttributesService::new(a.app.store.as_ref(), a.clock.as_ref(), Some(INTEGRATION))
}

/// A new local item of type Sideboard and its session.
fn item(a: &TestApp, sku: &str) -> (i64, i64) {
    let (item, session) = a
        .items()
        .create_local(sku, "Oak", "Sideboard", None)
        .unwrap();
    (item.id, session.session.id)
}

#[derive(Debug, PartialEq)]
struct Row {
    value: String,
    valid: bool,
    provenance: String,
    ai_confidence: Option<String>,
    ai_status: Option<String>,
    updated_at: i64,
}

/// The stored `attribute_values` row, read straight from the database.
fn row(a: &TestApp, item_id: i64, key: &str) -> Option<Row> {
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    conn.query_row(
        "SELECT value,valid,provenance,ai_confidence,ai_status,updated_at FROM attribute_values WHERE item_id=?1 AND key=?2",
        rusqlite::params![item_id, key],
        |r| {
            Ok(Row {
                value: r.get(0)?,
                valid: r.get(1)?,
                provenance: r.get(2)?,
                ai_confidence: r.get(3)?,
                ai_status: r.get(4)?,
                updated_at: r.get(5)?,
            })
        },
    )
    .optional()
    .unwrap()
}

/// `set_value` as a human, then the stored (value, valid).
fn set(a: &TestApp, item_id: i64, key: &str, raw: Json) -> (String, bool) {
    attrs(a)
        .set_value(item_id, key, raw, Provenance::Human)
        .unwrap();
    let row = row(a, item_id, key).unwrap();
    (row.value, row.valid)
}

fn canonical(text: &str) -> (String, bool) {
    (text.to_owned(), true)
}

fn verbatim(raw: Json) -> (String, bool) {
    (raw.to_string(), false)
}

fn session_row(a: &TestApp, session_id: i64) -> (i64, i64) {
    let s = a.store().session(session_id).unwrap().session;
    (s.last_activity_at, s.change_seq)
}

#[test]
fn c1a_text_nfc_trailing_whitespace_trimmed() {
    let a = app();
    let (i, _) = item(&a, "K");
    assert_eq!(
        set(&a, i, "custom.designer", json!(" He\u{301}llo  ")),
        canonical("\" H\u{e9}llo\"")
    );
}

#[test]
fn c1b_textarea_trailing_whitespace_trimmed() {
    let a = app();
    let (i, _) = item(&a, "K");
    assert_eq!(
        set(&a, i, "custom.condition_report", json!("a \n")),
        canonical("\"a\"")
    );
}

#[test]
fn c1c_url_trailing_whitespace_trimmed() {
    let a = app();
    let (i, _) = item(&a, "K");
    assert_eq!(
        set(&a, i, "custom.link", json!("https://x.se/ ")),
        canonical("\"https://x.se/\"")
    );
}

#[test]
fn c1d_color_lowercase_or_invalid() {
    let a = app();
    let (i, _) = item(&a, "K");
    assert_eq!(
        set(&a, i, "test.accent_color", json!("#AABBCC")),
        canonical("\"#aabbcc\"")
    );
    assert_eq!(
        set(&a, i, "test.accent_color", json!("red")),
        verbatim(json!("red"))
    );
}

#[test]
fn c1e_rich_text_one_blank_line_between_paragraphs() {
    let a = app();
    let (i, _) = item(&a, "K");
    assert_eq!(
        set(&a, i, "custom.restoration", json!("a\n\n\nb")),
        canonical("\"a\\n\\nb\"")
    );
}

#[test]
fn c1f_integer_only_json_integers() {
    let a = app();
    let (i, _) = item(&a, "K");
    let key = "custom.extensions_quantity";
    assert_eq!(set(&a, i, key, json!(12)), canonical("12"));
    assert_eq!(set(&a, i, key, json!(12.0)), verbatim(json!(12.0)));
    assert_eq!(set(&a, i, key, json!("12")), verbatim(json!("12")));
}

#[test]
fn c1g_decimal_strings_never_floats() {
    let a = app();
    let (i, _) = item(&a, "K");
    let key = "test.price";
    assert_eq!(set(&a, i, key, json!("1.50")), canonical("\"1.5\""));
    assert_eq!(set(&a, i, key, json!("2.0")), canonical("\"2\""));
    assert_eq!(set(&a, i, key, json!("-0.10")), canonical("\"-0.1\""));
    assert_eq!(set(&a, i, key, json!("1e3")), verbatim(json!("1e3")));
    assert_eq!(set(&a, i, key, json!(1.5)), verbatim(json!(1.5)));
}

#[test]
fn c1h_money_decimal_amount_upper_currency() {
    let a = app();
    let (i, _) = item(&a, "K");
    assert_eq!(
        set(
            &a,
            i,
            "test.value",
            json!({"amount":"10.50","currency":"sek"})
        ),
        canonical(r#"{"amount":"10.5","currency":"SEK"}"#)
    );
}

#[test]
fn c1i_dimension_listed_units_only() {
    let a = app();
    let (i, _) = item(&a, "K");
    let key = "custom.depth_dimension";
    assert_eq!(
        set(&a, i, key, json!({"value":"120.0","unit":"cm"})),
        canonical(r#"{"unit":"cm","value":"120"}"#)
    );
    let feet = json!({"value":"120.0","unit":"ft"});
    assert_eq!(set(&a, i, key, feet.clone()), verbatim(feet));
}

#[test]
fn c1j_boolean_only_json_booleans() {
    let a = app();
    let (i, _) = item(&a, "K");
    assert_eq!(set(&a, i, "test.is_signed", json!(true)), canonical("true"));
    assert_eq!(
        set(&a, i, "test.is_signed", json!("true")),
        verbatim(json!("true"))
    );
}

#[test]
fn c1k_date_only_dates() {
    let a = app();
    let (i, _) = item(&a, "K");
    assert_eq!(
        set(&a, i, "test.acquired_on", json!("2026-10-25")),
        canonical("\"2026-10-25\"")
    );
    assert_eq!(
        set(&a, i, "test.acquired_on", json!("2026-10-25T10:00:00Z")),
        verbatim(json!("2026-10-25T10:00:00Z"))
    );
}

#[test]
fn c1l_rating_decimal_strings_sorted_keys() {
    let a = app();
    let (i, _) = item(&a, "K");
    assert_eq!(
        set(
            &a,
            i,
            "reviews.rating",
            json!({"value":"4","min":"1.0","max":"5.0"})
        ),
        canonical(r#"{"max":"5","min":"1","value":"4"}"#)
    );
}

#[test]
fn c1m_numeric_looking_choices_stay_strings() {
    let a = app();
    let (i, _) = item(&a, "K");
    let key = "custom.extension_quantity";
    assert_eq!(set(&a, i, key, json!("1")), canonical("\"1\""));
    assert_eq!(set(&a, i, key, json!(1)), verbatim(json!(1)));
    assert_eq!(set(&a, i, key, json!("9")), verbatim(json!("9")));
}

#[test]
fn c1n_multi_choice_and_tags_deduplicated_in_order() {
    let a = app();
    let (i, _) = item(&a, "K");
    assert_eq!(
        set(&a, i, "custom.wood_type", json!(["Oak", "Teak", "Oak"])),
        canonical(r#"["Oak","Teak"]"#)
    );
    assert_eq!(
        set(&a, i, PRODUCT_TAGS, json!(["Oak", "Teak", "Oak"])),
        canonical(r#"["Oak","Teak"]"#)
    );
}

#[test]
fn c1o_media_list_stores_exclusions() {
    let a = app();
    let (i, _) = item(&a, "K");
    assert_eq!(
        set(&a, i, DAMAGE_REPORT, json!({"excluded":[3,1]})),
        canonical(r#"{"excluded":[3,1]}"#)
    );
}

#[test]
fn c1p_reference_opaque_valid_never_writable() {
    let a = app();
    let (i, _) = item(&a, "K");
    let key = "custom.inspection_report";
    let raw = json!({"id":"gid://shopify/Metaobject/1","fields":[{"k":"v"}, 2]});
    assert_eq!(set(&a, i, key, raw.clone()), (raw.to_string(), true));
    let definition = attrs(&a)
        .definitions(Some(INTEGRATION))
        .unwrap()
        .into_iter()
        .find(|d| d.key == key)
        .unwrap();
    assert!(!definition.writable);
}

#[test]
fn c2a_choice_outside_the_set_stored_invalid_verbatim() {
    let a = app();
    let (i, _) = item(&a, "K");
    assert_eq!(
        set(&a, i, "custom.condition_scratch", json!("minor")),
        ("\"minor\"".to_owned(), false)
    );
}

#[test]
fn c2b_human_edit_to_a_choice_makes_it_valid() {
    let a = app();
    let (i, _) = item(&a, "K");
    set(&a, i, "custom.condition_scratch", json!("minor"));
    let (value, valid) = set(&a, i, "custom.condition_scratch", json!("Minor"));
    assert_eq!((value.as_str(), valid), ("\"Minor\"", true));
    assert_eq!(
        row(&a, i, "custom.condition_scratch").unwrap().provenance,
        "human"
    );
}

#[test]
fn c3a_equal_canonical_value_is_not_written() {
    let a = app();
    let (i, s) = item(&a, "K");
    set(&a, i, "test.price", json!("1.50"));
    let first = (
        row(&a, i, "test.price").unwrap().updated_at,
        session_row(&a, s).0,
    );
    assert_eq!(first, (T0, T0));
    a.clock.advance(60_000);
    set(&a, i, "test.price", json!("1.5"));
    let second = (
        row(&a, i, "test.price").unwrap().updated_at,
        session_row(&a, s).0,
    );
    assert_eq!(second, first);
}

#[test]
fn c4a_human_edit_is_human_owned() {
    let a = app();
    let (i, _) = item(&a, "K");
    set(&a, i, "custom.designer", json!("Wegner"));
    assert_eq!(row(&a, i, "custom.designer").unwrap().provenance, "human");
}

#[test]
fn c4b_ai_never_overwrites_a_human_value() {
    let a = app();
    let (i, _) = item(&a, "K");
    set(&a, i, "custom.designer", json!("Wegner"));
    let before = row(&a, i, "custom.designer").unwrap();
    a.clock.advance(1_000);
    let refused = attrs(&a).ai_suggest(
        i,
        "custom.designer",
        json!("Juhl"),
        AiConfidence::High,
        Some("label on the back"),
        vec!["https://example.com".into()],
    );
    assert!(matches!(refused, Err(Error::InvalidValue(_))));
    assert_eq!(row(&a, i, "custom.designer").unwrap(), before);
}

#[test]
fn c4c_ai_suggestion_on_an_empty_field() {
    let a = app();
    let (i, _) = item(&a, "K");
    attrs(&a)
        .ai_suggest(
            i,
            "custom.designer",
            json!("Juhl"),
            AiConfidence::Medium,
            None,
            Vec::new(),
        )
        .unwrap();
    let r = row(&a, i, "custom.designer").unwrap();
    assert_eq!(
        (
            r.value.as_str(),
            r.provenance.as_str(),
            r.ai_status.as_deref(),
            r.ai_confidence.as_deref()
        ),
        ("\"Juhl\"", "ai", Some("suggested"), Some("medium"))
    );
}

#[test]
fn c4d_accept_all_skips_low_confidence() {
    let a = app();
    let (i, _) = item(&a, "K");
    let service = attrs(&a);
    for (key, confidence) in [
        ("custom.designer", AiConfidence::High),
        ("custom.manufacturer", AiConfidence::Medium),
        ("custom.shape", AiConfidence::Low),
    ] {
        service
            .ai_suggest(i, key, json!("x"), confidence, None, Vec::new())
            .unwrap();
    }
    assert_eq!(service.accept_all(i).unwrap(), 2);
    let status = |key| row(&a, i, key).unwrap().ai_status;
    assert_eq!(
        [
            status("custom.designer"),
            status("custom.manufacturer"),
            status("custom.shape")
        ],
        [
            Some("accepted".into()),
            Some("accepted".into()),
            Some("suggested".into())
        ]
    );
}

#[test]
fn c4e_clear_removes_the_value() {
    let a = app();
    let (i, _) = item(&a, "K");
    let service = attrs(&a);
    service
        .ai_suggest(
            i,
            "custom.designer",
            json!("Juhl"),
            AiConfidence::High,
            None,
            Vec::new(),
        )
        .unwrap();
    service.clear(i, "custom.designer").unwrap();
    assert_eq!(
        row(&a, i, "custom.designer").unwrap().ai_status.as_deref(),
        Some("cleared")
    );
    assert!(
        !service
            .values(i)
            .unwrap()
            .iter()
            .any(|v| v.key == "custom.designer")
    );
}

fn field<'f>(form: &'f Form, group: AttributeGroup, key: &str) -> &'f FormField {
    form.groups
        .iter()
        .find(|g| g.group == group)
        .unwrap()
        .fields
        .iter()
        .find(|f| f.definition.key == key)
        .unwrap()
}

fn selected(a: &TestApp, item_id: i64) -> Vec<i64> {
    let form = attrs(a).form_get(item_id).unwrap();
    field(&form, AttributeGroup::Condition, DAMAGE_REPORT)
        .selected
        .clone()
        .unwrap()
}

fn condition_photo(a: &TestApp, session_id: i64, name: &str) -> i64 {
    let photo = a.add_photo(session_id, name).unwrap();
    a.review()
        .classify(photo.id, Classification::Condition)
        .unwrap();
    photo.id
}

/// Condition photos p1, p2, p3 in this order; the damage report excludes p2.
fn c5_item(a: &TestApp) -> (i64, i64, [i64; 3]) {
    let (i, s) = item(a, "K");
    let p = [
        condition_photo(a, s, "p1.ARW"),
        condition_photo(a, s, "p2.ARW"),
        condition_photo(a, s, "p3.ARW"),
    ];
    set(a, i, DAMAGE_REPORT, json!({"excluded":[p[1]]}));
    (i, s, p)
}

#[test]
fn c5a_selection_is_condition_order_minus_exclusions() {
    let a = app();
    let (i, _, [p1, _, p3]) = c5_item(&a);
    assert_eq!(selected(&a, i), vec![p1, p3]);
}

#[test]
fn c5b_new_condition_photo_selected_by_default() {
    let a = app();
    let (i, s, [p1, _, p3]) = c5_item(&a);
    let p4 = condition_photo(&a, s, "p4.ARW");
    assert_eq!(selected(&a, i), vec![p1, p3, p4]);
}

#[test]
fn c5c_exclusion_of_a_photo_no_longer_condition_is_ignored() {
    let a = app();
    let (i, s, [p1, p2, p3]) = c5_item(&a);
    let p4 = condition_photo(&a, s, "p4.ARW");
    a.review().classify(p2, Classification::Main).unwrap();
    assert_eq!(selected(&a, i), vec![p1, p3, p4]);
    assert_eq!(
        row(&a, i, DAMAGE_REPORT).unwrap().value,
        format!(r#"{{"excluded":[{p2}]}}"#)
    );
}

#[test]
fn c6a_preset_orders_details_and_prefills_defaults() {
    let a = app();
    attrs(&a)
        .presets_set(Preset {
            item_type: "Sideboard".into(),
            group: AttributeGroup::Details,
            layout: vec![
                PresetField {
                    key: "custom.shape".into(),
                    default: Some(json!("Oak")),
                },
                PresetField {
                    key: "custom.years".into(),
                    default: None,
                },
            ],
        })
        .unwrap();
    let (i, _) = item(&a, "K");
    let form = attrs(&a).form_get(i).unwrap();
    let details = form
        .groups
        .iter()
        .find(|g| g.group == AttributeGroup::Details)
        .unwrap();
    let keys: Vec<&str> = details
        .fields
        .iter()
        .map(|f| f.definition.key.as_str())
        .collect();
    assert_eq!(keys, ["custom.shape", "custom.years"]);
    let k1 = &details.fields[0];
    let value = k1.value.as_ref().unwrap();
    assert_eq!(
        (value.value.as_str(), value.provenance, k1.is_default),
        ("\"Oak\"", Provenance::Default, true)
    );
}

#[test]
fn c6b_product_fields_never_removable() {
    let a = app();
    let refused = attrs(&a).presets_set(Preset {
        item_type: "Sideboard".into(),
        group: AttributeGroup::Product,
        layout: vec![
            PresetField {
                key: PRODUCT_DESCRIPTION.into(),
                default: None,
            },
            PresetField {
                key: PRODUCT_TAGS.into(),
                default: None,
            },
        ],
    });
    assert!(matches!(refused, Err(Error::InvalidValue(_))));
    let (i, _) = item(&a, "K");
    let form = attrs(&a).form_get(i).unwrap();
    let product: Vec<&str> = form
        .groups
        .iter()
        .find(|g| g.group == AttributeGroup::Product)
        .unwrap()
        .fields
        .iter()
        .map(|f| f.definition.key.as_str())
        .collect();
    assert_eq!(product, [PRODUCT_TITLE, PRODUCT_DESCRIPTION, PRODUCT_TAGS]);
}

#[test]
fn c7a_attribute_edit_sets_activity_and_keeps_change_seq() {
    let a = app();
    let (i, s) = item(&a, "K");
    let photo = a.add_photo(s, "a.ARW").unwrap();
    a.review().classify(photo.id, Classification::Main).unwrap();
    let n = session_row(&a, s).1;
    assert!(n > 0);
    a.clock.advance(3_600_000);
    let t = T0 + 3_600_000;
    set(&a, i, "custom.designer", json!("Wegner"));
    assert_eq!(session_row(&a, s), (t, n));
}

#[test]
fn c8a_item_json_attributes_hold_every_valid_value() {
    let a = app_with(true);
    let (i, s) = item(&a, "K");
    set(&a, i, "custom.years", json!("1950-1960s"));
    set(&a, i, "custom.extension_quantity", json!(1));
    a.app.export.run(s).unwrap();
    a.complete_all_renders().unwrap();
    let text = std::fs::read_to_string(a.export_root().join("K").join("item.json")).unwrap();
    let json: Json = serde_json::from_str(&text).unwrap();
    assert_eq!(json["attributes"], json!({"custom.years": "1950-1960s"}));
}
