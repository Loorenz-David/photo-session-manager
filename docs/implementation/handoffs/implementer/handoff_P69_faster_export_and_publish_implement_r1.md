---
plan: P69_faster_export_and_publish
role: implement
round: 1
state: DONE
date: 2026-11-12
actor: claude (opus, pipeline-implementer)
---
# Handoff — P69 faster export and publish (implement r1)

## What was built
- **8-bit masters.** Capture One's `PSM Master` recipe is now an 8-bit TIFF (still uncompressed, sRGB, full size, crop ignored). A
  Capture One that still has the old 16-bit recipe gets it repaired by the existing self-repair before the next master render. The app's
  output JPEG is still rendered once from the master and compressed once; the renderer already read 8-bit masters.
- **Masters prepared in the background.** A new low-priority job (`PrepareMastersService`, its own thread, one tick a second) looks for
  Ready sessions (newest first) whose photos have no master for their current adjustment, and asks Capture One for **one master at a
  time**, only while Capture One is idle: no capture on its way (Save-to-PC expected or an FTP upload running), no take-back, no render out,
  no preview waiting to be rendered, no export or publish running, and free space at or above the storage minimum. A refused request
  (Capture One closed or busy) is retried after 30 s. The master lands in the normal master cache, so a later publish or export reuses it
  through the existing cache key and only renders what is missing; an edit in Capture One makes it stale as before; the masters are
  deleted after publish (MC-28) and may be cleared by the storage budget (level 3), then made again on demand.
- **Setting** `export.prepare_masters` (default on), shown in Settings → Storage as "Prepare masters in advance"; saving it applies at once
  (no reopen).
- **Overlap in a batch.** Items of one confirmed batch now go **at most one ahead**: an item does not start until the item before it has
  left its prepare step; the moment an item leaves prepare (its masters and outputs are ready), the next item's export is started, so
  Capture One renders its masters while the current item uploads. Completion order, progress, cancel and waiting are unchanged. (Before,
  every item of a batch asked Capture One for all its masters at once; a long batch could time out — the 9-item failure.)
- **Per-step times.** Every published item logs one INFO line
  `publish.timing item=<sku> masters_ms=… outputs_ms=… upload_ms=… shopify_ms=… total_ms=… masters_reused=n/m`, and the publish progress
  popover shows `Masters ready in advance: n of m` while the item publishes.

## Tests → criterion rows
| Row | Test (boundary) | Asserts |
|---|---|---|
| C1(a) | `p69::c1a_the_master_is_an_8_bit_tiff_and_the_output_is_made_once_from_it` (fake Capture One's recipe + the exported file) | a 16-bit `PSM Master` is repaired to TIFF/8/Uncompressed before the master render; the output from an 8-bit master is the exported `K_01_main.jpg` (400×400, the master's colour), one `render_output` job, one JPEG |
| C2(a) | `p69::c2a_masters_are_prepared_one_at_a_time_and_the_publish_renders_none` (preparer tick, fake Capture One renders, master cache, log) | Ready session: masters asked a, b, c, each only after the previous arrived (`paused = preparing` meanwhile); each in the cache; the publish asks no master, logs `masters_reused=3/3`, gallery in order; masters gone after publish |
| C2(b) | `p69::c2b_nothing_is_asked_during_a_preview_render_another_publish_or_with_the_setting_off`, `p69::c2b_nothing_is_asked_while_a_capture_is_on_its_way` | no master request while a preview render is out (`render`), while another session publishes (`publish`), with the setting off (`off`, persisted `false`), during an FTP upload (`capture`); the master is asked once each ends / the setting is on again |
| C2(c) | `p69::c2c_a_stale_master_is_rendered_again_and_cleared_ones_are_made_again` | after preparing, an edited photo's master is the only one rendered at publish; the export manifest has every output at the current `adjusted_gen` and composition revision; the recomposed photo is its whole 600×400 frame, the others 400×400; masters cleared by the budget (and not re-prepared below the minimum: `disk`) are rendered again by the publish, gallery correct |
| C3(a) | `p69::c3a_the_next_items_masters_render_while_this_item_uploads` (fake shop upload hook + fake Capture One) | batch of 3 with 40 ms uploads: at every upload, no master of an item more than one ahead was asked for; during item 1's (item 2's) upload, item 2's (item 3's) two masters were already asked for; items finish in order 1, 2, 3; galleries as without overlap; 6 masters total |
| C4(a) | `p69::c4a_one_timing_line_per_item_and_the_popover_counts_masters_ready`; UI `p69_masters_ready.test.tsx` (2) | one `publish.timing` INFO line with `item=K`, the five times as integers ≥ 0, `masters_reused=1/2`; the service's activity shows `Masters ready in advance: 1 of 2` during the uploads and nothing after; the popover renders the line, and none without it |
| C2(b) UI | `p69_prepare_masters_setting.test.tsx` | Settings → Storage checkbox saves `{export: {prepare_masters: false}}` |
| C5(a) | closing gate (below) | suite, clippy, fmt, typecheck, `npm test` |

