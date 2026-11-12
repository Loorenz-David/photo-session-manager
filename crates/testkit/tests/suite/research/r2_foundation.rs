use std::sync::Arc;

use psm_core::{
    Error,
    domain::{
        attribute::{Provenance, Value},
        session::{ListFilter, PublishStep, RowLabel, SessionStatus},
    },
    mechanisms::status::{StatusInput, derive_status, row_label},
    ports::{commerce::CommerceAdapter, store::StoreTx},
};
use psm_infra::{paths::AppPaths, sqlite::store::SqliteStore};
use psm_integrations::commerce::shopify::adapter::{PollConfig, ShopifyAdapter, ShopifyConfig};
use psm_testkit::{TestApp, fake_transport::FakeTransport};
use rusqlite::{Connection, params};
use rusqlite_migration::Migrations;

#[test]
fn a07_c7a_schema_step_preserves_rows_and_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    let before = SqliteStore::migrations().len();
    let mut conn = Connection::open(paths.db()).unwrap();
    Migrations::new(SqliteStore::migrations())
        .to_latest(&mut conn)
        .unwrap();
    conn.execute("INSERT INTO items(id,sku,folder_key,folder_key_ci,item_type,title,origin,created_at,updated_at) VALUES (1,'R2','R2','r2','object','Test','local',0,0)", []).unwrap();
    conn.execute("INSERT INTO attribute_values(item_id,key,value,valid,provenance,updated_at) VALUES (1,'x','\"Oak\"',1,'human',1)", []).unwrap();
    drop(conn);
    for _ in 0..2 {
        let store = SqliteStore::open(&paths).unwrap();
        drop(store);
        let conn = Connection::open(paths.db()).unwrap();
        let row: (String, Option<String>) = conn
            .query_row(
                "SELECT value,ai_prior FROM attribute_values WHERE item_id=1 AND key='x'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(row, ("\"Oak\"".into(), None));
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM research_feedback", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(publish_items)")
            .unwrap()
            .query_map([], |r| r.get(1))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        for name in [
            "frozen_attribute_origin",
            "pending_conflicts",
            "resolved_attributes",
            "released_at",
        ] {
            assert!(columns.iter().any(|c| c == name), "{name}");
        }
    }
    assert_eq!(SqliteStore::migrations().len(), before);
}

#[test]
fn a09_c5a_held_status_and_unresolved_label() {
    let input = StatusInput {
        latest_step: Some(PublishStep::Held),
        failed_step: None,
        published_at: None,
        dirty: false,
        main_count: 1,
        unreviewed_count: 0,
        non_deleted_missing_preview_count: 0,
        uploaded: 0,
        total: 1,
        unresolved_conflicts: 1,
    };
    assert_eq!(derive_status(input), SessionStatus::Publishing);
    assert_eq!(row_label(input), RowLabel::WaitingForYou(1));
}

#[test]
fn a09_c5b_held_session_is_publishing_in_list() {
    let a = TestApp::new().unwrap();
    let (_, created) = a
        .items()
        .create_local("R2-HELD", "Held", "object", None)
        .unwrap();
    let session = created.session.id;
    a.insert_publish_item(session, "held", 100, None).unwrap();
    let conn = Connection::open(a.app.paths.db()).unwrap();
    conn.execute(
        "UPDATE publish_items SET pending_conflicts=?1,resolved_attributes=?2 WHERE session_id=?3",
        params![r#"{"a":{},"b":{}}"#, r#"{"a":{}}"#, session],
    )
    .unwrap();
    let row = a
        .sessions()
        .list(ListFilter::All, None)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .find(|r| r.id == session)
        .unwrap();
    assert_eq!(row.status, SessionStatus::Publishing);
    assert_eq!(row.label, RowLabel::WaitingForYou(1));
}

#[test]
fn f_c2a_research_transaction_commits_and_rolls_back_both_views() {
    let a = TestApp::new().unwrap();
    let (item, _) = a
        .items()
        .create_local("R2-TX", "Transaction", "object", None)
        .unwrap();
    let value = Value {
        item_id: item.id,
        key: "x".into(),
        value: "\"first\"".into(),
        valid: true,
        provenance: Provenance::Human,
        ai_confidence: None,
        ai_status: None,
        ai_reason: None,
        ai_sources: None,
        ai_prior: None,
        updated_at: 1,
    };
    a.app
        .store
        .research_transaction(|tx| {
            tx.conn()
                .execute("CREATE TABLE scratch (value TEXT)", [])
                .map_err(|e| Error::InvalidData(e.to_string()))?;
            tx.conn()
                .execute("INSERT INTO scratch(value) VALUES ('first')", [])
                .map_err(|e| Error::InvalidData(e.to_string()))?;
            tx.put_attribute_value(&value)?;
            Ok(())
        })
        .unwrap();
    let rejected: psm_core::Result<()> = a.app.store.research_transaction(|tx| {
        tx.conn()
            .execute("INSERT INTO scratch(value) VALUES ('second')", [])
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        tx.put_attribute_value(&Value {
            value: "\"second\"".into(),
            ..value.clone()
        })?;
        Err(Error::InvalidValue("abort".into()))
    });
    assert!(rejected.is_err());
    let conn = Connection::open(a.app.paths.db()).unwrap();
    let scratch: Vec<String> = conn
        .prepare("SELECT value FROM scratch")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(scratch, vec!["first"]);
    let stored = a.store().transaction(&mut |tx| {
        assert_eq!(tx.attribute_values(item.id)?[0].value, "\"first\"");
        Ok(())
    });
    stored.unwrap();
}

#[test]
fn f_c3a_public_listing_default_and_fake_values() {
    let a = TestApp::new().unwrap();
    let adapter = ShopifyAdapter::new(
        ShopifyConfig {
            shop_domain: "example.myshopify.com".into(),
            api_version: "2026-01".into(),
            client_id: String::new(),
            client_secret: String::new(),
            condition_photos_key: "custom.damage".into(),
        },
        Arc::new(FakeTransport::new()),
        a.clock.clone(),
        PollConfig::default(),
    );
    let empty = adapter.public_listing("remote-id").unwrap();
    assert!(empty.hosts.is_empty());
    assert_eq!(empty.item_url, None);
    assert_eq!(empty.item_path_tail, "");
    let fake = a.commerce.unwrap_or_else(|| {
        Arc::new(psm_testkit::fake_commerce::FakeCommerce::new(
            a.clock.clone(),
        ))
    });
    fake.set_public_listing(
        vec!["shop.example".into()],
        Some("https://shop.example/item/1".into()),
        "/item/1".into(),
    );
    let listed = fake.public_listing("remote-id").unwrap();
    assert_eq!(listed.hosts, vec!["shop.example"]);
    assert_eq!(
        listed.item_url.as_deref(),
        Some("https://shop.example/item/1")
    );
    assert_eq!(listed.item_path_tail, "/item/1");
}
