---
plan: P54_camera_cable_and_take_back
role: fix
round: 2
state: DONE
date: 2026-11-08
actor: codex
---

# P54 fix 1 handoff

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Finding → what changed → files

- **F1 (B1, production).** Registered a best-effort helper `usb-retry` command on the process `StartOrder`; the shared relaunch wait sends it once at the start when a cable is present and USB is not ready, then preserves the existing bounded wait. The runtime logs a send error without interrupting that wait. Files: `crates/integrations/src/camera/direct/start_order.rs`, `crates/integrations/src/camera/runtime.rs`.
- **F2 (B2, production).** Replaced direct `StartOrder::camera_state` manipulation in C2(a) with fake-helper `usb.initial`, `session.closed`, `usb.claimed_by`, and `connection.ready` events. The test asserts the single retry is received before launch and covers timeout. C2(b) now feeds `usb.initial` true for ready USB and false for no cable, and asserts no retry. File: `crates/testkit/tests/p54_camera_cable_and_take_back.rs`.

## Divergences

None. The runtime registers the retry callback on `StartOrder` with a weak runtime reference so the existing shared wait can issue the helper command without changing the Capture One gate wiring.

## Files changed

- `crates/integrations/src/camera/direct/start_order.rs`
- `crates/integrations/src/camera/runtime.rs`
- `crates/testkit/tests/p54_camera_cable_and_take_back.rs`
- `docs/implementation/plans/P54_camera_cable_and_take_back.md`
- `docs/implementation/master_plan.md` (P54 tracker row only)
- `docs/implementation/handoffs/implementer/handoff_P54_camera_cable_and_take_back_fix_r2.md`

## Checkpoint SHA

This handoff is included in the checkpoint commit; its SHA is reported in the session handoff message to avoid a self-referential hash in this file.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | Green; P54 integration file 6/6 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Green |
| `cargo fmt --all -- --check` | Green |
| `cd apps/desktop && npm run typecheck` | Green |
| `cd apps/desktop && npm test` | First run: known intermittent `p33_split_grid` C4(d), 406/407; rerun: 407/407 across 54 files |
| `tools/sidecar/build.sh --stub --test` | Green, 22 tests / 0 failed |
| `cargo test -p psm-testkit --test p54_camera_cable_and_take_back c2 -- --nocapture` | Green, 2/2 |

No live camera or Capture One probe was run; the suite uses the fake helper and fake editor.
