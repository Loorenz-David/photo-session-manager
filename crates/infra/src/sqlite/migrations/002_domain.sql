CREATE TABLE items (
    id INTEGER PRIMARY KEY,
    sku TEXT NOT NULL UNIQUE,
    folder_key TEXT NOT NULL,
    folder_key_ci TEXT NOT NULL UNIQUE,
    barcode TEXT,
    item_type TEXT NOT NULL,
    title TEXT NOT NULL,
    origin TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE TABLE item_links (
    item_id INTEGER NOT NULL REFERENCES items(id),
    integration TEXT NOT NULL,
    external_id TEXT NOT NULL,
    external_url TEXT,
    last_synced_at INTEGER,
    create_pending INTEGER NOT NULL DEFAULT 0,
    tags_base TEXT,
    PRIMARY KEY(item_id, integration)
);
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY,
    item_id INTEGER REFERENCES items(id),
    published_at INTEGER,
    published_seq INTEGER NOT NULL DEFAULT 0,
    change_seq INTEGER NOT NULL DEFAULT 0,
    last_activity_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    undo_slot TEXT
);
CREATE UNIQUE INDEX sessions_one_unassigned ON sessions(item_id) WHERE item_id IS NULL;
INSERT INTO sessions(id, item_id, last_activity_at, created_at) VALUES (1, NULL, 0, 0);
CREATE TABLE photos (
    id INTEGER PRIMARY KEY,
    uid TEXT NOT NULL UNIQUE,
    session_id INTEGER NOT NULL REFERENCES sessions(id),
    raw_path TEXT NOT NULL,
    raw_hash TEXT,
    integrity TEXT NOT NULL DEFAULT 'ok',
    provider_ref TEXT,
    original_preview_path TEXT,
    adjusted_preview_path TEXT,
    orientation INTEGER,
    classification TEXT NOT NULL DEFAULT 'unreviewed',
    position INTEGER,
    captured_at INTEGER,
    source TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE(session_id, raw_path)
);
CREATE TABLE raw_state (
    raw_path TEXT PRIMARY KEY,
    path_key TEXT,
    adjusted_gen INTEGER NOT NULL DEFAULT 0,
    rendered_gen INTEGER NOT NULL DEFAULT 0,
    retouch_gen INTEGER NOT NULL DEFAULT 0,
    last_own_write_at INTEGER
);
CREATE TABLE photo_links (
    id INTEGER PRIMARY KEY,
    photo_id INTEGER NOT NULL REFERENCES photos(id),
    integration TEXT NOT NULL,
    item_id INTEGER NOT NULL REFERENCES items(id),
    role TEXT NOT NULL,
    external_id TEXT,
    revision INTEGER NOT NULL DEFAULT 0,
    retouch_gen INTEGER NOT NULL DEFAULT 0,
    superseded_ids TEXT,
    upload_state TEXT,
    export_sha256 TEXT,
    error TEXT,
    updated_at INTEGER NOT NULL,
    UNIQUE(photo_id, integration, item_id, role)
);
CREATE TABLE attribute_definitions (
    integration TEXT NOT NULL,
    key TEXT NOT NULL,
    label TEXT NOT NULL,
    core_type TEXT NOT NULL,
    choices TEXT,
    validations TEXT,
    grp TEXT NOT NULL,
    description TEXT,
    writable INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY(integration, key)
);
CREATE TABLE attribute_definition_links (
    integration TEXT NOT NULL,
    key TEXT NOT NULL,
    platform_type TEXT NOT NULL,
    platform_meta TEXT,
    PRIMARY KEY(integration, key)
);
CREATE TABLE attribute_presets (
    item_type TEXT NOT NULL,
    grp TEXT NOT NULL,
    layout TEXT NOT NULL,
    PRIMARY KEY(item_type, grp)
);
CREATE TABLE attribute_values (
    item_id INTEGER NOT NULL REFERENCES items(id),
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    valid INTEGER NOT NULL DEFAULT 1,
    provenance TEXT NOT NULL,
    ai_confidence TEXT,
    ai_status TEXT,
    ai_reason TEXT,
    ai_sources TEXT,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY(item_id, key)
);
CREATE TABLE publish_runs (
    id INTEGER PRIMARY KEY,
    integration TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    confirmed_at INTEGER,
    confirmed_summary TEXT
);
CREATE TABLE publish_items (
    id INTEGER PRIMARY KEY,
    run_id INTEGER NOT NULL REFERENCES publish_runs(id),
    session_id INTEGER NOT NULL REFERENCES sessions(id),
    create_item INTEGER NOT NULL,
    title TEXT,
    frozen_main TEXT,
    frozen_condition TEXT,
    frozen_attributes TEXT,
    announced_removals TEXT,
    confirmed_change_seq INTEGER,
    step TEXT NOT NULL,
    failed_step TEXT,
    uploaded INTEGER NOT NULL DEFAULT 0,
    total INTEGER NOT NULL DEFAULT 0,
    error TEXT,
    finished_at INTEGER
);
CREATE TABLE jobs (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    target TEXT NOT NULL,
    payload TEXT NOT NULL,
    status TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    next_attempt_at INTEGER,
    error TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE TABLE notifications (
    id INTEGER PRIMARY KEY,
    session_id INTEGER REFERENCES sessions(id),
    run_id INTEGER REFERENCES publish_runs(id),
    kind TEXT NOT NULL,
    text TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
