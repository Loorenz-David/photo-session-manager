---
plan: P51_camera_setup_ui
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-08
actor: opus
---
# Re-review P51_camera_setup_ui, round 2 (after fix 1): APPROVED_WITH_NOTES

F1, the only blocking finding from round 1, is resolved as its correction described. A status-only fingerprint card no longer
shows a placeholder in the value slot, and it offers neither Confirm nor Not mine until `onFingerprintPending` delivers the real
value. The fix stayed inside its allowed files. The extended C6(c) test fails on the round-1 code. Suite, lint and typecheck:
green (the one red was the known intermittent `p33_split_grid` case, recorded below). No blocking item remains, so the triage
table is empty. Two by-reading notes on pre-existing behaviour went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
Round 1 (code tree `d8d6993`) settled C1(a–f), C2(a–d), C3(a–c), C4(a–c), C5(a–c), C6(a–b), C7(a) and C8(a–c), the wire values,
the P07/P19 edits and H1–H8. It raised F1 (B1) and backlog notes N1–N4. This round covers only the fix seam (C6(c),
`CameraSection.tsx`) plus anything seen in passing.

## Gate
Tracker row `IMPLEMENTED` (2026-11-08, codex, "fix 1: F1 resolved…"). Fix checkpoint `6145db7` (`CHECKPOINT (not approved): fix
P51 fingerprint confirmation state`) is in `git log`. HEAD `e6803e2` = `6145db7` + the runner's re-review prompt.
`git status --porcelain` was clean at the start.

## Perimeter check
`git show --stat 6145db7` lists 5 paths, and they match the fix handoff's "Files changed" exactly:
- `apps/desktop/src/features/settings/CameraSection.tsx`: allowed by F1.
- `apps/desktop/src/features/settings/__tests__/p51_camera_section.test.tsx`: allowed by F1.
- `docs/implementation/plans/P51_camera_setup_ui.md`: the C6(c) row was rewritten and a Review log entry was added.
- `docs/implementation/master_plan.md`: only the P51 tracker row (1 line).
- `docs/implementation/handoffs/implementer/handoff_P51_camera_setup_ui_fix_r2.md`.

No getter-route file changed (`services/camera.ts`, `commands/camera.rs`, `lib.rs`, `bindings.ts`, `adapter.rs`), which is
consistent with the declared frontend-only route. The C6(c) cell edit is a criterion edit by the fix role. It carries out F1's
"Extend C6(c)" correction word for word, so it is accepted, not a finding. The coordinator may want to confirm that a fix session
should edit criterion cells. That is a process question, not B1–B4.

## Evidence (reviewer run, code tree `6145db7` = HEAD minus the prompt; no tree changes during the run)
| Command | Result |
|---|---|
| `npm test` (apps/desktop), run 1 | exit 1: 400/401; failing ID `p33_split_grid.test.tsx > C4(d) the stage bar reads the places…` |
| `npx vitest run` ×3 (failing-ID capture) | run 1: 401/401 · run 2: 400/401, same `p33_split_grid` C4(d) ID · run 3: 401/401 |
| `npx vitest run` on the three p51 files | 3 files / 32 tests passed |
| `npm run typecheck` | exit 0 |
| `cargo test --workspace` | exit 0; every `test result` line `ok`, 0 failed (log kept the last 40 lines only, so no total) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |

