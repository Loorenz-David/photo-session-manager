//! The local copy of the conversations and the sync state (`master_plan_conversations.md` §4
//! Storage, §5 MC-C1): `ConversationStore` for `SqliteStore`. Each port method is one
//! transaction; the cursor is written in the same transaction as the change it follows.

use psm_core::{
    Error, Result,
    domain::conversation::{
        Attachment, Author, AuthorKind, ConnectionState, Conversation, ConversationStatus, Message,
        OutboxMessage, OutboxState, ProviderStore, Quote, ReplyRef, ResolvedBy,
    },
    ports::conversations::{ConversationStore, SyncRecord},
};
use rusqlite::{Connection, OptionalExtension, params};

use super::store::SqliteStore;

fn db(error: rusqlite::Error) -> Error {
    Error::InvalidData(error.to_string())
}

fn json_text<T: serde::Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|e| Error::InvalidData(e.to_string()))
}

fn from_json<T: serde::de::DeserializeOwned>(text: &str) -> Result<T> {
    serde_json::from_str(text).map_err(|e| Error::InvalidData(e.to_string()))
}

fn sync_read(conn: &Connection) -> Result<SyncRecord> {
    let row = conn
        .query_row(
            "SELECT state,store_id,store_name,shop_domain,server,visible_stores,cursor,\
             first_sync_done,last_ok_at,last_error,last_error_at FROM conversation_sync WHERE id=1",
            [],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, Option<i64>>(6)?,
                    r.get::<_, i64>(7)?,
                    r.get::<_, Option<i64>>(8)?,
                    r.get::<_, Option<String>>(9)?,
                    r.get::<_, Option<i64>>(10)?,
                ))
            },
        )
        .optional()
        .map_err(db)?;
    let Some((state, id, name, domain, server, visible, cursor, done, ok_at, error, error_at)) =
        row
    else {
        return Ok(SyncRecord {
            state: ConnectionState::NoKey,
            store: None,
            server: None,
            visible_stores: vec![],
            cursor: None,
            first_sync_done: false,
            last_ok_at: None,
            last_error: None,
            last_error_at: None,
        });
    };
    Ok(SyncRecord {
        state: ConnectionState::parse(&state).unwrap_or(ConnectionState::NoKey),
        store: id.map(|id| ProviderStore {
            id,
            name: name.unwrap_or_default(),
            shop_domain: domain.unwrap_or_default(),
        }),
        server,
        visible_stores: serde_json::from_str(&visible).unwrap_or_default(),
        cursor,
        first_sync_done: done != 0,
        last_ok_at: ok_at,
        last_error: error,
        last_error_at: error_at,
    })
}

fn ensure_sync_row(conn: &Connection) -> Result<()> {
    conn.execute("INSERT OR IGNORE INTO conversation_sync(id) VALUES (1)", [])
        .map_err(db)?;
    Ok(())
}

fn set_cursor(conn: &Connection, cursor: Option<i64>) -> Result<()> {
    if let Some(cursor) = cursor {
        ensure_sync_row(conn)?;
        conn.execute(
            "UPDATE conversation_sync SET cursor=?1 WHERE id=1",
            [cursor],
        )
        .map_err(db)?;
    }
    Ok(())
}

fn messages_read(conn: &Connection, conversation_id: &str) -> Result<Vec<Message>> {
    let mut stmt = conn
        .prepare(
            "SELECT id,author_kind,author_name,text,quote,reply_to,attachments,created_at,edited_at \
             FROM conversation_messages WHERE conversation_id=?1 ORDER BY position",
        )
        .map_err(db)?;
    let rows = stmt
        .query_map([conversation_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, i64>(7)?,
                r.get::<_, Option<i64>>(8)?,
            ))
        })
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)?;
    rows.into_iter()
        .map(
            |(id, kind, name, text, quote, reply_to, attachments, created_at, edited_at)| {
                Ok(Message {
                    id,
                    conversation_id: conversation_id.to_owned(),
                    author: Author {
                        kind: AuthorKind::parse(&kind).unwrap_or(AuthorKind::Other),
                        name,
                    },
                    text,
                    quote: quote.as_deref().map(from_json::<Quote>).transpose()?,
                    reply_to: reply_to.as_deref().map(from_json::<ReplyRef>).transpose()?,
                    attachments: from_json::<Vec<Attachment>>(&attachments)?,
                    created_at,
                    edited_at,
                })
            },
        )
        .collect()
}

