---
plan: P62_retention_after_publish
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-10
actor: opus (pipeline-reviewer)
---
# Review: P62 retention after publish (round 1)

**Verdict: CHANGES_REQUESTED.** One blocking finding, F1 (B3, H4; route `production`). Clearing an item is not resumable. The item is marked cleared before its files are deleted, and the file queue lives only in memory. If the app quits, crashes or restarts partway through the deletions, the rest of the item's files stay on disk permanently. No later scan picks them up, because a cleared item is never eligible again. A reviewer probe confirmed this. Every other row holds at its boundary. The deletion gate (eligibility, then Shopify verification, then a re-check, then the cleared mark, then a per-file guard) is sound on every call site I read. Seven notes went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. F1 is a production defect with a clear correction; it needs no product decision.

## Gate
- Tracker row read `IMPLEMENTED`. Checkpoint `5b82ffe87cff246a05f53788abdc1f96e991bf9b` is in `git log`.
- `git show --stat 5b82ffe`: 46 paths. They equal the handoff's declared "Files changed" list exactly: 22 core, 2 infra, 4 integrations, 3 testkit, 4 Tauri, 11 frontend, plus the tracker and the plan Review log. Nothing extra, nothing missing. The plan's perimeter was "≈ 22 paths". The extra paths (guards in six services, `fake_commerce.rs`, `p01_storage_identity.rs`, `transport.rs`, `HealthPage.tsx`) are disclosed under Deviations, and each is needed for a row.

