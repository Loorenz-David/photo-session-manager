---
plan: P36_editor_controls_and_overlays
role: implement
round: 1
state: DONE
date: 2026-11-05
actor: codex
---

# P36 implement r1

## What was built

While editing a photo, the right browser is now a persistent folded strip. F unfolds the full browser and folds it again; leaving Compose restores the workspace's previous panel state. The editor has five alignment overlays, a straightening grid, typed and stepped degrees, buttons and hints for its shortcuts, and snap guides at the line reached. Overlay, grid and snapping preferences persist locally. The old edge-reveal browser was removed. No backend path changed.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Tests → criterion rows

| Row | Boundary test |
|---|---|
| C1(a) | `p36_strip_overlays.test.tsx` — persistent strip and tile switch |
| C1(b) | `p36_strip_overlays.test.tsx` — F/panel fold, photo switch and restoration |
| C2(a) | `overlays.test.ts` — numeric line sets on full and split boxes |
| C2(b) | `p36_strip_overlays.test.tsx` — O cycle, selector, storage and invalid value |
| C3(a) | `p36_straighten.test.tsx` — grid timing, pattern and persisted switch |
| C4(a) | `p36_straighten.test.tsx` — typed degrees save and centred refit |
| C4(b) | `p36_straighten.test.tsx` — three exact arrow steps and focus |
| C4(c) | `p36_straighten.test.tsx` — clamp, bad text, decimal comma and Escape |
| C5(a) | `p36_controls.test.tsx` — all 30 roles/names/places and exact tooltip text |
| C5(b) | `p36_controls.test.tsx` — inactive Paste remains hintable and inert, Copy enables it |
| C6(a) | `p36_controls.test.tsx` — Revert restores the entry rect |
| C6(b) | `p36_controls.test.tsx` — stage arrows step without wrapping |
| C6(c) | `p36_controls.test.tsx` — Loupe toggle and location, no save |
| C6(d) | `p36_controls.test.tsx` — remembered snap toggle changes drag result and guide |
| C6(e) | `p36_controls.test.tsx` — Copy/Paste across different frame sizes, exact fitted values |
| C6(f) | `p36_controls.test.tsx` — Tools panel visibility, pressed and stored state |
| C6(g) | `p36_controls.test.tsx` — Frame meter visibility and pressed state |
| C6(h) | `p36_controls.test.tsx` — nudge normal/Shift steps |
| C7(a) | `p36_straighten.test.tsx` — Shape change gives X back to editor |
| C7(b) | `p36_straighten.test.tsx` — pointer slider gives X back to editor |
| C8(a) | `snapping.test.ts` — centre/start/end/null with exact rects |
| C8(b) | `p36_controls.test.tsx` — edge guide at x=174 and saved x=0 |

Eight prior test files were adjusted for the retired edge browser, degrees field and guide type. The focused compose suite passed 179 tests in 21 files. No hardware or external service is used by the suite.

## Assumptions

- C4(a)'s `0.816 ± 0.005` height is a repeated arithmetic slip. The existing P32 refit contract and reviewed P32 test give `1/(cos 10° + sin 10°) ≈ 0.8632`; the P36 test uses that result.
- C6(d)'s second `+3.7 px` drag cannot snap to centre if it starts from the first drag's saved x=0.162818. The test uses Revert to restore the entry rect before the second drag.
- C1(a)'s absence of `data-open` refers to the retired browser; the separate Tools section legitimately uses that attribute for its own fold state.
- A typed degree value commits on Enter or blur, and a slider pointer released anywhere ends the grid and blurs the slider. These are the smallest readings of the plan's interaction rules.

## Deviations

The three fixture/readings above differ from literal acceptance-row wording and are disclosed in the Review log. `C4(a)`'s expected height should be corrected upstream. No file outside the plan's source/test perimeter and closing artifacts was changed.

## Files changed

- New: `apps/desktop/src/features/compose/{EditorToolbar.tsx,StraightenControl.tsx,overlays.ts,useComposePrefs.ts}` and `__tests__/{overlays.test.ts,p36_controls.test.tsx,p36_straighten.test.tsx,p36_strip_overlays.test.tsx}`.
- Changed: `apps/desktop/src/features/compose/{ComposeFrame.tsx,ComposeMode.tsx,ComposeStage.tsx,CompositionSection.tsx,compose.css,machine.ts,snapping.ts,useComposeGesture.ts,useComposeKeys.ts}`, `apps/desktop/src/features/workspace/{Stage.tsx,WorkspacePage.tsx}`, and the eight prior test files named in the plan.
- Deleted: `apps/desktop/src/features/compose/BrowserOverlay.tsx`.
- Closing artifacts: this handoff, `docs/implementation/master_plan.md` tracker row, `docs/implementation/plans/P36_editor_controls_and_overlays.md` Review log.

## Checkpoint SHA

This handoff is in the checkpoint commit itself, so its SHA cannot be embedded without changing that SHA. Resolve with `git rev-parse HEAD` after the checkpoint; the final session line reports the literal SHA.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | Exit 0; all Rust tests and doctests passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0 |
| `cargo fmt --all -- --check` | Exit 0 |
| `cd apps/desktop && npm run typecheck` | Exit 0 |
| `cd apps/desktop && npm test` | Exit 0; 46 files, 339 tests |

No pre-existing failures appeared. The command stamp covers the source tree prior to this handoff and tracker/Review log edits; those edits are documentation only.

## Manual try-out

1. Open a photo and press R. The thin right strip stays visible. Click its next photo, press F to unfold the full browser, press F to fold it, then Return. The ordinary browser returns to its pre-Compose state.
2. Press R again. Thirds guides appear. Press O repeatedly for Grid, Golden ratio, Diagonals, Off and Thirds. Pick Golden ratio from Overlay, quit and reopen: it remains selected. Split a photo and inspect the guides in each half.
3. Focus Straighten degrees to see the fine grid. Type `2.5` and Enter, then press ↑, ↓ and Shift+↑. Hold and drag the slider to see the grid. Turn Straightening grid off and refocus the field.
4. Hover and click the editor controls in the Tools panel and above the photo. Try Loupe, Snapping, Copy/Paste framing, Tools panel, Frame meter, Revert, the nudge pad and the stage arrows.
5. Choose 4:5 in Shape and press X immediately. Drag the image until an edge snaps to the frame: the guide sits on that edge.
