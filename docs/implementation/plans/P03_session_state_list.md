---
phase: P03_session_state_list
title: Session status, row label, activity grouping and the sessions list
difficulty: normal
projection: no
depends_on: [P02_review_order]
---
# P03 — Session status, row label, activity grouping and the sessions list

## Goal
Derive session status and the list row label exactly as MC-7 orders them, group sessions by local calendar
activity per MC-4, and expose the sessions list query (filters Open / Published / All with counts, search by
title or SKU, Unassigned count) through `sessions_list`. **Not in this phase:** publish items are only *read*
(rows are inserted by the test through the Store); no UI (P07); no publish logic (P13).

## Read first
master plan §6.5 (`publish_items`, `sessions`), §6.6 (`SessionStatus`, `RowLabel`), §6.7 · book `01`, `14`, `07`
(Status, list rows, Open filter), `04` S-A · contracts `05A` MC-7 (status, label, progress formula), MC-4 (grouping
bullet, ordering ties), MC-17 (a stale preview still counts as present) · evidence `design_handoff.md` §Sessions list.

## Files expected to change
`crates/core/src/mechanisms/{status.rs,time.rs}` (status/label/progress; grouping), `crates/core/src/services/sessions.rs`
(`list(filter, query)`), `crates/core/src/domain/session.rs` (DTOs `SessionRow`, `ListGroup`), `crates/core/src/ports/store.rs`
+ `crates/infra/src/sqlite/store.rs` (latest publish item per session, counts query, publish_items insert for tests),
`apps/desktop/src-tauri/src/commands/sessions.rs` (`sessions_list`), bindings,
`crates/testkit/src/test_app.rs` (`insert_publish_item(session, step, confirmed_at, failed_step?)`),
`crates/testkit/tests/p03_session_state_list.rs` (new).

## Core mechanisms (unit tests allowed)
- `mechanisms::status::derive_status(input: StatusInput) -> SessionStatus` (MC-7 rules 1–5).
- `mechanisms::status::row_label(input) -> RowLabel` and `progress_percent(step, uploaded, total) -> u8`.
- `mechanisms::time::group_by_activity(rows: &[(id, last_activity_at)], now_ms, tz) -> Vec<ListGroup>` (MC-4).

## Ordered tasks
1. `StatusInput { latest_step: Option<PublishStep>, failed_step: Option<PublishStep>, published_at: Option<i64>,
   dirty: bool, main_count, unreviewed_count, non_deleted_missing_preview_count }` built by the service from the
   Store (latest item = greatest `confirmed_at`, ties by greater run id). `derive_status`, `row_label`,
   `progress_percent` (prepare → 0; upload → `floor(10 + 65·uploaded/total)`; update → 75; describe → 85).
2. `group_by_activity` using `chrono` with an explicit `TimeZone` parameter (`Local` in production); calendar
   dates and ISO weeks (Monday start); within a group `last_activity_at` desc, then id desc.
3. `SessionsService::list(filter: Open|Published|All, query: Option<String>) -> SessionList { groups, counts{open,
   published, all}, unassigned_count }`; Unassigned (id 1) never appears as a row; search matches `title` or `sku`
   case-insensitively as a substring.
4. Command + bindings; tests.

