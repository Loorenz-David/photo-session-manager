use crate::paths::AppPaths;
use psm_core::{
    Error, Result,
    domain::{
        attribute::{
            AiConfidence, AiPrior, AiStatus, AttributeGroup, CoreType, Definition, Preset,
            Provenance, Value as AttributeValue,
        },
        camera::{
            Association, DeliveryOutcome, DeliveryRow, DeliveryState, Exposure, ExposureInitiator,
            ExposureStatus, Link, NewDelivery, NewExposure, PendingUpdate,
        },
        composition::{
            Composition, CompositionState, CompositionValues, Fingerprint, Frame, FrameSource,
            Origin, OutputSlot, Rect,
        },
        editing::Job,
        item::{Item, ItemOrigin, NewItem},
        live::PhonePairing,
        output::slot_name,
        photo::{Classification, Integrity, NewPhoto, Photo, PhotoSource},
        publish::{DefinitionLink, Notification, PhotoLink, PublishItem},
        research::{Feedback, FeedbackAction},
        session::{
            LatestPublish, PublishStep, PublishedChanges, Session, SessionDto, SessionListRecord,
        },
        storage::{ArchivedImage, ItemRetention},
        template::{Container, PlaceRow, Presets, ShotBinding, ShotKey, Template, TemplateSummary},
    },
    mechanisms::{
        canon::canon,
        composition::{
            default_rect, fingerprint_compatible, orientation_of, refit_rect, review_reason,
            turned_frame,
        },
        remote::RemoteInput,
    },
    ports::store::{FrameCommit, PreviewCommit, RawPreviewState, Store, StoreTx},
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use rusqlite_migration::{M, Migrations};
use std::sync::Mutex;

pub struct SqliteStore {
    conn: Mutex<Connection>,
    default_shape: String,
}

impl SqliteStore {
    pub fn open(paths: &AppPaths) -> rusqlite_migration::Result<Self> {
        Self::open_with_default_shape(paths, "1:1")
    }
    pub fn open_with_default_shape(
        paths: &AppPaths,
        shape: &str,
    ) -> rusqlite_migration::Result<Self> {
        let mut conn = Connection::open(paths.db())?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        Migrations::new(Self::migrations()).to_latest(&mut conn)?;
        super::ensure::ensure_research_columns(&conn)?;
        super::ensure::ensure_live_columns(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            default_shape: shape.to_string(),
        })
    }

    /// The schema migrations in order (001…007); public so a test can stop at an earlier version
    /// and check a later migration over existing rows.
    pub fn migrations() -> Vec<M<'static>> {
        vec![
            M::up(include_str!("migrations/001_init.sql")),
            M::up(include_str!("migrations/002_domain.sql")),
            M::up(include_str!("migrations/003_retire_shape_on_arrival.sql")),
            M::up(include_str!("migrations/004_compositions.sql")),
            M::up(include_str!("migrations/005_photo_link_slots.sql")),
            M::up(include_str!("migrations/006_camera_ledger.sql")),
            M::up(include_str!("migrations/007_style_applied.sql")),
            M::up(include_str!("migrations/008_settings_fingerprint.sql")),
            M::up(include_str!("migrations/009_publish_item_type.sql")),
            M::up(include_str!("migrations/010_retention.sql")),
            M::up(include_str!("migrations/011_item_stock.sql")),
            M::up(include_str!("migrations/012_photo_denials.sql")),
            M::up(include_str!("migrations/013_form_values.sql")),
            M::up(include_str!("migrations/014_shooting_templates.sql")),
            M::up(include_str!("migrations/015_photo_places.sql")),
            M::up(include_str!("migrations/016_container_labels.sql")),
            M::up(include_str!("migrations/017_session_changes.sql")),
            M::up(include_str!("migrations/018_saved_pages.sql")),
            M::up(include_str!("migrations/019_conversations.sql")),
            M::up(include_str!("migrations/020_remote_items.sql")),
            M::up(include_str!("migrations/021_publish_details_only.sql")),
            M::up(include_str!("migrations/022_labels_access.sql")),
        ]
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

    /// One commit contains both app-store writes and research SQL writes.
    pub fn research_transaction<T>(
        &self,
        op: impl FnOnce(&mut SqliteTx<'_>) -> Result<T>,
    ) -> Result<T> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(db)?;
        let mut store_tx = SqliteTx {
            tx,
            default_shape: self.default_shape.clone(),
        };
        let value = op(&mut store_tx)?;
        store_tx.tx.commit().map_err(db)?;
        Ok(value)
    }
}

/// Replaces the item's remote pictures (`archived_images`) with `images` inside `tx`.
fn replace_archived_images(tx: &Transaction, item_id: i64, images: &[ArchivedImage]) -> Result<()> {
    tx.execute("DELETE FROM archived_images WHERE item_id=?1", [item_id])
        .map_err(db)?;
    for image in images {
        tx.execute(
            "INSERT INTO archived_images(item_id,role,position,media_id,url,bytes) VALUES (?1,?2,?3,?4,?5,?6)",
            params![item_id, image.role, image.position, image.media_id, image.url, image.bytes],
        )
        .map_err(db)?;
    }
    Ok(())
}

fn db(error: rusqlite::Error) -> Error {
    Error::InvalidData(error.to_string())
}

fn template_read(conn: &Connection, id: i64) -> Result<Template> {
    let item_type: String = conn
        .query_row(
            "SELECT item_type FROM shooting_templates WHERE id=?1",
            [id],
            |r| r.get(0),
        )
        .optional()
        .map_err(db)?
        .ok_or_else(|| Error::NotFound("template".into()))?;
    let mut stmt = conn
        .prepare(
            "SELECT id, final_no, shoot_no, example_file, split, shape, zoom, name, description
         FROM template_containers WHERE template_id=?1 ORDER BY final_no",
        )
        .map_err(db)?;
    let containers = stmt
        .query_map([id], |r| {
            let shape: Option<String> = r.get(5)?;
            let shape = shape.map(parse_enum).transpose()?;
            Ok(Container {
                id: r.get(0)?,
                final_no: r.get(1)?,
                shoot_no: r.get(2)?,
                example_file: r.get(3)?,
                example_path: None,
                presets: Presets {
                    split: r.get::<_, i64>(4)? != 0,
                    shape,
                    zoom: r.get(6)?,
                },
                name: r.get(7)?,
                description: r.get(8)?,
            })
        })
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)?;
    Ok(Template {
        id,
        item_type,
        containers,
    })
}

fn photo_place_read(conn: &Connection, photo_id: i64) -> Result<Option<PlaceRow>> {
    conn.query_row("SELECT photo_id,slot,place_by_hand,frame_by_hand,released,key_at,key_name,pending_zoom,shot_bound,shot_container_id FROM photo_places WHERE photo_id=?1", [photo_id], |r| Ok(PlaceRow {
        photo_id: r.get(0)?, slot: r.get(1)?, place_by_hand: r.get::<_,i64>(2)? != 0,
        frame_by_hand: r.get::<_,i64>(3)? != 0, released: r.get::<_,i64>(4)? != 0,
        key_at: r.get(5)?, key_name: r.get(6)?, pending_zoom: r.get(7)?,
        shot: (r.get::<_, i64>(8)? != 0).then_some(ShotBinding { container_id: r.get(9)? }),
    })).optional().map_err(db)
}

fn template_owner(conn: &Connection, container_id: i64) -> Result<i64> {
    conn.query_row(
        "SELECT template_id FROM template_containers WHERE id=?1",
        [container_id],
        |r| r.get(0),
    )
    .optional()
    .map_err(db)?
    .ok_or_else(|| Error::NotFound("container".into()))
}

fn template_touch(conn: &Connection, id: i64, at: i64) -> Result<()> {
    conn.execute(
        "UPDATE shooting_templates SET updated_at=?2 WHERE id=?1",
        params![id, at],
    )
    .map_err(db)?;
    Ok(())
}

fn template_tx<T>(
    store: &SqliteStore,
    op: impl FnOnce(&Transaction<'_>) -> Result<T>,
) -> Result<T> {
    let mut conn = store
        .conn
        .lock()
        .map_err(|e| Error::InvalidData(e.to_string()))?;
    let tx = conn.transaction().map_err(db)?;
    let result = op(&tx)?;
    tx.commit().map_err(db)?;
    Ok(result)
}

const STYLE_ELIGIBLE_PENDING: i64 = i64::MIN;

fn insert_default_composition(conn: &Connection, id: i64, shape: &str, at: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO compositions(photo_id,kind,shape,updated_at) VALUES (?1,'normal',?2,?3)",
        params![id, shape, at],
    )
    .map_err(db)?;
    Ok(())
}

fn parse_enum<T: serde::de::DeserializeOwned>(value: String) -> rusqlite::Result<T> {
    serde_json::from_value(serde_json::Value::String(value)).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}
fn composition_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Composition> {
    let x: Option<f64> = r.get(7)?;
    let fw: Option<i64> = r.get(11)?;
    let geometry: Option<String> = r.get(14)?;
    Ok(Composition {
        photo_id: r.get(0)?,
        kind: parse_enum(r.get(1)?)?,
        shape: parse_enum(r.get(2)?)?,
        orientation: parse_enum(r.get(3)?)?,
        quarter_turns: r.get(4)?,
        flip: parse_enum(r.get(5)?)?,
        straighten: r.get(6)?,
        rect: x
            .map(|x| -> rusqlite::Result<Rect> {
                Ok(Rect {
                    x,
                    y: r.get(8)?,
                    w: r.get(9)?,
                    h: r.get(10)?,
                })
            })
            .transpose()?,
        fingerprint: fw
            .map(|w| -> rusqlite::Result<Fingerprint> {
                Ok(Fingerprint {
                    frame: Frame { w, h: r.get(12)? },
                    source: parse_enum(r.get(13)?)?,
                    geometry: serde_json::from_str(geometry.as_deref().unwrap_or("{}")).map_err(
                        |e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                14,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        },
                    )?,
                })
            })
            .transpose()?,
        origin: parse_enum(r.get(15)?)?,
        revision: r.get(16)?,
        state: parse_enum(r.get(17)?)?,
        review_reason: r
            .get::<_, Option<String>>(18)?
            .map(parse_enum)
            .transpose()?,
        updated_at: r.get(19)?,
    })
}
const COMPOSITION_SELECT: &str = "SELECT c.photo_id,c.kind,c.shape,c.orientation,c.quarter_turns,c.flip,c.straighten,c.x,c.y,c.w,c.h,c.frame_w,c.frame_h,c.frame_source,c.geometry,c.origin,c.revision,c.state,c.review_reason,c.updated_at FROM compositions c";
fn composition(conn: &Connection, photo_id: i64) -> Result<Composition> {
    conn.query_row(
        &format!("{COMPOSITION_SELECT} WHERE c.photo_id=?1"),
        [photo_id],
        composition_row,
    )
    .optional()
    .map_err(db)?
    .ok_or_else(|| Error::NotFound(format!("composition {photo_id}")))
}
fn save_composition_in(
    conn: &Connection,
    photo_id: i64,
    values: &CompositionValues,
    origin: Origin,
    now_ms: i64,
) -> Result<(Composition, bool)> {
    let current = composition(conn, photo_id)?;
    if current.values().as_ref() == Some(values)
        && current.state == CompositionState::Ok
        && current.origin == origin
    {
        return Ok((current, false));
    }
    conn.execute("UPDATE compositions SET kind=?1,shape=?2,orientation=?3,quarter_turns=?4,flip=?5,straighten=?6,x=?7,y=?8,w=?9,h=?10,origin=?11,revision=revision+1,state='ok',review_reason=NULL,updated_at=?12 WHERE photo_id=?13", params![values.kind.as_str(),values.shape.as_str(),values.orientation.as_str(),values.quarter_turns,values.flip.as_str(),values.straighten,values.rect.x,values.rect.y,values.rect.w,values.rect.h,origin.as_str(),now_ms,photo_id]).map_err(db)?;
    conn.execute("UPDATE sessions SET change_seq=change_seq+1,last_activity_at=?1 WHERE id=(SELECT session_id FROM photos WHERE id=?2 AND classification!='deleted')", params![now_ms,photo_id]).map_err(db)?;
    // A10: a crop by hand is an edit in the app; one the app set itself is not.
    let kind = if origin == Origin::User {
        "app"
    } else {
        "auto"
    };
    conn.execute("INSERT INTO session_changes(session_id,seq,kind,raw_path,at) SELECT s.id,s.change_seq,?1,NULL,?2 FROM sessions s WHERE s.id=(SELECT session_id FROM photos WHERE id=?3 AND classification!='deleted')", params![kind,now_ms,photo_id]).map_err(db)?;
    Ok((composition(conn, photo_id)?, true))
}
fn adopt_frame_tx(conn: &Connection, raw: &str, observed: &Fingerprint) -> Result<Vec<(i64, i64)>> {
    let mut stmt = conn
        .prepare("SELECT id,session_id FROM photos WHERE raw_path=?1 ORDER BY id")
        .map_err(db)?;
    let rows: Vec<(i64, i64)> = stmt
        .query_map([raw], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(db)?
        .collect::<rusqlite::Result<_>>()
        .map_err(db)?;
    drop(stmt);
    let geometry =
        serde_json::to_string(&observed.geometry).map_err(|e| Error::InvalidData(e.to_string()))?;
    let mut needs_review = Vec::new();
    for &(id, session_id) in &rows {
        let current = composition(conn, id)?;
        if current.fingerprint.is_none() {
            let pending_zoom = photo_place_read(conn, id)?.and_then(|r| r.pending_zoom);
            let mut rect = current.rect.unwrap_or_else(|| {
                default_rect(
                    current.kind,
                    current.shape,
                    current.orientation,
                    turned_frame(observed.frame, current.quarter_turns),
                )
            });
            if let Some(zoom) = pending_zoom {
                rect = psm_core::mechanisms::template::zoomed(rect, zoom);
            }
            conn.execute("UPDATE compositions SET x=?1,y=?2,w=?3,h=?4,frame_w=?5,frame_h=?6,frame_source=?7,geometry=?8 WHERE photo_id=?9", params![rect.x,rect.y,rect.w,rect.h,observed.frame.w,observed.frame.h,observed.source.as_str(),geometry,id]).map_err(db)?;
            if pending_zoom.is_some() {
                conn.execute(
                    "UPDATE photo_places SET pending_zoom=NULL WHERE photo_id=?1",
                    [id],
                )
                .map_err(db)?;
            }
        } else if let Some(stored) = &current.fingerprint {
            if observed.source == FrameSource::Camera {
                // P58: the camera preview only stands in until Capture One has measured the frame.
                continue;
            }
            let provisional = stored.source == FrameSource::Camera;
            if provisional && orientation_of(stored.frame) == orientation_of(observed.frame) {
                // P58: Capture One's frame replaces the camera preview's. The composition is
                // fractions (MC-27), so it stays; it is re-fitted only if it would now leave the
                // image. Later Capture One geometry changes are compared against this render.
                let rect = current
                    .values()
                    .map(|values| refit_rect(&values, observed.frame))
                    .or(current.rect)
                    .unwrap_or_else(|| {
                        default_rect(
                            current.kind,
                            current.shape,
                            current.orientation,
                            turned_frame(observed.frame, current.quarter_turns),
                        )
                    });
                conn.execute("UPDATE compositions SET x=?1,y=?2,w=?3,h=?4,frame_w=?5,frame_h=?6,frame_source=?7,geometry=?8 WHERE photo_id=?9", params![rect.x,rect.y,rect.w,rect.h,observed.frame.w,observed.frame.h,observed.source.as_str(),geometry,id]).map_err(db)?;
            } else if provisional && current.origin == Origin::Default {
                // Capture One shows the photo the other way up from the camera preview and nobody
                // framed it yet: the default composition is made again on Capture One's frame.
                let rect = default_rect(
                    current.kind,
                    current.shape,
                    current.orientation,
                    turned_frame(observed.frame, current.quarter_turns),
                );
                conn.execute("UPDATE compositions SET x=?1,y=?2,w=?3,h=?4,frame_w=?5,frame_h=?6,frame_source=?7,geometry=?8 WHERE photo_id=?9", params![rect.x,rect.y,rect.w,rect.h,observed.frame.w,observed.frame.h,observed.source.as_str(),geometry,id]).map_err(db)?;
            } else if !fingerprint_compatible(stored, observed) {
                let reason = review_reason(stored, observed);
                conn.execute("UPDATE compositions SET frame_w=?1,frame_h=?2,frame_source=?3,geometry=?4,state='needs_review',review_reason=?5 WHERE photo_id=?6", params![observed.frame.w,observed.frame.h,observed.source.as_str(),geometry,reason.as_str(),id]).map_err(db)?;
                needs_review.push((id, session_id));
            } else if observed.source == FrameSource::Master {
                // A compatible master replaces the fingerprint: its size becomes the frame.
                conn.execute("UPDATE compositions SET frame_w=?1,frame_h=?2,frame_source='master',geometry=?3 WHERE photo_id=?4", params![observed.frame.w,observed.frame.h,geometry,id]).map_err(db)?;
            }
        }
    }
    Ok(needs_review)
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
            arranged_at: r.get(12)?,
        },
        dirty: published_at.is_some() && change_seq > published_seq,
        changes: None,
        photo_count: r.get(8)?,
        unreviewed_count: r.get(9)?,
        main_count: r.get(10)?,
        condition_count: r.get(11)?,
    })
}

