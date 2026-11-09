---
plan: P57_camera_auto_take_back_and_run2_fixes
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-09
actor: opus (pipeline-reviewer)
---

# P57 re-review r2: APPROVED_WITH_NOTES

Fix 1 (`dc5769c`) changed test files only. It resolves both r1 findings. I re-ran both strengthened tests on the pre-P57 code `edb6f83`: both now **fail** there,
and they pass 3/3 on the fix. Suite, lint, fmt, typecheck and UI tests are green. No blocking item remains. One new note goes to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Review history
r1 (`763b997`) settled every production row by reading: auto take-back trigger and waiting sentence, setting off, camera tick on its own thread,
FIFO late-exposure match, no-rebuild output save, in-lane launch-gate re-check, FTP state, cached Wi-Fi endpoint. It also settled H1–H8 and the assumptions.
It blocked only on B2 for C2(a) (F2) and C4(a) (F1), because both tests passed on pre-P57 code. This round is scoped to those two tests and the testkit seam they use.

## Perimeter check
**Gate.** Tracker `IMPLEMENTED` with a fix-1 note. Checkpoint `dc5769cecc301c9daf6476a65b8202c90d890cef` exists (`CHECKPOINT (not approved): P57 fix r2 blocked work tests`).

`git show --stat dc5769c` lists 5 paths:
| Path | Allowed by |
|---|---|
| `crates/testkit/src/fake_capture_one.rs` | F1/F2 "if a hook is needed" |
| `crates/testkit/src/test_app.rs` | F1/F2 "if a hook is needed" |
| `crates/testkit/tests/p57_camera_auto_take_back_and_run2_fixes.rs` | F1/F2 |
| `docs/implementation/master_plan.md` (P57 row only) | fix closing protocol |
| `docs/implementation/plans/P57_…md` (Review log line) | fix closing protocol |

No production file and nothing outside the perimeter. Later commits `addfc0c` and `72083ae` are runner commits (handoff and prompt).

**Evidence (reviewer run, clean tree `72083ae`, code = `dc5769c`).**
- `cargo test --workspace`: 849 passed / 0 failed (56 groups).
- `npm test`: 57 files, 413/413.
- `npm run typecheck`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check`: all exit 0.
- P57 test file re-run 3× more: 8/8 each time, so the 200 ms deadline is not flaky here.
- Sidecar stub not re-run: no native file changed since r1's 28/0.

## Findings resolved

**Variation (scratch worktree `/tmp/p57rev2/wt` at `edb6f83`, removed after).** Setup: testkit `src/` taken from `dc5769c`, which back-ports cleanly because it touches only testkit. The `wait_for_*` helpers and the C2(a)/C4(a) bodies were copied verbatim (diff-checked) into a scratch `p57_variation.rs`. Run with `--test-threads=1` and a separate target dir.

### F1 (C4(a)): resolved
- **The row's input is now present.** Startup work is on. The fake's per-call delay is 1.5 s and `document_info` takes 3 s. The test waits until startup's `document_info` has started, queues one preview render, then saves a changed `output_format.quality`.
- **Pre-P57 `edb6f83`:** fails at `assert!(save_elapsed < Duration::from_secs(1))`. The old save rebuilds the app while the shared loop is held in the slow Capture One call. That is the F18 hang.
- **Fix:** green. After the save, the test waits for the queued preview's `process` and asserts exactly one, and that the format took 85.
- The correction asked for "keep ≤ 1 s and no-new-process; red on `edb6f83`, green on the fix". The ≤ 1 s half meets this. The no-new-process half is weaker than it looks; see Remaining findings N1.

### F2 (C2(a)): resolved
- **The cause the handoff found is now reproduced.** Startup holds the shared background loop in a 3 s `document_info`. Later calls are delayed 1.5 s. The test waits for `document_info_started`, injects one `connection.ready`, and asserts `Connected { link: Wireless }` within 200 ms.
- **Pre-P57 `edb6f83`:** fails (`line took 202.351917ms`). The status was not reached by the deadline because the camera tick is stuck behind the blocked loop.
- **Fix:** green, with the tick on its own thread.

**Seam read adversarially.**
- `test_app.rs` strips `fake_call_delay_ms` / `fake_document_info_delay_ms` from the behaviour JSON before writing `behaviour.json`, and sets them on the fake before the app starts. So startup sees the delays. Nothing leaks into the fake helper's script.
- `fake_capture_one.rs` adds `document_info_started` and `process_started:<v>:<r>` to the shared op log. I checked every op filter in `crates/testkit` that these could affect:
  - `contains("document_info")` (p43, p46): any call pushes both ops, so negatives are unchanged and positives become true at call start instead of at return.
  - `rposition(op == "document_info")` and `starts_with` pairs (p17): exact or prefix match on `document_info`; the new op does not match.
  - `starts_with("process:")`: excludes `process_started:`.
  - Exact op-vector equalities (p24, p48): their fixtures make no `document_info`/`process` call.
- Suite green confirms this.
- Over-engineering test: the tests synchronize on fake Capture One ops (the integration boundary), not on production internals. Pass.

## Triage
| Item | Classification | Reason |
|---|---|---|
| F1 (r1) | resolved | red on `edb6f83`, green on fix |
| F2 (r1) | resolved | red on `edb6f83`, green on fix |
| N1 (new): C4(a) re-render window | DROP → backlog | more test strength against a hypothetical regression. The F18 outcome is caught, and the production save path cannot render (r1, by reading). No user-visible wrong outcome |

No MUST. Verdict `APPROVED_WITH_NOTES`. Tracker set to `APPROVED`.

## Remaining findings
- **N1 (note, backlog).** C4(a) waits for the first `process:` op, sleeps 200 ms, then asserts exactly one. The fake pushes `process:` only after `check_id`'s 1.5 s call delay. So a second (re-render) `process` issued after the first would record ≥ 1.5 s later, outside the window. The no-new-process half therefore only catches a re-render issued concurrently with the first. Fix: count `process_started:` ops. Not blocking (see Triage).

## Carry-forward dispositions
| Note | Destination |
|---|---|
| N1 and the 7 r1 notes (mid-export format mix, late-exposure match can hide "not received", C5 race unreproduced, book/registry drift, FTP warning on cable, silent auto take-back failure + 20 ms polling, `FtpUnreachable` rank tie) | `docs/implementation/backlog.md` (P57 entries). Book/registry drift goes to the orchestrator. Hardware-observable items go to the next camera run (P52 run 3 / work Mac) |

## Lessons for the plans
- A fix that adds fake delays to make a test discriminating can quietly weaken its other assertions: count windows that were fine at zero delay stop working. When a fix changes a fixture's timing, check each assertion against that timing, not only the one the finding named.
- "Red on the pre-phase commit" worked as the acceptance standard for both findings and cost one scratch worktree. It belongs in plan rows that fix run-observed defects (repeated from r1).

## Mutation-probe declaration and write perimeter
- No production file edited, not even temporarily (flow §4).
- Files written: this handoff; `docs/implementation/master_plan.md` (P57 row → `APPROVED`); `docs/implementation/plans/P57_camera_auto_take_back_and_run2_fixes.md` (Review log entry); `docs/implementation/backlog.md` (1 note appended); `docs/implementation/narratives/P57_camera_auto_take_back_and_run2_fixes.md` (new).
- Scratch, outside the repo: `/tmp/p57rev2/` (logs, a separate cargo target dir, worktree removed with `git worktree remove --force` and pruned). `git worktree list` shows only the main tree. Before my doc writes, `git status --porcelain` was clean at `72083ae`.
- Committed nothing.
