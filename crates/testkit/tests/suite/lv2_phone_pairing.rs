//! LV2·U3 — pairing and the `phone` setting (`master_plan_live.md` §5 PH1, §4.5, §6 port rule):
//! rows LV2·C13–C14 at the wired TestApp.
use psm_core::Error;
use psm_integrations::{
    PreparedSettings,
    phone::pairing::{PhonePairings, token_sha256},
};
use psm_testkit::TestApp;
use serde_json::json;
use sha2::{Digest, Sha256};

fn app() -> TestApp {
    TestApp::new().unwrap()
}
fn pairings(a: &TestApp) -> PhonePairings<'_> {
    PhonePairings::new(a.store(), a.clock.as_ref())
}
fn db(a: &TestApp) -> rusqlite::Connection {
    rusqlite::Connection::open(a.app.paths.db()).unwrap()
}
/// Every text value of every table of the database.
fn every_text(a: &TestApp) -> Vec<String> {
    let conn = db(a);
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    let mut out = Vec::new();
    for table in tables {
        let mut stmt = conn.prepare(&format!("SELECT * FROM \"{table}\"")).unwrap();
        let columns = stmt.column_count();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            for i in 0..columns {
                if let Ok(rusqlite::types::ValueRef::Text(t)) = row.get_ref(i) {
                    out.push(String::from_utf8_lossy(t).into_owned());
                }
            }
        }
    }
    out
}
fn settings_file(a: &TestApp) -> String {
    std::fs::read_to_string(a.app.paths.settings()).unwrap()
}

// ---- LV2·C13: pairings ----

#[test]
fn lv2_c13a_a_pairing_token_is_shown_once_and_stored_only_as_its_hash() {
    let a = app();
    let created = pairings(&a).create(None).unwrap();
    assert_eq!(created.token.len(), 43);
    assert!(
        created
            .token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    );
    let hex: String = Sha256::digest(created.token.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let (name, stored): (String, String) = db(&a)
        .query_row(
            "SELECT name,token_sha256 FROM phone_pairings WHERE id=?1",
            [created.id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((name.as_str(), stored.as_str()), ("Phone 1", hex.as_str()));
    assert_eq!(token_sha256(&created.token), hex);
    assert!(
        !every_text(&a).iter().any(|t| t.contains(&created.token)),
        "the token is in the database"
    );
}

#[test]
fn lv2_c13b_verify_finds_the_live_pairing_and_marks_it_seen() {
    let a = app();
    let created = pairings(&a).create(None).unwrap();
    a.clock.advance(5_000);
    let found = pairings(&a).verify(&created.token).unwrap().unwrap();
    assert_eq!(
        (found.id, found.name.as_str()),
        (created.id, created.name.as_str())
    );
    assert!(found.last_seen_at.is_some());
    assert_eq!(pairings(&a).verify("x").unwrap(), None);
}

#[test]
fn lv2_c13c_revoking_one_pairing_leaves_the_other() {
    let a = app();
    let first = pairings(&a).create(None).unwrap();
    let second = pairings(&a).create(None).unwrap();
    pairings(&a).revoke(first.id).unwrap();
    assert_eq!(pairings(&a).verify(&first.token).unwrap(), None);
    assert_eq!(
        pairings(&a).verify(&second.token).unwrap().map(|p| p.id),
        Some(second.id)
    );
}

#[test]
fn lv2_c13d_create_rename_and_list_without_tokens() {
    let a = app();
    let first = pairings(&a).create(None).unwrap();
    let second = pairings(&a).create(None).unwrap();
    pairings(&a).rename(second.id, "Android").unwrap();
    let list = pairings(&a).list().unwrap();
    assert_eq!(
        list.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        vec!["Phone 1", "Android"]
    );
    let shown = serde_json::to_string(&list).unwrap();
    assert!(!shown.contains(&first.token) && !shown.contains(&second.token));
}

// ---- LV2·C14: the `phone` setting ----

#[test]
fn lv2_c14a_a_port_change_is_saved_without_rebuilding_the_app() {
    let a = app();
    let prepared = a
        .app
        .settings_set_context()
        .prepare(&json!({"phone": {"port": 8790}}))
        .unwrap();
    assert!(matches!(prepared, PreparedSettings::Simple(_)));
    let saved: serde_json::Value = serde_json::from_str(&settings_file(&a)).unwrap();
    assert_eq!(saved["phone"], json!({"enabled": true, "port": 8790}));
}

#[test]
fn lv2_c14b_ports_below_1024_or_used_by_the_camera_receiver_are_refused() {
    let mut a = app();
    let before = settings_file(&a);
    for port in [2121, 80, 50005] {
        let refused = a.settings_set(&json!({"phone": {"port": port}}));
        assert!(
            matches!(refused, Err(Error::PhonePortInvalid)),
            "{port}: {refused:?}"
        );
        assert_eq!(
            Error::PhonePortInvalid.to_string(),
            "phone_port_invalid",
            "{port}"
        );
    }
    assert_eq!(settings_file(&a), before);
}

#[test]
fn lv2_c14c_a_fresh_settings_file_has_the_phone_server_on_at_8686() {
    let a = app();
    assert_eq!(
        a.app.settings_get().unwrap()["phone"],
        json!({"enabled": true, "port": 8686})
    );
}
