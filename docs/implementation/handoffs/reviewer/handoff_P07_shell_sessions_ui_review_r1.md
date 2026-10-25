---
plan: P07_shell_sessions_ui
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-10-25
actor: opus
---
# P07 shell, sessions list and new session — review r1

**Verdict: CHANGES_REQUESTED.** One blocking finding, `route: production`: F1 (B1). After a relaunch the app shows
the Sessions page (no session open), but captures still land in the session opened before quitting, against
ch04 S-E. All 19 criterion rows hold at their boundaries; suite, lint and typecheck are green. Six notes went to
the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Gate and evidence

- Tracker showed `IMPLEMENTED`; checkpoint `acfcc8c7ac9ec338be6459eefa94c16e14f12645` is in `git log`.
- `git show --stat acfcc8c`: 45 paths, equal to the handoff's declared list. Extra to the plan's file list and
  declared as deviations: `crates/core/src/services/sessions.rs` (C5(d) requires it), `globals.css`,
  `package.json`/`package-lock.json`, `src-tauri/src/lib.rs`, shadcn components. No finding.
- `git diff acfcc8c HEAD`: only the implementer handoff and this round's review prompt; the code tree equals the
  checkpoint. `git status --porcelain` clean at review entry.
- Stamp on that tree (Node 22, cargo from `~/.cargo/bin`): `cargo test --workspace` exit 0 (181 passed, P07 3/3);
  `cargo clippy --workspace --all-targets -- -D warnings` 0; `cargo fmt --all -- --check` 0;
  `npm run typecheck` 0; `npm test` 0 (17/17).
- No production file was edited, not even temporarily (flow §4). No probe touched any file.

## Verified correct

- **C1(a) grouping.** `SessionsPage` renders `list.groups` in delivered order, one `section` per group labelled by
  the group; the time filter only drops rows. The test checks header order and that each row sits only in its own
  group; reordering or merging groups fails it.
