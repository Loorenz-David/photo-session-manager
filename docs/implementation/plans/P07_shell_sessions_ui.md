---
phase: P07_shell_sessions_ui
title: App shell, sessions list and new session (frontend)
difficulty: normal
projection: no
depends_on: [P03_session_state_list, P04_capture_one_adapter]
---
# P07 — App shell, sessions list and new session (frontend)

## Goal
Build the React shell and the Sessions page from the design handoff: header (back, item type/SKU, camera dot with
retry and error text, bell + failed count placeholder, form and Publish/Export placeholders), the grouped sessions
list with Open / Published / All filters and counts, search, time filter, hover preview, status pills, the
Unassigned card, batch selection bar (visual only until P15), theme toggle, Sonner toasts, keyboard `N`; and the New
session dialog (search; "Create '…'" row creating a local item and opening its session). Also the small backend
`items_search` command; `ItemsService::next_sku()` + `items_next_sku` command (MC-22 proposal rule). **Not in this phase:** the workspace (P08), publish actions (P15), settings/health screens
(P16). Thin frontend phase — **refine at prompt time** from the screens listed below.

## Read first
master plan §6.1 (frontend layout), §6.7 (commands `sessions_list`, `session_create`, `item_create`, `items_search`,
`camera_*`), §6.8 (`sku_prefix`), §6.9 (frontend tests) · book `01`, `14`, `04` S-A, S-B, S-D, S-E (card), `07`
(list rows, Open filter), `02` (vocabulary: never "product" in UI copy except where the design says Shopify) · evidence
`design_handoff.md` §Session status, §Sessions list, §New session, §Keyboard, §Tokens and components · `claude_design/
NOTES.md` §Screens 1–3, §Keyboard; `COMPONENTS.md` §App shell, §Sessions page; screens `desktop/01–10`, `32`, `36`,
`narrow/01`.

## Files expected to change
`apps/desktop/src/{App.tsx (router), features/shell/{Header.tsx, CameraDot.tsx, NotificationsBell.tsx, ThemeToggle.tsx},
features/sessions/{SessionsPage.tsx, SessionRow.tsx, StatusPill.tsx, GroupHeader.tsx, Filters.tsx, TimeFilter.tsx,
HoverPreview.tsx, UnassignedCard.tsx, BatchBar.tsx, NewSessionDialog.tsx, __tests__/p07_shell_sessions_ui.test.tsx},
services/{sessions.ts, items.ts, camera.ts, events.ts}, lib/{keyboard.ts, format.ts}, components/ui/* (shadcn adds)}`,
`crates/core/src/services/items.rs` (`search(q)`), `apps/desktop/src-tauri/src/commands/items.rs` (`items_search`),
bindings, `crates/testkit/tests/p07_shell_sessions_ui.rs` (new: the one backend row C5(c)).

## Core mechanisms (unit tests allowed)
None (pure rendering; the status/label/grouping mechanisms are backend, P03).

## Ordered tasks
1. Shell: router (`/` sessions, `/session/:id` placeholder), header per `COMPONENTS.md` §App shell, Sonner, theme
   toggle (`.dark` on `<html>`, persisted in `settings.theme` via `settings_set` once P16 lands — until then
   `localStorage`), keyboard map scaffold (`N`, `Esc`).
2. Sessions page: `sessions_list` on mount and on `session_changed`/`photo_added` events; groups and rows; filters
   with counts; search (debounced, server-side via the query argument); time filter (Any time / Today / Last 7 days /
   Last 30 days, applied client-side on `last_activity_at`); hover preview; Unassigned card (opens `/session/1`);
   batch checkboxes + bar (actions disabled, "coming with publishing").
3. Camera dot: `camera_status` on mount and on `camera_status_changed`; click → `camera_retry`; error popover text
   from ch04 S-A ("check USB mode, close Imaging Edge").