type ConversationRow = (
    String,
    String,
    String,
    Option<String>,
    String,
    String,
    bool,
    String,
    Option<String>,
    i64,
    i64,
    Option<i64>,
);

const CONVERSATION_COLUMNS: &str = "id,store_id,product_id,sku,product_title,topic,blocking,status,\
     resolved_by,created_at,updated_at,read_mark";

fn conversation_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<ConversationRow> {
    Ok((
        r.get(0)?,
        r.get(1)?,
        r.get(2)?,
        r.get(3)?,
        r.get(4)?,
        r.get(5)?,
        r.get::<_, i64>(6)? != 0,
        r.get(7)?,
        r.get(8)?,
        r.get(9)?,
        r.get(10)?,
        r.get(11)?,
    ))
}

fn conversation_from(conn: &Connection, row: ConversationRow) -> Result<Conversation> {
    let (
        id,
        store_id,
        product_id,
        sku,
        product_title,
        topic,
        blocking,
        status,
        resolved_by,
        created_at,
        updated_at,
        read_mark,
    ) = row;
    let messages = messages_read(conn, &id)?;
    Ok(Conversation {
        store_id,
        product_id,
        sku,
        product_title,
        topic,
        blocking,
        status: ConversationStatus::parse(&status)
            .ok_or_else(|| Error::InvalidData(format!("conversation status {status}")))?,
        resolved_by: resolved_by.as_deref().and_then(ResolvedBy::parse),
        created_at,
        updated_at,
        read_mark,
        messages,
        id,
    })
}

fn conversation_read(conn: &Connection, id: &str) -> Result<Option<Conversation>> {
    let row = conn
        .query_row(
            &format!("SELECT {CONVERSATION_COLUMNS} FROM conversations WHERE id=?1"),
            [id],
            conversation_row,
        )
        .optional()
        .map_err(db)?;
    row.map(|row| conversation_from(conn, row)).transpose()
}

/// Writes `c` (row and messages) over whatever is stored.
fn conversation_write(conn: &Connection, c: &Conversation) -> Result<()> {
    let last_activity_at = c
        .messages
        .iter()
        .map(|m| m.created_at)
        .max()
        .unwrap_or(c.updated_at);
    conn.execute(
        "INSERT OR REPLACE INTO conversations(id,store_id,product_id,sku,product_title,topic,\
         blocking,status,resolved_by,created_at,updated_at,last_activity_at,read_mark) \
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
        params![
            c.id,
            c.store_id,
            c.product_id,
            c.sku,
            c.product_title,
            c.topic,
            i64::from(c.blocking),
            c.status.as_str(),
            c.resolved_by.map(ResolvedBy::as_str),
            c.created_at,
            c.updated_at,
            last_activity_at,
            c.read_mark,
        ],
    )
    .map_err(db)?;
    conn.execute(
        "DELETE FROM conversation_messages WHERE conversation_id=?1",
        [&c.id],
    )
    .map_err(db)?;
    for (position, m) in c.messages.iter().enumerate() {
        conn.execute(
            "INSERT OR REPLACE INTO conversation_messages(id,conversation_id,position,author_kind,\
             author_name,text,quote,reply_to,attachments,created_at,edited_at) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                m.id,
                c.id,
                position as i64,
                m.author.kind.as_str(),
                m.author.name,
                m.text,
                m.quote.as_ref().map(json_text).transpose()?,
                m.reply_to.as_ref().map(json_text).transpose()?,
                json_text(&m.attachments)?,
                m.created_at,
                m.edited_at,
            ],
        )
        .map_err(db)?;
    }
    Ok(())
}

