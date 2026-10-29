---
phase: P21_backend_robustness
title: Backend robustness — the app always starts, saves never race, the shop and health tell the truth, no job is stranded
difficulty: high
projection: no
depends_on: [P20_drop_continuity]
---
# P21 — Backend robustness: the app always starts, saves never race, the shop and health tell the truth, no job is stranded

## Goal
Close the backend leftovers of `docs/implementation/backlog.md` before the move to the work Mac and the real store
(owner 2026-10-29: "let's correct the known small leftovers"). A hand-edited or damaged `settings.json`, an incomplete
integration block or an unwritable `backup/` folder never stops the app from starting: it falls back (defaults, Export
mode, no backup), logs, and shows a health item. Two settings saves in quick succession keep both changes. A client
secret is kept only when the exact masked value comes back; the settings read fills the integration defaults so the
settings screen loads from any valid block. A job whose kind has no handler is never left `running`. A shop outage is
reported as an error by the shop search instead of "0 hits". The health check reads the shop's fields before judging
the condition-photos field. The publish notification text no longer carries an epoch number, and an unchanged
description is not re-sent. The barcode can be cleared. **Not in this phase:** every frontend half (P22), H7
"shopify literal" refactors, migrations for the state homes (`publish_export:*`, retouch generation), logging init
order, test-only notes, anything post-MVP.

## Read first
`docs/implementation/backlog.md` rows cited per task (line numbers as of 2026-10-29: 7, 97, 101, 109, 118, 126, 127,
130, 132, 138, 139, 140, 84) · master plan §3 (standing rules), §6.4 (ports; `CommerceAdapter`), §6.5 (`jobs`,
`notifications`, `items` tables), §6.6 (error and health codes), §6.7 (commands are `pub async fn` over
`AppState::handles()`/`mutate()`), §6.8 (settings keys; `integration` block), §6.9 (test layout), §7.2 (commands),
§7.5 (`TestApp`, `FakeCommerce`) · book `01` (H1–H8), `14` (M7, M4/M18, M20, M29, M30), `12` §Settings, §Backup,
§Health screen, §Startup reconciliation, `07` §Publish steps (step 5: notification row "Published · time"), `07A`
MC-11 (resume; jobs stay queued and retry) and MC-13 (only changed values written), `09A` MC-23 (credentials; the
secret never leaves the settings file) and MC-24 (misconfiguration health message), `05B` MC-22 (the shop search
behind the SKU proposal), `05` §Item (barcode optional) · code: `crates/infra/src/settings_file.rs` (`Settings::open`
`:60`, the `from_slice` error `:76`, `integration_settings`), `crates/infra/src/jobs.rs` (`tick` `:33`, the unknown-kind
error `:43`), `crates/infra/src/backup.rs` (`create`), `crates/integrations/src/lib.rs` (`SettingsSetContext::prepare`
`:120`, `AppHandles::health_check`/`settings_get` right after it, `App::build` with `backup::create(&paths)?` `:308` and the
two runners `:386–387`, `commerce_from_settings` `:628`, `PublishJob::run` with `backup::create(&self.paths)?` `:755`),
`crates/core/src/services/settings.rs` (mask `:20`, `starts_with("••••")` `:63`), `crates/core/src/services/health.rs`
(`check_integration`, details `:40–48`), `crates/core/src/services/items.rs` (`search_shop` `:57`, `if let Ok(hits)`
`:71`; `update_identity` `:253`, `barcode.or(old.barcode…)` `:277`), `crates/core/src/services/publish.rs` (`Published ·`
text `:369`, condition note `:372`; description compare and `update_description` `:865–873`), `crates/core/src/ports/
commerce.rs` (`product_fields` `:37`, `update_description` `:116`), `crates/integrations/src/commerce/shopify/adapter.rs`
(`description_html` `:125`, `product_fields` `:550`, `update_description` `:1020`), `apps/desktop/src-tauri/src/state.rs`
(`AppState::settings_set` `:7`: `prepare` under the read lock, `mutate` under the write lock), `apps/desktop/src-tauri/
src/commands/settings.rs` (`IntegrationSettingsDto` `:9`), `apps/desktop/src-tauri/src/lib.rs` (`p17_tests` `:208–301`:
`AppState(RwLock::new(app))` over a `TestApp`, mock IPC) · tests to reuse: `crates/testkit/tests/p16_operations.rs`
(`stored`, `shopify`, `publish_one_session` `:122`, `c3a` `:175`), `p14_publish_update.rs` (helpers `:16–127`: `app`,
`session`, `photo`, `def`, `schema`, `value`, `drain`, `run`, `linked`, `latest`; `c3b` `:459`), `p17_real_run_fixes.rs`
(`c1c` `:92`: `TestApp::new()` + an integration block builds a real adapter without network), `p13_publish_runs.rs:589`
(asserts the old text) · `crates/testkit/src/test_app.rs` (`kill_and_restart` `:150`, `insert_publish_item` `:272`),
`crates/testkit/src/fake_commerce.rs` (`unreachable` `:39`, `read` `:113`, `item_find` `:177`, `product_fields` `:206`,
`attributes_schema` `:251`, `update_description` `:485` stores `description_html(paragraphs)`).