fn session(conn: &Connection, id: i64) -> Result<SessionDto> {
    let mut dto = conn.query_row("SELECT s.id,s.item_id,s.published_at,s.published_seq,s.change_seq,s.last_activity_at,s.created_at,s.undo_slot,COUNT(p.id),COALESCE(SUM(p.classification='unreviewed'),0),COALESCE(SUM(p.classification='main'),0),COALESCE(SUM(p.classification='condition'),0),s.arranged_at FROM sessions s LEFT JOIN photos p ON p.session_id=s.id WHERE s.id=?1 GROUP BY s.id", [id], session_row)
        .optional().map_err(db)?.ok_or_else(|| Error::NotFound(format!("session {id}")))?;
    if dto.dirty {
        dto.changes = Some(published_changes(conn, &dto.session)?);
    }
    Ok(dto)
}

/// A10: why a published session is open again (`session_changes` rows past the publish; new
/// photos by creation time, which also covers changes made before that table existed).
fn published_changes(conn: &Connection, s: &Session) -> Result<PublishedChanges> {
    let published_at = s.published_at.unwrap_or(i64::MAX);
    let new_photos: i64 = conn.query_row("SELECT COUNT(*) FROM photos WHERE session_id=?1 AND classification<>'deleted' AND created_at>?2", params![s.id, published_at], |r| r.get(0)).map_err(db)?;
    let editor_edits: i64 = conn.query_row("SELECT COUNT(DISTINCT c.raw_path) FROM session_changes c WHERE c.session_id=?1 AND c.kind='editor' AND c.seq>?2 AND EXISTS(SELECT 1 FROM photos p WHERE p.session_id=?1 AND p.raw_path=c.raw_path AND p.classification IN ('main','condition'))", params![s.id, s.published_seq], |r| r.get(0)).map_err(db)?;
    let app_edits: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM session_changes WHERE session_id=?1 AND kind='app' AND seq>?2)", params![s.id, s.published_seq], |r| r.get(0)).map_err(db)?;
    let recorded: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM session_changes WHERE session_id=?1 AND seq>?2)",
            params![s.id, s.published_seq],
            |r| r.get(0),
        )
        .map_err(db)?;
    Ok(PublishedChanges {
        new_photos,
        editor_edits,
        app_edits,
        // Changes counted before this version kept no reason.
        unrecorded: new_photos == 0 && !recorded,
    })
}

/// A10: one `session_changes` row for the change just counted on `session_id`.
fn record_change(
    conn: &Connection,
    session_id: i64,
    kind: &str,
    raw: Option<&str>,
    at: i64,
) -> Result<()> {
    conn.execute("INSERT INTO session_changes(session_id,seq,kind,raw_path,at) SELECT id,change_seq,?2,?3,?4 FROM sessions WHERE id=?1", params![session_id, kind, raw, at]).map_err(db)?;
    Ok(())
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
        thumbnail_path: r.get(15)?,
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
    conn.query_row("SELECT id,uid,session_id,raw_path,raw_hash,integrity,provider_ref,original_preview_path,adjusted_preview_path,orientation,classification,position,captured_at,source,created_at,thumbnail_path FROM photos WHERE id=?1", [id], photo_row)
        .optional().map_err(db)?.ok_or_else(|| Error::NotFound(format!("photo {id}")))
}

fn raw_preview_state(conn: &Connection, raw: &str) -> Result<Option<RawPreviewState>> {
    conn.query_row("SELECT r.raw_path,r.adjusted_gen,r.rendered_gen,r.retouch_gen,r.last_own_write_at,(SELECT adjusted_preview_path FROM photos WHERE raw_path=r.raw_path ORDER BY id LIMIT 1),r.settings_fingerprint FROM raw_state r WHERE r.raw_path=?1 AND EXISTS(SELECT 1 FROM photos WHERE raw_path=r.raw_path)", [raw], |r| Ok(RawPreviewState { raw_path:r.get(0)?, adjusted_gen:r.get(1)?, rendered_gen:r.get(2)?, retouch_gen:r.get(3)?, last_own_write_at:r.get(4)?, adjusted_preview_path:r.get(5)?, settings_fingerprint:r.get(6)? })).optional().map_err(db)
}

/// A16: which RAWs a preview-state read covers.
#[derive(Clone, Copy)]
enum PreviewScope {
    /// Every RAW with a photo (startup, focus and Capture One reopen checks; masters).
    All,
    /// The RAWs the recurring work looks at: a photo that is not deleted, in a session that is
    /// neither published (and unchanged since) nor cleared (owner 2026-11-18).
    Live,
    /// One session's photos that are not deleted (a session switch).
    Session(i64),
}

/// A16: one query for every RAW's preview state (it was one query per RAW, so each recurring
/// read grew with the library), in the same order as before: the active session's photos first
/// (main, condition, unreviewed, then by position), then the rest by capture time.
fn preview_states_where(conn: &Connection, scope: PreviewScope) -> Result<Vec<RawPreviewState>> {
    let active: Option<i64> = conn
        .query_row(
            "SELECT value FROM settings WHERE key='active_session_id'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .map_err(db)?
        .and_then(|v| v.parse().ok());
    let (filter, session) = match scope {
        PreviewScope::All => ("1=1", None),
        PreviewScope::Live => (
            "p.classification<>'deleted' AND NOT (s.published_at IS NOT NULL AND s.change_seq<=s.published_seq) AND NOT EXISTS (SELECT 1 FROM item_retention ir WHERE ir.item_id=s.item_id AND ir.cleared_at IS NOT NULL)",
            None,
        ),
        PreviewScope::Session(id) => ("p.classification<>'deleted' AND p.session_id=?2", Some(id)),
    };
    let sql = format!(
        "SELECT r.raw_path,r.adjusted_gen,r.rendered_gen,r.retouch_gen,r.last_own_write_at,(SELECT adjusted_preview_path FROM photos WHERE raw_path=r.raw_path ORDER BY id LIMIT 1),r.settings_fingerprint FROM raw_state r JOIN photos p ON p.raw_path=r.raw_path JOIN sessions s ON s.id=p.session_id WHERE {filter} GROUP BY r.raw_path ORDER BY MIN(CASE WHEN p.session_id=?1 THEN 0 ELSE 1 END), MIN(CASE WHEN p.session_id=?1 THEN CASE p.classification WHEN 'main' THEN 0 WHEN 'condition' THEN 1 WHEN 'unreviewed' THEN 2 ELSE 3 END ELSE 4 END), MIN(CASE WHEN p.session_id=?1 THEN COALESCE(p.position,9223372036854775807) ELSE 9223372036854775807 END), MIN(p.captured_at), MIN(p.id)"
    );
    let mut stmt = conn.prepare(&sql).map_err(db)?;
    let row = |r: &rusqlite::Row<'_>| {
        Ok(RawPreviewState {
            raw_path: r.get(0)?,
            adjusted_gen: r.get(1)?,
            rendered_gen: r.get(2)?,
            retouch_gen: r.get(3)?,
            last_own_write_at: r.get(4)?,
            adjusted_preview_path: r.get(5)?,
            settings_fingerprint: r.get(6)?,
        })
    };
    let rows = match session {
        Some(id) => stmt.query_map(params![active, id], row),
        None => stmt.query_map(params![active], row),
    }
    .map_err(db)?;
    rows.collect::<rusqlite::Result<_>>().map_err(db)
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
        "held" => Ok(PublishStep::Held),
        "prepare" => Ok(PublishStep::Prepare),
        "upload" => Ok(PublishStep::Upload),
        "update" => Ok(PublishStep::Update),
        "describe" => Ok(PublishStep::Describe),
        "done" => Ok(PublishStep::Done),
        "failed" => Ok(PublishStep::Failed),
        "cancelled" => Ok(PublishStep::Cancelled),
        _ => Err(Error::InvalidData(format!("invalid publish step: {step}"))),
    }
}

fn latest_publish(conn: &Connection, session_id: i64) -> Result<Option<LatestPublish>> {
    type PublishRow = (i64, String, Option<String>, Option<String>, i64, i64);
    let raw: Option<PublishRow> = conn.query_row(
        "SELECT pi.id,pi.step,pi.failed_step,pi.error,pi.uploaded,pi.total FROM publish_items pi JOIN publish_runs pr ON pr.id=pi.run_id WHERE pi.session_id=?1 AND pr.confirmed_at IS NOT NULL ORDER BY pr.confirmed_at DESC,pr.id DESC,pi.id DESC LIMIT 1",
        [session_id],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?)),
    ).optional().map_err(db)?;
    raw.map(|(item_id, step, failed_step, error, uploaded, total)| {
        Ok(LatestPublish {
            item_id,
            step: parse_step(&step)?,
            failed_step: failed_step.map(|s| parse_step(&s)).transpose()?,
            error,
            uploaded,
            total,
        })
    })
    .transpose()
}

fn step_name(step: PublishStep) -> &'static str {
    match step {
        PublishStep::Held => "held",
        PublishStep::Prepare => "prepare",
        PublishStep::Upload => "upload",
        PublishStep::Update => "update",
        PublishStep::Describe => "describe",
        PublishStep::Done => "done",
        PublishStep::Failed => "failed",
        PublishStep::Cancelled => "cancelled",
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
    String,
);
fn publish_item(conn: &Connection, id: i64) -> Result<PublishItem> {
    let row: PublishItemRow = conn.query_row(
        "SELECT id,run_id,session_id,create_item,title,frozen_main,frozen_condition,frozen_attributes,announced_removals,confirmed_change_seq,step,failed_step,uploaded,total,error,(SELECT CAST(value AS INTEGER) FROM settings WHERE key='publish_export:'||publish_items.id),frozen_item_type FROM publish_items WHERE id=?1", [id],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?,r.get(9)?,r.get(10)?,r.get(11)?,r.get(12)?,r.get(13)?,r.get(14)?,r.get(15)?,r.get(16)?))
    ).optional().map_err(db)?.ok_or_else(|| Error::NotFound(format!("publish item {id}")))?;
    let (origin, pending, resolved, released, details_only): (Option<String>, Option<String>, Option<String>, Option<i64>, bool) = conn.query_row(
        "SELECT frozen_attribute_origin,pending_conflicts,resolved_attributes,released_at,details_only FROM publish_items WHERE id=?1", [id],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)),
    ).map_err(db)?;
    Ok(PublishItem {
        id: row.0,
        run_id: row.1,
        session_id: row.2,
        create_item: row.3,
        title: row.4.unwrap_or_default(),
        frozen_item_type: row.16,
        frozen_main: from_json(row.5.as_deref().unwrap_or("[]"))?,
        frozen_condition: from_json(row.6.as_deref().unwrap_or("[]"))?,
        frozen_attributes: from_json(row.7.as_deref().unwrap_or("{}"))?,
        frozen_attribute_origin: from_json(origin.as_deref().unwrap_or("{}"))?,
        pending_conflicts: from_json(pending.as_deref().unwrap_or("{}"))?,
        resolved_attributes: from_json(resolved.as_deref().unwrap_or("{}"))?,
        released_at: released,
        announced_removals: from_json(row.8.as_deref().unwrap_or("[]"))?,
        confirmed_change_seq: row.9.unwrap_or_default(),
        step: parse_step(&row.10)?,
        failed_step: row.11.as_deref().map(parse_step).transpose()?,
        uploaded: row.12,
        total: row.13,
        error: row.14,
        export_run_id: row.15,
        details_only,
    })
}

/// The columns `photo_link_row` reads, in order.
const PHOTO_LINK_COLUMNS: &str = "photo_id,item_id,role,external_id,upload_state,export_sha256,superseded_ids,revision,retouch_gen,slot,composition_rev";
type PhotoLinkRow = (
    i64,
    i64,
    String,
    Option<String>,
    String,
    Option<String>,
    Option<String>,
    i64,
    i64,
    String,
    i64,
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
        r.get(7)?,
        r.get(8)?,
        r.get(9)?,
        r.get(10)?,
    ))
}
fn map_photo_link(row: PhotoLinkRow) -> Result<PhotoLink> {
    Ok(PhotoLink {
        photo_id: row.0,
        item_id: row.1,
        role: row.2,
        slot: parse_slot(&row.9)?,
        external_id: row.3,
        upload_state: row.4,
        export_sha256: row.5,
        superseded_ids: from_json(row.6.as_deref().unwrap_or("[]"))?,
        revision: row.7,
        retouch_gen: row.8,
        composition_rev: row.10,
    })
}
fn parse_slot(text: &str) -> Result<OutputSlot> {
    serde_json::from_value(serde_json::Value::String(text.into()))
        .map_err(|_| Error::InvalidData(format!("unknown output slot {text}")))
}

