---
plan: P17_real_run_fixes
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-10-26
actor: opus
---
# P17 review, round 1: CHANGES_REQUESTED (1 blocking finding)

**Summary.** Fifteen of the 16 rows hold at their boundaries. C2(a) holds on its main outcome, but one of its clauses cannot fail (see the backlog notes). Both live Capture One rows, C2(b) and C3(e), were run by the reviewer against the owner's running Capture One, and both pass. Suite, lint and typecheck are green.

There is one blocking finding, F1 (H4). The task-3 reorder has the settings save build the new runtime while the old workers are still live. The build's startup crash-recovery then returns the job the old worker is executing to `queued`, and the new workers start straight away. A publish step can therefore run twice at once, which means the same photo is uploaded twice to the live store.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct

### Gate and perimeter
- Tracker `IMPLEMENTED`; checkpoint `7dd73eb8e52bbead147f5733eb1b4eb6c5bac184` exists.
- `git show --stat` lists 37 paths, equal to the declared files changed (the handoff itself is committed by the runner).
- Two files in the plan's list are unchanged, as disclosed:
  - `camera/via_capture_one.rs`: the user lane is wired in `App::build`.
  - `bindings.ts`: regenerated, identical.

### Stamp, re-run by the reviewer
| Command | Result |
|---|---|
| `cargo test --workspace` | 405 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | green |
| `cargo fmt --all -- --check` | green |
| `npm run typecheck` | green |
| `npm test` | 92/92 |

### Live rows (Capture One 16.8.6, document `session test 1`)
Scripts were generated exactly as `JxaScripting::new` writes them (`lib.js` + `\n` + script), into `/tmp/p17live.ITE3`.

**C2(b) holds.**
- `find_image_in_folder.js` printed `"130"`.
- `process.js` printed `"34048E26-0577-4155-970D-63D16E9B1E8B"`, exit 0.
- `get_color_tag.js` printed `0`.
- Capture One was browsing collection 29 `test-photo-session-001` before and after. That is a stronger condition than the row's `Capture`; the reviewer does not switch the browser.
- Output JPEG: `/Users/davidloorenz/Pictures/session test 1/Output/_session_manager/session test 12199.jpg`.

**C3(e) holds.**
- `id` is the document folder.
- `captures` ends `/Capture/PSM_PROBE_B`.
- The `capture` favourite's folder ends `/PSM_PROBE_B`.
- Collection `3` `Capture` has folder `…/session test 1/Capture`.
- The current collection was unchanged.
- Captures were restored to the as-found `…/Capture` and read back.

### Rows, each read production first, then the test
| Row | Verdict |
|---|---|
| C1(a) | Holds. The fake's `check_id` refuses stale ids on every per-document op, so `recipe_exists:PSM Preview` after the save discriminates. |
| C1(b) | Holds. Exactly `["Nope"]`, then none. |
| C1(c) | Holds. Commerce and publish appear and disappear with settings alone. `run_startup = false`, so there is no network call. |
| C2(a) | Holds for no `whose({file` and the single prelude definition. The per-script clause is vacuous (backlog). |
| C3(a) | Holds. The root comes from `capture_root(info.id)`, and the fake moves the `capture` favourite, so reading the favourite would fail the test. |
| C3(b) | Holds. Exact `set_captures` sequence A, A, `_Unassigned`, B. |
| C3(c) | Holds. `x` → B, `y` → session 1, no other rows. |
| C3(d) | Holds. `CaptureFolderNotSet`, and the health detail contains `<doc>/Capture`. |
| C4(a) | Holds. 35 `#[tauri::command]`, 35 `pub async fn` and 35 `spawn_blocking` across `commands/`. No lock access outside `state.rs`. |
| C4(b) | Holds. Mock IPC: `sessions_list` answers within 500 ms while `health_get` is still pending. It would block under the old `Mutex<App>`. |
| C4(c) | Holds. `on_window_focus` spawns its work. The old synchronous call would take at least 500 ms. |
| C5(a) | Holds. `process` at exactly 0, 1, 3, 7, 15, 31 s; 6 `recipe_exists` / `find_image` calls; job `done`. |
| C5(b) | Holds. Reset at 40 s; nothing in (40, 55) s; one render in flight at 55 s. |
| C5(c) | Holds. Background calls wait while `user_waiting > 0`. The render registry's mutex is never held across a scripting call. |

**Core mechanisms.** `render_backoff` gives 1, 2, 4, 8, 16, 32, 60, 60, 60. `capture_root` returns the canonical `<doc>/Capture` and errors when it is missing.

