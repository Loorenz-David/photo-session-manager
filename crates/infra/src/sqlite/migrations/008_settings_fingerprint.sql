-- P56: fingerprint of the adjustments in a RAW's Capture One settings sidecar when its preview was
-- last rendered or checked (10A MC-17 fallback; a rewrite without an adjustment change is no edit).
ALTER TABLE raw_state ADD COLUMN settings_fingerprint TEXT;
