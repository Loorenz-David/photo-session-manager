//! Owner correction A12: the "Search this photo" window's commands through the IPC: saved pages,
//! "Put in <field>" as a person's edit, and that a remote page can call none of them.

use crate::{commands, state::AppState};
use serde_json::{Value, json};
use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
use tauri::{
    ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
    webview::InvokeRequest,
};

type W = tauri::WebviewWindow<tauri::test::MockRuntime>;

fn call_from(w: &W, origin: &str, cmd: &str, args: Value) -> Result<Value, Value> {
    get_ipc_response(
        w,
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: origin.parse().unwrap(),
            body: InvokeBody::Json(args),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },
    )
    .map(|body| match body {
        InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
        InvokeResponseBody::Raw(_) => panic!("raw"),
    })
}

fn call(w: &W, cmd: &str, args: Value) -> Result<Value, Value> {
    call_from(w, "tauri://localhost", cmd, args)
}

struct Setup {
    _dir: tempfile::TempDir,
    _app: tauri::App<tauri::test::MockRuntime>,
    w: W,
    item: i64,
    session: i64,
}

fn setup() -> Setup {
    let a = psm_testkit::TestApp::new().unwrap();
    let (item, session) = a
        .items()
        .create_local("A12-1", "Teak sideboard", "Sideboard", None)
        .unwrap();
    let psm_testkit::TestApp { dir, app, .. } = a;
    let tauri_app = mock_builder()
        .manage(AppState::new(app))
        .invoke_handler(tauri::generate_handler![
            commands::search::search_pages,
            commands::search::search_page_save,
            commands::search::search_page_remove,
            commands::search::search_put_text,
            commands::attributes::form_get,
            commands::history::history_undo,
        ])
        .build(mock_context(noop_assets()))
        .unwrap();
    let w = tauri::WebviewWindowBuilder::new(&tauri_app, "search-ui-1", Default::default())
        .build()
        .unwrap();
    Setup {
        _dir: dir,
        _app: tauri_app,
        w,
        item: item.id,
        session: session.session.id,
    }
}

#[test]
fn a12_saved_pages_are_kept_per_item_until_removed() {
    let s = setup();
    let (w, item) = (&s.w, s.item);
    assert_eq!(
        call(w, "search_pages", json!({"itemId": item})).unwrap(),
        json!([])
    );
    let first = call(
        w,
        "search_page_save",
        json!({"itemId": item, "url": "https://e1.example/a", "title": "Teak sideboard – E1"}),
    )
    .unwrap();
    call(
        w,
        "search_page_save",
        json!({"itemId": item, "url": "https://e2.example/b", "title": "B"}),
    )
    .unwrap();
    // Saving the same address again keeps one row, in its place, with the newer title.
    call(
        w,
        "search_page_save",
        json!({"itemId": item, "url": "https://e1.example/a", "title": "Renamed"}),
    )
    .unwrap();
    let pages = call(w, "search_pages", json!({"itemId": item})).unwrap();
    let urls: Vec<&str> = pages
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["url"].as_str().unwrap())
        .collect();
    assert_eq!(urls, vec!["https://e1.example/a", "https://e2.example/b"]);
    assert_eq!(pages[0]["title"], "Renamed");
    // Only web pages.
    for bad in [
        "tauri://localhost/index.html",
        "file:///etc/hosts",
        "http://localhost:1420/",
    ] {
        assert!(
            call(
                w,
                "search_page_save",
                json!({"itemId": item, "url": bad, "title": ""})
            )
            .is_err(),
            "{bad}"
        );
    }
    call(w, "search_page_remove", json!({"pageId": first["id"]})).unwrap();
    let pages = call(w, "search_pages", json!({"itemId": item})).unwrap();
    assert_eq!(pages.as_array().unwrap().len(), 1);
    // Another item has its own list.
    assert_eq!(
        call(w, "search_pages", json!({"itemId": item + 1000})).unwrap(),
        json!([])
    );
}

#[test]
fn a12_put_text_is_a_persons_edit_with_one_undo_step() {
    let s = setup();
    let (w, item, session) = (&s.w, s.item, s.session);
    let stored = call(
        w,
        "search_put_text",
        json!({"sessionId": session, "key": "product.title", "text": "  Kai Kristiansen teak sideboard "}),
    )
    .unwrap();
    assert_eq!(stored["provenance"], "human");
    assert_eq!(
        stored["value"],
        json!(json!("Kai Kristiansen teak sideboard").to_string())
    );
    let undone = call(w, "history_undo", json!({"sessionId": session}));
    assert!(undone.is_ok(), "{undone:?}");
    let form = call(w, "form_get", json!({"itemId": item})).unwrap();
    let title = form["groups"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|g| g["fields"].as_array().unwrap().iter())
        .find(|f| f["definition"]["key"] == "product.title")
        .cloned()
        .unwrap();
    assert_ne!(
        title["value"]["value"], stored["value"],
        "undo took the edit back"
    );
    // Too long a selection, or nothing selected, is refused.
    let long = "x".repeat(psm_research::browse::MAX_SELECTION_CHARS + 1);
    assert!(
        call(
            w,
            "search_put_text",
            json!({"sessionId": session, "key": "product.title", "text": long})
        )
        .is_err()
    );
    assert!(
        call(
            w,
            "search_put_text",
            json!({"sessionId": session, "key": "product.title", "text": "  "})
        )
        .is_err()
    );
}

#[test]
fn a12_a_remote_page_cannot_call_the_windows_commands() {
    let s = setup();
    let (w, item, session) = (&s.w, s.item, s.session);
    for origin in ["https://www.google.com/search?q=x", "https://evil.example/"] {
        let refused = call_from(
            w,
            origin,
            "search_put_text",
            json!({"sessionId": session, "key": "product.title", "text": "pwned"}),
        );
        assert!(refused.is_err(), "{origin}");
        assert!(call_from(w, origin, "search_pages", json!({"itemId": item})).is_err());
    }
    // The app's own page can.
    assert!(call(w, "search_pages", json!({"itemId": item})).is_ok());
}

#[test]
fn a12_text_maps_into_the_fields_kind() {
    use commands::search::value_from_text;
    use psm_core::domain::attribute::{AttributeGroup, CoreType, Definition};
    let def = |core_type, choices: &[&str]| Definition {
        integration: "shop".into(),
        key: "k".into(),
        label: "Wood".into(),
        core_type,
        choices: choices.iter().map(|c| c.to_string()).collect(),
        validations: json!({}),
        group: AttributeGroup::Details,
        description: None,
        writable: true,
    };
    assert_eq!(
        value_from_text(&def(CoreType::Text, &[]), " Kai "),
        Ok(json!("Kai"))
    );
    assert_eq!(
        value_from_text(&def(CoreType::Choice, &["Oak", "Teak"]), "teak"),
        Ok(json!("Teak"))
    );
    assert!(value_from_text(&def(CoreType::Choice, &["Oak", "Teak"]), "walnut").is_err());
    assert_eq!(
        value_from_text(&def(CoreType::Integer, &[]), "1965"),
        Ok(json!(1965))
    );
    assert!(value_from_text(&def(CoreType::Integer, &[]), "circa 1965").is_err());
    assert!(value_from_text(&def(CoreType::Text, &[]), "  ").is_err());
}
