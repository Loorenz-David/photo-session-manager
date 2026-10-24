# 06 — Photo review and ordering

## Arrival
A capture reaches us two ways; both write `capture_events`, and the photo row is created once per
(session, raw_path) (M1):
1. the provider's capture-done hook posts the RAW path to the local API (primary);
2. the folder watcher sees the file (fallback, and reconciler at startup).
The photo joins the **active session** as `unreviewed` at the end of Main; with no active session
it joins Unassigned. The original preview (embedded camera JPEG) is written immediately; the
provider is asked for the adjusted preview; `adjusted_preview_path` arrives via the
processing-done hook or the watcher. Until then the photo is "Processing".

## Classification (one state, owned by the photographer)
`unreviewed` → `main` (A, swipe right) · → `condition` (C, swipe up) · → `deleted` (⌫, swipe left).
A condition photo's middle action is "Move to Main". After an action the next unreviewed photo
opens. `deleted` hides the photo; the RAW stays (H2). Only the last delete is undoable (⌘Z, toast
4.5 s, works after the toast until the next delete); undo restores the original position and
makes the photo active.

**Mirror to the provider (decision D2):** `main` → green tag, `condition` → blue tag, `deleted` →
red tag, `unreviewed` → no tag. A tag changed inside Capture One is read back by polling (every
3 s for the active session, and when a session opens) and applied as the same classification.
The last writer wins; conflicts within one poll interval resolve to the provider's value (M2).

## Order
Main and Condition are separately ordered, 1-based, contiguous. Main 01 is the cover. Reorder by
drag in the grid or the folded strip; a photo moving between groups goes to the end of the target.
Provider sort order is never used.

## Original / Adjusted
One toggle per session view (pill or Space): camera original or provider-adjusted. Crossfade
700 ms. While processing, the pill shows a spinner and the original is shown; adjusted fades in
when it arrives.

## Unassigned, move, copy
Unassigned is a built-in session with no item. Its workspace opens the sessions panel; dragging
photos onto a session **moves** them (re-parents the row). The ⋯ menu offers Move to session…
(same) and Copy to session… (creates a second photo row on the same RAW; both sessions may
publish it). "Assign product" assigns the whole Unassigned set to an item's session (creating the
session if needed). Published sessions are not move targets.

## Import
Photos may be added from files (drag onto the workspace or a picker). They follow the same path
as captures, with `source = import` and no provider hook; the provider is asked to import them
into the document so adjustment and export work identically.
