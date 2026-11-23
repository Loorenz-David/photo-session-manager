---
plan: H4_remote_workspace
role: implement
round: 1
state: DONE
date: 2026-11-23
actor: claude opus (pipeline-implementer, PSM_IMPLEMENTER=claude)
---
# H4_remote_workspace — implement r1

Gate: intention RATIFIED; `H3_bring_back` Tracker APPROVED; H4 Tracker was NOT_STARTED (a previous run had stopped on a
usage limit before any change; the tree was clean at `37b0c95`). Plan of 3 units (flow.md §10), worked in order.

## What was built
**For the photographer.** An archived product (files cleared after publishing) or a shop-only product (adopted from the
shop, never photographed here) now opens in the **usual workspace**: its Shopify pictures fill the stage and the Main /
Condition groups, arriving one after another in the background (1600 px, 4 at a time); a line under the header says
"Photos are on Shopify. Editing a photo brings them back to this Mac first." with **Bring back now**. **Opening it never
moves the camera's capture folder** — new photos keep going where they went — and while visiting such a product from a
conversation, **Shoot here instead** is not offered. Looking, selecting, stepping, the Details form need nothing back.
Every photo edit (classify by button / key / swipe, deny, drag, compose / R, split / normal, apply, move, copy, assign,
template actions) asks "Bring back N photos from Shopify to edit them? About X MB." (singular for one) with **Bring back**
/ **Cancel**; Bring back shows "Bringing back k of N photos…", then the product is an ordinary session (and the capture
folder follows it); a failure shows the reason and **Try again**. **Publish** of a remote product sends its **details
only** ("Details only: the photos stay as they are on Shopify." and **Publish details**): Shopify's gallery and damage
report photos are untouched. An adopted product is read the first time it is opened ("Checking Shopify…"; on failure
"Could not check Shopify for this product's photos." with Try again — then it behaves like any session). **Refresh from
Shopify** is in the session's ⋯ menu for remote products. In the sessions list, remote rows count their Shopify pictures
and keep "Archived in Shopify" unless a publish of theirs is running, held, failed or cancelled.

**In code.** U1: `RemoteItems::view` (read + queue push, never a download or a shop call) with `View` downloads in
`run_pending`'s batch (same limit, stale rule, `PictureReady {Remote {session_id, role, position}}`), `check_failed`
(emit only on false → true), `set_keep_files`; `ArchiveService` removed; `SessionsService::{claim_target, activate}`
return before any editing / camera / store work for `Remote(_)`; `SessionsService::view` gives a remote session no
photos; conversation previews only when the file exists; media-list writes refused by state; commands
`session_remote_get`, `session_remote_refresh`, `session_bring_back`; `TestApp::with_camera_and_shop()`. U2: migration
021 `publish_items.details_only`; preview / confirm / run of a details-only item (no export, no upload, no gallery block,
no media-list write; flag read from the stored item on every resume); remote rows (MC-H3.7) from one `remote_covers`
read. U3: `features/workspace/remote/*`, the gate in `WorkspacePage`, the capture-target gate in `App.tsx`, `VisitBar`,
`SessionActionsMenu`, `PublishPopover`; `ArchivedView` and `item_archive_get` removed.

## Tests → criterion rows
Rust suite stems `h4_remote_workspace` (18), `h4_details_only_publish` (18); desktop `h4_tests` (4); UI
`workspace/__tests__/h4_remote_workspace.test.tsx` (17), `workspace/__tests__/h4_bring_back.test.tsx` (4),
`shell/__tests__/h4_capture_target.test.tsx` (11). Every test discharges a row; no orphan test.

