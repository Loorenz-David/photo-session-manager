---
plan: P24_capture_one_provisioning
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-03
actor: opus
---
# Re-review P24_capture_one_provisioning, round 2: APPROVED_WITH_NOTES

Both blocking findings from round 1 are fixed as their corrections said. The fix stayed inside the allowed files.
Suite, lint and typecheck are green, and the suite no longer reaches Capture One (observed before and after the run).
A live run of the new `open_document.js` showed the throwaway-then-open form keeps the owner's document open.
Nothing blocking remains, so no triage was needed. One new note and one resolution go to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
- r1 (`1a4d277`) settled: perimeter of 45 paths; rows C1(a)–C1(f), C2(a)–(c), C3(a), C4(a), C5(a), C5(b); the
  ensure order and callers; settings; H1–H8 in production code; the four assumptions; C6(a) live. Open: F1 (B3,
  suite reaches live Capture One), F2 (B2, C4(b) cannot see the open-alongside form).
- r2 (this round) re-verifies only the fix's seam: the two p00 tests, `open_document.js`, `JxaScripting::open_document`,
  and the C4(b) open assertion.

## Perimeter check
- Gate: the tracker row was `IMPLEMENTED` with a fix-1 note, and checkpoint `79e5c44` exists.
- `git show --stat 79e5c44` lists four paths: `crates/integrations/src/editing/capture_one/jxa.rs`,
  `crates/integrations/src/editing/capture_one/jxa/open_document.js`, `crates/testkit/tests/p00_spike.rs`,
  `crates/testkit/tests/p24_capture_one_provisioning.rs`. That matches the union of F1's and F2's allowed files exactly.
  `test_app.rs` (optional for F1) is untouched. The runner commits `2d57bc0` (handoff, tracker row, Review log line)
  and `eed79ca` (this prompt) hold only process records. No extra path.
- Tree at review: HEAD `eed79ca`; `git status --porcelain` was empty before the run.

## Evidence (one run, as the prompt directs)
- Run on HEAD `eed79ca`, clean tree. The code is identical to the fix stamp's tree (`79e5c44`); the run is the
  prompt's mandated gate. It also adds new evidence for F1: Capture One's state was read before and after.
- `cargo test --workspace`: 445 passed, 0 failed. `cargo clippy --workspace --all-targets -- -D warnings`: clean.
  `cargo fmt --all -- --check`: clean. `npm run typecheck`: clean. `npm test`: 18 files, 129 passed. No B4.
- Capture One before and after the suite (JXA read of `documents.name()`, `currentDocument().id()`, the three hook
  properties): both reads were identical. Only `instagram pictures new camaera.cosessiondb` was open and in front, and
  the hooks pointed to `~/Library/Application Support/com.loorenz.sessionmanager/hooks/*.scpt`.
  `~/Pictures/Session Manager` does not exist after the run.

## Findings resolved
### F1 (B3), resolved
- `c1b_second_open_preserves_settings_and_migration` and `c4d_open_drains_file_left_while_closed` now call
  `App::open_with(paths, FakeCaptureOneScripting, ManualClock, false)` with `paths.home` set to the temp dir.
  `grep 'App::open('` over all `*.rs` now finds only the Tauri binary (`apps/desktop/src-tauri/src/lib.rs:160`).
  No test constructs the real `App`. The remaining `JxaScripting::new` uses in p16/p17/p23/p24 only write scripts
  for the node harness; they were settled earlier and never call `osascript` against Capture One.
- The tests still prove what they claimed. c1b checks settings bytes, mechanism version and migration version across
  two opens. c4d checks the spool drain on open, which still happens with `run_startup = false` (1 event row, file gone).
- Observed: the suite left Capture One untouched (Evidence above).

### F2 (B2), resolved
- `open_document.js` now runs `a.make({new:'document', withProperties:{name:'PSM Open Window', path:Path(argv[1])}})`
  and then `a.open(Path(argv[0]))` in one script. The error path guards `front &&` before `.name()`, which also
  closes backlog note "P24 `open_document.js`'s error path … `null`". `JxaScripting::open_document` creates the
  scratch folder, passes it as argv 2, and removes it after the call, whatever the outcome.
