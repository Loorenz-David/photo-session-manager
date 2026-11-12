//! P08 candidate criterion C0 (declared in the plan's Review log): the workspace loads its photos through
//! `session_get`, which must return the session's item and every photo except the deleted ones (ch06: `deleted`
//! hides the photo).
use psm_core::domain::photo::Classification as C;
use psm_testkit::TestApp;

#[test]
fn c0_session_view_lists_item_and_visible_photos() {
    let app = TestApp::new().unwrap();
    let (item, _) = app
        .items()
        .create_local("OAK-1", "Oak chair", "object", None)
        .unwrap();
    let session = app.sessions().create(item.id).unwrap().session.id;
    let kept = app.add_photo(session, "kept").unwrap().id;
    let main = app.add_photo(session, "main").unwrap().id;
    let gone = app.add_photo(session, "gone").unwrap().id;
    app.review().classify(main, C::Main).unwrap();
    app.review().delete(gone).unwrap();

    let view = app.sessions().view(session).unwrap();

    assert_eq!(view.item.map(|item| item.sku), Some("OAK-1".to_string()));
    let mut shown: Vec<(i64, C)> = view
        .photos
        .iter()
        .map(|photo| (photo.id, photo.classification))
        .collect();
    shown.sort_by_key(|(id, _)| *id);
    assert_eq!(shown, vec![(kept, C::Unreviewed), (main, C::Main)]);

    let unassigned = app.sessions().view(1).unwrap();
    assert_eq!(unassigned.item, None);
}
