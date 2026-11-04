---
phase: P26_frontend_leftovers
title: Frontend leftovers — the app's own session path shown in full, two quick drags never jump back
difficulty: high
projection: no
depends_on: [P25_tidy_up_after_provisioning]
---
# P26 — Frontend leftovers: the app's own session path shown in full, two quick drags never jump back

## Goal
Close the two frontend leftovers of the owner's 2026-11-04 "go" (the P24 narrative's "Not yet"; backlog row 157). In Settings, right
after "Use the app's own session", the detail line shows the same full path the app will use (`default_document_id`, delivered by P25),
not the short form `~/Pictures/Session Manager`. In the workspace, a reload is never issued while any review command is still in flight:
the first of two quick drags no longer replaces the second drag's optimistic order with the stored one when its command settles;
the one reload runs when the last command settles. **Not in this phase:** backend changes (all in P25), the overlay/projection work
(P22 C3(b), unchanged), anything post-MVP. Thin phase: refine at prompt time.

## Read first
`docs/implementation/backlog.md` row 157 (line as of 2026-11-04) · `docs/implementation/narratives/P24_capture_one_provisioning.md` §Not
yet (last bullet) · `docs/implementation/plans/P25_tidy_up_after_provisioning.md` task 3 and C3(a) (`default_document_id` in `settings_get`
and `SettingsDto`) · `docs/implementation/plans/P22_frontend_tidy_up.md` task 3 and C3(a) (the in-flight counter defers **event**
reloads; Review log r1 note: "a second drag's optimistic order can still be replaced by the first command's closing reload") · master
plan §6.1 (`apps/desktop` layout), §6.7 (`settings_get` keys), §6.9 (frontend tests) · book `06` §Order (the screen never shows an order
older than the one the photographer just made — P19 R8), `10C` MC-26 §Setting, `12` §Settings · code (line numbers at `c90b95d`,
2026-11-04): `apps/desktop/src/features/settings/SettingsPage.tsx` (`chooseSession :36–39`; the save patch `:47`; the Capture One block
`:88–97` — `:92` renders `settings.document_id || (page.data?.document_id ? '~/Pictures/Session Manager' : settings.effective_document_id)`),
`apps/desktop/src/services/settings.ts` (`Settings = SettingsDto_Serialize`), `apps/desktop/src/services/bindings.ts` (`SettingsDto_Serialize`
with `default_document_id` after P25), `apps/desktop/src/features/workspace/useWorkspaceState.ts` (`commandsInFlight :67`, `reloadWanted :68`,
the event handler `:103–109`, `run :127–148` — `finally` awaits `reload()` **unconditionally** at `:140` and only then decrements `:142`) ·
tests to reuse: `apps/desktop/src/features/settings/__tests__/p24_capture_one_provisioning.test.tsx` (`base :14–19`, C5(a) `:21–50`: the
render shape, `pickSessionFile`/`saveSettings` mocks, the `Effective path:` text), `apps/desktop/src/features/workspace/__tests__/
p22_workspace_tidy.test.tsx` (mocks and polyfills `:10–105`; `openWorkspace :138`, `layout :152`, `releaseAt :194`, `mainOrder :204`,
`original :206`, `stored :207`; C3(a) `:209–225` — the pending-`reorderPhoto` pattern and `getSessionView` call counting).

## Files expected to change
`apps/desktop/src/features/settings/SettingsPage.tsx` (the detail line reads `default_document_id`), `apps/desktop/src/features/workspace/
useWorkspaceState.ts` (`run` defers its own reload), `apps/desktop/src/features/settings/__tests__/p26_frontend_leftovers.test.tsx` (new:
C1(a)), `apps/desktop/src/features/workspace/__tests__/p26_double_drag.test.tsx` (new: C2(a)). No bindings change (P25 regenerated them).

## Core mechanisms (unit tests allowed)
None.

## Ordered tasks
1. **Full path after "Use the app's own session" (P24 narrative).** `SettingsPage`, Capture One block: the detail line is
   `Effective path: {settings.document_id || settings.default_document_id}` — the folder the photographer picked, else the app's own
   document path as the backend reports it. The `~/Pictures/Session Manager` literal and the `page.data?.document_id` branch go. The
   read-only name line (`Session Manager (created by the app)` / the picked folder), the two buttons and the save patch (`document_id:
   settings.document_id ?? ''`) are unchanged (P24 C5(a) keeps passing).