## Stamp (reviewer run, code tree `5b82ffe`; HEAD `2395dfd` adds only pipeline docs; porcelain clean before and after)
| Command | Result |
|---|---|
| `cargo test --workspace --no-fail-fast` | 896 passed, 1 failed. The failure is `p42_camera_runtime::c1c_restart_reaps_old_child` ("camera result did not settle", its 3 s settle bound under load; the machine also had the installed Session Manager.app, Capture One and fseventsd at 100 % running). It is the same known timing flake the implementer hit in that file (different test, same `settle`). No P62 code is on its path. The file run alone is green (last rows). |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` | clean |
| `npm test` | 62 files, 431/431 (default workers) |
| `cargo test -p psm-testkit --test p42_camera_runtime` (alone) | 12 passed, 0 failed |
| `cargo test -p psm-testkit --test p62_retention_after_publish` (alone, after the probe was removed) | 9 passed, 0 failed |

Not B4: the only red test is the documented load-sensitive p42 settle flake, not touched by this phase.

## Verified correct
- **Deletion gate (plan hazard note: every call site read).** Exactly one path queues item files: `StorageService::clear_item`. It runs only for items `plan_retention` chose from candidates that `mechanisms::retention::eligibility` passed. Those items are published (latest run `Done`, `published_at` set), not dirty, not pinned, not cleared, and an integration is configured: `retention_scan` returns early when `self.retention` is `None`, which is the case exactly when `App` has no commerce adapter (`with_retention(commerce.map(..))`). Then `retention::verify` must return `Holds`. Then eligibility is re-checked on fresh state, and the pause must be clear. Only then does `mark_item_cleared` run, followed by `item_files`. At delete time, `allowed(QueuedKind::Item)` → `allowed_item_file` requires `cleared_at` set and the path to be one of the session's RAWs, a Capture One file named after one, a file under one of the RAW's `previews|masters|outputs/<key[..2]>/<key>` folders, or a file in `.duplicates/`. P61's `delete` also re-checks that size and mtime are unchanged. No other code path deletes a RAW.
- **C1(a).** At T+3 d nothing is cleared. At T+4 d+1 min, the set of `retention` deletions equals the item's files exactly: 5 RAWs incl. hidden/deleted (Q1), `.cos` and cache files, previews/masters/outputs, and the duplicate. All are gone, the unpublished item's files are intact, and there is one `storage.cleared rule=retention path=… bytes=…` log line per deletion. Equality in both directions makes the test bite on an over-deletion as well as an under-deletion.
- **C1(b).** Dirty, unpublished and pinned items at T+30 d with 0 bytes free, plus an app without an integration holding an item marked published+done: no deletion under either rule. Each case has its own cause: the dirty item's images are still in the fake shop, so without the dirty check it would be cleared.
- **C2(a).** One gallery media removed → `Missing(1)` → no deletion, `cleared_at` unset, Health detail exactly `Not cleared: SKU-7 — Shopify is missing 1 images`. Shop unreachable → `Not cleared: SKU-8 — Shopify could not be read` (assumption 3, a separate wording for a failed read).
- **C3(a).** `ArchiveService::view` returns gallery 1–2 then condition 1 with the fake CDN URLs. The cached files equal the bytes served, the fetches asked for 1600 px (delegated choice, ≤ 2560), and a second view fetches nothing. The list row label is `RowLabel::Archived`; the UI test asserts the pill text `Archived in Shopify` and that the workspace renders the cached images with no review controls and no photo panel.
- **C3(b).** classify, reorder, composition set and publish confirm each return `Error::FilesCleared`, whose Display is exactly the row's text. The photo is unchanged and `product.description` saves. Guards also cover delete, move/copy (both directions), composition reset, undo/redo, export run and media-list attributes (assumption 10). The attribute guard reads before the write's transaction.
- **C4(a).** At T+60 d with 0 bytes free, the unpinned item is cleared (proving the job ran) and the pinned item keeps every file.
- **C5(a).** One byte short: only OLD (`retention`). Far short: OLD, then RECENT (`budget_published`), every OLD deletion before RECENT's first. OPEN is never cleared. `plan_retention` stops level 5 as soon as projected free space reaches the minimum.
- **C6(a).** `capture_one_file_of` matches exactly `CaptureOne/Settings*/<RAW name incl. ext>.cos` (depth 2) and `CaptureOne/Cache/**/<RAW name>.*`. The neighbour RAW and its Capture One files stay, and the catalogue's bytes and mtime are unchanged.
- **C7(a).** With another item's publish in `upload`, ticks return `paused = publish` and clear nothing. Once it ends, the item is cleared.
- **C8(a).** As in the stamp above.
- **Core mechanisms.** `mechanisms::retention::{eligibility, plan_retention}` are the plan's two listed mechanisms. Three input → output unit tests: the due boundary at exactly N days and at 0; Never for each of the 7 exclusions; the order with level 5's stop.
- **Shared RAWs (assumption 4).** A RAW used by another session that is not cleared is skipped together with its Capture One files, caches and duplicate. It goes when the last session using it is cleared. Consistent with M41.
- **Previews (P60 → P62 backlog note resolved).** `check_missing_files` skips photos of cleared sessions, so the app's own clearing is not reported as "deleted outside Session Manager".
- **Over-engineering test:** passed. The tests observe files on disk, `Cleared` outcomes, the log, Health, service results and the fake shop's recorded fetches. No internal call counts, no private helpers.
- **Hard constraints.** H1: `media_readback` is a GraphQL `query`, `fetch_image` is a GET; nothing writes to Shopify. H2: the amended exception is respected (published, clean items only, after verification). H3: no pixel processing (the archived images are stored as downloaded). H5: no UI automation. H6: no adapter → no retention. H7: the core uses the existing `"shopify"` integration-key convention; `archived_images` is a note (below). H8: the commands delegate to `ArchiveService`/settings. H4: **violated, F1.** Secrets: no credential in the diff. Live services: the suite uses `FakeCommerce` and made-up `cdn.fake-shop.test` URLs.

## Findings

### F1 (blocking, B3: H4; also B1 against 10F §3) · route: production
**What is wrong.** `clear_item` writes `cleared_at` (`mark_item_cleared`) and then pushes the item's files onto `Work.queue`, which exists only in memory. They are deleted 20 files every 2 s. From that moment the item is `Eligibility::Never` (cleared), and nothing else ever lists a cleared item's files again. So if the app quits or crashes, or the `App` is rebuilt, before the queue drains, the remaining files are never deleted. The same happens to a file whose size or mtime changes between queueing and deletion (P61's "still the one decided on" check skips it silently), for example a `.cos` that Capture One rewrites. An item with 200 photos is roughly 1 000+ files, which is minutes of batches.
**Evidence (reviewer probe, test code only, deleted after the run; tree clean after).** I published an item with 4 main, 1 condition, 1 hidden and 1 unreviewed photo plus Capture One files (47 files), and advanced the clock past retention. I ticked until the first batch ran: 20 deleted, `cleared_at` set. Then `TestApp::kill_and_restart()` and 40 more ticks: **27 files still on disk**, including RAWs (`HALF_C0.ARW`, `HALF_D.ARW`) and `.cos` files. The item reads "Archived in Shopify".
**Authority.** H4 ("after a crash at any point, resuming yields the same end state"). 10F §3 (N days after publish the item's local files are deleted). 10F §4 (the budget must be able to free them; today it would warn "nothing more can be cleared safely" while the cleared item's RAWs still use the disk).
**Correction.** Make clearing resumable. On each scan (and so at the first scan after start), also queue the files still on disk of items already marked cleared, from `item_files`, through the same `allowed_item_file` guard and with the rule they were cleared under (or `retention`). Do it without re-requesting a scan on every tick when a file cannot be deleted. Add one behaviour test at the storage boundary: clearing interrupted by `kill_and_restart` after the first batch → after restart every file of the item is gone and other items' files are untouched.
**Files the fix may touch.** `crates/core/src/services/storage.rs`; `crates/testkit/tests/p62_retention_after_publish.rs`; if the rule must persist, `crates/infra/src/sqlite/migrations/011_*.sql` + `crates/infra/src/sqlite/store.rs` + `crates/core/src/ports/store.rs` (migration 010 is in a checkpoint and must not be rewritten).

## Assumptions (handoff 1–13) read against the book
None contradicts the intention. In plain words, for the narrative at approval:
1. An item whose latest republish failed is not cleared, even if it was clean before (10F §4 excludes failed items). Age counts from the last completed publish.
2. "Every published image" means each output slot of the main and condition photos in the last completed publish. An image Shopify returns without a URL counts as missing.
3. A Shopify read failure shows "Shopify could not be read" rather than a number of missing images. All not-cleared items share one Health line.
4. A RAW also used by another, not yet cleared, session is kept until that session is cleared too.
5. The item's duplicates are found by SHA-256 against its RAWs.
6. Capture One files are matched by exact name.
7. The item becomes read-only before its files are deleted. (This ordering is right. F1 is about finishing the deletions, not about the order.)
8. A failed verification is retried after 15 minutes. At most 5 items are verified per scan.
9. Shopify images are downloaded at 1600 px on first view and cached. A failed download shows a placeholder and retries.
10. Read-only also covers local export, undo/redo, moving photos in or out, and the condition-photos field. Other attributes save but cannot be published again.
11. The pin is a checkbox in the item details card (key I). A cleared item shows no pin.
12. Saving the minimum free space and saving the retention days no longer overwrite each other.
13. The Shopify read-back query has not been run against the live store.

## Notes written to backlog
1. A new capture or import into a cleared item that is still the active session lands as a RAW the archived view never shows and retention never clears.
2. `allowed_item_file` accepts any file in `.duplicates/` once the item is cleared. The guard is wider than the queue (`item_files` hash-matches).
3. `archived_images` (core table) stores the Shopify media id and CDN URL with no integration column (H7-adjacent; link tables carry the integration).
4. Real adapter: the gallery URL falls back to `originalSource.url`, which can be a temporary upload URL; there is a `media(first: 250)` / `references(first: 100)` ceiling; the query is untested live (assumption 13). Verify on the test store before the work-Mac install.
5. The archived view downloads every missing image synchronously inside `item_archive_get`, so the first open of a large cleared item waits for all downloads.
6. Registry drift (orchestrator): migration 010 (`item_retention`, `archived_images`), commands `item_archive_get`, `item_keep_files_set`, `storage_retention_get`, `storage_retention_set`, Health code `retention_not_cleared`, port methods `CommerceAdapter::{media_readback, fetch_image}`, `HttpTransport::get_bytes`, the five `Store` retention methods, `RowLabel::Archived`, `Error::FilesCleared` (`files_cleared`), `mechanisms::retention`, `services::retention`. 10F's status header still reads "not planned".
7. The p42 camera settle flake recurred under load (`c1c_restart_reaps_old_child` this time).

## Lessons for the plans
- A destructive multi-step job needs a "resumes after a crash" row whenever its gate state is persisted before its work is. C1(a) proves "cleared at T+4 d" on an uninterrupted run only. The plan's hazard note named every call site, but not the restart path.

## Write perimeter of this session
- `docs/implementation/handoffs/reviewer/handoff_P62_retention_after_publish_review_r1.md` (this file)
- `docs/implementation/plans/P62_retention_after_publish.md` (Review log entry appended)
- `docs/implementation/master_plan.md` (P62 tracker row only)
- `docs/implementation/backlog.md` (7 P62 notes appended)
- Probe: `crates/testkit/tests/zz_reviewer_probe_p62.rs` was created and run, then deleted. `git status --porcelain` was empty before the probe and after its removal. No production file touched. Logs in `/tmp/p62_rev_*.log` (outside the repo).
