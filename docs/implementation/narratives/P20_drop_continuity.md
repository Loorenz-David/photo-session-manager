# Phase P20_drop_continuity: the photo you drag is the photo that lands, same size, no pop

## What you can do now
When you drag a photo in the session's photo panel, the floating copy is now exactly the size and shape of the tile you picked up. It no longer shrinks to a small tilted card. The empty slot it came from no longer shows a faded ghost. While you drag and while it glides into place, only one photo is visible. When you let go, the photo glides into its new slot in a fifth of a second, keeping its size the whole way, and the tile is simply there when it arrives, with no fade-in and no pop. This works in the grid, between Main and Condition, and in the folded strip, where the small strip tile keeps its own small size. If your Mac has "Reduce motion" turned on, the photo goes straight to its new slot with no glide.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or shop needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npx vitest run src/features/workspace/__tests__/p20_drop_continuity.test.tsx src/features/workspace/__tests__/p19_drop.test.tsx
   ```
   It should read `Tests  6 passed (6)`.
2. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Open a session with three or more Main photos. Drag the first photo onto the third slot and let go. **Watch closely.** The photo you hold is the same size as the tiles, its old slot is empty rather than faded, and it glides into the third slot and stays. Nothing should shrink, fade or jump.
4. Watch the small number in the photo's corner as it lands. It shows the old number ("01") while gliding and changes to the new one ("03") when it arrives. If you drag a photo into or out of the first slot, the "Cover" label appears or disappears at the same moment. Tell me if that bothers you (see "Choices made for you").
5. Drag a Main photo onto the Condition group. It should glide to the end of Condition in the same way.
6. Fold the photo panel into the strip and drag a small tile. It should stay small the whole time.
7. Optional: turn on System Settings → Accessibility → Display → Reduce motion, open a session again and drag. The photo should go straight to its slot with no glide.

## Choices made for you
- The photo you hold gets no lift effect (no shadow, no tilt, no enlargement), so it looks exactly like the tile it becomes. Say "give the dragged photo a slight shadow while I hold it" if you'd like it to look picked up.
- The glide takes 200 milliseconds and uses the app's standard easing. Say "make the drop slower" or "faster" to change it.
- With Reduce motion on, there is no glide at all. Nothing to change.
- The photo shows its old number and Cover label while gliding and switches to the new ones when it lands (step 4). Say "show the new number while I'm still dragging" to have the number follow the slot under your pointer, or "hide the number while dragging" for a plain picture.

## Not yet
- Nobody has watched the glide on a real screen yet. The automated tests cannot play animations, so step 3 is the only proof that nothing shrinks, fades or pops.
- The number and Cover label change at the moment the photo lands, not before (step 4).
- The detail form's reordering (customize mode) has no floating copy and is unchanged.

## Needs you
Nothing.
