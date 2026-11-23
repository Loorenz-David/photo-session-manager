//! `LabelStore` for `SqliteStore` (`master_plan_labels.md` §4 Storage): the one stored fact of
//! labels, `conversation_sync.labels_access` (whether the saved key may read labels), and the
//! read that joins sessions to their products' commerce links. No label state is ever written
//! here (M-L1).

use psm_core::{
    Error, Result,
    domain::label::{LabelsAccess, LinkedItem},
    ports::labels::LabelStore,
    services::attributes::LINK_INTEGRATION,
};
use rusqlite::{OptionalExtension, params};

use super::store::SqliteStore;

fn db(error: rusqlite::Error) -> Error {
    Error::InvalidData(error.to_string())
}

impl LabelStore for SqliteStore {
    fn labels_access(&self) -> Result<LabelsAccess> {
        self.research_transaction(|tx| {
            let stored: Option<String> = tx
                .conn()
                .query_row(
                    "SELECT labels_access FROM conversation_sync WHERE id=1",
                    [],
                    |r| r.get(0),
                )
                .optional()
                .map_err(db)?;
            Ok(stored
                .as_deref()
                .map_or(LabelsAccess::Unknown, LabelsAccess::from_stored))
        })
    }

    fn set_labels_access(&self, access: LabelsAccess) -> Result<()> {
        self.research_transaction(|tx| {
            let conn = tx.conn();
            conn.execute("INSERT OR IGNORE INTO conversation_sync(id) VALUES (1)", [])
                .map_err(db)?;
            conn.execute(
                "UPDATE conversation_sync SET labels_access=?1 WHERE id=1",
                [access.stored()],
            )
            .map_err(db)?;
            Ok(())
        })
    }

    fn session_products(&self, session_ids: &[i64]) -> Result<Vec<(i64, Option<String>)>> {
        self.research_transaction(|tx| {
            let conn = tx.conn();
            let mut stmt = conn
                .prepare(
                    "SELECT s.id, l.external_id FROM sessions s \
                     LEFT JOIN item_links l ON l.item_id = s.item_id AND l.integration = ?1 \
                     WHERE s.id = ?2",
                )
                .map_err(db)?;
            let mut out = Vec::with_capacity(session_ids.len());
            for id in session_ids {
                let row = stmt
                    .query_row(params![LINK_INTEGRATION, id], |r| {
                        Ok((r.get::<_, i64>(0)?, r.get::<_, Option<String>>(1)?))
                    })
                    .optional()
                    .map_err(db)?;
                if let Some((session_id, external)) = row {
                    out.push((session_id, external.filter(|e| !e.trim().is_empty())));
                }
            }
            Ok(out)
        })
    }

    fn linked_items(&self) -> Result<Vec<LinkedItem>> {
        self.research_transaction(|tx| {
            let conn = tx.conn();
            let mut stmt = conn
                .prepare(
                    "SELECT i.id, i.sku, \
                     (SELECT l.external_id FROM item_links l \
                      WHERE l.item_id = i.id AND l.integration = ?1 LIMIT 1), \
                     (SELECT MAX(s.id) FROM sessions s WHERE s.item_id = i.id) \
                     FROM items i ORDER BY i.id",
                )
                .map_err(db)?;
            let rows = stmt
                .query_map([LINK_INTEGRATION], |r| {
                    Ok(LinkedItem {
                        item_id: r.get(0)?,
                        sku: r.get(1)?,
                        external_id: r
                            .get::<_, Option<String>>(2)?
                            .filter(|e| !e.trim().is_empty()),
                        session_id: r.get(3)?,
                    })
                })
                .map_err(db)?;
            rows.collect::<rusqlite::Result<Vec<_>>>().map_err(db)
        })
    }
}
