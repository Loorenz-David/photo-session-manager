---
plan: P32_composition_tools
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-05
actor: opus (pipeline-reviewer, Claude Opus 5.5)
---
# P32_composition_tools — review r1: APPROVED_WITH_NOTES

The gate held: the tracker showed `IMPLEMENTED` and checkpoint `c0cc169` is in `git log`. Reviewed on clean tree `59cb210`. `c0cc169..59cb210` adds
only the implement handoff and the review prompt. Process authority is `docs/process/flow.md`, so there were no mutation probes and no production edits.

## Verified correct
- **Perimeter.** `git show --stat c0cc169` lists exactly the handoff's 25 paths: 7 new production files, 5 new test files, 11 changed files, the
  P32 tracker row and the Review log. Every path is in the plan's "Files expected to change". No Rust, no bindings, no intention-book change.
- **Suite (run once on the clean tree).** `cargo test --workspace`: 529 passed, 0 failed. `cargo clippy --workspace --all-targets -- -D warnings`:
  clean. `cargo fmt --all -- --check`: exit 0. `npm run typecheck`: exit 0. `npm test`: 31 files, 268 passed. No B4.
- **C1(a)** (`refit.ts`, `refit.test.ts`). The contract is implemented as written: centre and area are taken in the turned frame; the shape rule
  `w_px = sqrt(area·r)` is followed by `fit`; `fit` clamps to the frame and the minimum, then runs the straighten guard; straighten shrinks by a
  24-step bisection, is clamped to ±45 and rounded to 0.1°. The quarter-turn maps are the clockwise turn MC-27 defines, and flip keeps the same
  content. The plan literals that disagree with the contract are plan slips. I re-derived each one independently:
  - (4): `1365.6·1.25 = 1707`, so `w = 0.666796875` and `h = 0.8`.
  - (8): a centred square inside a 10° tilt of 2560×1707 needs `s(cos θ + sin θ) ≤ 1707`, so `h = 0.8632`. The test also shows a slightly larger
    rect is `rect_outside`.
  - (10): `1 − 100/652 = 0.846626`.

  Every case asserts the exact rect and passes P28's `validate`.
- **C1(b)** (`machine.ts`, `machine.test.ts`).
  - `SetValues`: idle gives View + Save; wheeling gives two Saves; dragging is ignored.
  - `Nudge{1,0}` gives `x −= 1/977.81`.
  - `Trim*`: the view stays frozen during the trim and re-fits on release. Esc restores `startRect` with no save. Blur commits.
  - The slider's `adjusting` phase: live input goes to the view only, and `change` saves once. React's value tracker drops the `onChange` that would
    follow the native `change` with the same value, so the machine ends `idle` and Esc keeps P31's revert-to-entry meaning.
  - `needsConfirm`: any `Save` clears it, and Return forces exactly one Save.
- **C2(a)–(b).** `aside.tools-panel` is rendered only in the compose branch of `WorkspacePage`. Fold state lives in `tools.sections` and hide state
  in `tools.hidden`, and both survive a remount through the hash route. The normal workspace keeps `aside.photo-panel` to the right of the stage column.
- **C3(a)–(e), C4(a)–(c).** Each test asserts the `setComposition` payload, the overlay frame box and the image transform or turn. C4(a) also checks
  the 552 × 552 box during the trim (with the image unmoved), one save on release, the 652 × 652 box after it, and `data-gliding` until `transitionend`.
  C3(e) checks the preview's pixel ratio and that it reuses the stage image's `src`.
