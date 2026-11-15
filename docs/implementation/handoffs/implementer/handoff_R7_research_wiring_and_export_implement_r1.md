---
plan: R7_research_wiring_and_export
role: implement
round: 1
state: DONE
date: 2026-11-12
actor: opus (Claude Opus 5.5, pipeline-implementer)
---
# Handoff — R7 research wiring and export, implement r1

## What was built
**Research is switched on in the app.** When the app opens it now builds the research engine over its own photos,
settings, shop connection, keys and job queue. A session that gets its first Main photo queues a research run, which
starts on research's own runner (its own thread), so a long research run never holds up previews, renders or
publishing. A run that was running when the app was quit is picked up again at the next launch and finishes.
Without a shop or keys, nothing about the core workflow changes. Research settings saved from Settings are checked
by the engine first (a bad value is refused with its reason); trusted-site edits are saved by their commands.

**The research record is kept for a set number of days and can be exported.** Call records older than "Keep research
records (days)" are removed at launch and every hour, with the image files only they used; runs, steps, feedback,
the outbound record and stage outputs stay. Settings → Research → **Export research** writes the chosen runs (a range
of days, or chosen items) to a folder you pick, as one folder plus one zip: `run.json`, `steps.json`, `writes.json`,
`feedback.json`, one file per call, and the images sent. A photo deleted since is left out and its requests say
`"withheld": "photo deleted"`; if any file would contain a key, nothing is written and the file is named.

**Review items carried into R7 (all done).** Only one research run can start at a time, also when two wake-ups race
(the runner takes one turn at a time, and the start transaction re-checks). The run reads the item's own listing in
the configured shop only. A field listed in both Details and Product is never written. A run that stopped at its
cost limit no longer marks earlier suggestions "No longer supported". Releasing a held publish item sends the same
update signal from every path, including undo and redo. The panel's "Waiting for Capture One" uses the engine's own
wait end time. The form's conflict buttons follow the backend's conflict rule (the TypeScript copy is gone), and
"Accept all (n)" counts only what it will accept.

Checkpoints: U1 `4a1de73`, U2 `b4a0464` (clean tree at `b4a0464` for the gate), closing commit (this handoff,
tracker, Review log) after it.

