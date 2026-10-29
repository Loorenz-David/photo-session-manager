---
phase: P19_navigation_and_drop
title: Navigation and drop — one page header with Back top-left, drop preview settles into the new slot
difficulty: normal
projection: no
depends_on: [P18_pending_state_ui]
---
# P19 — Navigation and drop: one page header with Back top-left, drop preview settles into the new slot

## Goal
Fix the two findings of the owner's second real run (`docs/implementation/real_run_2026-10-26.md`, "Second real run",
R7 and R8). **R7** — owner: "when I go into settings it was hard for me to identify the back buttons, I believe we
should standardize the back button on the top left because that is how we are used to." One shared page header is
used by every routed page: Back at the top left on every non-root page (its parent is fixed per route; the Sessions
list is the root and has no Back), the page title next to it, the page's actions on the right; the ad-hoc
"Sessions"/"Settings" buttons of Health and Settings go. ⌘[ goes back on every non-root page; Escape goes back on
Settings and Health only (in the workspace Escape keeps its handoff meaning: close the topmost layer). **R8** —
owner: "when re-organizing pictures … dragging on top moves the grid (great) but when I drop the picture the drag
preview moves to where it was first dragged; this is confusing and a common problem using dnd, an easy fix." A
dropped photo's preview settles into its **new** slot: the workspace applies the new order to what it shows the
instant the drop happens (before the backend answers), which is what dnd-kit's drop animation measures; a refused
reorder snaps back to the stored order with a toast. **Not in this phase:** visual redesign of Health and Settings
(undesigned in the handoff), a history-based Back (parents are static), backend changes (none), the folded strip's
and Customize mode's reordering beyond what the shared mechanism already covers (see Notes).

## Read first
`docs/implementation/real_run_2026-10-26.md` "Second real run" R7, R8 · master plan §6.1 (`apps/desktop` layout),
§6.9 (frontend tests), §7.2 (commands) · book `01`, `14`, `06` §Order (reorder by drag in the grid or the folded
strip; a photo moving between groups goes to the end of the target), §Unassigned, `11` §Frontend ↔ backend
(components call `services/`, never `invoke`), `12` §Settings, §Health · evidence `design_handoff.md` §Keyboard (Esc
closes the topmost layer; N, D, …) and §Tokens and components · `claude_design/COMPONENTS.md` row "Header bar
(56px)": "Back to Sessions, item type, SKU and ⓘ; camera status, bell, form button, Publish" (the only header
guidance; Health and Settings are "Not in the handoff") · `apps/desktop/src/App.tsx:15` (the four routes: `#/`,
`#/session/:id`, `#/health`, `#/settings`; hash routing) · `apps/desktop/src/features/shell/Header.tsx` (today's
Sessions and workspace headers; the workspace already has Back first) · `apps/desktop/src/features/workspace/
{WorkspacePage.tsx (`DndContext`, `DragOverlay`, `useDrop`, `onDragEnd`), useWorkspaceState.ts (`run`, `reorder`,
`classify`), order.ts (`displayOrder`), PhotoTile.tsx (`useSortable`), PhotoGrid.tsx}` ·
`apps/desktop/src/features/workspace/__tests__/p08_workspace_ui.test.tsx` (jsdom polyfills, `layout`, `drag`,
`view`, `photo`, `openWorkspace`, `panelOrder` helpers; C4(b)/C4(c) are the drags this phase re-uses) · P18 plan
Review log (pending-state primitives; `ActionButton`).

## Files expected to change
`apps/desktop/src/features/shell/PageHeader.tsx` (new: `PageHeader`), `apps/desktop/src/features/shell/Header.tsx` (both branches render through `PageHeader`),
`apps/desktop/src/features/health/HealthPage.tsx`, `apps/desktop/src/features/settings/SettingsPage.tsx` (headers replaced; ad-hoc navigation buttons removed),
`apps/desktop/src/App.tsx` (`parentRoute`), `apps/desktop/src/lib/keyboard.ts` (`useBackKeys`),
`apps/desktop/src/features/workspace/useWorkspaceState.ts` (optimistic order for `reorder` and drag `classify`), `apps/desktop/src/features/workspace/WorkspacePage.tsx` (only if `onDragEnd` needs to pass the drop kind),
`apps/desktop/src/styles/globals.css` (`.health-header` block retired or re-pointed; `.app-header` reused),
`apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx` (new), `apps/desktop/src/features/workspace/__tests__/p19_drop.test.tsx` (new),
optionally `apps/desktop/src/features/workspace/__tests__/p08_workspace_ui.test.tsx` and a new `__tests__/dnd_helpers.ts` if the P08 helpers are extracted rather than copied (Delegated choices),
`apps/desktop/src/features/settings/__tests__/p16_settings.test.tsx` only if a header query there stops matching (none expected: `Retry checks` and `Save settings` keep their names).

