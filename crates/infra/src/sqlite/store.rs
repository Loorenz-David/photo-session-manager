use crate::paths::AppPaths;
use psm_core::{
    Error, Result,
    domain::{
        attribute::{
            AiConfidence, AiStatus, AttributeGroup, CoreType, Definition, Preset, Provenance,
            Value as AttributeValue,
        },
        editing::Job,
        item::{Item, ItemOrigin, NewItem},
        photo::{Classification, Integrity, NewPhoto, Photo, PhotoSource, UndoSlot},
        publish::{DefinitionLink, Notification, PhotoLink, PublishItem},
        session::{PublishStep, Session, SessionDto, SessionListRecord},
    },
    mechanisms::canon::canon,
    ports::store::{PreviewCommit, RawPreviewState, Store, StoreTx},
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
        external_id: None,
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

fn raw_preview_state(conn: &Connection, raw: &str) -> Result<Option<RawPreviewState>> {
    conn.query_row("SELECT r.raw_path,r.adjusted_gen,r.rendered_gen,r.retouch_gen,r.last_own_write_at,(SELECT adjusted_preview_path FROM photos WHERE raw_path=r.raw_path ORDER BY id LIMIT 1) FROM raw_state r WHERE r.raw_path=?1 AND EXISTS(SELECT 1 FROM photos WHERE raw_path=r.raw_path)", [raw], |r| Ok(RawPreviewState { raw_path:r.get(0)?, adjusted_gen:r.get(1)?, rendered_gen:r.get(2)?, retouch_gen:r.get(3)?, last_own_write_at:r.get(4)?, adjusted_preview_path:r.get(5)? })).optional().map_err(db)
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

fn step_name(step: PublishStep) -> &'static str {
    match step {
        PublishStep::Prepare => "prepare",
        PublishStep::Upload => "upload",
        PublishStep::Update => "update",
        PublishStep::Describe => "describe",
        PublishStep::Done => "done",
        PublishStep::Failed => "failed",
    }
}

type PublishItemRow = (
    i64,
    i64,
    i64,
    bool,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<i64>,
    String,
    Option<String>,
    i64,
    i64,
    Option<String>,
    Option<i64>,
);
fn publish_item(conn: &Connection, id: i64) -> Result<PublishItem> {
    let row: PublishItemRow = conn.query_row(
        "SELECT id,run_id,session_id,create_item,title,frozen_main,frozen_condition,frozen_attributes,announced_removals,confirmed_change_seq,step,failed_step,uploaded,total,error,(SELECT CAST(value AS INTEGER) FROM settings WHERE key='publish_export:'||publish_items.id) FROM publish_items WHERE id=?1", [id],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?,r.get(9)?,r.get(10)?,r.get(11)?,r.get(12)?,r.get(13)?,r.get(14)?,r.get(15)?))
    ).optional().map_err(db)?.ok_or_else(|| Error::NotFound(format!("publish item {id}")))?;
    Ok(PublishItem {
        id: row.0,
        run_id: row.1,
        session_id: row.2,
        create_item: row.3,
        title: row.4.unwrap_or_default(),
        frozen_main: from_json(row.5.as_deref().unwrap_or("[]"))?,
        frozen_condition: from_json(row.6.as_deref().unwrap_or("[]"))?,
        frozen_attributes: from_json(row.7.as_deref().unwrap_or("{}"))?,
        announced_removals: from_json(row.8.as_deref().unwrap_or("[]"))?,
        confirmed_change_seq: row.9.unwrap_or_default(),
        step: parse_step(&row.10)?,
        failed_step: row.11.as_deref().map(parse_step).transpose()?,
        uploaded: row.12,
        total: row.13,
        error: row.14,
        export_run_id: row.15,
    })
}

