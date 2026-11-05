---
phase: P35_composed_display_and_chrome
title: Composed display and workspace chrome — the stage and the tiles show the photo as composed (tiles with a crop mark), a Compose button that says why when it cannot open, Split for any photo, the header Details button opens the form, Sessions moves to the photo panel header
difficulty: normal
projection: no
depends_on: [P34_composition_polish]
---
# P35 — Composed display and workspace chrome: the stage and the tiles show the photo as composed (tiles with a crop mark), a Compose button that says why when it cannot open, Split for any photo, the header Details button opens the form, Sessions moves to the photo panel header

## Goal
The workspace half of the owner's corrections after the first composition try-out (10D as amended `b75ad61`/`163857a`, R38, changelog 28).
**Outside the editor the stage shows the photo as composed** (turned, flipped, straightened, cut to the frame; a split shows both halves with
the middle hairline), not the whole frame with a rectangle drawn on it; **tiles show the composed picture too**, with a small **crop mark** on
grid tiles when the composition leaves part of the frame out. Both are CSS over the existing preview / 640 px thumbnail; **no Rust render**.
A visible **Compose button** on the stage bar (hint "Compose (R)"); Compose is unavailable **only** when `canCompose` is false (no full-frame
preview and frame yet), and then the button, the menu entry, R and double-click say why ("Waiting for Capture One's full-frame preview")
instead of doing nothing. **Split is allowed for any photo**, unreviewed included (places and outputs still counted only for Main and
Condition). **Chrome:** the header's Details button opens the product details form (as D does); the stage's duplicate Details button goes; the
Sessions button leaves the stage and becomes an icon with a hint in the photo panel's header (and in the folded strip). A shared `Hint`
tooltip component (Radix, shadcn style) is introduced here and used by every control this phase touches. Folded in (cheap, same files, from
the P31–P34 narratives' "Not yet"): the **Re-frame badge clears after "Apply composition to selected"**, and the item card's **"Edit details →"**
(the same "arrives in a later phase" placeholder as the header button) opens the form.
**Not in this phase:** the editor's right browser (still the P31 edge reveal; P36 replaces it with the strip), overlays, the straightening grid,
the typed straighten, the editor's controls and hints (P36). No backend change (one Rust test only).

## Read first
Book: `10D_composition_and_framing.md` §What the photographer sees (bullets "Compose …", "Split …", "The result is shown", "Workspace chrome"),
§MC-27 Values (rect fractions of the turned, flipped, unstraightened frame; straighten about the image centre) and Pixel rule, §MC-28 Thumbnail
("tiles … show the composition's result from it with CSS … with a crop mark"), §MC-29 Outputs (places per output; only Main and Condition are
outputs) · `15_decisions.md` R38 · `16_changelog.md` entry 28 (the orchestrator's readings: the crop mark "whenever the rect is smaller than the
frame or the photo is straightened") · `06` §Original / Adjusted (Space toggles; 700 ms crossfade) · evidence `design_handoff.md` §Keyboard (D
opens the details form) · process `flow.md` §4, §6, §9 · master plan §6.10 (amended for P35/P36 by this planning round), §7.2 (commands) ·
plans `P31_compose_mode.md` (fixture: stage `1000 × 700`, frame `2560 × 1707`, default square), `P33_split_ui.md` (C1(a)(1) the split re-fit
`(0.0285, 0.1465, 0.943, 0.7071)`; the grid fixture of `p33_split_grid.test.tsx`), `P34_composition_polish.md` (Apply, clipboard) and the
narratives `narratives/P31…P34` ("Not yet").
Code at `163857a` (line numbers as of that commit):
- `apps/desktop/src/services/compositions.ts:44–46` `canCompose` (the one availability rule; keep it, add `COMPOSE_WAITING` beside it).
- `apps/desktop/src/features/compose/OutputPreview.tsx` and `SplitPreviews.tsx` — the CSS pattern this phase generalises: a box at the rect's
  pixel ratio, inside it the turned frame scaled by `1/w`, `1/h`, offset `−x/w`, `−y/h` and rotated by `straighten` about its centre, the image
  inside with `rotate(turns·90deg) scaleX(±1)`; `compose.css:66–70` (`.output-preview-frame`, `[data-turned] img { width: 100cqh; height: 100cqw }`).
- `apps/desktop/src/features/workspace/StageImage.tsx:44–104` (layers, crossfade, `orientationTransform` for the camera original, `.stage-frame`
  which `Stage.tsx:55` uses for the double-click), `Stage.tsx:61–78` (`.stage-bar`, `.stage-tools`: Done while composing, ⋯, Info).
- `apps/desktop/src/features/workspace/PhotoTile.tsx:16–58` (`Thumb`, `TileComposition` with `.tile-composition`, `.tile-frame`, `.tile-split`),
  `:74–92` (`PhotoTileFace`: badges are grid-only except the unreviewed dot and the unsaved dot); `workspace.css:80–88`.
- `apps/desktop/src/features/workspace/PhotoMenu.tsx:16–33` (`entries`: Compose… disabled by `canCompose`; Split disabled for `unreviewed`).
- `apps/desktop/src/features/workspace/WorkspacePage.tsx`: `formOpen :136`, `enterCompose :204–211`, grid `setKind` unreviewed skip `:227`, the
  Apply patch `:256–266` (keeps `state: 'needs_review'` — the stale Re-frame badge), `useKeyboard d :309` and `r :311`, `stageExtras :371–385`
  (the stage Details and Sessions buttons), `FormDialog :456`.
- `apps/desktop/src/features/compose/CompositionSection.tsx:11–12, 43, 54–56, 75–84` and `ComposeMode.tsx:13, 147` (`canSplit`).
- `apps/desktop/src/features/shell/Header.tsx:174–177` (the disabled Details placeholder), `App.tsx:96, 107–123` (the `assignOpen` lift this phase
  mirrors for details), `features/workspace/PhotoPanel.tsx:25–32` (`.panel-tools`: Import, Fold), `FoldedStrip.tsx:13–15` (Unfold),
  `ItemDetailsCard.tsx:37` (the disabled "Edit details →").
- Backend (verify only): `crates/core/src/services/composition.rs:35–45` (`set`: validation only, no classification rule) and
  `crates/infra/src/sqlite/store.rs:975–1000` (`save_composition`: clears `needs_review`, `review_reason NULL`).
- Tests that **change their expectation** in this phase (the behaviour changed, R38): `features/compose/__tests__/p31_browser_and_tiles.test.tsx`
  C8(c) (`.tile-composition`/`.tile-frame` → `.tile-composed`/`.composed-frame`), `p31_gestures.test.tsx:321–322, 357`, `p32_keys.test.tsx:119–121`,
  `p34_clipboard.test.tsx:55` (a tile's rect was read from `.tile-frame` `left = x`; it is now `.composed-frame` `left = −100·x/w %`, 4 decimals),
  `features/workspace/__tests__/p33_split_grid.test.tsx:69–78` (`.tile-split` → `.composed-split` at `left 50%`, `top 0%`, `height 100%` of the
  composed box; `.tile-frame` → `.composed-frame`) and `:161–163` (inverted: an unreviewed photo's `Split` entry is enabled). The P08/P19/P20/P22/
  P26 fixtures have `rect: null` and keep passing unchanged (the uncomposed stage and tile are today's).

## Files expected to change
Frontend (new): `apps/desktop/src/components/ui/tooltip.tsx` (Radix Tooltip in the shadcn style + `Hint`), `apps/desktop/src/features/compose/
{composed.ts, ComposedPicture.tsx}`.
Frontend (changed): `apps/desktop/src/App.tsx`, `features/shell/Header.tsx`, `features/workspace/{WorkspacePage.tsx, Stage.tsx, StageImage.tsx,
PhotoTile.tsx, PhotoMenu.tsx, PhotoPanel.tsx, FoldedStrip.tsx, ItemDetailsCard.tsx, workspace.css}`, `features/compose/{ComposeMode.tsx,
CompositionSection.tsx, compose.css}` (only the `canSplit` removal and the `.composed-*` rules if they live there), `services/compositions.ts`.
Tests (new): `apps/desktop/src/features/compose/__tests__/composed.test.ts`, `features/workspace/__tests__/{p35_composed_display.test.tsx,
p35_chrome.test.tsx}`, `crates/testkit/tests/p35_split_any_photo.rs`. Tests (changed): the five listed in Read first.
Estimated: 18 source paths, 4 new tests, 5 adjusted tests (mostly selector/expectation edits).

## Core mechanisms (unit tests allowed)
- `features/compose/composed.ts` — `composedLayout(values, frame) → { ratio: string, ratioValue: number, frame: { width, height, left, top:
  string }, rotate: string, imageTransform: string, swapped: boolean }` (box ratio `"<px(w·W_t)> / <px(h·H_t)>"` with `px = Number(v.toFixed(4))`;
  percentages `${Number((v·100).toFixed(4))}%` of `1/w`, `1/h`, `−x/w`, `−y/h`; `rotate(<straighten>deg)`; `rotate(<turns·90>deg) scaleX(<±1>)`;
  `swapped = turns odd`) and `leavesFrameOut(values, frame) → boolean` (C3(c)).

## Ordered tasks
0. **Registry first (master plan §6.10, amended).** `COMPOSE_WAITING = "Waiting for Capture One's full-frame preview"` (straight apostrophe, as the
   book writes it) exported from `services/compositions.ts`; `HINT_DELAY_MS = 400`; sonner toast id `compose-waiting`; CSS `.stage-composed`,
   `.tile-composed`, `.composed-frame`, `.composed-split`, `img[data-swapped]`, `.tile-crop-mark`, `.stage-compose`, `.panel-sessions`; CSS
   variable `--composed-ratio`. Retired: `.tile-composition`, `.tile-frame`, `.tile-split`, `.sessions-toggle`. No command, no key, no setting.
1. **`Hint`** (`components/ui/tooltip.tsx`): the shadcn Tooltip primitives over `radix-ui`'s `Tooltip` plus `Hint({ text, children })` — the child
   is the trigger (`asChild`), the content reads `text` exactly; each `Hint` carries its own `Tooltip.Provider` with `delayDuration =
   HINT_DELAY_MS` so it works in any tree (tests render `App` and components alone). A control that cannot act is marked `aria-disabled="true"`,
   **never** `disabled` (a disabled button gets no pointer or focus events, so its hint could not say why). Controls this phase touches lose
   their `title` (no double tooltip).
2. **`ComposedPicture`** (`ComposedPicture.tsx`, the one shared renderer, callers: `StageImage` and `PhotoTileFace`): props `{ values, frame,
   className, children }`; renders `<div class={className} style={{ aspectRatio, '--composed-ratio' }}>` → `<div class="composed-frame"
   style={{ width, height, left, top, transform: rotate }}>{children}</div>` and, when `values.kind === 'split'`, `<span class="composed-split"
   style={{ left: '50%' }} aria-hidden>` on the box. CSS: the box is centred in its container and contained (`width: min(100cqw, calc(100cqh *
   var(--composed-ratio)))`, `aspect-ratio` from the style, `overflow: hidden`, `container-type: size` on `.composed-frame`), images inside at
   `left/top 50%`, `translate: −50% −50%`, `width/height 100%`, and `img[data-swapped] { width: 100cqh; height: 100cqw }`. `OutputPreview`/
   `SplitPreviews` may move onto it (Delegated); they are not required to.
3. **Stage** (`StageImage.tsx`): when `valuesOf(photo.composition)` and `photo.composition.frame` are non-null, `.stage-frame` contains
   `<ComposedPicture className="stage-composed">` whose children are today's layers (same `alt`, `crossfade`, `data-visible`, `data-newest`,
   `onLoad` layering): each adjusted layer gets `style.transform = imageTransform` and `data-swapped` when `swapped`; the camera original gets
   `imageTransform` followed by its `orientationTransform(photo.orientation)` (when not `undefined`) and `data-swapped` when `turns + (orientation
   ∈ 5…8 ? 1 : 0)` is odd (the camera JPEG is stored unrotated; the adjusted preview is already upright). Otherwise (no rect or frame) the stage
   is exactly today's. `.stage-frame` stays the double-click target.
4. **Tiles** (`PhotoTile.tsx`): `TileComposition` becomes `<ComposedPicture className="tile-composed">` around `<Thumb>` (thumbnail, else adjusted
   preview; `Thumb` takes the transform and `data-swapped`); `.tile-frame`/`.tile-split` are gone. Grid variant only: `<span class="tile-crop-mark"
   aria-label="Part of the frame is left out">` (a small `Crop` glyph; placement Delegated, clear of the number, Cover, Processing, Re-frame and
   upload badges) when `leavesFrameOut(values, frame)`. The drag overlay and the strip follow through `PhotoTileFace`.
5. **`leavesFrameOut`** (`composed.ts`): `straighten ≠ 0`, or with the MC-27 pixel rule on the turned frame `round(w·W_t) < W_t` or
   `round(h·H_t) < H_t` (half away from zero; float noise never marks a full-frame photo).
6. **Compose button** (`Stage.tsx`): outside the editor, in `.stage-tools` before the ⋯ button, `<button class="stage-compose" aria-label="Compose">`
   (`Crop` icon + the word "Compose") inside `Hint` with `text = canCompose(active) ? 'Compose (R)' : COMPOSE_WAITING`; unavailable →
   `aria-disabled="true"`. A click calls the new `onCompose` (WorkspacePage: `canCompose ? enterCompose : toast(COMPOSE_WAITING, { id:
   'compose-waiting' })`). The same handler serves `r` (its `useKeyboard` is enabled whenever a photo is on the stage, not only when composable)
   and the stage double-click. Hidden while composing (Done is there).
7. **Menu** (`PhotoMenu.tsx`): the unavailable `Compose…` entry stays `disabled` and shows `<small class="photo-menu-why">{COMPOSE_WAITING}</small>`
   in place of the `R` hint. The `Split` entry is disabled only by `!canCompose(photo)` (the unreviewed rule goes).
8. **Split for any photo**: remove `canSplit` (`CompositionSection.tsx`, `ComposeMode.tsx`) and the unreviewed skip in `WorkspacePage.setKind`; the
   Split segment's title/hint stays `Split (S)` for every photo (P36 wires the hints; this phase only removes the gate and the "Split is for Main
   and Condition photos" wording). `outputs.ts` is unchanged (an unreviewed photo still counts one place).
9. **Header Details** (`App.tsx`, `Header.tsx`, `WorkspacePage.tsx`): lift `formOpen` to `App` as `detailsOpen` exactly like `assignOpen`
   (`WorkspacePage` props `detailsOpen`, `onDetailsOpenChange`; reset to closed when the session changes); `Header` gets `onDetails` and its
   Details `Button` is enabled (no `disabled`, no placeholder title) inside `Hint text="Details (D)"`; D keeps opening it; the stage's "Product
   details" button is deleted from `stageExtras`. `ItemDetailsCard` gets `onEdit` and its "Edit details →" is enabled and opens the same form
   (fold-in).
10. **Sessions icon** (`PhotoPanel.tsx`, `FoldedStrip.tsx`, `WorkspacePage.tsx`): delete the stage's Sessions button; `PhotoPanel` and `FoldedStrip`
    take `sessionsOpen` + `onToggleSessions` and render `<button class="stage-icon-button panel-sessions" aria-label="Sessions"
    aria-pressed={sessionsOpen}>` (`PanelsRightBottom` icon, no text) inside `Hint text="Sessions"` — in the panel head beside Import and Fold, in
    the strip under Unfold. A click toggles the Sessions panel. Import and Fold/Unfold move from `title` to `Hint` ("Import photos", "Fold panel
    (F)", "Unfold panel (F)"). The edge-reveal browser (P31) passes them through (P36 replaces it).
11. **Apply clears Re-frame** (fold-in, `WorkspacePage.tsx` `apply`): the patch of an applied photo also sets `state: 'ok'`, `review_reason: null`
    (what `save_composition` stores, `store.rs:993`).
12. Tests (component + one Rust file), the five adjusted tests, `npm run typecheck`, `npm test`, `cargo test --workspace`, clippy, fmt.

## Criteria (7 criteria, 18 rows; 10 `[rule]` rows — UI rows with no ledger measurement behind them, master plan §3)
Fixture: the compose harness (`features/compose/__tests__/harness.tsx`: frame `2560 × 1707`, default square `A = (0.16660156250000002, 0,
0.666796875, 1)`, `photo()`/`view()`/`openWorkspace`); `S` = the split default of `p33_split_grid.test.tsx` (`x 0.028503408013501264, y
0.1464466094067262, w 0.9429931839729975, h 0.7071067811865476`). Hints are read as the text of the `role="tooltip"` element shown after the
control receives focus (`act(() => el.focus())`) or a pointer move, waiting up to 1 s.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `<App>` workspace, photo 1 (default square) active | focus the stage-bar button named `Compose`; click it | the button sits in `.stage-bar .stage-tools`, has no `aria-disabled`; the tooltip reads `Compose (R)`; the click shows `.compose-stage` | `[rule]` 10D ("or the Compose button on the stage"); R38 |
| C1(b) | photo 1 with `composition.rect: null, frame: null` (preview present) active | focus `Compose`; click it; double-click `.stage-frame`; press `r`; open the ⋯ menu | `aria-disabled="true"`; tooltip `Waiting for Capture One's full-frame preview`; after each of click / double-click / `r` no `.compose-stage`, and exactly one visible toast with that text (id `compose-waiting`); the menu's `Compose…` item has `data-disabled` and its text contains the same sentence and no `R` | `[rule]` 10D ("Compose is unavailable only for a photo without a full-frame preview yet; its button and menu entry say why") |
| C1(c) | photo 1 `state: 'needs_review'`, `review_reason: 'geometry'`, frame present | click `Compose` | enabled; `.compose-stage` shown (needing review never blocks Compose: "only") | 10D ("unavailable only …"); MC-27 §Revisions (the editor confirms with Return) |
| C2(a) | stage, photo 1 = `A`, adjusted mode | — | `.stage-frame .stage-composed` `style.aspectRatio` `1707 / 1707`; its `.composed-frame` `width 149.9707%`, `height 100%`, `left -24.9854%`, `top 0%`, `transform rotate(0deg)`; the `img[alt="Adjusted"]` inside it has `transform rotate(0deg) scaleX(1)` and no `data-swapped`; no `.composed-split` | `[rule]` 10D ("the stage shows the photo as composed … cut to the frame … not the whole frame") |
| C2(b) | photo 1 = `S` (`kind split`) | — | `aspectRatio` `2414.0626 / 1207.0313`; `.composed-frame` `width 106.0453%`, `height 141.4214%`, `left -3.0227%`, `top -20.7107%`; `.stage-composed .composed-split` present with `left 50%` | `[rule]` 10D ("a split shows both halves with the middle hairline") |
| C2(c) | photo 1: `quarter_turns 1`, `flip 'horizontal'`, `straighten 5`, rect `(0, 0.16660156250000002, 1, 0.666796875)`, orientation 1 | — | `aspectRatio` `1707 / 1707`; `.composed-frame` `width 100%`, `height 149.9707%`, `left 0%`, `top -24.9854%`, `transform rotate(5deg)`; adjusted img `transform rotate(90deg) scaleX(-1)` with `data-swapped` | 10D ("turned, flipped, straightened, cut to the frame"); MC-27 §Values (order: turns and flip, then straighten about the centre) |
| C2(d) | photo 1: frame `1707 × 2560`, orientation 6, rect `(0, 0.16660156250000002, 1, 0.666796875)`, no turn; press Space (Original); separately photo 2 with `rect: null, frame: null`, orientation 6, active | the camera original; the uncomposed stage | photo 1: the original img is inside `.composed-frame`, `transform rotate(0deg) scaleX(1) rotate(90deg)`, `data-swapped` present, `data-visible="true"`; the adjusted img `rotate(0deg) scaleX(1)` without `data-swapped`; photo 2: no `.stage-composed`, the original is a `.stage-layer` with `transform rotate(90deg)` (today's stage) | 10D ("The result is shown"); 06 §Original / Adjusted; MC-20 (orientation of the camera JPEG) |
| C3(a) | grid (`.photo-panel`): photo 1 = `A`; photo 2 = `shape 'original'`, rect `(0, 0, 1, 1)`; photo 3 = `S`; photo 4 `rect: null, frame: null` | the tiles | tile 1: `.tile-composed` `aspectRatio 1707 / 1707`, `.composed-frame` `left -24.9854%`, `width 149.9707%`, `img.thumb-image` src ends `thumb-1.jpg`, `.tile-crop-mark` present; tile 2: `aspectRatio 2560 / 1707`, `.composed-frame` `width 100%`, `left 0%`, no `.tile-crop-mark`; tile 3: `.composed-split` present, `.tile-crop-mark` present; tile 4: no `.tile-composed`, `img.thumb-image` src `asset:///previews/4/thumb-1.jpg`, no `.tile-crop-mark` | 10D ("Tiles show the composed picture too, with a small crop mark when the composition leaves part of the frame out"); MC-28 §Thumbnail |
| C3(b) | press `f` (folded strip), photo 1 = `A` | strip tile | `.folded-strip` tile 1 has `.tile-composed` (`aspectRatio 1707 / 1707`) and no `.tile-crop-mark` (badges stay grid-only, plan Notes) | `[rule]` 10D ("Tiles show the composed picture") |
| C3(c) | `composed.test.ts`, `leavesFrameOut` on frame `2560 × 1707` | `(0,0,1,1)` st 0; `A` st 0; `(0, 0, 0.99999, 1)` st 0; `(0, 0, 0.9998, 1)` st 0; `(0,0,1,1)` with `quarter_turns 1`; `(0.05, 0.05, 0.9, 0.9)` st 3; `S` | `false`; `true`; `false` (2559.97 rounds to 2560); `true` (2559.49 → 2559); `false`; `true`; `true` | changelog 28 reading ("the crop mark shown whenever the rect is smaller than the frame or the photo is straightened"); MC-27 §Pixel rule |
| C4(a) | the P33 grid fixture (1 split, 2, 3 split, 4 unreviewed with a frame; 5 split, 6 condition); right-click tile 4 → `Split S` | split an unreviewed photo | the item has no `aria-disabled`/`data-disabled`; one `setComposition(4, v)` with `kind 'split'`, `shape '1:1'`, rect `(0.0285, 0.1465, 0.943, 0.7071)` ± 1e-3; the answer `saved(4, v)` → tile 4 shows `.composed-split`; Main numbers stay `01–02, 03, 04–05, 06` (an unreviewed photo fills one place) | 10D ("Allowed for any photo, unreviewed included … places and outputs are counted only once it is Main or Condition"); MC-29 §Outputs |
| C4(b) | twoPhotos with photo 1 `unreviewed`; press `r`, then `s`; separately click the `Split` segment | split in the editor | the `Split` segment has no `aria-disabled`; each input sends one `setComposition(1, v)` with `kind 'split'` and the rect of C4(a) | `[rule]` 10D ("Split … Allowed for any photo") |
| C4(c) | `crates/testkit/tests/p35_split_any_photo.rs`: one session, one `unreviewed` photo whose frame is materialised at `2560 × 1707` (as `p34_composition_apply.rs` builds it); `CompositionService::set(photo, { kind split, shape 1:1, rect S })` | the service has no classification rule | `Ok`, stored `kind split`, `revision 1`, `state ok`; reading the photo back gives the same rect (fractions equal to `1e-12`) | 10D ("Split … any photo"); MC-27 §Validation (the only refusal reasons) |
| C5(a) | session workspace (item `Chair`); `@/services/form` mocked | click the header button named `Details`; close; press `d`; close | the button has no `disabled` and its tooltip reads `Details (D)`; each input opens `role="dialog"` named `Chair` (the product details form); `.stage-column` holds no button named `Product details` | `[rule]` 10D ("the header's Details button opens the product details form (D); the duplicate Details button on the stage is removed") |
| C5(b) | press `i` (item card); click `Edit details →` | the card's edit | the button is enabled and opens the same dialog | `[rule]` fold-in: P31 narrative / design handoff §Keyboard (the details form); same placeholder as C5(a) |
| C6(a) | session workspace, panel unfolded | look for the Sessions control; click it twice | nothing named `Show sessions panel` anywhere; `.panel-head .panel-tools` holds a button named `Sessions` with empty text content (icon only), tooltip `Sessions`, `aria-pressed="false"`; first click → `aside[aria-label="Sessions"]` present, `aria-pressed="true"`; second click → the panel gone | `[rule]` 10D ("The Sessions button moves from the stage to the photo panel's header (beside fold and export) as an icon with a tooltip") |
| C6(b) | the same, press `f`; separately the Unassigned workspace (session 1) | the strip; Unassigned | the `.folded-strip` holds the same `Sessions` button and it toggles the panel; in Unassigned the panel is open on entry and the button reads `aria-pressed="true"` | `[rule]` 10D ("Workspace chrome"); ch06 §Unassigned (sessions panel open) |
| C7(a) | photo 2 `state: 'needs_review'` (tile shows `Re-frame`); clipboard holds `V` (⌥⌘C in the editor on photo 1, then Return); ⌘-click tile 2; right-click → `Apply composition to selected`; mock answers `{ applied: [2], refused: [] }` | Apply on a photo needing review | tile 2 has no `.tile-review` afterwards; `getSessionView` not called again | fold-in: P34 narrative "Not yet" (badge kept until reload); MC-27 §Revisions (a saved composition clears `needs_review`) |

## Delegated choices
Whether `OutputPreview`/`SplitPreviews` are rebuilt on `ComposedPicture`; the crop mark's glyph, size and corner; the Compose button's icon and
whether its label text hides at narrow widths (the accessible name stays `Compose`); the hint's styling (shadcn tooltip tokens) and side; the
toast kind for `COMPOSE_WAITING` (info or default); how `App` resets `detailsOpen` on a session change; how tests mock `@/services/form`.

## Manual try-out
1. Open a session with framed photos. The stage now shows each photo **as it will be exported**: a square crop is a square on the stage, a
   turned or straightened photo appears turned or straight, and a split photo shows its two halves side by side with a thin line in the middle.
   Press Space: the camera original appears with the same framing; Space again returns.
2. Look at the tiles: they show the same composed pictures. A small crop mark sits on every grid tile whose framing leaves part of the photo out
   (with the default Square, that is most photos); a photo framed as Original has none. Press F: the strip shows the composed pictures too.
3. A **Compose** button sits on the stage bar; hover it: "Compose (R)". On a photo whose full-frame preview is still coming from Capture One
   (right after first starting this build, or a new capture still processing), the button is greyed, its hint and the ⋯ menu say "Waiting for
   Capture One's full-frame preview", and R shows that message instead of doing nothing.
4. Right-click a photo still to review → Split: it splits; its number stays one place until you approve it into Main or Condition.
5. Click **Details** in the header: the product details form opens (D still works). The stage no longer has its own Details or Sessions
   buttons. In the photo panel's header a Sessions icon sits beside Import and Fold; hover: "Sessions"; click: the sessions panel opens and
   closes. The item card's "Edit details →" opens the form too.
6. Copy a framing onto a photo that shows "Re-frame" with "Apply composition to selected": the badge goes away at once.

## Notes
- **Planner readings (reversible; for the narrative's "Choices made for you")**:
  (1) The crop mark shows whenever the composition is smaller than the whole frame or straightened (changelog 28), so with the default Square
  nearly every tile carries it; say "mark only photos I re-framed" to restrict it to `origin user`.
  (2) The crop mark is drawn on grid tiles only; strip tiles (44 × 33 px) carry no badges today and stay that way.
  (3) Space shows the camera original **with the same framing** (both layers sit in the composed box), so the comparison is like for like.
  (4) A photo that needs review is shown with its stored framing (its tile also shows "Re-frame"); Compose stays available for it.
  (5) Clicking the greyed Compose button, pressing R or double-clicking shows the waiting message as a toast (one at a time), besides the hint.
  (6) "Beside fold and export" (book): the photo panel's header holds Import and Fold (Export is in the app header), so the Sessions icon sits
  beside Import and Fold; it also sits in the folded strip so the sessions panel stays reachable when the panel is folded.
  (7) The header Details button also works while composing (it is a click, not a workspace key); the form opens over the editor.
- **Why a DOM tooltip, not `title`**: the owner asked that hovering any control show its name and key; a Radix tooltip shows the same way in
  WebKit and in jsdom, and works on an `aria-disabled` control, which a `disabled` one cannot. P36 applies `Hint` to every editor control.
- **Hazard (P31–P34 reviews)**: never assert a value the implementation computes for itself — every expected string above is hand-computed from
  the fixture (`composed.ts` is tested through the DOM rows and C3(c)).
- **jsdom limits**: container queries (`cqw`/`cqh`), the real contained size of the composed box and the visual hairline are not evaluated in
  jsdom; the rows assert the inline styles and attributes that produce them, and the manual try-out covers what the eye sees.
- Registry additions made by the planner in master plan §6.10 (P35 block). No intention-book change; no backend change (C4(c) is a guard).
- Estimated size: ~27 paths (18 source, 4 new tests, 5 adjusted tests), ~900 lines (≈ 400 features, ≈ 500 tests).

## Review log

- **2026-11-05 implement r1 (codex).** Added one CSS composition renderer shared by the stage and tiles, the pixel-rule crop mark, Compose availability messaging and button, Split for unreviewed photos, working header/card Details, Sessions controls in the photo panel and strip, and immediate Re-frame clearing after Apply. Updated earlier UI tests whose overlay selectors and menu expectations changed; added P35 UI and Rust boundary tests. The browser control remains the P31 edge-reveal browser for P36 to replace. Assumptions: the crop mark uses the `Crop` glyph at the upper right below the unreviewed dot; waiting Compose uses Sonner's default toast; Details closes on session change using an App effect; preview components retain their own renderer. Deviation: `BrowserOverlay.tsx` gains Sessions prop pass-through, which task 10 requires though the estimated changed-path list did not name it. Checks: `cargo test --workspace`, clippy with `-D warnings`, fmt check, `npm run typecheck`, and `npm test` green (320 UI tests). No live camera probe or visual browser run; the Browser plugin and a local Playwright package are unavailable in this session.
- **2026-11-05 review r1 (opus) — CHANGES_REQUESTED.** Gate: tracker `IMPLEMENTED`, checkpoint `c8d3715` in `git log`. Perimeter: `git show --stat c8d3715` = exactly the 31 paths the handoff declares (`BrowserOverlay.tsx`, `p31_compose_mode.test.tsx` disclosed; `compose.css` unchanged, allowed). Checks (code tree `c8d3715`, HEAD `45252df`, clean): `cargo test --workspace` all `ok`, clippy `-D warnings` and fmt clean, typecheck clean; `npm test` 1 failed / 319 on the first run (failing ID not captured), then 320/320 on 12 re-runs incl. 3×3 parallel-load runs: an unreproduced flake, noted, not B4.
  **F1 (blocking, B1, route production).** The stage does not show the photo as composed: it is drawn a quarter of the frame height too high. Inside `.stage-composed` the layers keep `class="stage-layer …"`, and `.stage-layer { inset: 0; margin: auto }` (`workspace.css:35`) survives `.composed-frame img` (`:88`, which sets only `left/top/width/height`). With `top 50%`, `height 100%`, `bottom 0` and both block margins `auto`, CSS (abspos replaced, block axis) gives equal negative margins: measured in WebKit (WKWebView probe, real `workspace.css`, C2(a)/C2(b) DOM on a 1000 × 700 stage): adjusted img `margin-top −175px`, img top `−175` vs `.composed-frame` top `0` (C2(b): `−176.8px`); the tile (`.thumb-image`, no inset) is exact. Tailwind's preflight sits in `@layer base` and cannot override the unlayered rule. Authority: 10D §What the photographer sees, "The result is shown" ("the stage shows the photo as composed … cut to the frame"); plan task 2 (images at `left/top 50%`, `translate −50% −50%`). Correction: in `.composed-frame img` add `right: auto; bottom: auto; margin: 0` (verified in the same probe: img rect = frame rect on both fixtures), or drop the `stage-layer` class inside the composed box. Files: `apps/desktop/src/features/workspace/workspace.css` (optionally `StageImage.tsx`). No new test required (jsdom does not lay out; plan Notes "jsdom limits").
  **Verified correct.** `composedLayout` / `leavesFrameOut` re-derived by hand for C2(a)–(c), C3(a), C3(c) (pixel rule half-away on the turned frame; straighten ≠ 0); the transform order (turns+flip on the image, straighten on the frame about its centre; camera original = `imageTransform` then orientation, swap parity `turns + (orientation 5–8)`) matches MC-27 §Values and MC-20; Compose button / menu reason / R / double-click all route through `enterCompose`, which toasts `COMPOSE_WAITING` with id `compose-waiting` (changed-input probe: R alone and double-click alone each show the toast); Split gate removed in all three sites, `outputs.ts` untouched; Details lifted to `App` like `assignOpen`, reset on session change, header/D/card open the same `FormDialog`, stage duplicate gone; Sessions icon in panel head and strip with `aria-pressed`, old toggle gone; Apply patches `state 'ok'`, `review_reason null` as `save_composition` stores. All 18 rows have a test at their boundary that fails on a wrong outcome (C2 rows assert the inline styles, which are right; F1 is in the cascade). Adjusted P31–P34 expectations re-derived (`−x/w`). Assumptions: none contradicts the intention. H1–H8 hold (frontend only; one guard test in Rust).
  Notes → backlog (5). Probe files: `/tmp/p35_probe/*` (outside the repo); a temporary `src/features/workspace/__tests__/zz_review_probe.test.tsx` was created, run and deleted (`git status --porcelain` empty afterwards). No production file touched.
- **2026-11-05 fix r2 (codex).** Resolved F1 by adding `right: auto; bottom: auto; margin: 0` to `.composed-frame img`, clearing the inherited `.stage-layer` positioning values. No test added as directed by the finding (jsdom does not lay out). Checks: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `npm test` (320 UI tests), and `npm run typecheck` green. Perimeter: workspace CSS, this Review log, the P35 tracker row, and fix r2 handoff.