Changed existing tests: `p27::c3b_ensure_creates_uncompressed_master` now expects `bits = 8` (the rule changed, changelog 47 (1));
`p56::c3c_restart_with_every_preview_current_orders_nothing` now counts preview renders only — the restarted app may now prepare the
Ready session's masters in the background (changelog 47 (2)); the test's subject (no preview re-render at restart) is unchanged.

## Assumptions
- **A1 Compression.** Kept `Uncompressed`: the fake cannot measure Capture One; VERIFY-15 measured ZIP 3–4× slower for 15 % less, and the
  recipe offers no faster lossless option.
- **A2 "Capture One idle"** = no capture on its way (Save-to-PC expected or FTP upload running), no take-back, no render out (any purpose),
  no stale preview the app will still ask for (failed-for-good and missing files excepted), no export run, no active publish item **of any
  session** (a publish of the same session renders its own masters anyway), and free space ≥ the storage minimum (so preparing and the
  budget never fight). After a refusal, 30 s before the next try.
- **A3 Which photos.** Sessions whose derived status is Ready (07; includes a dirty published session), with an item, not cleared, newest
  created first; in each, non-deleted photos with an intact RAW: main and unreviewed by position, then condition (unreviewed and excluded
  condition photos are included: a small, harmless extra).
- **A4** One master at a time; a master that fails for good (not "Capture One not ready") is not asked again at that adjustment; the
  publish renders it on demand.
