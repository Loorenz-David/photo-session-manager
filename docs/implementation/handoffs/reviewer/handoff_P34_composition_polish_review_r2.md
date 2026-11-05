---
plan: P34_composition_polish
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-05
actor: opus (pipeline-reviewer)
---
# Re-review P34_composition_polish, round 2: APPROVED_WITH_NOTES

All four r1 blocking findings are resolved as their corrections said. No blocking item remains, so triage finds 0 MUST. There are two new
non-blocking notes, both written to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

**Review history.** In r1, 9 of the 13 rows were settled: C1(a), C2(b), C3(b), C3(c), C4(a), C5(a,b), C6(a), C7(a), C7(b). C7(c) went to the
owner's try-out by orchestrator ruling 1. The r1 findings were F1 (loupe ignores turn/flip/straighten), F2 (loupe opens at a stale pointer),
F3 (C2(a) translate not asserted) and F4 (C3(a) cannot tell re-fitted values from raw ones). Those settled rows were not re-verified.

**Gate.** The tracker row read `IMPLEMENTED` with the fix-1 note, and checkpoint `34299bacecdbb06c4c24535a92e9b63c37d23b68` exists. Pass.

## Perimeter check
`git show --stat 34299ba` lists 7 paths:
- `apps/desktop/src/features/compose/{Loupe.tsx, ComposeMode.tsx, compose.css}`
- `apps/desktop/src/features/compose/__tests__/{p34_loupe_snap.test.tsx, p34_clipboard.test.tsx}`
- `docs/implementation/master_plan.md`: the P34 row only
- `docs/implementation/plans/P34_composition_polish.md`: one Review log line

All five code paths are inside the r1 fix perimeter. `ComposeStage.tsx` was allowed but not needed, because F2 was fixed at the caller. Between
`34299ba` and HEAD (`48503a1`) only the fix handoff and this round's prompt were added. Clean. No finding.

**Suite, lint, typecheck.** I ran these on `48503a1` with a clean `git status --porcelain`; the code tree is `34299ba`. This is the single gate
run the prompt asks for.
- `cargo test --workspace`: 531 passed / 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo fmt --all -- --check`: exit 0.
- `npm run typecheck`: exit 0.
- `npm test`: 39 files / 301 tests passed.

Not B4. After that I ran `npx vitest run p34` once (9 passed) to confirm the narrative's step-1 count. It was not a re-stamp.

## Findings resolved
- **F1 resolved.** `Loupe.tsx` now draws the image the same way the stage does, and the pointer mapping is now correct.
  - *Structure.* The loupe builds a box the size of the turned frame (`view.turned`), rotated by `view.rotateDeg`. Inside it, the img has the
    source size (`state.frame`), is centred with `translate: -50% -50%` and carries `rotate(turns·90°) scaleX(±1)`. The stage does the same:
    the turn and flip sit on the img, which is centred in a turned-frame-sized wrapper, and the straighten sits on the wrapper.
  - *Pointer mapping.* The pointer goes through the inverse straighten about `view.centre`, the wrapper's rotation origin:
    `Lx = cx + cos·dx + sin·dy` and `Ly = cy − sin·dx + cos·dy`. I re-derived this for a clockwise CSS rotation in y-down screen space. At
    θ = 90°, a pointer 10 px right of centre maps to the local point 10 px *above* centre, and rotating local-up clockwise by 90° gives screen-right.
  - *Centring.* The box sits at `120 − turnedX, 120 − turnedY` and rotates about `turnedX turnedY`, so the mapped image point stays at the loupe
    centre and the area around it rotates as it does on the stage.
  - *Edge case.* A negative `quarter_turns % 4` gives the same rotation modulo 360°. Equivalent, no finding.
- **F2 resolved.** `ComposeStage` now gets `onPointer={setPointer}` unconditionally. The C2(a) test moves the pointer *before* Z and asserts
  `left 616px`. On r1 code the loupe would sit at `16px`, so the test fails on the old behaviour.
- **F3 resolved.** C2(a) asserts the view offset `−1421.8 / −602.6` with `toBeCloseTo(…, 1)`, which is a 0.05 px tolerance. These are the plan's
  `−(1542 − 120), −(722.6 − 120)`, and any other centring fails. The new turned/flipped/straightened case (`quarter_turns 1`, `flip horizontal`,
  `straighten 8`) asserts the box at `1707×2560` with `rotate(8deg)`, and the img at `2560×1707` with `rotate(90deg) scaleX(-1)`. The r1 defect,
  an unrotated `1707×2560` img with no transform, fails these assertions. See note 1 for the centring half of this case.
- **F4 resolved.** C3(a) asserts `(0.26002, 0.21998, 0.47995, 0.40004)` to 4 decimals. A raw paste of `(0.3, 0.18, 0.4, 0.48)` fails on all four.

**Seen in passing, checked and fine.**
- The loupe uses `state.values` for turn and flip, the same source as `ComposeStage`.
- The inline `transformOrigin` overrides the CSS `transform-origin: center` on `.compose-loupe-view`, as intended.

## Triage
| Item | Kind | Class | Why |
|---|---|---|---|
| Turned-case centring assertion holds by construction (note 1) | test strength | DROP | More tests: the code is right by derivation, no user-visible wrong outcome, and the r1 F1 defect is caught by the dimension and transform assertions |
| Pointer state re-renders the compose tree on every hover move (note 2) | speculative performance | DROP | No wrong outcome; the frame meter and the owner's try-out are where jank would show |

No MUST.

## Remaining findings
None blocking. Notes written to `docs/implementation/backlog.md`:
1. **Turned-case centring is unpinned.** In the turned/flipped/straightened loupe case, `left + transformOrigin.x = 120` (and the same for y)
   holds whatever `turnedX` is, because both come from it. The inverse-straighten mapping is pinned only by my derivation, not by a test.
   Hardening: assert `view.style.left/top` against the hand-computed value for the 8° case.
2. **Hover re-renders.** Fix 1 made the pointer React state that is updated on every stage `pointermove` while composing. Hover now re-renders
   `ComposeMode` with the Tools panel, `Stage` and `BrowserOverlay`. Drags already re-render it through the controller snapshot. If jank shows,
   keep the pointer in a ref and copy it into state only while the loupe is shown.

**Carry-forward.** These two plus the six r1 P34 rows stay in `backlog.md`, the backlog being the named destination for P34 notes. The
per-photo save queue rows (P32 Reset, P33 Split, P34 Apply) are the cluster a later editor phase should take as one item.

## Lessons for the plans
- A test that checks the loupe centre should compare a position against an independently computed value, not two style values derived from the
  same variable.

## Probe declaration
No mutation probes; no production or test file was edited, created or deleted. No database or app state was touched.

## Write perimeter
- `docs/implementation/handoffs/reviewer/handoff_P34_composition_polish_review_r2.md` (this file)
- `docs/implementation/master_plan.md`: P34 tracker row → `APPROVED`
- `docs/implementation/plans/P34_composition_polish.md`: Review log entry
- `docs/implementation/backlog.md`: 2 rows
- `docs/implementation/narratives/P34_composition_polish.md`: the owner narrative

No commits.
