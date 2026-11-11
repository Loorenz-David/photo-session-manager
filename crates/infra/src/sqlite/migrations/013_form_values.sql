-- P67 (changelog 46 (3), (4)): saved values per (item type, field key), most recent first, and the
-- product title last read from an integration (the form labels the title "from Shopify" while the
-- item's title still equals it). New tables only (`IF NOT EXISTS`: a test that rewinds
-- `user_version` re-runs this).
CREATE TABLE IF NOT EXISTS attribute_saved_values (
    item_type TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    saved_at INTEGER NOT NULL,
    PRIMARY KEY (item_type, key, value)
);
CREATE TABLE IF NOT EXISTS item_remote_titles (
    item_id INTEGER NOT NULL REFERENCES items(id),
    integration TEXT NOT NULL,
    title TEXT NOT NULL,
    PRIMARY KEY (item_id, integration)
);