## Tests → criterion rows
| Row | Test | Boundary | Shape |
|---|---|---|---|
| A13·C1(c) | `research::r7_wiring::a13_c1c_core_workflow_is_untouched_without_research` | wired app, no shop/key; review + export | export folder names exact, last progress (3,3,Done), 0 runs, 0 calls, 0 requests to each fake — as specified |
| A13·C2(a) | `research::r7_wiring::a13_c2a_first_main_photo_queues_an_auto_run` | classify Main through `ReviewService`, `TestApp::tick` | exactly one run, (session, item, queued, auto) — as specified |
| A13·C8(c) | `research::r7_wiring::a13_c8c_a_running_run_is_resumed_after_a_restart` | kill inside a provider call, `kill_and_restart`, tick | one `research` job queued and due; run ends `finished` — as specified |
| A13·C8(d) | `research::r7_wiring::a13_c8d_research_has_its_own_runner` | research job blocked in the fake's hook on another thread; publish ticks | publish item reaches `done` while the research run is still `running` — as specified |
| carried MA13 (R4) | `research::r7_wiring::r7_u1_two_concurrent_wake_ups_never_run_two` | two threads wake the runner at once | every provider call sees exactly 1 running run; both runs finish |
| carried settings (R6) | `research::r7_wiring::r7_u1_research_settings_are_validated_and_saved` | `settings_set` | valid patch saved, merged and read back; invalid refused with the validation text, file unchanged |
| carried release (R6) | `research::r7_wiring::r7_u1_a_redo_that_releases_emits_the_event` | `HistoryService::redo` | item `held`→`prepare`; `SessionChanged` and `PublishProgress{prepare}` for that session/item emitted |
| carried capped run (owner) | engine `stages::r7_u1_a_capped_run_lowers_nothing` | `execute` | follow-up ends `stopped_at_cap`, reads no page; designer row's value, confidence, reason, sources unchanged (red without the fix: checked once, reverted) |
| carried Product step (R3) | engine `writes::r7_u1_a_key_in_details_and_product_is_never_written` | `writes::suggest` | `NotResearched`, no row |
| A21·C1(a) | engine `export::a21_c1a_a_call_is_deleted_exactly_at_record_days` | `purge_expired` | call at exactly 30 days gone, 1 ms younger kept |
| A21·C1(b) | engine `export::a21_c1b_an_image_a_kept_call_references_stays` | `purge_expired` | shared file stays, expired-only file deleted, only the kept call remains |
| A21·C1(c) | engine `export::a21_c1c_runs_steps_feedback_outbound_and_outputs_are_kept` | `purge_expired` | calls 0; runs, steps, feedback, outbound, outputs counts unchanged (all > 0) |
| A21·C1(d) | `research::r7_export_schedule::a21_c1d_purged_at_launch_and_hourly_without_a_call` | `kill_and_restart`, `TestApp::tick` on the manual clock | expired call gone at launch; the other kept before the hour, gone after |
| A21·C2(a) | engine `export::a21_c2a_selection_by_local_date` | `export(Range)` | only the 00:01 run's folder |
| A21·C2(b) | engine `export::a21_c2b_selection_by_items` | `export(Items)` | both runs of the chosen item, none of the other |
| A21·C3(a) | engine `export::a21_c3a_layout_and_zip` | `export` | exact file tree (4 json + 6 calls + 3 images by SHA-256); zip entries equal, byte for byte; image bytes equal the stored copies |
| A21·C3(b) | engine `export::a21_c3b_content_steps_calls_and_feedback` | `export` | `steps.json` equals the steps; each call's request/response text byte for byte; feedback = accepted (Teak→Teak), edited (Designer A→Designer B) |
| A21·C3(c) | engine `export::a21_c3c_expired_calls_are_named_in_run_json` | purge then `export` | `call_record` = "call record expired"; no `calls/`; steps and writes present |
| A21·C4(a) | engine `export::a21_c4a_a_deleted_photo_is_withheld` | delete the cover, `export` | its image absent; both of its requests carry `{"withheld":"photo deleted"}` and not its SHA; other images present |
| A21·C5(a) | engine `export::a21_c5a_a_secret_in_a_file_stops_the_export` | key written into a call record via the store | error names `calls/<id>.json`, does not contain the key; destination empty |
| A21·C6(a) | engine `export::a21_c6a_only_json_and_jpg_never_a_raw` | `export` | every zip entry `.json`/`.jpg`, none equals a RAW of the session |
| A21·C7(a) | UI `features/settings/__tests__/r7_research_export.test.tsx` | Settings → Research | `exportResearch({kind:'range',from:20261110,to:20261111}, folder)`; "Exported to <path>" shown |

