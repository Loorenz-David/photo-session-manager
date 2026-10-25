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
- 2026-10-25 21:06 (orchestrator): implement r1 stopped by the Codex usage limit (resets 2026-10-26 01:00); attempt archived to `archive/P07_shell_sessions_ui_20261025-210645`. Partial work is left uncommitted in the tree (shadcn ui components, `features/sessions`, `features/shell`, services, `crates/testkit/tests/p07_shell_sessions_ui.rs`, edits to items/sessions services and commands). The next implementer continues from it: verify each piece against this plan, keep what fits, replace what does not.
- 2026-10-25 implement r1 (resumed, actor `codex` per prompt; run on Claude Opus): **IMPLEMENTED.** Built the React shell
  (hash router `#/`, `#/session/:id`, header for both, camera dot with retry and help text, bell placeholder, theme toggle,
  Sonner, `N`/`Esc`), the Sessions page (groups as delivered, Open/Published/All with counts, debounced server-side search,
  client-side time filter, hover preview, status pills, Unassigned card, batch selection + disabled "Review & publish"),
  the New session dialog (`Dialog` + `Command`; existing item → `session_create`; "Create “…”" row → SKU box pre-filled
  from `items_next_sku`, checked with `items_search` before Create enables → `item_create` → `session_create` → navigate),
  and the backend `ItemsService::search`, `ItemsService::next_sku(prefix)`, `items_search`, `items_next_sku`, plus
  `SessionsService::create` returning an item's existing session. Kept from the stopped attempt: the backend pieces
  (reviewed against MC-22, `next_sku` rewritten for clippy) and the shadcn components; the frontend was rewritten.
  Judgment calls: hash routing (bundled file URLs); debounce 250 ms (list), 200 ms (item search), 250 ms (SKU check);
  time filter calendar-day based; theme in `localStorage` until P16; camera help text in `CameraDot.CAMERA_HELP`;
  `next_sku` takes the prefix as an argument (core cannot read the settings file; the command passes `sku_prefix`);
  a search hit with `origin` ≠ `local` counts as "in the shop" (P12 fixes the shop-hit shape); a local SKU clash reads
  "<SKU> is already used by “title”"; no auto-advance of the proposal when the shop holds it (row C5(f) only disables);
  new local items get an empty `item_type` (filled in the form later), rows show `item_type` or else the title;
  "Create “…”" copy uses the design's curly double quotes; opening a session calls `session_activate`, returning to
  Sessions calls `session_deactivate` (ch04 S-B/S-E); the P00 health page stays reachable at `#/health`; checkboxes
  only on publishable rows (ready, failed) as in the design; shadcn `Progress` fixed to forward `value` (no
  `aria-valuenow` before); `next-themes` dropped, `cmdk`/`sonner` pinned exactly. Session thumbnails use
  `convertFileSrc`; they display once P08 enables the asset protocol (P08 owns that choice). Camera not tethered:
  no hardware probe run. Handoff `handoffs/implementer/handoff_P07_shell_sessions_ui_implement_r1.md`.