## Core mechanisms (unit tests allowed)
None.

## Ordered tasks
1. **Route parents.** `App.tsx` exports `parentRoute(hash: string): string | null`: `#/` (and `''`) → `null` (root);
   `#/session/<n>` (including `#/session/1`, Unassigned) → `#/`; `#/settings` → `#/`; `#/health` → `#/settings`
   (Health is reached only from Settings today: `SettingsPage.tsx:55`); any other hash → `#/`. Back always sets
   `window.location.hash` to the parent (same navigation the pages use today; no history API).
2. **Shared header.** `features/shell/PageHeader.tsx`: `PageHeader({ back?: { label: string; to: string }; title:
   ReactNode; escapeGoesBack?: boolean; children?: ReactNode })` renders `<header className="app-header">` with, in DOM
   order: the Back control **first** when `back` is given (`Button variant="ghost" className="back-button"`,
   `<ArrowLeft />` + the parent's name as visible text, `aria-label="Back to <label>"`, click → `window.location.hash =
   back.to`), then the title slot (`.header-identity`), then `.header-actions` with `children` on the right. It mounts
   `useBackKeys(back?.to ?? null, escapeGoesBack ?? false)` (task 3). `Header.tsx` keeps its props and renders both
   branches through `PageHeader`: Sessions → no `back`, title `Session Manager` (`h1.app-title`), actions as today;
   workspace → `back={{ label: 'Sessions', to: '#/' }}`, title = identity (type, SKU, ⓘ) or `Unassigned photos` + `No
   product`, actions as today (`escapeGoesBack` false). `HealthPage` → `back={{ label: 'Settings', to: '#/settings' }}`,
   title `Health`, actions: `Retry checks` only (the `Sessions` and `Settings` buttons are removed: Settings is the
   parent), `escapeGoesBack`. `SettingsPage` → `back={{ label: 'Sessions', to: '#/' }}`, title `Settings`, actions:
   `Health` (outline, → `#/health`; a child, not the parent), `escapeGoesBack`; its `Sessions` button is removed. The
   header sits at the top of the window above the existing `.health-shell` column; `.health-header`, `.health-title`
   and `.health-subtitle` lose their callers (remove them or keep the title classes for the `h1`). No other routed page
   exists (`App.tsx:15`); the notifications drawer, popovers and dialogs are layers, not pages.
3. **Back keys.** `lib/keyboard.ts` exports `useBackKeys(parent: string | null, escape: boolean)`: a `window`
   `keydown` listener (same pattern as `useKeyboard`) that, when `parent !== null` and **no layer is open** (no element
   with `role="dialog"`, `role="alertdialog"` or `role="menu"` in the document), on `key === '['` with ⌘ or Ctrl calls
   `preventDefault()` and sets `window.location.hash = parent`; when `escape` is true it does the same on `Escape`
   unless `isTyping(event.target)`. Nothing else changes: `useWorkspaceKeyboard` keeps Escape (item card → selection)
   and ignores ⌘[ already (`useKeyboard.ts:35`); `useKeyboard` ignores ⌘ presses on letters.
4. **Drop settles into the new slot (R8).** In `useWorkspaceState`, `run` takes an optional `optimistic: (photos:
   Photo[]) => Photo[]`; when given it is applied with `setView` **synchronously, before** `command()` is awaited, so
   the tiles re-render in the new order in the same React commit in which dnd-kit clears `active` (its drop animation
   then measures the dragged tile at its new slot — see Notes). `reload()` on success and on failure replaces the
   optimistic list with the stored one (failure already toasts `refusalText(error)`). `reorder(photoId, position)`
   passes the transform "remove the photo from its group's display order (`displayOrder`), insert it at `position − 1`,
   renumber that group's `position` 1..n"; `classify(photoId, target, advance = false)` (the drag path) passes "set
   `classification = target`, `position = <target group length> + 1`, renumber the source group" (ch06 §Order: a
   photo moving between groups goes to the end of the target). Review-key classifies (`advance = true`), delete, undo,
   move and copy are unchanged (Delegated choices). `DragOverlay` keeps dnd-kit's default drop animation; set
   `dropAnimation={null}` **only** if the manual try-out (step 3) still shows a return animation after task 4 — record
   that in the Review log. The folded strip (`FoldedStrip` → `PhotoGrid variant="strip"`) shares the `DndContext`,
   the overlay and `useDrop`, so it is covered by the same change.
5. Tests (§6.9 layout), `npm run typecheck`, `npm test`; the Rust suite is untouched.

## Criteria (3 criteria, 9 rows; 9 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `<HealthPage>` (vitest, Testing Library; P16 fixtures: `getHealth`, `getAppPaths` resolved) with `window.location.hash = '#/health'` | click the header's first button | before the click: the first `button` inside the `banner` (`<header>`) has accessible name `Back to Settings`; the header contains `Retry checks` and no button named `Sessions` or `Settings`; after the click: `window.location.hash === '#/settings'` | `[rule]` real run R7 (Back top-left on every non-root page; parent = Settings); COMPONENTS.md "Header bar" |
| C1(b) | `<SettingsPage>` (P16 fixture `getSettings` resolved) at `#/settings` | click the first header button | before: first header button named `Back to Sessions`; the header contains a `Health` button and exactly one button whose name mentions `Sessions` (the Back control); after: `window.location.hash === '#/'` | `[rule]` R7; ch12 §Settings, §Health |
| C1(c) | `<App>` (P07/P08 mocks) opened at `#/session/5` (P08 `view([a, b])`, item `Oak chair`) and, in a second render, at `#/session/1` (`view([], 1)`, Unassigned) | click the first header button, each render | in each render: the first header button is named `Back to Sessions` (Unassigned shows the title `Unassigned photos`); after the click: `window.location.hash === '#/'` and the Sessions page renders (`New session` button present, no `.photo-panel`) | `[rule]` R7; ch06 §Unassigned |
| C1(d) | `<App>` at `#/` (`listSessions` fixture with one row) | — | the header has no button whose name starts with `Back`; the header's first element is the title `Session Manager` (`h1`); `New session` is present | `[rule]` R7 (Sessions list is the root) |
| C2(a) | `<SettingsPage>` at `#/settings` and `<HealthPage>` at `#/health`, each in its own render | per page: (1) `keydown` `[` with `metaKey` on `window`; fresh render; (2) `keydown` `Escape` on `window`; fresh render (Settings only); (3) `Escape` dispatched on the focused `Client secret` input | Settings: (1) → `#/`, (2) → `#/`, (3) → hash stays `#/settings`; Health: (1) → `#/settings`, (2) → `#/settings` | `[rule]` R7 keyboard (delegated choice: Esc/⌘[); design handoff §Keyboard |
| C2(b) | `<App>` at `#/session/1` (Unassigned, P08 `view([], 1)`; `searchItems`/`searchItemsShop` resolve `[]`) | (1) `Escape` on `window`; (2) click `Assign product` (opens the item picker, `role="dialog"`), then `[`+meta; (3) `Escape` (closes the dialog), then `[`+meta | (1) hash stays `#/session/1`; (2) dialog present and hash stays `#/session/1`; (3) no dialog and `window.location.hash === '#/'` | `[rule]` R7 (⌘[ on every non-root page; Esc keeps "close the topmost layer" in the workspace); design handoff §Keyboard |
| C3(a) | `<App>` workspace with Main `a(1, pos 1)`, `b(2, pos 2)`, `c(5, pos 3)`; P08 C4(b) `layout` and `drag` | drag tile 1 to `[50, 122]` (tile 5's slot); `reorderPhoto` returns a promise the test resolves later; `getSessionView` then answers the stored view `b(1), c(2), a(3)` | synchronously after the release, before the promise resolves: `panelOrder(Main region)` = `[2, 5, 1]` and `reorderPhoto` called once with `(1, 3)`; after resolve + reload: `panelOrder` still `[2, 5, 1]` | `[rule]` real run R8 (the drop animation measures the dragged tile's slot at drop time, Notes); ch06 §Order |
| C3(b) | same fixture | same drag; `reorderPhoto` rejects with `Error('nope')`; `getSessionView` keeps answering the original view | after the rejection settles: `panelOrder(Main)` = `[1, 2, 5]` again and `toast.error` called once | `[rule]` R8; ch06 §Order (the screen never keeps an order the store refused) |
| C3(c) | workspace with Main `a(1)`, `b(2)` and Condition `d(4, pos 1)`; P08 C4(c) `layout` | drag tile 4 to `[50, 37]` (Main's first slot); `classifyPhoto` pending | synchronously after the release: Main region order `[1, 2, 4]`, Condition region empty; `classifyPhoto` called once with `(4, 'main')`; `reorderPhoto` not called | `[rule]` R8; ch06 §Order (a photo moving between groups goes to the end of the target) |

## Delegated choices
Back control wording and icon (parent's name after `←`, or the word `Back`; the `aria-label` stays `Back to <parent>`),
whether Health and Settings keep the 900 px `.health-shell` column under the full-width header, the `Health` action's
exact button variant, whether Ctrl+[ joins ⌘[ (the hook accepts both), Escape leaving Settings with unsaved field
edits (values are saved only by `Save settings`), whether the optimistic transform also serves review-key classifies
(A/C) and delete, whether the P08 test helpers are extracted into a shared `__tests__/dnd_helpers.ts` (two callers)
or copied into the P19 test, and the exact selector set that counts as an open layer (`dialog`, `alertdialog`,
`menu` at minimum).

## Manual try-out
1. Open Settings from the gear: the header shows `← Sessions` at the top left and `Settings` beside it; `Health` is
   on the right. Click `← Sessions`: the list. Open Settings again and press ⌘[ : the list. Again, press Escape: the list.
2. Settings → `Health`: the header shows `← Settings` top left; no `Sessions`/`Settings` buttons on the right, only
   `Retry checks`. ⌘[ returns to Settings.
3. Open a session with three or more Main photos. Drag the first photo onto the third slot and release: the preview
   settles into the third slot (no flight back to the first) and the numbers renumber at once. Drag a Main photo onto
   Condition: it settles at the end of Condition.
4. In the session, press Escape: nothing navigates (selection clears). Press ⌘[ : back to the list. On the list, the
   header has no back control.

## Notes
- **Why R8 happens (grounding, `apps/desktop/node_modules/@dnd-kit/core/dist/core.esm.js`, v6.3.1):** when `active`
  clears, `AnimationManager` (`:3578–3607`) runs `useDropAnimation` (`:3755–3800`), which measures the dragged
  sortable node with the default transform-agnostic measure (`defaultMeasuringConfiguration`, `:2478–2490`) and
  animates the overlay to that rect (`createDefaultDropAnimation`, `:3815–3883`). Today `onDragEnd` → `drop` →
  `state.reorder` → `run` awaits `reorderPhoto` and only then `reload()`s (`useWorkspaceState.ts:124–139, 182`), so at
  measurement time the tile is still at its old index: the overlay flies back, then the grid jumps. Task 4 moves the
  order change into the drop's own commit; the animation then targets the new slot. In jsdom the animation is inert
  (zero rects / no `Element.animate`), so the rows assert what the animation reads: the DOM order right after release.
- **Other dnd-kit sites inspected:** `form/CustomizeMode.tsx:122–127` reorders with `arrayMove` into local state
  synchronously and has no `DragOverlay` (the row itself is the preview) — no return animation is possible; no change.
  `workspace/SessionsPanel.tsx` (`useDroppable` targets for move) and `PhotoGrid.tsx` (group droppables) carry no
  preview of their own. The folded strip is covered by task 4 (shared context).
- Escape as Back is **not** in the handoff (§Keyboard: "Esc — Anywhere — Close the topmost layer"); it is applied only
  where no layer exists (Settings, Health) and recorded as a delegated choice; ⌘[ is the macOS convention (Finder,
  Safari) and the book is silent on it.
- Estimated size: ~10 files, ~350 lines; no backend or bindings change.

## Review log
