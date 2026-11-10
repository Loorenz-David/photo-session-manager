---
plan: P64_publish_speed_and_visibility
role: implement
round: 1
state: DONE
date: 2026-11-10
actor: claude-opus (pipeline-implementer, PSM_IMPLEMENTER=claude; tracker actor `codex` as the prompt asks)
---
# P64 — Publish speed and visibility: implementer handoff (round 1)

## What was built
- **Unreviewed photos are published (changelog 44).** At confirmation the main photo list is the session's
  approved main photos in their order, followed by the photos not yet reviewed in the order the workspace shows them (arrival order); condition and
  deleted photos are unchanged. The confirmation note now reads "n unreviewed photos will be published after
  the main photos". A session is **Ready · Publish** with at least one main or unreviewed photo once every
  non-deleted photo has its adjusted preview; reviewing everything is no longer required.
- **Files are found by name, not by listing the shop.** The condition-photo recovery lookup is one Shopify
  file search `filename:<stem>` (the grounded SV-2 form). The paging query over every file in the shop is gone.
  The lookup now runs only for a link whose upload already reached Shopify (state `staged`/`uploaded`); a fresh
  publish does no file lookups at all.
- **Batched, parallel uploads.** Per publish item and group (main, then condition): every missing file is
  reserved in one `stagedUploadsCreate` (chunks of 50); uploads run in waves of up to 4 in parallel; each
  wave is registered in **one** request (`productUpdate` with several media / `fileCreate` with several files),
  recorded, then polled to ready. The Shopify HTTP transport now runs requests concurrently (multi-thread Tokio
  runtime on its private thread, each request spawned) so the 4 uploads really overlap. Gallery order is still
  set by the unchanged reorder step (MC-12/MC-21).
- **Progress explains itself.** Clicking or hovering the publish bar (session row and session header) opens a
  popover: the step in words (`Preparing outputs`, `Checking Shopify`, `Uploading main photos 3 of 8`,
  `Uploading condition photos 1 of 3`, `Updating the product`, `Writing the description`, `Cancelling…`),
  the percentage and elapsed time (`56% · Elapsed 1:05`), and the last error with its retry state
  ("Retrying after this error" while a retried step runs, "Stopped · press Retry to continue" while failed).
  Backend: `PublishService::activity(session_id)` → Tauri `publish_activity`.
- **Cancel publish.** The popover's **Cancel publish** asks to confirm ("Stop this publish? Photos already on
  Shopify stay there; the next Publish continues from them." · Keep publishing / Stop publish), then
  `publish_cancel(item_id)`. The publish stops at the next safe point: after the request in flight, before any
  new upload or mutation, never between a mutation and recording its result. The item ends in the new step
  `cancelled`; the row reads **Publish cancelled · Publish**; the session is not published (retention never
  starts); nothing on Shopify is deleted; a product created by the cancelled first publish stays linked as a
  draft; the next Publish reuses what was uploaded.

## Tests → criterion rows
| Row | Test | Boundary |
|---|---|---|
| C1(a) | `p64_publish_speed_and_visibility::c1a_lookup_by_name_in_a_shop_of_50000_files` | real `ShopifyAdapter` over `FakeTransport` holding 500 pages × 100 files: found in 1 request, every `files` request carries a search query |
| C1(b) | `…::c1b_lookup_keeps_the_match_rules` | same: `SKU-c01_a1b2.jpg` (prefix) and alt `SKU-c02.jpg` (processing) both found, an unrelated hit dropped |
| C2(a) | `…::c2a_reservations_batched_and_at_most_four_uploads_in_flight` | publish service over `FakeCommerce`: 8 main + 3 condition → ≤ 2 `staged_uploads_create`, 2–4 uploads in flight at once (fake counts concurrency), frozen gallery, 3 files |
| C2(b) | `…::c2b_crash_mid_batch_resumes_without_uploading_a_file_twice` | process killed during the 6th upload (5 done), restart: no media or file twice, photos on Shopify before the crash never staged again, gallery in frozen order |
| C3(a) | `…::c3a_progress_names_the_step_then_the_error_and_its_retry_state` (service) and `apps/desktop/src/features/publish/__tests__/p64_publish_progress.test.tsx` (3 popover tests) | `activity()` at the 3rd condition poll = `Uploading condition photos 2 of 3`, elapsed ≥ 65 s; failed upload → error + `retry_available`; after Retry → `retrying`; popover renders text, `56% · Elapsed 1:05`, error and retry state |
| C3(b) | `…::c3b_cancel_stops_after_the_request_in_flight_and_the_next_publish_continues` and the two cancel tests in `p64_publish_progress.test.tsx` | cancel while the bar reads `Uploading main photos 3 of 8`: no mutation after, step `cancelled`, row `PublishCancelled`, not published, nothing deleted, product linked; next publish stages/uploads only the 4 photos without media and ends in the full frozen order; popover asks to confirm before `publish_cancel(7)`; row reads "Publish cancelled" + Publish |
| C4(a) | `…::c4a_unreviewed_photos_are_published_after_the_main_photos` | B, C, A, D, E, F in workspace order → preview and `frozen_main` = B, A, C, D; E condition file; F never staged; gallery B, A, C, D |
| C4(b) | `…::c4b_ready_without_every_photo_reviewed` | session list row: main A + unreviewed C with previews → `ready`; C's preview missing → `in_progress` |
| C5(a) | the full suite, see **Stamp** | |

