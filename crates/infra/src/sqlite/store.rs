use crate::paths::AppPaths;
use psm_core::{
    Error, Result,
    domain::{
        item::{Item, ItemOrigin, NewItem},
        photo::{Classification, Integrity, NewPhoto, Photo, PhotoSource},
        session::{Session, SessionDto},
    },
    mechanisms::canon::canon,
    ports::store::{Store, StoreTx},
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use rusqlite_migration::{M, Migrations};
use std::sync::Mutex;

pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    pub fn open(paths: &AppPaths) -> rusqlite_migration::Result<Self> {
        let mut conn = Connection::open(paths.db())?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        Migrations::new(vec![
            M::up(include_str!("migrations/001_init.sql")),
            M::up(include_str!("migrations/002_domain.sql")),
        ])
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

fn db(error: rusqlite::Error) -> Error {
    Error::InvalidData(error.to_string())
}

fn item_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Item> {
    let origin: String = r.get(6)?;
    Ok(Item {
        id: r.get(0)?,
        sku: r.get(1)?,
        folder_key: r.get(2)?,
        barcode: r.get(3)?,
        item_type: r.get(4)?,
        title: r.get(5)?,
        origin: if origin == "linked" {
            ItemOrigin::Linked
        } else {
            ItemOrigin::Local
        },
        created_at: r.get(7)?,
        updated_at: r.get(8)?,
    })
}

fn item(conn: &Connection, id: i64) -> Result<Item> {
    conn.query_row("SELECT id,sku,folder_key,barcode,item_type,title,origin,created_at,updated_at FROM items WHERE id=?1", [id], item_row)
        .optional().map_err(db)?.ok_or_else(|| Error::NotFound(format!("item {id}")))
}

fn session_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<SessionDto> {
    Ok(SessionDto {
        session: Session {
            id: r.get(0)?,
            item_id: r.get(1)?,
            published_at: r.get(2)?,
            published_seq: r.get(3)?,
            change_seq: r.get(4)?,
            last_activity_at: r.get(5)?,
            created_at: r.get(6)?,
            undo_slot: r.get(7)?,
        },
        photo_count: r.get(8)?,
        unreviewed_count: r.get(9)?,
        main_count: r.get(10)?,
        condition_count: r.get(11)?,
    })
}

fn photo_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Photo> {
    let integrity: String = r.get(5)?;
    let classification: String = r.get(10)?;
    let source: String = r.get(13)?;
    Ok(Photo {
        id: r.get(0)?,
        uid: r.get(1)?,
        session_id: r.get(2)?,
        raw_path: r.get(3)?,
        raw_hash: r.get(4)?,
        integrity: match integrity.as_str() {
            "raw_changed" => Integrity::RawChanged,
            "raw_missing" => Integrity::RawMissing,
            _ => Integrity::Ok,
        },
        provider_ref: r.get(6)?,
        original_preview_path: r.get(7)?,
        adjusted_preview_path: r.get(8)?,
        orientation: r.get(9)?,
        classification: match classification.as_str() {
            "main" => Classification::Main,
            "condition" => Classification::Condition,
            "deleted" => Classification::Deleted,
            _ => Classification::Unreviewed,
        },
        position: r.get(11)?,
        captured_at: r.get(12)?,
        source: if source == "import" {
            PhotoSource::Import
        } else {
            PhotoSource::Capture
        },
        created_at: r.get(14)?,
    })
}

fn photo(conn: &Connection, id: i64) -> Result<Photo> {
    conn.query_row("SELECT id,uid,session_id,raw_path,raw_hash,integrity,provider_ref,original_preview_path,adjusted_preview_path,orientation,classification,position,captured_at,source,created_at FROM photos WHERE id=?1", [id], photo_row)
        .optional().map_err(db)?.ok_or_else(|| Error::NotFound(format!("photo {id}")))
}

struct SqliteTx<'a> {
    tx: Transaction<'a>,
}

impl StoreTx for SqliteTx<'_> {
    fn items(&self) -> Result<Vec<Item>> {
        let mut stmt = self.tx.prepare("SELECT id,sku,folder_key,barcode,item_type,title,origin,created_at,updated_at FROM items ORDER BY id").map_err(db)?;
        let rows = stmt.query_map([], item_row).map_err(db)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(db)
    }
    fn item(&self, id: i64) -> Result<Item> {
        item(&self.tx, id)
    }
    fn insert_item(&mut self, item: NewItem<'_>) -> Result<i64> {
        self.tx.execute("INSERT INTO items(sku,folder_key,folder_key_ci,barcode,item_type,title,origin,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?8)", params![item.sku,item.folder_key,item.folder_key.to_ascii_lowercase(),item.barcode,item.item_type,item.title,item.origin,item.at]).map_err(db)?;
        Ok(self.tx.last_insert_rowid())
    }
    fn update_item_identity(
        &mut self,
        id: i64,
        sku: &str,
        folder_key: &str,
        barcode: Option<&str>,
        at: i64,
    ) -> Result<()> {
        self.tx.execute("UPDATE items SET sku=?1,folder_key=?2,folder_key_ci=?3,barcode=?4,updated_at=?5 WHERE id=?6", params![sku,folder_key,folder_key.to_ascii_lowercase(),barcode,at,id]).map_err(db)?;
        Ok(())
    }
    fn item_has_photos(&self, item_id: i64) -> Result<bool> {
        self.tx.query_row("SELECT EXISTS(SELECT 1 FROM photos JOIN sessions ON sessions.id=photos.session_id WHERE sessions.item_id=?1)", [item_id], |r| r.get(0)).map_err(db)
    }
    fn insert_session(&mut self, item_id: i64, at: i64) -> Result<i64> {
        self.tx
            .execute(
                "INSERT INTO sessions(item_id,last_activity_at,created_at) VALUES (?1,?2,?2)",
                params![item_id, at],
            )
            .map_err(db)?;
        Ok(self.tx.last_insert_rowid())
    }
    fn insert_item_link(
        &mut self,
        item_id: i64,
        integration: &str,
        external_id: &str,
        external_url: Option<&str>,
    ) -> Result<()> {
        self.tx.execute("INSERT INTO item_links(item_id,integration,external_id,external_url,create_pending) VALUES (?1,?2,?3,?4,0) ON CONFLICT(item_id,integration) DO UPDATE SET external_id=excluded.external_id,external_url=excluded.external_url,create_pending=0", params![item_id,integration,external_id,external_url]).map_err(db)?;
        Ok(())
    }
}