fn enqueue_mirror_tag(conn: &Connection, raw_path: &str, photo_id: i64, now_ms: i64) -> Result<()> {
    let payload = serde_json::json!({"photo_id": photo_id}).to_string();
    let queued: Option<i64> = conn.query_row("SELECT id FROM jobs WHERE kind='mirror_tag' AND target=?1 AND status='queued' ORDER BY id DESC LIMIT 1", [raw_path], |r| r.get(0)).optional().map_err(db)?;
    if let Some(id) = queued {
        conn.execute(
            "UPDATE jobs SET payload=?1,next_attempt_at=?2,error=NULL,updated_at=?2 WHERE id=?3",
            params![payload, now_ms, id],
        )
        .map_err(db)?;
    } else {
        conn.execute("INSERT INTO jobs(kind,target,payload,status,attempts,next_attempt_at,created_at,updated_at) VALUES ('mirror_tag',?1,?2,'queued',0,?3,?3,?3)",params![raw_path,payload,now_ms]).map_err(db)?;
    }
    Ok(())
}

pub struct SqliteTx<'a> {
    tx: Transaction<'a>,
    default_shape: String,
}

impl SqliteTx<'_> {
    pub fn conn(&self) -> &Connection {
        &self.tx
    }
}

/// The `research_feedback` rows matching `filter` (a `WHERE` clause over `params`), oldest first.
fn feedback_rows(
    conn: &rusqlite::Connection,
    filter: &str,
    params: impl rusqlite::Params,
) -> Result<Vec<Feedback>> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT id,item_id,key,suggested_value,confidence,final_value,action,at,step_token FROM research_feedback {filter} ORDER BY id"
        ))
        .map_err(db)?;
    let rows = stmt
        .query_map(params, |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, i64>(7)?,
                r.get::<_, Option<String>>(8)?,
            ))
        })
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)?;
    rows.into_iter()
        .map(
            |(
                id,
                item_id,
                key,
                suggested_value,
                confidence,
                final_value,
                action,
                at,
                step_token,
            )| {
                Ok(Feedback {
                    id,
                    item_id,
                    key,
                    suggested_value,
                    confidence: AiConfidence::parse(&confidence)
                        .ok_or_else(|| Error::InvalidData("feedback confidence".into()))?,
                    final_value,
                    action: FeedbackAction::parse(&action)
                        .ok_or_else(|| Error::InvalidData("feedback action".into()))?,
                    at,
                    step_token,
                })
            },
        )
        .collect()
}

