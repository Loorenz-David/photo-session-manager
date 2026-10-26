---
phase: P16_operations
title: Operations — settings and health screens, backup, logs, secrets, final build
difficulty: normal
projection: no
depends_on: [P15_publish_ui]
---
# P16 — Operations: settings and health screens, backup, logs, secrets, final build

## Goal
Close the MVP's operational loop (ch12): a settings screen over `settings.json` (document, recipes, style, export
folder, preview delay, theme, integration: shop domain, client ID, client secret masked, condition-photos key; 09A MC-23/MC-24), a health screen
(camera, document, hooks, recipes, capture folder, Shopify token obtainable, condition-photos field found), SQLite backup at start and after every completed publish run (30 kept), daily
log rotation with provider-call durations and run ids on mutations, the secrets rule proven, and the final ad-hoc
signed build with the operational measurements recorded. **Not in this phase:** Keychain (post-MVP), auto-update,
the phone remote.

## Read first
master plan §6.3, §6.8, §7.2 (build command), §3 (secrets) · book `01` (rule 8 via H-constraints context), `14`
(operational measurements paragraph), `12` (all), `13` (only-if-cheap list, to know what not to build) · contracts
`10A` MC-17 (delay clamp), `07A` MC-10 (adapter logs the run id with every mutation) · evidence `design_handoff.md`
§Not in the handoff (settings and health screens are undesigned: plain shadcn forms).

## Files expected to change
`crates/infra/src/{backup.rs (new), logging.rs (rotation, redaction), settings_file.rs (`set`, validation)}`,
`crates/core/src/services/{settings.rs (new), health.rs (integration check via `CommerceAdapter::check_token`, and
`condition_field_missing` from the stored definitions)}`, `crates/core/src/ports/commerce.rs` (`check_token`),
`crates/integrations/src/commerce/shopify/adapter.rs` (`check_token` = obtain a token through P12's
`ClientCredentialsAuth`, then a `shop { name }` query; errors are the 09A MC-23 ones), `apps/desktop/src-tauri/src/commands/settings.rs` (new: `settings_get`, `settings_set`),
bindings, `apps/desktop/src/features/{settings/SettingsPage.tsx, health/HealthPage.tsx (complete), shell/Header.tsx (gear)}`,
`apps/desktop/src/services/settings.ts`, `docs/intention/evidence/packaging_spike.md` (final build + measurements appended),
`crates/testkit/tests/p16_operations.rs` (new), `apps/desktop/src/features/settings/__tests__/p16_settings.test.tsx`,
`crates/integrations/src/editing/capture_one/jxa/find_image_in_folder.js` (task 0).

## Core mechanisms (unit tests allowed)
- `psm_infra::backup::prune(dir, keep = 30)`.
- `psm_infra::logging::redact(line, secrets) -> String`.

## Ordered tasks
0. **Capture One folder lookup (VERIFY-11, live 2026-10-26).** On the real app `doc.images whose path` searches only
   the collection being browsed, and `collections.whose({file: Path(...)})` raises -1725 "Illegal logical operator",
   so `find_image_in_folder.js` (the 10A MC-19 fallback the adapter uses when `find_image` misses) always errors.
   Rewrite it to iterate `d.collections()` and pick the first whose `file()` (when not null) `.toString()` equals the
   folder argument after canonicalisation (strip a leading `/private`, drop a trailing `/`), then
   `images.whose({path: raw})` on that collection and the position-1 variant as today. It must not change
   `currentCollection`. No change to `find_image.js` or the adapter order. Evidence: `capture_one_16.8.6.md`
   "VERIFY-11 result".
1. `SettingsService::get()` (`client_secret` masked as `••••` + last 4; shop domain, client ID and
   `condition_photos_key` shown as stored) and `set(patch)` (validates `preview_delay_s` 10–30 on
   write? **No** — stored as given, clamped on read per MC-17; validates `export_folder` is absolute); file rewritten
   atomically with mode 0600; `health_changed` after a change that touches providers (re-run `EditingStartup`).
2. Backup: `backup/session_manager-<ts>.db` via SQLite online backup API at `App::open` and when a publish item reaches
   `done`; keep the newest 30.
3. Logging: `tracing-appender` daily rotation into `logs/`; a redaction layer that replaces the configured client secret
   and any access token wherever they appear; provider calls log `duration_ms`; adapter mutations log `run_id`.
4. Screens: Settings (form), Health (items with codes and detail; "Retry checks").
5. Final: `npm run tauri -- build --bundles app`; launch the bundle; record cold start time (3 runs) and, if the camera
   releases, shot-to-original-preview and shot-to-adjusted-preview (5 frames) in `packaging_spike.md`.
6. Tests.