type PhotoLinkRow = (
    i64,
    i64,
    String,
    Option<String>,
    String,
    Option<String>,
    Option<String>,
);
fn photo_link_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<PhotoLinkRow> {
    Ok((
        r.get(0)?,
        r.get(1)?,
        r.get(2)?,
        r.get(3)?,
        r.get(4)?,
        r.get(5)?,
        r.get(6)?,
    ))
}
fn map_photo_link(row: PhotoLinkRow) -> Result<PhotoLink> {
    Ok(PhotoLink {
        photo_id: row.0,
        item_id: row.1,
        role: row.2,
        external_id: row.3,
        upload_state: row.4,
        export_sha256: row.5,
        superseded_ids: from_json(row.6.as_deref().unwrap_or("[]"))?,
    })
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
    fn update_item_title(&mut self, id: i64, title: &str, at: i64) -> Result<()> {
        self.tx
            .execute(
                "UPDATE items SET title=?1,updated_at=?2 WHERE id=?3",
                params![title, at, id],
            )
            .map_err(db)?;
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
    fn replace_attribute_definition_links(
        &mut self,
        integration: &str,
        links: &[(String, DefinitionLink)],
    ) -> Result<()> {
        self.tx
            .execute(
                "DELETE FROM attribute_definition_links WHERE integration=?1",
                [integration],
            )
            .map_err(db)?;
        for (key, link) in links {
            self.tx.execute("INSERT INTO attribute_definition_links(integration,key,platform_type,platform_meta) VALUES (?1,?2,?3,?4)",params![integration,key,link.platform_type,to_json(&link.platform_meta)?]).map_err(db)?;
        }
        Ok(())
    }
    fn insert_publish_run(&mut self, integration: &str, at: i64, summary: &str) -> Result<i64> {
        self.tx.execute("INSERT INTO publish_runs(integration,created_at,confirmed_at,confirmed_summary) VALUES (?1,?2,?2,?3)",params![integration,at,summary]).map_err(db)?;
        Ok(self.tx.last_insert_rowid())
    }
    fn insert_publish_item(&mut self, item: &PublishItem) -> Result<i64> {
        self.tx.execute("INSERT INTO publish_items(run_id,session_id,create_item,title,frozen_main,frozen_condition,frozen_attributes,announced_removals,confirmed_change_seq,step,uploaded,total) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,0,?11)",
            params![item.run_id,item.session_id,item.create_item,item.title,to_json(&item.frozen_main)?,to_json(&item.frozen_condition)?,to_json(&item.frozen_attributes)?,to_json(&item.announced_removals)?,item.confirmed_change_seq,step_name(item.step),item.total]).map_err(db)?;
        Ok(self.tx.last_insert_rowid())
    }
    fn attribute_definitions(&self, integration: &str) -> Result<Vec<Definition>> {
        let mut stmt = self.tx.prepare("SELECT integration,key,label,core_type,choices,validations,grp,description,writable FROM attribute_definitions WHERE integration=?1 ORDER BY rowid").map_err(db)?;
        let rows = stmt
            .query_map([integration], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, Option<String>>(7)?,
                    r.get::<_, bool>(8)?,
                ))
            })
            .map_err(db)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db)?;
        rows.into_iter()
            .map(
                |(
                    integration,
                    key,
                    label,
                    core_type,
                    choices,
                    validations,
                    grp,
                    description,
                    writable,
                )| {
                    Ok(Definition {
                        integration,
                        core_type: CoreType::parse(&core_type).ok_or_else(|| {
                            Error::InvalidData(format!("core type {core_type} of {key}"))
                        })?,
                        group: AttributeGroup::parse(&grp)
                            .ok_or_else(|| Error::InvalidData(format!("group {grp} of {key}")))?,
                        key,
                        label,
                        choices: choices
                            .map(|c| from_json(&c))
                            .transpose()?
                            .unwrap_or_default(),
                        validations: validations
                            .map(|v| from_json(&v))
                            .transpose()?
                            .unwrap_or_else(|| serde_json::json!({})),
                        description,
                        writable,
                    })
                },
            )
            .collect()
    }
    fn replace_attribute_definitions(
        &mut self,
        integration: &str,
        definitions: &[Definition],
    ) -> Result<()> {
        self.tx
            .execute(
                "DELETE FROM attribute_definitions WHERE integration=?1",
                [integration],
            )
            .map_err(db)?;
        for d in definitions {
            self.tx.execute("INSERT INTO attribute_definitions(integration,key,label,core_type,choices,validations,grp,description,writable) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)", params![integration,d.key,d.label,d.core_type.as_str(),to_json(&d.choices)?,to_json(&d.validations)?,d.group.as_str(),d.description,d.writable]).map_err(db)?;
        }
        Ok(())
    }
    fn attribute_values(&self, item_id: i64) -> Result<Vec<AttributeValue>> {
        let mut stmt = self.tx.prepare("SELECT item_id,key,value,valid,provenance,ai_confidence,ai_status,ai_reason,ai_sources,updated_at FROM attribute_values WHERE item_id=?1 ORDER BY key").map_err(db)?;
        let rows = stmt
            .query_map([item_id], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, bool>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, Option<String>>(6)?,
                    r.get::<_, Option<String>>(7)?,
                    r.get::<_, Option<String>>(8)?,
                    r.get::<_, i64>(9)?,
                ))
            })
            .map_err(db)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db)?;
        rows.into_iter()
            .map(
                |(
                    item_id,
                    key,
                    value,
                    valid,
                    provenance,
                    confidence,
                    status,
                    reason,
                    sources,
                    updated_at,
                )| {
                    let bad = |what: &str| Error::InvalidData(format!("{what} of value {key}"));
                    Ok(AttributeValue {
                        item_id,
                        provenance: Provenance::parse(&provenance)
                            .ok_or_else(|| bad("provenance"))?,
                        ai_confidence: confidence
                            .map(|c| AiConfidence::parse(&c).ok_or_else(|| bad("confidence")))
                            .transpose()?,
                        ai_status: status
                            .map(|s| AiStatus::parse(&s).ok_or_else(|| bad("ai status")))
                            .transpose()?,
                        ai_reason: reason,
                        ai_sources: sources.map(|s| from_json(&s)).transpose()?,
                        key,
                        value,
                        valid,
                        updated_at,
                    })
                },
            )
            .collect()
    }
    fn put_attribute_value(&mut self, v: &AttributeValue) -> Result<()> {
        let sources = v.ai_sources.as_ref().map(to_json).transpose()?;
        self.tx.execute("INSERT INTO attribute_values(item_id,key,value,valid,provenance,ai_confidence,ai_status,ai_reason,ai_sources,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10) ON CONFLICT(item_id,key) DO UPDATE SET value=excluded.value,valid=excluded.valid,provenance=excluded.provenance,ai_confidence=excluded.ai_confidence,ai_status=excluded.ai_status,ai_reason=excluded.ai_reason,ai_sources=excluded.ai_sources,updated_at=excluded.updated_at", params![v.item_id,v.key,v.value,v.valid,v.provenance.as_str(),v.ai_confidence.map(AiConfidence::as_str),v.ai_status.map(AiStatus::as_str),v.ai_reason,sources,v.updated_at]).map_err(db)?;
        Ok(())
    }
    fn set_item_activity(&mut self, item_id: i64, at: i64) -> Result<()> {
        self.tx
            .execute(
                "UPDATE sessions SET last_activity_at=?1 WHERE item_id=?2",
                params![at, item_id],
            )
            .map_err(db)?;
        Ok(())
    }
    fn attribute_preset(&self, item_type: &str, group: AttributeGroup) -> Result<Option<Preset>> {
        let layout: Option<String> = self
            .tx
            .query_row(
                "SELECT layout FROM attribute_presets WHERE item_type=?1 AND grp=?2",
                params![item_type, group.as_str()],
                |r| r.get(0),
            )
            .optional()
            .map_err(db)?;
        layout
            .map(|layout| {
                Ok(Preset {
                    item_type: item_type.to_owned(),
                    group,
                    layout: from_json(&layout)?,
                })
            })
            .transpose()
    }
    fn put_attribute_preset(&mut self, preset: &Preset) -> Result<()> {
        self.tx.execute("INSERT INTO attribute_presets(item_type,grp,layout) VALUES (?1,?2,?3) ON CONFLICT(item_type,grp) DO UPDATE SET layout=excluded.layout", params![preset.item_type,preset.group.as_str(),to_json(&preset.layout)?]).map_err(db)?;
        Ok(())
    }
}

