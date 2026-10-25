---
phase: P13_publish_runs
title: Publish runs — confirmation freeze, prepare, upload, gallery mirror, resume and retry
difficulty: high
projection: no
depends_on: [P09_local_export, P12_shopify_adapter]
---
# P13 — Publish runs: confirmation freeze, prepare, upload, gallery mirror, resume and retry

## Goal
Implement the publish service as the resumable step machine of MC-11 over `CommerceAdapter`: confirmation writes
the frozen run (MC-10, M17), `prepare` creates or adopts the product and exports (P09 path), `upload` drives each
link through its states with crash-safe adoption by filename stem (M18), `update` reconciles the gallery exactly
(MC-12, M19, M3) — attribute writes come in P14 — `done` records `published_at`/`published_seq` and a notification,
retry re-runs the failed step only, and startup resumes active items. Exposes `publish_preview`, `publish_confirm`,
`publish_retry`, `notifications_list` and the `publish_progress` event. **Not in this phase:** attribute writes,
damage-report files, retouch replacement (P14); any UI (P15).

## Read first
master plan §6.5 (`publish_runs`, `publish_items`, `photo_links`, `notifications`), §6.6 (`PublishStep`, `UploadState`),
§6.7, §7.5 (`FakeCommerce`) · book `01` (H1, H4), `14`, `07` (Publish, Failure and retry), `04` S-G, S-I, S-J, S-K ·
contracts `07A` MC-10, MC-11 (all but the MC-21 pointer), MC-12; `05A` MC-6 (`published_seq`), MC-7 (status inputs),
MC-3 (checks a), MC-4 (`last_activity_at` on done/failed) · evidence `shopify_admin.md` SV-2 (uniquifier), SV-4.

## Files expected to change
`crates/core/src/services/publish.rs` (new: `PublishService`, step handlers), `crates/core/src/mechanisms/reconcile.rs`
(new), `crates/core/src/domain/publish.rs` (`ConfirmedRun` constructor used only here; `PublishPreview` DTO),
`crates/core/src/services/{export.rs (reuse inside prepare), sessions.rs (status inputs from items)}`, Store trait + impl
(runs, items, links, notifications, resume queries), `crates/infra/src/jobs.rs` (`PublishStep` handler),
`apps/desktop/src-tauri/src/commands/publish.rs` (new), bindings, `crates/testkit/src/fake_commerce.rs` (new),
`crates/testkit/src/test_app.rs` (`fake_shop()`, `kill_and_restart()` = drop and re-open the `App` on the same data dir),
`crates/testkit/tests/p13_publish_runs.rs` (new).

## Core mechanisms (unit tests allowed)
- `mechanisms::reconcile::plan(d: &[MediaId], current: &[MediaId], own: &HashSet<MediaId>, announced: &HashSet<MediaId>) -> Result<Plan{deletes, moves}, ReconcileError::Foreign(n)>` (MC-12 steps 2–4).
- `mechanisms::publish::progress(step, uploaded, total)` already exists (P03); `publish_summary(preview) -> String` (the exact confirmation text).

## Ordered tasks
1. `publish_preview(session_ids) -> PublishPreview` (per session: main thumbnails in order, condition thumbnails after
   exclusions, details fill count, unreviewed count, `announced_removals` from `item_get` = current media not owned by
   this item, `create_item` flag) and `publish_summary` (text: "Publish N images …", the N2 sentence, the N1 count, the
   create note). `publish_confirm(session_ids, summary_text)`: one transaction → `publish_runs` + `publish_items` with
   the frozen columns of MC-10; returns run id; enqueues `PublishStep` jobs. `ConfirmedRun` is constructed only from a
   persisted run with `confirmed_at` (crate-private `ConfirmedRun::from_persisted`).