## Criteria (5 criteria, 29 rows; 3 `[rule]` rows)
| Row | Boundary | Input (`StatusInput` unless noted) | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `derive_status` | latest step ∈ {prepare, upload, update, describe} (4 sub-checks), any other field | `publishing` | M14 |
| C1(b) | `derive_status` | latest step `failed`, `published_at` null | `failed` | M14 |
| C1(c) | `derive_status` | latest step `failed`, `published_at` set, not dirty | `published` | M14 (rule 2 needs never-published) |
| C1(d) | `derive_status` | `published_at` set, dirty, main 1, unreviewed 0, missing previews 0 | `ready` | M14 |
| C1(e) | `derive_status` | `published_at` set, dirty, unreviewed 1 | `in_progress` | M14 |
| C1(f) | `derive_status` | no publish, main 2, unreviewed 0, missing 0 | `ready` | M14 |
| C1(g) | `derive_status` via service | a main photo whose `adjusted_preview_path` is set but `raw_state.adjusted_gen > rendered_gen` | `ready` (stale counts as present) | M14, MC-17 |
| C1(h) | `derive_status` | main 2, one non-deleted photo with `adjusted_preview_path` null | `in_progress` | M14 |
| C1(i) | `derive_status` | main 0, condition 3, all reviewed with previews | `in_progress` | M14 |
| C1(j) | `derive_status` | empty session | `in_progress` | M14 |
| C1(k) | `derive_status` via service | a `deleted` photo with no preview, otherwise ready | `ready` (deleted ignored) | M14 |
| C1(l) | `derive_status` via service | two publish items: older `failed`, newer `done` with greater `confirmed_at`; `published_at` set | `published` (latest by `confirmed_at`) | MC-7 (latest item) |
| C2(a) | `row_label` + `progress_percent` | publishing at prepare / upload 2 of 4 / update / describe | `Publishing 0%` / `Publishing 42%` / `Publishing 75%` / `Publishing 85%` | M14 |
| C2(b) | `row_label` | latest `failed` at `upload`, `published_at` set | `PublishFailed` | M14 (R-I7) |
| C2(c) | `row_label` | latest `failed` at `describe` | `DescriptionFailed` | M14 |
| C2(d) | `row_label` | status `published` | `Published` | M14 |
| C2(e) | `row_label` | status `ready` | `Ready` ("Ready · Publish") | M14 |
| C2(f) | `row_label` | status `in_progress`, unreviewed 3 | `InProgress(3)` | M14 |
| C3(a) | `group_by_activity` (tz Europe/Stockholm, now = 2026-10-25 00:30 and 23:30 local, two runs) | instant 2026-10-25 09:00 local | in `Today` in both runs | M11 |
| C3(b) | same | 2026-10-24 23:59 local | `Yesterday` in both runs | M11 |
| C3(c) | same | 2026-10-19 00:00 local (Monday) | `This week` in both runs | M11 |
| C3(d) | same | 2026-10-18 23:59 local (Sunday) | `Last week` in both runs | M11 |
| C3(e) | same | 2026-10-11 and 2026-09-30 | month groups `October 2026` then `September 2026`, in that order after `Last week` | M11 |
| C3(f) | same | two Today rows with equal `last_activity_at` and ids 7, 9; one with a later instant | order: later instant first, then id 9, then id 7 | MC-4 (ties by id desc) |
| C4(a) | `list(Open, None)` | sessions: in_progress A, ready B, published C, publishing D | rows A, B, D; `counts = {open: 3, published: 1, all: 4}` | `[rule]` ch07 §Open filter |
| C4(b) | `list(Published, None)` | same | rows: C only | M14 |
| C4(c) | `list(All, Some("oak"))` | titles "Oak sideboard", "Teak desk"; sku "OAK-2" on a third | the two matching rows only | `[rule]` ch04 S-A (search by title or SKU) |
| C4(d) | `list(Open, None)` | Unassigned holds 2 photos | no row for session 1; `unassigned_count = 2` | MC-2 (Unassigned bucket) |
| C5(a) | `sessions_list` command DTO | any | each row carries `id, sku, item_type, title, counts{main, condition, unreviewed}, status, label, progress, last_activity_at, cover_preview_path` | `[rule]` evidence design_handoff §Sessions list |

## Delegated choices
Month-group label format (`October 2026` as in the design), `cover_preview_path` = main 01's adjusted (else
original) preview, whether counts are computed in SQL or in Rust, `chrono-tz` version.

## Manual try-out
No UI yet (P07 shows the list). The owner can wait for P07.

## Notes
- `[rule]` rows: C4(a), C4(c), C5(a) — list filters, search and the row DTO are chapter/design rules without a
  ledger entry; M14 still governs what each row's status and label say.
- `now` in C3 is injected (`ManualClock`); the production call site passes `Local` and `SystemClock`.
- Estimated size: ~7 files, ~900 lines.

