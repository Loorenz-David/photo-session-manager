---
phase: P22_frontend_tidy_up
title: Frontend tidy-up — readable publish times, safe typing in Settings, steady drops, form fixes, one source for a page's parent
difficulty: high
projection: no
depends_on: [P21_backend_robustness]
---
# P22 — Frontend tidy-up: readable publish times, safe typing in Settings, steady drops, form fixes, one source for a page's parent

## Goal
Close the frontend leftovers of `docs/implementation/backlog.md` (owner 2026-10-29: "let's correct the known small
leftovers"). The notifications drawer shows a local, readable date and time instead of a 13-digit number. In Settings
the client-secret field is cleared when focused (typing replaces the mask) and ⌘[ never navigates away while the
photographer is typing. In the workspace a reload triggered by an event while a drag command is in flight no longer
makes the dropped tile jump back, and the gliding copy already shows the number and Cover badge of the slot it will
land in (owner: "one component, no pop"). In the form, two quick AI actions on different fields are both carried out
(and a failed one reloads the form), an invalid stored entry is seeded into the input so it can be corrected in
place, closing with Escape right after editing the Title refreshes the workspace card, and the barcode can be cleared
from the header. One source of truth for a page's parent (the dead `parentRoute`/`goHome`/`.health-subtitle` go).
**Not in this phase:** backend changes (all in P21), redesign of Settings/Health, the one-frame `shop unavailable`
flicker as a tested row (fixed in passing, see task 2), anything post-MVP.

## Read first
`docs/implementation/backlog.md` rows cited per task (line numbers as of 2026-10-29: 82, 84, 86, 101/118, 132, 141,
145, 146, 147, 148, 150) · `docs/implementation/plans/P21_backend_robustness.md` tasks 3, 8, 9 (the backend halves
this phase completes: exact-mask secret, `Published` text with `created_at`, `barcode: ""` clears) · master plan §6.1
(`apps/desktop` layout), §6.7 (events), §6.9 (frontend tests), §7.2 (commands) · book `01`, `14`, `04` S-B/S-G,
`05` §Item (barcode), ch05 title has one home (P11 F1), `06` §Order (reorder by drag; a photo moving between groups
goes to the end of the target; 01 = cover), `07` §Publish steps 5 ("Published · time"), `08` §AI pill actions
(Looks right / Clear / Accept all), `12` §Settings, §Health screen/header (drawer) · evidence `design_handoff.md`
§Keyboard, §Review (undo), §AI pill · `docs/implementation/plans/P19_navigation_and_drop.md` (C2(a) Esc/⌘[ rows,
C3 rows, task 4 optimistic order) and `P20_drop_continuity.md` Review log (why the overlay shows the pre-drop number:
dnd-kit queues `DragEnd` before `onDragEnd`, so the `flushSync` there is inert) · code: `apps/desktop/src/features/
publish/NotificationsDrawer.tsx`, `apps/desktop/src/lib/format.ts` (`activityLabel`, the Intl formatters),
`apps/desktop/src/features/settings/SettingsPage.tsx` (`shop()` renders the secret `Input type="password"`),
`apps/desktop/src/lib/keyboard.ts` (`isTyping` `:4`, `useBackKeys` `:33–47`: `commandBack` ignores typing),
`apps/desktop/src/features/workspace/useWorkspaceState.ts` (`reload`, the `onWorkspaceEvents` effect, `run` with
`optimistic`, `reorder`, `classify`), `apps/desktop/src/features/workspace/WorkspacePage.tsx` (`useDrop`, `onDragEnd`
with `flushSync`, `draggedNumber`, `DragOverlay` → `PhotoTileFace`), `apps/desktop/src/features/workspace/PhotoTile.tsx`
(`PhotoTileFace`: `tile-number`, `tile-cover` when `main && number === 1`), `apps/desktop/src/features/form/
FormDialog.tsx` (one shared `act = useAction(...)` for Looks right / Clear / Accept all; `saveAndClose`, `blurField`,
the X button calls `close` directly), `apps/desktop/src/features/form/useFormState.ts` (`storedValue` hides invalid
values, `valueOf`), `apps/desktop/src/features/form/MetafieldInput.tsx` (`verbatim`, the Invalid marker),
`apps/desktop/src/features/form/inputs/{DecimalInput.tsx, IntInput.tsx, TextInput.tsx}` (text boxes accept a string
value), `apps/desktop/src/features/form/IdentityEditor.tsx` (`save`: `""` becomes `null`), `apps/desktop/src/features/
shell/PageHeader.tsx`, `Header.tsx` (`goHome` `:21`), `apps/desktop/src/App.tsx` (`parentRoute` `:25`, `goHome` `:117`),
`apps/desktop/src/styles/globals.css:50` (`.health-subtitle`), `apps/desktop/src/features/sessions/NewSessionDialog.tsx
:105–125, :167` and `workspace/PickerDialogs.tsx:80–81, :112–119, :133` (shop heading state) · tests to reuse:
`features/workspace/__tests__/p19_drop.test.tsx` (mocks incl. `delivered.workspace`, `openWorkspace`, `layout`,
`releaseAt`, `mainOrder`, `conditionOrder`, `original`, `stored`), `features/form/__tests__/p11_form_ui.test.tsx`
(`fieldElement` `:50`, `def` `:72`, `stored` `:76`, `field` `:92`, `form` `:97`, `renderForm` `:124`; C2(b) `:215`,
C5(b) `:296`), `features/settings/__tests__/p16_settings.test.tsx` (`shown` fixture; mocks), `features/publish/
__tests__/p15_publish_ui.test.tsx:152` (drawer render), `features/shell/__tests__/p19_navigation.test.tsx`.

## Files expected to change
`apps/desktop/src/lib/format.ts` (`dateTimeLabel`), `apps/desktop/src/features/publish/NotificationsDrawer.tsx`,
`apps/desktop/src/features/settings/SettingsPage.tsx` (secret field focus/blur; `back` prop dropped), `apps/desktop/src/lib/keyboard.ts` (`useBackKeys`),
`apps/desktop/src/features/workspace/{useWorkspaceState.ts (deferred reload), WorkspacePage.tsx (projected overlay number/badge; `onDragOver`; `flushSync` removed)}`,
`apps/desktop/src/features/form/{FormDialog.tsx (queued actions; close path), useFormState.ts (`valueOf` seeds an invalid value), IdentityEditor.tsx (`""` when emptied), AiPopover.tsx (only if its close timing needs the queued promise)}`,
`apps/desktop/src/features/sessions/NewSessionDialog.tsx`, `apps/desktop/src/features/workspace/PickerDialogs.tsx` (unavailable heading keyed to its query, task 2),
`apps/desktop/src/features/shell/{PageHeader.tsx (derives the parent), Header.tsx (`goHome`, `back` removed)}`, `apps/desktop/src/features/health/HealthPage.tsx` (`back` removed), `apps/desktop/src/App.tsx` (`parentRoute` + `routeLabel` as the one source; `goHome` removed), `apps/desktop/src/styles/globals.css` (`.health-subtitle` removed),
`apps/desktop/src/features/shell/__tests__/p22_shell_tidy.test.tsx` (new: C1, C2, C5), `apps/desktop/src/features/workspace/__tests__/p22_workspace_tidy.test.tsx` (new: C3), `apps/desktop/src/features/form/__tests__/p22_form_tidy.test.tsx` (new: C4),
`apps/desktop/src/features/workspace/__tests__/p09_export_action.test.tsx:31` (drops the `goHome` prop), `apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx` only if a query stops matching (none expected: labels unchanged).

## Core mechanisms (unit tests allowed)
None.

## Ordered tasks
1. **Readable publish time (backlog 101/118 frontend half).** `lib/format.ts` exports `dateTimeLabel(ms)` =
   `new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(ms)` (local zone, the
   webview's locale). `NotificationsDrawer` renders each row as `<p>{text} · <time dateTime={new Date(created_at)
   .toISOString()}>{dateTimeLabel(created_at)}</time></p>`; a `text` that still ends in ` · ` followed by 13 digits
   (rows written before P21) is shown without that suffix. Nothing else in the drawer changes (Open, Retry).
2. **Settings typing (backlog 132 frontend half, 148; 145 in passing).** The `Client secret` input: on focus, when its
   value is the masked value delivered by `settings_get` (starts with `••••` and equals what was loaded), the field is
   emptied so typing replaces the mask; on blur, when it is still empty, the masked value is restored (so Save sends
   the exact mask and P21 keeps the stored secret). `useBackKeys`: ⌘[ / Ctrl+[ is ignored while `isTyping(event.target)`,
   like Escape already is. In passing, the two dialogs key their `unavailable` state to the query that failed
   (`shopFailedFor`), so the heading `shop unavailable` is derived from `shopFailedFor === currentQuery` and a new
   query never shows it for a frame (backlog 145; no row: not observable after an `act` flush — recorded here).
3. **Steady drops (backlog 147, 150).** `useWorkspaceState`: a counter of commands in flight (`run` increments before
   `command()` and decrements after its own `reload`); the `onWorkspaceEvents` handler, while the counter is above
   zero, does **not** call `reload()` but marks a reload as wanted; `run`'s closing `reload()` serves it (one reload,
   after the command settles; `photo_added` still sets the active photo at once). `WorkspacePage`: `onDragOver` keeps
   `over` (id) in state; the overlay's number and Cover badge come from the **projected** slot: over a tile of the
   same group → that tile's index + 1 in the group's display order (what `useDrop` will pass to `reorder`); over a
   tile or group droppable of the **other** group → that group's length + 1 (end of target, ch06 §Order); over the
   photo's own tile, a session row, or nothing → the photo's current number. Cover = projected group `main` and
   projected number 1. `PhotoTileFace` is unchanged (it already renders from `number`). Remove the inert `flushSync`
   call and its comment from `onDragEnd` (P20 review note: dnd-kit dispatches `DragEnd` before `onDragEnd`); keep the
   P19/P20 behaviour (optimistic order, 200 ms translate glide, hidden source) intact.
4. **Form fixes (backlog 141, 82, 86, 84 frontend half).** (a) `FormDialog`: Looks right / Clear / Accept all run
   through one **queue** (each action starts when the previous one settles; none is dropped); a failed action toasts
   `<failure>: <message>` **and** reloads the form (`state.reload()`), a successful one reloads as today. (b)
   `useFormState.valueOf`: when the stored value is invalid (`valid = false`) and the field's kind renders a text box
   (`text`, `textarea`, `url`, `int`, `decimal`), the input is seeded with the verbatim text (`MetafieldInput`'s
   `verbatim` rule: a JSON string without quotes, else the raw text); the Invalid marker stays until the photographer
   edits; other kinds keep today's behaviour (empty input + marker). (c) Closing the form by **any** path (Escape,
   click outside, the X button, Save & close) runs the same close sequence: flush pending writes, call `onItemChanged`
   when the Title was edited since the form opened (`isEdited(PRODUCT_TITLE)` before the flush, or a Title write that
   already happened without a blur), then `onOpenChange(false)`. (d) `IdentityEditor.save`: the barcode argument is
   `null` when unchanged and the trimmed text otherwise — including `""` when the field was emptied (P21 stores
   `NULL`); the SKU rule is unchanged.
5. **One source for a page's parent (backlog 146).** `App.tsx` keeps `parentRoute(hash)` and gains `routeLabel(hash)`
   (`#/` → `Sessions`, `#/settings` → `Settings`, `#/health` → `Health`, `#/session/<n>` → the workspace); `PageHeader`
   loses its `back` prop and derives Back from `parentRoute(window.location.hash)` (label `routeLabel(parent)`,
   `aria-label` still `Back to <label>`; root → no Back control). `Header`, `HealthPage`, `SettingsPage` stop passing
   `back`; `HeaderProps.goHome` and `App`'s `goHome={…}` go; `.health-subtitle` goes. P19's visible behaviour and
   aria-labels are unchanged.
6. Tests (§6.9), `npm run typecheck`, `npm test`; the Rust suite is untouched.

## Criteria (5 criteria, 10 rows; 8 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `<NotificationsDrawer>` (p15 render shape) | two rows: `{kind:'published', text:'Published', created_at: 1761350400000}` and a legacy `{kind:'published', text:'Published · 1761350400000', created_at: 1761350400000}` | each row's paragraph text is `Published · ` + `new Intl.DateTimeFormat(undefined, {dateStyle:'medium', timeStyle:'short'}).format(1761350400000)`; each row has a `<time>` with `dateTime === '2025-10-25T00:00:00.000Z'`; no row's text matches `/\d{13}/` | `[rule]` ch07 §Publish steps 5 ("Published · time"); ch12 §Health screen/header (drawer) |
| C2(a) | `<SettingsPage>` with the p16 `shown` fixture (secret `••••1234`) | (1) focus `Client secret`, blur without typing, click `Save settings`; (2) fresh render: focus, type `NEW-SECRET-77`, click `Save settings` | (1) on focus the input's value is `''`; after blur it is `••••1234`; the save patch has `integration.client_secret === '••••1234'`; (2) the patch has `integration.client_secret === 'NEW-SECRET-77'` | M30 / 09A MC-23 (a typed secret is never silently dropped; the exact mask keeps the stored one — P21 C2(b)) |
| C2(b) | `<SettingsPage>` at `#/settings` | (1) `keydown` `[` with `metaKey` dispatched on the focused `Client secret` input; (2) the same on `window` | (1) `window.location.hash` stays `#/settings`; (2) `#/` | `[rule]` P19 R7 keyboard (⌘[ as Back; never while typing — companion of P19 C2(a)(3)) |
| C3(a) | `<App>` workspace, P19 C3(a) fixture (`original` → Main `1, 2, 5`; `layout`; `reorderPhoto` pending) | drag tile 1 to `[50, 122]`; while pending, deliver `{kind:'photo_changed', photo_id: 2, session_id: 5}` through `delivered.workspace` with `getSessionView` answering `original`; then set `getSessionView` to answer `stored` and resolve `reorderPhoto` | after the event and before the resolve: `mainOrder() === [2, 5, 1]` (no jump back); after the resolve: `mainOrder() === [2, 5, 1]` and `getSessionView` was called at least once more after the resolve (the wanted reload ran) | `[rule]` R8 / ch06 §Order (the screen never shows an order older than the one the photographer just made) |
| C3(b) | `<App>` workspace with Main `a(1, pos 1)`, `b(2, pos 2)`, `c(5, pos 3)` and Condition `d(4, pos 1)`; `layout` `1:[0,0]`, `2:[110,0]`, `5:[0,85]`, `4:[0,300]` | pointerDown on tile 1, move 10 px, move to `[50, 122]` (tile 5's slot) — **not released**; then move to `[50, 337]` (tile 4's slot); then move to `[50, 37]` (its own slot); release; second drag: tile 5 to `[50, 37]`, not released | over tile 5: `.drag-overlay .tile-number` reads `03` and the overlay has no `.tile-cover`; over tile 4: `02`, no Cover; over its own slot: `01` with `.tile-cover`; after the release on its own slot `reorderPhoto` and `classifyPhoto` are not called; second drag over tile 1's slot: overlay reads `01` with `.tile-cover` | `[rule]` owner 2026-10-29 ("one component, no pop"); P20 Review log note; ch06 §Order (01 = cover) |
| C4(a) | `<FormDialog>` (p11 `renderForm`) with two text fields `custom.era` (stored `1960s`) and `custom.maker` (stored `Oak & Co`), both with provenance `ai` (p11 C5(b) `ai` shape); `setAttributeValue` returns pending promises `pA` (era) then `pB` (maker) | open era's AI pill → `Looks right`; at once open maker's pill → `Looks right`; resolve `pA`; reject `pB` with `Error('nope')` | after both clicks and before `pA` resolves: `setAttributeValue` called once (era); after `pA` resolves: called twice, the second with `(ITEM_ID, 'custom.maker', 'Oak & Co')` (maker's stored value); after `pB` rejects: `toast.error` called once with a message containing `nope`, and `getForm` was called again after the rejection | `[rule]` ch08 §AI pill (Looks right accepts the value as a human edit; no click is lost); design handoff §AI pill |
| C4(b) | `<FormDialog>` with a `decimal` field `custom.price` stored `"12,5"`, `valid: false` | render; change the input to `12.5`; blur | before the edit: the input's value is `12,5` and the field shows the Invalid marker with `12,5`; after the blur: `setAttributeValue(ITEM_ID, 'custom.price', '12.5')` called once and the marker is gone after the re-fetch | M16 / 05A MC-9 (an invalid value is shown verbatim and not written back until edited; now correctable in place) |
| C4(c) | `<FormDialog>` with `product.title` stored `Oak chair`; `onOpenChange` and `onItemChanged` are spies | type ` deluxe` at the end of the Title (focused) and press Escape immediately (no blur) | `setAttributeValue(ITEM_ID, 'product.title', 'Oak chair deluxe')` called once; `onItemChanged` called once; `onOpenChange(false)` called once, **after** `onItemChanged` | `[rule]` ch05 (the product title has one home; P11 F1: list, header, card and form agree) |
| C4(d) | `<IdentityEditor item={…barcode:'4006381333931'} onSaved={spy} />`; `updateIdentity` resolves the item with `barcode: null` | click `Edit SKU and barcode`, clear the `Barcode` input, submit | `updateIdentity` called once with `(ITEM_ID, null, '')`; `onSaved` called once with the returned item (`barcode === null`); the popover closes | `[rule]` ch05 §Item (barcode optional and removable); P21 C6(a) |
| C5(a) | `<PageHeader title={<h1>X</h1>} />` (no `back` prop exists any more) rendered at `#/settings`, `#/health`, `#/session/5` and `#/` in four renders | click the first header button where present | `#/settings`: first button `Back to Sessions`, click → `#/`; `#/health`: `Back to Settings`, click → `#/settings`; `#/session/5`: `Back to Sessions`; `#/`: no button whose name starts with `Back` | `[rule]` R7 (one source of truth for a page's parent: P19 C1 rows keep passing) |

## Delegated choices
The date style (`dateStyle: 'medium', timeStyle: 'short'` is the plan's default; an `activityLabel`-like relative form
is acceptable if the full date is still shown for days before today), whether the legacy-suffix strip is kept after
the owner's database is replaced, where `routeLabel` lives (`App.tsx` or a new `lib/routes.ts`), whether the secret
field also selects-all instead of clearing on focus (the blur restore is required either way), the queue primitive
(a promise chain in a ref is enough), whether `Clear` and `Accept all` close their popover before or after their
queued action settles, the projected number when the pointer is over a session row (unchanged number), and the
in-flight counter's name.

## Manual try-out
1. Bell → notifications: every published entry reads like "Published · 29 Oct 2026, 14:03"; no long number anywhere,
   including entries from before this phase.
2. Settings: click into Client secret: it empties; click elsewhere: the dots come back. Click in and type the real
   secret, Save: Health shows the Shopify token green. While typing in any Settings field press ⌘[ : nothing happens;
   click outside the fields and press ⌘[ : back to Sessions.
3. Open a session with three or more Main photos and one Condition photo. Drag photo 01 over slot 3 and hold: the
   gliding copy already reads 03 without Cover; release: it lands identical, no flip. Drag 03 over slot 1 and hold:
   it reads 01 with Cover. Drag a Main photo over Condition and hold: it reads the next Condition number. Shoot or
   import while dragging and dropping: the dropped tile never jumps back.
4. Form with AI suggestions on two fields: click "Looks right" on one and immediately on the other: both are
   accepted. Type `12,5` in a decimal field and leave it: the input keeps `12,5` with the Invalid marker; change it to
   `12.5`: saved, marker gone. Edit the Title and press Escape straight away: the workspace card shows the new title.
5. Form header pencil: clear the barcode, Save: the barcode disappears from the header and stays gone after reopening
   the session.
6. Settings and Health: Back controls as before (Settings → Sessions, Health → Settings; ⌘[ and Escape still work).

## Notes
- **Why the overlay flips at landing (P20 review, verified by its reviewer probe):** the glide is a clone of the last
  during-drag render, and `draggedNumber` is computed from the photo's current position, which the optimistic order
  changes only at drop. Showing the projected slot during the drag (task 3) makes the clone and the landed tile
  identical; jsdom renders the overlay (P20 C1(a)), so the rows read its badges during the drag.
- **Why a reload can jump back (reading `useWorkspaceState.ts`):** `run` sets the optimistic order, awaits the
  command, then reloads; an event between those two steps reloads the stored (old) order. Deferring event reloads
  while a command is in flight closes it; the final reload still happens.
- **FormDialog's shared `useAction`:** `run` ignores calls while locked, and `AiPopover` closes itself on click, so the
  second click vanished without a message. A queue keeps every click; per-field locks would also work but need a
  busy state per pill (Delegated choices allow either as long as C4(a) holds as written: two calls, in order).
- **Escape in the form:** Radix `Dialog` calls `onOpenChange(false)` on Escape and outside clicks; today that unmounts
  `FormBody`, whose unmount effect writes pending edits but cannot call `onItemChanged`. Routing every close through
  one sequence (task 4c) fixes Escape, the X button and outside clicks alike.
- **145 (one-frame `shop unavailable`)** is fixed inside task 2 without a row: the pre- and post-fix states are
  identical after React's effects flush, so no jsdom test discriminates them; the reviewer reads the derivation.
- Frontend only; no bindings change. No intention-book change is needed (ch07's "Published · time" is what the drawer
  now shows). Estimated size: ~17 files, ~550 lines (half tests).

## Review log
- 2026-10-29, implement r1 (codex): Notifications now show local publish date/time and strip legacy millisecond suffixes. Settings clears the loaded secret mask on focus and restores it on empty blur; Back ignores focused inputs, and shop failure headings are tied to their query. Workspace event reloads defer during review commands, and the drag overlay shows the projected destination number and Cover badge. Form AI actions queue and reload on failure, invalid text values seed their inputs, every form close flushes Title and refreshes the workspace, and clearing a barcode sends an empty string. PageHeader derives its parent from shared route functions; dead navigation props and CSS removed. Ten boundary tests cover C1(a), C2(a–b), C3(a–b), C4(a–d), C5(a). Rust suite, 124 frontend tests, Clippy, formatting, and TypeScript pass. Assumptions: the route label for `#/session/<n>` is `Workspace`; the full medium local date/time format is used; popovers close before their queued AI action settles; the form close sequence guards repeat dismissal; no live Capture One/drag visual try-out was available. Deviations: added `lib/routes.ts` per delegated choice to avoid a PageHeader/App import cycle; updated the P20 overlay test's obsolete `01` expectation to `03`; corrected C1(a)'s ISO year from 2026 to 2025 because its specified millisecond value is 2025-10-25 UTC. No backend or bindings changes.
- 2026-10-29, review r1 (opus): **CHANGES_REQUESTED.** Checkpoint `a55f974` paths = declared files. Re-run: `cargo test --workspace` 421 passed, clippy and fmt exit 0, `npm run typecheck` exit 0, `npm test` 15 files / 124 tests. All 10 rows hold by reading production and test (each test fails on the pre-P22 code); H1–H8 untouched (frontend only, no secret logged). **F1 (B1, route: production)** `WorkspacePage.tsx` projected overlay number: a Main photo held over an unreviewed tile (shown after Main in the same grid) gets `ordered.main.findIndex(...) + 1 = 0` → overlay reads `00`, while `useDrop` passes `group.length` (index −1) to `reorder` and the tile lands as e.g. `02` (reviewer probe, temporary test deleted: held `00`, `reorderPhoto(1, 2)`). Authority: task 3 ("that tile's index + 1 … (what `useDrop` will pass to `reorder`)"), owner "one component, no pop", ch06 §Order. Correction: mirror `useDrop` (`index === -1 ? group.length : index + 1`) in the projection. Files: `WorkspacePage.tsx`; optionally one hover added to `p22_workspace_tidy.test.tsx`. Notes to backlog: unreviewed photo dropped on its own slot is approved to Main (P19 `useDrop`); a second drag's optimistic order can still be replaced by the first command's closing reload; shop heading `shopDone` no longer checks freshness; implementer edited C1(a)'s ISO literal (correct value, wrong role).
- 2026-10-29, fix r2 (codex): **F1 resolved.** Added one `projectedSlot` helper shared by the drag overlay projection and `useDrop`; an unreviewed tile absent from the ordered Main group now projects to the same end slot `useDrop` sends to reorder. Extended C3(b) with a hover over an unreviewed tile and exact overlay/reorder assertions. No divergence or additional scope. Full Rust/frontend suites, Clippy, formatting and typecheck green (421 Rust, 124 UI tests).
