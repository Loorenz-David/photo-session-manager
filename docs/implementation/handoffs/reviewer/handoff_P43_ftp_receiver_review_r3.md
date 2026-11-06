---
plan: P43_ftp_receiver
role: review
round: 3
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-06
actor: opus
---

# P43 FTP receiver: re-review r3 (after fix 2)

Fix 2 resolved F3 exactly as its correction said. `c7b` now waits up to 15 s for the fake sidecar pid. Every other wait and every
assertion is unchanged, and nothing outside the one allowed file changed. The full suite, clippy, fmt, typecheck and UI are green on
the review tree. The r2 failure mode (red at the 3 s bound, 3 of 12 runs) did not occur in 21 runs. One run of 20 of the `p43` binary was
still red. The cause is a different, rarer mechanism: a ~70 s OS stall while freshly copied fake sidecars were starting. It held three
apps at once, and the receiver was not involved. Triaged **DROP** (reasoning below) and written to the backlog for P44/testkit. No MUST
remains → **APPROVED_WITH_NOTES**. Tracker → `APPROVED`; narrative written.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Review history

- r1 (CHANGES_REQUESTED): F1 (B4, testkit `stor_rst` FIN before RST), F2 (B1, PASV advertised `0,0,0,0`). 24 of 27 rows and all
  non-delta areas verified and settled. 11 backlog notes.
- fix 1 (`5677669`) → r2: F1 and F2 resolved and settled. New F3 (B4, MUST): `c7b` 3 s pid wait was red 3/12 under the per-app
  sidecar-copy exec queue.
- Orchestrator: `FIX2_AUTHORISED` (Review log), re-reviewer to repeat the `p43` binary ≥10×.
- fix 2 (`7e8178d`): F3 only. This round checks only that delta.

## Perimeter check

- Gate: tracker `P43_ftp_receiver | IMPLEMENTED` with the fix 2 note; `7e8178d` ("CHECKPOINT (not approved): P43 FTP receiver fix 2")
  in `git log`; `FIX2_AUTHORISED` present in the plan's Review log. Passes.
- `git show --stat 7e8178d`: `crates/testkit/tests/p43_ftp_receiver.rs` (the one file F3 allowed),
  `docs/implementation/master_plan.md` (diff: P43 row only), `docs/implementation/plans/P43_ftp_receiver.md` (one Review log line).
  3 paths, equal to the fix handoff's list (its own handoff is excluded by design). **Nothing outside the perimeter.**
- `git diff 7e8178d HEAD --stat`: only the runner's fix handoff and this round's prompt. Code tree at review = `7e8178d`. HEAD `1818599`,
  `git status --porcelain` empty at start.

## Evidence (tree `1818599`, clean; code = `7e8178d`)

