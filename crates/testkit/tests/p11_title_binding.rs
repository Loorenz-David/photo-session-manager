//! P11 fix 1: the form's Title is the item's title (ch05 one home, ch08 "Title is a Product-step
//! field"): `form_get`, `session_get`'s item and the item row all show the same single title.

use psm_core::{
    domain::attribute::{AttributeGroup, Form, Provenance},
    services::attributes::{AttributesService, PRODUCT_TITLE},
};
use psm_testkit::TestApp;
use rusqlite::OptionalExtension;
use serde_json::json;

const T0: i64 = 1_761_350_400_000; // 2025-10-25T00:00:00Z

fn attrs(a: &TestApp) -> AttributesService<'_> {
    AttributesService::new(a.app.store.as_ref(), a.clock.as_ref(), None)
}

/// The title the form presents for the Product step's `product.title` field.
fn form_title(form: &Form) -> Option<String> {
    let product = form
        .groups
        .iter()
        .find(|g| g.group == AttributeGroup::Product)
        .unwrap();
    let field = product
        .fields
        .iter()
        .find(|f| f.definition.key == PRODUCT_TITLE)
        .unwrap();
    field.value.as_ref().map(|v| {
        assert!(v.valid);
        assert_eq!(v.provenance, Provenance::Human);
        serde_json::from_str::<String>(&v.value).unwrap()
    })
}

fn stored_rows(a: &TestApp, item_id: i64) -> Option<String> {
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            "SELECT value FROM attribute_values WHERE item_id=?1 AND key='product.title'",
            [item_id],
            |r| r.get(0),
        )
        .optional()
        .unwrap()
}

#[test]
fn title_set_in_the_form_is_the_items_title_everywhere() {
    let a = TestApp::new().unwrap();
    a.clock.advance(T0);
    let (item, session) = a
        .items()
        .create_local("K", "Oak", "Sideboard", None)
        .unwrap();
    let (item_id, session_id) = (item.id, session.session.id);
    assert_eq!(
        form_title(&attrs(&a).form_get(item_id).unwrap()).as_deref(),
        Some("Oak")
    );

    a.clock.advance(60_000);
    attrs(&a)
        .set_value(
            item_id,
            PRODUCT_TITLE,
            json!("Danish teak sideboard "),
            Provenance::Human,
        )
        .unwrap();

    let view = a.sessions().view(session_id).unwrap();
    assert_eq!(view.item.unwrap().title, "Danish teak sideboard");
    assert_eq!(
        a.store().item(item_id).unwrap().title,
        "Danish teak sideboard"
    );
    assert_eq!(
        form_title(&attrs(&a).form_get(item_id).unwrap()).as_deref(),
        Some("Danish teak sideboard")
    );
    assert_eq!(
        stored_rows(&a, item_id),
        None,
        "no second copy of the title"
    );
    assert_eq!(
        a.store()
            .session(session_id)
            .unwrap()
            .session
            .last_activity_at,
        T0 + 60_000
    );

    // Emptying the field (attribute_ai_clear) empties the item's title and the form shows no value.
    attrs(&a).clear(item_id, PRODUCT_TITLE).unwrap();
    assert_eq!(a.store().item(item_id).unwrap().title, "");
    assert_eq!(form_title(&attrs(&a).form_get(item_id).unwrap()), None);
}