2. Step handler: commit `step` before work; `prepare` (SKU search 0/1/>1 rule, `item_links` written before any other
   call, `IntegrityCheck`, export via `ExportService` into the run's staging; `export_sha256` per link); `upload`
   (frozen main then condition; skip `deleted`/moved-away; `ready` links skipped; otherwise list media, adopt by stem,
   else stage → upload → create → poll; each transition committed after the remote effect; progress events); `update`
   = `reconcile::plan` executed (fail before any deletion on `Foreign`; delete; reorder; poll; read back equals `D`),
   P14 appends attribute writes after this; `describe` skipped; `done` → `published_at`, `published_seq =
   confirmed_change_seq`, notification "Published · <time>", `last_activity_at`.
3. Failure: `step = failed`, `failed_step`, `error`; `publish_retry(item)` → `step = failed_step` and re-enqueue.
   Startup: `running` jobs → `queued`; items in an active step re-queued.
4. `FakeCommerce` (in-memory shop; `fail_after_effect(op)` makes the op take effect remotely then return an error
   once, simulating a kill between remote effect and local record; records `(op, run_id, mono_time)`).
5. Commands, bindings, tests.

## Criteria (8 criteria, 28 rows; 0 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `publish_confirm([S], text)` | main [a,b,c], condition [d,e] with exclusion e, unreviewed [u] | one run + one item in one transaction; `frozen_main = [a,b,c]`, `frozen_condition = [d]`; u nowhere | M17 |
| C1(b) | same | values on the item incl. one invalid | `frozen_attributes` holds the canonical valid values only | M17, MC-10 |
| C1(c) | same | `item_get` shows media `[m1 (own), m9 (not owned)]` | `announced_removals = [m9]`; preview's removal count 1; `confirmed_summary == text` byte for byte | M17 (N1 count), MC-10 |
| C1(d) | same | session `change_seq = 7` | `confirmed_change_seq = 7`; `confirmed_at = now_ms` | MC-10 |
| C2(a) | run a full publish | — | every `FakeCommerce` mutation record carries `run_id` of a run whose `confirmed_at ≤` the call's time; zero mutations without a run | M17, M5 |
| C3(a) | `prepare` | `create_pending`, `item_search` → 0 | `item_create` called; `item_links` row (`external_id`, `create_pending = 0`) exists **before** the next remote call (fake op order) | MC-11 (prepare) |
| C3(b) | `prepare` | search → 1 | adopted into `item_links`; no `item_create` | MC-11 |
| C3(c) | `prepare` | search → 2 | item `failed`, `failed_step = prepare`, error `several_products_carry_sku` | MC-11 |
| C3(d) | `prepare` | RAW of a frozen photo changed | `failed` at `prepare` with `raw_changed{photo}`; no upload | M10 |
| C3(e) | `prepare` success | — | export files exist in the run staging; each link has `export_sha256` = SHA-256 of its file; `upload_state = exported` | MC-11, M15 |
| C4(a) | `upload` with `fail_after_effect(staged_uploads_create)` then `kill_and_restart` | 2 main photos | resume completes; final shop has exactly one media per photo; links `ready` | M18, M4 |
| C4(b) | same with `fail_after_effect(product_create_media)` | — | on resume `list_media` shows the created media; adopted (`attached`) without a second create; one media per photo | M18 |
| C4(c) | same with `fail_after_effect(poll → READY recorded)` i.e. kill after `attached` before `ready` | — | resume polls to `ready`; no duplicate | M18 |
| C4(d) | `upload` | one link already `ready` | fake receives no staged upload for it | MC-11 (ready skipped) |
| C4(e) | `upload` | photo p moved from item A's session (link ready for A) to item B's session; publish B | p uploaded again for B (filename stem same uid); A's link untouched | MC-11 (R-I4, link scoped by item) |
| C5(a) | `reconcile::plan` | `current = [x, m1, m2]`, `D = [m1, m2]`, x ∉ own ∪ announced | `Err(Foreign(1))`; fake received no delete | M19 (R-I5) |
| C5(b) | `plan` | `current = [o1, m2, m1, a1]`, `D = [m1, m2]`, own ∋ o1, announced ∋ a1 | `deletes = {o1, a1}`; `moves` = `[(m1, 0), (m2, 1)]` | M19 |
| C5(c) | `update` executed | — | fake product gallery ids `== D` in order; `cover == main 01`; no condition media in the gallery | M3, M19 |
| C5(d) | `update` | fake read-back returns a different order after reorder | item `failed` at `update`, error `gallery_verification` | MC-12 step 5 |
| C5(e) | `update` | a `D` element not `READY` | `failed`, error `upload_incomplete`, no deletion | MC-12 step 1 |
| C5(f) | `update` | fake returns not-found on one delete | counted as deleted; step proceeds | MC-12 step 3 |
| C6(a) | `upload` | frozen photo b marked `deleted` after confirmation | b skipped (no upload); gallery `D` excludes b; one notification at done | M17 (R-I6), M3 (no deleted photo leaves the Mac) |
| C6(b) | `upload` | frozen photo moved to another session after confirmation | skipped likewise | M17 |
| C7(a) | `done` | — | `published_at = now_ms`, `published_seq = confirmed_change_seq`, `notifications` row `kind = published` text starting "Published · "; status `published` | MC-11 (done), M14 |
| C7(b) | a `classify` during the run, then `done` | — | `dirty = true` after done (`change_seq > published_seq`) | M13 (R-I13) |
| C7(c) | `failed` at `upload` | — | `last_activity_at` updated; status `failed` if never published else label `PublishFailed` | MC-4, M14 |
| C8(a) | `publish_retry(item)` after a failure at `upload` | — | re-runs from `upload` only: fake receives no `item_search`/`item_create`; completes | M18 (retry from failed step) |
| C8(b) | `kill_and_restart` with an item in `upload` and a job `running` | — | job `queued` again; item re-queued at `upload`; run completes with no duplicates | MC-11 (startup) |

## Delegated choices
Job granularity (one `PublishStep` job per item stepping through, or one job per step), staging folder for the run's
exports (`.<key>.partial-<run>` as P09, then swapped to `<key>/` at `done` so the local export mirrors the publish),
notification text time format, how `publish_progress` events are throttled.

## Manual try-out
1. With Shopify configured and a session Ready, open the Publish popover (P15 brings it; until then call
   `publish_confirm` via a dev control the implementer may add) — after P15: Publish → the row shows "Publishing n%"
   → "Published"; the product in the shop shows the gallery in your Main order.
2. Reorder two photos and publish again: the gallery follows; nothing else on the product changed.
3. Kill the app mid-upload (Activity Monitor), relaunch: the row resumes and finishes; the shop has one media per photo.

## Notes
- `update` here ends after the gallery read-back; P14 appends the attribute, title, tags, description and
  damage-report writes to the same step (the step is committed once, before any of it).
- Estimated size: ~10 files, ~2 400 lines (FakeCommerce ≈ 500).

## Review log
(empty)