- C4(b) asserts `calls` equal exactly `[make {new 'document', withProperties {name 'PSM Open Window', path '/tmp/x/scratch'}},
  open '/tmp/x/Session Manager/Session Manager.cosessiondb']`. That is an exact, ordered equality, so dropping the
  throwaway `make`, reordering it after `open`, or opening the wrong path each makes the test fail. Judged by reading.
- **Live variation (new; the fix session could not run Apple events).** Capture One 16.8.6.25. Scripts were built as
  `JxaScripting::new` writes them (`lib.js` + script) in `/private/tmp/psm_r2_scripts`; scratch was
  `/private/tmp/psm_p24_live_r2/20261103-1935`.
  1. `make_document.js "$S" "PSM Live Session"` returned `D` in about 1.8 s; on disk: `.cosessiondb`, `Capture`,
     `Selects`, `Output`, `Trash`.
  2. Closed `D`, put the owner's document in front, then ran the new `open_document.js "<D>/PSM Live Session.cosessiondb"
     "$S/scratch-open-window"`. It exited 0 in about 3.9 s. `documents.name()` then read
     `[instagram pictures new camaera, PSM Live Session]`, and front was `D`. The throwaway was replaced (not listed),
     and the owner's document stayed open.
  3. Error-path variation: `open_document.js` on a file that does not exist. Capture One's `open` did **not** throw
     (exit 0, `null`), so the throwaway `PSM Open Window` stayed open and in front. See note N1.
  4. Restored: both probe documents closed, the owner's document in front. The read-back equals the pre-suite state
     (names, current, hooks). Scratch and script folders removed.

## Triage
No blocking item remains. flow §5 triage is empty: no MUST, no DROP.

| Item | Class | Reason |
|---|---|---|
| — | — | F1 and F2 resolved; N1 below is a note, not a B1–B4 finding |

## Remaining findings
None blocking. Notes written to `docs/implementation/backlog.md`:
- **N1 (P24, seen in passing, predates the fix).** Capture One's `open` does not throw when it cannot open the file
  (seen live with a missing file). In that case `open_document.js` returns success and the `PSM Open Window` throwaway
  stays open and in front, while Rust deletes its scratch folder. The ensure then times out after `inputs.wait`
  with `document_not_open` (provision.rs:187–200), which is correct, but the throwaway is left in Capture One.
  Round 1's code behaved the same way, so the fix did not introduce this. It is reachable only when an on-disk
  session refuses to open. Suggested correction: after `open`, if the front document's name starts with
  `PSM Open Window`, close it and throw.
- Resolution recorded: the r1 note on the `currentDocument()` null dereference is fixed by F2's `front &&` guard.

r1's other five notes stand unchanged.

## Lessons for the plans
- A fix session that cannot run Apple events should hand any changed JXA script to the re-reviewer for one live
  run. A harness models the dictionary; it cannot show what Capture One does on failure (N1 was found only live).

## Carry-forward dispositions
| Note | Destination |
|---|---|
| N1 (throwaway left after a silent `open` failure) | backlog, P24 entry; next phase touching `open_document.js` |
| r1 notes 1, 2, 4, 5, 6 | backlog, unchanged |

## Write perimeter and probe declaration
- Documents written: this handoff; `docs/implementation/master_plan.md` (P24 tracker row only);
  `docs/implementation/plans/P24_capture_one_provisioning.md` (one Review log line);
  `docs/implementation/backlog.md` (two appended rows);
  `docs/implementation/narratives/P24_capture_one_provisioning.md` (new).
- No production or test code edited, and no mutation probe was applied. No archgraph delta.
- Live Capture One side effects, all restored: two probe documents created and closed; front restored to
  `/Users/davidloorenz/Pictures/instagram pictures new camaera`; hooks never written. Removed scratch:
  `/private/tmp/psm_p24_live_r2/`, `/private/tmp/psm_r2_scripts/`, `/tmp/psm_r2_state.js`, `/tmp/psm_r2_test.log`.
  Other `/private/tmp/psm*` folders predate this session and were not touched.
- Commits: none (the runner commits).
