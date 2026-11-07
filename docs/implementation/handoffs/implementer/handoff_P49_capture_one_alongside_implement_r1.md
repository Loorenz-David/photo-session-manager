---
plan: P49_capture_one_alongside
role: implement
round: 1
state: DONE
date: 2026-11-07
actor: codex
---

# P49 implement handoff

## What was built

Capture One now follows the active session's capture folder on activation, startup, take-back step 5, and every direct-camera arrival or imported photo that lands in another collection. It checks that Capture One is running before an arrival script. A new photo waits up to 15 seconds for its Capture One variant, polled every second; if still absent, preview attempts continue on the 1, 2, 4, 8, 16, then 60-second-capped backoff. Switching collections retries a stale preview immediately. A configured style is applied to the found variant before its first render, read back, and marked in SQLite once per RAW; a style already present after an interrupted write is not applied twice.

Import copies each source into the active session's capture folder, or `_Unassigned` if none is active. A collision gets `__dupN`; the source and prior destination are untouched. The former Capture One `import` script and provider operation are gone. The fake Capture One now models current-collection lookup and per-variant styles.

## Tests → criterion rows

| Rows | Boundary tests in `crates/testkit/tests/p49_capture_one_alongside.rs` |
|---|---|
| C1(a)–(c) | `c1a_activation_sets_collection`, `c1b_failed_collection_is_health_and_startup_repairs`, `c1c_startup_switches_collection` (also drives take-back step 5) |
| C2(a)–(d) | `c2a_arrival_switches_from_other_collection`, `c2b_arrival_in_current_collection_does_not_switch`, `c2c_each_arrival_switches_again_after_browsing_away`, `c2d_stopped_editor_is_never_scripted_or_launched` |
| C3(a)–(d) | `c3a_variant_wait_polls_once_per_second_without_render`, `c3b_style_is_applied_before_first_render`, `c3c_late_variant_recovers_with_backoff_and_no_failure` (exact poll times and late style), `c3d_collection_switch_retries_immediately` |
| C4(a)–(e) | `c4a_style_once_even_after_retouch`, `c4b_style_not_reapplied_after_restart`, `c4c_unlisted_style_sets_health_but_renders`, `c4d_empty_style_neither_applies_nor_warns`, `c4e_each_raw_receives_style_on_its_own_variant` |
| C5(a)–(c) | `c5a_import_copies_and_leaves_original`, `c5b_import_collision_keeps_both_files_and_rows`, `c5c_unassigned_import_follows_its_folder` |

Earlier behavior tests updated for the new schema and import destination: P01 C1(a), P05 C8(a)/C8(c), P27 C1(b)/C4(a). No P49 unit tests were added; its Core mechanisms list is empty.

## Assumptions

- The collection setter reads Capture One's live collection every time. It uses the existing `collectionForFolder` helper, assigns `doc.currentCollection`, and compares the read-back `file` path. This catches browsing away after the app's previous switch.
- The import file is copied to a hidden temporary file in the destination folder, published with an exclusive hard link, then the hidden name is removed. This gives the planned atomic no-overwrite result on the same filesystem.
- Variant waiting is for direct-camera capture deliveries and imports. Tethered Capture One captures keep their established immediate render path.

## Deviations

- `apps/desktop/src-tauri/src/commands/sessions.rs` is outside the planned path list; it wires collection-switch retries into the actual `session_activate` and `session_deactivate` commands.
- P01 and P27 test files are outside the planned path list. Their old exact schema and import-path fixtures could not remain correct after migration 007 and copy-on-import. P05 C8(c) also now expects a second copy after a same-name import.
- The plan says the `SCRIPTS` array becomes 21 entries. It was 20 plus a separately installed `import.js`; removing that install and adding two new scripts leaves 22 array entries. Both new scripts are installed.
- Task 0's pre-change fake probe **did recover** after the variant appeared: `PreviewService::request` stored `retry_after` and retried after exponential delay. This differs from live V4; no stuck `render_in_flight` or permanently uncleared `retry_after` appeared in the fake. The new C3(c) test checks exact recovery timing.

## Files changed

- Core ports and services: `crates/core/src/ports/{camera,editing,store}.rs`, `crates/core/src/services/{arrival,editing_startup,photos,previews,sessions}.rs`.
- SQLite: `crates/infra/src/sqlite/store.rs`, `crates/infra/src/sqlite/migrations/007_style_applied.sql`.
- Capture One and app wiring: `crates/integrations/src/editing/capture_one/{adapter,follow,gate,jxa,mod,scripting}.rs`, `jxa/{apply_style,set_current_collection}.js`, removed `jxa/import.js`, `crates/integrations/src/lib.rs`, `crates/integrations/src/camera/direct/take_back.rs`, `apps/desktop/src-tauri/src/commands/sessions.rs`.
- Testkit and behavior tests: `crates/testkit/src/{fake_capture_one,test_app}.rs`, `crates/testkit/tests/{p01_storage_identity,p05_capture_arrival,p27_retire_shape_on_arrival,p49_capture_one_alongside}.rs`.
- Process: `docs/implementation/master_plan.md`, `docs/implementation/plans/P49_capture_one_alongside.md`.

## Checkpoint SHA

`df50d3d19e1fc053c5f3ac41211dab5af544a7bb` — `CHECKPOINT (not approved): implement P49 Capture One alongside`. The handoff is written after the checkpoint for its SHA; the runner commits handoffs.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | PASS, including 19/19 P49 rows (final run) |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cd apps/desktop && npm run typecheck` | PASS, Node 22 selected through `PATH` |
| `cd apps/desktop && npm test` | 368/369 twice; only the documented pre-existing P33 C4(d) timing flake failed in parallel |
| `cd apps/desktop && npm test -- --maxWorkers=1` | PASS, 369/369 |
| `git diff --check` | PASS before checkpoint |

No live Capture One or camera probe was run in this session. The automated suite used fakes and a disposable database.

## Manual try-out

1. Open a session in Session Manager. Capture One's browser should show that session's capture folder.
2. Drag a RAW file from Finder onto the workspace. The copied file should appear in the session's `Capture/<folder key>/` folder, in the app, and in Capture One's browser; the original should stay where it was. An adjusted preview should arrive after Capture One makes the variant.
3. If Settings → Capture One has a style name, inspect the new variant's Styles panel. It should carry that style once.
4. Browse another collection in Capture One, then import another RAW. Capture One should jump back to the session folder immediately.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