- **C2(a–f) status pill.** One branch per `RowLabelResponse.kind` with the exact copy. `Progress` now forwards
  `value` (a real fix: before, the bar had no `aria-valuenow`). The tests pin the whole text where the row says
  "nothing else" (c, d, f), so an extra "to review" or a wrong label fails. The Ready and Retry buttons render
  disabled until P15 (visual only, as the plan's Goal says).
- **C3(a) filter, C3(b) search.** `Filters` drives `sessions_list(filter, query)`; counts come from the response,
  per key. The test's counts (7/6/13) are distinct per key, so a swapped key fails. Search is debounced (250 ms) and
  server-side; the test first asserts no "oak" call before the debounce, then the last call.
- **C4(a–c) camera dot.** `data-state` drives the green/red light in `globals.css`; title carries the camera name;
  the tooltip text names USB mode and Imaging Edge; click calls `camera_retry` once and shows its result.
  `camera_status_changed` is followed.
- **C5(a), C5(b), C5(f), C5(d) dialog.** The Create row appears only after the search for the exact typed title
  returned and no result has that exact title (case-insensitive). Create is enabled only when the SKU box's
  current value has been checked (`skuCheck.sku === typedSku`), so a stale check cannot enable it. Create sends
  `item_create(sku, title, '', null)` then `session_create(item.id)` and navigates to the returned id; call order is
  asserted. A non-`local` hit with the same SKU shows "<SKU> is already in the shop" and disables Create; clicking
  it creates nothing. Picking an existing item calls only `session_create`.
- **C5(c) search, C5(d) one session per item, C5(e) MC-22 proposal (Rust).** `ItemsService::search` filters local
  items by SKU or title substring, case-insensitive; the fixture's "Teak desk"/"TEAK-1" do not contain "oak", so
  an unfiltered search fails the test. `SessionsService::create` returns the item's existing session; the test
  calls it twice after `create_local` and checks the store still maps the item to the first session. This is also
  the production path of C5(b), since `create_local` already makes the session. `next_sku` follows MC-22 exactly:
  case-sensitive prefix, `-`, all-digits, largest + 1, at least 4 digits; `vf-0999` (case) and `VF-12a` (non-digit)
  are excluded and the test would fail on either mistake; absent and empty prefix → `None`. Empty `item_type`
  satisfies the `NOT NULL` column (backlog P01 → P07 note resolved this way).
- **C6(a) Unassigned card.** Not rendered at 0, rendered with exactly "3" at 3; opens `#/session/1`.
- **Tests.** One behavioural test per row, services mocked at `src/services/*` (master plan §6.9). No test of
  internals, no orphan tests. Each passes the over-engineering test.
- **Assumptions 1–7, 9, 10, 12–14.** None contradicts the book. Assumption 8 and assumption 11 are discussed below.
- **Hard constraints.** H1 no external writes. H2/H3/H5 untouched. H4 `session_create` is now idempotent per item.
  H6 the whole flow works with no integration. H7 core `search`/`next_sku` name no platform concept. H8
  `items_search` and `items_next_sku` are thin; the command passes `settings.sku_prefix` to the core service.
  No secrets, no live services in tests.

## Findings

### F1 (B1) — after a relaunch, captures go to the last opened session while the UI shows none open

- **route:** production
- **What is wrong.** P04's `EditingStartup::run` re-applies the persisted `active_session_id`'s capture folder at
  launch (P04 Task 5, by design). The window loads without a hash, so `App` opens on the Sessions page, and
  `useCaptureTarget` calls `session_deactivate` only when `active.current` is true, which is never the case on a
  fresh launch. Sequence: open session A → quit → relaunch → the app shows the Sessions list, no session open →
  shoot → MC-2 attributes the frame by folder to A. P07 makes this reachable: it is the first phase that sets
  `active_session_id` from the UI (assumption 11).
- **Authority.** Book ch04 S-E ("photos arrive with no session open. They land in Unassigned"), S-A (the app opens
  on the Sessions list); ch06 §arrival (no active session → Unassigned).
- **Correction.** At launch, make the UI and the backend agree: either open the active session's view when one is
  active (needs a small read of `active_session_id`, e.g. a `session_active` command), or switch captures to
  Unassigned when the app opens on the Sessions page. If the second option is chosen, it must not race P04's
  background startup re-apply (startup can read the old id before the UI's `deactivate` writes `null`, then set
  the old folder after it). Add one behaviour test for it at the services boundary.
- **Files the fix may touch.** `apps/desktop/src/App.tsx`, `apps/desktop/src/services/sessions.ts`,
  `apps/desktop/src/services/bindings.ts` (regenerated), `apps/desktop/src-tauri/src/commands/sessions.rs`,
  `apps/desktop/src-tauri/src/lib.rs`, `crates/core/src/services/sessions.rs`,
  `crates/core/src/services/editing_startup.rs` (only if the race is closed there), the two P07 test files.

## Assumptions in plain words (for the narrative at approval)

- Routes are `#/` and `#/session/<id>`; the old health page stays at `#/health`.
- Search waits 250 ms after typing (200 ms in New session, 250 ms for the SKU check).
- "Last 7 days" is today plus the six days before, by calendar day.
- The light/dark choice is remembered in the app window's local storage until Settings arrives.
- A new item's type starts empty; the list and header show the title until a type is set.
- A SKU clash with a local item reads "<SKU> is already used by “title”" and blocks Create.
- Only ready and failed rows get a checkbox.
- Thumbnails stay grey until the workspace phase enables preview files.
- **Assumption 8 (no auto-advance)** differs from MC-22's parenthesis "(the proposal advances to the next free
  number)". Not B1 today: `items_search` returns local items only, so the shop branch cannot fire until the
  Shopify adapter exists. Routed to P12 in the backlog. Plan row C5(f) asked only for message + disabled.

## Notes written to backlog

1. P07 → P12: MC-22 proposal advance on a shop hit not implemented (assumption 8); "in the shop" = `origin ≠ local`
   is provisional (assumption 7).
2. P07 → P08: the session header reads the item's type/SKU by fetching the whole `sessions_list(all)` on each open
   and never refreshes on `session_changed`.
3. P07 → P08: `unassigned_count` (P03 note) still counts deleted photos; the card shows it as is.
4. P07 → P16: theme lives in `localStorage` (`psm.theme`); move to `settings.theme` with `settings_set`.
5. P07 → P08/P16: error codes other than `sku_collision` reach the UI as raw messages (P01 note still open).
6. P07: the filter counts ignore the client-side time filter (counts are for the whole filter).

## Lessons for the plans

- A UI phase that first wires a backend state the user cannot see (here the active session) needs a row for its
  launch state, not only for its transitions.
- Rows that restate a contract should restate it whole (C5(f) vs MC-22).

## Write perimeter

`docs/implementation/handoffs/reviewer/handoff_P07_shell_sessions_ui_review_r1.md` (this file),
`docs/implementation/plans/P07_shell_sessions_ui.md` (Review log entry), `docs/implementation/master_plan.md`
(P07 tracker row only), `docs/implementation/backlog.md` (6 rows appended). No code, no probes, no archgraph.
