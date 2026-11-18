//! "Delete session" in the database (owner correction A17): every row that names the session,
//! its item, its photos or the RAWs only it uses goes in one transaction, so nothing is left
//! pointing at a session that no longer exists. Shopify is never involved (the owner's scope).
//!
//! The research tables live in this database file but belong to the research engine
//! (`psm_research::schema`, never a numbered migration); they are cleared here when they exist,
//! by name. A new table that names a session, item, photo or research run must be added here
//! (`docs/guide/04_change_impact.md`); the suite's schema test fails until it is.

use psm_core::{Error, Result, domain::storage::DeletedRows};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

fn db(error: rusqlite::Error) -> Error {
    Error::InvalidData(error.to_string())
}

fn table_exists(conn: &Connection, name: &str) -> Result<bool> {
    conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1",
        [name],
        |_| Ok(()),
    )
    .optional()
    .map(|found| found.is_some())
    .map_err(db)
}

fn ids(conn: &Connection, sql: &str, id: i64) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare(sql).map_err(db)?;
    stmt.query_map([id], |r| r.get(0))
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<i64>>>()
        .map_err(db)
}

/// A JSON array of ids or texts, for `IN (SELECT value FROM json_each(?))`.
fn json<T: serde::Serialize>(values: &[T]) -> Result<String> {
    serde_json::to_string(values).map_err(|e| Error::InvalidData(e.to_string()))
}

pub fn research_running(conn: &Connection, session_id: i64) -> Result<bool> {
    if !table_exists(conn, "research_runs")? {
        return Ok(false);
    }
    conn.query_row(
        "SELECT 1 FROM research_runs WHERE session_id=?1 AND state='running' LIMIT 1",
        [session_id],
        |_| Ok(()),
    )
    .optional()
    .map(|found| found.is_some())
    .map_err(db)
}