impl Store for SqliteStore {
    fn transaction(&self, op: &mut dyn FnMut(&mut dyn StoreTx) -> Result<()>) -> Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(db)?;
        let mut store_tx = SqliteTx { tx };
        op(&mut store_tx)?;
        store_tx.tx.commit().map_err(db)
    }

    fn item(&self, id: i64) -> Result<Item> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        item(&conn, id)
    }

    fn session(&self, id: i64) -> Result<SessionDto> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        conn.query_row("SELECT s.id,s.item_id,s.published_at,s.published_seq,s.change_seq,s.last_activity_at,s.created_at,s.undo_slot,COUNT(p.id),COALESCE(SUM(p.classification='unreviewed'),0),COALESCE(SUM(p.classification='main'),0),COALESCE(SUM(p.classification='condition'),0) FROM sessions s LEFT JOIN photos p ON p.session_id=s.id WHERE s.id=?1 GROUP BY s.id", [id], session_row)
            .optional().map_err(db)?.ok_or_else(|| Error::NotFound(format!("session {id}")))
    }

    fn photo_insert_or_get(&self, mut new: NewPhoto) -> Result<Photo> {
        new.raw_path = canon(std::path::Path::new(&new.raw_path))?;
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(db)?;
        let old: Option<i64> = tx
            .query_row(
                "SELECT id FROM photos WHERE session_id=?1 AND raw_path=?2",
                params![new.session_id, new.raw_path],
                |r| r.get(0),
            )
            .optional()
            .map_err(db)?;
        if let Some(id) = old {
            return photo(&tx, id);
        }
        let shared_hash: Option<String> = tx.query_row("SELECT raw_hash FROM photos WHERE raw_path=?1 AND raw_hash IS NOT NULL ORDER BY id LIMIT 1", [&new.raw_path], |r| r.get(0)).optional().map_err(db)?;
        let uid = uuid::Uuid::new_v4().to_string();
        tx.execute("INSERT INTO photos(uid,session_id,raw_path,raw_hash,source,captured_at,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)", params![uid,new.session_id,new.raw_path,shared_hash.or(new.raw_hash),new.source.as_str(),new.captured_at,new.created_at]).map_err(db)?;
        let id = tx.last_insert_rowid();
        tx.execute(
            "UPDATE sessions SET change_seq=change_seq+1,last_activity_at=?1 WHERE id=?2",
            params![new.created_at, new.session_id],
        )
        .map_err(db)?;
        tx.commit().map_err(db)?;
        photo(&conn, id)
    }

    fn photo(&self, id: i64) -> Result<Photo> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        photo(&conn, id)
    }

    fn set_photo_integrity(&self, id: i64, integrity: Integrity) -> Result<()> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .execute(
                "UPDATE photos SET integrity=?1 WHERE id=?2",
                params![integrity.as_str(), id],
            )
            .map_err(db)?;
        Ok(())
    }

    fn upsert_raw_state(&self, raw_path: &str, path_key: Option<&str>) -> Result<()> {
        let raw_path = canon(std::path::Path::new(raw_path))?;
        self.conn.lock().map_err(|e| Error::InvalidData(e.to_string()))?
            .execute("INSERT INTO raw_state(raw_path,path_key) VALUES (?1,?2) ON CONFLICT(raw_path) DO UPDATE SET path_key=excluded.path_key", params![raw_path,path_key]).map_err(db)?;
        Ok(())
    }

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
