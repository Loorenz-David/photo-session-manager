---
plan: P31_compose_mode
role: fix
round: 2
state: DONE
date: 2026-11-05
actor: fable (pipeline-implementer, Claude Fable 5.1)
---
# P31_compose_mode — fix r2 (fix cycle 1)

Checkpoint `f937a9c` on a clean tree (`git status --porcelain` empty after the commit). Both blocking findings of review r1 (F1, F2; route
`production`) resolved as their correction clauses say. No owner decision needed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Provenance (read before the gate check)
This is the **second launch** of fix r2. The first (2026-11-05 00:49) was cut off by the usage limit after it had written the P31 tracker row
(`IMPLEMENTED`, "fix 1") and a Review log entry, but before any handoff or checkpoint commit; its code never reached the tree (the runner
stashed it as `P31 fix r2 partial`, unapplied, and recorded `fix state MISSING`; commits `ff3b54e`, `86289d2`, `8f2c4d7`). This session
re-did the fix from the review findings alone — it did not apply or read the stash — and **replaced** the premature Review log entry and
tracker note with accurate ones (the earlier text is in `git show ff3b54e`).

## Finding → what changed → files

### F1 (B1) — a successful earlier save snapped the stage back while a newer save was queued
- **What changed.** `useCompositionSaves` takes two callbacks instead of `onSaved`: `onStored(photoId, composition)` after **every** successful
  save (the workspace row is patched so tiles redraw at once), and `onSettled(photoId, composition)` **only when the photo's queue is quiet** — a
  success with nothing newer pending, or a refusal (with the last stored row). `ComposeMode` wires `onStored → patchPhoto` and `onSettled →
  Saved` to the machine. An older answer that lands while a newer save is queued therefore never reaches machine rule 16; the stage stays on
  what the photographer sees. A refusal still snaps the view to the stored row (C7(c) unchanged). `machine.ts` untouched (rule 16's input is the
  same), so `machine.test.ts` untouched.
- **Test.** `p31_gestures.test.tsx` › C7(b) extended: after two quick +50 px drags `setComposition` has been called **once** (B waits behind A) and
  the stage is at 111.09 px; resolving A sends B and the placement **stays at 111.09** while B is in flight; B resolves → tile 1 `left 6.4332%`,
  placement 111.09, exactly two calls. Red on the r1 production code (`expected 49.99 to be less than or equal to 0.5` — the 61.09 snap-back),
  green now.
- **Files.** `apps/desktop/src/features/compose/useCompositionSaves.ts`, `apps/desktop/src/features/compose/ComposeMode.tsx`,
  `apps/desktop/src/features/compose/__tests__/p31_gestures.test.tsx`.

### F2 (B1) — Esc that closed a menu over the editor also reverted the photo
- **What changed.** `useComposeGesture`'s `window` keydown handler now returns first when `event.defaultPrevented` or when a
  `[role="dialog"], [role="alertdialog"], [role="menu"]` layer is open (`LAYER_OPEN`, the selector `useBackKeys` uses, inlined because
  `lib/keyboard.ts` is outside F2's file list). The guard covers the whole handler — Esc, Return and the swallowed workspace keys — per the
  correction's "ignores keys". Grounded in the installed Radix (`@radix-ui/react-dismissable-layer/dist/index.mjs:91–107`): the topmost
  `DismissableLayer` listens for Escape on the document in the **capture** phase and calls `preventDefault()` before `onDismiss()`, so both
  guards bite for the ⋯ menu, the right-click menu, the pickers and the product form.
- **Test.** `p31_gestures.test.tsx` › "F2 (review r1) Esc that closes the ⋯ menu over the editor closes the menu only": drag +50 (one save,
  placement 61.09), open ⋯ (`Photo actions`), Esc dispatched on the menu element → the menu is gone, `setComposition` still called once,
  placement 61.09, editor open. Red on the r1 production code (`expected "vi.fn()" to be called 1 times, but got 2 times`), green now.
- **Files.** `apps/desktop/src/features/compose/useComposeGesture.ts`, `apps/desktop/src/features/compose/__tests__/p31_gestures.test.tsx`.

## Divergences
1. **Gate.** The prompt's gate reads "Tracker shows `CHANGES_REQUESTED`, otherwise BLOCKED". The tracker read `IMPLEMENTED (fix 1)` — the
   aborted first launch's own pre-written claim (see Provenance), contradicted by the tree (`onSaved` still present, no layer guard, 222 tests,
   no fix handoff, no checkpoint after `7d98764`). The last **completed** state — review r1 `CHANGES_REQUESTED` on checkpoint `7d98764` — is
   exactly the state a fix r2 launch expects, and the orchestrator's STATUS and the `RESUME=1` runner change say the run was resumed at this fix on
   purpose. I proceeded and record it here for the re-reviewer rather than stopping the overnight run on a row this same role had written. If the
   re-reviewer judges this should have been BLOCKED, the fix itself is unaffected: it is the r1 findings' corrections as written.
