---
plan: R11_research_on_arranged
role: fix
round: 2
state: DONE
date: 2026-11-15
actor: sonnet
---
# Fix 1 — R11 Research starts when the session is arranged

## Finding → what changed → files
**F1 (B2, production): A25·C23(a) did not show `session_arrange_next` reaching the engine.**
The test `a25_c23a_new_commands_reach_wired_app_and_bindings` now builds a session whose photo is classified Main, leaves the prices intact, and calls `session_arrange_next` first. It asserts `next_session_id == null`, `research_queued == true`, `arranged_at` set, and exactly one run for the session with state `queued` and trigger `arranged`. After that it writes the emptied price table to the settings file and runs the unchanged reset half (prices restored, saved file equals the response, bindings checks).
Files: `apps/desktop/src-tauri/src/lib.rs` (that test only).

Check that the test can fail: with the photo classified Condition (so no Main photo), the run fails at the `research_queued` assertion (`left: false, right: true`). Reverted; the file was copied back and the diff holds only the intended change.

## Divergences
None. Judgment call: the emptied prices are written to the settings file directly after the arrange call, because the settings live in a file and the commands read it at call time.

## Files changed
- `apps/desktop/src-tauri/src/lib.rs` (test only)
- `docs/implementation/plans/R11_research_on_arranged.md` (Tracker line, Review log entry)
- `docs/implementation/handoffs/implementer/handoff_R11_research_on_arranged_fix_r2.md` (this file)

Probe files touched (applied and reverted): `apps/desktop/src-tauri/src/lib.rs` only (a one-word change of the classification in the test, restored).

## Checks
`cargo test -p psm-desktop` 25 green · `cargo clippy --workspace --all-targets -- -D warnings` green · `cargo fmt --all -- --check` green · `npm run typecheck` green. The full suite was not re-run: no shared code changed (test-only edit).

## Checkpoint SHA
See the `CHECKPOINT (not approved):` commit on this branch (`git log -1 --format=%h -- docs/implementation/handoffs/implementer/handoff_R11_research_on_arranged_fix_r2.md`).

## Stamp
Tree clean after the checkpoint commit; psm-desktop 25/25, no failure delta.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
