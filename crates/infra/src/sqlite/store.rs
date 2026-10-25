use crate::paths::AppPaths;
use psm_core::{
    Error, Result,
    domain::{
        editing::Job,
        item::{Item, ItemOrigin, NewItem},
        photo::{Classification, Integrity, NewPhoto, Photo, PhotoSource, UndoSlot},
        session::{PublishStep, Session, SessionDto, SessionListRecord},
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
    let published_at: Option<i64> = r.get(2)?;
    let published_seq: i64 = r.get(3)?;
    let change_seq: i64 = r.get(4)?;
    Ok(SessionDto {
        session: Session {
            id: r.get(0)?,
            item_id: r.get(1)?,
            published_at,
            published_seq,
            change_seq,
            last_activity_at: r.get(5)?,
            created_at: r.get(6)?,
            undo_slot: r.get(7)?,
        },
        dirty: published_at.is_some() && change_seq > published_seq,
        photo_count: r.get(8)?,
        unreviewed_count: r.get(9)?,
        main_count: r.get(10)?,
        condition_count: r.get(11)?,
    })
}

fn session(conn: &Connection, id: i64) -> Result<SessionDto> {
    conn.query_row("SELECT s.id,s.item_id,s.published_at,s.published_seq,s.change_seq,s.last_activity_at,s.created_at,s.undo_slot,COUNT(p.id),COALESCE(SUM(p.classification='unreviewed'),0),COALESCE(SUM(p.classification='main'),0),COALESCE(SUM(p.classification='condition'),0) FROM sessions s LEFT JOIN photos p ON p.session_id=s.id WHERE s.id=?1 GROUP BY s.id", [id], session_row)
        .optional().map_err(db)?.ok_or_else(|| Error::NotFound(format!("session {id}")))
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

fn job(conn: &Connection, id: i64) -> Result<Job> {
    conn.query_row(
        "SELECT id,kind,target,payload,status,attempts,next_attempt_at,error FROM jobs WHERE id=?1",
        [id],
        |r| {
            Ok(Job {
                id: r.get(0)?,
                kind: r.get(1)?,
                target: r.get(2)?,
                payload: r.get(3)?,
                status: r.get(4)?,
                attempts: r.get(5)?,
                next_attempt_at: r.get(6)?,
                error: r.get(7)?,
            })
        },
    )
    .optional()
    .map_err(db)?
    .ok_or_else(|| Error::NotFound(format!("job {id}")))
}

fn parse_step(step: &str) -> Result<PublishStep> {
    match step {
        "prepare" => Ok(PublishStep::Prepare),
        "upload" => Ok(PublishStep::Upload),
        "update" => Ok(PublishStep::Update),
        "describe" => Ok(PublishStep::Describe),
        "done" => Ok(PublishStep::Done),
        "failed" => Ok(PublishStep::Failed),
        _ => Err(Error::InvalidData(format!("invalid publish step: {step}"))),
    }
}

struct SqliteTx<'a> {
    tx: Transaction<'a>,
}

impl StoreTx for SqliteTx<'_> {
    fn enqueue_mirror_tag(&mut self, raw_path: &str, photo_id: i64, now_ms: i64) -> Result<()> {
        let payload = serde_json::json!({"photo_id": photo_id}).to_string();
        let queued: Option<i64> = self.tx.query_row("SELECT id FROM jobs WHERE kind='mirror_tag' AND target=?1 AND status='queued' ORDER BY id DESC LIMIT 1", [raw_path], |r| r.get(0)).optional().map_err(db)?;
        if let Some(id) = queued {
            self.tx.execute("UPDATE jobs SET payload=?1,next_attempt_at=?2,error=NULL,updated_at=?2 WHERE id=?3", params![payload,now_ms,id]).map_err(db)?;
        } else {
            self.tx.execute("INSERT INTO jobs(kind,target,payload,status,attempts,next_attempt_at,created_at,updated_at) VALUES ('mirror_tag',?1,?2,'queued',0,?3,?3,?3)",params![raw_path,payload,now_ms]).map_err(db)?;
        }
        Ok(())
    }
    fn session(&self, id: i64) -> Result<Session> {
        Ok(session(&self.tx, id)?.session)
    }
    fn photo(&self, id: i64) -> Result<Photo> {
        photo(&self.tx, id)
    }
    fn photos(&self, session_id: i64) -> Result<Vec<Photo>> {
        let mut stmt = self.tx.prepare("SELECT id,uid,session_id,raw_path,raw_hash,integrity,provider_ref,original_preview_path,adjusted_preview_path,orientation,classification,position,captured_at,source,created_at FROM photos WHERE session_id=?1 ORDER BY CASE classification WHEN 'main' THEN 0 WHEN 'condition' THEN 1 WHEN 'unreviewed' THEN 2 ELSE 3 END,position,captured_at,id").map_err(db)?;
        let rows = stmt.query_map([session_id], photo_row).map_err(db)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(db)
    }
    fn set_photo_review(
        &mut self,
        id: i64,
        session_id: i64,
        classification: Classification,
        position: Option<i64>,
    ) -> Result<()> {
        self.tx
            .execute(
                "UPDATE photos SET session_id=?1,classification=?2,position=?3 WHERE id=?4",
                params![session_id, classification.as_str(), position, id],
            )
            .map_err(db)?;
        Ok(())
    }
    fn set_positions(&mut self, positions: &[(i64, i64)]) -> Result<()> {
        for (id, position) in positions {
            self.tx
                .execute(
                    "UPDATE photos SET position=?1 WHERE id=?2",
                    params![position, id],
                )
                .map_err(db)?;
        }
        Ok(())
    }
    fn copy_photo(
        &mut self,
        photo: &Photo,
        target_session: i64,
        position: Option<i64>,
    ) -> Result<i64> {
        let uid = uuid::Uuid::new_v4().to_string();
        self.tx.execute("INSERT INTO photos(uid,session_id,raw_path,raw_hash,integrity,provider_ref,original_preview_path,adjusted_preview_path,orientation,classification,position,captured_at,source,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)", params![uid,target_session,photo.raw_path,photo.raw_hash,photo.integrity.as_str(),photo.provider_ref,photo.original_preview_path,photo.adjusted_preview_path,photo.orientation,photo.classification.as_str(),position,photo.captured_at,photo.source.as_str(),photo.created_at]).map_err(db)?;
        Ok(self.tx.last_insert_rowid())
    }
    fn set_undo_slot(&mut self, session_id: i64, slot: Option<&UndoSlot>) -> Result<()> {
        let json = slot
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        self.tx
            .execute(
                "UPDATE sessions SET undo_slot=?1 WHERE id=?2",
                params![json, session_id],
            )
            .map_err(db)?;
        Ok(())
    }
    fn touch_session(&mut self, session_id: i64, at: i64) -> Result<()> {
        self.tx
            .execute(
                "UPDATE sessions SET change_seq=change_seq+1,last_activity_at=?1 WHERE id=?2",
                params![at, session_id],
            )
            .map_err(db)?;
        Ok(())
    }
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
    fn active_session_id(&self) -> Result<Option<i64>> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .query_row(
                "SELECT value FROM settings WHERE key='active_session_id'",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(db)?
            .map(|v| {
                v.parse::<i64>()
                    .map_err(|e| Error::InvalidData(e.to_string()))
            })
            .transpose()
    }
    fn set_active_session_id(&self, id: Option<i64>) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        if let Some(id) = id {
            conn.execute("INSERT INTO settings(key,value) VALUES ('active_session_id',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [id.to_string()]).map_err(db)?;
        } else {
            conn.execute("DELETE FROM settings WHERE key='active_session_id'", [])
                .map_err(db)?;
        }
        Ok(())
    }
    fn job_due(&self, now_ms: i64) -> Result<Option<Job>> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(db)?;
        let id: Option<i64> = tx.query_row("SELECT id FROM jobs WHERE status='queued' AND next_attempt_at<=?1 ORDER BY next_attempt_at,id LIMIT 1", [now_ms], |r| r.get(0)).optional().map_err(db)?;
        let job = if let Some(id) = id {
            tx.execute(
                "UPDATE jobs SET status='running',updated_at=?1 WHERE id=?2",
                params![now_ms, id],
            )
            .map_err(db)?;
            Some(job(&tx, id)?)
        } else {
            None
        };
        tx.commit().map_err(db)?;
        Ok(job)
    }
    fn job_done(&self, id: i64, now_ms: i64) -> Result<()> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .execute(
                "UPDATE jobs SET status='done',error=NULL,updated_at=?1 WHERE id=?2",
                params![now_ms, id],
            )
            .map_err(db)?;
        Ok(())
    }
    fn job_retry(&self, id: i64, code: &str, next_attempt_at: i64, now_ms: i64) -> Result<()> {
        self.conn.lock().map_err(|e| Error::InvalidData(e.to_string()))?
            .execute("UPDATE jobs SET status='queued',attempts=attempts+1,error=?1,next_attempt_at=?2,updated_at=?3 WHERE id=?4",params![code,next_attempt_at,now_ms,id]).map_err(db)?;
        Ok(())
    }
    fn requeue_running_jobs(&self) -> Result<()> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .execute("UPDATE jobs SET status='queued' WHERE status='running'", [])
            .map_err(db)?;
        Ok(())
    }
    fn mark_own_write(&self, raw_path: &str, now_ms: i64) -> Result<()> {
        self.conn.lock().map_err(|e| Error::InvalidData(e.to_string()))?
            .execute("INSERT INTO raw_state(raw_path,last_own_write_at) VALUES (?1,?2) ON CONFLICT(raw_path) DO UPDATE SET last_own_write_at=excluded.last_own_write_at",params![raw_path,now_ms]).map_err(db)?;
        Ok(())
    }
    fn job(&self, id: i64) -> Result<Job> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        job(&conn, id)
    }
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
        session(&conn, id)
    }

    fn session_list_records(&self) -> Result<Vec<SessionListRecord>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT s.id,i.sku,i.item_type,i.title FROM sessions s JOIN items i ON i.id=s.item_id WHERE s.id<>1 ORDER BY s.id").map_err(db)?;
        let base = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(db)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db)?;
        base.into_iter().map(|(id,sku,item_type,title)| {
            let session = session(&conn, id)?;
            let non_deleted_missing_preview_count = conn.query_row("SELECT COUNT(*) FROM photos WHERE session_id=?1 AND classification<>'deleted' AND adjusted_preview_path IS NULL", [id], |r| r.get(0)).map_err(db)?;
            let cover_preview_path = conn.query_row("SELECT COALESCE(adjusted_preview_path,original_preview_path) FROM photos WHERE session_id=?1 AND classification='main' AND position=1", [id], |r| r.get(0)).optional().map_err(db)?.flatten();
            let latest: Option<(String,Option<String>,i64,i64)> = conn.query_row("SELECT pi.step,pi.failed_step,pi.uploaded,pi.total FROM publish_items pi JOIN publish_runs pr ON pr.id=pi.run_id WHERE pi.session_id=?1 AND pr.confirmed_at IS NOT NULL ORDER BY pr.confirmed_at DESC,pr.id DESC,pi.id DESC LIMIT 1", [id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(db)?;
            let (latest_step,failed_step,uploaded,total) = match latest {
                Some((step,failed,uploaded,total)) => (Some(parse_step(&step)?),failed.map(|s| parse_step(&s)).transpose()?,uploaded,total),
                None => (None,None,0,0),
            };
            Ok(SessionListRecord { session,sku,item_type,title,non_deleted_missing_preview_count,cover_preview_path,latest_step,failed_step,uploaded,total })
        }).collect()
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

    fn arrival_insert(&self, mut new: NewPhoto, path_key: &str) -> Result<(Photo, bool)> {
        new.raw_path = canon(std::path::Path::new(&new.raw_path))?;
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(db)?;
        let existing: Option<i64> = if new.source == PhotoSource::Capture {
            tx.query_row(
                "SELECT id FROM photos WHERE raw_path=?1 ORDER BY id LIMIT 1",
                [&new.raw_path],
                |r| r.get(0),
            )
            .optional()
            .map_err(db)?
        } else {
            tx.query_row(
                "SELECT id FROM photos WHERE session_id=?1 AND raw_path=?2",
                params![new.session_id, new.raw_path],
                |r| r.get(0),
            )
            .optional()
            .map_err(db)?
        };
        if let Some(id) = existing {
            return Ok((photo(&tx, id)?, false));
        }
        let shared_hash: Option<String> = tx.query_row("SELECT raw_hash FROM photos WHERE raw_path=?1 AND raw_hash IS NOT NULL ORDER BY id LIMIT 1", [&new.raw_path], |r| r.get(0)).optional().map_err(db)?;
        let hash_job_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM jobs WHERE kind='hash' AND target=?1)",
                [&new.raw_path],
                |r| r.get(0),
            )
            .map_err(db)?;
        let uid = uuid::Uuid::new_v4().to_string();
        tx.execute("INSERT INTO photos(uid,session_id,raw_path,raw_hash,source,captured_at,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)", params![uid,new.session_id,new.raw_path,shared_hash.clone().or(new.raw_hash),new.source.as_str(),new.captured_at,new.created_at]).map_err(db)?;
        let id = tx.last_insert_rowid();
        tx.execute("INSERT INTO raw_state(raw_path,path_key) VALUES (?1,?2) ON CONFLICT(raw_path) DO UPDATE SET path_key=excluded.path_key", params![new.raw_path,path_key]).map_err(db)?;
        for kind in ["original_preview", "hash", "render_preview"] {
            if kind == "hash" && (shared_hash.is_some() || hash_job_exists) {
                continue;
            }
            tx.execute("INSERT INTO jobs(kind,target,payload,status,attempts,next_attempt_at,created_at,updated_at) VALUES (?1,?2,'{}','queued',0,?3,?3,?3)", params![kind,new.raw_path,new.created_at]).map_err(db)?;
        }
        tx.execute(
            "UPDATE sessions SET change_seq=change_seq+1,last_activity_at=?1 WHERE id=?2",
            params![new.created_at, new.session_id],
        )
        .map_err(db)?;
        tx.commit().map_err(db)?;
        Ok((photo(&conn, id)?, true))
    }

    fn session_folders(&self) -> Result<Vec<(String, i64)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT i.folder_key,s.id FROM sessions s JOIN items i ON i.id=s.item_id ORDER BY s.id DESC").map_err(db)?;
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(db)?
            .collect::<rusqlite::Result<_>>()
            .map_err(db)
    }
    fn session_for_item(&self, item_id: i64) -> Result<Option<i64>> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .query_row(
                "SELECT id FROM sessions WHERE item_id=?1 ORDER BY id DESC LIMIT 1",
                [item_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db)
    }
    fn photos_in_session(&self, session_id: i64) -> Result<Vec<Photo>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT id,uid,session_id,raw_path,raw_hash,integrity,provider_ref,original_preview_path,adjusted_preview_path,orientation,classification,position,captured_at,source,created_at FROM photos WHERE session_id=?1 ORDER BY CASE classification WHEN 'main' THEN 0 WHEN 'condition' THEN 1 WHEN 'unreviewed' THEN 2 ELSE 3 END,position,captured_at,id").map_err(db)?;
        stmt.query_map([session_id], photo_row)
            .map_err(db)?
            .collect::<rusqlite::Result<_>>()
            .map_err(db)
    }
    fn record_arrival_event(
        &self,
        source: &str,
        payload: &str,
        at: i64,
        error: Option<&str>,
    ) -> Result<i64> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        conn.execute(
            "INSERT INTO capture_events(source,payload,received_at,error) VALUES (?1,?2,?3,?4)",
            params![source, payload, at, error],
        )
        .map_err(db)?;
        Ok(conn.last_insert_rowid())
    }
    fn set_capture_event_error(&self, id: i64, error: &str) -> Result<()> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .execute(
                "UPDATE capture_events SET error=?1 WHERE id=?2",
                params![error, id],
            )
            .map_err(db)?;
        Ok(())
    }
    fn set_raw_hash(&self, raw_path: &str, hash: &str) -> Result<()> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .execute(
                "UPDATE photos SET raw_hash=?1 WHERE raw_path=?2 AND raw_hash IS NULL",
                params![hash, raw_path],
            )
            .map_err(db)?;
        Ok(())
    }
    fn set_original_preview(
        &self,
        raw_path: &str,
        path: Option<&str>,
        orientation: Option<i64>,
    ) -> Result<()> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .execute(
                "UPDATE photos SET original_preview_path=?1,orientation=?2 WHERE raw_path=?3",
                params![path, orientation, raw_path],
            )
            .map_err(db)?;
        Ok(())
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
