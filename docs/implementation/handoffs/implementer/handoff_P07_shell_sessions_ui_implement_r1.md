---
plan: P07_shell_sessions_ui
role: implement
round: 1
state: DONE
date: 2026-10-25
actor: codex
---
# Handoff — P07_shell_sessions_ui, implement r1

Actor note: the prompt names actor `codex`. This session ran on Claude Opus (owner decision of 2026-10-25, flow §2).
It resumed the attempt that the Codex usage limit stopped. Gate passed: the intention is RATIFIED, P03 and P04 are
APPROVED, and P07 was NOT_STARTED.

## What was built
- **Opening the studio (S-A).** The app opens on the Sessions page. The header shows "Session Manager", the camera
  status, a bell, a light/dark toggle and **New session**. The camera status is a green dot plus the camera name when
  Capture One reports a camera. Otherwise it is a red dot reading "Camera disconnected". Hovering it explains what to
  check: USB mode, and that Imaging Edge must be closed. Clicking it retries. It also follows `camera_status_changed`.
- **Sessions list.** Rows are grouped exactly as the backend delivers them (Today, Yesterday, …). Filters: Open
  (default), Published and All, each with its count. Search by title or SKU is debounced and runs on the server. A
  time filter (Any time, Today, Last 7 days, Last 30 days, Reset) puts a red dot on its button while active. Hovering
  a row's thumbnail or name opens a preview card. Status pills: Publishing n% with a bar, Publish failed + Retry,
  Description failed, Published, Ready · Publish, and In progress + "n to review". The dashed Unassigned card appears
  only while unassigned photos exist and opens `#/session/1`. Ready and failed rows have checkboxes, and the header
  box selects all of them. The batch bar shows the count, "Select all N" and a disabled "Review & publish N" (it
  arrives with publishing). Esc clears the selection. The list refreshes on `session_changed` and `photo_added`.
- **New session (S-B, S-D, MC-22).** Press `N` or click New session. Typing searches items. Picking an existing item
  opens its session; the backend now returns the item's existing session (one session per item). When no item has
  that exact title, a "Create “…”" row appears. Choosing it shows a SKU box pre-filled with the proposal
  `<prefix>-<NNNN>` (empty and required when no prefix is set). The SKU is checked against existing items before
  **Create session** enables. A SKU held by the shop reads "<SKU> is already in the shop" and Create stays disabled.
  Create makes the item and its session, then opens `#/session/<id>`, which reads "Ready for the first photo" until
  P08 builds the workspace.
- **Session header.** It shows Back to Sessions, the item's type (or its title until a type is set) and its SKU,
  the camera status, the bell, and disabled Details and Publish placeholders.
- **Capture folder.** Opening a session calls `session_activate`, so captures go to that session. Returning to
  Sessions calls `session_deactivate`, so captures go to Unassigned. A failure shows an error toast.
- **Backend.** `ItemsService::search(q)` matches local items whose SKU or title contains q, ignoring case.
  `ItemsService::next_sku(prefix)` follows MC-22 verbatim. Commands `items_search` and `items_next_sku` were added,
  and the bindings regenerated. `SessionsService::create(item_id)` returns the existing session when there is one.

## Tests → criterion rows
Frontend: `apps/desktop/src/features/sessions/__tests__/p07_shell_sessions_ui.test.tsx`, vitest with services
mocked. Backend: `crates/testkit/tests/p07_shell_sessions_ui.rs`.