- **A5** The background job ticks every second on its own thread; tests call the same `tick` directly (as P61's storage job).
- **A6 Batch.** "Batch" = the items of one confirmation. A waiting item stays in its prepare step with the usual text. When an item leaves
  prepare the next item's export starts at once; its product search/creation still happens in its own prepare step. After a restart the
  gate applies again.
- **A7 Timing fields.** `masters_ms` = export start → last master arrived (0 if none needed); `outputs_ms` = last master → export done;
  `upload_ms` = time in the upload step; `shopify_ms` = product search/create in prepare + the update (and describe) step; `total_ms` =
  this attempt's start (Retry restarts it) → done; `masters_reused` counts photos (not outputs) whose master or output was ready when the
  export started. A staging folder reused from an earlier attempt reports 0 ms and `m/m`. Times use the app clock (wall time in the app).
- **A8** The popover line appears only while the item runs and its export's numbers are known in this process.
- **A9** Setting name `export.prepare_masters` exactly as the plan writes it (the registry's `is_`/`has_` boolean prefix rule yields to
  the plan).
- **A10** A prepared master's frame is adopted like any master's (a photo whose frame no longer fits is flagged for review, as an
  export would do).

## Deviations
- Book 10D MC-28 still reads "TIFF, 16 bits"; changelog 47 (1) supersedes it — book fold for the orchestrator (not edited here).
- The P69 Rust tests run one at a time (a file-local lock): two read the `publish.timing` line through a thread-scoped log subscriber,
  which missed the line when other tests of the file hit the same log site in parallel (observed 3/3 runs; 7/7 green twice serialised).
- Two earlier phases' tests were adjusted to the new rules (p27 `c3b`, p56 `c3c`; see Tests).
- Tracker actor written as `claude` (this session ran as the Claude implementer), not `codex` as the prompt template says.

## Files changed
Production: `crates/integrations/src/editing/capture_one/recipes.rs`, `crates/core/src/services/export.rs`,
`crates/core/src/services/prepare_masters.rs` (new), `crates/core/src/services/mod.rs`, `crates/core/src/services/publish.rs`,
`crates/core/src/services/sessions.rs`, `crates/core/src/services/settings.rs`, `crates/core/src/domain/publish.rs`,
`crates/core/src/domain/storage.rs`, `crates/infra/src/settings_file.rs`, `crates/integrations/src/lib.rs`,
`apps/desktop/src-tauri/src/commands/settings.rs`, `apps/desktop/src-tauri/src/commands/publish.rs`, `apps/desktop/src/services/bindings.ts`
(regenerated), `apps/desktop/src/features/settings/StorageSection.tsx`, `apps/desktop/src/features/publish/PublishProgress.tsx`.
Tests and harness: `crates/testkit/tests/p69_faster_export_and_publish.rs` (new), `crates/testkit/tests/p27_retire_shape_on_arrival.rs`, `crates/testkit/tests/p56_camera_claim_and_render_loop.rs`,
`crates/testkit/src/fixtures/synthetic_master.rs` (8-bit writer), `crates/testkit/src/test_app.rs` (`complete_master_render_bytes`),
`apps/desktop/src/features/publish/__tests__/p69_masters_ready.test.tsx` (new),
`apps/desktop/src/features/settings/__tests__/p69_prepare_masters_setting.test.tsx` (new).
Docs: `docs/implementation/plans/P69_faster_export_and_publish.md` (Review log), `docs/implementation/master_plan.md` (P69 tracker row),
this handoff. No mutation probes (flow §4). No architecture graph in this repo.

## Checkpoint SHA
`b1129e1` (`b1129e17bb3c5f6d4a5bcb5793598ed383bd8916`) — the 25 paths above except this handoff (left for the runner to commit, as in earlier phases); tree clean apart from it.

## Stamp
| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | first run: 1 error (`type_complexity` in the new p69 test) → fixed (type alias) → re-run exit 0 |
| `cargo test --workspace` | every crate's unit tests and testkit p00–p55: 852 passed, 0 failed; stopped at `p56::c3c` (1 failed: it counted the masters the new background job asks for at restart) → test fixed |
| `cargo test -p psm-testkit --no-fail-fast --test p56 … --test p69` (the failing file and the files the stopped run never reached) | 93 passed, 0 failed (p56 4, p57 8, p58 6, p59 5, p60 8, p61 7, p62 12, p63 4, p64 8, p65 8, p66 6, p67 7, p68 3, p69 7) |
| `npm run typecheck` | exit 0 |
| `npm test` | 70 files, 466 tests passed |

Rust total: 945 passed, 0 failed. Pre-existing intermittent failures: none seen in this run.

## Manual try-out
1. Build and open the app; Settings → Storage shows "Prepare masters in advance" ticked.
2. Shoot or open a session until its row says Ready (every photo has its Capture One preview); leave the app alone for a few seconds.
3. In `~/Library/Application Support/com.loorenz.sessionmanager/masters/` one `master-<n>.tif` appears per photo, one after the other
   (each ≈ 180 MB on the a7R V, half of before). The app log (`logs/psm.<date>.log`) shows `master asked for in advance` and
   `master prepared in advance`.
4. Publish the session: the progress popover shows `Masters ready in advance: n of n`, and publishing skips the Capture One wait. The log
   has one `publish.timing item=<SKU> … masters_reused=n/n` line.
5. Select three Ready sessions → Review & publish → confirm: while the first item uploads, Capture One is already rendering the second
   item's masters (the third waits for the second).
6. Untick the setting: new Ready sessions get no masters until you publish them.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