Every test in the new files maps to a row above. Changed existing tests: `p01_storage_identity::c1a` (table list +5
engine tables, as the plan notes), the R2 desktop test (see Deviations), five form test fixtures (new DTO fields; the
R6 conflict fixture now carries `conflict: true`, the panel's wait test uses `wait_ends_at`).

## Assumptions
- Export date range: local dates as `YYYYMMDD` numbers, inclusive; the zip's path is returned, the folder stays beside it; JSON field names written to `master_plan_research.md` §6.7.
- "At launch" purge = when the app builds the engine (also after a settings save that rebuilds the runtime; harmless). Hourly = the runner's turn when an hour has passed on the app clock.
- `session_changed`: one observer on the app's event hub (delegated choice). `HistoryChanged` counts only when the session's `change_seq` moved, so a form edit does not restart the quiet period; an export render's `PhotoChanged` is heard (a quiet-period restart, never an extra run).
- Research runner wakes every 250 ms; `TestApp::tick` runs one research turn after the other runners.
- Secret values for the redaction scan and the export check: the three research keys, the camera's three secrets, the shop's client secret.
- Release events are emitted by the core services after the commit (not by the commands), so attribute actions, undo and redo share one signal.
- Research settings take effect without rebuilding the app, except a change of `research.providers` (the adapters are built with it).
- A run chosen by item includes runs that never started (their `run.json` has the queued row only).

## Deviations
- Files beyond the units' lists, each forced by a carried item or a changed shape: core `services/{settings, attributes, history, publish}.rs`; research `api.rs` (`wait_ends_at`), `decide.rs`, `runs.rs`, `service.rs`, `writes.rs`, `gateway.rs` (`secret_values`); desktop `commands/{attributes, research}.rs`, `src-tauri/src/lib.rs` (R2 test); `features/form/{AiPopover, FormDialog, ResearchPanel}.tsx` and five form test fixtures; `master_plan_research.md` §6.7 + amendment line (the plan delegates the names there).
- R2's `f_c1a_unwired_commands_return_research_unavailable` asserted the state R7 ends; it is now `f_c1a_research_commands_reach_the_wired_engine` (no command answers `research_unavailable`; no form value written).
- `store::start_run` now returns whether it started (guarded by "nothing running" in the same transaction): a second guard beyond the plan's single call site.
- Harness `link_item` links under the harness's integration (the configured one) since `external_id` now filters by it.

## Files changed
U1 (`4a1de73`): `crates/integrations/src/lib.rs`; `crates/core/src/services/{settings,attributes,history,publish}.rs`; `crates/research/src/{api,decide,runs,service,store,writes}.rs`; `crates/research/tests/engine/{harness,stages,writes}.rs`; `crates/testkit/src/{lib,test_app}.rs`; `crates/testkit/tests/suite/p01_storage_identity.rs`; `crates/testkit/tests/suite/research/{mod,r7_wiring}.rs`; `apps/desktop/src-tauri/src/{lib.rs,commands/attributes.rs,commands/research.rs}`; `apps/desktop/src/services/bindings.ts` (generated); `apps/desktop/src/features/form/{AiPopover,FormDialog,ResearchPanel}.tsx`; `apps/desktop/src/features/form/__tests__/{p11_form_ui,p22_form_tidy,p67_form_values_ui,r6_conflicts_in_form,r6_research_panel}.test.tsx`.
U2 (`b4a0464`): `crates/research/src/{retention,export}.rs` (new), `crates/research/src/{lib,api_impl,store,files,gateway}.rs`, `crates/research/Cargo.toml` (`zip`), `Cargo.lock`, `crates/research/tests/engine/{main,export}.rs`, `crates/integrations/src/lib.rs`, `crates/testkit/tests/suite/research/{mod,r7_export_schedule}.rs`, `apps/desktop/src/features/settings/ResearchSection.tsx`, `apps/desktop/src/features/settings/__tests__/r7_research_export.test.tsx` (new), `apps/desktop/src/services/research.ts`, `docs/implementation/master_plan_research.md`.
Closing: `docs/implementation/plans/R7_research_wiring_and_export.md` (Tracker line, Review log), this handoff.
Probe files (applied and reverted): `crates/research/src/decide.rs` (the capped-run condition removed once to see `r7_u1_a_capped_run_lowers_nothing` fail; restored before U1's checkpoint).

## Checkpoint SHA
U1 `4a1de73` · U2 `b4a0464` · closing commit follows (docs only).

## Stamp
Tree `b4a0464` (clean), once, in order:
| Command | Result |
|---|---|
| `cargo test --workspace` | green: suite 870, engine 135, research lib 120, desktop 24, others; 0 failed (2 min 29 s) |
| `cargo clippy --workspace --all-targets -- -D warnings` | green |
| `cargo fmt --all -- --check` | green |
| `cd apps/desktop && npm run typecheck` | green |
| `npm test` | 75 files, 488 tests green |
| `cargo test -p psm-research` | 120 + 135 + 1 green |
No pre-existing failures.

## Manual try-out
On scratch data (`cd apps/desktop && PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev`):
1. Open a form and the research panel: it says why research is not running ("Connect a shop to use research", or "Add the model key in Settings → Research"; real providers arrive with R8).
2. Settings → Research: add a trusted site, move it, reopen Settings — the order is kept. Type `0,5` as the per-run limit and save: the refusal names the field.
3. Settings → Research → Export research: keep today's dates, click Export research, pick the Desktop — the panel shows "Exported to …research-export-….zip" (empty of runs until R8 makes real runs).
4. Shoot, review and export a session as usual: nothing behaves differently.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