## Files expected to change
`crates/infra/src/settings_file.rs` (lenient open), `crates/infra/src/jobs.rs` (unknown kinds), `crates/integrations/src/lib.rs` (start fallbacks, health items, non-fatal backups, `settings_get` normalisation),
`crates/core/src/services/{settings.rs (exact mask compare), health.rs (schema refresh inside the check; "not read yet" detail), items.rs (search_shop error, barcode clear), publish.rs (notification text; description compare)}`,
`crates/core/src/ports/commerce.rs` (`render_description`), `crates/integrations/src/commerce/shopify/adapter.rs` (impl), `crates/testkit/src/fake_commerce.rs` (impl),
`apps/desktop/src-tauri/src/state.rs` (one save at a time), `apps/desktop/src-tauri/src/commands/settings.rs` (DTO defaults, optional), `apps/desktop/src-tauri/src/commands/items.rs` (doc of `barcode: ""`), `apps/desktop/src-tauri/src/lib.rs` (new test module `p21_tests`),
`crates/testkit/tests/p21_backend_robustness.rs` (new), `crates/testkit/tests/p13_publish_runs.rs` (`:589` asserts the new text), `crates/testkit/tests/p14_publish_update.rs` (only if `c4d` names the epoch; none expected),
`apps/desktop/src/services/bindings.ts` (regenerated; no signature changes, so no diff expected).

## Core mechanisms (unit tests allowed)
None.

## Ordered tasks
1. **Start always (backlog 7, 97, 130, 134).** `Settings::open` never fails on content: when `settings.json` exists but
   does not parse as `Settings`, it returns the defaults **in memory**, leaves the file untouched (bytes equal before
   and after), logs at `warn`, and reports the problem to the caller (`Settings::open_lenient(&paths) -> (Settings,
   Option<String /* problem */>)` or an equivalent, Delegated choices). `App::build` records that problem as health
   item `settings_invalid` (`ok = false`, detail starts with `settings.json: ` and carries the parser message). An
   `integration` block that is present but not a complete `IntegrationSettings` (`integration_settings()` → `Err`) no
   longer fails `build`: `commerce = None`, `publish = None` (Export mode, M7), health `settings_invalid` with detail
   `integration: <serde message>` (names the missing field), and `settings_get()` shows `integration: null` for that
   block. A failing start backup (`backup::create` → `Err`) is logged and recorded as health `backup_failed` (detail =
   the error text); `build` continues (`requeue_running_jobs` and the publish re-enqueue still run). The same
   non-fatal rule applies to the post-publish backup in `PublishJob::run` `:755`: the item stays `done`, the job
   completes, health `backup_failed` is set. A later successful backup clears `backup_failed`. The next `settings_set`
   writes a complete file from the in-memory settings (so saving from the Settings screen repairs a damaged file) and
   the rebuilt runtime has no `settings_invalid` item.