2. **No reload while any command is in flight (backlog 157).** `useWorkspaceState.run`: in `finally`, decrement `commandsInFlight` **first**;
   if it is now `0`, run `reload()` once (this also serves a `reloadWanted` set by an event — P22 task 3 — and clears it); if commands are
   still in flight, set `reloadWanted = true` and do not reload. The event handler is unchanged (`commandsInFlight > 0` → mark wanted).
   Every other behaviour of `run` (optimistic order applied before the command, `next` applied after, the refusal toast) is unchanged;
   `undo` keeps its own `reload()` (it is not a `run` command).
3. Tests (§6.9), `npm run typecheck`, `npm test`; the Rust suite is untouched.

## Criteria (2 criteria, 2 rows; 2 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `<SettingsPage>` (p24 C5(a) render shape) with `getSettings` → p24 `base` plus `document_id: '/Users/me/Pictures/Other'`, `effective_document_id: '/Users/me/Pictures/Other'`, `default_document_id: '/Users/me/Pictures/Session Manager'`; `pickSessionFile` → `/Users/me/Pictures/Two/Two.cosessiondb`; `saveSettings` not called | click `Use the app's own session`; then click `Use another session…` | after the first click (no save): the texts `Session Manager (created by the app)` and `Effective path: /Users/me/Pictures/Session Manager` are shown and no text contains `~/`; after the second click: `Effective path: /Users/me/Pictures/Two` is shown; `saveSettings` was never called | `[rule]` 10C MC-26 §Setting (the screen names the document the app will use); ch12 §Settings; P24 narrative "Not yet" |
| C2(a) | `<App>` workspace, P22 C3(a) fixture (`original` → Main `1, 2, 5`; `layout` `1:[0,0]`, `2:[110,0]`, `5:[0,85]`); `reorderPhoto` returns pending `p1` for the first call and pending `p2` for the second; `getSessionView` answers `original` | `releaseAt(tile(1), [50, 122])` (tile 1 to slot 3); without resolving, `releaseAt(tile(2), [50, 122])` (tile 2 to tile 5's slot); then `getSessionView` → `original` still, resolve `p1`; then `getSessionView` → `view([1 pos 3, 2 pos 2, 5 pos 1])` (the stored result of both drags), resolve `p2` | after the two drags: `reorderPhoto` was called with `(1, 3)` then `(2, 2)` and `mainOrder() === [5, 2, 1]`; after `p1` resolves: `mainOrder()` is still `[5, 2, 1]` and `getSessionView`'s call count has **not** grown since the drags; after `p2` resolves: `getSessionView` was called exactly once more and `mainOrder() === [5, 2, 1]` (the stored order) | `[rule]` ch06 §Order / P19 R8 (the screen never shows an order older than the one the photographer just made); P22 C3(a) companion (events) — now also a command's own reload |

## Delegated choices
Whether `reloadWanted` survives as a flag or the last-settling command always reloads (the row holds either way: one reload after
`p2`); the exact pointer path of the second drag if dnd-kit needs an extra move (the drop target is tile 5's rect; the outcome
`reorderPhoto(2, 2)` is what is fixed); the new test files' names.

## Manual try-out
1. Settings → "Use another session…", pick `~/Pictures/session test 1/session test 1.cosessiondb`, then press "Use the app's own session"
   **without saving**: the detail line reads `Effective path: /Users/<you>/Pictures/Session Manager` (the full path), identical to what
   it shows after Save.
2. Open a session with four or more Main photos. Drag photo 01 to the last slot and, before it has settled, drag the new first photo to
   the second slot: both land where you dropped them and neither jumps back; after a second the order is the one you made.

## Notes
- **Why the first command's reload can replace the second drag (reading `useWorkspaceState.ts:138–144`):** `run`'s `finally` awaits
  `reload()` while the counter still counts the command itself, so a second command in flight is not seen; the reload then paints the
  stored order, which lacks the second drag. Decrementing first and reloading only at zero makes the last command the one that reloads.
- **Why the detail line needs a backend key (P25 task 3):** while `document_id` is set, `effective_document_id` equals it; the frontend has
  no `$HOME`; only the backend can name the app's own document before a save.
- Frontend only; no bindings change (P25 regenerated them with `default_document_id`). Estimated size: 4 files, ~150 lines (mostly tests).

## Review log
