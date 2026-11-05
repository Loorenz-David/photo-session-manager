---
plan: P35_composed_display_and_chrome
role: implement
round: 1
state: DONE
date: 2026-11-05
actor: codex
---

# P35 implementer handoff

## What was built

The stage and photo tiles now show the saved framing: crop, turn, flip, straighten and the middle line for a split. Grid tiles carry a small crop mark when framing leaves part of the photo out. A Compose button explains when its full-frame preview is still pending; R and double-click give the same message. Split works on unreviewed photos. The header Details button and the item card open the product form. Sessions moved to an icon in the photo panel and folded strip. Applying a copied composition clears Re-frame immediately.

Implementation uses shared `ComposedPicture` CSS over the existing preview or thumbnail; no backend renderer or data model changed. The one Rust test verifies the existing service accepts an unreviewed split.

## Tests → criterion rows

| Row | Boundary test and exact asserted outcome |
|---|---|
| C1(a) | `p35_chrome.test.tsx`: Compose button placement, hint, editor opening |
| C1(b) | `p35_chrome.test.tsx`: waiting hint, aria-disabled, toast, no editor, menu reason |
| C1(c) | `p35_chrome.test.tsx`: needs-review photo opens Compose |
| C2(a) | `p35_composed_display.test.tsx`: square stage box, frame offsets, adjusted transform |
| C2(b) | `p35_composed_display.test.tsx`: split ratio, frame offsets, middle hairline |
| C2(c) | `p35_composed_display.test.tsx`: turned and straightened frame, flipped image |
| C2(d) | `p35_composed_display.test.tsx`: camera original orientation within the crop; unframed fallback |
| C3(a) | `p35_composed_display.test.tsx`: grid thumbnails, crop marks, split and unframed states |
| C3(b) | `p35_composed_display.test.tsx`: folded tile shows composition without a crop badge |
| C3(c) | `composed.test.ts`: pixel rounding and straighten crop-mark cases |
| C4(a) | `p35_chrome.test.tsx`: unreviewed context-menu Split saves the fitted rect and keeps one Main place |
| C4(b) | `p35_chrome.test.tsx`: S and the Split segment both save on an unreviewed photo |
| C4(c) | `p35_split_any_photo.rs`: service stores unreviewed Split, revision 1, state ok and the exact rect |
| C5(a) | `p35_chrome.test.tsx`: header Details and D open the Chair form; stage duplicate absent |
| C5(b) | `p35_chrome.test.tsx`: item card Edit details opens the same form |
| C6(a) | `p35_chrome.test.tsx`: panel Sessions icon hints and toggles its panel |
| C6(b) | `p35_chrome.test.tsx`: folded Sessions icon toggles; Unassigned opens with it pressed |
| C7(a) | `p35_chrome.test.tsx`: Apply clears Re-frame without a session reload |

`composed.test.ts` also checks layout math as the plan's named core mechanism. Earlier P31–P34 tests were updated for changed selectors and the now-enabled unreviewed Split menu.

## Assumptions

- The crop mark is a `Crop` glyph at the tile's upper right, below the unreviewed dot and clear of the number and lower badges.
- Waiting Compose uses Sonner's default toast with the required stable id.
- `App` closes Details when the session id changes.
- OutputPreview and SplitPreviews keep their existing renderer; the shared renderer serves the stage and tiles.

## Deviations

- `BrowserOverlay.tsx` was edited to pass Sessions state and its toggle into the P31 browser. Task 10 requires this while composing, though the estimated file list did not name the file.
- No visual browser run: Browser plugin is absent, Playwright is not installed locally, and the browser-only Vite surface needs Tauri commands for a real workspace. Component DOM tests cover the requested UI states; visual sizing awaits the owner's manual try-out.

## Files changed

- Frontend source: `apps/desktop/src/App.tsx`, `components/ui/tooltip.tsx`, `services/compositions.ts`, `features/shell/Header.tsx`, `features/compose/{BrowserOverlay,ComposeMode,CompositionSection,ComposedPicture,composed}.tsx/ts`, `features/workspace/{FoldedStrip,ItemDetailsCard,PhotoMenu,PhotoPanel,PhotoTile,Stage,StageImage,WorkspacePage}.tsx`, `features/workspace/workspace.css`.
- Tests: `apps/desktop/src/features/compose/__tests__/{composed,p31_browser_and_tiles,p31_compose_mode,p31_gestures,p32_keys,p34_clipboard}.test.ts/tsx`, `features/workspace/__tests__/{p33_split_grid,p35_chrome,p35_composed_display}.test.tsx`, `crates/testkit/tests/p35_split_any_photo.rs`.
- Process: `docs/implementation/master_plan.md`, `docs/implementation/plans/P35_composed_display_and_chrome.md`, this handoff.

## Checkpoint SHA

`HEAD` after the single checkpoint commit. A commit cannot contain its own SHA; the final runner message provides the resolved hash.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | PASS, including the new Rust boundary test |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cd apps/desktop && npm run typecheck` | PASS |
| `cd apps/desktop && npm test` | PASS, 42 files / 320 tests |

No pre-existing failures. An initial clippy run flagged the new fixture's literal approximation of `FRAC_1_SQRT_2`; the test now uses the constant and clippy passes. An earlier full UI run exposed a timing-sensitive pre-existing P33 stage-bar assertion after a drag test; the assertion now waits for the observable stage state and the final full run passes.

## Manual try-out

1. Open a session with framed photos. Check that a Square crop is square on the stage, a split shows two halves with a line, and Space compares the camera original in the same frame.
2. Check the grid crop marks; press F to see the framed images in the strip without marks.
3. Hover Compose for `Compose (R)`. On a photo awaiting its full-frame preview, click Compose, double-click the stage and press R to see the waiting message.
4. Right-click an unreviewed photo and choose Split. It stays one Main place while unreviewed.
5. Open Details from the header, D, and the item card. Toggle Sessions from the photo panel header and folded strip.
6. Apply a copied composition to a photo with Re-frame and confirm the badge disappears at once.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
