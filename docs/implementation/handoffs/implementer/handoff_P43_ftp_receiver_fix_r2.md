---
plan: P43_ftp_receiver
role: fix
round: 2
state: DONE
date: 2026-11-06
actor: codex
---

# P43 FTP receiver — fix 1

## Finding → what changed → files

- **F1 (B4, reset fixture):** `FtpClient::stor_rst` now waits 300 ms after writing data, then closes with `SO_LINGER {1,0}`; removed `shutdown(SHUT_RDWR)` so the fixture sends RST without a preceding FIN. Files: `crates/testkit/src/ftp_client.rs`.
- **F2 (B1, PASV address):** PASV now advertises the accepted control socket's local IPv4 address instead of the configured listener bind address. The control connection is IPv4 in the receiver; the branch returns an I/O error if this assumption is ever broken. Files: `crates/infra/src/ftp/session.rs`.

## Divergences

None. The test file did not need an edit; the review explicitly required none for F2 and allowed the fixture-only correction for F1.

## Files changed

Checkpoint perimeter:
- `crates/testkit/src/ftp_client.rs`
- `crates/infra/src/ftp/session.rs`
- `docs/implementation/master_plan.md` (P43 tracker row only)
- `docs/implementation/plans/P43_ftp_receiver.md` (fix 1 Review log entry)

Runner-owned handoff, outside the checkpoint commit so it can record the resulting commit SHA: `docs/implementation/handoffs/implementer/handoff_P43_ftp_receiver_fix_r2.md`.

## Checkpoint SHA

`5677669` — `CHECKPOINT (not approved): P43 FTP receiver fix 1`.

## Stamp

- `cargo test --workspace` — passed; P43 FTP receiver integration suite 26/26.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo fmt --all -- --check` — passed.
- `cd apps/desktop && npm run typecheck` — passed.
- `cd apps/desktop && npm test` — passed, 50 files / 369 tests.
- `git diff --check` — passed.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
