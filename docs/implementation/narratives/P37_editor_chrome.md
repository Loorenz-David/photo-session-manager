# Phase P37_editor_chrome: a tidier editor — icons above the photo, hover cards, a Tools panel that hides into a rail, and rarely used tools folded away

## What you can do now
Above the photo, the editor now has five small icons instead of words: Revert, Loupe, Snapping, Copy framing and Paste framing, then
Done. Hover any editor control and a small card gives its name, its key and one line on what it does, for example "Loupe · Z — 100 %
view under the pointer". The frame meter no longer has a button. ⌥⌘P still shows and hides it.

The Tools panel has its own header, "Tools", with a hide button. When you hide it, a thin rail stays on the left edge with one button
that brings the panel back. ⌘T still does the same, and the app still remembers whether the panel was hidden. In Composition, the Turn
button is a small icon beside the Shape menu. Rotate left, Rotate right and Flip sit in a folded "Rotate and flip" drawer. The
straightening-grid switch is a grid icon beside the degrees field. The nudge arrows sit in a small keyboard-style pad behind a "Nudge"
button. Nothing the editor does has changed, only where the controls sit and how they look.

## Try it yourself
1. Run this phase's checks (no camera or Capture One needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npx vitest run p37
   ```
   12 should pass.
2. Start the app and open a session with photos:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. **Icons and cards.** Press R. Above the photo you see five icons and Done, with no "Frame meter" or "Tools panel" buttons. Hover each
   icon: a card gives name · key — one line. Press ⌥⌘P: the frame meter appears. Press ⌥⌘P again: it goes away.
4. **Tools panel.** Click the hide button in the panel's "Tools" header. The panel becomes a thin rail on the left with one button. Hover
   it: "Show Tools panel · ⌘T — Brings the Tools panel back". Click it to bring the panel back. Press ⌘T twice to hide and show it again.
   Hide it, press Return, then press R: the rail is still there.
5. **Turn and drawer.** Pick 4:5 in the Shape menu and click the turn icon beside it: the frame turns. Open "Rotate and flip" and click
   Rotate right: the photo turns under the frame. Press Return, then R: the drawer is closed again.
6. **Grid icon.** Click the grid icon beside the degrees field to turn it off, then click into the degrees field: no grid appears. Quit and
   relaunch the app: the grid is still off.
7. **Nudge pad.** Click "Nudge": a small pad opens with ↑ on top and ← ↓ → below. Click → once (one pixel), then hold ⇧ and click it
   (ten pixels). Press ] for the next photo: the pad stays open. Press Return, then R: the pad is closed again.
8. **Look closely.** These checks ran in a test browser that does not draw the screen. Check that the rail looks slim, the pad looks like
   an inverted T, and the long cards (Paste framing, Snapping) fit on screen near the right edge.

## Choices made for you
- The rail is 36 pixels wide. Say "a wider rail" to change this.
- The Rotate and flip and Nudge toggles show a short word beside their icon, so you can find them. The buttons inside are icons only.
  Say "icons only for the drawers too" to change this.
- The cards use the hint font you already had, with the name in bold. Say how you want them styled to change this.
- The Rotate and flip drawer and the Nudge pad start closed every time you open the editor. While you edit, they stay as you left them
  when you switch photos, hide and show the panel, or fold and unfold Composition. Say "remember the drawers" to change this.
- The planner wrote the wording on every card. The book fixed only the format and the Loupe example. Say which line to reword.
- The ‹ › arrows on the stage and the photo panel's fold and unfold buttons show the same cards outside the editor too.
- Controls with no key (the drawer, the rotate and flip icons, the straighten slider, the grid switch) show just name — one line.

## Not yet
- The Item details (ⓘ) button above the photo still shows the old plain grey tooltip, and the ⋯ menu button shows none. Both are workspace
  buttons and were not part of this phase. This is noted for later.
- Colour and light tools, zoom buttons and one shared save queue for every editor action are later work.
- One older automatic check from the split phase still fails now and then when all checks run together. It failed once during building
  and passed in the review run.

## Needs you
Nothing.
