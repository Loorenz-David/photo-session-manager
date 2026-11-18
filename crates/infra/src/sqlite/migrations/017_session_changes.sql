-- Why a published session changed (owner correction A10, 2026-11-18). One row per change counted
-- on a session (`sessions.change_seq`), with the counter value it produced and its kind:
-- 'arrival' (a new photo, and its placement), 'editor' (a Capture One adjustment seen, raw_path set),
-- 'app' (an edit in the app: review, move, crop by hand, template action, undo), 'auto' (a crop the
-- app set itself). The rows with seq > published_seq say why the session is open again; rows at or
-- below a new publish are removed when it completes.
CREATE TABLE IF NOT EXISTS session_changes (
  id INTEGER PRIMARY KEY,
  session_id INTEGER NOT NULL,
  seq INTEGER NOT NULL,
  kind TEXT NOT NULL,
  raw_path TEXT,
  at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS session_changes_session ON session_changes(session_id, seq);