| Row | Test | Asserts |
|---|---|---|
| C1(a) | `C1(a) renders the delivered groups in order with their rows` | headers `["Today 2","Yesterday 1"]` in order; A and B only under Today, C only under Yesterday |
| C2(a) | `C2(a) Publishing(42)…` | text "Publishing 42%"; progressbar `aria-valuenow=42` |
| C2(b) | `C2(b) PublishFailed…` | "Publish failed" + Retry button |
| C2(c) | `C2(c) DescriptionFailed…` | whole text exactly "Description failed" |
| C2(d) | `C2(d) Published…` | whole text exactly "Published" |
| C2(e) | `C2(e) Ready…` | button named "Ready · Publish" |
| C2(f) | `C2(f) InProgress(3)…` | "In progress" and "3 to review", nothing else |
| C3(a) | `C3(a) clicking Published…` | `listSessions({filter:"published", query:null})`; counts shown 7/6/13 = the mock's |
| C3(b) | `C3(b) typing "oak"…` | not called with "oak" before the debounce; then last call `{filter:"open", query:"oak"}` |
| C4(a) | `C4(a) Connected("Sony ZV-E10")…` | `data-state=connected` (green); title "Camera connected: Sony ZV-E10" |
| C4(b) | `C4(b) Disconnected…` | `data-state=disconnected` (red); hover help mentions "USB mode" and "Imaging Edge" |
| C4(c) | `C4(c) clicking the dot…` | `cameraRetry` called exactly once |
| C5(a) | `C5(a) no match…` | row "Create “Brass floor lamp”" shown after `searchItems("Brass floor lamp")` → [] |
| C5(b) | `C5(b) creating with SKU BFL-1…` | proposal replaced by BFL-1; `createItem({sku:"BFL-1", title:"Brass floor lamp"})` once, then `createSession(17)` (call order checked); hash `#/session/42`; placeholder rendered |
| C5(c) | Rust `c5c_search_returns_local_oak_item_only` | SKUs returned == `["OAK-1"]`, origin `Local` |
| C5(d) | `C5(d) picking an existing item…` + Rust `c5d_session_create_returns_the_existing_session_of_an_item` | UI: `createSession(5)` once, no `createItem`, hash `#/session/9`. Backend: repeated `create(item)` returns the first session id |
| C5(e) | Rust `c5e_next_sku_follows_largest_exact_prefix_number` | `VF` with VF-0419, VF-0420, vf-0999, VF-12a → `"VF-0421"`; absent/empty prefix → `None` |
| C5(f) | `C5(f) a proposal the shop already holds…` | "VF-0421 is already in the shop"; Create disabled; clicking it creates nothing |
| C6(a) | `C6(a) is absent at 0…` | empty at 0; card with count "3" at 3 |

Core-mechanism unit tests: none (the plan lists none).

## Assumptions
1. **Routing:** hash routes (`#/`, `#/session/:id`). They survive reloads of the bundled app's file URL. `#/health`
   keeps the P00 health report reachable until P16.
2. **Debounce:** 250 ms for the list search, 200 ms for the item search, 250 ms for the SKU check.
3. **Time filter:** calendar-day based, like MC-4. "Last 7 days" means today plus the six local days before it.
4. **Theme:** stored in `localStorage` (`psm.theme`) until P16's `settings_set`. Applied as `.dark` and
   `data-theme` on `<html>`.
5. **Camera help text:** "Capture One does not see a camera. Check that the camera is on and connected by USB in PC
   Remote (USB mode), and that Imaging Edge is closed, since it holds the camera. Click to retry." It shows on hover
   or focus while disconnected.
6. **`next_sku` signature:** `next_sku(prefix)` takes the prefix as an argument, because core cannot read the
   settings file. The `items_next_sku` command passes `settings.sku_prefix`.
7. **"In the shop" detection:** a search hit whose `origin` is not `local` counts as being in the shop. Today the
   backend returns only local items; P12 decides how shop hits are shaped. A clash with a local item reads
   "<SKU> is already used by “title”" and also disables Create.
8. **No proposal auto-advance:** when the shop holds the proposed SKU, the proposal does not move to the next number
   on its own. Row C5(f) asks only for the message and the disabled button. The user edits the SKU.
9. **New local item type:** `item_type` is empty, to be filled in later in the form. Rows and the header show the
   type, or the title while the type is empty.
10. **Create row copy:** uses the design screenshot's curly double quotes, "Create “Brass floor lamp”". The
    sub-line is "New item · saved locally; created on the shop when you publish". No "product" appears in UI copy.
11. **Capture folder:** the session is activated when its view opens and deactivated when you return to Sessions
    (S-B, S-E). P07 is the first phase with routes, and no other plan wires this.
12. **Checkboxes:** only on publishable rows (ready, failed), as in design screen 06.
13. **Thumbnails** use `convertFileSrc(path)`. They show only once the Tauri asset protocol is enabled, which the P08
    plan delegates to P08. Until then rows show the grey placeholder.
14. **Placeholders:** the bell has no count until P13/P15, and the session header's Details and Publish buttons are
    disabled.