## Review log
- 2026-10-25, codex, fix 1: Resolved F1 by asserting C1(l)'s row label is `Published`, which distinguishes the newer successful publish item from the older failed item. Full workspace suite, Clippy, formatting, desktop typecheck, and frontend test command pass (95 Rust tests; frontend reports no test files). No assumption or behavior changes; no divergence.
- 2026-10-25, codex, implement round 1: Built MC-7 status and row labels, MC-4 local activity groups, read-only sessions list with counts, search, cover preview and latest confirmed publish item, plus the `sessions_list` command and generated bindings. All 29 criterion rows have one behavior test; the workspace suite, Clippy, formatting, frontend typecheck and frontend tests pass.
- Assumptions: A publish item without `confirmed_at` is excluded from latest-item derivation, as MC-7 names the greatest confirmation time. Counts describe all sessions before search, matching the filter tabs in the design. Progress is present only while publishing; labels carry the displayed percentage or review count. The cover uses main position 1's adjusted preview, falling back to original, as delegated by the plan.
- Deviations: Added `Cargo.lock` and `crates/testkit/Cargo.toml` for the plan's Europe/Stockholm DST test dependency; registered the command in `apps/desktop/src-tauri/src/lib.rs` and status module in `mechanisms/mod.rs`. No semantic deviations.
- 2026-10-25, review r1 (opus): **CHANGES_REQUESTED** (one B2, test-only). Gate: tracker `IMPLEMENTED`, checkpoint `203a3de` present; its 16 committed paths equal the handoff's declared list (4 support files beyond the plan list, `Cargo.lock`, `crates/testkit/Cargo.toml`, `lib.rs` command registration, `mechanisms/mod.rs`, are declared deviations, all necessary). Stamp on tree `a622771` (clean): `cargo test --workspace` 0 (P03 28 + desktop `c5a` 1, P02 25, P01 20, P00 16), clippy `-D warnings` 0, `fmt --check` 0, `npm run typecheck` 0, `npm test` 0 (no frontend tests). Verified correct: `derive_status` is MC-7 rules 1–5 in order (publishing before failed before published; failed only when never published; ready = main>0, unreviewed 0, no non-deleted photo without `adjusted_preview_path`; staleness not consulted, MC-17); `row_label` precedence Publishing → Publish failed (any non-describe failed step, published or not, R-I7) → Description failed → Published → Ready → InProgress(unreviewed); `progress_percent` upload = 10 + ⌊65·u/t⌋ (2/4 → 42), prepare 0, update 75, describe 85; latest item SQL orders confirmed runs by `confirmed_at` desc then run id desc; `group_by_activity` uses calendar dates in the passed zone and `IsoWeek` (Monday start, year-qualified), Today/Yesterday before week tests, month groups `%B %Y` newest first, rows `last_activity_at` desc then id desc; service passes `chrono::Local`; Unassigned (id 1) excluded from rows in SQL (resolves the P01 "January 1970" note); Open = not published; search case-insensitive substring on title or SKU. Tests C1(a)–C1(k), C2(a)–C2(f), C3(a)–C3(f), C4(a)–C4(d), C5(a) each assert the row's exact outcome at its boundary and would fail on a wrong one (C3 runs both 00:30 and 23:30 on the 25-hour DST day). H1–H8: read-only phase, no external writes (H1), no file I/O (H2/H3), no provider concepts in core (H7), command is lock → service → map (H8). Assumptions consistent with ch05A/07 (unconfirmed runs excluded follows H1 and MC-7's "greatest `confirmed_at`"). **F1 (B2, route production, file `crates/testkit/tests/p03_session_state_list.rs` only):** `c1l_latest_confirmed_wins` cannot fail on a wrong latest-item choice. With `published_at` set and not dirty, status is `published` whether the latest item is the `done` or the `failed` one (rule 3 applies either way). Changed-input probe (scratch test, deleted, tree clean): failed newer than done → `status=Published label=PublishFailed`. Correction: also assert the row's `label == RowLabel::Published` (a wrong choice yields `PublishFailed`). Notes (backlog): Unassigned count includes deleted photos; flat count fields in row DTO; per-session queries; future `last_activity_at` grouping. Lesson for the plan: C1(l)'s input makes the status outcome independent of the rule it traces to; rows proving a selection rule need an input where the two candidates give different outcomes.
- 2026-10-25, review r2 (opus): **APPROVED_WITH_NOTES**. Gate: tracker `IMPLEMENTED` with fix note, checkpoint `9d0d9b6` present. Perimeter: `git show --stat 9d0d9b6` = `crates/testkit/tests/p03_session_state_list.rs` (+1 line, the only file F1 allowed) plus this plan's Review log and the tracker row (closing protocol); commits after it are runner-only (fix handoff, re-review prompt). Stamp on tree `7b8c919` (clean): `cargo test --workspace` 0 (95 tests; P03 28 incl. `c1l_latest_confirmed_wins` ok), clippy `-D warnings` 0, `fmt --check` 0, `npm run typecheck` 0, `npm test` 0 (no frontend tests). **F1 resolved** as its correction said: `c1l` now also asserts `label == RowLabel::Published`; choosing the older failed item yields `PublishFailed` (label precedence: failed non-describe step beats Published, R-I7), as r1's reversed-input probe showed, so the row can now fail on a wrong latest-item choice. No divergence. Triage: no blocking items remain. Note (backlog): in C1(l) run ids and `confirmed_at` rise together, so an ordering by run id alone would also pass; the SQL orders by `confirmed_at` first (verified by reading). Settled areas from r1 not re-verified.