2. **One save at a time (backlog 138).** `AppState::settings_set` holds one save lock for the whole of prepare →
   apply → shutdown of the old runtime, so a second save starts from the settings the first one wrote. Shape
   delegated (`Mutex<()>` beside the `RwLock<App>`, or the `prepare` re-reading `settings.json` under the lock);
   `AppState(pub RwLock<App>)` may become a struct if a field is needed — the `p17_tests` constructor then follows.
3. **Secret kept only on the exact mask (backlog 132 backend half).** `SettingsService::set`: the stored client secret
   is kept **only** when the patch's `client_secret` equals the exact masked string `settings_get` showed
   (`••••` + the last four characters of the stored secret); any other value is stored as typed. (P22 clears the field
   on focus so typing replaces the mask; the two halves are independent.)
4. **Integration defaults on read (backlog 126).** `AppHandles::settings_get` (via `SettingsService::get`) shows the
   `integration` block normalised through `IntegrationSettings` (so `api_version` and `condition_photos_key` are always
   present, with their defaults when the file lacks them) before masking the secret; `IntegrationSettingsDto` may
   additionally take `#[serde(default = …)]` for the two keys (Delegated choices).
5. **No stranded jobs (backlog 139).** `JobRunner::tick`: a runner with **no** handler registered claims nothing and
   returns `Ok(0)`; a claimed job whose kind has no handler is returned to `queued` with `error = "unknown_job_kind"`
   and `next_attempt_at = now + 60 000` (through `job_retry`), is counted, and the loop continues with the next due
   job. No job is ever left `running` by the runner. (When the integration is removed, the publish runner has no
   handler; when it is re-entered, the rebuilt runner has one and the queued `publish_step` runs — 07A MC-11.)
6. **Honest shop search (backlog 140).** `ItemsService::search_shop` propagates `adapter.item_find` errors (`?`), so
   the `items_search_shop` command rejects (`CommandError.code = provider`) and the dialogs' existing `shop unavailable`
   heading appears. Everything else about `search_shop` (minus linked externals, minus SKUs already local) is
   unchanged; `search_local` never touches the adapter.
7. **Health reads the fields first (backlog 127).** `HealthService::check_integration` with an adapter: after the
   token check it refreshes the schema (`AttributesService::new(store, clock, Some(<integration>)).refresh_schema(
   adapter)` — the service needs a `Clock`; pass it in) and then evaluates the stored definitions as today. When the
   refresh **fails and nothing is stored** for the integration, the item is `condition_field_missing`, `ok = false`,
   detail exactly `Condition photos field <key> unknown: shop fields not read yet`. When the refresh fails but
   definitions are stored, the stored ones are evaluated (today's `found` / `not found` / `not a file list` details,
   09A MC-24). `p16 c3a` keeps its `not found` outcome (the fake's schema is empty there).
8. **Publish notification text and description compare (backlog 101/118 backend half, 109).** `complete_publish_item`
   text is `Published`, followed (when the condition field is missing and the item has condition photos, as today) by
   `. Condition photos not published: field <key> not found in the shop`; the time is the row's `created_at` (P22
   formats it: "Published · time", ch07 step 5). `CommerceAdapter` gains `fn render_description(&self, paragraphs:
   &str) -> String` (the text as the platform stores it; the Shopify adapter returns `description_html(paragraphs)`;
   the fake the same). `update` sends `update_description` only when `remote_description != Some(render_description
   (local))`. `p13_publish_runs.rs:589` follows the new text.
9. **Barcode can be cleared (backlog 84 backend half).** `ItemsService::update_identity(item, sku, barcode)`:
   `barcode = None` → unchanged; `Some("")` (after trim) → stored `NULL`; `Some(text)` → stored text. The command
   signature is unchanged (`Option<String>`); `item_update_identity`'s doc states the `""` rule. (P22 sends `""` when
   the field is emptied.)
10. Tests (§6.9: `crates/testkit/tests/p21_backend_robustness.rs`; the desktop row in `apps/desktop/src-tauri/src/lib.rs`
    `p21_tests`), `cargo test --workspace`, clippy, fmt, bindings regeneration (no diff expected), `npm run typecheck`,
    `npm test`.