| Row | Test | Assertion shape |
|---|---|---|
| C1(a) | `h4_remote_workspace::h4_c1a_the_view_answers_at_once_without_downloading` | exact: state, 5 (role, position, None, false), bytes 5 000 000, `image_fetches == []` |
| C1(b) | `…::h4_c1b_the_background_writes_each_picture_four_at_a_time` | exact: each file at the computed name, fetch set `(url, 1600)`, 15 events = expected set, max in flight 4 |
| C1(c) | `…::h4_c1c_a_cached_picture_is_served_from_its_file` | exact paths; fetch count unchanged after a further run |
| C1(d) | `…::h4_c1d_a_local_session_and_the_unassigned_session` | exact (Local, 0, keep_files true); `None` |
| C1(e) | `…::h4_c1e_the_first_open_read_is_queued_by_the_view` | exact: Unknown/0/false, 0 reads before, 1 read + 1 SessionChanged after, ShopOnly with 2 |
| C1(f) | `…::h4_c1f_a_failed_first_open_read_is_reported_then_tried_again` | exact: Unknown/true + 1 SessionChanged; then ShopOnly/false |
| C1(g) | `…::h4_c1g_gone_addresses_are_refreshed_in_chunks_of_25` | exact: `url_reads` lengths `[25, 5]`; 30 files |
| C1(h) | `…::h4_c1h_viewing_and_downloading_change_nothing_in_the_shop` | exact: `mutation_ops() == []` after the a/b/c/e/g flows |
| C1(i) | `…::h4_c1i_a_second_failed_read_in_a_row_announces_nothing` | exact: SessionChanged count stays 1; Unknown/true |
| C2(a) | `…::h4_c2a_opening_an_archived_item_leaves_the_active_session` | exact: no fake Capture One op since the call; active B |
| C2(b) | `…::h4_c2b_opening_a_shop_only_item_with_no_session_active` | exact: no op; active none |
| C2(c) | desktop `h4_tests::h4_c2c_session_activate_of_an_archived_item_keeps_the_filing_target` | through the command in camera-independent mode: active B, no op |
| C2(d) | `…::h4_c2d_local_and_unknown_items_activate_as_today` | `set_captures:` and `set_current_collection:` ending in `/<folder key>`; active L, then U |
| C2(e) | `…::h4_c2e_after_bring_back_the_capture_folder_follows_it` | as (d) for A after a real bring back; active A |
| C2(f) | `…::h4_c2f_a_capture_after_opening_an_archived_item_goes_where_photos_went` | the arrived row is in B and not in A |
| C3(a) | `…::h4_c3a_the_session_view_of_an_archived_item_has_no_photos` | 5 stored rows; `photos == []` with `with_remote` and as `session_get` builds it |
| C3(b) | `…::h4_c3b_after_bring_back_the_view_holds_the_brought_back_photos` | photo ids == the bring back's ids (5) |
| C3(c) | `…::h4_c3c_attachable_photos_of_an_archived_item_carry_no_dead_preview` | exact list (media id, photo id, `None`) after the preview files were deleted |
| C3(d) | `…::h4_c3d_a_remote_item_refuses_media_lists_and_saves_other_attributes` | `Err(FilesCleared)`; both text writes saved |
| C4(a) | desktop `h4_tests::h4_c4a_session_remote_get_answers_the_view` | exact JSON: `archived`, 5, 7 500 000, false, null; `null` for session 1 |
| C4(b) | desktop `h4_tests::h4_c4b_session_bring_back_of_a_local_session_is_not_remote` | code `not_remote` |
| C4(c) | desktop `h4_tests::h4_c4c_session_remote_refresh_answers_the_new_order` | exact gallery (position, media id) in the new order |
| C5(a) | `h4_details_only_publish::h4_c5a_an_archived_item_is_previewed_details_only` | exact tuple of the preview item; no new `item_get` read |
| C5(b) | `…::h4_c5b_a_local_published_session_is_previewed_as_today` | `details_only` false; `main` == its two main photos |
| C5(c) | `…::h4_c5c_a_shop_only_item_is_previewed_details_only` | as (a) |
| C5(d) | `…::h4_c5d_an_unknown_adopted_item_is_read_then_previewed_details_only` | 1 `media_readback`; as (a); the only `item_get` is the fake readback's own (see Assumptions); ShopOnly after |
| C6(a) | `…::h4_c6a_an_archived_item_publishes_its_details_and_no_media` | done; shop title and attribute; no media op in the run; gallery `[m1,m2,m3]`, condition `[c1,c2]` |
| C6(b) | `…::h4_c6b_details_only_is_stored_with_empty_photo_lists` | after `kill_and_restart`: `(true, [], [])` |
| C6(c) | `…::h4_c6c_a_run_killed_after_the_attribute_write_resumes_details_only` | killed after `attributes_write`, restarted: done; pictures unchanged; no media op in that run |
| C6(d) | `…::h4_c6d_a_shop_only_item_publishes_its_details` | done; attribute written; no media op; `published_at` set; still ShopOnly |
| C6(e) | `…::h4_c6e_the_item_stays_archived_after_its_details_publish` | `cleared_at` unchanged; Archived |
| C7(a) | `…::h4_c7a_a_stored_condition_value_is_left_out_of_the_writes` | the written keys are exactly `[custom.material]`; condition field unchanged |
| C8(a) | `…::h4_c8a_a_published_shop_only_item_is_never_cleared` | after 30 days of scans: `cleared_at` null, record equal, no `media_readback` |
| C9(a)–(g) | `…::h4_c9a_…` to `…::h4_c9g_…` (7 tests) | exact counts / status / label per row (C9(c): see Fixture finding) |
| C10(a) | UI `h4_c10a` (Core mechanism `remotePhotos`) | exact ids, classes, positions, paths; non-null frame and rect |
| C10(b)–(h) | UI `H4·C10(b)` … `H4·C10(h)` | rendered `<img>` sources per group, note text, `getRemote` call counts, `onDetailsOpenChange(true)` with no dialog |
| C11(a)–(i) | UI `H4·C11(a)` … `H4·C11(i)` | the exact question text; the named photo command never called; no editor / picker |
| C12(a)–(d) | UI `h4_bring_back.test.tsx` | `bringBack` called once / twice / never; progress text; panel shows the 5 own photos |
| C13(a)–(f) | UI `h4_capture_target.test.tsx` (C13(c) two runs, C13(e) three runs) | the whole ordered `activateSession`/`deactivateSession` list; Shoot here hidden / shown |
| C14(a) | UI `H4·C14(a)` | `refreshRemote(7)` once, `getRemote` again after it; no entry for a local session |
| C14(b) | UI `H4·C14(b)` | Publish offered; the details-only line; `Publish details` confirms `[7]` |

