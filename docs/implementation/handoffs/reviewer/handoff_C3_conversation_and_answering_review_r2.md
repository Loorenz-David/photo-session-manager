---
plan: C3_conversation_and_answering
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-20
actor: opus (pipeline-reviewer)
---
# Re-review: C3_conversation_and_answering, round 2 (after fix 1)

**Summary.** Fix 1 resolved the only blocking finding (B2-1) exactly as its correction said. It changed only the
allowed test file and the plan's own Tracker line and Review log. No blocking item remains, so the triage is empty. The four r1 notes are still open
in the phase backlog, so the verdict is `APPROVED_WITH_NOTES`. Tracker → `APPROVED`. Narrative written. Nothing needs
the owner.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

**Review history.** In r1 the production code was verified correct on all 75 rows, against MC-C2, 09B §7–§8 and H1–H8.
74 rows were settled. B2-1 (the C3·C16(b) test could not fail) was the one blocker, and the r1 handoff wrote 4 notes to
the backlog. This round re-verifies only the fix's seam.

**Gate.** The Tracker line read `IMPLEMENTED` with a fix-1 note. Fix checkpoint `a717be1` is in `git log`. The intention is RATIFIED.

## Perimeter check
- `git show --stat a717be1` lists 2 files:
  - `apps/desktop/src/features/conversations/__tests__/c3_composer.test.tsx` (+3/−1). This is the only file B2-1 allowed.
  - `docs/implementation/plans/C3_conversation_and_answering.md`: the Tracker line and the Review log, which the fix session owns.
- The runner commits `025f5a6` (fix handoff) and `5589416` (this prompt) contain only those documents.
- `git diff --stat 9076272 HEAD -- . ':!docs'` lists only the test file. So the Rust and production TypeScript are the tree r1 reviewed, byte for byte.
- Nothing outside the perimeter changed. `git status --porcelain` was clean at the start.

## Findings resolved
**B2-1 → resolved, as corrected (no divergence declared, none found).**
- **The change.** After `enter(true)`, the test now does three things in order:
  1. `await Promise.resolve()`;
  2. asserts that `conversationReply` was not called **and** that `box().value === 'a'`;
  3. types `a\nb` and asserts that value.
- **Why one microtask is enough.** `Composer.send` (`Composer.tsx:72`) queues the call with `Promise.resolve().then(…)` on a promise that is already resolved. That callback is queued before the test's own `await` continuation, so it runs first. A Shift+Enter that sends would therefore be recorded before the `not.toHaveBeenCalled` assertion.
- **A second, independent cause of red.** `send` calls `setText('')` synchronously (`Composer.tsx:68`), inside the keyDown that `fireEvent` wraps in act. A sending Shift+Enter empties the box, and `toBe('a')` fails without any microtask. The old test hid this by re-typing first.
- **Agreement.** The fix session changed the input to `enter(false)` and the test went red. That matches this reading. I spent no extra probe, because the reading settles it, and flow.md §4 forbids editing production.
- **Over-engineering test:** passes. The test changes only if what Shift+Enter does changes.

## Evidence identity
Tree: HEAD `5589416`, clean. Each command ran once, one cargo process at a time:

| Command | Result |
|---|---|
| `npx vitest run src/features/conversations/__tests__/c3_composer.test.tsx` | 1 file, 19/0 |
| `npm run typecheck` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy -p psm-core -p psm-infra -p psm-desktop -p psm-testkit --all-targets -- -D warnings` | clean (0.6 s, cached; the Rust tree has not changed since r1) |

No cargo tests were run. The fix touched no crate, so the r1 runs and the implementer's full-suite stamp stand for the
Rust tree, which is identical. The fix's vitest run (`src/features/conversations` 67/0) is on the same tree, and the
composer file was re-run here as the finding's named file.

## Triage
No blocking items remain, so there is nothing to classify.

| Item | Class | Reason |
|---|---|---|
| — | — | B2-1 resolved. Nothing else was seen wrong on the changed seam. |

## Remaining findings
None blocking. Nothing new was seen in passing. The four r1 notes carry forward, and none of them is a B1–B4:

| r1 note | Destination |
|---|---|
| The composer clears the text at the press, so a refused reply loses the typed answer | C4 or a later conversations UI touch (`backlog_conversations/C3_…`) |
| MC-C2 gap: a `sending` row stays "Sending…" when a check turns the state `refused`/`store_gone` | Orchestrator: fold into master plan MC-C2 before C4 is dispatched |
| Contract folds: implementer assumptions 1, 2, 5, 6 → MC-C2 | Orchestrator: master plan MC-C2, before C4 |
| Stale guide line in `docs/guide/02_architecture.md` (`discard` is not `conversations_off`) | The next phase that touches the guide's conversations rows (C4) |

No new backlog entries were written. `docs/implementation/backlog.md` is unchanged, because the triage dropped no items.

## Write perimeter of this session
- `docs/implementation/plans/C3_conversation_and_answering.md`: the Tracker line (→ `APPROVED`) and one Review log entry.
- `docs/implementation/narratives/C3_conversation_and_answering.md` (new).
- This handoff.
- No probes: no scratch files, and no production or test file was touched. No database, archgraph or tool state was changed. Nothing was committed.

## Lessons for the plans
- None new. The r1 lesson stands: "no call" UI rows for components that call their service in a microtask need a microtask flush before the assertion. C4's "calls neither" rows under MC-C3 are the next place it applies.
