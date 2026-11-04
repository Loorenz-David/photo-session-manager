CREATE TABLE compositions (
 photo_id INTEGER PRIMARY KEY REFERENCES photos(id),
 kind TEXT NOT NULL, shape TEXT NOT NULL,
 orientation TEXT NOT NULL DEFAULT 'named', quarter_turns INTEGER NOT NULL DEFAULT 0,
 flip TEXT NOT NULL DEFAULT 'none', straighten REAL NOT NULL DEFAULT 0,
 x REAL, y REAL, w REAL, h REAL,
 frame_w INTEGER, frame_h INTEGER, frame_source TEXT, geometry TEXT,
 origin TEXT NOT NULL DEFAULT 'default', revision INTEGER NOT NULL DEFAULT 0,
 state TEXT NOT NULL DEFAULT 'ok', review_reason TEXT, updated_at INTEGER NOT NULL
);
ALTER TABLE photos ADD COLUMN thumbnail_path TEXT;