## Criteria (7 criteria, 14 rows; 7 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `TestApp::new()`, then `settings.json` overwritten with `{ "preview_delay_s": ` (truncated), then `kill_and_restart()`; then `settings_get()`, `health_check()`, then `settings_set({"preview_delay_s": 20})` | the damaged file | `kill_and_restart` → `Ok`; `settings_get()["preview_delay_s"] == 15` and `["document_id"]` null; the health report has an item `settings_invalid`, `ok == false`, detail starting `settings.json: `; the file's bytes are unchanged after the restart and the health check; after the save: the file parses, `preview_delay_s == 20`, and a new `health_check()` has no `settings_invalid` item | `[rule]` ch12 §Settings (MC-17's "missing or not a number → 15 s" fallback, extended to an unreadable file) |
| C1(b) | `TestApp::new()`, `settings.json` rewritten as the defaults plus `"document_id": <the fake's document>` and `"integration": {"kind":"shopify","shop_domain":"x.myshopify.com"}`, `kill_and_restart()` | the incomplete block | `Ok`; `app.commerce.is_none()` and `app.publish.is_none()`; `settings_get()["document_id"]` is the document (other keys applied) and `["integration"]` is null; health item `settings_invalid`, `ok == false`, detail starting `integration: ` and containing `client_id` | M7 (the core workflow never depends on the integration); 09A MC-23 |
| C1(c) | `TestApp::with_fake_shop()`; `<data>/backup` directory deleted and replaced by a regular file; `kill_and_restart()`; then one session published to `done` (p16 `publish_one_session` pattern) | the unwritable backup location | `Ok`; health item `backup_failed`, `ok == false`, detail non-empty; the publish item's step is `Done` and its `publish_step` job `done`; `backup_failed` still present; no panic, no `Err` from any tick | `[rule]` ch12 §Backup, §Startup reconciliation |
| C2(a) | psm-desktop test: `AppState` over `TestApp::new()`'s app (the `p17_tests` constructor); four threads call `state.settings_set` at once with `{"preview_recipe":"R1"}`, `{"web_recipe":"W1"}`, `{"style_name":"S1"}`, `{"sku_prefix":"PX"}` | four concurrent non-theme saves | all four return `Ok`; afterwards `state.handles().settings_get()` shows `preview_recipe == "R1"`, `web_recipe == "W1"`, `style_name == "S1"`, `sku_prefix == "PX"`, and the stored `settings.json` holds the same four values | `[rule]` ch12 §Settings (a save never loses another save's change) |
| C2(b) | `app.settings_set({"integration": shopify("S-1234")})` then `settings_set({"integration": <shown block with client_secret "••••1234">})`, then with `"••••1234-typed"`, then with `"••••9999"` | p16 `shopify` fixture | after the second save the stored secret is `S-1234`; after the third it is `••••1234-typed`; after the fourth it is `••••9999` (only the exact masked string keeps the stored secret) | M30 / 09A MC-23 (the secret lives in the settings file only; a typed value is never silently dropped) |
| C2(c) | `TestApp::new()`, `settings.json` rewritten as the defaults plus `"integration": {"kind":"shopify","shop_domain":"x.myshopify.com","client_id":"ID","client_secret":"S-1234"}` (no `api_version`, no `condition_photos_key`), `kill_and_restart()`, `settings_get()` | a block without the two keys | `["integration"]["api_version"] == "2026-10"`, `["condition_photos_key"] == "custom.damage_report"`, `["client_secret"] == "••••1234"`, `["shop_domain"] == "x.myshopify.com"`; `app.commerce.is_some()` | `[rule]` ch12 §Settings; 09A MC-23/MC-24 defaults (master plan §6.8) |
| C3(a) | `JobRunner::new(store).only_kind("publish_step")` with **no** handler, over `TestApp::with_fake_shop()`'s store with one publish item (`insert_publish_item`) enqueued (`enqueue_publish_item(item, now)`); `tick(now)` twice | a queued `publish_step` and a handler-less runner | both ticks return `Ok(0)`; the job's `status` is `queued` after each (never `running`); `attempts` unchanged | M4 / 07A MC-11 (a publish resumes once a handler exists; no job is stranded) |
| C3(b) | `app.jobs.tick(now)` (the shared runner) after inserting through `rusqlite` one `jobs` row `kind='nope', status='queued', next_attempt_at=0` and, the same way, one row of a registered kind due at the same time (`kind='hash'`, `target` = an `add_photo` RAW path, `payload='{}'`) | an unknown kind beside a known one | `tick` returns `Ok(2)`; the `nope` row has `status='queued'`, `error='unknown_job_kind'`, `next_attempt_at == now + 60_000`; the `hash` row is `done` | `[rule]` 07A MC-11 (jobs stay queued and retry; the runner never leaves a job `running`) |
| C4(a) | `ItemsService::new(store, clock).with_commerce(Some(fake))` with `fake_shop().state.lock().unwrap().unreachable = true` | `search_shop("oak")`, then `search_local("oak")` | `search_shop` → `Err(Error::Provider(_))` (the command maps it to code `provider`); `search_local` → `Ok(vec![])`, and the fake's `reads` hold exactly one `item_find` | M29 / 05B MC-22 (no "0 in the shop" while the shop is unknown → no duplicate item); ch04 S-B |
| C5(a) | `TestApp::with_fake_shop()`; one session published to `done` (p16 `publish_one_session`); `store.notifications(Some(session))` | — | exactly one row: `kind == "published"`, `text == "Published"`, `created_at` between the confirmation time and `clock.now_ms()`; no digit in `text` | `[rule]` ch07 §Publish steps 5 (notification row "Published · time": the time is `created_at`, shown by P22) |
| C5(b) | p14 `app()`; `session`, one Main `photo`, `linked` product; `value(item, "product.description", "a\n\nb")`; `run` once; `value(item, "product.tags", ["x"])`; `run` again | two publishes, the description unchanged between them | the fake's `mutations` hold exactly one `update_description` in total; `descriptions[&id] == "<p>a</p><p>b</p>"`; the second run's `latest(..).step == Done` | M20 / 07A MC-13 (only changed values are written) |
| C6(a) | `items().create_local("B-1","Oak","Chair",Some("4006381333931"))`, then `update_identity(id, None, Some(""))`, then `update_identity(id, None, Some("123"))`, then `update_identity(id, None, None)` | — | after the first call `store.item(id).barcode == None`; after the second `Some("123")`; after the third still `Some("123")`; the SKU is `B-1` throughout | `[rule]` ch05 §Item (barcode optional and editable); backlog 84 |
| C7(a) | `TestApp::with_fake_shop()` (nothing stored, no startup refresh); `fake_shop().state.lock().unwrap().definitions = vec![def("custom.damage_report", MediaList, true)]` (p14 `def` shape); `health_check()` | a shop that has the field, an app that has not read it yet | the fake's `reads` contain `attributes_schema`; the report's `condition_field_missing` item has `ok == true`, detail `Condition photos field custom.damage_report found`; `store.attribute_definitions("shopify")` now holds the key | M30 / 09A MC-24 (no false "not found" before the first schema read) |
| C7(b) | `TestApp::with_fake_shop()`, `unreachable = true`, nothing stored → `health_check()`; then (second app) definitions stored by a successful `health_check()` as in (a), **then** `unreachable = true` → `health_check()` | an unreachable shop, before and after a first read | first: `condition_field_missing`, `ok == false`, detail exactly `Condition photos field custom.damage_report unknown: shop fields not read yet`; second: `ok == true`, detail `Condition photos field custom.damage_report found` (stored definitions still count) | 09A MC-24 (the misconfiguration message is only shown for a read schema) |

