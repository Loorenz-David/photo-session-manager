---
plan: P43_ftp_receiver
role: review
round: 2
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-06
actor: opus
---

# P43 FTP receiver: re-review r2 (after fix 1)

Fix 1 resolved both r1 findings exactly as their corrections said. The reset fixture is now the lab's shape, and the C2 rows were green
in all 12 of my runs (r1: about 50 % red). PASV now advertises the address the camera reached. The Rust suite is **still red at random**
for a different reason, **F3**: `c7b_port_reopens_in_place` waits only 3 s for the fake sidecar's pid. When many apps start in
parallel, macOS queues the first exec of each fresh sidecar copy at about 125 ms, and the wait runs out. It failed 3 of 12 runs. The
receiver is not at fault. It is a one-file test fix. Under flow.md §5 a red suite is **MUST**, so the verdict is `CHANGES_REQUESTED`
with that single item. The orchestrator decides whether to authorise fix 2.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Review history

- r1 (CHANGES_REQUESTED): F1 (B4, testkit `stor_rst` sent FIN before RST and skipped the lab's pause, so the suite was ~50 % red), F2 (B1,
  PASV advertised `0,0,0,0` under the production bind). 24 of 27 rows and all non-delta areas were verified correct and are settled.
  11 backlog notes, including row "`c7b` flaked once … lengthen the bound if it recurs after the C2 fix".
- fix 1 (codex, `5677669`): F1 and F2, no declared divergence.

## Perimeter check

- Gate: tracker `P43_ftp_receiver | IMPLEMENTED` with a fix note; `5677669` ("CHECKPOINT (not approved): P43 FTP receiver fix 1") is in
  `git log`. Passes.
- `git show --stat 5677669`: `crates/infra/src/ftp/session.rs` (F2's allowed file), `crates/testkit/src/ftp_client.rs` (F1's allowed
  file), `docs/implementation/master_plan.md` (P43 row only, checked in the diff), `docs/implementation/plans/P43_ftp_receiver.md` (one
  Review log entry). 4 paths, equal to the fix handoff's list. The optional test file was not touched. **Nothing outside the perimeter.**
- `git diff 5677669 HEAD`: only the runner's fix handoff and this round's prompt. Code tree at review = `5677669`. HEAD `6956b7c`,
  `git status --porcelain` empty at start.

## Evidence (tree `6956b7c`, clean; code = `5677669`)

