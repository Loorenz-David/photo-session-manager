-- Conversations with a provider (book 09B; master_plan_conversations.md §4 Storage). The local copy
-- of the conversations addressed to this app, the sync state and the outbox of replies (C3). No
-- table names a session, an item or a photo: "Delete session" and file moves never touch them
-- (09B §10 rule 3). Times are UTC milliseconds.
CREATE TABLE IF NOT EXISTS conversation_sync (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  state TEXT NOT NULL DEFAULT 'no_key',
  store_id TEXT,
  store_name TEXT,
  shop_domain TEXT,
  server TEXT,
  visible_stores TEXT NOT NULL DEFAULT '[]',
  cursor INTEGER,
  first_sync_done INTEGER NOT NULL DEFAULT 0,
  last_ok_at INTEGER,
  last_error TEXT,
  last_error_at INTEGER
);
INSERT OR IGNORE INTO conversation_sync(id) VALUES (1);

CREATE TABLE IF NOT EXISTS conversations (
  id TEXT PRIMARY KEY,
  store_id TEXT NOT NULL,
  product_id TEXT NOT NULL,
  sku TEXT,
  product_title TEXT NOT NULL,
  topic TEXT NOT NULL,
  blocking INTEGER NOT NULL,
  status TEXT NOT NULL,
  resolved_by TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  last_activity_at INTEGER NOT NULL,
  read_mark INTEGER
);
CREATE INDEX IF NOT EXISTS conversations_sku ON conversations(sku);

CREATE TABLE IF NOT EXISTS conversation_messages (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  position INTEGER NOT NULL,
  author_kind TEXT NOT NULL,
  author_name TEXT NOT NULL,
  text TEXT NOT NULL,
  quote TEXT,
  reply_to TEXT,
  attachments TEXT NOT NULL DEFAULT '[]',
  created_at INTEGER NOT NULL,
  edited_at INTEGER
);
CREATE INDEX IF NOT EXISTS conversation_messages_order ON conversation_messages(conversation_id, position);

CREATE TABLE IF NOT EXISTS conversation_outbox (
  client_mutation_id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  text TEXT NOT NULL,
  reply_to TEXT,
  attachment_media_ids TEXT NOT NULL DEFAULT '[]',
  state TEXT NOT NULL,
  failure TEXT,
  server_message_id TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS conversation_outbox_order ON conversation_outbox(conversation_id, created_at);
