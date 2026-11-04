---
plan: P30_outputs_in_export_and_publish
role: implement
round: 1
state: DONE
date: 2026-11-04
actor: claude-opus (pipeline-implementer, Claude fallback; the prompt template names codex)
---
# Handoff — P30 outputs in export and publish, implement r1

## What was built
- **A split photo is two pictures everywhere.** Local export already numbered files per output (P29); a split main photo at place 1 now
  shows up as `K_01_main.jpg` (left) and `K_02_main.jpg` (right) and later photos move down. Publishing does the same: a split photo
  becomes two media in the shop gallery, Left then Right, side by side, and a split condition photo becomes two files in the damage report.
- **Upload names.** `K-<uid12>.jpg` for a normal photo, `K-<uid12>-L.jpg` / `-R.jpg` for the halves, and `-r<n>` after a revision. Core
  builds the stem (`mechanisms::upload_name::stem`). The Shopify adapter and the fake only add `.jpg`.
- **Re-framing replaces in place.** Each shop link now belongs to one output slot (`single`/`left`/`right`) and records the composition
  revision it was rendered from (migration 005). A new composition revision re-uploads that photo's output(s) as `-r1` in the same
  gallery place and removes the old media. Normal ↔ Split retires the slots the photo no longer has. Their media leave the gallery, and
  the rows stay as proof the media were the app's own.
- **Refusal while a photo needs re-framing.** Publish confirmation refuses with `Re-frame first: Main 02` and writes nothing. If a photo
  gets flagged after confirmation, the item fails at `prepare` with the same message, before any shop call. Retry publishes once it is
  re-framed.
- **No stale upload.** An earlier attempt's staging folder is reused only when its `manifest.json` matches the current outputs: photo,
  slot, master generation, composition revision, output format, and entry count. A retried `upload` whose staging went stale goes back
  to `prepare` and renders again.
- **Masters freed.** After a publish completes and the session is not dirty, the session's 16-bit masters are deleted. The outputs stay
  cached.
- Session view badge = the state of the photo's first slot link.

## Tests → criterion rows
All in `crates/testkit/tests/p30_outputs_in_export_and_publish.rs`. Each asserts the row's exact outcome at its boundary.

| Row | Test | Assertion shape vs row |
|---|---|---|
| C1(a) | `c1a_split_main_and_condition_photos_export_one_file_per_output` | exact sorted file list, `item.json` main/condition lists, manifest `(photo, slot)` for K_01/02/03 main + both condition files — as specified |
| C1(b) | `c1b_a_photo_deleted_mid_run_renumbers_the_outputs` | b deleted after masters, before outputs; main files exactly K_01..K_03, `item.json` lists three, manifest a-left/a-right/c — as specified |
| C2(a) | `c2a_split_halves_upload_with_l_and_r` | fake staged filenames in order = `[-L, -R, b]`, gallery names equal — as specified |
| C2(b) | `c2b_a_split_revision_uploads_both_halves_with_r1` | revision 2; second-run staged names exactly `[-L-r1, -R-r1]`; gallery exactly three in order — as specified |
| C2(c) | `c2c_a_split_condition_photo_creates_two_files` | shop files `[d-L, d-R]`; metafield `= [file(d-L), file(d-R)]` — as specified |
| C3(a) | `c3a_links_are_per_slot_and_record_the_composition_revision` | full link rows `(photo, role, slot, state, composition_rev, revision)`; `photo_link(a, Left)`; no `(a, single)` — as specified |
| C3(b) | `c3b_migration_005_keeps_existing_links_as_single` | migrations()[..4] + `to_version(4)`, rows inserted, `SqliteStore::open`; slot/composition_rev/external_id/state/revision/retouch_gen/id; unique index over the 5 columns — as specified |
| C3(c) | `c3c_a_composition_revision_replaces_the_media_in_place` | a's link `(1, 1, [old], ready)`; one media of a, at position 1, named `-r1`; b revision 0, only a staged — as specified |
| C3(d) | `c3d_an_identical_save_uploads_nothing` | no new staged name, revision unchanged, not dirty; link `(0, 1)` — **deviation (b)**: `composition_rev 1` not 0 |
| C4(a) | `c4a_normal_to_split_retires_the_single_link` | single link `retired` with its id; id gone from gallery; left/right ready; gallery `[a-L, a-R, b]` — as specified (a foreign media would fail the run, so `done` proves no `foreign_media_added`) |
| C4(b) | `c4b_split_to_normal_retires_the_halves` | halves retired, gone; single `ready`, revision 0, uploaded as `K-<uid12(a)>.jpg` (no `-r`), a fresh media; gallery `[a, b]` — as specified |
| C4(c) | `c4c_retired_rows_stay_as_own_evidence` | left/right retired rows with ids kept, first single id in the single row's `superseded_ids`; `view().upload_states[a] == "ready"` — **deviation (a)**: two retired rows, not three |
| C5(a) | `c5a_gallery_is_the_main_outputs_and_the_cover_is_the_left_half` | gallery names in order; first media (position 1) is `-L` — as specified |
| C5(b) | `c5b_a_reorder_keeps_the_halves_adjacent_left_first` | gallery `[b, a-L, a-R]` — as specified |
| C6(a) | `c6a_damage_report_lists_condition_outputs_in_order` | metafield `[d-L, d-R, e]`; none in gallery (main photo `m` present so the gallery is non-empty) — as specified |
| C7(a) | `c7a_confirmation_refuses_while_a_photo_needs_review` | `Err(NeedsReview{["Main 02"]})`, `publish_runs` count unchanged, no fake mutation — as specified |
| C7(b) | `c7b_needs_review_after_confirmation_fails_prepare_then_retry_publishes` | item failed at prepare, error contains `Re-frame first: Main 02`, no mutation; re-framed, retry → done, gallery `[a, b]` — as specified |
| C8(a) | `c8a_a_stale_staging_from_a_failed_attempt_is_rendered_again` | failed at upload; revision 1; retry → done; `uploaded_sha["K-<uid12>.jpg"]` = sha of `<uid12>-single-g<g>-r1-jpeg-2048-q90.jpg` = link `export_sha256`; old `.K.partial-<old>` gone; `K/manifest.json` composition_rev 1 — as specified |
| C8(b) | `c8b_masters_are_released_after_a_publish_of_a_clean_session` | done, not dirty, masters dir absent, outputs dir present — as specified |
| C8(c) | `c8c_a_session_dirty_at_done_keeps_its_masters` | done, dirty, `master-<g>.tif` present — **deviation (c)**: the reorder happens while `upload` holds at its first media poll |

