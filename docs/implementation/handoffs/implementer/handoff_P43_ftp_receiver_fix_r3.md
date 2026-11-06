---
plan: P43_ftp_receiver
role: fix
round: 3
state: DONE
date: 2026-11-06
actor: codex
---

# P43 FTP receiver fix r3

## Finding → what changed → files

- **F3 (B4, c7b sidecar PID wait):** `c7b_port_reopens_in_place` now waits up to 15 seconds for the fake sidecar PID. All other 3 second settles and all existing assertions remain unchanged. Added `settle_for` with the existing 10 ms polling cadence; the bound covers the reviewer's measured ~3.1 second queue with margin.
- Files for F3: `crates/testkit/tests/p43_ftp_receiver.rs` only.

## Divergences

None. Fix 2 authorization was recorded in the Review log before this session. No extra behaviour or test assertions were added.

## Files changed

- `crates/testkit/tests/p43_ftp_receiver.rs` — F3 correction.
- `docs/implementation/master_plan.md` — P43 tracker row to `IMPLEMENTED` (fix 2).
- `docs/implementation/plans/P43_ftp_receiver.md` — fix 2 Review log entry, including the bounded-wait judgment and validation.
- `docs/implementation/handoffs/implementer/handoff_P43_ftp_receiver_fix_r3.md` — this runner handoff; excluded from the checkpoint commit because it records that commit's SHA.

## Checkpoint SHA

`7e8178dcf422e2aeee52b06ddab8727b3be30148` — `CHECKPOINT (not approved): P43 FTP receiver fix 2`.

## Stamp

- `cargo test -q -p psm-testkit --test p43_ftp_receiver` ×10 — all 10 runs passed, 26/26 each.
- `cargo test --workspace` — exit 0; all workspace tests and doc tests passed, including P43 26/26.
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
- `cargo fmt --all -- --check` — exit 0.
- `cd apps/desktop && npm run typecheck` — exit 0.
- `cd apps/desktop && npm test` — exit 0; 50 files, 369 tests.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