## Delegated choices
How `Settings::open` reports a problem (tuple, `SettingsProblem` struct, or a second constructor), the exact detail
wording beyond the required prefixes/substrings, the save lock's shape (`Mutex<()>` beside the `RwLock`, or a
`prepare` that re-reads the file under the lock), whether `IntegrationSettingsDto` also gets serde defaults, whether
the publish worker thread is still spawned when no store is configured (the runner guard makes it harmless), the
`render_description` default (none; both adapters implement it), where `backup_failed` is cleared (on the next
successful `backup::create`), and whether the schema refresh inside the health check logs its duration like other
provider calls (ch12 §Logs says yes).

## Manual try-out
1. Quit the app. Open `~/Library/Application Support/com.loorenz.sessionmanager/settings.json` in a text editor,
   delete the last `}` and save. Launch the app: it opens on the Sessions list. Settings → Health: a red "Settings
   file" row names `settings.json`. Back in Settings press "Save settings", then Health → "Retry checks": the row is
   gone and the file is valid again.
2. Quit again and delete the `"client_secret"` line of the `integration` block. Launch: the session header shows
   Export instead of Publish (Export mode) and Health shows the red settings row naming `client_id`/`client_secret`.
   Enter the secret in Settings and save: Publish is back without a relaunch.
3. In Settings, change the preview recipe, press Save, and immediately change the export folder and press Save
   again. Relaunch: both changes are there.