impl StoreTx for SqliteTx<'_> {
    fn template_for_session(&self, session_id: i64) -> Result<Option<Template>> {
        let Some(item_id) = session(&self.tx, session_id)?.session.item_id else {
            return Ok(None);
        };
        let key = psm_core::mechanisms::template::type_key(&item(&self.tx, item_id)?.item_type);
        let id: Option<i64> = self
            .tx
            .query_row(
                "SELECT id FROM shooting_templates WHERE type_key=?1",
                [key],
                |r| r.get(0),
            )
            .optional()
            .map_err(db)?;
        id.map(|id| template_read(&self.tx, id)).transpose()
    }
    fn shot_keys(&self, session_id: i64) -> Result<Vec<ShotKey>> {
        let mut stmt = self.tx.prepare("SELECT p.id,COALESCE(pp.key_at,p.captured_at,p.created_at),COALESCE(pp.key_name,(SELECT d.file_key FROM camera_deliveries d WHERE d.final_path=p.raw_path AND d.state='done' AND d.outcome!='duplicate' ORDER BY d.id DESC LIMIT 1)),COALESCE(pp.released,0),(SELECT d.exposure_id FROM camera_deliveries d WHERE d.final_path=p.raw_path AND d.state='done' AND d.outcome!='duplicate' ORDER BY d.id DESC LIMIT 1) FROM photos p LEFT JOIN photo_places pp ON pp.photo_id=p.id WHERE p.session_id=?1 ORDER BY p.id").map_err(db)?;
        stmt.query_map([session_id], |r| {
            Ok(ShotKey {
                photo_id: r.get(0)?,
                at: r.get(1)?,
                name: r.get(2)?,
                released: r.get::<_, i64>(3)? != 0,
                exposure_id: r.get(4)?,
            })
        })
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)
    }
    fn hold_times(&self, session_id: i64, now_ms: i64) -> Result<Vec<(i64, i64)>> {
        let mut stmt = self.tx.prepare("SELECT e.exposed_at,e.id FROM camera_exposures e WHERE e.session_id=?1 AND ?2-e.exposed_at<=?3 AND NOT EXISTS(SELECT 1 FROM camera_deliveries d JOIN photos p ON p.session_id=e.session_id AND p.raw_path=d.final_path WHERE d.exposure_id=e.id AND d.outcome!='duplicate') ORDER BY e.exposed_at,e.id").map_err(db)?;
        stmt.query_map(
            params![session_id, now_ms, psm_core::domain::camera::FIFO_WINDOW_MS],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)
    }
    fn exposure_shot(&self, exposure_id: i64) -> Result<Option<ShotBinding>> {
        self.tx
            .query_row(
                "SELECT shot_bound,shot_container_id FROM camera_exposures WHERE id=?1",
                [exposure_id],
                |r| {
                    Ok((r.get::<_, i64>(0)? != 0).then_some(ShotBinding {
                        container_id: r.get(1)?,
                    }))
                },
            )
            .optional()
            .map(Option::flatten)
            .map_err(db)
    }
    fn insert_exposure(&mut self, e: &NewExposure) -> Result<i64> {
        insert_exposure_row(&self.tx, e)
    }
    fn match_late_exposure(&mut self, exposure_id: i64, at: i64) -> Result<Option<ShotBinding>> {
        let row: Option<(i64, Option<i64>)> = self
            .tx
            .query_row(
                "SELECT d.id,(SELECT p.id FROM photos p WHERE p.raw_path=d.final_path ORDER BY p.id LIMIT 1) FROM camera_deliveries d WHERE d.exposure_id IS NULL AND d.state='done' AND d.outcome!='duplicate' AND d.completed_at BETWEEN ?1 AND ?2 ORDER BY d.completed_at,d.id LIMIT 1",
                params![at - psm_core::domain::camera::FIFO_WINDOW_MS, at],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(db)?;
        let Some((delivery_id, photo_id)) = row else {
            return Ok(None);
        };
        self.tx
            .execute(
                "UPDATE camera_deliveries SET exposure_id=?1, association='fifo' WHERE id=?2",
                params![exposure_id, delivery_id],
            )
            .map_err(db)?;
        self.tx
            .execute(
                "UPDATE camera_exposures SET status='received',session_id=(SELECT session_id FROM camera_deliveries WHERE id=?2) WHERE id=?1",
                params![exposure_id, delivery_id],
            )
            .map_err(db)?;
        let binding = match photo_id {
            Some(id) => photo_place_read(&self.tx, id)?.and_then(|row| row.shot),
            None => None,
        };
        if let Some(shot) = binding {
            self.tx
                .execute(
                    "UPDATE camera_exposures SET shot_bound=1,shot_container_id=?2 WHERE id=?1",
                    params![exposure_id, shot.container_id],
                )
                .map_err(db)?;
        }
        Ok(binding)
    }
    fn photo_place_row(&self, photo_id: i64) -> Result<Option<PlaceRow>> {
        photo_place_read(&self.tx, photo_id)
    }
    fn photo_place_put(&mut self, row: &PlaceRow) -> Result<()> {
        self.tx.execute("INSERT INTO photo_places(photo_id,slot,place_by_hand,frame_by_hand,released,key_at,key_name,pending_zoom,shot_bound,shot_container_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10) ON CONFLICT(photo_id) DO UPDATE SET slot=excluded.slot,place_by_hand=excluded.place_by_hand,frame_by_hand=excluded.frame_by_hand,released=excluded.released,key_at=excluded.key_at,key_name=excluded.key_name,pending_zoom=excluded.pending_zoom,shot_bound=excluded.shot_bound,shot_container_id=excluded.shot_container_id", params![row.photo_id,row.slot,row.place_by_hand as i64,row.frame_by_hand as i64,row.released as i64,row.key_at,row.key_name,row.pending_zoom,row.shot.is_some() as i64,row.shot.and_then(|b| b.container_id)]).map_err(db)?;
        Ok(())
    }
    fn photo_place_mark(
        &mut self,
        photo_id: i64,
        place: bool,
        frame: bool,
        slot: Option<i64>,
    ) -> Result<()> {
        let photo = photo(&self.tx, photo_id)?;
        if self
            .template_for_session(photo.session_id)?
            .is_none_or(|t| t.containers.is_empty())
        {
            return Ok(());
        }
        let mut row = photo_place_read(&self.tx, photo_id)?.unwrap_or(PlaceRow {
            photo_id,
            ..PlaceRow::default()
        });
        if place {
            row.place_by_hand = true;
            // The slot is the container the photo was last in: a classification or a Deny keeps
            // it (undo brings the photo back to it); only a Main photo is given a new one.
            if slot.is_some() && photo.classification == Classification::Main {
                row.slot = slot;
            }
        }
        if frame {
            row.frame_by_hand = true;
            row.pending_zoom = None;
        }
        self.photo_place_put(&row)
    }
    fn photo_place_delete(&mut self, photo_id: i64) -> Result<()> {
        self.tx
            .execute("DELETE FROM photo_places WHERE photo_id=?1", [photo_id])
            .map_err(db)?;
        Ok(())
    }
    fn template_frame_unknown(
        &mut self,
        photo_id: i64,
        kind: psm_core::domain::composition::CompositionKind,
        shape: psm_core::domain::composition::Shape,
        origin: Origin,
        zoom: Option<u8>,
        at: i64,
    ) -> Result<()> {
        self.tx.execute("UPDATE compositions SET kind=?2,shape=?3,orientation='named',x=NULL,y=NULL,w=NULL,h=NULL,origin=?4,revision=revision+1,state='ok',review_reason=NULL,updated_at=?5 WHERE photo_id=?1",params![photo_id,kind.as_str(),shape.as_str(),origin.as_str(),at]).map_err(db)?;
        self.tx
            .execute(
                "UPDATE photo_places SET pending_zoom=?2 WHERE photo_id=?1",
                params![photo_id, zoom],
            )
            .map_err(db)?;
        Ok(())
    }
    fn default_shape(&self) -> psm_core::domain::composition::Shape {
        psm_core::domain::composition::Shape::parse(&self.default_shape)
            .unwrap_or(psm_core::domain::composition::Shape::Square)
    }
    fn enqueue_mirror_tag(&mut self, raw_path: &str, photo_id: i64, now_ms: i64) -> Result<()> {
        enqueue_mirror_tag(&self.tx, raw_path, photo_id, now_ms)
    }
    fn set_photo_denied(&mut self, id: i64, at: i64) -> Result<()> {
        self.tx
            .execute(
                "INSERT INTO photo_denials(photo_id,denied_at,files_deleted_at) VALUES (?1,?2,NULL) ON CONFLICT(photo_id) DO UPDATE SET denied_at=excluded.denied_at WHERE files_deleted_at IS NULL",
                params![id, at],
            )
            .map_err(db)?;
        Ok(())
    }
    fn photo_denied_at(&self, id: i64) -> Result<Option<i64>> {
        self.tx
            .query_row(
                "SELECT denied_at FROM photo_denials WHERE photo_id=?1 AND files_deleted_at IS NULL",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db)
    }
    fn session(&self, id: i64) -> Result<Session> {
        Ok(session(&self.tx, id)?.session)
    }
    fn photo(&self, id: i64) -> Result<Photo> {
        photo(&self.tx, id)
    }
    fn photos(&self, session_id: i64) -> Result<Vec<Photo>> {
        let mut stmt = self.tx.prepare("SELECT id,uid,session_id,raw_path,raw_hash,integrity,provider_ref,original_preview_path,adjusted_preview_path,orientation,classification,position,captured_at,source,created_at,thumbnail_path FROM photos WHERE session_id=?1 ORDER BY CASE classification WHEN 'main' THEN 0 WHEN 'condition' THEN 1 WHEN 'unreviewed' THEN 2 ELSE 3 END,position,captured_at,id").map_err(db)?;
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
        if classification != Classification::Deleted {
            // P66: a restored photo is no longer denied (unless its files are already gone).
            self.tx
                .execute(
                    "DELETE FROM photo_denials WHERE photo_id=?1 AND files_deleted_at IS NULL",
                    [id],
                )
                .map_err(db)?;
        }
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
        self.tx.execute("INSERT INTO photos(uid,session_id,raw_path,raw_hash,integrity,provider_ref,original_preview_path,adjusted_preview_path,orientation,classification,position,captured_at,source,created_at,thumbnail_path) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)", params![uid,target_session,photo.raw_path,photo.raw_hash,photo.integrity.as_str(),photo.provider_ref,photo.original_preview_path,photo.adjusted_preview_path,photo.orientation,photo.classification.as_str(),position,photo.captured_at,photo.source.as_str(),photo.created_at,photo.thumbnail_path]).map_err(db)?;
        let id = self.tx.last_insert_rowid();
        self.tx.execute("INSERT INTO compositions SELECT ?1,kind,shape,orientation,quarter_turns,flip,straighten,x,y,w,h,frame_w,frame_h,frame_source,geometry,origin,revision,state,review_reason,updated_at FROM compositions WHERE photo_id=?2", params![id,photo.id]).map_err(db)?;
        Ok(id)
    }
    fn delete_photo_row(&mut self, id: i64) -> Result<()> {
        self.tx
            .execute("DELETE FROM compositions WHERE photo_id=?1", [id])
            .map_err(db)?;
        self.tx
            .execute("DELETE FROM photo_places WHERE photo_id=?1", [id])
            .map_err(db)?;
        self.tx
            .execute("DELETE FROM photo_denials WHERE photo_id=?1", [id])
            .map_err(db)?;
        self.tx
            .execute("DELETE FROM photos WHERE id=?1", [id])
            .map_err(db)?;
        Ok(())
    }
    fn publish_confirmed_since(&self, session_id: i64, seq: i64) -> Result<bool> {
        self.tx.query_row("SELECT EXISTS(SELECT 1 FROM publish_items WHERE session_id=?1 AND confirmed_change_seq IS NOT NULL AND confirmed_change_seq>=?2)", params![session_id, seq], |r| r.get(0)).map_err(db)
    }
    fn touch_session(&mut self, session_id: i64, at: i64) -> Result<()> {
        self.tx
            .execute(
                "UPDATE sessions SET change_seq=change_seq+1,last_activity_at=?1 WHERE id=?2",
                params![at, session_id],
            )
            .map_err(db)?;
        record_change(&self.tx, session_id, "app", None, at)?;
        Ok(())
    }
    fn touch_session_on_arrival(&mut self, session_id: i64, at: i64) -> Result<()> {
        self.tx
            .execute(
                "UPDATE sessions SET change_seq=change_seq+1,last_activity_at=?1 WHERE id=?2",
                params![at, session_id],
            )
            .map_err(db)?;
        record_change(&self.tx, session_id, "arrival", None, at)?;
        Ok(())
    }
    fn set_published_seq(&mut self, session_id: i64, seq: i64) -> Result<()> {
        self.tx
            .execute(
                "UPDATE sessions SET published_seq=?1 WHERE id=?2",
                params![seq, session_id],
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
    fn update_item_type(&mut self, id: i64, item_type: &str, at: i64) -> Result<()> {
        self.tx
            .execute(
                "UPDATE items SET item_type=?1,updated_at=?2 WHERE id=?3",
                params![item_type, at, id],
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
        self.tx.execute("INSERT INTO publish_items(run_id,session_id,create_item,title,frozen_main,frozen_condition,frozen_attributes,announced_removals,confirmed_change_seq,step,uploaded,total,frozen_item_type,frozen_attribute_origin,pending_conflicts,resolved_attributes,released_at,details_only) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,0,?11,?12,?13,?14,?15,?16,?17)",
            params![item.run_id,item.session_id,item.create_item,item.title,to_json(&item.frozen_main)?,to_json(&item.frozen_condition)?,to_json(&item.frozen_attributes)?,to_json(&item.announced_removals)?,item.confirmed_change_seq,step_name(item.step),item.total,item.frozen_item_type,to_json(&item.frozen_attribute_origin)?,to_json(&item.pending_conflicts)?,to_json(&item.resolved_attributes)?,item.released_at,item.details_only]).map_err(db)?;
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
        let mut stmt = self.tx.prepare("SELECT item_id,key,value,valid,provenance,ai_confidence,ai_status,ai_reason,ai_sources,updated_at,ai_prior FROM attribute_values WHERE item_id=?1 ORDER BY key").map_err(db)?;
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
                    r.get::<_, Option<String>>(10)?,
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
                    prior,
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
                        ai_prior: prior.map(|s| from_json::<AiPrior>(&s)).transpose()?,
                        key,
                        value,
                        valid,
                        updated_at,
                    })
                },
            )
            .collect()
    }
    fn held_publish_item_for_item(&self, item_id: i64) -> Result<Option<PublishItem>> {
        let sessions: Vec<i64> = self
            .tx
            .prepare("SELECT id FROM sessions WHERE item_id=?1 ORDER BY id")
            .map_err(db)?
            .query_map([item_id], |r| r.get(0))
            .map_err(db)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db)?;
        let mut held = None;
        for session_id in sessions {
            if let Some(latest) = latest_publish(&self.tx, session_id)?
                && latest.step == PublishStep::Held
                && held.is_none_or(|id| latest.item_id > id)
            {
                held = Some(latest.item_id);
            }
        }
        held.map(|id| publish_item(&self.tx, id)).transpose()
    }
    fn publish_item_in_tx(&self, id: i64) -> Result<PublishItem> {
        publish_item(&self.tx, id)
    }
    fn set_publish_resolved(
        &mut self,
        id: i64,
        resolved: &std::collections::BTreeMap<String, serde_json::Value>,
    ) -> Result<()> {
        self.tx
            .execute(
                "UPDATE publish_items SET resolved_attributes=?1 WHERE id=?2",
                params![to_json(resolved)?, id],
            )
            .map_err(db)?;
        Ok(())
    }
    fn release_publish_item(&mut self, id: i64, at: i64) -> Result<()> {
        self.tx
            .execute(
                "UPDATE publish_items SET step='prepare',released_at=?1 WHERE id=?2 AND step='held'",
                params![at, id],
            )
            .map_err(db)?;
        let queued: bool = self.tx.query_row("SELECT EXISTS(SELECT 1 FROM jobs WHERE kind='publish_step' AND target=?1 AND status='queued')", [id.to_string()], |r| r.get(0)).map_err(db)?;
        if !queued {
            self.tx.execute("INSERT INTO jobs(kind,target,payload,status,attempts,next_attempt_at,created_at,updated_at) VALUES ('publish_step',?1,'{}','queued',0,?2,?2,?2)", params![id.to_string(), at]).map_err(db)?;
        }
        Ok(())
    }
    fn cancel_held_publish_item(&mut self, id: i64) -> Result<bool> {
        let n = self
            .tx
            .execute(
                "UPDATE publish_items SET step='cancelled' WHERE id=?1 AND step='held'",
                [id],
            )
            .map_err(db)?;
        Ok(n == 1)
    }
    fn feedback_by_key(&self, item_id: i64, key: &str) -> Result<Vec<Feedback>> {
        feedback_rows(
            &self.tx,
            "WHERE item_id=?1 AND key=?2",
            params![item_id, key],
        )
    }
    fn feedback_by_step(&self, step_token: &str) -> Result<Vec<Feedback>> {
        feedback_rows(&self.tx, "WHERE step_token=?1", params![step_token])
    }
    fn update_feedback_final_value(&mut self, id: i64, final_value: Option<&str>) -> Result<()> {
        self.tx
            .execute(
                "UPDATE research_feedback SET final_value=?1 WHERE id=?2",
                params![final_value, id],
            )
            .map_err(db)?;
        Ok(())
    }
    fn insert_feedback(&mut self, feedback: &Feedback) -> Result<i64> {
        self.tx.execute("INSERT INTO research_feedback(item_id,key,suggested_value,confidence,final_value,action,at,step_token) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)", params![feedback.item_id,feedback.key,feedback.suggested_value,feedback.confidence.as_str(),feedback.final_value,feedback.action.as_str(),feedback.at,feedback.step_token]).map_err(db)?;
        Ok(self.tx.last_insert_rowid())
    }
    fn delete_feedback_by_step(&mut self, step_token: &str) -> Result<()> {
        self.tx
            .execute(
                "DELETE FROM research_feedback WHERE step_token=?1",
                [step_token],
            )
            .map_err(db)?;
        Ok(())
    }
    fn reinsert_feedback(&mut self, feedback: &Feedback) -> Result<()> {
        self.tx.execute("INSERT INTO research_feedback(id,item_id,key,suggested_value,confidence,final_value,action,at,step_token) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)", params![feedback.id,feedback.item_id,feedback.key,feedback.suggested_value,feedback.confidence.as_str(),feedback.final_value,feedback.action.as_str(),feedback.at,feedback.step_token]).map_err(db)?;
        Ok(())
    }
    fn delete_attribute_value(&mut self, item_id: i64, key: &str) -> Result<()> {
        self.tx
            .execute(
                "DELETE FROM attribute_values WHERE item_id=?1 AND key=?2",
                params![item_id, key],
            )
            .map_err(db)?;
        Ok(())
    }
    fn composition(&self, photo_id: i64) -> Result<Composition> {
        composition(&self.tx, photo_id)
    }
    fn save_composition(
        &mut self,
        photo_id: i64,
        values: &CompositionValues,
        origin: Origin,
        now_ms: i64,
    ) -> Result<(Composition, bool)> {
        save_composition_in(&self.tx, photo_id, values, origin, now_ms)
    }
    fn put_attribute_value(&mut self, v: &AttributeValue) -> Result<()> {
        let sources = v.ai_sources.as_ref().map(to_json).transpose()?;
        let prior = v.ai_prior.as_ref().map(to_json).transpose()?;
        self.tx.execute("INSERT INTO attribute_values(item_id,key,value,valid,provenance,ai_confidence,ai_status,ai_reason,ai_sources,updated_at,ai_prior) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11) ON CONFLICT(item_id,key) DO UPDATE SET value=excluded.value,valid=excluded.valid,provenance=excluded.provenance,ai_confidence=excluded.ai_confidence,ai_status=excluded.ai_status,ai_reason=excluded.ai_reason,ai_sources=excluded.ai_sources,updated_at=excluded.updated_at,ai_prior=excluded.ai_prior", params![v.item_id,v.key,v.value,v.valid,v.provenance.as_str(),v.ai_confidence.map(AiConfidence::as_str),v.ai_status.map(AiStatus::as_str),v.ai_reason,sources,v.updated_at,prior]).map_err(db)?;
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
    fn saved_attribute_values(&self, item_type: &str, key: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .tx
            .prepare("SELECT value FROM attribute_saved_values WHERE item_type=?1 AND key=?2 ORDER BY saved_at DESC, rowid DESC")
            .map_err(db)?;
        let rows = stmt
            .query_map(params![item_type, key], |r| r.get(0))
            .map_err(db)?;
        rows.collect::<rusqlite::Result<Vec<String>>>().map_err(db)
    }
    fn put_saved_attribute_value(
        &mut self,
        item_type: &str,
        key: &str,
        value: &str,
        at: i64,
    ) -> Result<()> {
        self.tx.execute("INSERT INTO attribute_saved_values(item_type,key,value,saved_at) VALUES (?1,?2,?3,?4) ON CONFLICT(item_type,key,value) DO UPDATE SET saved_at=excluded.saved_at", params![item_type,key,value,at]).map_err(db)?;
        Ok(())
    }
    fn delete_saved_attribute_value(
        &mut self,
        item_type: &str,
        key: &str,
        value: &str,
    ) -> Result<()> {
        self.tx
            .execute(
                "DELETE FROM attribute_saved_values WHERE item_type=?1 AND key=?2 AND value=?3",
                params![item_type, key, value],
            )
            .map_err(db)?;
        Ok(())
    }
    fn item_title_base(&self, item_id: i64, integration: &str) -> Result<Option<String>> {
        self.tx
            .query_row(
                "SELECT title FROM item_remote_titles WHERE item_id=?1 AND integration=?2",
                params![item_id, integration],
                |r| r.get(0),
            )
            .optional()
            .map_err(db)
    }
    fn set_item_title_base(&mut self, item_id: i64, integration: &str, title: &str) -> Result<()> {
        self.tx.execute("INSERT INTO item_remote_titles(item_id,integration,title) VALUES (?1,?2,?3) ON CONFLICT(item_id,integration) DO UPDATE SET title=excluded.title", params![item_id,integration,title]).map_err(db)?;
        Ok(())
    }
    fn item_folder_key_frozen(&self, item_id: i64) -> Result<bool> {
        item_folder_key_frozen(&self.tx, item_id)
    }
    fn insert_delivery(&mut self, d: &NewDelivery) -> Result<i64> {
        self.tx
            .execute(
                "INSERT INTO camera_deliveries(exposure_id,camera_filename,file_key,sha256,size,via,completed_at,association,outcome,session_id,written_path,final_path,state,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,'pending',?13)",
                params![
                    d.exposure_id,
                    d.camera_filename,
                    d.file_key,
                    d.sha256,
                    d.size as i64,
                    d.via.label(),
                    d.completed_at,
                    d.association.as_str(),
                    d.outcome.as_str(),
                    d.session_id,
                    d.written_path,
                    d.final_path,
                    d.created_at
                ],
            )
            .map_err(db)?;
        Ok(self.tx.last_insert_rowid())
    }
    fn update_pending_delivery(&mut self, id: i64, u: &PendingUpdate) -> Result<()> {
        self.tx
            .execute(
                "UPDATE camera_deliveries SET sha256=?2,size=?3,completed_at=?4,final_path=?5,outcome=?6,association=?7,exposure_id=?8,session_id=?9 WHERE id=?1 AND state='pending'",
                params![
                    id,
                    u.sha256,
                    u.size as i64,
                    u.completed_at,
                    u.final_path,
                    u.outcome.as_str(),
                    u.association.as_str(),
                    u.exposure_id,
                    u.session_id
                ],
            )
            .map_err(db)?;
        Ok(())
    }
    fn set_exposure_status(&mut self, id: i64, status: ExposureStatus) -> Result<()> {
        self.tx
            .execute(
                "UPDATE camera_exposures SET status=?2 WHERE id=?1",
                params![id, status.as_str()],
            )
            .map_err(db)?;
        Ok(())
    }
}

const DELIVERY_COLUMNS: &str = "id,exposure_id,camera_filename,file_key,sha256,size,via,completed_at,association,outcome,session_id,written_path,final_path,state,created_at";

fn bad_enum(column: usize, text: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        column,
        rusqlite::types::Type::Text,
        format!("unknown value {text}").into(),
    )
}

fn delivery_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<DeliveryRow> {
    let via: String = r.get(6)?;
    let association: String = r.get(8)?;
    let outcome: String = r.get(9)?;
    let state: String = r.get(13)?;
    Ok(DeliveryRow {
        id: r.get(0)?,
        exposure_id: r.get(1)?,
        camera_filename: r.get(2)?,
        file_key: r.get(3)?,
        sha256: r.get(4)?,
        size: r.get::<_, i64>(5)? as u64,
        via: Link::parse(&via).ok_or_else(|| bad_enum(6, &via))?,
        completed_at: r.get(7)?,
        association: Association::parse(&association).ok_or_else(|| bad_enum(8, &association))?,
        outcome: DeliveryOutcome::parse(&outcome).ok_or_else(|| bad_enum(9, &outcome))?,
        session_id: r.get(10)?,
        written_path: r.get(11)?,
        final_path: r.get(12)?,
        state: DeliveryState::parse(&state).ok_or_else(|| bad_enum(13, &state))?,
        created_at: r.get(14)?,
    })
}

fn phone_pairing_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<PhonePairing> {
    Ok(PhonePairing {
        id: r.get(0)?,
        name: r.get(1)?,
        created_at: r.get(2)?,
        last_seen_at: r.get(3)?,
        revoked_at: r.get(4)?,
    })
}

/// One `camera_exposures` row, `expecting`, with its binding (CS5; `shot_bound = 0` without one).
fn insert_exposure_row(conn: &Connection, e: &NewExposure) -> Result<i64> {
    conn.execute(
        "INSERT INTO camera_exposures(capture_id,exposed_at,initiated,session_id,status,expected_files,created_at,shot_bound,shot_container_id) VALUES (?1,?2,?3,?4,'expecting',NULL,?5,?6,?7)",
        params![
            e.capture_id,
            e.exposed_at,
            e.initiated.as_str(),
            e.session_id,
            e.created_at,
            i64::from(e.shot.is_some()),
            e.shot.and_then(|b| b.container_id)
        ],
    )
    .map_err(db)?;
    Ok(conn.last_insert_rowid())
}

