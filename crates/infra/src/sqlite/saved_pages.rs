//! Pages saved in the "Search this photo" window (owner correction A12): per item, in the order
//! they were saved. Only the address and the title the page had are stored, never its content.

use psm_core::{Error, Result};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

use super::store::SqliteStore;

/// The longest address and title kept (longer ones are refused / cut).
pub const MAX_URL_CHARS: usize = 2_048;
pub const MAX_TITLE_CHARS: usize = 300;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedPage {
    pub id: i64,
    pub item_id: i64,
    pub url: String,
    pub title: String,
    pub saved_at: i64,
}

fn db(error: rusqlite::Error) -> Error {
    Error::InvalidData(error.to_string())
}

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<SavedPage> {
    Ok(SavedPage {
        id: r.get(0)?,
        item_id: r.get(1)?,
        url: r.get(2)?,
        title: r.get(3)?,
        saved_at: r.get(4)?,
    })
}

/// A saved page's address: an `http`/`https` address of at most [`MAX_URL_CHARS`].
fn checked_url(url: &str) -> Result<String> {
    let url = url.trim();
    let lower = url.to_ascii_lowercase();
    if !(lower.starts_with("https://") || lower.starts_with("http://")) {
        return Err(Error::InvalidValue(
            "only web pages (http or https) can be saved".into(),
        ));
    }
    if url.chars().count() > MAX_URL_CHARS {
        return Err(Error::InvalidValue("the page address is too long".into()));
    }
    Ok(url.to_owned())
}

fn cut_title(title: &str) -> String {
    title.trim().chars().take(MAX_TITLE_CHARS).collect()
}

impl SqliteStore {
    /// The item's saved pages, oldest first.
    pub fn saved_pages(&self, item_id: i64) -> Result<Vec<SavedPage>> {
        self.research_transaction(|tx| {
            let mut stmt = tx
                .conn()
                .prepare(
                    "SELECT id,item_id,url,title,saved_at FROM saved_pages WHERE item_id=?1 \
                     ORDER BY saved_at, id",
                )
                .map_err(db)?;
            let rows = stmt
                .query_map([item_id], row)
                .map_err(db)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(db)?;
            Ok(rows)
        })
    }

    /// Saves a page for the item. Saving an address the item already has keeps its place and
    /// takes the newer title. Returns the stored row.
    pub fn saved_page_add(
        &self,
        item_id: i64,
        url: &str,
        title: &str,
        now: i64,
    ) -> Result<SavedPage> {
        let url = checked_url(url)?;
        let title = cut_title(title);
        self.research_transaction(|tx| {
            let conn = tx.conn();
            let exists: Option<i64> = conn
                .query_row("SELECT id FROM items WHERE id=?1", [item_id], |r| r.get(0))
                .optional()
                .map_err(db)?;
            if exists.is_none() {
                return Err(Error::NotFound(format!("item {item_id}")));
            }
            conn.execute(
                "INSERT INTO saved_pages(item_id,url,title,saved_at) VALUES (?1,?2,?3,?4) \
                 ON CONFLICT(item_id,url) DO UPDATE SET title=CASE WHEN excluded.title<>'' \
                 THEN excluded.title ELSE saved_pages.title END",
                params![item_id, url, title, now],
            )
            .map_err(db)?;
            conn.query_row(
                "SELECT id,item_id,url,title,saved_at FROM saved_pages WHERE item_id=?1 AND url=?2",
                params![item_id, url],
                row,
            )
            .map_err(db)
        })
    }

    /// Removes one saved page; removing one that is gone is not an error.
    pub fn saved_page_remove(&self, id: i64) -> Result<()> {
        self.research_transaction(|tx| {
            tx.conn()
                .execute("DELETE FROM saved_pages WHERE id=?1", [id])
                .map_err(db)?;
            Ok(())
        })
    }
}
