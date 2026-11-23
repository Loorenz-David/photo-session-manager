-- H1 (10H §3, §6, §7; master plan archive §4 Storage): when an item's pictures were brought back
-- and when its remote pictures were last read from the shop; each remote picture's original size.
-- `archived_images` stays the table of remote pictures for both kinds (archived, shop-only).
ALTER TABLE item_retention ADD COLUMN restored_at INTEGER;
ALTER TABLE item_retention ADD COLUMN remote_checked_at INTEGER;
ALTER TABLE archived_images ADD COLUMN bytes INTEGER;