## Assumptions
1. A `getRemote` that fails is treated as `local` by the frontend: the screen and the capture-target gate behave as
   before H4 (the backend's own MC-H3.4 guard still refuses to point the camera at a remote product). Side effect: the
   12 existing App / Workspace test files listed in U3 needed **no edit** and pass unchanged.
2. `check_failed` is kept per session (one session per item).
3. The attribute guard refuses a media-list write for `Remote(_)` and still for any cleared item.
4. The pseudo-photos' string composition fields take the app's defaults (`orientation 'named'`, `flip 'none'`,
   `origin 'default'`; the plan's "null or 0" cannot apply to a string), `session_id 0`.
5. `View` downloads log `remote.view_picture item= outcome=`.
6. Refresh from Shopify is offered in the ⋯ menu of the session on screen (the header), not on sessions-list rows.
7. Before the first progress event the dialog reads "Bringing back 0 of N photos…".
8. `details_only` is left out of the preview JSON when false (`skip_serializing_if`, `serde(default)`), so the existing
   UI fixtures of the preview type stay valid.

## Deviations
Three files outside the declared perimeter, each forced by the change, plus one finding:
- `crates/testkit/tests/suite/c3_conversations_answer.rs`: C3·C8(a)'s fixture stored preview paths whose files never
  existed; MC-H3.5 / L22 now hide such paths, so the fixture writes the files in the test folder and expects their
  paths. The row's intent (adjusted first, else original) is unchanged.
- `crates/testkit/tests/suite/p01_storage_identity.rs`: the schema inventory gains `details_only` (as H1 added its columns).
- `apps/desktop/src/features/sessions/usePictures.ts`: one guard `else if (event.kind === 'search')` — the new
  `remote` kind of `picture_ready` would otherwise be filed as a search picture.
- **Fixture finding, H4·C9(c):** an attribute edit never moves `change_seq` (attributes MC-6), so "an archived item
  with an edited attribute (dirty)" is not dirty in this codebase; the test makes the session dirty through the store
  and the row's outcome holds. A dirty archived row is reachable only by data. For the planner / reviewer: the row's
  fixture text, not the behaviour, is off.
- H4·C5(d): the fake's `media_readback` reads through `item_get` internally, so the test asserts that exactly that one
  `item_get` happens (the preview itself makes none).

## Files changed
U1 `16606c1`: `apps/desktop/src-tauri/src/commands/{items.rs, mod.rs, sessions.rs, remote.rs (new)}`,
`apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/services/{bindings.ts, retention.ts, remote.ts (new)}`,
`crates/core/src/domain/remote.rs`, `crates/core/src/services/{attributes.rs, conversations.rs, remote.rs, retention.rs,
sessions.rs}`, `crates/testkit/src/test_app.rs`, `crates/testkit/tests/suite/{c3_conversations_answer.rs, main.rs,
p62_retention_after_publish.rs, h4_remote_workspace.rs (new)}`, `docs/guide/{02_architecture.md, 04_change_impact.md}`.
U2 `e9dd489`: `crates/infra/src/sqlite/migrations/021_publish_details_only.sql` (new), `crates/infra/src/sqlite/store.rs`,
`crates/core/src/{domain/publish.rs, services/publish.rs, services/sessions.rs, services/remote.rs, mechanisms/remote.rs}`,
`crates/integrations/src/lib.rs`, `apps/desktop/src-tauri/src/commands/publish.rs`, `apps/desktop/src/services/bindings.ts`,
`crates/testkit/tests/suite/{main.rs, p27_retire_shape_on_arrival.rs, p62_retention_after_publish.rs,
h4_details_only_publish.rs (new)}`, `docs/guide/{02_architecture.md, 03_rules_and_principles.md, 04_change_impact.md}`.
U3 `264133a`: `apps/desktop/src/App.tsx`, `apps/desktop/src/features/workspace/{WorkspacePage.tsx, useWorkspaceState.ts,
workspace.css, remote/{remotePhotos.ts, useRemoteItem.ts, RemoteNote.tsx, BringBackDialog.tsx} (new)}`,
`apps/desktop/src/features/{conversations/VisitBar.tsx, publish/PublishPopover.tsx, sessions/DeleteSession.tsx,
sessions/usePictures.ts}`, `apps/desktop/src/services/{bindings.ts, events.ts, retention.ts}`,
`apps/desktop/src-tauri/src/{lib.rs, commands/items.rs}`, new UI tests
`features/workspace/__tests__/{h4_remote_workspace, h4_bring_back}.test.tsx`, `features/shell/__tests__/h4_capture_target.test.tsx`,
`docs/guide/{01_product.md, 02_architecture.md}`; removed `features/workspace/ArchivedView.tsx`,
`features/workspace/__tests__/p62_archived_ui.test.tsx`.
Close `b19474a`: `crates/testkit/tests/suite/p01_storage_identity.rs`, `docs/implementation/plans/H4_remote_workspace.md`
(tracker line and Review log). This handoff is left for the runner to commit.
No file was edited temporarily to probe a test (flow.md §4: no mutation regime). `Header.tsx` and the 12 existing UI
test files were not changed.

## Checkpoint SHA
U1 `16606c1` → U2 `e9dd489` → U3 `264133a` → close `b19474a` (clean tree).

## Stamp
On `264133a` + the p01 line (identical code to `b19474a`):
- `cargo test --workspace` → 1334 passed, 1 failed (`p01_storage_identity::c1a_open_creates_all_domain_columns`, the
  missing `details_only` line); fixed, `cargo test -p psm-testkit --test suite p01_storage_identity::` → 20 passed;
  `cargo test --workspace --doc` (the doc-tests the stopped run skipped) → green. All other binaries green (core 151,
  desktop 92, infra 14, integrations 77, research 270, engine 340, tool 30). No pre-existing failure.
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- `cargo fmt --all -- --check` → clean.
- `cd apps/desktop && npm run typecheck` → clean.
- `npm test` → 125 files, 849 tests passed.
Per-unit commands were run after each unit (all green): U1 stems h4_remote_workspace, p62, a16, c3, c4 (+ c2), desktop,
typecheck, clippy; U2 stems h4_details_only_publish, p62, p13, p14, p03, p27 (+ h4_remote_workspace), desktop,
typecheck, clippy; U3 the six UI folders (54 files, 364 tests), typecheck, desktop, clippy.

## Manual try-out
On the installed app (the owner's; a session never launches the build):
1. In Sessions, open a product labelled "Archived in Shopify": the usual workspace opens, its pictures appear one after
   another, and the line "Photos are on Shopify. Editing a photo brings them back to this Mac first." shows under the header.
2. Take a photo: it lands where photos went before you opened the product, not in it.
3. Press Details, change the title, Publish: the confirmation says "Details only: the photos stay as they are on
   Shopify." with **Publish details**; on Shopify the title changed and the pictures are the same.
4. Drag a picture to another place: the app asks "Bring back N photos from Shopify to edit them? About X MB."; press
   Bring back and watch "Bringing back k of N photos…".
5. Afterwards it is an ordinary session: reorder, turn, re-frame, open in Capture One; the next Publish uploads only what changed.
6. Search (N) for a product photographed on the other Mac and open it: same workspace, same Bring back.
7. In the session's ⋯ menu, "Refresh from Shopify" re-reads the pictures after a change in Shopify's admin.
8. Open such a product from a conversation (Open session): the visit bar has no "Shoot here instead".

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
