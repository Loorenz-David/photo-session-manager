# 12 — Operations

- **Data directory:** the app's data directory under `~/Library/Application Support/` (Tauri
  app-data path for the bundle) holds the SQLite file, previews cache, logs and settings.
  Nothing runtime lives in the repo.
- **Settings:** edited in a settings screen; stored as a JSON file in the data directory.
  Integration credentials (Shopify client ID and secret, 09A MC-23) are stored there in the MVP with
  file permissions 600; Keychain after the MVP. The Shopify access token is never stored.
  Secrets are never logged or committed (rule 8).
- **Export folder:** chosen in settings; default `~/Pictures/Session Manager Export/`.
- **Preview refresh delay:** 15 s default, 10–30 s (chapter 10).
- **Logs:** structured, one rotating file per day; every provider call with duration; every
  integration mutation with the publish run id.
- **Backup:** SQLite copied to `backup/` at every start and after every completed publish run;
  last 30 kept.
- **Health screen / header:** camera, provider document open, hooks installed, recipes found,
  integration token can be obtained (09A MC-23 messages), condition-photos field found (09A MC-24). Notifications drawer lists publish history with Retry and Open.
- **Startup reconciliation:** scan the active document's capture folder against `photos`;
  resume incomplete publish items; re-install hooks.