Core-mechanism unit tests (plan **Core mechanisms**): `mechanisms::publish::tests::frozen_main_is_main_then_unreviewed_each_in_display_order`,
`mechanisms::publish::tests::upload_waves_are_runs_of_at_most_four_in_order`.

## Assumptions
0. **"Position order" of unreviewed photos.** In this codebase only classified photos have a position (Main
   1…n, Condition 1…n); an unreviewed photo has none, and the workspace shows unreviewed photos by capture
   time, then id. `frozen_main` takes the main photos by position, then the unreviewed photos in that same
   display order (position, capture time, id). The C4(a) test builds the row's B, C, A, D layout this way.
1. **Cancel at "3 of 8" → next publish uploads 4, not 5.** A wave of 4 is registered in one request before its
   photos are polled to ready; the bar counts *ready* photos. When the photographer cancels at "3 of 8", the 4th
   photo's media already exists on Shopify (its readiness check is the request in flight). The next publish
   therefore completes the 5 photos that were not done, of which 4 need an upload and 1 only a readiness check.
   Uploading that 4th photo again would create a duplicate (H4). The C3(b) test asserts exactly this.
2. **After a cancel, uploaded-but-unregistered files are not registered** ("no new mutation after the request
   in flight"). Nothing exists on Shopify for them; the next publish uploads them again (bytes only, no duplicate).
3. **A cancel arriving while the last step finishes** (update done) lets the publish complete as Published;
   cancelling a finished publish would only hide a done product.
4. **Ready rule:** at least one main or unreviewed photo, and every **non-deleted** photo (condition photos
   included, as before) has its adjusted preview. Changelog 44 only drops the review requirement.
5. **Session status of a cancelled publish:** no new `SessionStatus`; the status derives as usual (ready /
   in progress / published) and only the row label is `PublishCancelled` (like `PublishFailed`, which also wins
   over the label). So the row's Publish button is enabled when the session is ready.
6. **Cancel and activity state live in memory.** After an app restart a pending cancel request is forgotten
   (the publish resumes, it was confirmed — H1 holds) and the popover shows the persisted step and counts.
7. **Elapsed time** counts from the first step this app run executed for the item (from the Retry press after a
   retry); without that, from the confirmation time.
8. **Delegated choices:** `stagedUploadsCreate` chunk 50; waves of 4 (`PARALLEL_UPLOADS`); main-media
   readiness: the batch registration polls the created media until READY to map names to links (SV-2: names
   appear only when READY), then each link's readiness is confirmed; files are polled one by one; popover
   aligned to the bar's end, opened on click or hover.
9. **Batch request shapes are not live-probed.** Several inputs in `stagedUploadsCreate`, several `media` in
   `productUpdate` (with `newest: media(first: $count, reverse: true)`) and several files in `fileCreate` are the
   documented list forms of the grounded single-item requests; one item still uses the exact grounded request.
   The owner's manual try-out exercises them on the test store.
10. **Search lag (evidence SV-3).** The files search index can lag right after `fileCreate`. A process kill in the
    narrow window between Shopify creating a condition file and the app recording it could, if the search has not
    caught up at resume, upload that photo once more as an orphan file in Shopify Files (never on the product:
    the damage report lists only recorded ids). The plan directs the search; listing the shop is what made the
    publish take minutes. The window is a crash between one request and one SQLite write.

## Deviations
- **Existing assertions changed** (the plan allows changes where the frozen set or `ready` changes; two are
  call counts that batching changes by design):
  - `p03_session_state_list::c1e_published_dirty_unreviewed` → `Ready` (ready rule).
  - `p03_session_state_list::c2f_in_progress_count` → input gains a missing preview so `InProgress(3)` still holds (ready rule).
  - `p13_publish_runs::c1a_confirm_freezes_lists_and_excludes_unreviewed` → `frozen_main`/preview include the
    unreviewed photo last; note text "will be published after the main photos" (frozen set).
  - `p14_publish_update::c6a_summary_counts_exclusions_removal_and_create` → "Publish 6 images", new note (frozen set).
  - `p13_publish_runs::c4b_resume_adopts_media_created_before_local_record` and
    `c4c_resume_polls_attached_media_without_creating_again` → `product_create_media` count 2 → 1 (both photos
    registered in one request; still 2 media, none created twice) — batching (C2).
  - `apps/desktop/src/features/publish/__tests__/p15_publish_ui.test.tsx` → new unreviewed note text.
- `graphql::FILES_ALL` removed (no caller left; nothing may list the shop's files).
- `CommerceAdapter` gained `staged_uploads_create_many`, `product_create_media_many`, `file_create_many` with
  default implementations over the single calls (port change in core; no platform shape in core).
- Chapter 07's text (ready rule, N2 note) is not edited here: changelog 44 records the decision and editing the
  book is not this role's lane.

## Files changed
Production: `crates/core/src/domain/{publish.rs,session.rs}`, `crates/core/src/mechanisms/{publish.rs,status.rs}`,
`crates/core/src/ports/commerce.rs`, `crates/core/src/services/publish.rs`, `crates/infra/src/sqlite/store.rs`,
`crates/integrations/src/commerce/shopify/{adapter.rs,graphql.rs,transport.rs}`,
`apps/desktop/src-tauri/src/{lib.rs,commands/publish.rs,commands/sessions.rs}`,
`apps/desktop/src/services/{publish.ts,bindings.ts}`, `apps/desktop/src/features/publish/{PublishProgress.tsx,PublishPopover.tsx}`,
`apps/desktop/src/features/sessions/{StatusPill.tsx,SessionRow.tsx}`, `apps/desktop/src/features/shell/Header.tsx`.
Test harness: `crates/testkit/src/fake_commerce.rs` (batched calls, upload concurrency counter, call hook).
Tests: `crates/testkit/tests/p64_publish_speed_and_visibility.rs` (new),
`apps/desktop/src/features/publish/__tests__/p64_publish_progress.test.tsx` (new),
`crates/testkit/tests/{p03_session_state_list.rs,p13_publish_runs.rs,p14_publish_update.rs}`,
`apps/desktop/src/features/publish/__tests__/p15_publish_ui.test.tsx`.
Docs: `docs/implementation/master_plan.md` (P64 tracker row only), `docs/implementation/plans/P64_publish_speed_and_visibility.md` (Review log),
this handoff. No migration, no fixture files (C1 builds its made-up replies in the test).

## Checkpoint SHA
`d11a2d4` — `CHECKPOINT (not approved): implement P64 publish speed and visibility` (29 paths; this handoff is left for the runner to commit).

## Stamp
Code tree = the checkpoint commit below (handoff excluded). Run on the dev Mac, 2026-11-10.
| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean (exit 0) |
| `cargo test -p psm-core mechanisms::publish` | 4 passed (incl. the 2 new core-mechanism tests) |
| `cargo test -p psm-testkit --test p64_publish_speed_and_visibility` | **8 passed**, 0 failed |
| `p03_session_state_list` | 28 passed |
| `p13_publish_runs` | 33 passed on the full file; the 3 changed tests (`c1a`, `c4b`, `c4c`) then re-run after their edit: 3 passed |
| `p12` 69 · `p14` 22 · `p15` 3 · `p16` 11 · `p17` 15 · `p18` 1 · `p21` 13 · `p25` 9 · `p30` 21 · `p39` 24 · `p59` 5 · `p62` 12 | all passed (test binaries of the final build, run directly) |
| `cargo test --workspace` (started) | every unit-test crate, `psm-desktop` (20, incl. `export_bindings`), `p00`–`p02` passed before I stopped it at `p05`: see below |
| `cargo test -p psm-desktop export_bindings` | passed; `bindings.ts` regenerated |
| `npm run typecheck` | clean |
| `npm test` | 435 passed, 1 failed: `p33_split_grid` C4(d), the plan's named intermittent; re-run alone: 7/7 passed. New `p64_publish_progress.test.tsx`: 5 passed |

**Full workspace run not completed (environment, not code).** On this Mac `syspolicyd` (Gatekeeper) was using 50–70 % CPU
assessing every freshly built binary, and `fseventsd` stalled each `TestApp`'s file watcher; one test file took 10–22 minutes
(`p13` alone 22 min). `cargo test --workspace` would have taken many hours, so I stopped it after `p05` and ran every
test file that references publishing, the commerce port, the Shopify adapter or the status rules (the table above), plus
P03 and P64. Files not run in this session after the change: the camera, composition, capture and storage suites
(`p04`–`p11`, `p19`–`p29` except `p21`/`p25`, `p34`–`p61` except `p39`/`p59`), whose code this phase does not touch.
The reviewer (who is told not to re-run the workspace) should know this stamp is partial.

## Manual try-out
1. Start the app on this Mac (`cd apps/desktop && npm run tauri -- dev`) with the test store configured.
2. In a session with a Shopify product, approve some photos as main, leave 2–3 unreviewed, mark 3 as condition.
   The row shows **Ready · Publish** even with unreviewed photos; the Publish popover says the unreviewed ones
   will be published after the main photos.
3. Publish. Hover or click the **Publishing n%** bar in the row or the header: the popover shows the current
   step (`Uploading main photos 3 of 8` …), the percentage and the elapsed time. The publish should finish well
   under a minute.
4. In Shopify admin: the product shows the approved photos first, then the unreviewed ones; the condition photos
   are in the damage report field.
5. Publish another session and press **Cancel publish** in the popover, then **Stop publish**. The row turns to
   **Publish cancelled · Publish**. Press Publish again: it continues from what was uploaded and ends with the
   full gallery, no duplicates.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
