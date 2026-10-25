---
plan: P06_live_previews
role: implement
round: 1
state: DONE
date: 2026-10-25
actor: codex
---
# P06 live previews — implementation handoff

Adjusted previews now refresh from the Capture One render tied to each RAW and job id. Edits refresh after the configured pause or when Session Manager regains focus; a settings sidecar can recover missed adjustment hooks. Retouches change the session's publish state while the app's own colour-tag echo does not.

## ⚠ OWNER DECISIONS REQUIRED (0)

none.

## What was built

- Matched `processing_done` by job id and RAW, rejected missing or old outputs, copied accepted JPEGs atomically to generation-specific preview paths, updated all rows sharing a RAW, deleted the prior file after the database commit, and emitted `photo_changed` for each row.
- Added a case-insensitive per-base-name render queue. Preview renders retry once at 60 seconds and then fail with a timeout reason. The adapter also supports the 120-second export timeout and exposes an export completion callback for P09.
- Tracked adjusted/rendered/retouch generations per RAW. The background tick applies the clamped 10–30 second delay; focus renders stale RAWs immediately in active-session display order. Startup and focus inspect the newest `CaptureOne/Settings*/*.cos` sidecar.
- Marked reviewed photos in every owning session dirty on external retouch and emitted `session_changed`; own tag writes retain stale-preview refresh without a retouch bump.
- Connected the actual hook dispatcher and main-window `Focused(true)` event. The phase tracker is `IMPLEMENTED`.

## Tests → criterion rows

| Row | Boundary test | Outcome asserted |
|---|---|---|
| C1(a) | `c1a_completion_records_bytes_for_every_copy` | exact copied bytes, both paths, generation, per-row events |
| C1(b) | `c1b_old_output_rejected` | old output records no path or generation |
| C1(c) | `c1c_unknown_job_ignored` | no path or event |
| C1(d) | `c1d_new_generation_replaces_old_file` | new path and old file gone |
| C1(e) | `c1e_timeout_reissues_once_then_fails` | two provider processes, failed timeout job, no path |
| C2(a) | `c2a_delay_triggers_at_fifteen_seconds` | no process before 15 s, one at 15 s |
| C2(b) | `c2b_second_event_restarts_delay` | one process 15 s after second event |
| C2(c) | `c2c_event_during_render_stays_stale` | rendered 1, adjusted 2, second process |
| C2(d) | `c2d_delay_settings_are_clamped_by_service` | 5/40/missing/invalid → 10/30/15/15 s |
| C2(e) | `c2e_unknown_raw_adjustment_ignored` | no state or process |
| C3(a) | `c3a_focus_renders_stale_active_rows_first` | ordered three renders, fresh RAW omitted |
| C4(a) | `c4a_new_sidecar_creates_one_adjustment` | one generation increment across two focus events |
| C4(b) | `c4b_old_sidecar_does_not_adjust` | generation unchanged |
| C4(c) | `c4c_sidecar_within_own_write_window_ignored` | generation unchanged |
| C5(a) | `c5a_retouch_dirties_only_reviewed_session` | retouch bump, S1 dirty/event, S2 unchanged |
| C5(b) | `c5b_own_tag_echo_is_stale_but_not_retouch` | no retouch/dirty, delayed render |
| C5(c) | `c5c_retouch_dirties_both_copy_sessions` | both published sessions dirty |
| C6(a) | `c6a_same_base_name_waits_for_first_completion` | second process only after first completion |

`cargo test -p psm-testkit --test p06_live_previews`: 18 passed. Unit tests were added only for the plan's `staleness` and `accept_output` core mechanisms.

## Assumptions

- The existing app background tick is the delay timer. Returning focus starts every stale RAW immediately.
- Active-session display order uses classification, then position, then capture time; other sessions follow.
- The newest sidecar under `CaptureOne/Settings*` is the fallback signal. Its mtime must clear both the last render and own-write thresholds.
- P09 supplies the export staging callback; P06 leaves export naming and staging to that phase.
- A provider request that fails before returning a job id waits two seconds before another background attempt.

## Deviations

- Necessary wiring outside the plan's abbreviated expected-file list: `crates/integrations/src/lib.rs`, module declarations, `crates/core/src/events.rs`, `crates/infra/src/settings_file.rs`, and `crates/integrations/Cargo.toml` plus `Cargo.lock`. No other phase's behavior was implemented.
- A first full-suite run found P03 `c1g_stale_preview_present` failing because that test deliberately inserts its own `raw_state` row. I removed the new row creation from the direct photo storage path and prepared the P06 fixture locally. The final full suite passed.
- No live Capture One or camera smoke was run; the automated suite uses the fake adapter and local files, as required by flow §6A.

## Files changed

Checkpoint commit: `Cargo.lock`; `apps/desktop/src-tauri/src/lib.rs`; `crates/core/src/{events.rs,mechanisms/{mod.rs,staleness.rs},ports/{editing.rs,store.rs},services/{editing_events.rs,mod.rs,previews.rs}}`; `crates/infra/src/{lib.rs,previews_store.rs,settings_file.rs,sqlite/store.rs}`; `crates/integrations/{Cargo.toml,src/lib.rs,src/editing/capture_one/{adapter.rs,mod.rs,renders.rs}}`; `crates/testkit/{src/test_app.rs,tests/p06_live_previews.rs}`; `docs/implementation/{master_plan.md,plans/P06_live_previews.md}`.

This handoff is a separate uncommitted file for the runner's handoff commit, per flow §8. No mutation-probe files were touched.

## Checkpoint SHA

`4d3aef271f90c514e42cd2fd12f229c1820806e7` — `CHECKPOINT (not approved): implement P06 live previews`. The tracked tree was clean immediately after this commit, before writing this handoff.

## Stamp

- `cargo test --workspace` — passed: 177 Rust tests including 18 P06 boundary tests; doc tests passed. First attempt failed on inherited P03 `c1g_stale_preview_present`; repaired before the final passing run.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo fmt --all -- --check` — passed.
- `cd apps/desktop && PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm run typecheck` — passed.
- `cd apps/desktop && PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm test` — passed (no frontend test files in this phase).
- `git diff --check` — passed before checkpoint.

## Manual try-out

1. With the camera tethered and awake and a Capture One session open, start Session Manager on this Mac (`cd apps/desktop && PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm run tauri -- dev`). Shoot one frame. In the Health page, check the last capture event, then inspect the database: `sqlite3 "$HOME/Library/Application Support/com.loorenz.sessionmanager/session_manager.db" 'select raw_path,adjusted_preview_path from photos order by id desc limit 1;'`. The preview path should end in `adjusted-1.jpg` within a few seconds.
2. Move the Exposure slider for that photo in Capture One. Wait about 15 seconds and run the query again. The path should advance to `adjusted-2.jpg`, and the prior file should be gone. Change Exposure again, then click the Session Manager window: the next render should start immediately.
3. Inspect `sqlite3 "$HOME/Library/Application Support/com.loorenz.sessionmanager/session_manager.db" 'select adjusted_gen,rendered_gen,retouch_gen from raw_state order by rowid desc limit 1;'`. An edit increases `retouch_gen`; a colour-tag write by the app may refresh the preview but must not increase it. Published-session dirty state can be exercised after the publishing phases are available.
