-- 10D MC-29 §Links (P30): links are per output slot and record the composition revision their
-- output was rendered from. SQLite cannot alter a unique constraint, so the table is recreated.
-- `upload_state` gains the value 'retired' (a link whose slot the photo no longer has).
CREATE TABLE photo_links_v2 (
    id INTEGER PRIMARY KEY,
    photo_id INTEGER NOT NULL REFERENCES photos(id),
    integration TEXT NOT NULL,
    item_id INTEGER NOT NULL REFERENCES items(id),
    role TEXT NOT NULL,
    slot TEXT NOT NULL DEFAULT 'single',
    external_id TEXT,
    revision INTEGER NOT NULL DEFAULT 0,
    retouch_gen INTEGER NOT NULL DEFAULT 0,
    composition_rev INTEGER NOT NULL DEFAULT 0,
    superseded_ids TEXT,
    upload_state TEXT,
    export_sha256 TEXT,
    error TEXT,
    updated_at INTEGER NOT NULL,
    UNIQUE(photo_id, integration, item_id, role, slot)
);
INSERT INTO photo_links_v2 (id, photo_id, integration, item_id, role, slot, external_id, revision,
    retouch_gen, composition_rev, superseded_ids, upload_state, export_sha256, error, updated_at)
SELECT id, photo_id, integration, item_id, role, 'single', external_id, revision, retouch_gen, 0,
    superseded_ids, upload_state, export_sha256, error, updated_at
FROM photo_links;
DROP TABLE photo_links;
ALTER TABLE photo_links_v2 RENAME TO photo_links;