Core mechanism unit tests: `mechanisms::publish::tests::{mc21_replacement_candidates_only_newer_ready_links (extended),
mc29_replacement_on_a_newer_composition_revision}`. No other test was added. Red baseline: **not captured before the first production edit**
(flow.md has no mutation regime). I wrote the tests after the production code. C3(b)/C3(c)/C8(a) are rows the pre-change code could
not satisfy (no `slot` column, no composition clause, stale staging uploaded).

## Assumptions
1. `UploadState::Retired` is the string `"retired"`: `upload_state` is a `String` in the code (the registry's enum never existed).
2. Stem case: stored folder key as is, uid hex lowercase, `-L`/`-R` uppercase as 10D writes them. `upload_name::matches` now ignores case,
   so names the old adapter lowercased are still recognised. Folder keys are unique ignoring case (05B). 07A says "lowercase", but 10D and
   the plan's rows write `K-…-L`.
3. A publish item resumed at `upload` whose staging no longer matches goes back to `prepare` (Retry re-runs the failed step, `upload`, in
   C8(a)).
4. A known export run that failed now fails the item with `export failed: <reason>`, where before it waited forever (the P29 backlog's
   stuck-run note). Retry starts a fresh export.
5. Links not yet `ready` take the staged file's `retouch_gen` and `composition_rev` at `prepare`.
6. Adoption by stem never adopts an id in the link's `superseded_ids`.
7. In the fake, `fail_before_effect` on `staged_uploads_create`, `product_create_media` and `file_create` now fails before anything is
   created. Before, the fake pushed the media and then returned the error, which made C8(a) adopt the stale media.
8. Delegated: `Error::NeedsReview` reused, retirement at `prepare` only, counts per output in both the item and the progress event,
   `migrations()` is `pub`, fake field `uploaded_sha`.

## Deviations
- (a) **C4(b)/(c) and the plan's own key.** UNIQUE(photo, integration, item, role, slot) allows one `(a, single)` row, so the returning
  single slot reuses its retired row. Its state goes back to `none`, the old media id moves to `superseded_ids` (still `own`), and the
  revision is kept (`K-<uid12>.jpg`, no `-r`, as C4(b) asks). After C4(b) there are two retired rows plus the first single id in
  `superseded_ids`, not "three retired rows". The intention (MC-29 §Links, MC-12 `own`) holds.
- (b) **C3(d).** P28's `save_composition` counts default → user origin as a change. `set` with identical values on a default composition
  bumps the revision, and `reset` recomputes the default rect on the master frame. So the row publishes a user composition first
  (revision 1) and re-saves it identically: link `composition_rev 1`, not 0. **For P31:** opening and closing the editor without a change
  must not call `set` on a default composition, or the session becomes dirty.
- (c) **C8(c).** One worker tick runs `upload` → `update` → `done` in a row, so the photo change happens while `upload` holds at its first
  media poll. That is still a change during the run, before `update`.
- (d) **Files outside the listed perimeter, all forced by the signature changes:** `crates/integrations/examples/shopify_probe.rs` (two
  `staged_uploads_create` call sites), `crates/testkit/tests/p01_storage_identity.rs` (the schema column list for `photo_links`),
  `crates/testkit/tests/p15_publish_backend.rs` (`PhotoLink` literals), and `crates/testkit/Cargo.toml` + `Cargo.lock` (`rusqlite_migration`
  dev-dependency, needed for C3(b)'s `to_version(4)`). I removed the adapter's `upload_filename` and its unit test. Core's `stem` now builds
  the name, and the C2 rows cover it.
- `naming.rs`, `error.rs`, `integrations/src/lib.rs` and `test_app.rs` needed no change: masters are released in core's `run_item`, and
  `TestApp::fake_shop()` already exposes the fake.

## Files changed
Production: `crates/core/src/{domain/publish.rs, mechanisms/publish.rs, mechanisms/upload_name.rs, ports/commerce.rs, ports/store.rs,
services/export.rs, services/publish.rs, services/sessions.rs}`, `crates/infra/src/sqlite/store.rs`,
`crates/infra/src/sqlite/migrations/005_photo_link_slots.sql` (new), `crates/integrations/src/commerce/shopify/adapter.rs`,
`crates/integrations/examples/shopify_probe.rs`.
Test support: `crates/testkit/src/fake_commerce.rs`, `crates/testkit/Cargo.toml`, `Cargo.lock`.
Tests: `crates/testkit/tests/p30_outputs_in_export_and_publish.rs` (new); amended `p01_storage_identity.rs`, `p12_shopify_adapter.rs`,
`p13_publish_runs.rs`, `p14_publish_update.rs`, `p15_publish_backend.rs`.
Docs: `docs/implementation/master_plan.md` (P30 tracker row only), `docs/implementation/plans/P30_outputs_in_export_and_publish.md`
(Review log), this handoff.
No bindings change (no command changed). No mutation probes: none were run, and no file was applied-and-reverted.

## Checkpoint SHA
`a3d561f` — `CHECKPOINT (not approved): implement P30 outputs in export and publish` (clean tree after the commit; this handoff is left for
the runner to commit).

## Stamp
Tree `a3d561f` (all changes committed before stamping, re-run unchanged):
- `cargo test --workspace` → exit 0, 528 passed, 0 failed (P29 baseline 508; +20 rows, +1 core unit test, −1 adapter unit test removed)
- `cargo clippy --workspace --all-targets -- -D warnings` → clean
- `cargo fmt --all -- --check` → clean
- `npm run typecheck` → clean; `npm test` → 132 passed (21 files)
No pre-existing failures.

## Manual try-out
1. With the test store configured, open a session with a few main photos and publish. The gallery shows the square compositions at 2048 px
   in order. Afterwards the session's folders under `masters/` in the app's data directory are gone, and `outputs/` keeps the JPEGs.
2. Publish again without changes. Nothing is uploaded and the run completes at once.
3. Make one photo split in the database (`sqlite3 <data dir>/session_manager.db "update compositions set kind='split', revision=revision+1, x=0,
   y=0.1251, w=1, h=0.7499 where photo_id=<id>"`) and publish. That photo becomes two square media, `…-L` then `…-R`, side by side, and
   later photos move down. In a local export the folder shows `_0n` and `_0n+1` for it.
4. Set it back to normal the same way (`kind='normal'`, a square rect, `revision=revision+1`) and publish. The two halves leave the
   gallery and one media returns in the same place.
5. Rotate a photo 90° in Capture One. Once its new preview arrives it needs re-framing, and Publish is refused with "Re-frame first:
   Main 0n". Export is refused the same way.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