pub fn delete_session_rows(tx: &Transaction<'_>, session_id: i64) -> Result<DeletedRows> {
    let item_id: Option<i64> = tx
        .query_row(
            "SELECT item_id FROM sessions WHERE id=?1",
            [session_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(db)?
        .ok_or_else(|| Error::NotFound(format!("session {session_id}")))?;
    let Some(item_id) = item_id else {
        return Err(Error::DeleteRefused(
            "The Unassigned session cannot be deleted.".into(),
        ));
    };
    // An item has one session (`SessionsService::create`); should another session ever name it,
    // the item and its rows stay.
    let item_shared: bool = tx
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE item_id=?1 AND id<>?2",
            params![item_id, session_id],
            |r| r.get::<_, i64>(0),
        )
        .map_err(db)?
        > 0;
    let photos = ids(tx, "SELECT id FROM photos WHERE session_id=?1", session_id)?;
    let photos_json = json(&photos)?;
    // RAWs no other session uses: their state and jobs go with the session.
    let own_raws: Vec<String> = {
        let mut stmt = tx
            .prepare(
                "SELECT DISTINCT p.raw_path FROM photos p WHERE p.session_id=?1 AND NOT EXISTS \
                 (SELECT 1 FROM photos o WHERE o.raw_path=p.raw_path AND o.session_id<>?1)",
            )
            .map_err(db)?;
        stmt.query_map([session_id], |r| r.get(0))
            .map_err(db)?
            .collect::<rusqlite::Result<_>>()
            .map_err(db)?
    };
    let raws_json = json(&own_raws)?;
    let publish_items = ids(
        tx,
        "SELECT id FROM publish_items WHERE session_id=?1",
        session_id,
    )?;
    let publish_json = json(&publish_items)?;

    let research = table_exists(tx, "research_runs")?;
    let runs = if research {
        ids(
            tx,
            "SELECT id FROM research_runs WHERE session_id=?1",
            session_id,
        )?
    } else {
        Vec::new()
    };
    let runs_json = json(&runs)?;

    // Jobs: the RAWs only this session uses, the session's export runs, its publish items, its
    // research runs, and any job whose payload names one of its photos.
    let session_target = format!("session:{session_id}");
    tx.execute(
        "DELETE FROM jobs WHERE target IN (SELECT value FROM json_each(?1)) \
         OR target=?2 \
         OR (kind='publish_step' AND target IN (SELECT CAST(value AS TEXT) FROM json_each(?3))) \
         OR (kind='research' AND target IN (SELECT CAST(value AS TEXT) FROM json_each(?4))) \
         OR (CASE WHEN json_valid(payload) THEN json_extract(payload,'$.photo_id') END) \
            IN (SELECT value FROM json_each(?5))",
        params![
            raws_json,
            session_target,
            publish_json,
            runs_json,
            photos_json
        ],
    )
    .map_err(db)?;

    if research {
        for table in [
            "research_steps",
            "research_calls",
            "research_outbound",
            "research_stage_outputs",
        ] {
            if table_exists(tx, table)? {
                tx.execute(
                    &format!(
                        "DELETE FROM {table} WHERE run_id IN (SELECT value FROM json_each(?1))"
                    ),
                    [&runs_json],
                )
                .map_err(db)?;
            }
        }
        if table_exists(tx, "research_stage_outputs")? {
            // Another run's output reused from a deleted run keeps its content, not the link.
            tx.execute(
                "UPDATE research_stage_outputs SET reused_from_run_id=NULL \
                 WHERE reused_from_run_id IN (SELECT value FROM json_each(?1))",
                [&runs_json],
            )
            .map_err(db)?;
        }
        tx.execute(
            "DELETE FROM research_runs WHERE id IN (SELECT value FROM json_each(?1))",
            [&runs_json],
        )
        .map_err(db)?;
    }
    if !item_shared && table_exists(tx, "research_feedback")? {
        tx.execute("DELETE FROM research_feedback WHERE item_id=?1", [item_id])
            .map_err(db)?;
    }

    // The photos and what hangs off them.
    for table in ["compositions", "photo_places", "photo_denials"] {
        tx.execute(
            &format!("DELETE FROM {table} WHERE photo_id IN (SELECT value FROM json_each(?1))"),
            [&photos_json],
        )
        .map_err(db)?;
    }
    tx.execute(
        "DELETE FROM photo_links WHERE photo_id IN (SELECT value FROM json_each(?1)) \
         OR (?3=0 AND item_id=?2)",
        params![photos_json, item_id, item_shared as i64],
    )
    .map_err(db)?;
    tx.execute(
        "DELETE FROM raw_state WHERE raw_path IN (SELECT value FROM json_each(?1))",
        [&raws_json],
    )
    .map_err(db)?;
    let photo_count = tx
        .execute("DELETE FROM photos WHERE session_id=?1", [session_id])
        .map_err(db)? as i64;

    // Publishing records of the session (Shopify itself is not touched).
    tx.execute(
        "DELETE FROM publish_item_stock WHERE publish_item_id IN (SELECT value FROM json_each(?1))",
        [&publish_json],
    )
    .map_err(db)?;
    for id in &publish_items {
        tx.execute(
            "DELETE FROM settings WHERE key=?1",
            [format!("publish_export:{id}")],
        )
        .map_err(db)?;
    }
    tx.execute(
        "DELETE FROM publish_items WHERE session_id=?1",
        [session_id],
    )
    .map_err(db)?;
    tx.execute(
        "DELETE FROM notifications WHERE session_id=?1",
        [session_id],
    )
    .map_err(db)?;
    tx.execute(
        "DELETE FROM session_changes WHERE session_id=?1",
        [session_id],
    )
    .map_err(db)?;
    // The capture ledger keeps its history; a delivery or exposure of this session now has none
    // (a late file is filed to Unassigned, never into a deleted session's folder).
    for table in ["camera_exposures", "camera_deliveries"] {
        tx.execute(
            &format!("UPDATE {table} SET session_id=NULL WHERE session_id=?1"),
            [session_id],
        )
        .map_err(db)?;
    }
    tx.execute(
        "DELETE FROM settings WHERE key='active_session_id' AND value=?1",
        [session_id.to_string()],
    )
    .map_err(db)?;

    tx.execute("DELETE FROM sessions WHERE id=?1", [session_id])
        .map_err(db)?;
    // The item and what hangs off it.
    let mut item = false;
    if !item_shared {
        for table in [
            "attribute_values",
            "item_links",
            "item_stock",
            "item_remote_titles",
            "item_retention",
            "archived_images",
            "saved_pages",
        ] {
            tx.execute(&format!("DELETE FROM {table} WHERE item_id=?1"), [item_id])
                .map_err(db)?;
        }
        item = tx
            .execute("DELETE FROM items WHERE id=?1", [item_id])
            .map_err(db)?
            > 0;
    }
    Ok(DeletedRows {
        photos: photo_count,
        item,
        research_runs: runs.len() as i64,
    })
}