The `p33_split_grid` C4(d) case is the plan-acknowledged intermittent (plan Notes: "if it is red in one full run and green on a
rerun, record both"; backlog). It is not in P51's perimeter, the fix touched nothing it imports, and it passed in 2 of 3 reruns.
This is not B4.

## Findings resolved
### F1 (B1, production): Confirm offered without the fingerprint. **Resolved.**
- **Code.** In `CameraSection.tsx`, a `fingerprint_unconfirmed` or `fingerprint_mismatch` status now sets
  `fingerprintNeedsAction` and no longer writes a literal into `fingerprint`. The card renders when
  `fingerprint || fingerprintNeedsAction`. The `<code>` value, Confirm and Not mine render only in the `fingerprint ?` branch. The
  other branch renders `The fingerprint will appear when the camera reports it.` The title still follows the status
  (`Fingerprint changed` for a mismatch). Confirm and Not mine clear both states. So no path offers either action without a real
  value: neither button is reachable unless `fingerprint` is set, and the only writer of `fingerprint` is the
  `onFingerprintPending` handler.
- **The route works in practice.** F1 accepted the frontend-only route on the premise that the value is reported again. I checked
  that premise in the code and did not take it from the round-1 prose. `camera_connection.cpp:466–473`: when the fingerprint check
  fails, `r.fatal = false` ("Retry on the sidecar's normal backoff"). Each attempt runs `check_fingerprint` again and emits
  `ssh.fingerprint {status: unknown|MISMATCH, fingerprint}` (`:321`, `:327–328`). `DirectCameraAdapter::apply` pushes it to
  `fingerprint_pending` (`adapter.rs:420–428`), and `CameraRuntime::perform` emits `AppEvent::FingerprintPending` for every one,
  with no dedupe (`runtime.rs:770–777`). The backoff is capped at `backoff_max_ms = 30000` (`transport.hpp:92`). An open status-only
  card therefore receives the real value within about 30 s. Repeated events overwrite one state value, so nothing stacks (backlog
  P50→P51 "treat repeats as the same pending fingerprint" holds).
- **Authority.** 10E MC-31 §Fingerprint ("shows the camera's fingerprint … pinned only on their confirmation"): met. The
  implement-r1 assumption ("status without a pending event shows a comparison prompt") is withdrawn by this fix, so no assumption
  contradicts the intention.
- **Test (C6(c), rewritten).** `it.each` over both reasons, each with its expected title. It asserts: the title is present; the
  waiting sentence is present; Confirm and Not mine are absent (`queryByRole`, exact names); the old literal is absent. Then,
  after `fingerprintEvent({fingerprint:'ab:cd:ef'})`: the value renders in `CODE` and both buttons appear. **It can fail.** Read
  against the round-1 code, the Confirm-absent assertion fails (the literal filled `fingerprint`, which rendered Confirm), and so
  does the literal-absent assertion. A version that never renders the card fails `findByText(title)`. A version that never shows
  the buttons after the event fails the last two assertions. The mismatch row also pins the title per status. The test checks
  only rendered output, so it passes the over-engineering test.

## Triage
No blocking item remains, so there is nothing to triage.

| Item | Class | Classification | Reason |
|---|---|---|---|
| none | | | |

## Remaining findings
None blocking. The two notes below were seen in passing while reading the changed seam. Both are behaviour that already existed
at round 1, and neither is B1–B4. They were appended to `docs/implementation/backlog.md`.
- **N5 (note, by reading).** `fingerprintChanged` is set only from the mount-time `cameraStatus`, and the event payload has no
  mismatch flag. A `MISMATCH` that arrives while Settings → Camera is already open is therefore titled `Confirm camera
  fingerprint`, not `Fingerprint changed`. Confirm still sends `replace=1`, because the adapter keeps the flag
  (`adapter.rs:604–608`, `:278–289`), and the dot shows the mismatch sentence. What is wrong is the title only. Remedy: carry
  `replace` in the event, or re-read `cameraStatus` when the event arrives.
- **N6 (note, by reading).** `Not mine` calls `cameraFingerprintReject`, which only checks that a fingerprint is pending
  (`adapter.rs:290–295`). It tells the sidecar nothing, so the sidecar keeps retrying and the card comes back at the next attempt
  (≤ 30 s). The photographer has no way to say "stop trying this access point". MC-31 does not define rejection beyond not
  pinning, which holds. This belongs to P52's hardware rows or a later backend phase.

## Carry-forward dispositions
| Note | Destination |
|---|---|
| N1–N3 (round 1) | next frontend polish phase (unchanged) |
| N4 (round 1, R-I32) | P52 or a later backend phase adding the precondition query (unchanged) |
| N5 | next frontend polish phase |
| N6 | P52 (hardware acceptance, fingerprint rows) or a later backend phase |

## Lessons for the plans
- A fix finding that says "Extend C6(c)" leaves open who edits the criterion cell. This time the fix role edited it, faithfully.
  Fix prompts should say whether the fix session may edit criterion cells or must leave that to the coordinator.
- When a correction depends on a premise about runtime behaviour (here, "the sidecar re-reports on its backoff retries"), the
  finding should cite the code line that establishes the premise. The re-review would then have a citation to consume instead of a
  premise to re-derive.

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P51_camera_setup_ui_review_r2.md` (this file)
- `docs/implementation/narratives/P51_camera_setup_ui.md` (new)
- `docs/implementation/plans/P51_camera_setup_ui.md` (Review log entry appended)
- `docs/implementation/master_plan.md` (P51 tracker row → `APPROVED`)
- `docs/implementation/backlog.md` (N5, N6 appended)
- No production or test file was touched. There were no probes. Logs were written to `/tmp/p51r2_*` only (outside the repo). No
  database or app state was used.