fn to_json<T: serde::Serialize + ?Sized>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|e| Error::InvalidData(e.to_string()))
}

fn from_json<T: serde::de::DeserializeOwned>(text: &str) -> Result<T> {
    serde_json::from_str(text).map_err(|e| Error::InvalidData(e.to_string()))
}

impl Store for SqliteStore {
    fn preview_states(&self) -> Result<Vec<RawPreviewState>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let active: Option<i64> = conn
            .query_row(
                "SELECT value FROM settings WHERE key='active_session_id'",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(db)?
            .and_then(|v| v.parse().ok());
        let mut stmt = conn.prepare("SELECT p.raw_path FROM photos p GROUP BY p.raw_path ORDER BY MIN(CASE WHEN p.session_id=?1 THEN 0 ELSE 1 END), MIN(CASE WHEN p.session_id=?1 THEN CASE p.classification WHEN 'main' THEN 0 WHEN 'condition' THEN 1 WHEN 'unreviewed' THEN 2 ELSE 3 END ELSE 4 END), MIN(CASE WHEN p.session_id=?1 THEN COALESCE(p.position,9223372036854775807) ELSE 9223372036854775807 END), MIN(p.captured_at), MIN(p.id)").map_err(db)?;
        let raws: Vec<String> = stmt
            .query_map([active], |r| r.get(0))
            .map_err(db)?
            .collect::<rusqlite::Result<_>>()
            .map_err(db)?;
        raws.into_iter()
            .filter_map(|raw| match raw_preview_state(&conn, &raw) {
                Ok(Some(v)) => Some(Ok(v)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            })
            .collect()
    }
    fn preview_state(&self, raw: &str) -> Result<Option<RawPreviewState>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        raw_preview_state(&conn, raw)
    }
    fn ensure_initial_generation(&self, raw: &str) -> Result<Option<i64>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let changed = conn.execute("UPDATE raw_state SET adjusted_gen=MAX(adjusted_gen,1) WHERE raw_path=?1 AND EXISTS(SELECT 1 FROM photos WHERE raw_path=?1)", [raw]).map_err(db)?;
        if changed == 0 {
            return Ok(None);
        }
        conn.query_row(
            "SELECT adjusted_gen FROM raw_state WHERE raw_path=?1",
            [raw],
            |r| r.get(0),
        )
        .map(Some)
        .map_err(db)
    }
    fn adjusted_event(&self, raw: &str, retouch: bool) -> Result<Option<Vec<i64>>> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(db)?;
        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM photos WHERE raw_path=?1)",
                [raw],
                |r| r.get(0),
            )
            .map_err(db)?;
        if !exists {
            return Ok(None);
        }
        tx.execute("INSERT INTO raw_state(raw_path,adjusted_gen,retouch_gen) VALUES (?1,1,?2) ON CONFLICT(raw_path) DO UPDATE SET adjusted_gen=adjusted_gen+1,retouch_gen=retouch_gen+excluded.retouch_gen", params![raw, i64::from(retouch)]).map_err(db)?;
        let sessions = if retouch {
            let mut stmt = tx.prepare("SELECT DISTINCT session_id FROM photos WHERE raw_path=?1 AND classification IN ('main','condition') ORDER BY session_id").map_err(db)?;
            let ids: Vec<i64> = stmt
                .query_map([raw], |r| r.get(0))
                .map_err(db)?
                .collect::<rusqlite::Result<_>>()
                .map_err(db)?;
            for id in &ids {
                tx.execute(
                    "UPDATE sessions SET change_seq=change_seq+1 WHERE id=?1",
                    [id],
                )
                .map_err(db)?;
            }
            ids
        } else {
            Vec::new()
        };
        tx.commit().map_err(db)?;
        Ok(Some(sessions))
    }
    fn complete_preview(&self, raw: &str, generation: i64, path: &str) -> Result<PreviewCommit> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(db)?;
        let prior: Option<String> = tx
            .query_row(
                "SELECT adjusted_preview_path FROM photos WHERE raw_path=?1 ORDER BY id LIMIT 1",
                [raw],
                |r| r.get(0),
            )
            .optional()
            .map_err(db)?
            .flatten();
        let mut stmt = tx
            .prepare("SELECT id,session_id FROM photos WHERE raw_path=?1 ORDER BY id")
            .map_err(db)?;
        let ids: Vec<(i64, i64)> = stmt
            .query_map([raw], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(db)?
            .collect::<rusqlite::Result<_>>()
            .map_err(db)?;
        drop(stmt);
        tx.execute(
            "UPDATE photos SET adjusted_preview_path=?1 WHERE raw_path=?2",
            params![path, raw],
        )
        .map_err(db)?;
        tx.execute(
            "UPDATE raw_state SET rendered_gen=MAX(rendered_gen,?1) WHERE raw_path=?2",
            params![generation, raw],
        )
        .map_err(db)?;
        tx.commit().map_err(db)?;
        Ok(PreviewCommit {
            previous_path: prior,
            photos: ids,
        })
    }
    fn set_missing_preview_paths(&self, raw: &str, path: &str) -> Result<Vec<(i64, i64)>> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(db)?;
        let mut stmt = tx.prepare("SELECT id,session_id FROM photos WHERE raw_path=?1 AND adjusted_preview_path IS NULL ORDER BY id").map_err(db)?;
        let rows = stmt
            .query_map([raw], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(db)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db)?;
        drop(stmt);
        tx.execute("UPDATE photos SET adjusted_preview_path=?1 WHERE raw_path=?2 AND adjusted_preview_path IS NULL", params![path, raw]).map_err(db)?;
        tx.commit().map_err(db)?;
        Ok(rows)
    }
    fn new_render_job(&self, raw: &str, now_ms: i64) -> Result<i64> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        conn.execute("INSERT INTO jobs(kind,target,payload,status,attempts,next_attempt_at,created_at,updated_at) VALUES ('render_preview',?1,'{}','running',0,?2,?2,?2)", params![raw,now_ms]).map_err(db)?;
        Ok(conn.last_insert_rowid())
    }
    fn new_export_run(&self, session_id: i64, snapshot: &str, now_ms: i64) -> Result<i64> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        conn.execute("INSERT INTO jobs(kind,target,payload,status,attempts,next_attempt_at,created_at,updated_at) VALUES ('render_export',?1,?2,'running',0,?3,?3,?3)", params![format!("session:{session_id}"),snapshot,now_ms]).map_err(db)?;
        Ok(conn.last_insert_rowid())
    }
    fn new_export_render_job(
        &self,
        raw: &str,
        run_id: i64,
        photo_id: i64,
        now_ms: i64,
    ) -> Result<i64> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let payload = serde_json::json!({"run_id": run_id, "photo_id": photo_id}).to_string();
        conn.execute("INSERT INTO jobs(kind,target,payload,status,attempts,next_attempt_at,created_at,updated_at) VALUES ('render_export',?1,?2,'running',0,?3,?3,?3)", params![raw,payload,now_ms]).map_err(db)?;
        Ok(conn.last_insert_rowid())
    }
    fn finish_render_job(&self, id: i64, error: Option<&str>, now_ms: i64) -> Result<()> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .execute(
                "UPDATE jobs SET status=?1,error=?2,updated_at=?3 WHERE id=?4",
                params![
                    if error.is_some() { "failed" } else { "done" },
                    error,
                    now_ms,
                    id
                ],
            )
            .map_err(db)?;
        Ok(())
    }
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

    fn confirmed_run(&self, run_id: i64) -> Result<Option<i64>> {
        let confirmed: Option<Option<i64>> = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .query_row(
                "SELECT confirmed_at FROM publish_runs WHERE id=?1",
                [run_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db)?;
        Ok(confirmed.flatten())
    }

    fn item_link(
        &self,
        item_id: i64,
        integration: &str,
    ) -> Result<Option<psm_core::domain::item::ItemLink>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        conn.query_row("SELECT item_id,integration,external_id,external_url,last_synced_at,create_pending,tags_base FROM item_links WHERE item_id=?1 AND integration=?2",params![item_id,integration],|r| Ok(psm_core::domain::item::ItemLink{item_id:r.get(0)?,integration:r.get(1)?,external_id:r.get(2)?,external_url:r.get(3)?,last_synced_at:r.get(4)?,create_pending:r.get(5)?,tags_base:r.get(6)?})).optional().map_err(db)
    }
    fn all_item_links(&self, integration: &str) -> Result<Vec<psm_core::domain::item::ItemLink>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut stmt=conn.prepare("SELECT item_id,integration,external_id,external_url,last_synced_at,create_pending,tags_base FROM item_links WHERE integration=?1 ORDER BY item_id").map_err(db)?;
        stmt.query_map([integration], |r| {
            Ok(psm_core::domain::item::ItemLink {
                item_id: r.get(0)?,
                integration: r.get(1)?,
                external_id: r.get(2)?,
                external_url: r.get(3)?,
                last_synced_at: r.get(4)?,
                create_pending: r.get(5)?,
                tags_base: r.get(6)?,
            })
        })
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)
    }
    fn definition_links(&self, integration: &str) -> Result<Vec<(String, DefinitionLink)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut stmt=conn.prepare("SELECT key,platform_type,platform_meta FROM attribute_definition_links WHERE integration=?1 ORDER BY rowid").map_err(db)?;
        let rows = stmt
            .query_map([integration], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(db)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db)?;
        rows.into_iter()
            .map(|(key, platform_type, meta)| {
                Ok((
                    key,
                    DefinitionLink {
                        platform_type,
                        platform_meta: from_json(&meta)?,
                    },
                ))
            })
            .collect()
    }
    fn publish_item(&self, item_id: i64) -> Result<PublishItem> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        publish_item(&conn, item_id)
    }
    fn active_publish_items(&self) -> Result<Vec<i64>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut stmt=conn.prepare("SELECT id FROM publish_items WHERE step IN ('prepare','upload','update','describe') ORDER BY id").map_err(db)?;
        stmt.query_map([], |r| r.get(0))
            .map_err(db)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db)
    }
    fn set_publish_step(
        &self,
        item_id: i64,
        step: PublishStep,
        failed_step: Option<PublishStep>,
        error: Option<&str>,
        at: i64,
    ) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        conn.execute(
            "UPDATE publish_items SET step=?1,failed_step=?2,error=?3 WHERE id=?4",
            params![step_name(step), failed_step.map(step_name), error, item_id],
        )
        .map_err(db)?;
        if step == PublishStep::Failed {
            conn.execute("UPDATE sessions SET last_activity_at=?1 WHERE id=(SELECT session_id FROM publish_items WHERE id=?2)",params![at,item_id]).map_err(db)?;
        }
        Ok(())
    }
    fn set_publish_progress(&self, item_id: i64, uploaded: i64, total: i64) -> Result<()> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .execute(
                "UPDATE publish_items SET uploaded=?1,total=?2 WHERE id=?3",
                params![uploaded, total, item_id],
            )
            .map_err(db)?;
        Ok(())
    }
    fn enqueue_publish_item(&self, item_id: i64, at: i64) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let queued:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM jobs WHERE kind='publish_step' AND target=?1 AND status='queued')",[item_id.to_string()],|r|r.get(0)).map_err(db)?;
        if !queued {
            conn.execute("INSERT INTO jobs(kind,target,payload,status,attempts,next_attempt_at,created_at,updated_at) VALUES ('publish_step',?1,'{}','queued',0,?2,?2,?2)",params![item_id.to_string(),at]).map_err(db)?;
        }
        Ok(())
    }
    fn set_publish_export(&self, item_id: i64, export_run_id: i64) -> Result<()> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .execute(
                "INSERT INTO settings(key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                params![format!("publish_export:{item_id}"),export_run_id.to_string()],
            )
            .map_err(db)?;
        Ok(())
    }
    fn photo_link(&self, photo_id: i64, item_id: i64, role: &str) -> Result<Option<PhotoLink>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        conn.query_row("SELECT photo_id,item_id,role,external_id,upload_state,export_sha256,superseded_ids FROM photo_links WHERE photo_id=?1 AND integration='shopify' AND item_id=?2 AND role=?3",params![photo_id,item_id,role],photo_link_row).optional().map_err(db)?.map(map_photo_link).transpose()
    }
    fn upsert_photo_link(&self, link: &PhotoLink, at: i64) -> Result<()> {
        self.conn.lock().map_err(|e| Error::InvalidData(e.to_string()))?.execute("INSERT INTO photo_links(photo_id,integration,item_id,role,external_id,upload_state,export_sha256,superseded_ids,updated_at) VALUES (?1,'shopify',?2,?3,?4,?5,?6,?7,?8) ON CONFLICT(photo_id,integration,item_id,role) DO UPDATE SET external_id=excluded.external_id,upload_state=excluded.upload_state,export_sha256=excluded.export_sha256,superseded_ids=excluded.superseded_ids,updated_at=excluded.updated_at",params![link.photo_id,link.item_id,link.role,link.external_id,link.upload_state,link.export_sha256,to_json(&link.superseded_ids)?,at]).map_err(db)?;
        Ok(())
    }
    fn photo_links_for_item(&self, item_id: i64) -> Result<Vec<PhotoLink>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut stmt=conn.prepare("SELECT photo_id,item_id,role,external_id,upload_state,export_sha256,superseded_ids FROM photo_links WHERE integration='shopify' AND item_id=?1 ORDER BY id").map_err(db)?;
        stmt.query_map([item_id], photo_link_row)
            .map_err(db)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db)?
            .into_iter()
            .map(map_photo_link)
            .collect()
    }
    fn complete_publish_item(&self, item_id: i64, at: i64, text: &str) -> Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(db)?;
        tx.execute("UPDATE publish_items SET step='done',failed_step=NULL,error=NULL,finished_at=?1 WHERE id=?2",params![at,item_id]).map_err(db)?;
        tx.execute("UPDATE sessions SET published_at=?1,published_seq=(SELECT confirmed_change_seq FROM publish_items WHERE id=?2),last_activity_at=?1 WHERE id=(SELECT session_id FROM publish_items WHERE id=?2)",params![at,item_id]).map_err(db)?;
        tx.execute("INSERT INTO notifications(session_id,run_id,kind,text,created_at) SELECT session_id,run_id,'published',?1,?2 FROM publish_items WHERE id=?3",params![text,at,item_id]).map_err(db)?;
        tx.execute(
            "DELETE FROM settings WHERE key=?1",
            [format!("publish_export:{item_id}")],
        )
        .map_err(db)?;
        tx.commit().map_err(db)
    }
    fn notifications(&self, session_id: Option<i64>) -> Result<Vec<Notification>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut stmt=conn.prepare("SELECT id,session_id,run_id,kind,text,created_at FROM notifications WHERE (?1 IS NULL OR session_id=?1) ORDER BY id DESC").map_err(db)?;
        stmt.query_map([session_id], |r| {
            Ok(Notification {
                id: r.get(0)?,
                session_id: r.get(1)?,
                run_id: r.get(2)?,
                kind: r.get(3)?,
                text: r.get(4)?,
                created_at: r.get(5)?,
            })
        })
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)
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