fn exposure_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Exposure> {
    let initiated: String = r.get(3)?;
    let status: String = r.get(5)?;
    Ok(Exposure {
        id: r.get(0)?,
        capture_id: r.get(1)?,
        exposed_at: r.get(2)?,
        initiated: ExposureInitiator::parse(&initiated).ok_or_else(|| bad_enum(3, &initiated))?,
        session_id: r.get(4)?,
        status: ExposureStatus::parse(&status).ok_or_else(|| bad_enum(5, &status))?,
        expected_files: r.get::<_, Option<i64>>(6)?.map(|n| n.max(0) as u32),
        created_at: r.get(7)?,
        shot: (r.get::<_, i64>(8)? != 0).then_some(ShotBinding {
            container_id: r.get(9)?,
        }),
    })
}

fn deliveries_where(
    conn: &Connection,
    condition: &str,
    args: impl rusqlite::Params,
) -> Result<Vec<DeliveryRow>> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {DELIVERY_COLUMNS} FROM camera_deliveries WHERE {condition} ORDER BY id"
        ))
        .map_err(db)?;
    stmt.query_map(args, delivery_row)
        .map_err(db)?
        .collect::<rusqlite::Result<_>>()
        .map_err(db)
}

fn item_folder_key_frozen(conn: &Connection, item_id: i64) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM photos JOIN sessions ON sessions.id=photos.session_id WHERE sessions.item_id=?1) OR EXISTS(SELECT 1 FROM camera_deliveries d JOIN sessions s ON s.id=d.session_id WHERE s.item_id=?1 AND d.outcome IN ('photo','renamed_photo') AND d.state IN ('pending','done'))",
        [item_id],
        |r| r.get(0),
    )
    .map_err(db)
}

fn to_json<T: serde::Serialize + ?Sized>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|e| Error::InvalidData(e.to_string()))
}

fn from_json<T: serde::de::DeserializeOwned>(text: &str) -> Result<T> {
    serde_json::from_str(text).map_err(|e| Error::InvalidData(e.to_string()))
}

