---
plan: H1_points_and_remote_record
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-23
actor: claude (opus, pipeline-reviewer)
---
# Review: H1_points_and_remote_record, round 1

**Verdict: APPROVED_WITH_NOTES.** The plan has 12 criteria and 56 rows. Every row is implemented as the plan and 10H state, and each one has a behaviour test at its boundary that would fail on a wrong outcome. The review-mode checks are green. There are no blocking findings (B1–B4). Four notes are written to the track backlog. Tracker set to APPROVED. The narrative is written.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- Tracker read `IMPLEMENTED`.
- Checkpoints `9ec9baf` (U1), `74e952b` (U2) and `decdc29` (close) are in `git log`.

## Verified correct

**Perimeter.** `git show --stat` of the three checkpoints equals the handoff's declared files. It includes the two declared deviations:
- `p12_shopify_adapter.rs` has one match arm, forced by the new `Recorded::Download` variant.
- `p01_storage_identity.rs` is the schema tripwire. It lists every column.

U2's file list names `crates/testkit/src/fake_commerce.rs`, but only U1 edits it. This is harmless.

**Checks (run once, in sequence, Review mode):**

| Command | Result |
|---|---|
| `cargo test -p psm-core --lib mechanisms::remote` | 11/11 |
| `cargo test -p psm-testkit --test suite h1_shopify_reads::` | 21/21 |
| `cargo test -p psm-testkit --test suite h1_remote_record::` | 16/16 |
| `cargo test -p psm-testkit --test suite p62_retention_after_publish::` | 12/12 |
| `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-testkit --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |

The implementer's full-suite stamp stands: workspace suite 1205/1205, workspace clippy, typecheck, vitest 806.

**Review-mode reading checks:**
- **The three new port methods take no `ConfirmedRun`** (`media_urls`, `fetch_original`, `budget`). The defaults match master plan archive §4 Ports. The adapter sends only the queries `MediaUrls` and `MediaReadback`, and CDN GETs (H1).
- **The budget is read before the error returns in `call_inner`.**
  - `note_budget(&reply.body)` runs directly after `with_token` returns. That is before the retry branch and before the HTTP-status and GraphQL-error returns.
  - The mutex is held only to store or copy the reading, never across a request or the retry sleep.
  - H1·C1(f) would fail if the read came after those returns.
- **The new fixtures** (`h1_media_urls.json`, `h1_media_readback_sizes.json`) have a `source` that says "schema-grounded … not captured". They hold placeholder ids, `/0000/0000/0000/` CDN paths, and no shop domain or token.
- **`set_remote_record` leaves `cleared_at` and `keep_files` alone.** It runs `INSERT … ON CONFLICT DO UPDATE SET remote_checked_at` only, and replaces the record in the same transaction. `restored_at` is also untouched. `mark_item_cleared` still sets only `cleared_at`, so `restored_at` survives a clearing (MC-H4.8).
- **No Shopify, Capture One or Sony name was added to `crates/core`.** A grep over the added lines found none. `remote_input` reuses `retention::INTEGRATION`.

**Per criterion (production code read first, then its test):**

- **C1, budget (MC-H5.1–2).**
  - `ThrottleReading::of` needs all three numbers as `as_f64`, so integers count too.
  - An answer without `extensions` leaves the stored reading.
  - Tests (b)–(f) assert exact `Some {available, maximum}` values. (c) would fail if a missing block reset the reading, and (e) if the first reading were kept.
- **C2, `projected`.** It computes `min(max, avail + rate × s)`. The four exact values match.
- **C3, `must_wait`.** It tests `maximum > 0 ∧ available < 0.25 × maximum`. 249 waits and 250 does not; `None` and a zero maximum do not wait.
- **C4, `media_urls`.**
  - It makes one `call` per 25 ids, in order, and maps `null` to `None`.
  - A `MediaImage` takes its URL from `image.url`, a `GenericFile` from `url`. The size comes from `originalSource.fileSize` or `originalFileSize`.
  - A reply with too few nodes is refused with `"<got> of <asked>"`.
  - (b)'s second rule is keyed on the exact variables `ids[25..]`, so wrong chunking or wrong order would panic as an unmatched request.
  - (c) checks that no request at all was made.
- **C5, readback sizes.** The gallery keeps the `image.url`-first order, which the hazard note requires. Condition entries follow the order of the metafield value. A missing node gives `None/None`. The tests compare whole `Vec`s.
- **C6, downloads.**
  - `download_bytes` maps 200 → bytes, 403/404 → `ImageGone`, and anything else → `shopify download: HTTP <n>`, with no URL.
  - `fetch_original` refuses a non-https URL before any request; (f) checks that `requests()` is empty.
  - `FakeTransport` records every download, and an unmatched URL panics.
  - The behaviour of `ReqwestTransport::get_file` is unchanged, apart from returning the status (the size cap is kept).
- **C7, `remote_state`.** It matches MC-H3.1 branch for branch, and each test asserts the exact state.
- **C8, the record at the store.**
  - (a) hands the store the condition picture first, so "gallery first" really comes from the store's `ORDER BY role<>'gallery',position`.
  - (b) migrates a database to 19, writes rows by SQL, and opens the store. The pin and `cleared_at` are kept, and the new columns read as `NULL`.
- **C9, first-open read.**
  - It runs under one `record_lock` and re-reads the input after taking the lock. The shop is read only while the item is `Unknown`.
  - A missing URL is refused with `Provider("… still processing …")` before anything is written.
  - `SessionChanged` is emitted once. (d)'s 100 ms hook forces two overlapping callers, and (g) waits and then works.
- **C10, refresh.**
  - It works only on `Remote(_)` items and never consults the budget; (b) sets `{100, 1000}` after setup and the read still happens.
  - `cleared_at` is unchanged.
  - Pruning deletes every `cover-*` and every `archived/` file not named by `archived_file_name`. That one function is shared with `cached_path` (L13).
  - (d) places one current file and two stale files.
  - (e) compares `mutation_ops()` before and after (M-H1).
- **C11, `eligibility`.** It adds `shop_only` and `published_at ≤ restored_at`. The boundary pair `restored_at = published_at` → `Never`, `published_at − 1` → `Due` is tested.
- **C12, the clearing.**
  - `verify` copies `bytes`.
  - The scan checks `must_wait(config.commerce.budget())` before each verification, stops, and sets `scan_requested`.
  - (b) would see both items cleared, and media reads recorded, if the check were missing. (c) clears both once the budget recovers.

**Other hard constraints.**
- H2: only cache files under `previews/` are deleted.
- H4: each record write is one transaction.
- H6: `RemoteItems::state` and `ensure_record` answer `Local` with no integration, and `run_pending` is idle.
- H8: there is no Tauri dependency in core, and the command layer only gains the two error-code arms.
- No test calls a live service.

**Wiring.** `remote_background` starts only with `run_startup`. It sleeps 250 ms on `Idle`/`Waiting` and stops on the shared `stop` flag. Its shutdown join is bounded by `BACKGROUND_JOIN_BOUND`.

**Rewind test.** Migration 020 has its `M::up` line, and the three `DROP COLUMN` lines are in the p27 rewind test.

**Guide (§7A).** `02_architecture.md` and `04_change_impact.md` were updated for the new columns, the remote-items service and the budget rule.

**Tests overall.** One test per row, or per pair of rows where the plan chains them (C12(b)→(c)). There are no orphan tests, and none touches internals, call counts or query text.

**Assumptions (handoff §Assumptions).** None contradicts 10H or the master plan. Each is told to the owner in plain words in the narrative. Assumption 3 (a partial batch answers `Worked`) is a small difference from MC-H5.3's wording and is noted for a contract fold. Assumption 12 (camera-script `TestApp` variants start the thread) is noted.

## Findings
None blocking. (No B1–B4.)

## Notes written to backlog
These are in `docs/implementation/backlog_archive/H1_points_and_remote_record.md`, the destination the plan's Review mode names. The folder was created here. A pointer line was added to `docs/implementation/backlog.md`.

1. **A condition reference without a node is read as "still processing"** (MC-H3.2 as written). If Shopify keeps a deleted file's id in the field, the first-open read and Refresh fail on every attempt. Destination: the live check after H4 and the H4 Refresh rows.
2. **An adopted item that is still `Unknown` is not `shop_only` for retention.** If such an item ever became published and clean with zero photos, it would be cleared with an empty record. Destination: H4·C8.
3. **The `TestApp` camera-script variants start `remote_background`.** This is harmless today because they have no shop. Destination: test hygiene, whenever a test first combines `run_startup` with a shop.
4. **Contract fold for MC-H5.3:** `run_pending` answers `Worked` after a partial batch. Destination: the orchestrator, before H2 extends `run_pending`.

**Carry-forward dispositions:**

| Note | Destination |
|---|---|
| 1 | H4 Manual try-out (the live check) and the H4 Refresh rows |
| 2 | H4·C8 |
| 3 | Backlog (test hygiene) |
| 4 | Fold into `master_plan_archive.md` MC-H5.3 before H2 |

## Lessons for the plans
- The review prompt template sends notes to `backlog.md`, while archive-track plans name `backlog_archive/<phase>.md`. I followed the plan and added a pointer line, as the research track does. The template could read the destination from the plan.
- U2's Files list included `fake_commerce.rs`, which only U1 needed.

## Write perimeter (this session)
- `docs/implementation/plans/H1_points_and_remote_record.md`: the Tracker line and one Review log entry.
- `docs/implementation/backlog_archive/H1_points_and_remote_record.md` (new; folder created).
- `docs/implementation/backlog.md`: one pointer line appended.
- `docs/implementation/narratives/H1_points_and_remote_record.md` (new).
- This handoff.

No production or test file was edited. No probes were run (flow.md §4: no mutation regime). No database or state side effects. No commits.
