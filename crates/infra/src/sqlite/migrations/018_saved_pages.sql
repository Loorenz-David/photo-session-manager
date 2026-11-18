-- Pages the owner saved in the "Search this photo" window (owner correction A12, 2026-11-18): one
-- row per item and address, in the order they were saved. They stay until the owner removes them,
-- so a later search for the same item shows them again. The page itself is never stored.
CREATE TABLE IF NOT EXISTS saved_pages (
  id INTEGER PRIMARY KEY,
  item_id INTEGER NOT NULL,
  url TEXT NOT NULL,
  title TEXT NOT NULL DEFAULT '',
  saved_at INTEGER NOT NULL,
  UNIQUE(item_id, url)
);
