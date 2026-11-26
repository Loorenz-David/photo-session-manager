//! App-owned research columns, installed after the numbered migrations on every open.

use rusqlite::Connection;

fn has_column(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let names = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for name in names {
        if name? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn ensure_research_columns(conn: &Connection) -> rusqlite::Result<()> {
    for (table, column, definition) in [
        ("attribute_values", "ai_prior", "TEXT"),
        ("publish_items", "frozen_attribute_origin", "TEXT"),
        ("publish_items", "pending_conflicts", "TEXT"),
        ("publish_items", "resolved_attributes", "TEXT"),
        ("publish_items", "released_at", "INTEGER"),
        ("sessions", "arranged_at", "INTEGER"),
    ] {
        if !has_column(conn, table, column)? {
            conn.execute(
                &format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"),
                [],
            )?;
        }
    }
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS research_feedback (
        id INTEGER PRIMARY KEY,
        item_id INTEGER NOT NULL,
        key TEXT NOT NULL,
        suggested_value TEXT NOT NULL,
        confidence TEXT NOT NULL,
        final_value TEXT,
        action TEXT NOT NULL,
        at INTEGER NOT NULL,
        step_token TEXT
    );
    CREATE INDEX IF NOT EXISTS research_feedback_item_key ON research_feedback(item_id, key);",
    )?;
    Ok(())
}

pub fn ensure_live_columns(conn: &Connection) -> rusqlite::Result<()> {
    for table in ["camera_exposures", "photo_places"] {
        if !has_column(conn, table, "shot_bound")? {
            conn.execute(
                &format!("ALTER TABLE {table} ADD COLUMN shot_bound INTEGER NOT NULL DEFAULT 0"),
                [],
            )?;
        }
        if !has_column(conn, table, "shot_container_id")? {
            conn.execute(
                &format!("ALTER TABLE {table} ADD COLUMN shot_container_id INTEGER"),
                [],
            )?;
        }
    }
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS phone_pairings (
        id INTEGER PRIMARY KEY, name TEXT NOT NULL, token_sha256 TEXT NOT NULL UNIQUE,
        created_at INTEGER NOT NULL, last_seen_at INTEGER, revoked_at INTEGER
    )",
    )?;
    Ok(())
}