4. Turn Wi-Fi off. New session → type a title: the heading reads "… · shop unavailable", not "0 in the shop". Turn
   Wi-Fi on.
5. Open Health right after entering a store (or after a relaunch): the "Condition photos field" row is green, or
   says "shop fields not read yet" while offline — never a false "not found".
6. Publish one session twice without touching its description: `logs/psm.<today>.log` shows one
   `update_description` line in total. The bell's entry reads "Published" (P22 adds the formatted time).

## Notes
- **Verified before planning (2026-10-29), dropped from scope because P17/P18 already fixed them:** backlog 124
  (`PublishPopover.tsx:34` disables Cancel while `confirm.busy`), 125 / P00 note 1 / 103 (every command is `pub async
  fn` over `AppState::handles()` inside `spawn_blocking`; `health_check` runs on `AppHandles`, no app lock held — P17
  C4(a)/(b)), 129 (`SettingsSetContext::prepare` builds the new runtime first and restores the old file on failure;
  the old workers keep running — P17 task 3 and its handoff).
- **Order with P22.** This phase changes the stored notification text to `Published` (time = `created_at`); until
  P22 lands the drawer shows that bare word. The book's "Published · time" (ch07 step 5) is read as what the
  photographer sees; P22 renders `text · <local time>`. Rows that P22 completes: 101/118, 132, 84.
- **Why the description was re-sent (reading `publish.rs:865–873`):** the compare is between the shop's
  `descriptionHtml` and the plain local text; they never match. The renderer lives in the Shopify adapter
  (`adapter.rs:125`), so core asks the adapter to render (H7: platform rendering stays in the adapter).
- **Why `health_check` reports a false red (reading `health.rs`):** it reads only stored definitions; the startup
  refresh runs on its own thread and is absent when it failed. Reading the schema inside the check adds one GraphQL
  call to an action that already fetches a token (ch12 §Health), acceptable for an on-demand page.
- **C2(a) is a race row.** With the lock in place the outcome is deterministic; without it four overlapping rebuilds
  almost always lose at least one key (each `prepare` starts from the same snapshot). The reviewer judges by reading
  `state.rs`, not by repeating the run.
- No intention-book change is needed: every row traces to an existing contract or chapter rule. Health codes
  `settings_invalid` and `backup_failed`, the third `condition_field_missing` detail, the `render_description` port
  method and the `barcode: ""` rule are registry additions (master plan §6.4, §6.6, §6.7, §6.8), made by the planner.
- Estimated size: ~16 files, ~700 lines (half tests).

## Review log

- **2026-10-29, implement r1, codex.** Built the 14 planned boundary outcomes across settings,
  startup backup, jobs, shop search, health, publishing and barcode identity. Added 13 testkit
  cases and one desktop concurrency case; updated inherited P13 expectations for the new shop
  outage and notification semantics. Assumptions: malformed settings falls back to the complete
  defaults until a save; a schema refresh error retains stored definitions; an existing backup
  health failure survives a settings runtime rebuild until a successful backup. Deviation:
  `crates/infra/src/paths.rs` also changed because its eager `backup/` creation rejected the
  regular-file fixture before backup handling could report the failure. Bindings regenerated
  with the documented empty-barcode rule. No camera or live Shopify probe was needed.