## Criteria (6 criteria, 12 rows; 8 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `settings_set({preview_delay_s: 40, export_folder: "/tmp/x"})` then `settings_get` | — | `settings.json` holds `40`; `settings_get` returns `preview_delay_s: 40` and `effective_preview_delay_s: 30`; file mode `0600` | MC-17 (clamp on read); `[rule]` ch12 §Settings |
| C1(b) | `settings_set({integration: {…, client_secret: "S-1234"}})` then `settings_get` | — | file holds `S-1234`; the DTO's `client_secret` is `"••••1234"`; `settings_set` with the masked value leaves the stored secret unchanged | `[rule]` ch12 (credentials stored 600), 09A MC-23 |
| C2(a) | `App::open` | fresh dir | one file `backup/session_manager-*.db` that opens as SQLite | `[rule]` ch12 §Backup |
| C2(b) | a publish item reaches `done` | — | a second backup file | `[rule]` ch12 |
| C2(c) | `backup::prune` via open | 31 pre-existing backup files | 30 remain: the newest 30 by name | `[rule]` ch12 (last 30 kept) |
| C3(a) | `health_get` (Rust) | fake document missing, fake commerce `check_token → Err(app_not_installed)`, configured `condition_photos_key = custom.nope` with no `media_list` definition stored | items `document_not_open`, `integration_token_invalid` (detail "The Session Manager app is not installed on this store") and `condition_field_missing` (detail names `custom.nope`, "not found") with `ok = false`; the rest `ok = true` | `[rule]` ch12 §Health |
| C3(b) | `HealthPage` render (frontend) | the DTO of C3(a) | the two red items with their detail text; "Retry checks" calls `health_get` again | `[rule]` ch12 |
| C4(a) | log capture around one editing-provider call | — | a log line with the operation name and `duration_ms` | MC-19 / ch10 (every provider call with duration) |
| C4(b) | log capture around a publish run | — | every adapter mutation line contains `run_id=<id>` | M5, MC-10 |
| C5(a) | log capture around a publish run with client secret `"SECRETTOKEN123"` and fake access token `"ACCESSTOKEN456"` | — | neither string ever appears in any log line or in any `capture_events`/`notifications` row | B3 (rule 8: secrets never logged) |
| C6(a) | `osascript -l JavaScript find_image_in_folder.js "/Users/davidloorenz/Pictures/session test 1" "/Users/davidloorenz/Pictures/session test 1/Capture/PSM_PROBE_A" "/Users/davidloorenz/Pictures/session test 1/Capture/PSM_PROBE_A/session test 12199.ARW"` against the running Capture One while it browses `Capture` | live frames from 2026-10-26 | prints `"130"`; Capture One still browses `Capture` afterwards; the script contains no `whose({file` | `[rule]` 10A MC-19 (live; the implementer runs it if its sandbox allows Apple events, else the reviewer runs it) |
| C5(b) | `settings.json` after any `settings_set` | — | mode `0600`; the repo tree contains no file with the client secret (test greps the temp data dir only; the reviewer greps the checkpoint diff) | B3 (rule 8) |

## Delegated choices
Settings form layout, whether the health screen is a page or a sheet, backup file naming beyond the pattern, log
format (JSON lines or key=value), how cold start is measured (launch → first `health_get` response).

## Manual try-out
1. Gear icon → Settings: change the preview delay to 20, pick another export folder; check the Shopify shop, client ID and condition-photos key
   (the secret shows masked); Save. Health shows the Shopify token as obtainable and the condition-photos field as found.
2. Look in `~/Library/Application Support/com.loorenz.sessionmanager/backup/`: one file per launch and per publish.
3. `logs/psm.<today>.log` shows Capture One calls with durations and, after a publish, lines with the run id — and
   never your client secret (`grep <secret> logs/*` finds nothing).
4. Build: `cd apps/desktop && npm run tauri -- build --bundles app`; copy `Session Manager.app` to the work Mac;
   right-click → Open.

## Notes
- Chapter 13 "only if cheap" items (tag read-back, capture button, rating display, Keychain, dark mode polish) are not
  criteria anywhere; an implementer may add one as a delegated choice only if it costs nothing and is recorded.
- Estimated size: ~12 files, ~1 200 lines.

## Review log
- 2026-10-26 (orchestrator): owner answers (book 09A, R32): settings show shop, client ID, masked secret and the
  condition-photos key; health checks token obtainability (09A MC-23 messages) and the condition-photos field; rows
  C1(b), C3(a), C5(a)/(b) reworded. Counts unchanged (11 rows).
- 2026-10-26 (orchestrator): live camera probe found the 10A MC-19 fallback script broken on the real Capture One
  (`collections.whose({file})` → -1725; `images whose path` limited to the browsed collection). Added task 0 and row
  C6(a) (12 rows). Evidence `capture_one_16.8.6.md` "VERIFY-11 result".