| Hypothesis | Scope / command | Result |
|---|---|---|
| suite green (prompt: run once) | L4 `cargo test --workspace` | run 1: `c7b_port_reopens_in_place` panicked at `p43_ftp_receiver.rs:85` (`settle`). My `grep \| sort \| head` filter hid the `FAILED` line and zsh lost the exit code, so I re-ran for recovery, not variation. Run 2: exit 0, 44 binaries ok |
| lint / format / types / UI | `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `npm run typecheck` · `npm test` | 0 · 0 · 0 · 50 files / 369 tests |
| F1 gone and `c7b` rate (recorded before running: one run cannot measure a flake) | L1 `cargo test -q -p psm-testkit --test p43_ftp_receiver` ×10 | 8 green; 2 red, both `c7b` at line 85. C2(a)–(g): 0 failures in 10 runs (plus 2 full runs) |
| `c7b` load-dependent? | L1 `… p43_ftp_receiver c7b` ×15 | 0/15 red |
| late or failed sidecar spawn? | scratch probe `crates/testkit/tests/zz_review_probe_p43r2.rs` (24 threads × `TestApp::with_camera()`, time until `pids` non-empty, 30 s cap), ×3 | every sidecar started. Latencies rise by ~125 ms per app, max 3000 / 3136 / 3041 ms. Probe deleted afterwards |

## Findings resolved

- **F1: resolved as its correction said.** `stor_rst` is now `write_all(bytes)` → `sleep(300 ms)` → `SO_LINGER {1,0}` → `drop`, and
  `shutdown(SHUT_RDWR)` is removed. That is the lab's `_interrupted_upload` sequence (sendall, sleep 0.3, linger-close) and plan Ordered
  task 5's wording. C2(a)–(g), including C2(d)/(g) which open with `stor_rst`, were green in all 12 runs.
- **F2: resolved as its correction said.** The `227` octets come from `control.get_ref().local_addr()?.ip()`. Under a `0.0.0.0` bind
  that is the interface address the camera already reached. The passive listener still binds `SocketAddr::new(config.bind, port)`, so
  it accepts on that interface, and `accept_data` still checks the data peer against the control peer. The EPSV reply is unchanged (port
  only). Loopback tests still get `127,0,0,1`. No test was required (r1). Verified by reading. Passing side effect: a non-IPv4 control
  connection now gets an `io::Error` that ends the session, where it used to fall back to `127,0,0,1`. This is unreachable while the
  bind is IPv4 (backlog note).

## Remaining findings

### F3: B4: route production (test fixture; no tester role): `c7b` pid wait too short under parallel load, suite red 3/12
- **What is wrong.** `crates/testkit/tests/p43_ftp_receiver.rs::c7b_port_reopens_in_place` line 462 waits for
  `a.fake_sidecar_recording().pids` with the local `settle` (300 × 10 ms = 3 s). `TestApp::with_camera()` copies the fake sidecar
  binary into each app's temp dir, and the runtime spawns it in `bind`. The probe shows macOS serialises the first exec of each fresh
  copy (~125 ms each). The binary runs 14 test threads, every P43 test builds a camera app, and the C1/C2 rows add load. So when `c7b`'s
  app lands behind enough queued first execs, its pid arrives after 3 s and the test panics. The receiver and runtime are correct.
  Every sidecar starts; it starts late.
- **Observed.** 3 red runs out of 12 (full workspace 1/2, binary 2/10), all at line 85 from `c7b`. 0/15 alone.
- **Authority.** flow.md §5 B4 (suite red). r1 backlog row "P43 (suite flake) … lengthen the bound if it recurs after the C2 fix": it
  recurs.
- **Correction.** In `c7b`, wait for `pids` non-empty with a bound that covers the queue, e.g. 15 s. A `settle_for(Duration, pred)`
  variant or an inline loop both work. Keep the other 3 s settles unchanged. No production change. The row's assertions stay as they
  are.
- **Files the fix may touch.** `crates/testkit/tests/p43_ftp_receiver.rs` only.

## Triage (flow.md §5)

| Item | Class | Why |
|---|---|---|
| F3 `c7b` 3 s pid wait (suite red ~25 %) | **MUST** | red suite. Every later phase's implementer and reviewer runs `cargo test --workspace` and would get false B4s at this rate (r1's F1 cost a fix cycle the same way). One-file test fix |

No DROP items. Verdict `CHANGES_REQUESTED` (MUST: F3).

## Verified correct (this round)

- Perimeter = exactly the files F1/F2 allowed; the tracker diff touches only the P43 row.
- F1/F2 as above. Clippy, fmt, typecheck and the UI suite are green on the review tree.
- Settled in r1 and not re-verified: the other 24 rows, completion/failure split, staging, refusal order, namespace, network/auth,
  runtime wiring, settings, H1–H8, secrets. Nothing seen wrong in passing beyond F3 and the two backlog notes.

## Notes written to backlog (2)

- P43 r2 → P42/testkit: the per-app sidecar copy serialises first execs (~125 ms each). P42's 3 s ticking `settle` has the same exposure.
  The root fix is a single per-process copy or a hard link.
- P43 r2: PASV on a non-IPv4 control connection now ends the session. Decide together with the S7 dual-stack question.

## Lessons for the plans

- A test that waits for a spawned helper should size its wait to the parallel test load, not to one process's start-up time. On macOS
  the first exec of a freshly written binary is serialised. The `TestApp` fixture (master plan §7 testkit) should state its pid-wait
  bound, or copy the sidecar once per process.
- A re-review prompt that says "run the suite once" cannot detect a flake. For suites whose last round had a random red, the coordinator
  should authorise a repeat budget up front.

## Write perimeter

- `docs/implementation/master_plan.md`: P43 tracker row only (→ `CHANGES_REQUESTED`).
- `docs/implementation/plans/P43_ftp_receiver.md`: re-review r2 entry appended to the Review log.
- `docs/implementation/backlog.md`: 2 rows inserted before the P43 H7 lesson row.
- `docs/implementation/handoffs/reviewer/handoff_P43_ftp_receiver_review_r2.md`: this file.
- **Probe file (created, run 3×, deleted, never committed):** `crates/testkit/tests/zz_review_probe_p43r2.rs`. After deletion,
  `git status --porcelain` was empty. Production code and existing tests were not touched. Loopback and temp dirs only, with no camera
  and no Capture One.
- Untracked: `target/` build artefacts; logs in `/tmp/p43r2/` (outside the repo).
- No narrative (written at approval). No commit.