- **C5(a)–(e), C6(a)–(b).** `useComposeKeys` is a window **capture** listener, so ⌘[ and every non-editor key are stopped before the app's maps see
  them. The workspace keyboard, ⌘Z included, is already disabled while composing. `isTyping` targets keep their keys: C6(b) asserts
  `defaultPrevented === false` and no save. `[` and `]` follow `displaySequence` and do not wrap. ⌘⌥R and Reset call `resetComposition` once and
  snap the view to the answer.
- **C7(a)–(d), C8(a)–(b).**
  - On entry, `entryValues` re-fits a `needs_review` photo to its current frame. This also happens on `SwitchPhoto`, so `[`/`]` onto such a photo
    re-fits too.
  - The banner text comes from `review_reason`. Return or any save confirms, and the `ok` answer removes the banner and the tile's corner.
  - C7(d)'s `y = 0.1666 + 50/977.81` is correct under P31 rule 2. The plan's `− 50/652` uses the wrong side and the wrong sign.
  - A `needs_review` export or publish refusal shows the backend's message on its own, and the publish popover stays the same node.
- **Assumptions (13).** None contradicts 10D or MC-27. All of them are reported to the owner in the narrative's "Choices made for you".
- **Hard constraints.**
  - H1: no new external write.
  - H2, H5, H6, H7: not touched.
  - H3: CSS transforms only, no pixel processing.
  - H4: saves stay latest-wins per photo, and the backend validates each one.
  - H8: the frontend change gives the UI no business authority, since the backend still decides the stored composition.

## Findings
None blocking (B1–B4 all clear).

## Notes written to backlog
1. **P32 → P34, confirmed by probe.** Every control or key that re-fits computes `refit(snapshot.values, …)` before the machine commits an active
   wheel, and while `wheeling` the `values` still hold the pre-wheel rect. Probe: a wheel pan, then `3` within the 120 ms settle. Save 1 carries
   the panned centre (0.4478) and save 2 carries a 4:5 rect at the old centre (0.5), so the scroll is discarded. The plan itself prescribes
   `refit(values, change)` at the caller, so the code is faithful to the text; the fix belongs in the design (`SetValues` carries the change).
2. **P32 → P34.** WebKit focuses the Shape select and the straighten range on click. While either has focus, `isTyping` turns every editor key into a
   field key, Return and Esc included, until the user clicks the stage.
3. **P32.** Reset goes around `useCompositionSaves`' queue, so a queued save can be sent after the reset and undo it.
4. **P31 → P32 → P34 (still open).** The `LAYER_OPEN` guard now mutes every P32 key while the non-modal Item details card or a header popover is open.
5. **P32 (not verified live).** The panel layout, the glide, the output preview's clipping for a turned, flipped or tilted photo, and the handle
   hit areas have only been checked in jsdom.

**Carry-forward dispositions:** notes 1, 2 and 4 → P34 (composition polish, which owns the keyboard and save-path refinements). Note 3 → P34
(save path). Note 5 → the owner's manual try-out (narrative steps 3–7).

## Probe declaration
I created one scratch test, `apps/desktop/src/features/compose/__tests__/zz_review_probe.test.tsx`, ran it and deleted it. No production file was
touched. `git status --porcelain` was empty afterwards. No database or state side effects.

## Lessons for the plans
- Four hand-computed literals were wrong: re-fit (4), (8) and (10), and C7(d). When a row states a contract formula and a derived number, the row
  should name the formula as the authority and give the number as "≈".
- C1(b) "TrimStart, Blur → Save" cannot happen without a move, because nothing changed. The implementer tested it with a move in between.
- `SetValues { values }` computed outside the machine cannot honour "commit the wheel first; apply" (note 1). P33/P34 rows that add controls should
  pass the *change* to the machine, not the result.
- A plan that adds keys behind an existing guard (`LAYER_OPEN`, `isTyping`) should say what those guards now mute (notes 2 and 4).

## Write perimeter (this session)
- `docs/implementation/master_plan.md`: the P32 tracker row only (→ `APPROVED`).
- `docs/implementation/plans/P32_composition_tools.md`: the Review log entry, appended.
- `docs/implementation/backlog.md`: five rows appended.
- `docs/implementation/narratives/P32_composition_tools.md`: new.
- This handoff.

Nothing committed: the runner commits these files.

⚠ OWNER DECISIONS REQUIRED (0): none.