| Hypothesis | Scope / command | Result |
|---|---|---|
| suite green (prompt: run once) | L4 `cargo test --workspace` | exit 0; 44 `test result: ok` lines, 634 passed, 0 failed |
| lint / format / types / UI | `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `npm run typecheck` · `npm test` | 0 · 0 · 0 · 50 files / 369 tests |
| F3 flake gone (orchestrator ruling: ≥10×; one run cannot measure a flake) | L1 `cargo test -q -p psm-testkit --test p43_ftp_receiver` ×10 | 9 green (21.4–26.8 s). Run 8 red: `c7b` panicked at `:96` `condition did not settle within 15s`; run took 96.75 s; `c7a` and `c7c` reported "has been running for over 60 seconds" and passed |
| run 8 an isolated stall or a recurring rate? (written before running: one red in 10 cannot tell) | same ×10 (runs 11–20) | 10/10 green, 22.5–26.0 s each, no "running for over" lines |
| receiver/settings implicated in the stall? | reading `runtime.rs` `bind`/`shutdown`, `supervisor.rs` `spawn`/`quit_and_wait`, `fake_sidecar.rs` `main` | no (Remaining findings, R1) |

The system log (`log show`) was not readable from this session, so the OS-side cause is inferred from the code paths, not logged.

## Findings resolved

- **F3: resolved as its correction said.** A new local helper `settle_for(timeout, pred)` polls every 10 ms until a deadline and then
  runs a final `assert!(f(), …)`, so a pid that appears at the deadline still passes. `c7b` uses it with `Duration::from_secs(15)` for
  `!fake_sidecar_recording().pids.is_empty()` only. The old `settle` (300 × 10 ms) is unchanged and still serves every other wait. The
  four row assertions (old port refuses, `receiver_port() == free`, generation + 1, pids unchanged) are byte-identical, so C7(b)
  still fails on a wrong outcome. No production change. No declared divergence. The r2 signature (`settle` panic at ~3 s) occurred 0
  times in 21 runs (r2: 3/12).

## Remaining findings

### R1 (candidate B4): `p43` binary red 1/20 on a ~70 s fake-sidecar start stall
- **Observed.** Run 8 of 20: `c7b` exceeded its 15 s pid wait. The run took 96.75 s against 21.4–26.8 s for the other 19. `c7a` and
  `c7c`, which were started at about the same time, ran for over 60 s and passed. The full workspace run was green.
- **Why the receiver is not involved (by reading).** The fake sidecar writes its pid as its first act in `main`, so an empty `pids`
  after 15 s means the process had not started running. `c7a`/`c7c` do not wait on the sidecar. `settings_set` reaches `bind`, which
  returns once a supervisor exists, and takes no sidecar wait. Their extra minute fits `TestApp` drop → `CameraRuntime::shutdown` →
  `quit_and_wait` → `kill` + `wait` on a child that has not finished starting. So three apps whose freshly copied sidecars were being
  exec'd together were held ~70 s by the OS. This is the same root as r2's backlog row (each `TestApp::with_camera` execs a new file),
  but on a far longer tail than the ~125 ms/app queue F3 sized for.
- **Authority if it were blocking.** flow.md §5 B4.

## Triage (flow.md §5)

| Item | Class | Why |
|---|---|---|
| R1 OS stall of fresh fake-sidecar execs, `p43` binary red 1/20 | **DROP** | Nothing a user or the camera can observe: production installs the sidecar once and the receiver does not depend on it. It is not a defect of the reviewed delta, which did what F3 asked and removed the 3/12 failure. The mechanism belongs to the shared testkit (per-app sidecar copy), and P42's approved camera tests share it. Within this phase, r1 treated a single unexplained `c7b` flake as a backlog note, and r2 escalated only when it recurred at ~25 %. Any bound large enough to cover a 70 s stall would also be outside the plan's 3 s settle convention. The real remedy (one sidecar copy per test process) is a testkit change for the next phase that touches it. → backlog, routed P44/testkit, with the signature so a later reviewer does not raise a false B4 |

No MUST. Verdict `APPROVED_WITH_NOTES`. The orchestrator can override this DROP. If it does, the fix budget is spent (two fixes), so
the remedy would have to be scheduled as P44 work, not as a third P43 fix.

## Verified correct (this round)

- Perimeter = exactly the one file F3 allowed, plus the fix's own tracker row and log line.
- F3 as above. Workspace suite, clippy, fmt, typecheck and UI are green on `1818599`.
- Settled earlier and not re-verified: the 24 r1 rows, F1, F2, completion/failure split, staging, refusal order, namespace,
  network/auth, runtime wiring, settings, H1–H8, secrets. Seen in passing: `c7b` still uses the file-local `settle`, not P42's ticking
  `settle(a, pred)` (plan L7). It is already in the backlog (r1 flake row) and does not affect the outcome, so no new note.

## Notes written to backlog (1 new, 1 updated)

- Updated: the r1 "P43 (suite flake)" row is marked `✓ resolved by P43 fix 2` (15 s bound), with a pointer to the new row.
- New: "P43 r3 → P44/testkit (run-observed)": the stall signature (`c7b` 15 s panic, ~97 s run, `c7a`/`c7c` "running for over 60
  seconds"), why the receiver is not involved, and the remedy: copy the fake sidecar once per test process or hard-link it.

## Carry-forward dispositions

| Open item | Destination |
|---|---|
| Fresh-exec stall of per-app fake sidecar copies (r2 row + r3 row) | P44 (next phase built on `TestApp::with_camera`) / testkit |
| All r1/r2 P43 notes | unchanged destinations as written in `backlog.md` (P44, P46, P47, P50, P52, planner) |

## Lessons for the plans

- When a test's waits depend on spawning a helper binary, the testkit fixture should spawn **one** copy per process. Per-app copies
  have two tails on macOS: a ~125 ms serialised queue (r2) and a rare stall of over a minute (r3). No per-test bound handles both.
  Master plan §7 testkit `TestApp::with_camera` is the place to state this.
- The orchestrator's "repeat ≥10×" ruling caught this. A flake budget should be sized to the rate it must rule out: 10 runs see a
  5 % flake with only ~40 % probability. Here the second 10 supplied the rate.

## Write perimeter

- `docs/implementation/master_plan.md`: P43 tracker row only (→ `APPROVED`).
- `docs/implementation/plans/P43_ftp_receiver.md`: re-review r3 entry appended to the Review log.
- `docs/implementation/backlog.md`: r1 flake row marked resolved; 1 row inserted before the "P43 r2 (by reading)" PASV row.
- `docs/implementation/narratives/P43_ftp_receiver.md`: new (owner narrative).
- `docs/implementation/handoffs/reviewer/handoff_P43_ftp_receiver_review_r3.md`: this file.
- No mutation probes, no scratch test files, no production or test edits. Logs in `/tmp/p43r3/`, outside the repo. Untracked
  `target/` build artefacts. Loopback and temp dirs only; no camera, no Capture One.
- No commit.