## Deviations
- The shadcn `Progress` (`components/ui/progress.tsx`) never passed `value` to the Radix root, so it had no
  `aria-valuenow`. It now forwards it.
- `next-themes` was removed (it was pulled in by the shadcn Sonner template), and the Toaster takes `theme` as a prop.
  `cmdk` 1.1.1 and `sonner` 2.0.8 are pinned exactly, as master plan §6.2 requires.
- Unused shadcn components from the stopped attempt (`badge`, `card`) were removed. `input` and `textarea` stay
  because `input-group`, which `command` uses, imports them.
- The P00 `HealthPage` is no longer the root. It moved to `#/health`.
- `crates/core/src/services/sessions.rs` is not in the plan's file list. It changed because row C5(d) and the plan's
  Notes require `create` to return the existing session.

## Files changed
Production: `apps/desktop/src/App.tsx`; `apps/desktop/src/features/shell/{Header,CameraDot,NotificationsBell,ThemeToggle}.tsx`;
`apps/desktop/src/features/sessions/{SessionsPage,SessionRow,StatusPill,GroupHeader,Filters,TimeFilter,HoverPreview,UnassignedCard,BatchBar,NewSessionDialog}.tsx`;
`apps/desktop/src/services/{sessions,items,camera,events}.ts`; `apps/desktop/src/services/bindings.ts` (regenerated);
`apps/desktop/src/lib/{format,keyboard}.ts`; `apps/desktop/src/components/ui/{checkbox,command,dialog,hover-card,input-group,input,popover,progress,sonner,textarea,toggle-group,toggle}.tsx`;
`apps/desktop/src/styles/globals.css`; `apps/desktop/package.json`, `package-lock.json`;
`apps/desktop/src-tauri/src/commands/items.rs`, `apps/desktop/src-tauri/src/lib.rs`;
`crates/core/src/services/items.rs`, `crates/core/src/services/sessions.rs`.
Tests: `apps/desktop/src/features/sessions/__tests__/p07_shell_sessions_ui.test.tsx`, `crates/testkit/tests/p07_shell_sessions_ui.rs`.
Docs: `docs/implementation/master_plan.md` (P07 tracker row only), `docs/implementation/plans/P07_shell_sessions_ui.md` (Review log entry).
This handoff file is left to the runner to commit. No mutation probes were run (flow §4), and no other files were touched.

## Checkpoint SHA
`acfcc8c` — `CHECKPOINT (not approved): P07 shell, sessions list and new session`. It contains the 45 paths above;
the tree was clean after the commit.

## Stamp
Run on the tree that became `acfcc8c`. Node 22 via `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH`;
cargo via `~/.cargo/bin`.

| Command | Result |
|---|---|
| `cargo test --workspace` | exit 0; 181 passed, 0 failed (P07: 3/3) |
| `cargo test -p psm-desktop export_bindings` | ok; the committed `bindings.ts` equals the regenerated one |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | exit 0; 17/17 (P07 file) |
| `cd apps/desktop && npx vite build` | built (sanity check only, not part of B4) |

No pre-existing failures. No hardware probe was run: the camera is not tethered (STATUS 15:55), and P07 has no
capture-dependent row.

## Manual try-out
1. `cd apps/desktop && PSM_DATA_DIR=/tmp/psm-p07 npm run tauri -- dev`. The Sessions page shows "No sessions yet".
   The header shows the camera status: green with the camera name when the ZV-E10 is tethered in Capture One,
   otherwise red "Camera disconnected". Hover it for the help text; click it to retry.
2. Press `N` and type "Brass floor lamp". Choose "Create “Brass floor lamp”". With no `sku_prefix` in
   `/tmp/psm-p07/settings.json` the SKU box is empty, so type one (e.g. `BFL-1`). With `"sku_prefix": "VF"` set
   before launch, the box reads `VF-0001`. Click **Create session**: the session view opens, reading "Ready for the
   first photo", with the item and SKU in the header.
3. Click **Sessions**. The row shows "In progress", and the filters read Open 1, Published 0, All 1. Type part of
   the title in the search field to filter, use the slider icon for the time filter, and toggle the theme with the
   moon or sun icon.
4. Press `N` and type "Brass" again, then pick the existing item. The same session opens; no second one is created.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
