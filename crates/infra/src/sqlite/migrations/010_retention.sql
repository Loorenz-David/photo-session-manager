-- P62 (10F §3, Q2, Q3): an item's "keep files on this Mac" pin and when its local files were
-- cleared after publishing; the published images Shopify held when it was verified, shown from
-- then on. New tables only (`IF NOT EXISTS`: a test that rewinds `user_version` re-runs this).
CREATE TABLE IF NOT EXISTS item_retention (
    item_id INTEGER PRIMARY KEY REFERENCES items(id),
    keep_files INTEGER NOT NULL DEFAULT 0,
    cleared_at INTEGER
);
CREATE TABLE IF NOT EXISTS archived_images (
    item_id INTEGER NOT NULL REFERENCES items(id),
    role TEXT NOT NULL,
    position INTEGER NOT NULL,
    media_id TEXT NOT NULL,
    url TEXT NOT NULL,
    PRIMARY KEY (item_id, role, position)
);