impl Store for SqliteStore {
    fn photo_place_row(&self, photo_id: i64) -> Result<Option<PlaceRow>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        photo_place_read(&conn, photo_id)
    }
    fn template_list(&self) -> Result<Vec<TemplateSummary>> {
        template_tx(self, |tx| {
            let mut stmt = tx.prepare("SELECT t.id, t.item_type, COUNT(c.id) FROM shooting_templates t LEFT JOIN template_containers c ON c.template_id=t.id GROUP BY t.id ORDER BY t.item_type").map_err(db)?;
            stmt.query_map([], |r| {
                Ok(TemplateSummary {
                    id: r.get(0)?,
                    item_type: r.get(1)?,
                    container_count: r.get(2)?,
                })
            })
            .map_err(db)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db)
        })
    }
    fn template_get(&self, key: &str) -> Result<Option<Template>> {
        template_tx(self, |tx| {
            let id: Option<i64> = tx
                .query_row(
                    "SELECT id FROM shooting_templates WHERE type_key=?1",
                    [key],
                    |r| r.get(0),
                )
                .optional()
                .map_err(db)?;
            id.map(|id| template_read(tx, id)).transpose()
        })
    }
    fn template_get_by_id(&self, id: i64) -> Result<Template> {
        template_tx(self, |tx| template_read(tx, id))
    }
    fn template_create(&self, item_type: &str, key: &str, at: i64) -> Result<Template> {
        template_tx(self, |tx| {
            if tx
                .query_row(
                    "SELECT id FROM shooting_templates WHERE type_key=?1",
                    [key],
                    |r| r.get::<_, i64>(0),
                )
                .optional()
                .map_err(db)?
                .is_some()
            {
                return Err(Error::TemplateExists);
            }
            tx.execute("INSERT INTO shooting_templates(item_type,type_key,created_at,updated_at) VALUES (?1,?2,?3,?3)", params![item_type, key, at]).map_err(db)?;
            template_read(tx, tx.last_insert_rowid())
        })
    }
    fn template_delete(&self, id: i64) -> Result<()> {
        template_tx(self, |tx| {
            template_read(tx, id)?;
            tx.execute("DELETE FROM template_containers WHERE template_id=?1", [id])
                .map_err(db)?;
            tx.execute("DELETE FROM shooting_templates WHERE id=?1", [id])
                .map_err(db)?;
            Ok(())
        })
    }
    fn template_container_add(
        &self,
        id: i64,
        example_file: Option<&str>,
        at: i64,
    ) -> Result<Template> {
        template_tx(self, |tx| {
            let t = template_read(tx, id)?;
            let next = t.containers.len() as i64 + 1;
            tx.execute("INSERT INTO template_containers(template_id,final_no,shoot_no,example_file) VALUES (?1,?2,?2,?3)", params![id, next, example_file]).map_err(db)?;
            template_touch(tx, id, at)?;
            template_read(tx, id)
        })
    }
    fn template_container_example_set(
        &self,
        id: i64,
        example_file: &str,
        at: i64,
    ) -> Result<(Template, Option<String>)> {
        template_tx(self, |tx| {
            let owner = template_owner(tx, id)?;
            let old: Option<String> = tx
                .query_row(
                    "SELECT example_file FROM template_containers WHERE id=?1",
                    [id],
                    |r| r.get(0),
                )
                .map_err(db)?;
            tx.execute(
                "UPDATE template_containers SET example_file=?2 WHERE id=?1",
                params![id, example_file],
            )
            .map_err(db)?;
            template_touch(tx, owner, at)?;
            Ok((template_read(tx, owner)?, old))
        })
    }
    fn template_container_remove(&self, id: i64, at: i64) -> Result<(Template, Option<String>)> {
        template_tx(self, |tx| {
            let owner = template_owner(tx, id)?;
            let old = template_read(tx, owner)?;
            let removed = old
                .containers
                .iter()
                .find(|c| c.id == id)
                .and_then(|c| c.example_file.clone());
            tx.execute("DELETE FROM template_containers WHERE id=?1", [id])
                .map_err(db)?;
            let mut final_order: Vec<_> = old.containers.iter().filter(|c| c.id != id).collect();
            final_order.sort_by_key(|c| c.final_no);
            for (n, c) in final_order.iter().enumerate() {
                tx.execute(
                    "UPDATE template_containers SET final_no=?2 WHERE id=?1",
                    params![c.id, n as i64 + 1],
                )
                .map_err(db)?;
            }
            final_order.sort_by_key(|c| c.shoot_no);
            for (n, c) in final_order.iter().enumerate() {
                tx.execute(
                    "UPDATE template_containers SET shoot_no=?2 WHERE id=?1",
                    params![c.id, n as i64 + 1],
                )
                .map_err(db)?;
            }
            template_touch(tx, owner, at)?;
            Ok((template_read(tx, owner)?, removed))
        })
    }
    fn template_order_set(
        &self,
        id: i64,
        ids: &[i64],
        shooting: bool,
        at: i64,
    ) -> Result<Template> {
        template_tx(self, |tx| {
            let current = template_read(tx, id)?;
            let expected: std::collections::HashSet<i64> =
                current.containers.iter().map(|c| c.id).collect();
            let actual: std::collections::HashSet<i64> = ids.iter().copied().collect();
            if ids.len() != expected.len() || actual != expected {
                return Err(Error::InvalidValue(
                    "container order must include every container once".into(),
                ));
            }
            let column = if shooting { "shoot_no" } else { "final_no" };
            let query = format!("UPDATE template_containers SET {column}=?2 WHERE id=?1");
            for (n, container_id) in ids.iter().enumerate() {
                tx.execute(&query, params![container_id, n as i64 + 1])
                    .map_err(db)?;
            }
            template_touch(tx, id, at)?;
            template_read(tx, id)
        })
    }
    fn template_container_label_set(
        &self,
        id: i64,
        name: Option<&str>,
        description: Option<&str>,
        at: i64,
    ) -> Result<Template> {
        template_tx(self, |tx| {
            let owner = template_owner(tx, id)?;
            tx.execute(
                "UPDATE template_containers SET name=?2, description=?3 WHERE id=?1",
                params![id, name, description],
            )
            .map_err(db)?;
            template_touch(tx, owner, at)?;
            template_read(tx, owner)
        })
    }
    fn template_container_presets_set(
        &self,
        id: i64,
        presets: &Presets,
        at: i64,
    ) -> Result<Template> {
        template_tx(self, |tx| {
            let owner = template_owner(tx, id)?;
            tx.execute(
                "UPDATE template_containers SET split=?2, shape=?3, zoom=?4 WHERE id=?1",
                params![
                    id,
                    i64::from(presets.split),
                    presets.shape.map(|s| s.as_str()),
                    presets.zoom
                ],
            )
            .map_err(db)?;
            template_touch(tx, owner, at)?;
            template_read(tx, owner)
        })
    }
    fn preview_states(&self) -> Result<Vec<RawPreviewState>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        preview_states_where(&conn, PreviewScope::All)
    }
    fn preview_states_live(&self) -> Result<Vec<RawPreviewState>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        preview_states_where(&conn, PreviewScope::Live)
    }
    fn preview_states_in_session(&self, session_id: i64) -> Result<Vec<RawPreviewState>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        preview_states_where(&conn, PreviewScope::Session(session_id))
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
                // A10: an adjustment made in Capture One.
                let now: i64 = tx
                    .query_row(
                        "SELECT CAST(strftime('%s','now') AS INTEGER)*1000",
                        [],
                        |r| r.get(0),
                    )
                    .map_err(db)?;
                record_change(&tx, *id, "editor", Some(raw), now)?;
            }
            ids
        } else {
            Vec::new()
        };
        tx.commit().map_err(db)?;
        Ok(Some(sessions))
    }
    fn complete_preview(
        &self,
        raw: &str,
        generation: i64,
        path: &str,
        thumbnail: Option<&str>,
        observed: Option<Fingerprint>,
    ) -> Result<PreviewCommit> {
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
        let prior_thumbnail: Option<String> = tx
            .query_row(
                "SELECT thumbnail_path FROM photos WHERE raw_path=?1 ORDER BY id LIMIT 1",
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
            "UPDATE photos SET adjusted_preview_path=?1,thumbnail_path=?2 WHERE raw_path=?3",
            params![path, thumbnail, raw],
        )
        .map_err(db)?;
        tx.execute(
            "UPDATE raw_state SET rendered_gen=MAX(rendered_gen,?1) WHERE raw_path=?2",
            params![generation, raw],
        )
        .map_err(db)?;
        let needs_review = observed
            .as_ref()
            .map(|f| adopt_frame_tx(&tx, raw, f))
            .transpose()?
            .unwrap_or_default();
        tx.commit().map_err(db)?;
        Ok(PreviewCommit {
            previous_path: prior,
            previous_thumbnail: prior_thumbnail,
            needs_review,
            photos: ids,
        })
    }
    fn composition(&self, photo_id: i64) -> Result<Composition> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        composition(&conn, photo_id)
    }
    fn compositions_in_session(&self, session_id: i64) -> Result<Vec<Composition>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut stmt = conn.prepare(&format!("{COMPOSITION_SELECT} JOIN photos p ON p.id=c.photo_id WHERE p.session_id=?1 ORDER BY CASE p.classification WHEN 'main' THEN 0 WHEN 'condition' THEN 1 WHEN 'unreviewed' THEN 2 ELSE 3 END,p.position,p.captured_at,p.id")).map_err(db)?;
        stmt.query_map([session_id], composition_row)
            .map_err(db)?
            .collect::<rusqlite::Result<_>>()
            .map_err(db)
    }
    fn save_composition(
        &self,
        photo_id: i64,
        values: &CompositionValues,
        origin: Origin,
        now_ms: i64,
    ) -> Result<(Composition, bool)> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(db)?;
        let saved = save_composition_in(&tx, photo_id, values, origin, now_ms)?;
        tx.commit().map_err(db)?;
        Ok(saved)
    }
    fn backfill_compositions(&self, default_shape: &str, now_ms: i64) -> Result<usize> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        conn.execute("INSERT INTO compositions(photo_id,kind,shape,updated_at) SELECT p.id,'normal',?1,?2 FROM photos p WHERE NOT EXISTS(SELECT 1 FROM compositions c WHERE c.photo_id=p.id)", params![default_shape,now_ms]).map_err(db)
    }
    fn mark_previews_stale_once(&self, flag_key: &str, _now_ms: i64) -> Result<usize> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(db)?;
        let already: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM settings WHERE key=?1)",
                [flag_key],
                |r| r.get(0),
            )
            .map_err(db)?;
        if already {
            return Ok(0);
        }
        let count = tx.execute("UPDATE raw_state SET adjusted_gen=adjusted_gen+1 WHERE EXISTS(SELECT 1 FROM photos WHERE photos.raw_path=raw_state.raw_path AND photos.adjusted_preview_path IS NOT NULL)", []).map_err(db)?;
        tx.execute(
            "INSERT INTO settings(key,value) VALUES (?1,'done')",
            [flag_key],
        )
        .map_err(db)?;
        tx.commit().map_err(db)?;
        Ok(count)
    }
    fn adopt_frame(&self, raw: &str, observed: Fingerprint) -> Result<FrameCommit> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(db)?;
        let needs_review = adopt_frame_tx(&tx, raw, &observed)?;
        tx.commit().map_err(db)?;
        Ok(FrameCommit { needs_review })
    }
    fn adjusted_gen(&self, raw: &str) -> Result<i64> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        conn.query_row(
            "SELECT adjusted_gen FROM raw_state WHERE raw_path=?1",
            [raw],
            |r| r.get(0),
        )
        .optional()
        .map_err(db)
        .map(|g| g.unwrap_or(0))
    }
    fn enqueue_render_output(&self, raw: &str, payload: &str, now_ms: i64) -> Result<i64> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        conn.execute("INSERT INTO jobs(kind,target,payload,status,attempts,next_attempt_at,created_at,updated_at) VALUES ('render_output',?1,?2,'queued',0,?3,?3,?3)", params![raw,payload,now_ms]).map_err(db)?;
        Ok(conn.last_insert_rowid())
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
    fn new_master_render_job(
        &self,
        raw: &str,
        run_id: i64,
        photo_id: i64,
        adjusted_gen: i64,
        now_ms: i64,
    ) -> Result<i64> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let payload = serde_json::json!({"run_id": run_id, "photo_id": photo_id, "adjusted_gen": adjusted_gen})
            .to_string();
        conn.execute("INSERT INTO jobs(kind,target,payload,status,attempts,next_attempt_at,created_at,updated_at) VALUES ('render_master',?1,?2,'running',0,?3,?3,?3)", params![raw,payload,now_ms]).map_err(db)?;
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
    fn phone_pairing_insert(&self, name: &str, token_sha256: &str, created_at: i64) -> Result<i64> {
        let conn = self.locked()?;
        conn.execute(
            "INSERT INTO phone_pairings(name,token_sha256,created_at) VALUES (?1,?2,?3)",
            params![name, token_sha256, created_at],
        )
        .map_err(db)?;
        Ok(conn.last_insert_rowid())
    }
    fn phone_pairings_ever(&self) -> Result<i64> {
        self.locked()?
            .query_row("SELECT COUNT(*) FROM phone_pairings", [], |r| r.get(0))
            .map_err(db)
    }
    fn phone_pairing_seen(&self, token_sha256: &str, at: i64) -> Result<Option<PhonePairing>> {
        let conn = self.locked()?;
        conn.execute(
            "UPDATE phone_pairings SET last_seen_at=?2 WHERE token_sha256=?1 AND revoked_at IS NULL",
            params![token_sha256, at],
        )
        .map_err(db)?;
        conn.query_row(
            "SELECT id,name,created_at,last_seen_at,revoked_at FROM phone_pairings WHERE token_sha256=?1 AND revoked_at IS NULL",
            [token_sha256],
            phone_pairing_row,
        )
        .optional()
        .map_err(db)
    }
    fn phone_pairing_revoke(&self, id: i64, at: i64) -> Result<()> {
        self.locked()?
            .execute(
                "UPDATE phone_pairings SET revoked_at=?2 WHERE id=?1 AND revoked_at IS NULL",
                params![id, at],
            )
            .map_err(db)?;
        Ok(())
    }
    fn phone_pairing_rename(&self, id: i64, name: &str) -> Result<()> {
        let changed = self
            .locked()?
            .execute(
                "UPDATE phone_pairings SET name=?2 WHERE id=?1 AND revoked_at IS NULL",
                params![id, name],
            )
            .map_err(db)?;
        if changed == 0 {
            return Err(Error::NotFound(format!("phone pairing {id}")));
        }
        Ok(())
    }
    fn phone_pairings(&self) -> Result<Vec<PhonePairing>> {
        let conn = self.locked()?;
        let mut stmt = conn
            .prepare("SELECT id,name,created_at,last_seen_at,revoked_at FROM phone_pairings WHERE revoked_at IS NULL ORDER BY id")
            .map_err(db)?;
        stmt.query_map([], phone_pairing_row)
            .map_err(db)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db)
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
    fn job_due(
        &self,
        now_ms: i64,
        only_kind: Option<&str>,
        excluded_kinds: &[&str],
    ) -> Result<Option<Job>> {
        let excluded =
            serde_json::to_string(excluded_kinds).map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(db)?;
        let id: Option<i64> = tx.query_row(
            "SELECT id FROM jobs WHERE status='queued' AND next_attempt_at<=?1 AND (?2 IS NULL OR kind=?2) AND kind NOT IN (SELECT value FROM json_each(?3)) ORDER BY next_attempt_at,id LIMIT 1",
            rusqlite::params![now_ms, only_kind, excluded],
            |r| r.get(0),
        ).optional().map_err(db)?;
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
    fn job_fail(&self, id: i64, code: &str, now_ms: i64) -> Result<()> {
        self.conn.lock().map_err(|e| Error::InvalidData(e.to_string()))?
            .execute("UPDATE jobs SET status='failed',attempts=attempts+1,error=?1,updated_at=?2 WHERE id=?3", params![code, now_ms, id]).map_err(db)?;
        Ok(())
    }
    fn count_jobs(&self, kind: &str, status: &str, exclude_error: Option<&str>) -> Result<i64> {
        self.conn.lock().map_err(|e| Error::InvalidData(e.to_string()))?
            .query_row("SELECT COUNT(*) FROM jobs WHERE kind=?1 AND status=?2 AND (?3 IS NULL OR error IS NULL OR error<>?3)", params![kind,status,exclude_error], |r| r.get(0)).map_err(db)
    }
    fn requeue_failed_jobs(
        &self,
        kind: &str,
        exclude_error: Option<&str>,
        now_ms: i64,
    ) -> Result<usize> {
        self.conn.lock().map_err(|e| Error::InvalidData(e.to_string()))?
            .execute("UPDATE jobs SET status='queued',attempts=0,error=NULL,next_attempt_at=?1,updated_at=?1 WHERE kind=?2 AND status='failed' AND (?3 IS NULL OR error IS NULL OR error<>?3)", params![now_ms,kind,exclude_error]).map_err(db)
    }
    fn requeue_running_jobs(&self) -> Result<()> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .execute("UPDATE jobs SET status='queued' WHERE status='running'", [])
            .map_err(db)?;
        Ok(())
    }
    fn set_settings_fingerprint(&self, raw_path: &str, fingerprint: &str) -> Result<()> {
        self.conn.lock().map_err(|e| Error::InvalidData(e.to_string()))?
            .execute("INSERT INTO raw_state(raw_path,settings_fingerprint) VALUES (?1,?2) ON CONFLICT(raw_path) DO UPDATE SET settings_fingerprint=excluded.settings_fingerprint",params![raw_path,fingerprint]).map_err(db)?;
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
        let mut store_tx = SqliteTx {
            tx,
            default_shape: self.default_shape.clone(),
        };
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

    fn arrange_session_once(&self, id: i64, at: i64) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let changed = conn.execute("UPDATE sessions SET arranged_at=?1 WHERE id=?2 AND item_id IS NOT NULL AND arranged_at IS NULL", params![at, id]).map_err(db)?;
        Ok(changed != 0)
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
            let non_deleted_missing_preview_count = conn.query_row("SELECT COUNT(*) FROM photos WHERE session_id=?1 AND classification<>'deleted' AND integrity<>'raw_missing' AND adjusted_preview_path IS NULL", [id], |r| r.get(0)).map_err(db)?;
            let (cover_adjusted_preview_path, cover_original_preview_path) = conn.query_row("SELECT adjusted_preview_path,original_preview_path FROM photos WHERE session_id=?1 AND classification='main' AND position=1", [id], |r| Ok((r.get(0)?, r.get(1)?))).optional().map_err(db)?.unwrap_or((None, None));
            let publish = latest_publish(&conn, id)?;
            let unresolved_conflicts = if let Some(latest) = &publish {
                if latest.step == PublishStep::Held {
                    publish_item(&conn, latest.item_id)?.unresolved_conflicts() as i64
                } else { 0 }
            } else { 0 };
            let (latest_step,failed_step,uploaded,total) = match &publish {
                Some(latest) => (Some(latest.step),latest.failed_step,latest.uploaded,latest.total),
                None => (None,None,0,0),
            };
            let waiting_for_editor: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM jobs WHERE kind='render_export' AND target=?1 AND status='running' AND error='waiting_for_editor')", [format!("session:{id}")], |r| r.get(0)).map_err(db)?;
            Ok(SessionListRecord { session,sku,item_type,title,non_deleted_missing_preview_count,cover_adjusted_preview_path,cover_original_preview_path,latest_step,failed_step,uploaded,total,unresolved_conflicts,publish,waiting_for_editor })
        }).collect()
    }

    fn latest_publish(
        &self,
        session_id: i64,
    ) -> Result<Option<psm_core::domain::session::LatestPublish>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        latest_publish(&conn, session_id)
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
        insert_default_composition(&tx, id, &self.default_shape, new.created_at)?;
        tx.execute(
            "UPDATE sessions SET change_seq=change_seq+1,last_activity_at=?1 WHERE id=?2",
            params![new.created_at, new.session_id],
        )
        .map_err(db)?;
        record_change(
            &tx,
            new.session_id,
            "arrival",
            Some(&new.raw_path),
            new.created_at,
        )?;
        tx.commit().map_err(db)?;
        photo(&conn, id)
    }

    fn arrival_insert(
        &self,
        mut new: NewPhoto,
        path_key: &str,
        classification: Classification,
    ) -> Result<(Photo, bool)> {
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
        let position: Option<i64> = if classification.is_ordered() {
            Some(
                tx.query_row(
                    "SELECT COUNT(*)+1 FROM photos WHERE session_id=?1 AND classification=?2",
                    params![new.session_id, classification.as_str()],
                    |r| r.get(0),
                )
                .map_err(db)?,
            )
        } else {
            None
        };
        tx.execute("INSERT INTO photos(uid,session_id,raw_path,raw_hash,source,captured_at,created_at,classification,position) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)", params![uid,new.session_id,new.raw_path,shared_hash.clone().or(new.raw_hash),new.source.as_str(),new.captured_at,new.created_at,classification.as_str(),position]).map_err(db)?;
        let id = tx.last_insert_rowid();
        insert_default_composition(&tx, id, &self.default_shape, new.created_at)?;
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
        record_change(
            &tx,
            new.session_id,
            "arrival",
            Some(&new.raw_path),
            new.created_at,
        )?;
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
        let mut stmt = conn.prepare("SELECT id,uid,session_id,raw_path,raw_hash,integrity,provider_ref,original_preview_path,adjusted_preview_path,orientation,classification,position,captured_at,source,created_at,thumbnail_path FROM photos WHERE session_id=?1 ORDER BY CASE classification WHEN 'main' THEN 0 WHEN 'condition' THEN 1 WHEN 'unreviewed' THEN 2 ELSE 3 END,position,captured_at,id").map_err(db)?;
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

    fn photos_for_raw(&self, raw_path: &str) -> Result<Vec<Photo>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT id,uid,session_id,raw_path,raw_hash,integrity,provider_ref,original_preview_path,adjusted_preview_path,orientation,classification,position,captured_at,source,created_at,thumbnail_path FROM photos WHERE raw_path=?1 ORDER BY id").map_err(db)?;
        stmt.query_map([raw_path], photo_row)
            .map_err(db)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db)
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

    fn style_applied_at(&self, raw_path: &str) -> Result<Option<i64>> {
        let raw_path = canon(std::path::Path::new(raw_path))?;
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .query_row(
                "SELECT CASE WHEN style_applied_at=?2 THEN NULL ELSE style_applied_at END FROM raw_state WHERE raw_path=?1",
                params![raw_path, STYLE_ELIGIBLE_PENDING],
                |r| r.get(0),
            )
            .optional()
            .map(|value| value.flatten())
            .map_err(db)
    }

    fn mark_style_eligible(&self, raw_path: &str) -> Result<()> {
        let raw_path = canon(std::path::Path::new(raw_path))?;
        self.conn.lock().map_err(|e| Error::InvalidData(e.to_string()))?
            .execute("INSERT INTO raw_state(raw_path,style_applied_at) VALUES (?1,?2) ON CONFLICT(raw_path) DO UPDATE SET style_applied_at=CASE WHEN style_applied_at IS NULL THEN excluded.style_applied_at ELSE style_applied_at END", params![raw_path, STYLE_ELIGIBLE_PENDING]).map_err(db)?;
        Ok(())
    }

    fn style_eligible_before_first_render(&self, raw_path: &str) -> Result<bool> {
        let raw_path = canon(std::path::Path::new(raw_path))?;
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .query_row(
                "SELECT style_applied_at IS ?2 AND rendered_gen=0 FROM raw_state WHERE raw_path=?1",
                params![raw_path, STYLE_ELIGIBLE_PENDING],
                |r| r.get(0),
            )
            .optional()
            .map(|eligible| eligible.unwrap_or(false))
            .map_err(db)
    }

    fn mark_style_applied(&self, raw_path: &str, now_ms: i64) -> Result<()> {
        let raw_path = canon(std::path::Path::new(raw_path))?;
        self.conn.lock().map_err(|e| Error::InvalidData(e.to_string()))?
            .execute("INSERT INTO raw_state(raw_path,style_applied_at) VALUES (?1,?2) ON CONFLICT(raw_path) DO UPDATE SET style_applied_at=CASE WHEN style_applied_at IS NULL OR style_applied_at=?3 THEN excluded.style_applied_at ELSE style_applied_at END", params![raw_path,now_ms,STYLE_ELIGIBLE_PENDING]).map_err(db)?;
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
        conn.query_row("SELECT item_id,integration,external_id,external_url,last_synced_at,create_pending,tags_base,item_type_base FROM item_links WHERE item_id=?1 AND integration=?2",params![item_id,integration],|r| Ok(psm_core::domain::item::ItemLink{item_id:r.get(0)?,integration:r.get(1)?,external_id:r.get(2)?,external_url:r.get(3)?,last_synced_at:r.get(4)?,create_pending:r.get(5)?,tags_base:r.get(6)?,item_type_base:r.get(7)?})).optional().map_err(db)
    }
    fn all_item_links(&self, integration: &str) -> Result<Vec<psm_core::domain::item::ItemLink>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut stmt=conn.prepare("SELECT item_id,integration,external_id,external_url,last_synced_at,create_pending,tags_base,item_type_base FROM item_links WHERE integration=?1 ORDER BY item_id").map_err(db)?;
        stmt.query_map([integration], |r| {
            Ok(psm_core::domain::item::ItemLink {
                item_id: r.get(0)?,
                integration: r.get(1)?,
                external_id: r.get(2)?,
                external_url: r.get(3)?,
                last_synced_at: r.get(4)?,
                create_pending: r.get(5)?,
                tags_base: r.get(6)?,
                item_type_base: r.get(7)?,
            })
        })
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)
    }
    fn set_item_tags_base(&self, item_id: i64, integration: &str, tags: &[String]) -> Result<()> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .execute(
                "UPDATE item_links SET tags_base=?3 WHERE item_id=?1 AND integration=?2",
                params![item_id, integration, to_json(tags)?],
            )
            .map_err(db)?;
        Ok(())
    }
    fn set_item_type_base(&self, item_id: i64, integration: &str, item_type: &str) -> Result<()> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .execute(
                "UPDATE item_links SET item_type_base=?3 WHERE item_id=?1 AND integration=?2",
                params![item_id, integration, item_type],
            )
            .map_err(db)?;
        Ok(())
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
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(db)?;
        tx.execute(
            "UPDATE publish_items SET step=?1,failed_step=?2,error=?3 WHERE id=?4",
            params![step_name(step), failed_step.map(step_name), error, item_id],
        )
        .map_err(db)?;
        if step == PublishStep::Failed {
            tx.execute("UPDATE sessions SET last_activity_at=?1 WHERE id=(SELECT session_id FROM publish_items WHERE id=?2)",params![at,item_id]).map_err(db)?;
            tx.execute("INSERT INTO notifications(session_id,run_id,kind,text,created_at) SELECT session_id,run_id,'failed',?1,?2 FROM publish_items WHERE id=?3",params![format!("Publish failed · {}", error.unwrap_or_default()),at,item_id]).map_err(db)?;
        }
        tx.commit().map_err(db)
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
    fn set_export_waiting(&self, export_run_id: i64, waiting: bool) -> Result<()> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .execute(
                "UPDATE jobs SET error=?2 WHERE id=?1 AND kind='render_export' AND status='running'",
                params![export_run_id, waiting.then_some("waiting_for_editor")],
            )
            .map_err(db)?;
        Ok(())
    }
    fn item_stock(
        &self,
        item_id: i64,
        integration: &str,
    ) -> Result<Option<psm_core::domain::publish::StockChoice>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        conn.query_row(
            "SELECT location_id,location_name,quantity FROM item_stock WHERE item_id=?1 AND integration=?2",
            params![item_id, integration],
            |r| {
                Ok(psm_core::domain::publish::StockChoice {
                    location_id: r.get(0)?,
                    location_name: r.get(1)?,
                    quantity: r.get(2)?,
                })
            },
        )
        .optional()
        .map_err(db)
    }
    fn set_item_stock(
        &self,
        item_id: i64,
        integration: &str,
        choice: Option<&psm_core::domain::publish::StockChoice>,
    ) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        match choice {
            Some(c) => conn.execute(
                "INSERT INTO item_stock(item_id,integration,location_id,location_name,quantity) VALUES (?1,?2,?3,?4,?5) ON CONFLICT(item_id,integration) DO UPDATE SET location_id=excluded.location_id,location_name=excluded.location_name,quantity=excluded.quantity",
                params![item_id, integration, c.location_id, c.location_name, c.quantity],
            ),
            None => conn.execute(
                "DELETE FROM item_stock WHERE item_id=?1 AND integration=?2",
                params![item_id, integration],
            ),
        }
        .map_err(db)?;
        Ok(())
    }
    fn publish_item_stock(
        &self,
        publish_item_id: i64,
    ) -> Result<Option<psm_core::domain::publish::StockChoice>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        conn.query_row(
            "SELECT location_id,location_name,quantity FROM publish_item_stock WHERE publish_item_id=?1",
            [publish_item_id],
            |r| {
                Ok(psm_core::domain::publish::StockChoice {
                    location_id: r.get(0)?,
                    location_name: r.get(1)?,
                    quantity: r.get(2)?,
                })
            },
        )
        .optional()
        .map_err(db)
    }
    fn set_publish_item_stock(
        &self,
        publish_item_id: i64,
        c: &psm_core::domain::publish::StockChoice,
    ) -> Result<()> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .execute(
                "INSERT OR REPLACE INTO publish_item_stock(publish_item_id,location_id,location_name,quantity) VALUES (?1,?2,?3,?4)",
                params![publish_item_id, c.location_id, c.location_name, c.quantity],
            )
            .map_err(db)?;
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
    fn export_retouch_gen(&self, export_run_id: i64, photo_id: i64) -> Result<Option<i64>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        conn.query_row("SELECT json_extract(payload,'$.retouch_gens.\"' || ?2 || '\"') FROM jobs WHERE id=?1 AND kind='render_export'", params![export_run_id,photo_id], |r| r.get(0)).optional().map_err(db).map(|value| value.flatten())
    }
    fn photo_link(
        &self,
        photo_id: i64,
        item_id: i64,
        role: &str,
        slot: OutputSlot,
    ) -> Result<Option<PhotoLink>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        conn.query_row(
            &format!("SELECT {PHOTO_LINK_COLUMNS} FROM photo_links WHERE photo_id=?1 AND integration='shopify' AND item_id=?2 AND role=?3 AND slot=?4"),
            params![photo_id, item_id, role, slot_name(slot)],
            photo_link_row,
        )
        .optional()
        .map_err(db)?
        .map(map_photo_link)
        .transpose()
    }
    fn upsert_photo_link(&self, link: &PhotoLink, at: i64) -> Result<()> {
        self.conn.lock().map_err(|e| Error::InvalidData(e.to_string()))?.execute("INSERT INTO photo_links(photo_id,integration,item_id,role,slot,external_id,upload_state,export_sha256,superseded_ids,updated_at,revision,retouch_gen,composition_rev) VALUES (?1,'shopify',?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12) ON CONFLICT(photo_id,integration,item_id,role,slot) DO UPDATE SET external_id=excluded.external_id,upload_state=excluded.upload_state,export_sha256=excluded.export_sha256,superseded_ids=excluded.superseded_ids,updated_at=excluded.updated_at,revision=excluded.revision,retouch_gen=excluded.retouch_gen,composition_rev=excluded.composition_rev",params![link.photo_id,link.item_id,link.role,slot_name(link.slot),link.external_id,link.upload_state,link.export_sha256,to_json(&link.superseded_ids)?,at,link.revision,link.retouch_gen,link.composition_rev]).map_err(db)?;
        Ok(())
    }
    fn retire_photo_links(
        &self,
        item_id: i64,
        photo_id: i64,
        role: &str,
        keep: &[OutputSlot],
        at: i64,
    ) -> Result<Vec<PhotoLink>> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let tx = conn.transaction().map_err(db)?;
        let candidates = {
            let mut stmt = tx
                .prepare(&format!("SELECT {PHOTO_LINK_COLUMNS} FROM photo_links WHERE photo_id=?1 AND integration='shopify' AND item_id=?2 AND role=?3 AND upload_state IS NOT 'retired' ORDER BY id"))
                .map_err(db)?;
            stmt.query_map(params![photo_id, item_id, role], photo_link_row)
                .map_err(db)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(db)?
        };
        let mut retired = Vec::new();
        for row in candidates {
            let mut link = map_photo_link(row)?;
            if keep.contains(&link.slot) {
                continue;
            }
            tx.execute(
                "UPDATE photo_links SET upload_state='retired', updated_at=?5 WHERE photo_id=?1 AND integration='shopify' AND item_id=?2 AND role=?3 AND slot=?4",
                params![photo_id, item_id, role, slot_name(link.slot), at],
            )
            .map_err(db)?;
            link.upload_state = "retired".into();
            retired.push(link);
        }
        tx.commit().map_err(db)?;
        Ok(retired)
    }
    fn photo_links_for_item(&self, item_id: i64) -> Result<Vec<PhotoLink>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut stmt = conn
            .prepare(&format!("SELECT {PHOTO_LINK_COLUMNS} FROM photo_links WHERE integration='shopify' AND item_id=?1 ORDER BY id"))
            .map_err(db)?;
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
        // A10: the reasons this publish answered are gone.
        tx.execute("DELETE FROM session_changes WHERE session_id=(SELECT session_id FROM publish_items WHERE id=?1) AND seq<=(SELECT published_seq FROM sessions WHERE id=(SELECT session_id FROM publish_items WHERE id=?1))",params![item_id]).map_err(db)?;
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
        let mut stmt=conn.prepare("SELECT n.id,n.session_id,n.run_id,n.kind,n.text,n.created_at,pi.id FROM notifications n JOIN publish_items pi ON pi.run_id=n.run_id AND pi.session_id=n.session_id WHERE (?1 IS NULL OR n.session_id=?1) ORDER BY n.id DESC").map_err(db)?;
        stmt.query_map([session_id], |r| {
            Ok(Notification {
                id: r.get(0)?,
                item_id: r.get(6)?,
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
    fn insert_exposure(&self, exposure: &NewExposure) -> Result<i64> {
        self.ledger_insert_exposure(exposure)
    }
    fn match_late_exposure(&self, exposure_id: i64, at: i64) -> Result<bool> {
        let conn = self.locked()?;
        let changed = conn.execute(
            "UPDATE camera_deliveries SET exposure_id=?1, association='fifo' WHERE id=(SELECT id FROM camera_deliveries WHERE exposure_id IS NULL AND state='done' AND outcome!='duplicate' AND completed_at BETWEEN ?2 AND ?3 ORDER BY completed_at,id LIMIT 1)",
            params![exposure_id, at - psm_core::domain::camera::FIFO_WINDOW_MS, at],
        ).map_err(db)?;
        if changed > 0 {
            conn.execute("UPDATE camera_exposures SET status='received',session_id=(SELECT session_id FROM camera_deliveries WHERE exposure_id=?1) WHERE id=?1", [exposure_id]).map_err(db)?;
        }
        Ok(changed > 0)
    }
    fn set_exposure_status(&self, id: i64, status: ExposureStatus) -> Result<()> {
        self.locked()?
            .execute(
                "UPDATE camera_exposures SET status=?2 WHERE id=?1",
                params![id, status.as_str()],
            )
            .map_err(db)?;
        Ok(())
    }
    fn set_exposures_status_where(
        &self,
        from: ExposureStatus,
        to: ExposureStatus,
    ) -> Result<usize> {
        self.locked()?
            .execute(
                "UPDATE camera_exposures SET status=?2 WHERE status=?1",
                params![from.as_str(), to.as_str()],
            )
            .map_err(db)
    }
    fn exposures(&self, status: Option<ExposureStatus>) -> Result<Vec<Exposure>> {
        self.ledger_exposures(status)
    }
    fn set_delivery_done(&self, id: i64) -> Result<()> {
        self.locked()?
            .execute(
                "UPDATE camera_deliveries SET state='done' WHERE id=?1",
                [id],
            )
            .map_err(db)?;
        Ok(())
    }
    fn deliveries_by_key(&self, file_key: &str) -> Result<Vec<DeliveryRow>> {
        deliveries_where(&*self.locked()?, "file_key=?1", [file_key])
    }
    fn deliveries_by_stem(&self, stem_key: &str) -> Result<Vec<DeliveryRow>> {
        let escaped = stem_key
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let rows = deliveries_where(
            &*self.locked()?,
            "file_key=?1 OR file_key LIKE ?2 ESCAPE '\\'",
            params![stem_key, format!("{escaped}.%")],
        )?;
        Ok(rows
            .into_iter()
            .filter(|row| psm_core::mechanisms::capture_ledger::stem_key(&row.file_key) == stem_key)
            .collect())
    }
    fn deliveries_for_exposure(&self, exposure_id: i64) -> Result<Vec<DeliveryRow>> {
        deliveries_where(&*self.locked()?, "exposure_id=?1", [exposure_id])
    }
    fn pending_deliveries(&self) -> Result<Vec<DeliveryRow>> {
        deliveries_where(&*self.locked()?, "state='pending'", [])
    }
    fn pending_delivery_for_written_path(&self, written_path: &str) -> Result<Option<DeliveryRow>> {
        Ok(deliveries_where(
            &*self.locked()?,
            "state='pending' AND written_path=?1",
            [written_path],
        )?
        .pop())
    }
    fn delivery_for_path(&self, canonical: &str) -> Result<Option<(Option<i64>, i64)>> {
        self.locked()?
            .query_row(
                "SELECT CASE WHEN EXISTS(SELECT 1 FROM sessions s WHERE s.id=d.session_id) THEN d.session_id END, COALESCE(e.exposed_at,d.completed_at) FROM camera_deliveries d LEFT JOIN camera_exposures e ON e.id=d.exposure_id WHERE d.final_path=?1 ORDER BY d.id DESC LIMIT 1",
                [canonical],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(db)
    }
    fn completed_names_since(&self, since_ms: i64) -> Result<Vec<(String, u64, i64)>> {
        self.ledger_completed_names_since(since_ms)
    }
    fn item_folder_key_frozen(&self, item_id: i64) -> Result<bool> {
        item_folder_key_frozen(&*self.locked()?, item_id)
    }
    fn deliveries(&self) -> Result<Vec<DeliveryRow>> {
        deliveries_where(&*self.locked()?, "1=1", [])
    }

    fn item_retention(&self, item_id: i64) -> Result<ItemRetention> {
        let conn = self.locked()?;
        Ok(conn
            .query_row(
                "SELECT keep_files,cleared_at,restored_at,remote_checked_at FROM item_retention WHERE item_id=?1",
                [item_id],
                |r| {
                    Ok(ItemRetention {
                        keep_files: r.get::<_, i64>(0)? != 0,
                        cleared_at: r.get(1)?,
                        restored_at: r.get(2)?,
                        remote_checked_at: r.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(db)?
            .unwrap_or_default())
    }
    fn set_item_keep_files(&self, item_id: i64, keep: bool) -> Result<()> {
        let conn = self.locked()?;
        conn.execute(
            "INSERT INTO item_retention(item_id,keep_files) VALUES (?1,?2) ON CONFLICT(item_id) DO UPDATE SET keep_files=excluded.keep_files",
            params![item_id, keep as i64],
        )
        .map_err(db)?;
        Ok(())
    }
    fn mark_item_cleared(&self, item_id: i64, at: i64, images: &[ArchivedImage]) -> Result<()> {
        let mut conn = self.locked()?;
        let tx = conn.transaction().map_err(db)?;
        tx.execute(
            "INSERT INTO item_retention(item_id,cleared_at) VALUES (?1,?2) ON CONFLICT(item_id) DO UPDATE SET cleared_at=excluded.cleared_at",
            params![item_id, at],
        )
        .map_err(db)?;
        replace_archived_images(&tx, item_id, images)?;
        tx.commit().map_err(db)
    }
    fn set_remote_record(
        &self,
        item_id: i64,
        images: &[ArchivedImage],
        checked_at: i64,
    ) -> Result<()> {
        let mut conn = self.locked()?;
        let tx = conn.transaction().map_err(db)?;
        // MC-H3.2: the pin, the clearing and the bring-back times stay as they are.
        tx.execute(
            "INSERT INTO item_retention(item_id,remote_checked_at) VALUES (?1,?2) ON CONFLICT(item_id) DO UPDATE SET remote_checked_at=excluded.remote_checked_at",
            params![item_id, checked_at],
        )
        .map_err(db)?;
        replace_archived_images(&tx, item_id, images)?;
        tx.commit().map_err(db)
    }
    fn update_remote_urls(
        &self,
        item_id: i64,
        updates: &[(String, Option<String>, Option<i64>)],
    ) -> Result<()> {
        let mut conn = self.locked()?;
        let tx = conn.transaction().map_err(db)?;
        for (media_id, url, bytes) in updates {
            // A `None` URL stores the empty string: the picture is gone (MC-H1.4).
            tx.execute(
                "UPDATE archived_images SET url=?3, bytes=COALESCE(?4,bytes) WHERE item_id=?1 AND media_id=?2",
                params![item_id, media_id, url.as_deref().unwrap_or(""), bytes],
            )
            .map_err(db)?;
        }
        tx.commit().map_err(db)
    }
    fn bring_back(&self, input: psm_core::domain::remote::BringBackInput) -> Result<Vec<i64>> {
        let mut conn = self.locked()?;
        let tx = conn.transaction().map_err(db)?;
        let at = input.at;
        // (a) Every photo row the session had is retired: hidden, its files gone (M-H4), its
        // links for the item gone, its place released (10G §6.4: it leaves the shot count and
        // leaves no container to refill). The rows stay (history, UNIQUE(session_id, raw_path)).
        tx.execute(
            "DELETE FROM photo_links WHERE item_id=?1 AND photo_id IN (SELECT id FROM photos WHERE session_id=?2)",
            params![input.item_id, input.session_id],
        )
        .map_err(db)?;
        tx.execute(
            "INSERT INTO photo_places(photo_id,released) SELECT id,1 FROM photos WHERE session_id=?1 ON CONFLICT(photo_id) DO UPDATE SET released=1",
            [input.session_id],
        )
        .map_err(db)?;
        tx.execute(
            "UPDATE photos SET classification='deleted',position=NULL,original_preview_path=NULL,adjusted_preview_path=NULL,thumbnail_path=NULL,integrity='raw_missing' WHERE session_id=?1",
            [input.session_id],
        )
        .map_err(db)?;
        // The containers of the session's shooting template (none for an untemplated session).
        let item_type: String = tx
            .query_row(
                "SELECT item_type FROM items WHERE id=?1",
                [input.item_id],
                |r| r.get(0),
            )
            .map_err(db)?;
        let containers: Vec<i64> = {
            let mut stmt = tx
                .prepare("SELECT c.final_no FROM template_containers c JOIN shooting_templates t ON t.id=c.template_id WHERE t.type_key=?1")
                .map_err(db)?;
            stmt.query_map(
                [psm_core::mechanisms::template::type_key(&item_type)],
                |r| r.get(0),
            )
            .map_err(db)?
            .collect::<rusqlite::Result<_>>()
            .map_err(db)?
        };
        let mut ids = Vec::with_capacity(input.files.len());
        for file in &input.files {
            let main = file.role == psm_core::services::retention::ROLE_GALLERY;
            let classification = if main { "main" } else { "condition" };
            // (b) The photo, its composition exactly as published, its RAW state and its jobs.
            let uid = uuid::Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO photos(uid,session_id,raw_path,raw_hash,source,captured_at,created_at,classification,position) VALUES (?1,?2,?3,?4,'import',?5,?5,?6,?7)",
                params![uid, input.session_id, file.path, file.sha256, at, classification, file.position],
            )
            .map_err(db)?;
            let id = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO compositions(photo_id,kind,shape,orientation,quarter_turns,flip,straighten,x,y,w,h,origin,revision,updated_at) VALUES (?1,'normal','original','named',0,'none',0,0,0,1,1,'default',0,?2)",
                params![id, at],
            )
            .map_err(db)?;
            tx.execute(
                "INSERT INTO raw_state(raw_path,path_key) VALUES (?1,?2) ON CONFLICT(raw_path) DO UPDATE SET path_key=excluded.path_key",
                params![file.path, psm_core::mechanisms::arrival::path_key(&file.path)],
            )
            .map_err(db)?;
            for kind in ["original_preview", "render_preview"] {
                tx.execute(
                    "INSERT INTO jobs(kind,target,payload,status,attempts,next_attempt_at,created_at,updated_at) VALUES (?1,?2,'{}','queued',0,?3,?3,?3)",
                    params![kind, file.path, at],
                )
                .map_err(db)?;
            }
            if main && !containers.is_empty() {
                // 10G §4.4, §6.1: the published order is kept; main photo k is placed by hand in
                // the container whose final number is k (none past the last container).
                let slot = containers.contains(&file.position).then_some(file.position);
                tx.execute(
                    "INSERT INTO photo_places(photo_id,slot,place_by_hand) VALUES (?1,?2,1)",
                    params![id, slot],
                )
                .map_err(db)?;
            }
            // (c) Linked to the media it came from, as uploaded at its current state.
            let retouch_gen: i64 = tx
                .query_row(
                    "SELECT retouch_gen FROM raw_state WHERE raw_path=?1",
                    [&file.path],
                    |r| r.get(0),
                )
                .map_err(db)?;
            let role = if main {
                psm_core::services::retention::ROLE_GALLERY.to_owned()
            } else {
                input.link_role_condition.clone()
            };
            tx.execute(
                "INSERT INTO photo_links(photo_id,integration,item_id,role,slot,external_id,revision,retouch_gen,composition_rev,superseded_ids,upload_state,updated_at) VALUES (?1,'shopify',?2,?3,'single',?4,0,?5,0,NULL,'ready',?6)",
                params![id, input.item_id, role, file.media_id, retouch_gen, at],
            )
            .map_err(db)?;
            ids.push(id);
        }
        // (d) The item is on this Mac again; the record rows stay (ignored while it is local).
        tx.execute(
            "INSERT INTO item_retention(item_id,restored_at) VALUES (?1,?2) ON CONFLICT(item_id) DO UPDATE SET cleared_at=NULL,restored_at=excluded.restored_at",
            params![input.item_id, at],
        )
        .map_err(db)?;
        // (e) `sessions.change_seq` is left as it is: bringing back is not an edit.
        tx.commit().map_err(db)?;
        Ok(ids)
    }
    fn remote_input(&self, session_id: i64) -> Result<RemoteInput> {
        let conn = self.locked()?;
        let input = conn
            .query_row(
                "SELECT r.cleared_at IS NOT NULL, \
                        EXISTS(SELECT 1 FROM item_links l WHERE l.item_id=i.id AND l.integration=?2), \
                        i.origin='linked', \
                        (SELECT COUNT(*) FROM photos p WHERE p.session_id=s.id), \
                        r.remote_checked_at IS NOT NULL, \
                        (SELECT COUNT(*) FROM archived_images a WHERE a.item_id=i.id) \
                 FROM sessions s JOIN items i ON i.id=s.item_id \
                 LEFT JOIN item_retention r ON r.item_id=i.id WHERE s.id=?1",
                params![session_id, psm_core::services::retention::INTEGRATION],
                |r| {
                    Ok(RemoteInput {
                        cleared: r.get::<_, Option<bool>>(0)?.unwrap_or(false),
                        linked: r.get(1)?,
                        adopted: r.get(2)?,
                        photo_rows: r.get::<_, i64>(3)? as usize,
                        checked: r.get::<_, Option<bool>>(4)?.unwrap_or(false),
                        pictures: r.get::<_, i64>(5)? as usize,
                    })
                },
            )
            .optional()
            .map_err(db)?;
        Ok(input.unwrap_or_default())
    }
    fn remote_covers(&self) -> Result<Vec<psm_core::mechanisms::remote::RemoteCover>> {
        let conn = self.locked()?;
        let mut stmt = conn
            .prepare(
                "SELECT s.id, i.id, i.sku, \
                        r.cleared_at IS NOT NULL, \
                        EXISTS(SELECT 1 FROM item_links l WHERE l.item_id=i.id AND l.integration=?1), \
                        i.origin='linked', \
                        (SELECT COUNT(*) FROM photos p WHERE p.session_id=s.id), \
                        r.remote_checked_at IS NOT NULL, \
                        (SELECT COUNT(*) FROM archived_images a WHERE a.item_id=i.id), \
                        c.position, c.media_id, c.url, c.bytes, \
                        (SELECT COUNT(*) FROM archived_images g WHERE g.item_id=i.id AND g.role='gallery'), \
                        (SELECT COUNT(*) FROM archived_images k WHERE k.item_id=i.id AND k.role='condition') \
                 FROM sessions s JOIN items i ON i.id=s.item_id \
                 LEFT JOIN item_retention r ON r.item_id=i.id \
                 LEFT JOIN archived_images c ON c.item_id=i.id AND c.role='gallery' \
                   AND c.position=(SELECT MIN(position) FROM archived_images m \
                                   WHERE m.item_id=i.id AND m.role='gallery') \
                 ORDER BY s.id",
            )
            .map_err(db)?;
        stmt.query_map([psm_core::services::retention::INTEGRATION], |r| {
            let cover = match r.get::<_, Option<i64>>(9)? {
                Some(position) => Some(ArchivedImage {
                    role: psm_core::services::retention::ROLE_GALLERY.into(),
                    position,
                    media_id: r.get(10)?,
                    url: r.get(11)?,
                    bytes: r.get(12)?,
                }),
                None => None,
            };
            Ok(psm_core::mechanisms::remote::RemoteCover {
                session_id: r.get(0)?,
                item_id: r.get(1)?,
                sku: r.get(2)?,
                input: RemoteInput {
                    cleared: r.get::<_, Option<bool>>(3)?.unwrap_or(false),
                    linked: r.get(4)?,
                    adopted: r.get(5)?,
                    photo_rows: r.get::<_, i64>(6)? as usize,
                    checked: r.get::<_, Option<bool>>(7)?.unwrap_or(false),
                    pictures: r.get::<_, i64>(8)? as usize,
                },
                cover,
                gallery: r.get::<_, i64>(13)? as usize,
                condition: r.get::<_, i64>(14)? as usize,
            })
        })
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)
    }
    fn archived_images(&self, item_id: i64) -> Result<Vec<ArchivedImage>> {
        let conn = self.locked()?;
        let mut stmt = conn
            .prepare("SELECT role,position,media_id,url,bytes FROM archived_images WHERE item_id=?1 ORDER BY role<>'gallery',position")
            .map_err(db)?;
        stmt.query_map([item_id], |r| {
            Ok(ArchivedImage {
                role: r.get(0)?,
                position: r.get(1)?,
                media_id: r.get(2)?,
                url: r.get(3)?,
                bytes: r.get(4)?,
            })
        })
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)
    }
    fn cleared_sessions(&self) -> Result<Vec<i64>> {
        let conn = self.locked()?;
        let mut stmt = conn
            .prepare("SELECT s.id FROM sessions s JOIN item_retention r ON r.item_id=s.item_id WHERE r.cleared_at IS NOT NULL ORDER BY s.id")
            .map_err(db)?;
        stmt.query_map([], |r| r.get(0))
            .map_err(db)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db)
    }
    fn pending_denials(&self) -> Result<Vec<psm_core::domain::storage::PhotoDenial>> {
        let conn = self.locked()?;
        let mut stmt = conn
            .prepare("SELECT photo_id,denied_at,files_deleted_at FROM photo_denials WHERE files_deleted_at IS NULL ORDER BY denied_at,photo_id")
            .map_err(db)?;
        stmt.query_map([], denial_row)
            .map_err(db)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db)
    }
    fn photo_denial(
        &self,
        photo_id: i64,
    ) -> Result<Option<psm_core::domain::storage::PhotoDenial>> {
        let conn = self.locked()?;
        conn.query_row(
            "SELECT photo_id,denied_at,files_deleted_at FROM photo_denials WHERE photo_id=?1",
            [photo_id],
            denial_row,
        )
        .optional()
        .map_err(db)
    }
    fn mark_denial_files_deleted(&self, photo_id: i64, at: i64) -> Result<()> {
        let conn = self.locked()?;
        conn.execute(
            "UPDATE photo_denials SET files_deleted_at=?1 WHERE photo_id=?2",
            params![at, photo_id],
        )
        .map_err(db)?;
        Ok(())
    }
    fn session_research_running(&self, session_id: i64) -> Result<bool> {
        let conn = self.locked()?;
        super::session_delete::research_running(&conn, session_id)
    }
    fn delete_session_rows(
        &self,
        session_id: i64,
    ) -> Result<psm_core::domain::storage::DeletedRows> {
        let mut conn = self.locked()?;
        let tx = conn.transaction().map_err(db)?;
        let deleted = super::session_delete::delete_session_rows(&tx, session_id)?;
        tx.commit().map_err(db)?;
        Ok(deleted)
    }

    fn relocate_raw(&self, old: &str, new: &str, cache_dirs: &[(String, String)]) -> Result<()> {
        let mut conn = self.locked()?;
        let tx = conn.transaction().map_err(db)?;
        let taken: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM photos WHERE raw_path=?1)",
                [new],
                |r| r.get(0),
            )
            .map_err(db)?;
        if taken {
            return Err(Error::InvalidData(format!(
                "{new} is already the file of another photo"
            )));
        }
        // A leftover state row of a RAW no photo names any more must not block the key.
        tx.execute("DELETE FROM raw_state WHERE raw_path=?1", [new])
            .map_err(db)?;
        tx.execute(
            "UPDATE photos SET raw_path=?2 WHERE raw_path=?1",
            params![old, new],
        )
        .map_err(db)?;
        for (from, to) in cache_dirs {
            let prefix = format!("{from}/");
            for column in [
                "original_preview_path",
                "adjusted_preview_path",
                "thumbnail_path",
            ] {
                tx.execute(
                    &format!("UPDATE photos SET {column}=?3 || substr({column}, length(?2)) WHERE raw_path=?1 AND substr({column}, 1, length(?2))=?2"),
                    params![new, prefix, to],
                )
                .map_err(db)?;
            }
        }
        tx.execute(
            "UPDATE raw_state SET raw_path=?2, path_key=?3 WHERE raw_path=?1",
            params![old, new, psm_core::mechanisms::arrival::path_key(new)],
        )
        .map_err(db)?;
        tx.execute(
            "UPDATE session_changes SET raw_path=?2 WHERE raw_path=?1",
            params![old, new],
        )
        .map_err(db)?;
        tx.execute(
            "UPDATE jobs SET target=?2 WHERE target=?1",
            params![old, new],
        )
        .map_err(db)?;
        tx.commit().map_err(db)?;
        Ok(())
    }
}

