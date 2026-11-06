-- 10E MC-33 R-I25 (P44): the capture ledger. One row per observed exposure and one per delivered
-- camera file; rows are kept. A delivery is written `pending` with its final path before the
-- rename and set `done` after it; `written_path` (the staging file) is what recovery and the
-- re-send take-over need. `missing_at_deadline` is reserved: nothing writes it in P44.
-- `IF NOT EXISTS`: a test that rewinds `user_version` re-runs this migration over its own tables.
CREATE TABLE IF NOT EXISTS camera_exposures (
    id INTEGER PRIMARY KEY,
    capture_id TEXT,
    exposed_at INTEGER NOT NULL,
    initiated TEXT NOT NULL CHECK (initiated IN ('app', 'camera')),
    session_id INTEGER,
    status TEXT NOT NULL CHECK (status IN ('expecting', 'received', 'missing_at_deadline', 'uncertain', 'dismissed')),
    expected_files INTEGER,
    created_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS camera_deliveries (
    id INTEGER PRIMARY KEY,
    exposure_id INTEGER,
    camera_filename TEXT NOT NULL,
    file_key TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    size INTEGER NOT NULL,
    via TEXT NOT NULL CHECK (via IN ('cable', 'wireless')),
    completed_at INTEGER NOT NULL,
    association TEXT NOT NULL CHECK (association IN ('filename', 'stem', 'fifo', 'none')),
    outcome TEXT NOT NULL CHECK (outcome IN ('photo', 'duplicate', 'renamed_photo')),
    session_id INTEGER,
    written_path TEXT NOT NULL,
    final_path TEXT NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('pending', 'done')),
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS camera_deliveries_file_key ON camera_deliveries(file_key);
CREATE INDEX IF NOT EXISTS camera_deliveries_final_path ON camera_deliveries(final_path);
CREATE INDEX IF NOT EXISTS camera_deliveries_written_path ON camera_deliveries(written_path);
CREATE INDEX IF NOT EXISTS camera_exposures_status ON camera_exposures(status, exposed_at);
