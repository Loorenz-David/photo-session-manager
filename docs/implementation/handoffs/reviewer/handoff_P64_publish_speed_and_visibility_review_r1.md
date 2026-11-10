---
plan: P64_publish_speed_and_visibility
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-10
actor: opus (pipeline-reviewer)
---
# P64 publish speed and visibility: review r1

**Verdict: APPROVED_WITH_NOTES.** No B1–B4 finding. All 9 criterion rows hold at their boundaries; perimeter equals the declared
files; lint, typecheck and the touched suites are green. 8 notes written to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
Tracker row `IMPLEMENTED`; checkpoint `d11a2d4` present in `git log` (HEAD `125cca7`, the runner's prompt commit on top; working tree
clean apart from this session's files).

## Verified correct
**Perimeter.** `git show --stat d11a2d4` = 29 paths, equal to the handoff's "Files changed" (20 production, the fake, 6 test files,
master plan tracker row, plan Review log). No extra or missing path.

**Evidence this session** (tree = `d11a2d4`, code unchanged since; per the plan's time-boxed review mode, `cargo test --workspace` not re-run):
| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test -p psm-testkit --test p64_publish_speed_and_visibility` | 8 passed, 0 failed (128 s) |
| `cargo test -p psm-core mechanisms::` | 64 passed (incl. `frozen_main…`, `upload_waves…`) |
| `cargo test -p psm-integrations` | 36 passed, 0 failed |
| `npm run typecheck` | exit 0 |
| `npx vitest run src/features/publish src/features/sessions src/features/shell` | 12 files, 73 passed (incl. `p64_publish_progress` 5, `p15_publish_ui`) |
Implementer stamp consumed for p03, p12–p18, p21, p25, p30, p39, p59, p62 and `npm test` (435/1 named intermittent `p33_split_grid`).
It is partial (workspace stopped after `p05`, environment); see backlog. Nothing seen contradicts it: the new enum variants compile
in every target and the unrun suites do not reach publishing.

**Rows.**
- **C1(a)** `ShopifyAdapter::list_files_by_name` now makes one `FilesByName` request (`files(first: 20, query: "filename:<stem>")`,
  the SV-2 form); `FILES_ALL` is deleted, so nothing can list the shop. Test drives the real adapter over `FakeTransport` with
  500 × 100 `FilesAll` pages registered and asserts 1–2 `files(` requests, all with a non-empty query. A listing implementation
  would make ≥ 500 requests → red.
- **C1(b)** Match rules unchanged (name starts with stem, or alt = `<stem>.jpg`), now applied to the search hits. Test: prefix
  `SKU-c01_a1b2.jpg` found, unrelated hit `SKU-c03.jpg` dropped, processing file found by alt only. Discriminating.
- **C2(a)** `upload()` per group reserves every pending file with one `staged_uploads_create_many` (adapter chunks at 50), uploads
  in `upload_waves` of `PARALLEL_UPLOADS = 4` on scoped threads, registers each wave in one `product_create_media_many` /
  `file_create_many`. Test over `FakeCommerce` (40 ms upload delay, concurrency counter): 11 uploads, 1–2 reservations, 2–4 in flight,
  gallery in frozen order, 3 files. A per-file reservation (default port impl) → 11 calls → red; sequential → 1 in flight → red.
  Transport: `ReqwestTransport` now spawns each request on a 4-worker multi-thread runtime, so the real uploads overlap (read).
- **C2(b)** Links are recorded `staged` → `uploaded` → `attached` → `ready` around each remote effect; a wave's unregistered uploads
  restart from bytes only. Test panics inside the 6th upload, restarts the app: done, gallery = frozen order with no duplicate,
  3 files, each pre-crash photo staged exactly once, no deletes. Re-uploading registered photos would duplicate gallery entries → red.
- **C3(a)** `PublishService::activity` (Tauri `publish_activity`) returns step text, percent, elapsed, last error, retry state,
  `can_cancel`. Service test: at the 3rd `poll_file`, `Uploading condition photos 2 of 3`, elapsed ≥ 65 s, no error; failed
  upload → `Publish failed`, error, `retry_available`; after Retry → `retrying` with the error; done → cleared. UI test renders
  the popover from both bars' component: text, `56% · Elapsed 1:05`, alert with error + retry text.
- **C3(b)** `cancel(item)` sets an in-memory flag; every upload wave, registration, poll and update mutation checks it first; a
  wave's registration is skipped once cancelled; `run_item` ends the item in `PublishStep::Cancelled` (stored `cancelled`; no
  DB CHECK on `publish_items.step`). Resume query (`step IN ('prepare','upload','update','describe')`) and retention
  (`latest_step == Done`) both exclude `cancelled`. Test: cancel during the 4th readiness poll while the text reads
  `Uploading main photos 3 of 8`; no mutation after; step `Cancelled`, no error, row `PublishCancelled`, not published,
  no deletes, product linked; next publish stages and uploads exactly the 4 photos without media and ends in the full frozen
  order, one `item_create`. UI: confirm step before `publish_cancel(7)`; row pill "Publish cancelled" + Publish.
  The row's literal "5 missing" is 4 here because the 4th photo's media was registered with its wave; uploading it again would
  duplicate it (H4). Recorded as handoff assumption 1; consistent with H4 and the row's intent (only missing photos, none twice).
- **C4(a)** `mechanisms::publish::frozen_main` = Main by position, then Unreviewed by (position, captured_at, id), used by preview
  and confirm. Test B, C, A, D, E, F → preview and `frozen_main` = B, A, C, D; E condition file; F never staged; gallery
  B, A, C, D. Ordering by id alone would give B, C, A, D → red; excluding unreviewed → red.
- **C4(b)** `derive_status` Ready = main + unreviewed > 0 and no non-deleted photo missing its preview. Session-list test: Ready
  with unreviewed C; InProgress when C has no preview. Old rule → red on the first assertion.
- **C5(a)** Lint, typecheck, touched suites green (above); full suite per implementer's partial stamp.

**Assumptions read (none contradicts the intention).** 0 unreviewed order = workspace display order (no position exists);
1 cancel at 3 of 8 → 4 uploads next; 2 uploaded-but-unregistered files not registered after cancel; 3 a cancel landing during the
last step lets it finish; 4 ready still needs condition photos' previews; 5 no new session status for cancel; 6 cancel flag in
memory; 7 elapsed counts from this run's first step; 8 chunk 50 / waves of 4 / popover on click or hover; 9 batch shapes not
live-probed; 10 search-lag orphan-file window. All go into the narrative.

**Hard constraints.** H1: only the confirmed run writes; cancel adds no write, and no write starts after a cancel. H4: every
remote effect recorded before the next request; registration per wave; resume adopts by name; tested by C2(b)/C3(b); the SV-3
window is a note. H6/H7: platform shapes stay in the adapter; the port gains defaulted list methods in core terms. Secrets and
no-live-services: tests use `FakeTransport` / `FakeCommerce` with made-up data only.

**Existing-assertion changes** (handoff Deviations) checked: each follows from the frozen set, the ready rule, or one registration
per wave (`p13::c4b/c4c` count 2 → 1 `product_create_media`: same two media, none twice).

## Findings
None blocking.

## Notes written to backlog
1. The "Stopped · press Retry" popover state is unreachable: the popover wraps only the publishing bar.
2. Hover opens the popover; leaving does not close it.
3. Batch request shapes and the concurrent transport are not live-probed.
4. Condition-file recovery depends on the files search index (SV-3 lag window → possible orphan file).
5. The closing stamp is partial (workspace stopped after `p05`).
6. Plan row C3(b) "5 missing" assumes sequential uploads (lesson for plan authors).
7. Registry and book drift (new commands, step, label, port methods; chapters 07/07A text).
8. A cancelled publish has no session status of its own (filters count it as Ready).

## Lessons for the plans
- State resume rows as "only the photos without media on the platform", not a count derived from a sequential model.
- When a row names a popover state (retry available), name the bar/pill it is reachable from.

## Write perimeter (this session)
`docs/implementation/handoffs/reviewer/handoff_P64_publish_speed_and_visibility_review_r1.md` (this file),
`docs/implementation/narratives/P64_publish_speed_and_visibility.md`, `docs/implementation/backlog.md` (8 lines appended),
`docs/implementation/master_plan.md` (P64 tracker row only), `docs/implementation/plans/P64_publish_speed_and_visibility.md`
(Review log entry). No production or test code touched; no probes; build outputs under `target/` only. No archgraph.
