---
plan: P34_composition_polish
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-05
actor: opus (pipeline-reviewer)
---
# Review P34_composition_polish, round 1: CHANGES_REQUESTED

**Gate.** The tracker row read `IMPLEMENTING`, not `IMPLEMENTED`. The orchestrator's ruling (commit `11d13a7`, plan Review log) moved the
implementer handoff to `DONE` and resumed the run at review r1, and checkpoint `a4f9b961e4192295099c2490144267291d748d44` exists. I
proceeded on that recorded ruling. The stale row is a backlog note.

## Verified correct
- **Perimeter.** `git show --stat a4f9b96` lists 38 paths. Every file in the plan's list is present. The extras the handoff discloses are
  `Stage.tsx`, `PhotoGrid.tsx`, `harness.tsx`, `machine.test.ts` and `render_output.rs`. `export.rs` is the timing site named in plan task 7.
  `ComposeMode.tsx` and `p34_meter.test.tsx` are not named in Deviations, though both trace to P34 rows (note). `master_plan.md` and
  the plan file carry only the implementer's tracker row and Review log line.
- **Suite, lint, typecheck (reviewer run; no code change between `a4f9b96` and HEAD):** `cargo test --workspace` 531 passed / 0
  failed; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo fmt --all -- --check` exit 0; `npm run typecheck` exit 0;
  `npm test` 39 files / 300 tests passed. Not B4.
- **C1(a):** `snap` returns exact centre, edge and unchanged outcomes with guides. The test pins all three.
- **C2(b):** the machine snaps only on drag `PointerMove`, while ⌥ (`altKey`) bypasses it. The guide is cleared on commit. Guides are suppressed when
  `settle` refuses the snapped rect. Orchestrator ruling 2 accepts the test (hardening note in the backlog).
- **C3(b):** one `applyComposition` call with ids `[2, 3]` and re-fitted values. The toast reads exactly `Applied to 1 photo; 1 refused: 03`, and
  tile 2's frame updates in place.
- **C3(c):** `CompositionService::apply` returns `applied [a, c]` and `refused [(b, "off_shape")]`. Revisions are `1, 0, 1` and `change_seq`
  rose by 2. Each item is its own `set`. The Tauri command is a thin DTO loop (ruling 3 accepts the service boundary).
- **C4(a):** `useCompositionSaves` marks a photo pending on enqueue and clears it on the quiet success and on refusal. The dot appears on the
  stage bar and the overlay tile, and clears on both outcomes with the P31 toast.
- **C5(a,b):** the crossfade is `0ms` under reduced motion and `700ms` otherwise (tested). By reading (ruling 4), the glide (`.compose-image`,
  overlay rects) and `.compose-browser` use `var(--dur-med)`. `globals.css` (imported after `tokens.css`) sets `--dur-med`, `--dur-slow` and
  `--dur-crossfade` to `0ms` under `prefers-reduced-motion: reduce`. `data-gliding` is still set and cleared, the clear coming from the fallback timer.
- **C6(a):** `tauri.conf.json` reads `1440 × 900`, min `1100 × 720` (test reads the JSON).
- **C7(a):** 10 × 16 ms + 48 ms → `avg 19 ms · worst 48 ms`, and ⌥⌘P again hides the meter. **C7(b):** `master rendered` (raw, adjusted_gen,
  ms = now − issued) and `render_output done` (photo_id, slot, ms elapsed) are both logged with integer `ms` ≥ 0. **C7(c):** moved to the owner's
  try-out by ruling 1.
- **C2(a) arithmetic (scratch probe):** for an unturned photo the loupe translate at `(600, 300)` is `(−1421.81 px, −602.60 px)`, equal to the
  plan's `−(1542 − 120), −(722.6 − 120)`. The production code is right in this case; the test does not pin it (F3).
- **Assumptions:** the in-memory clipboard, refusing a frameless selected photo in the toast, and sampling the meter only during a gesture all
  agree with 10D and the plan's notes. None is B1.
- **H1–H8:** nothing external is written (H1). No RAW is touched (H2). No new pixel processing; the loupe is CSS (H3). Each apply item is an
  idempotent `set` (H4). No UI automation (H5). No commerce dependency (H6). No platform concept in core (H7). `composition_apply` stays thin
  over the service (H8).

## Findings
All four findings are blocking (B1 and B2). Their fixes touch only the loupe, its wiring and two test files.

**F1 (B1) · route: production.** The loupe ignores the photo's quarter turns, flip and straighten. `Loupe.tsx` sizes the unrotated preview
to the turned frame (`view.turned.w × view.turned.h`) and maps the pointer in turned space. The stage image, meanwhile, is drawn with
`rotate(turns·90°) scaleX(±1)` plus the straighten. Probe (scratch test, deleted): with `quarter_turns 1, flip horizontal` the stage img is
`rotate(90deg) scaleX(-1)` and the loupe img is `1707px × 2560px` with no rotation. The result is a landscape preview squeezed into a portrait
box, showing pixels that are not under the pointer. *Authority:* 10D §What the photographer sees (the editor rotates 90° and flips; "hold Z for a
100 % loupe"); plan task 1 ("shows the preview at 100 % centred on the image point under the pointer"). *Correction:* render the loupe
image at the preview's source size with the same turn, flip and straighten as the stage image, so the point at the loupe's centre is the point
under the pointer. *Files:* `apps/desktop/src/features/compose/Loupe.tsx`, `compose.css`, `__tests__/p34_loupe_snap.test.tsx`.

**F2 (B1) · route: production.** The pointer position is recorded only while the loupe is shown (`onPointer={loupe ? setPointer :
undefined}`). Pressing Z with the pointer resting on the photo therefore opens the loupe at the last stored point, which is `{0, 0}` the first time.
Probe: a pointer move to `(600, 300)`, then Z → `.compose-loupe` at `left 16px, top 16px`, showing the stage's top-left corner until the pointer moves.
*Authority:* plan task 1 ("follows the pointer … centred on the image point under the pointer"). *Correction:* keep the last pointer
position while composing, whether or not the loupe is shown, so Z opens it under the resting pointer. *Files:* `ComposeMode.tsx`, `ComposeStage.tsx`,
`__tests__/p34_loupe_snap.test.tsx`.

**F3 (B2) · route: production.** The C2(a) test checks `img.style.transform` only with `toContain('translate(')`, so any centring passes. The
row's exact outcome is `translate(−(ix − 120), −(iy − 120))` with `ix = 1542`, `iy = 722.6`. *Authority:* plan row C2(a); flow §5 B2.
*Correction:* assert the translate values (≈ `−1421.8px`, `−602.6px`, to 0.1 px). Add one turned/flipped case (F1) and one case pressing Z
without moving the pointer (F2). *Files:* `__tests__/p34_loupe_snap.test.tsx`.

**F4 (B2) · route: production.** The C3(a) test checks only the pasted rect's centre. The copied rect `(0.3, 0.18, 0.4, 0.48)` already has centre
`(0.5, 0.42)`, so pasting the raw values with no re-fit also passes. Probe: `refitClipboard(V, {w: 1707, h: 2560}).rect = (0.26002,
0.21998, 0.47995, 0.40004)`. *Authority:* plan row C3(a) ("`V'` = `V` re-fitted to the portrait frame … area-preserving"); flow §5 B2.
*Correction:* assert the re-fitted rect (x, y, w, h to 4 decimals). *Files:* `__tests__/p34_clipboard.test.tsx`.

