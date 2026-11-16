CREATE TABLE IF NOT EXISTS photo_places (
  photo_id INTEGER PRIMARY KEY REFERENCES photos(id),
  slot INTEGER,
  place_by_hand INTEGER NOT NULL DEFAULT 0,
  frame_by_hand INTEGER NOT NULL DEFAULT 0,
  released INTEGER NOT NULL DEFAULT 0,
  key_at INTEGER,
  key_name TEXT,
  pending_zoom INTEGER
);