### Assumptions (handoff)
None contradicts the book:
- Collections without a folder are kept in `DocumentInfo`.
- One gate is shared across rebuilds.
- Focus work runs on a plain thread.
- The prelude is joined at write time.
- A failed build restores the prior settings file and keeps the old runtime.

### Hard constraints
- H1, H2, H3, H5, H6, H7 and H8 hold for the diff.
- H4 is violated (F1).
- No secret is in the diff.

## Findings

### F1: blocking (B3, H4), route: production
**A settings save requeues the job the live runtime is executing, so a publish step can run twice at once.**

**What is wrong.**
- `AppState::settings_set` → `SettingsSetContext::prepare` → `App::build` now runs while the old runtime's background and publish workers are still live. They are joined only after `apply_settings`.
- `App::build` runs start-only crash recovery: `store.requeue_running_jobs()` (`crates/integrations/src/lib.rs` ~l.311) and the re-enqueue of `active_publish_items`. It then spawns its own workers.

**Evidence.** A reviewer probe (a throwaway testkit test, deleted afterwards): claim a `mirror_tag` job through `Store::job_due`, then call `settings_set({preview_delay_s: 20})`. The job went from `running` to `queued`.

**What follows in production** (`run_startup = true`):
- A `publish_step` the old worker is executing becomes claimable within 50 ms by the new publish worker.
- Both run `PublishService::upload` for the same item. Each reads the photo link with `external_id = None`, and each calls `staged_uploads_create` + `upload` + `product_create_media` (or `file_create`).
- The result is a duplicate upload to the live store and two concurrent update steps.
- A running `render_export` row is also requeued, then closed by `InterruptedExport` while the export is still running.

**Why this is new.** Before P17, `reopen` joined the old workers first, so nothing was `running` at requeue time.

**Authority.**
- Book `01` H4: "Every step is idempotent … no duplicate uploads".
- Plan task 3: build before stopping so that a failed build leaves the old runtime running. That intent does not include two runtimes sharing live jobs.

**Correction.** On a settings rebuild, skip the start-only recovery steps (`requeue_running_jobs` and the active-publish re-enqueue). The existing `start_backup` flag already distinguishes a start from a rebuild. Alternatively, make the re-enqueue treat a `running` job as present.

**Test.** Add one behaviour test at the stored-state boundary: a job claimed before a non-theme `settings_set` is still `running` after it, and an active publish item still has exactly one `publish_step` job.

**Files the fix may touch:**
- `crates/integrations/src/lib.rs`
- `crates/infra/src/sqlite/store.rs` (only if the enqueue check changes)
- `crates/testkit/tests/p17_real_run_fixes.rs`

## Notes written to backlog
1. **P17 (test) → plan.** C2(a)'s per-script clause is vacuous on the written files: the prelude in every file contains both names. Verified by reading, and by `c2b`, P16 `c6a` and the live C2(b) probe.
2. **P17 (test).** The `c2b` / `c3e` Node harnesses model Capture One themselves. `c3e`'s "favourite follows captures" assertion checks the harness's own model. The live rows are the authority, and the suite needs `node` on `PATH`.
3. **P17.** Render backoff is cleared when a render is issued, not when it completes as task 5 says.
4. **P17.** Two quick settings saves both prepare from the same old settings, so the second overwrites the first's change.

## Lessons for the plans
- "Build the new runtime before stopping the old" needs its corollary in the row: the build must not run start-only recovery (requeue, re-enqueue) while the old workers are live. A rebuild row should name the state an in-flight job must keep.
- When a row inspects generated files that share a prelude, say whether the prelude counts.

## Write perimeter and side effects

**Files this session wrote:**
- `docs/implementation/master_plan.md` (P17 tracker row only)
- `docs/implementation/plans/P17_real_run_fixes.md` (Review log entry)
- `docs/implementation/backlog.md` (4 rows appended)
- this handoff

**Probe file.** `crates/testkit/tests/zz_reviewer_probe_p17.rs` was created, run and deleted; `git status` was clean afterwards. No production file was edited.

**Capture One side effects:**
- One JPEG written by the `PSM Preview` recipe: `…/session test 1/Output/_session_manager/session test 12199.jpg`.
- The capture folder was set to `…/Capture/PSM_PROBE_B`, then restored to the as-found `…/Capture` and verified by read-back.
- No capture was fired, and the browsed collection was never changed.

**Temp scripts:** `/tmp/p17live.ITE3`, outside the repo.

**No narrative was written**, because the verdict is not an approval.
