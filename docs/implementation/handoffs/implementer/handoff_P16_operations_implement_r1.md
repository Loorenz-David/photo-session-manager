---
plan: P16_operations
role: implement
round: 1
state: DONE
date: 2026-10-26
actor: opus (Claude, PSM_IMPLEMENTER=claude; continues the archived Codex attempt's uncommitted tree)
---
# P16_operations: implement r1 handoff

Gate: intention RATIFIED; P15_publish_ui APPROVED; P16 was NOT_STARTED. Passed.

## What was built
- **Settings screen** (gear icon in the header, `#/settings`). It is a plain form over `settings.json` with these fields: Capture One session folder (with a folder picker), preview recipe, web recipe, style, export folder (with a folder picker), and preview refresh delay with its value "in effect" (clamped 10–30 s, MC-17). The Shopify fields are shop domain, client ID, client secret and condition-photos key. The secret is shown masked as `••••` plus its last 4 characters, and saving the masked value keeps the stored secret.
  - Saving rewrites `settings.json` atomically with mode 0600. The running app then reopens over the new settings, so the document, recipes, export folder, preview delay and Shopify connection apply at once, and `health_changed` is emitted.
  - A relative export folder is refused (`relative_path`). A preview delay outside 10–30 is stored as given and clamped when read.
- **Health screen** (`#/health`, linked from Settings). It lists every check with its detail and a red or green dot, plus a **Retry checks** button. `health_get` re-runs these checks:
  - Capture One: document, hooks, recipes and capture folder (via `EditingStartup`).
  - Camera status.
  - Shopify token can be obtained: `check_token` fetches a token through `ClientCredentialsAuth`, then queries `shop { name }`. Errors carry the 09A MC-23 messages.
  - Condition-photos field: looked up in the stored definitions as the one `media_list`. The detail reads "Condition photos field `<key>` not found in the shop" or "… is not a file list".
- **Backups.** An SQLite online backup goes to `backup/session_manager-<yyyyMMdd-HHmmss>.db` at every app start and each time a publish item reaches `done`. The newest 30 are kept, by name.
- **Logs.** Logs go to one file per day, `logs/psm.<date>.log`. A redacting writer replaces the configured client secret and every fetched access token with `[REDACTED]`.
  - Each Capture One call logs `op=<script> duration_ms=…`.
  - Each Shopify call logs `provider="shopify" op=<operation> duration_ms=… ok=…`.
  - Each Shopify mutation logs `run_id=<id>` (already in place since P12, kept).
- **Capture One folder lookup fix (task 0, VERIFY-11).** `find_image_in_folder.js` now iterates `d.collections()` and compares each `file()` after canonicalisation (`/private` prefix and trailing `/` stripped). It does not use `whose({file…})` and does not change `currentCollection`.
- **Final build.** An ad-hoc signed `Session Manager.app` (22.49 MiB) was built and verified with `codesign --verify --deep --strict`. Cold start was measured 3 times and recorded in `packaging_spike.md`.

## Tests → criterion rows
All rows are in `crates/testkit/tests/p16_operations.rs` unless noted. Every test asserts the exact outcome at the row's boundary.

| Row | Test | Assertion shape |
|---|---|---|
| C1(a) | `c1a_settings_store_delay_as_given_and_read_it_clamped` | Calls `App::settings_set({preview_delay_s:40, export_folder:"/tmp/x"})` then `settings_get`. The file holds `40`; the DTO has `preview_delay_s 40` and `effective 30`; the file mode is `0600`. Exact. |
| C1(b) | `c1b_client_secret_is_masked_and_masked_save_keeps_it`, plus the UI test `p16_settings.test.tsx` "C1(b)…" | The file holds `S-1234` and the DTO shows `••••1234`. Shop, client ID and key are shown as stored. Saving the masked integration leaves `S-1234` in the file. The UI test shows the masked value in the field and sends it back unchanged. Exact. |
| C2(a) | `c2a_open_creates_readable_sqlite_backup` | Exactly one `session_manager-*.db`. It opens as SQLite and holds the Unassigned session row. Exact. |
| C2(b) | `c2b_publish_done_creates_second_backup` | One backup before the publish run and two after the item reaches `Done`. Exact. |
| C2(c) | `c2c_open_prunes_backups_to_newest_thirty` | 31 pre-existing files, then a restart through `App::open`. The remaining set equals the newest 30 by name (old `000002…000030` plus the start's own). Exact list. |
| C3(a) | `c3a_health_reports_document_token_and_field_failures` | `App::health_check()` (what `health_get` wraps) with the fake document missing, `check_token → app_not_installed` and key `custom.nope`. The failing codes are exactly `[document_not_open, integration_token_invalid, condition_field_missing]`, with the MC-23 detail and "Condition photos field custom.nope not found in the shop". Everything else is `ok`. |
| C3(b) | UI test `p16_settings.test.tsx` "C3(b)…" | The two Shopify items render with their detail text and `data-ok="false"`; an ok item renders `data-ok="true"`; "Retry checks" calls `getHealth` a second time. |
| C4(a) | `c4a_editing_provider_call_logs_operation_and_duration` | A real `osascript` run of a script named `inspect_document.js`. The log line has `op=inspect_document` and a numeric `duration_ms`. |
| C4(b) | `c4b_every_adapter_mutation_line_carries_the_run_id` | The real Shopify adapter over `FakeTransport`, one `item_create`. Every mutation or upload line contains `run_id=<the run's id>`, and there is at least one such line. |
| C5(a) | `c5a_secrets_never_reach_logs_events_or_notifications` | Client secret `SECRETTOKEN123` is configured through `settings_set`. Logs are captured through the production redacting writer around a full publish run plus a real-adapter token fetch (`ACCESSTOKEN456`) and mutation. Neither string appears in the log. `capture_events` and `notifications` are non-empty and contain neither string. |
| C5(b) | `c5b_settings_file_is_0600_and_the_secret_lives_nowhere_else` | After two `settings_set` calls the mode is `0600`. Scanning every file in the temp data dir finds the secret only in `settings.json`. The reviewer still greps the checkpoint diff. I grepped it for the owner's real secret (loaded from `~/.psm_shopify.env`, never printed) and found 0 hits. |
| C6(a) | Live run, plus the suite stand-in `c6a_folder_lookup_finds_variant_without_switching_collection` | **Live, run by me 2026-10-26 ~08:50** against the running Capture One while it browsed `Capture`: the exact command in the row printed `"130"` in 0.25 s. `currentCollection` read `Capture` before and after. `grep -c "whose({file"` returned 0. The suite test runs the real script under node with a simulated document (`/private` prefix and trailing `/` on the collection's file). It asserts `"130"`, no collection switch, and no `whose({file`. |

Core-mechanism unit tests: `psm_infra::backup::tests::prune_keeps_newest_names` and `psm_infra::logging::tests::redact_replaces_each_configured_secret`. No other tests were added; every test above maps to a row.

## Assumptions
1. Saving settings reopens the app's runtime: background workers stop and are rebuilt over the new `settings.json`, and the UI event target is kept. This makes the export folder, preview delay, recipes, document and Shopify settings take effect immediately. A theme-only change does not reopen. (Plan task 1 says "re-run EditingStartup", which alone would not apply the new document or recipes.)
2. That reopen does **not** write a backup. Only a real app start does, per ch12 "at every start".
3. The theme is not in the settings form. The header toggle stays the one theme control, and `sku_prefix` passes through untouched. ch12 does not list either as a form field.
4. The condition-field detail uses 09A MC-24's wording ("Condition photos field `<key>` not found in the shop" / "… is not a file list"). It names the key and says "not found", as the row asks.
5. Cold start is measured as launch → first window, with launch → backend open recorded alongside. The default screen (Sessions) never calls `health_get`, so "first `health_get` response" would need a navigation that is not part of a launch.
6. A backup happens once per publish **item** reaching `done`, which is what the plan row and the plan wording say. A multi-session run therefore makes one backup per session.
7. Health is a page (`#/health`), reached from Settings, and Settings is reached from the header gear.

## Deviations
- **Files outside the plan's list:**
  - `apps/desktop/src-tauri/src/commands/{health.rs, mod.rs}` and `src-tauri/src/lib.rs`: command registration, the `health_changed` event, and `health_get` → `App::health_check`.
  - `apps/desktop/src/App.tsx`: the `#/settings` route and a toaster on that route.
  - `crates/core/src/services/{mod.rs, editing_startup.rs}`.
  - `crates/infra/{Cargo.toml (rusqlite "backup" feature, chrono, tempfile dev), src/lib.rs, src/osascript.rs (op name on the provider-call line)}`.
  - `crates/integrations/src/lib.rs`: backup wiring, `health_check`, `settings_get/set`, `reopen`/`shutdown`.
  - `crates/integrations/src/commerce/shopify/auth.rs`: registers each fetched token for redaction.
  - `crates/testkit/{Cargo.toml, src/fake_commerce.rs}`: `check_token` and `token_app_not_installed`.
  - `Cargo.lock`.
- **P00's `capture_one` health item removed.** It was set together with `document_not_open`/`document` and only duplicated them, which would have made C3(a)'s "the rest ok = true" false. No test depended on it. The health page now shows its "checked at" time on the document row.
- **`settings_get`/`settings_set` are typed** (`SettingsDto_Serialize`, `SettingsPatchDto_Deserialize`, `IntegrationSettingsDto` in the bindings). The Codex attempt had passed JSON strings.
- The prompt template says actor `codex`. This session ran on Claude, so the tracker row and this handoff say `opus`.
- Scratch dirs `/tmp/psm-p16-cold-{1,2,3}` and three small `/tmp/psm_*.{js,py}` probe files remain. `rm -rf` of them was refused by the permission rules. They are outside the repo and harmless.

## Files changed
These files are in checkpoint `f18db80`:
- `Cargo.lock`
- `apps/desktop/src-tauri/src/commands/{health.rs, mod.rs, settings.rs (new)}`, `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src/App.tsx`
- `apps/desktop/src/features/health/HealthPage.tsx`, `apps/desktop/src/features/shell/Header.tsx`
- `apps/desktop/src/features/settings/{SettingsPage.tsx, __tests__/p16_settings.test.tsx}` (new)
- `apps/desktop/src/services/{bindings.ts, settings.ts (new)}`
- `crates/core/src/ports/commerce.rs`, `crates/core/src/services/{editing_startup.rs, health.rs, mod.rs, settings.rs (new)}`
- `crates/infra/Cargo.toml`, `crates/infra/src/{backup.rs (new), lib.rs, logging.rs, osascript.rs, settings_file.rs}`
- `crates/integrations/src/commerce/shopify/{adapter.rs, auth.rs}`, `crates/integrations/src/editing/capture_one/jxa/find_image_in_folder.js`, `crates/integrations/src/lib.rs`
- `crates/testkit/{Cargo.toml, src/fake_commerce.rs, tests/p16_operations.rs (new)}`
- `docs/intention/evidence/packaging_spike.md`
- `docs/implementation/master_plan.md` (P16 tracker row only)
- `docs/implementation/plans/P16_operations.md` (Review log entry)

No mutation probes were run (flow §4), so no production file was touched and reverted. There are no tool-recorded state deltas (no architecture graph).

## Checkpoint SHA
`f18db80` — `CHECKPOINT (not approved): P16_operations implement r1 — …`; the tree was clean after the commit.

## Stamp
Run on tree `f18db80` (identical working tree):
- `cargo test --workspace` → 387 passed, 0 failed (P15 approved at 374; +11 P16 behaviour tests, +2 core-mechanism unit tests).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all -- --check` → exit 0.
- `npm run typecheck` → exit 0.
- `npm test` → 7 files, 92 passed (+2).
- `cargo test -p psm-desktop export_bindings` regenerated `bindings.ts`, and the committed file equals the regenerated one.
- `npm run tauri -- build --bundles app` → `Session Manager.app` built, ad-hoc signed; `codesign --verify --deep --strict` OK.

No failures before or after this session.

**Live measurements.**
- C6(a) live: `"130"`.
- Cold start: run 1 (first launch after build) 0.61 s backend / 16.33 s window; run 2 0.36 / 0.64 s; run 3 0.34 / 0.57 s.
- Shot-to-preview (5 frames): **not measured.** Capture One reported `camera: null`, meaning the camera was not tethered.

## Manual try-out
1. `cd apps/desktop && npm run tauri -- build --bundles app`, then open `src-tauri/target/release/bundle/macos/Session Manager.app` (or run `npm run tauri -- dev`).
2. Click the gear icon in the header to open Settings:
   - Set the preview refresh delay to 20 and check that "In effect" shows 20 s.
   - Choose another export folder.
   - Enter or check the Shopify shop domain, client ID and condition-photos key. The secret shows as `••••` plus its last 4 characters.
   - Click Save. A "Settings saved" toast appears.
3. Click **Health** on the settings page. With your Shopify credentials it shows "Shopify token obtainable" and "Condition photos field custom.damage_report found" (after the app has read the shop's fields once). **Retry checks** runs everything again.
4. In Finder, open `~/Library/Application Support/com.loorenz.sessionmanager/backup/`. There is one file per launch and one more after each published session; at most 30 are kept.
5. `logs/psm.<today>.log` shows Capture One calls (`op=… duration_ms=…`). After a publish it shows lines with `run_id=…`. `grep '<your secret>' logs/*` finds nothing.
6. To measure shot-to-preview, tether the camera, open the session, fire 5 frames and note when the original and adjusted previews appear. This measurement is still open.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