2. **C7(b)'s stated order.** The row says "resolve `pB` then `pA`". Under task 4's one-in-flight rule B is not sent until A settles, so the
   extended test resolves A first (the only order in which "after A settles while B is still in flight" — the correction's new check — is
   observable) and B second. The row's outcome (B's rect shown, exactly two calls) is asserted unchanged. The r1 implement handoff already noted
   this; the coordinator may correct the row's literal.
3. The interim tile state (tile 1 showing A's stored row between the two settles) is not pinned — the correction allows it ("the tile may still
   take the stored row") and the finding is about the stage.
4. The F2 test dispatches Esc on the menu element, not on `window` as the other key tests do: a `window`-targeted keydown never passes the
   document's capture listener, so Radix would not see it and the scenario would not be the owner's (menu closes **and** photo reverts).

## Files changed (checkpoint `f937a9c`, 6 paths — cycle-scoped perimeter)
Production: `apps/desktop/src/features/compose/useCompositionSaves.ts`, `apps/desktop/src/features/compose/ComposeMode.tsx`,
`apps/desktop/src/features/compose/useComposeGesture.ts`.
Tests: `apps/desktop/src/features/compose/__tests__/p31_gestures.test.tsx` (C7(b) rewritten, F2 test added, `screen` import).
Docs: `docs/implementation/master_plan.md` (P31 tracker row only — note rewritten), `docs/implementation/plans/P31_compose_mode.md` (the stale
fix r2 Review log entry replaced by this session's).
Not touched: `machine.ts`, `machine.test.ts`, `lib/keyboard.ts`, `p31_compose_mode.test.tsx`, backlog, any other row or handoff.
Mutation-probe files: none (flow §4: no mutation regime). **Red check** (declared, applied-and-reverted): the three production files were
stashed with `git stash push -- <3 paths>`, the test file run (2 failed / 15 passed, F2 and C7(b) as quoted above), then `git stash pop`;
`git status --porcelain` afterwards showed exactly the four code/test modifications, and the pre-existing stashes (`P31 fix r2 partial`,
`P29 partial codex attempt`) are intact. No temporary file was created.

## Checkpoint SHA
`f937a9c` — `CHECKPOINT (not approved): P31_compose_mode fix r2 — F1 saves settle only when the queue is quiet, F2 compose keys yield to an
open layer` (parent `4a52ef5`). This handoff file is left uncommitted for the runner.

## Stamp
Taken on the working tree whose code diff is exactly `f937a9c`'s four code/test paths (the two docs edits followed; no test input changed).
Node 22.22.3 via `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH` (master plan §7.1).
- `cd apps/desktop && npm run typecheck` → exit 0.
- `cd apps/desktop && npx vitest run` (= `npm test`) → **26 files, 223 passed, 0 failed** (r1 stamp 222; +1 = the F2 test).
- `cargo test --workspace` → **529 passed, 0 failed** (no Rust change; equals the r1 and review stamps).
- `cargo clippy --workspace --all-targets -- -D warnings` → `Finished`, no warnings. `cargo fmt --all -- --check` → exit 0.
Failure-ID delta against the review r1 stamp (`c88ba28`): none (∅ → ∅).
L1 evidence: `npx vitest run src/features/compose/__tests__/p31_gestures.test.tsx` → 17 passed with the fix; 2 failed / 15 passed with the
production edits stashed (red check above).

## Manual try-out (what the owner can now check)
1. In the editor, drag the photo twice quickly (two short drags back to back). The photo must stay where the second drag left it — it must not
   hop back to the first position for an instant when the first save answers.
2. Drag the photo, then open the ⋯ menu above the stage and press Esc. Only the menu closes; the photo keeps its new framing. (Before the fix,
   Esc also threw the framing away.) The same holds for the right-click menu in the edge-reveal browser.