fn outbox_state(text: &str) -> OutboxState {
    match text {
        "sent" => OutboxState::Sent,
        "failed" => OutboxState::Failed,
        _ => OutboxState::Sending,
    }
}

fn outbox_state_text(state: OutboxState) -> &'static str {
    match state {
        OutboxState::Sending => "sending",
        OutboxState::Sent => "sent",
        OutboxState::Failed => "failed",
    }
}

impl ConversationStore for SqliteStore {
    fn sync(&self) -> Result<SyncRecord> {
        self.research_transaction(|tx| sync_read(tx.conn()))
    }

    fn connect_store(
        &self,
        store: &ProviderStore,
        server: Option<&str>,
        reset: bool,
    ) -> Result<()> {
        self.research_transaction(|tx| {
            let conn = tx.conn();
            ensure_sync_row(conn)?;
            if reset {
                conn.execute_batch(
                    "DELETE FROM conversation_messages; DELETE FROM conversations; \
                     DELETE FROM conversation_outbox; \
                     UPDATE conversation_sync SET cursor=NULL, first_sync_done=0 WHERE id=1;",
                )
                .map_err(db)?;
            }
            conn.execute(
                "UPDATE conversation_sync SET state='ready', store_id=?1, store_name=?2, \
                 shop_domain=?3, server=?4, visible_stores='[]', last_error=NULL, \
                 last_error_at=NULL WHERE id=1",
                params![store.id, store.name, store.shop_domain, server],
            )
            .map_err(db)?;
            Ok(())
        })
    }

    fn store_unmatched(&self, visible: &[String]) -> Result<()> {
        let visible = json_text(&visible)?;
        self.research_transaction(|tx| {
            let conn = tx.conn();
            ensure_sync_row(conn)?;
            conn.execute(
                "UPDATE conversation_sync SET state='store_unmatched', store_id=NULL, \
                 store_name=NULL, shop_domain=NULL, visible_stores=?1 WHERE id=1",
                [visible],
            )
            .map_err(db)?;
            Ok(())
        })
    }

    fn record_error(&self, state: ConnectionState, error: &str, at: i64) -> Result<()> {
        self.research_transaction(|tx| {
            let conn = tx.conn();
            ensure_sync_row(conn)?;
            conn.execute(
                "UPDATE conversation_sync SET state=?1, last_error=?2, last_error_at=?3 WHERE id=1",
                params![state.as_str(), error, at],
            )
            .map_err(db)?;
            Ok(())
        })
    }

    fn finish_first_sync(&self, cursor: i64, now: i64) -> Result<()> {
        self.research_transaction(|tx| {
            let conn = tx.conn();
            ensure_sync_row(conn)?;
            conn.execute(
                "UPDATE conversation_sync SET cursor=?1, first_sync_done=1, last_ok_at=?2, \
                 last_error=NULL, last_error_at=NULL WHERE id=1",
                params![cursor, now],
            )
            .map_err(db)?;
            Ok(())
        })
    }

    fn set_cursor(&self, cursor: i64) -> Result<()> {
        self.research_transaction(|tx| set_cursor(tx.conn(), Some(cursor)))
    }

    fn apply(
        &self,
        conversation: &Conversation,
        entering_read_mark: Option<i64>,
        cursor: Option<i64>,
    ) -> Result<bool> {
        self.research_transaction(|tx| {
            let conn = tx.conn();
            let stored = conversation_read(conn, &conversation.id)?;
            let mut next = conversation.clone();
            next.read_mark = match &stored {
                Some(stored) => stored.read_mark,
                None => entering_read_mark,
            };
            let changed = stored.as_ref() != Some(&next);
            if changed {
                conversation_write(conn, &next)?;
            }
            set_cursor(conn, cursor)?;
            Ok(changed)
        })
    }

    fn delete_conversation(&self, id: &str, cursor: Option<i64>) -> Result<bool> {
        self.research_transaction(|tx| {
            let conn = tx.conn();
            let removed = conn
                .execute("DELETE FROM conversations WHERE id=?1", [id])
                .map_err(db)?;
            conn.execute(
                "DELETE FROM conversation_messages WHERE conversation_id=?1",
                [id],
            )
            .map_err(db)?;
            let outbox = conn
                .execute(
                    "DELETE FROM conversation_outbox WHERE conversation_id=?1 AND state<>'sent'",
                    [id],
                )
                .map_err(db)?;
            set_cursor(conn, cursor)?;
            Ok(removed + outbox > 0)
        })
    }