**Fix perimeter (all findings):** `apps/desktop/src/features/compose/{Loupe.tsx, ComposeMode.tsx, ComposeStage.tsx, compose.css}`,
`apps/desktop/src/features/compose/__tests__/{p34_loupe_snap.test.tsx, p34_clipboard.test.tsx}`.

## Notes written to backlog
1. The snap guide is always drawn at the stage centre line, even for an edge snap.
2. "Apply composition to selected" patches tiles with a client-built row, so a `needs_review` photo keeps its "Re-frame" badge until reload.
3. Apply runs outside the per-photo save queue and shows no unsaved dot: the same race as the P32 Reset and P33 Split notes.
4. C2(b) hardening: assert the ⌥ save's `x` and that the snapped drag sent no save.
5. Process: the resume ruling did not set the tracker row the gate reads.
6. Perimeter: `ComposeMode.tsx` and `p34_meter.test.tsx` were not named in Deviations.

## Lessons for the plans
- Pick a C3(a)-style re-fit fixture whose re-fitted rect differs from the source rect on the property the test asserts. Here the centre is
  preserved by construction, so the fixture should differ in w/h.
- A loupe row (C2(a)) should cover a turned or flipped photo whenever the editor offers turn and flip; the unturned case alone cannot
  catch a missing transform.

## Probe declaration
No production file was edited. Two scratch test files were created in `apps/desktop/src/features/compose/__tests__/`
(`zz_reviewer_probe.test.tsx`, `zz_reviewer_probe2.test.ts`), run after the full suite had finished, and deleted. `git status` afterwards shows
only this review's documents. Probe output is in `/tmp/p34rev/` (outside the repo). No database or app state was touched.

## Write perimeter
`docs/implementation/handoffs/reviewer/handoff_P34_composition_polish_review_r1.md` (this file), `docs/implementation/plans/P34_composition_polish.md`
(Review log entry), `docs/implementation/master_plan.md` (P34 tracker row → `CHANGES_REQUESTED`), `docs/implementation/backlog.md` (6 rows).
No commits. Narrative not written: it is written at approval.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. The implementer's card (computer control for live timings) was answered by the orchestrator's ruling 1: the timings move to the
owner's try-out.