4. New session dialog (`Dialog` + `Command`): `items_search(q)` (local items + integration matches when configured);
   result rows with thumbnail/note; "Create '…'" row when no exact title match → SKU box pre-filled per 05B
   MC-22 (`<sku_prefix>-<NNNN>` from `items_next_sku()`, editable; empty and required when no prefix; "<SKU> is
   already in the shop" disables Create when the integration finds it) → `item_create` → `session_create(item_id)` → navigate.
5. Backend: `ItemsService::search(q)` (local items by SKU/title substring, plus `CommerceAdapter::item_search` when
   `settings.integration` is set — the adapter arrives in P12; until then local only), `items_search` command.
6. Tests (vitest, services mocked) for each row.

## Criteria (6 criteria, 19 rows; 10 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `SessionsPage` render | `sessions_list` mock → groups `[Today: [A, B], Yesterday: [C]]` | DOM has headers "Today", "Yesterday" in that order; A and B under Today, C under Yesterday | M11 (grouping rendered as delivered) |
| C2(a–f) | `StatusPill` | label `Publishing(42)` / `PublishFailed` / `DescriptionFailed` / `Published` / `Ready` / `InProgress(3)` (6 rows) | text "Publishing 42%" + progress bar / "Publish failed" + Retry / "Description failed" / "Published" / "Ready · Publish" (button) / "In progress" + "3 to review" | M14 |
| C3(a) | filter toggle | click "Published" | `sessions_list` called with `filter: "published"`; counts shown equal the mock's `counts` | `[rule]` ch07 §Open filter |
| C3(b) | search input | type "oak" | `sessions_list` called (after debounce) with `query: "oak"` | `[rule]` ch04 S-A |
| C4(a) | `CameraDot` | `camera_status` → `Connected("Sony ZV-E10")` | green dot; title contains the camera name | `[rule]` ch04 S-A |
| C4(b) | `CameraDot` | `Disconnected` | red dot; popover text mentions USB mode and Imaging Edge | `[rule]` ch04 S-A |
| C4(c) | `CameraDot` click | — | `camera_retry` called once | `[rule]` ch04 S-A |
| C5(a) | `NewSessionDialog` | `items_search("Brass floor lamp")` → `[]` | a row "Create 'Brass floor lamp'" is shown | `[rule]` ch04 S-D |
| C5(b) | select the create row after replacing the proposal with `BFL-1` | — | `item_create({sku:"BFL-1", title:"Brass floor lamp"})` then `session_create(item_id)` called; navigation to `/session/<id>` | `[rule]` ch04 S-D |
| C5(e) | backend `ItemsService::next_sku()` (Rust test) | prefix `VF`; local SKUs `VF-0419`, `VF-0420`, `vf-0999`, `VF-12a` / prefix absent | `"VF-0421"` / `None` (box empty, Create disabled until typed) | MC-22 (05B) |
| C5(f) | `NewSessionDialog` | proposal `VF-0421`; `items_search` reports a shop product with SKU `VF-0421` | text "VF-0421 is already in the shop"; Create disabled | MC-22 (05B) |
| C5(c) | backend `ItemsService::search("oak")` (Rust test) | local items "Oak sideboard" (sku OAK-1), "Teak desk"; no integration | returns the Oak item only, `origin = local` | `[rule]` ch04 S-B |
| C5(d) | select an existing result | item with a session | `session_create(item_id)` returns the existing session id (no second session); navigation to it | `[rule]` ch02 (one Session per item, R6) — see Notes |
| C6(a) | `UnassignedCard` | `unassigned_count = 0` / `= 3` | not rendered / rendered with "3" | `[rule]` ch04 S-E |

## Delegated choices
Routing library (React Router or a tiny hash router), debounce interval, hover-preview flip logic, time-filter
boundaries (calendar-day based like MC-4), exact copy of the camera error text, how the theme is persisted before P16.

## Manual try-out
1. Launch the app: the Sessions page shows "No sessions yet" (or your sessions), the camera dot is green when the
   ZV-E10 is tethered in Capture One; unplug → red; click → retries.
2. Press `N`, type "Brass floor lamp" → "Create 'Brass floor lamp'" → keep or edit the proposed SKU (set a prefix in Settings first, or type one) → the (empty) workspace route opens.
3. Back to Sessions: the row shows "In progress"; the Open / Published / All counts read 1 / 0 / 1.

## Notes
- **Planner assumption (not an owner card):** the MVP keeps **one session per item** (`session_create(item_id)` returns
  the existing session). R6 defines a Session as one item's photography job; a second job for the same item is not
  in the scope ladder. The orchestrator can lift this by amending P07 and `SessionsService::create`.
- `sku_prefix`: owner card 2 answered "generate, editable" (05B MC-22); folded into task 4 and rows C5(b), C5(e), C5(f).
- Estimated size: ~22 files, ~1 900 lines (mostly TSX).

## Review log
(empty)
