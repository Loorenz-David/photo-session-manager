use crate::paths::AppPaths;
use psm_core::{Error, Result, ports::store::Store};
use rusqlite::{Connection, params};
use rusqlite_migration::{M, Migrations};
use std::sync::Mutex;

pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    pub fn open(paths: &AppPaths) -> rusqlite_migration::Result<Self> {
        let mut conn = Connection::open(paths.db())?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        Migrations::new(vec![M::up(include_str!("migrations/001_init.sql"))])
            .to_latest(&mut conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn event_rows(&self) -> Result<Vec<(String, String, String)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT source, hook, payload FROM capture_events ORDER BY id")
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let rows = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| Error::InvalidData(e.to_string()))
    }

    pub fn migration_version(&self) -> Result<i64> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(|e| Error::InvalidData(e.to_string()))
    }
}

impl Store for SqliteStore {
    fn mechanism_version(&self) -> Result<String> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .query_row(
                "SELECT value FROM settings WHERE key='mechanisms_version'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| Error::InvalidData(e.to_string()))
    }

    fn record_hook(
        &self,
        spool_name: &str,
        hook: &str,
        payload: &str,
        received_at: i64,
    ) -> Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let inserted = tx
            .execute(
                "INSERT OR IGNORE INTO settings(key, value) VALUES (?1, 'processed')",
                params![format!("spool:{spool_name}")],
            )
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        if inserted == 1 {
            tx.execute("INSERT INTO capture_events(source, hook, payload, received_at) VALUES ('hook', ?1, ?2, ?3)", params![hook, payload, received_at]).map_err(|e| Error::InvalidData(e.to_string()))?;
        }
        tx.commit().map_err(|e| Error::InvalidData(e.to_string()))?;
        Ok(())
    }
}
