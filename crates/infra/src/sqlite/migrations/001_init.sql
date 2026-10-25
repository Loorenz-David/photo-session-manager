CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS capture_events (
    id INTEGER PRIMARY KEY,
    source TEXT NOT NULL,
    hook TEXT,
    payload TEXT NOT NULL,
    received_at INTEGER NOT NULL,
    error TEXT
);
INSERT OR IGNORE INTO settings(key, value) VALUES ('mechanisms_version', '1');