    fn complete_check(&self, now: i64) -> Result<()> {
        self.research_transaction(|tx| {
            let conn = tx.conn();
            ensure_sync_row(conn)?;
            conn.execute(
                "UPDATE conversation_sync SET last_ok_at=?1, last_error=NULL, last_error_at=NULL \
                 WHERE id=1",
                [now],
            )
            .map_err(db)?;
            Ok(())
        })
    }

    fn mark_read(&self, id: &str) -> Result<bool> {
        self.research_transaction(|tx| {
            let conn = tx.conn();
            let Some(stored) = conversation_read(conn, id)? else {
                return Ok(false);
            };
            let newest = stored.messages.iter().map(|m| m.created_at).max();
            if newest.is_none() || newest == stored.read_mark {
                return Ok(false);
            }
            conn.execute(
                "UPDATE conversations SET read_mark=?1 WHERE id=?2",
                params![newest, id],
            )
            .map_err(db)?;
            Ok(true)
        })
    }

    fn conversation(&self, id: &str) -> Result<Option<Conversation>> {
        self.research_transaction(|tx| conversation_read(tx.conn(), id))
    }

    fn conversations(&self) -> Result<Vec<Conversation>> {
        self.research_transaction(|tx| {
            let conn = tx.conn();
            let rows = {
                let mut stmt = conn
                    .prepare(&format!(
                        "SELECT {CONVERSATION_COLUMNS} FROM conversations ORDER BY created_at, id"
                    ))
                    .map_err(db)?;
                stmt.query_map([], conversation_row)
                    .map_err(db)?
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(db)?
            };
            rows.into_iter()
                .map(|row| conversation_from(conn, row))
                .collect()
        })
    }

    fn outbox_insert(&self, m: &OutboxMessage, now: i64) -> Result<()> {
        self.research_transaction(|tx| {
            tx.conn()
                .execute(
                    "INSERT INTO conversation_outbox(client_mutation_id,conversation_id,text,\
                     reply_to,attachment_media_ids,state,failure,server_message_id,created_at,\
                     updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                    params![
                        m.client_mutation_id,
                        m.conversation_id,
                        m.text,
                        m.reply_to.as_ref().map(json_text).transpose()?,
                        json_text(&m.attachment_media_ids)?,
                        outbox_state_text(m.state),
                        m.failure,
                        m.server_message_id,
                        m.created_at,
                        now,
                    ],
                )
                .map_err(db)?;
            Ok(())
        })
    }

    fn outbox(&self) -> Result<Vec<OutboxMessage>> {
        self.research_transaction(|tx| {
            let mut stmt = tx
                .conn()
                .prepare(
                    "SELECT client_mutation_id,conversation_id,text,reply_to,attachment_media_ids,\
                     state,failure,server_message_id,created_at FROM conversation_outbox \
                     ORDER BY created_at, client_mutation_id",
                )
                .map_err(db)?;
            let rows = stmt
                .query_map([], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, Option<String>>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, String>(5)?,
                        r.get::<_, Option<String>>(6)?,
                        r.get::<_, Option<String>>(7)?,
                        r.get::<_, i64>(8)?,
                    ))
                })
                .map_err(db)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(db)?;
            rows.into_iter()
                .map(
                    |(cmid, conversation_id, text, reply_to, media, state, failure, server, at)| {
                        Ok(OutboxMessage {
                            client_mutation_id: cmid,
                            conversation_id,
                            text,
                            reply_to: reply_to.as_deref().map(from_json::<ReplyRef>).transpose()?,
                            attachment_media_ids: from_json(&media)?,
                            state: outbox_state(&state),
                            failure,
                            server_message_id: server,
                            created_at: at,
                        })
                    },
                )
                .collect()
        })
    }
}