- 2026-10-25 review r1 (opus): **CHANGES_REQUESTED.** Checkpoint `acfcc8c` = the 45 declared paths; stamp on that
  tree green (`cargo test --workspace` 181, clippy, fmt, typecheck, `npm test` 17/17). Verified correct: all 19 rows
  at their boundaries (C1(a) group order; C2(a–f) exact pill copy, `Progress` now forwards `value`; C3(a) filter +
  per-key counts; C3(b) debounced server-side query; C4(a–c) dot state, help text, one retry; C5(a/b/f/d) Create row
  only without an exact title hit, Create enabled only for a checked current SKU, `item_create` → `session_create`
  → navigate, shop hit disables Create; C5(c) local substring search; C5(d) `create` returns the existing session;
  C5(e) MC-22 verbatim; C6(a)); one test per row, none over-engineered; H1–H8 hold. **F1 (B1, route production):**
  `EditingStartup` re-applies the persisted active session's capture folder at launch, the app opens on Sessions,
  and `useCaptureTarget` deactivates only after an in-run activation, so after a relaunch frames shot with no
  session open land in the last opened session (ch04 S-E, S-A; ch06 arrival). Correction: at launch make UI and
  backend agree (open the active session's view, or switch to Unassigned without racing the startup re-apply),
  with one behaviour test. Files: `App.tsx`, `services/sessions.ts`, `bindings.ts`, `src-tauri/src/commands/sessions.rs`,
  `src-tauri/src/lib.rs`, `crates/core/src/services/sessions.rs`, `crates/core/src/services/editing_startup.rs`, the
  two P07 test files. Assumption 8 (no proposal advance on a shop hit) differs from MC-22 but is unreachable until
  P12: backlog. 6 notes in backlog. Handoff `handoffs/reviewer/handoff_P07_shell_sessions_ui_review_r1.md`.
- 2026-10-25 fix 1 (sonnet): **F1 resolved, first of the two offered corrections** (open the active session's view).
  `SessionsService::active()` reads the stored `active_session_id`; command `session_active` returns that session or
  null; `App` calls it once at launch and, if a session is active and the route is still the default, navigates to it
  (`useResumeActiveSession`). The UI only reads the stored id, never writes it, so it cannot race `EditingStartup`'s
  background re-apply; `EditingStartup` is untouched (P04 Task 5 stays as designed). Judgment: this means a relaunch
  resumes the session left open rather than showing the Sessions list (ch04 S-A) only when a session was active when
  the app quit; with none active the list shows. Going back to Sessions still deactivates (S-E). The command returns
  the session (`Option<SessionResponse>`) because `CommandId` is input-only. Tests: `F1` x2 in the frontend file
  (active session opens its view without a deactivate; none active stays on the list) and
  `f1_active_reports_the_stored_active_session` in the Rust file. Stamp: `cargo test --workspace` 182 passed, clippy
  and fmt clean, typecheck clean, `npm test` 19/19. Handoff `handoffs/implementer/handoff_P07_shell_sessions_ui_fix_r2.md`.
- 2026-10-25, review r2 (opus): **APPROVED_WITH_NOTES**. Gate: tracker `IMPLEMENTED` with fix note; checkpoint `7f1a0c0` present. Perimeter: `git show --stat 7f1a0c0` = 8 code/test paths, all in F1's allowed list (`App.tsx`, `services/sessions.ts`, `bindings.ts`, `commands/sessions.rs`, `src-tauri/lib.rs`, `core/services/sessions.rs`, both P07 test files) + fix handoff, P07 tracker row, this log; `editing_startup.rs` untouched; `git diff 7f1a0c0 HEAD` = the r2 prompt only. Stamp (prompt-directed, approval gate) on code tree = `7f1a0c0`: `cargo test --workspace` 182 passed 0 failed, clippy `-D warnings` 0, fmt 0, `npm run typecheck` 0, `npm test` 19/19. F1 resolved per correction option 1: `session_active` (thin; `SessionsService::active()` reads `active_session_id`, the same value MC-2 arrival attributes by) is read once at launch (`navigate` is a stable `useCallback`, so the effect runs once; StrictMode double-run guarded by `cancelled`), and only a default route (`""`/`#/`) is redirected; `useCaptureTarget` then re-activates that id (idempotent). Read-only, so no race with `EditingStartup`. Tests discriminate: removing the hook leaves hash `#/` and fails F1(a); F1(b) pins the no-active case on the list. Divergence from the literal S-A wording (relaunch resumes the open session) is sound: S-E requires screen and capture target to agree, and the session view keeps Sessions one click away (which deactivates). Triage: no blocking items remain. 1 note to backlog (P07 → book 04, S-A wording). Narrative `narratives/P07_shell_sessions_ui.md`. Handoff `handoffs/reviewer/handoff_P07_shell_sessions_ui_review_r2.md`.
