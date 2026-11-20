---
plan: C3_conversation_and_answering
role: fix
round: 2
state: DONE
date: 2026-11-20
actor: claude sonnet (pipeline-implementer)
---
# Fix 1 for C3_conversation_and_answering (round 2)

**Gate.** The tracker showed `CHANGES_REQUESTED`; the intention is RATIFIED. Passed.

## Finding → what changed → files
- **B2-1** (route production, test-only): `c3_c16b_shift_enter_is_a_new_line` could not fail when Shift+Enter sends.
  - **Change.** After `enter(true)` the test now awaits `Promise.resolve()`. It then asserts that `conversationReply` was not called **and** that the box still holds `a`. Only then does it type `a\nb` and assert that value.
  - **Proof it can fail.** I changed the input in the test file, `enter(true)` to `enter(false)`, and ran the test. It went red with "expected vi.fn() to not be called at all, but actually been called 1 times". I then restored the file.
  - **Files.** `apps/desktop/src/features/conversations/__tests__/c3_composer.test.tsx`.

## Divergences
None.

## Files changed
This cycle only:
- `apps/desktop/src/features/conversations/__tests__/c3_composer.test.tsx` (3 lines added, 1 changed)
- `docs/implementation/plans/C3_conversation_and_answering.md` (Tracker line → `IMPLEMENTED`, one Review log entry)
- this handoff

No production file was touched. The changed-input probe above touched only the finding's own test file, and that change was reverted. No tool state (archgraph, database) was changed.

## Runs
Run once each, one command at a time:

| Command | Result |
|---|---|
| `cd apps/desktop && npx vitest run src/features/conversations` | 5 files, 67 / 0 |
| `npm run typecheck` | exit 0 |

No Rust file changed, so clippy, fmt and cargo tests were not run; the review's green results for those on the same code tree stand. The full suite was not run because no shared code changed.

## Checkpoint SHA
`a717be1` (`CHECKPOINT (not approved): …`, only the test file and the plan file). This handoff is written after the commit, so the runner commits it.

## Stamp
Tree: `a717be1`. The only delta against the reviewed tree (`9076272` code) is the test file above. vitest `src/features/conversations` 67/0, typecheck exit 0.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