fn denial_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<psm_core::domain::storage::PhotoDenial> {
    Ok(psm_core::domain::storage::PhotoDenial {
        photo_id: r.get(0)?,
        denied_at: r.get(1)?,
        files_deleted_at: r.get(2)?,
    })
}

impl SqliteStore {
    fn locked(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))
    }
}

/// Capture ledger (10E MC-33, P44).
impl SqliteStore {
    fn ledger_insert_exposure(&self, e: &NewExposure) -> Result<i64> {
        let conn = self.locked()?;
        insert_exposure_row(&conn, e)
    }
    fn ledger_exposures(&self, status: Option<ExposureStatus>) -> Result<Vec<Exposure>> {
        let conn = self.locked()?;
        let mut stmt = conn
            .prepare("SELECT id,capture_id,exposed_at,initiated,session_id,status,expected_files,created_at,shot_bound,shot_container_id FROM camera_exposures WHERE ?1 IS NULL OR status=?1 ORDER BY exposed_at,id")
            .map_err(db)?;
        stmt.query_map([status.map(ExposureStatus::as_str)], exposure_row)
            .map_err(db)?
            .collect::<rusqlite::Result<_>>()
            .map_err(db)
    }
    fn ledger_completed_names_since(&self, since_ms: i64) -> Result<Vec<(String, u64, i64)>> {
        let conn = self.locked()?;
        let mut stmt = conn
            .prepare("SELECT camera_filename,size,completed_at FROM camera_deliveries WHERE state='done' AND completed_at>=?1 ORDER BY completed_at,id")
            .map_err(db)?;
        let rows: Vec<(String, i64, i64)> = stmt
            .query_map([since_ms], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(db)?
            .collect::<rusqlite::Result<_>>()
            .map_err(db)?;
        let mut latest: Vec<(String, u64, i64)> = Vec::new();
        for (name, size, at) in rows {
            latest.retain(|(known, _, _)| known != &name);
            latest.push((name, size.max(0) as u64, at));
        }
        Ok(latest)
    }
}