- 2026-10-29, review r1 (opus): **APPROVED_WITH_NOTES.** Perimeter: `git show --stat e33ddfb` = the 17 declared paths (plan list minus the optional `commands/settings.rs`, plus the disclosed `crates/infra/src/paths.rs`). Reviewer re-ran the stamp on the clean checkpoint tree (`6c5e3c4`, porcelain empty): `cargo test --workspace` 421 passed / 0 failed, clippy `-D warnings` and `fmt --check` exit 0, `npm run typecheck` exit 0, `npm test` 12 files / 114 passed. Verified correct by reading code + test per row: C1(a) `Settings::open_lenient` returns defaults and the parser message without writing; `build` records `settings_invalid` `settings.json: …`; the test checks the bytes after restart + health check, then a runtime save writes a full file and the rebuilt app has no item. C1(b) an `integration_settings()` error → commerce/publish `None` (the override is also refused), detail `integration: integration settings: missing field \`client_id\``; `settings_get` nulls the block. C1(c) start backup error → `backup_failed`, start continues (requeue/re-enqueue still run); `PublishJob` sets/clears the item instead of returning `Err`; the start half fails on pre-phase code (`kill_and_restart` was `Err`). C2(a) `AppState.save: Mutex<()>` is held across context read → `prepare` → `apply_settings` → `old.shutdown()`, so each save's context is the previous save's applied settings. C2(b) keep only when `secret == "••••" + last four chars of the stored secret` (same char-based tail as `get`); the three later saves are exact. C2(c) `settings_get` serialises the block through `IntegrationSettings` before masking. C3(a) a handler-less runner returns before `job_due` (nothing claimed); pre-phase code would claim and error. C3(b) unknown kind → `job_retry(id,"unknown_job_kind",now+60_000)`, counted, loop continues; `Ok(2)` and the `hash` row `done` asserted exactly. C4(a) `item_find(query)?`; `Error::Provider` maps to `provider` (`commands/items.rs:67`); `reads == ["item_find"]` proves local search does not call the adapter. C5(a) text `Published`, `created_at` bounded. C5(b) compare against `render_description(local)`; exactly one `update_description` across two runs (pre-phase compare would send twice). C6(a) `Some("")` → `NULL`, `None` keeps, SKU kept. C7(a)/(b) `check_integration` refreshes the schema, then distinguishes "not read yet" (refresh failed, nothing stored) from stored definitions. H1–H8: no platform write added (the health refresh is a read; the description compare removes writes); no RAW touched; H6 strengthened (an incomplete block → Export mode); H7: one new `"shopify"` literal in core `health.rs` (tracked family, out of scope by the plan); H8: the save lock lives in the desktop state container, the logic in services. No secret in the diff or in a health detail. Assumptions (defaults until a save; stored definitions survive a failed refresh; `backup_failed` survives a rebuild until a successful backup; a theme-only save clears a repaired-file warning) are consistent with ch12 and 09A. Reviewer probes (temporary test file `crates/testkit/tests/zz_review_p21_probe.rs`, deleted, tree clean afterwards): (1) a backup that works at start and breaks before publishing → job `done` at `attempts 0`, `backup_failed` set; (2) with an incomplete block, `settings_set({"preview_recipe":"R2"})` → `Err("invalid data: integration settings: missing field \`client_id\`")`; with a full block in the same patch → `Ok`, commerce back. Notes (backlog): (1) any save that does not carry a full block (incl. the theme toggle) fails while the stored block is incomplete; (2) C1(c)'s publish half cannot fail on a fatal post-publish backup (the 1 s retry re-runs with `was_done` true and ends `done`; the start failure already holds the item); probe (1) covers it; (3) H7 literal in `health.rs`; (4) the test-only merged `ItemsService::search` now fails whole on an outage.
