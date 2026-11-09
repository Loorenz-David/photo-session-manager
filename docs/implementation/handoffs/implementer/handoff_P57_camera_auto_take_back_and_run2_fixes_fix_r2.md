---
plan: P57_camera_auto_take_back_and_run2_fixes
role: fix
round: 2
state: DONE
date: 2026-11-09
actor: codex
---

# P57 fix r2 — reviewer findings

## Finding → what changed → files

### F1 (B2), C4(a)

Configured the test app with startup work enabled and delayed fake Capture One calls. The test queues an existing photo preview while the startup document call is blocked, saves a changed output quality, checks the save returns within one second, then waits for the preview queue to resume and asserts exactly one `process` call. This makes a rebuild-on-save regression observable after the slow call finishes.

On the pre-P57 `edb6f83` scratch worktree, with the startup/delay fixture back-ported, the isolated C4(a) case failed its ≤1 second save assertion (observed elapsed time exceeded one second). On the P57 fix, it passed and recorded one preview process.

Files: `crates/testkit/tests/p57_camera_auto_take_back_and_run2_fixes.rs`, `crates/testkit/src/test_app.rs`, `crates/testkit/src/fake_capture_one.rs`.

### F2 (B2), C2(a)

Configured startup with delayed fake `document_info` and waited for its started marker before injecting the single `connection.ready` line. The test asserts Wireless status within the criterion's 200 ms while the shared background loop remains blocked.

On pre-P57 `edb6f83`, the isolated C2(a) case failed at the status deadline (`line took 200.369917ms`). On the P57 fix, it passed within 200 ms.

Files: `crates/testkit/tests/p57_camera_auto_take_back_and_run2_fixes.rs`, `crates/testkit/src/test_app.rs`, `crates/testkit/src/fake_capture_one.rs`.

## Divergences

None. No production code changed.

## Files changed

- `crates/testkit/src/fake_capture_one.rs` — records start markers for fake document and process calls so the tests synchronize on active blocking work.
- `crates/testkit/src/test_app.rs` — adds startup fixture configuration for fake Capture One call delays before the app starts.
- `crates/testkit/tests/p57_camera_auto_take_back_and_run2_fixes.rs` — strengthens C2(a) and C4(a) at their stated boundaries.
- `docs/implementation/master_plan.md` — P57 tracker state → `IMPLEMENTED` (fix 1).
- `docs/implementation/plans/P57_camera_auto_take_back_and_run2_fixes.md` — fix 1 Review log entry.
- `docs/implementation/handoffs/implementer/handoff_P57_camera_auto_take_back_and_run2_fixes_fix_r2.md` — this handoff.

## Checkpoint SHA

`dc5769cecc301c9daf6476a65b8202c90d890cef` — `CHECKPOINT (not approved): P57 fix r2 blocked work tests`.

## Stamp

- Pre-P57 baseline (`edb6f83`, scratch worktree, fixture back-ported): C2(a) failed its 200 ms status assertion; C4(a) failed its ≤1 s settings-save assertion.
- `cargo test --workspace`: passed, 849 tests, 0 failures.
- `npm test`: passed, 413/413.
- `npm run typecheck`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `tools/sidecar/build.sh --stub --test`: passed, 28/28.
- Targeted P57 test file: 8/8 passed as part of the workspace suite.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
