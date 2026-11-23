-- H4 (10H §6 "Publish of details only"; master plan archive §4 Storage, §5 MC-H3.6): a publish item
-- confirmed for a remote item sends its details only; decided at confirmation, read by every run
-- and resume, never recomputed. Added with ALTER TABLE: `publish_items` carries research columns.
ALTER TABLE publish_items ADD COLUMN details_only INTEGER NOT NULL DEFAULT 0;
