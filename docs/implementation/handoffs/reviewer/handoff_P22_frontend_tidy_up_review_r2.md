---
plan: P22_frontend_tidy_up
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-29
actor: opus
---
# P22 re-review, round 2: APPROVED_WITH_NOTES

F1, round 1's only blocking finding, is resolved as its correction said. The perimeter is clean, and the suite, lint and
typecheck are green. No blocking item remains, so no triage is needed. Round 1's five backlog notes stand, and this
round adds none.

## ⚠ OWNER DECISIONS REQUIRED (0)
none

## Review history
Round 1 (`a55f974`) settled all 10 criterion rows, H1–H8, the assumptions and task 2's in-passing fix. It raised one B1:
**F1**, the overlay showed `00` when a Main photo was held over a to-review tile. This round looks only at the fix
checkpoint `2f5308b` and the seam it touches.

**Gate.** The tracker showed `IMPLEMENTED` with the note "fix 1: F1 resolved…". The checkpoint `2f5308b` exists, and it
contains the fix handoff.

## Perimeter check
`git show --stat 2f5308b` lists 5 paths:
- `apps/desktop/src/features/workspace/WorkspacePage.tsx`: allowed by F1.
- `apps/desktop/src/features/workspace/__tests__/p22_workspace_tidy.test.tsx`: allowed by F1 (optional hover).
- `docs/implementation/master_plan.md`: P22 tracker row only.
- `docs/implementation/plans/P22_frontend_tidy_up.md`: one appended Review log line.
- `docs/implementation/handoffs/implementer/handoff_P22_frontend_tidy_up_fix_r2.md`: the fix's own handoff.

From `2f5308b` to HEAD `3a399cf` there is only the runner's re-review prompt. `git status --porcelain` was clean at
entry. **Nothing is outside the perimeter.**

## Evidence (one run, as the prompt directs, tree `3a399cf`, clean)
- `cargo test --workspace`: 421 passed, 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo fmt --all -- --check`: exit 0.
- `npm run typecheck`: exit 0.
- `npm test`: 15 files, 124 tests passed.

These match the fix's stamp. The repo has no separate frontend lint script. (One extra read-only `grep -c` counted the
tests in the three P22 files, 4 + 4 + 2 = 10, for the narrative's try-out step.)

## Findings resolved
**F1: resolved, as corrected.** The fix adds `projectedSlot(group, photoId)`, which computes
`index === -1 ? group.length : index + 1`. It is now the only computation for both:
- the overlay's same-group `projectedNumber` (`projectedSlot(ordered[target], overPhoto.id)`), and
- `useDrop`'s `reorder` position (`projectedSlot(group, overPhoto.id)`, where `group = displayOrder(photos)[target]`).

Both read the same `displayOrder` of the same `state.photos`, so the number shown and the slot sent cannot disagree.
The original correction asked for exactly this ("share one helper").

The other branches are unchanged:
- Cross-group (including a to-review photo over Main): end of target + 1, matching `useDrop`'s classify-to-end.
- Own tile: unchanged number, no command.
- Session row: unchanged number.

Test: C3(b)'s fixture gains the unreviewed tile 6 at `[110, 85]`. A third drag holds tile 1 over it. The test asserts
the overlay reads `03` with no `.tile-cover`, and that release calls `reorderPhoto(1, 3)`. By the time of that drag,
the second drag has already committed `reorderPhoto(5, 1)`. That makes Main `[5, 1, 2]` optimistically, and the
end-of-Main slot is still 3, so the expectation is right whatever happened before. On the pre-fix code the overlay reads
`00`, so the first new assertion fails. The row's original assertions are untouched. Only `mainOrder()` now includes
tile 6, because to-review tiles render inside Main. Over-engineering test: the test reads the visible overlay and the
command sent at the boundary, so it passes.

## Triage
No blocking items remain. The table is empty.

| Item | Class | Why |
|---|---|---|
| none | — | — |

Verdict: **APPROVED_WITH_NOTES**. The notes are round 1's five backlog rows, already in `docs/implementation/backlog.md`.

## Remaining findings
None blocking, and no new notes. In passing: the test variable `afterOwnSlot` names the third drag, which in fact starts
after the second drag's committed drop. The name misleads, but the behaviour is unaffected. This is style and below the
note threshold.

## Carry-forward dispositions (open notes → destination)
| Note (backlog, round 1) | Destination |
|---|---|
| To-review photo dropped on its own slot is approved to Main | backlog. Owner question at a future frontend phase. Narrative "Not yet". |
| A command's own closing reload can replace a second drag's optimistic order | backlog. Next workspace phase. Narrative "Not yet". |
| `shopDone` no longer checks freshness | backlog. Next item-search touch. Narrative "Not yet". |
| Implementer edited C1(a)'s ISO literal | lesson for the planner (already in round 1's lessons) |
| Not verified live | owner try-out steps 5–6 in the narrative |

## Lessons for the plans
None new. Round 1's lesson ("when a row projects a value another function computes, name the shared function") is the
one this fix confirmed.

## Probe declaration
No probes this round. No file was created, edited or deleted, apart from this review's own writes. No database or app
state was touched.

## Write perimeter
- `docs/implementation/master_plan.md`: P22 tracker row → `APPROVED`.
- `docs/implementation/plans/P22_frontend_tidy_up.md`: re-review r2 Review log entry.
- `docs/implementation/narratives/P22_frontend_tidy_up.md`: new.
- This handoff.
- No backlog rows added. No commits.
