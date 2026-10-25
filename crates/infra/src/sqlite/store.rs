use std::sync::Mutex;
use rusqlite::{params, Connection};
use rusqlite_migration::{Migrations, M};
use psm_core::{Error, Result, ports::store::Store};
use crate::paths::AppPaths;

pub struct SqliteStore { conn: Mutex<Connection> }

impl SqliteStore {
    pub fn open(paths: &AppPaths) -> rusqlite_migration::Result<Self> {
        let mut conn = Connection::open(paths.db())?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        Migrations::new(vec![M::up(include_str!("migrations/001_init.sql"))]).to_latest(&mut conn)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn event_rows(&self) -> Result<Vec<(String, String, String)>> {
        let conn = self.conn.lock().map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT source, hook, payload FROM capture_events ORDER BY id").map_err(|e| Error::InvalidData(e.to_string()))?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).map_err(|e| Error::InvalidData(e.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| Error::InvalidData(e.to_string()))
    }

    pub fn migration_count(&self) -> Result<i64> {
        self.conn.lock().map_err(|e| Error::InvalidData(e.to_string()))?
            .query_row("SELECT COUNT(*) FROM __rusqlite_migrations", [], |r| r.get(0))
            .map_err(|e| Error::InvalidData(e.to_string()))
    }
}

impl Store for SqliteStore {
    fn mechanism_version(&self) -> Result<String> {
        self.conn.lock().map_err(|e| Error::InvalidData(e.to_string()))?
            .query_row("SELECT value FROM settings WHERE key='mechanisms_version'", [], |r| r.get(0))
            .map_err(|e| Error::InvalidData(e.to_string()))
    }

    fn record_hook(&self, hook: &str, payload: &str, received_at: i64) -> Result<()> {
        let mut conn = self.conn.lock().map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(|e| Error::InvalidData(e.to_string()))?;
        tx.execute("INSERT INTO capture_events(source, hook, payload, received_at) VALUES ('hook', ?1, ?2, ?3)", params![hook, payload, received_at]).map_err(|e| Error::InvalidData(e.to_string()))?;
        tx.commit().map_err(|e| Error::InvalidData(e.to_string()))?;
        Ok(())
    }
}
