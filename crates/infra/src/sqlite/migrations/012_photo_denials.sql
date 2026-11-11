-- P66 (changelog 46 (2)): a denied photo's RAW is deleted when the undo window closes. One row per
-- photo denied since P66 (a photo deleted before keeps its RAW, H2): when it was denied, and when its
-- files were deleted (then it can no longer be restored). Undo removes the row. New table only
-- (`IF NOT EXISTS`: a test that rewinds `user_version` re-runs this).
CREATE TABLE IF NOT EXISTS photo_denials (
    photo_id INTEGER PRIMARY KEY REFERENCES photos(id),
    denied_at INTEGER NOT NULL,
    files_deleted_at INTEGER
);
