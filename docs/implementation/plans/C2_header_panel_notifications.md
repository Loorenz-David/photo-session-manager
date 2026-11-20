---
phase: C2_header_panel_notifications
title: Header icon, panel and notifications — a chat icon in the header with the number of conversations that have news (inside a session: that session's, with a dot when others have news), a right-side panel listing Open and Resolved conversations, an "N open" / "Pricing is waiting" badge on the sessions list, and a toast, a soft chime and a Mac banner when a question or a pricer reply arrives, with a mute button
difficulty: medium
projection: no
depends_on: [C1_connection_and_sync]
blocked_on: none
---
# C2 — Header icon, panel list, counts, notifications

**Tracker:** IMPLEMENTED (2026-11-20, opus, implement r1: 3 units checkpointed 89809fe, e3fd424, 2076b3d; 66 rows, 66 tests at their boundaries (rules 6, suite 17, desktop c2_tests 10 + alerts 1, chime 3, UI 29); closing gate green: workspace 2035/0, clippy, fmt, typecheck, npm test 709/0)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `09B` §6, §7 (Panel → List), §9 over C1's local copy. (U1) The list, counts, badges and the session match behind commands. (U2) The header icon, the panel list, the sessions-list badges, the mute button. (U3) Toast, chime and Mac banner. **Not here:** the conversation view and the composer (C3: selecting a row shows a placeholder line until then); Open session and the visit (C4).

## How to work this plan (flow.md §10, master_plan_conversations.md §3)
One runner phase of 3 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `C2·C<n>(<row>)`; tests named after their row (`c2_c2a_<slug>`). Open details: the smallest option consistent with 09B §6, §7, §9, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_conversations/C2_header_panel_notifications.md`. **Never call Pricing or Shopify, never launch the built app** (master plan §3).

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The list, counts, badges and session match behind commands | 7 | 28 | **yes** |
| U2 | Header icon, panel list, sessions-list badges, mute | 8 | 24 | **yes** (TypeScript) |
| U3 | Toast, chime, Mac banner | 4 | 14 | **yes** |
| **Total** | | **19** | **66** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §10) · `docs/implementation/master_plan_conversations.md` §3, §4, §5 MC-C1.9–10, §6, §7 · book `09B` §3, §6, §7 (Panel, List), §9, §12 (CV-1, CV-2, CV-8) · the package's `CHAT_UX.md` §1, §6, §7 with `images/07-inbox.png` (read-only) · the C1 plan, Units 3 and 4 · `docs/guide/02_architecture.md` §Frontend · `docs/intention/evidence/design_handoff.md`.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **The icon is on every page** (Sessions, a session, Settings, Health); the count is the number of conversations with an unread pricer message.
- **A conversation belongs to the session whose SKU is exactly its SKU;** otherwise it reads "Not on this Mac".
- **The Resolved tab lists conversations whose last update is at most 30 days old.**
- **The chime is a short tone the app makes itself** and plays through macOS, so it also sounds right after launch; the mute button in the panel and the switch in Settings are the same setting.
- **macOS asks once whether Session Manager may show notifications;** if you refuse, banners never appear and the app cannot tell (System Settings → Notifications turns them back on).
- **Clicking a banner brings Session Manager forward;** the toast with Open stays until the window is focused again, and 10 seconds after.
- **Opening a conversation arrives with the next step;** until then a row shows its summary only.

## Unit 1 — The list, counts, badges and session match behind commands

**Unit id:** `C2·U1` · **Rows:** C2·C1–C7 · **Changes app files:** yes

### Goal
`ConversationsService::{list, counts, session_badges}` and the commands `conversations_list`, `conversations_counts`, `conversations_session_badges`.

### Read first
Master plan §4 (Service, Commands), §5 MC-C1.10 · book `09B` §3 (Product = session), §6, §7 (List) · `crates/core/src/services/conversations.rs`, `crates/core/src/mechanisms/conversation_rules.rs` (as C1 left them) · `crates/core/src/ports/store.rs` (`items`, `session_for_item`) · `crates/core/src/domain/item.rs` (`sku`) · `apps/desktop/src-tauri/src/commands/conversations.rs`.

### Files
Edited: `crates/core/src/{mechanisms/conversation_rules.rs, ports/conversations.rs, services/conversations.rs, domain/conversation.rs}`, `crates/infra/src/sqlite/conversations.rs`, `apps/desktop/src-tauri/src/{commands/conversations.rs, lib.rs}`, `apps/desktop/src/services/{bindings.ts (generated), conversations.ts}`, `crates/testkit/tests/suite/main.rs`. New: `crates/testkit/tests/suite/c2_conversations_lists.rs`.

### Core mechanisms (the only functions with unit tests)
`conversation_rules::list_order(a, b)` (news first, then blocking, then latest activity, then id) · `conversation_rules::resolved_recent(updated_at, now)` (`now − updated_at ≤ 30 × 86 400 000 ms`) · `conversation_rules::first_line(text)` (the first non-empty line, trimmed, at most 120 characters, cut on a character boundary with `…`).

### Ordered tasks
1. **Session match:** `session_of(conversation)` = the session of the item whose `sku` equals the conversation's `sku` exactly (`store.items()`, `session_for_item`); no SKU, no such item or no session → none. The Unassigned session has no item and never matches.
2. `list(scope: All | Session(id), tab: Open | Resolved, now) -> ConversationList {state: ConnectionState, problem: Option<String>, rows: Vec<ConversationRow>, news_elsewhere: i64}` with `ConversationRow {id, sku, product_title, topic, status, waiting: bool, last_line: Option<String>, last_activity_at, has_news, session_id: Option<i64>}`. Open = status `open` or `answered`, sorted by `list_order`; Resolved = status `resolved` ∧ `resolved_recent`, sorted by `last_activity_at` descending then id. `waiting` = `blocking` ∧ status ≠ `resolved`. `last_line` = `first_line` of the newest message. Scope `Session(id)` keeps the rows whose `session_of` is `id`; `news_elsewhere` = conversations with news outside the scope (0 for `All`). State `no_key`, `no_shop` or `store_unmatched` → no rows, `news_elsewhere` 0. `problem` = `last_error` when the state is `refused` or `store_gone`.
3. `counts(session_id: Option<i64>) -> Counts {here, elsewhere}`: conversations with news (any status) of that session and of all others; without a session `here` = all, `elsewhere` = 0; zeros in the three hidden states.
4. `session_badges() -> Vec<SessionBadge {session_id, open, waiting}>`: per session with at least one unresolved conversation; `waiting` = one of them is blocking; empty in the three hidden states.
5. Commands, DTOs, bindings, wrappers. Tests for the rows.

### Criteria (7 criteria, 28 rows)
Boundary: `ConversationsService` on `TestApp::with_pricing()` with conversations brought in by `tick` from `FakePricing` (outcome = the returned values); the three rules (pure); C2·C7 at the command layer (`c2_tests` in `apps/desktop/src-tauri/src/lib.rs`).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C2·C1(a) | `list(All, Open)` | a session whose item has SKU `BEYO-1880`; a conversation with SKU `BEYO-1880` | the row's `session_id` is that session | 09B §3 |
| C2·C1(b) | same | the item's SKU is `BEYO-1880`, the conversation's `beyo-1880` | `session_id` none | 09B §3 (exact match) |
| C2·C1(c) | same | a conversation with no SKU; a conversation whose SKU no item has | `session_id` none for both; both rows are listed | 09B §3 ("Not on this Mac"); CV-7 |
| C2·C1(d) | same | the session of that SKU is deleted (`delete_session`) | the conversation is still listed, `session_id` none | CR3 |
| C2·C2(a) | `list_order` | A has news, not blocking, older; B no news, blocking, newer | A before B | 09B §7 (news first) |
| C2·C2(b) | same | both no news; A blocking, older; B not blocking, newer | A before B | 09B §7 (then blocking) |
| C2·C2(c) | same | both no news, both not blocking; A activity 200, B activity 100 | A before B | 09B §7 (then latest activity) |
| C2·C2(d) | same | both have news; A not blocking, newer; B blocking, older | B before A | 09B §7 |
| C2·C2(e) | `list(All, Open)` | four conversations covering (a)–(d) plus one resolved | the four in the order the rule gives; the resolved one absent | 09B §7 |
| C2·C3(a) | `resolved_recent` | updated 29 days before now; exactly 30 days; 30 days and 1 ms | true; true; false | 09B §7 (last 30 days) |
| C2·C3(b) | `list(All, Resolved)` | resolved 2 days ago (activity 500), resolved 5 days ago (activity 900), resolved 40 days ago, one open | the two recent ones, activity 900 first; the others absent | 09B §7 |
| C2·C4(a) | `first_line` | "Is the glass original?\nSecond line"; "\n  Send a close-up  \n"; 200 letters | "Is the glass original?"; "Send a close-up"; 119 letters + "…" | 09B §7 (the last message's first line) |
| C2·C4(b) | `list(All, Open)` | a conversation whose last message is the pricer's "Close-up please\nof the dent" at t | `last_line` "Close-up please", `last_activity_at` t, `has_news` true | 09B §7 |
| C2·C4(c) | same | a blocking open conversation; a blocking answered one; a non-blocking open one | `waiting` true, true, false | 09B §6; CHAT_UX §7 rule 2 |
| C2·C4(d) | `list(All, Resolved)` | a blocking resolved conversation | `waiting` false | 09B §6 |
| C2·C5(a) | `list(Session(A), Open)` | conversations for sessions A and B and one not on this Mac; those of B and the third have news | only A's rows; `news_elsewhere` 2 | 09B §7 (footer) |
| C2·C5(b) | `counts(None)` | three conversations with news (one of them resolved, one not on this Mac), two without | `here` 3, `elsewhere` 0 | 09B §6 |
| C2·C5(c) | `counts(Some(A))` | A has one conversation with news; two others have news elsewhere | `here` 1, `elsewhere` 2 | 09B §6; CV-2 |
| C2·C5(d) | `counts(Some(A))` then `mark_read` then `counts` | A's conversation with news | `here` 1, then 0 | 09B §6; CV-1 |
| C2·C6(a) | `session_badges()` | session A: two unresolved conversations, none blocking | one badge `{session_id: A, open: 2, waiting: false}` | 09B §6 |
| C2·C6(b) | same | session A: one blocking conversation already answered, one open | `{open: 2, waiting: true}` | 09B §6 (blocking until resolved) |
| C2·C6(c) | same | session A: one blocking conversation, resolved | no badge for A | 09B §6 |
| C2·C6(d) | same | a conversation not on this Mac | no badge | 09B §6 |
| C2·C7(a) | `conversations_list` | state `no_key` with conversations still in the copy | `state` `no_key`, no rows | 09B §4 (panel says connect) |
| C2·C7(b) | same | state `refused` with two conversations in the copy | `state` `refused`, `problem` "Pricing refused the key: check Settings → Pricing", the two rows | 09B §5.4; CV-6 |
| C2·C7(c) | `conversations_counts`, `conversations_session_badges` | state `store_unmatched` with conversations with news in the copy | `{here: 0, elsewhere: 0}`; no badge | 09B §4 |
| C2·C7(d) | `conversations_list` | scope `{kind: "session", session_id}`, tab `resolved` | the same rows as the service gives for `Session(id)`, `Resolved` | master plan §4 Commands |
| C2·C7(e) | any of the three commands | — | the fake's request record is unchanged by the call (lists never wait on the network) | 09B §5.1; CR5 |

### Delegated choices
SQL or in-memory filtering (the copy is small) · DTO field names beyond those listed.

### Notes
- Hazard: "has news" counts resolved conversations too (a pricer message nobody opened); only the Open tab and the badges filter by status.
- Hazard: a list call never takes the lane and never calls the provider.

### Unit test commands
`cargo test -p psm-core --lib mechanisms::conversation_rules` · `cargo test -p psm-testkit --test suite c2_conversations_lists::` · `cargo test -p psm-desktop` · `cargo clippy -p psm-core -p psm-infra -p psm-desktop -p psm-testkit --all-targets -- -D warnings`

## Unit 2 — Header icon, panel list, sessions-list badges, mute

**Unit id:** `C2·U2` · **Rows:** C2·C8–C15 · **Changes app files:** yes (TypeScript)

### Goal
`ConversationsButton`, `ConversationsPanel` (list only), `ConversationRow`, `SessionBadge`, `useConversations`, mounted once in `App.tsx` for every route.

### Read first
`apps/desktop/src/App.tsx` (the early returns for Health and Settings; `Header` props) · `apps/desktop/src/features/shell/{Header.tsx (the bell and `NotificationsDrawer`), PageHeader.tsx}` · `apps/desktop/src/features/health/HealthPage.tsx`, `apps/desktop/src/features/settings/SettingsPage.tsx` (their `PageHeader`) · `apps/desktop/src/features/sessions/{SessionsPage.tsx, SessionRow.tsx, ResearchPill.tsx (`useResearchOverview`: the read-and-refresh pattern)}` · `apps/desktop/src/features/publish/NotificationsDrawer.tsx` (a right-side drawer) · `apps/desktop/src/services/{conversations.ts, events.ts}` · `apps/desktop/src/features/shell/__tests__/a11_header_review.test.tsx`, `apps/desktop/src/features/sessions/__tests__/p07_shell_sessions_ui.test.tsx` (how header and list are tested).

### Files
New: `apps/desktop/src/features/conversations/{ConversationsButton.tsx, ConversationsPanel.tsx, ConversationRow.tsx, SessionBadge.tsx, useConversations.ts}`, `apps/desktop/src/features/conversations/__tests__/{c2_header_icon.test.tsx, c2_panel_list.test.tsx}`, `apps/desktop/src/features/sessions/__tests__/c2_session_badges.test.tsx`. Edited: `apps/desktop/src/App.tsx`, `apps/desktop/src/features/shell/Header.tsx`, `apps/desktop/src/features/health/HealthPage.tsx`, `apps/desktop/src/features/settings/SettingsPage.tsx`, `apps/desktop/src/features/sessions/{SessionsPage.tsx, SessionRow.tsx}`, `apps/desktop/src/styles/globals.css`.

### Core mechanisms
None.

### Ordered tasks
1. `useConversations(sessionId)` in `App.tsx`: `{open, selected, scope, counts, openPanel(conversationId?), close, select, back, showAll}`; counts from `conversationsCounts(sessionId ?? null)`, re-read on `conversations_changed`; provided to the tree through a React context so the button works in `Header`, `HealthPage` and `SettingsPage`. The panel and (U3) the toasts are rendered for every route, including the Health and Settings branches.
2. `ConversationsButton`: a chat icon (lucide `MessageCircle`), `aria-label` "Conversations"; the count `here` when above 0; a small dot when `elsewhere` is above 0 on a session route; placed before the bell in both `Header` branches and in the `PageHeader` children of Health and Settings.
3. `ConversationsPanel`: a right-side panel with a title, the mute button, a close button; tabs **Open** and **Resolved**; rows; on a session route the list is scoped to that session with the footer `N new in other sessions → All conversations` (shown when `news_elsewhere` > 0), which switches the scope to all; hidden states → `Connect Pricing in Settings → Pricing` with a button to `#/settings`; `problem` → its sentence above the list; an empty list → "No open conversations" / "No resolved conversations". Re-read on `conversations_changed`.
4. `ConversationRow`: SKU and product title, topic, a status dot (open, answered, resolved), amber `Pricing is waiting` when `waiting`, `last_line`, the time of `last_activity_at`, an unread dot when `has_news`, `Not on this Mac` when `session_id` is null. Click selects it: until C3 the panel then shows a back arrow, the row's SKU, title and topic, and the line "Opening conversations arrives with the next step".
5. Mute button (bell / bell-off icon, `aria-pressed`): reads `conversationsSettingsGet().sound`, click saves `{sound: !sound}`.
6. `SessionBadge` in `SessionRow` (inside `.row-pills`, shown also when the row has no other pill): `N open`, or amber `Pricing is waiting` when `waiting`; `SessionsPage` reads `conversationsSessionBadges()` and re-reads on `conversations_changed`.
7. Tests for the rows.

### Criteria (8 criteria, 24 rows)
Boundary: the components rendered in UI tests with `services/conversations.ts` and `services/events.ts` mocked (outcome = what is shown and the service calls with their arguments).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C2·C8(a) | `Header` on the Sessions page | counts `{here: 3, elsewhere: 0}` | a "Conversations" button showing 3; `conversationsCounts(null)` was called | `[rule]` 09B §6 |
| C2·C8(b) | same | counts `{here: 0, elsewhere: 0}` | the button without a count and without a dot | `[rule]` 09B §6 |
| C2·C8(c) | `Header` in session 7 | counts `{here: 1, elsewhere: 2}` | the button shows 1 and a dot; `conversationsCounts(7)` was called | `[rule]` 09B §6; CV-2 |
| C2·C8(d) | `App` on `#/settings` and on `#/health` | counts `{here: 2, elsewhere: 0}` | each page shows the button with 2 | `[rule]` 09B §6 (Settings, Health) |
| C2·C9(a) | click the button on the Sessions page | — | the panel opens on the Open tab; `conversationsList({kind: 'all'}, 'open')` was called | `[rule]` 09B §7 |
| C2·C9(b) | click **Resolved** | — | `conversationsList({kind: 'all'}, 'resolved')` was called and its rows are shown | `[rule]` 09B §7 |
| C2·C9(c) | click the close button | — | the panel is gone | `[rule]` 09B §7 |
| C2·C10(a) | `ConversationRow` | sku `BEYO-1880`, title "Vintage Brass Wall Mirror, Round", topic Condition, status open, `waiting` true, `last_line` "Is the glass original?", `has_news` true, `session_id` 7 | all of these are shown; "Pricing is waiting" is present; an unread dot is present; no "Not on this Mac" | `[rule]` 09B §7 |
| C2·C10(b) | same | `waiting` false, `has_news` false | no "Pricing is waiting", no unread dot | `[rule]` 09B §7 |
| C2·C10(c) | same | `session_id` null | "Not on this Mac" | `[rule]` 09B §7; CV-7 |
| C2·C10(d) | same | `sku` null | the product title alone, no empty SKU text | `[rule]` 09B §7 ("not recorded", never invented) |
| C2·C11(a) | the panel in session 7 | — | `conversationsList({kind: 'session', session_id: 7}, 'open')` was called | `[rule]` 09B §7 |
| C2·C11(b) | same | `news_elsewhere` 2 | the footer "2 new in other sessions → All conversations"; a click calls `conversationsList({kind: 'all'}, 'open')` and shows its rows | `[rule]` 09B §7; CV-2 |
| C2·C11(c) | same | `news_elsewhere` 0 | no footer | `[rule]` 09B §7 |
| C2·C12(a) | the panel | list state `no_key` | "Connect Pricing in Settings → Pricing"; no tab rows | `[rule]` 09B §4 |
| C2·C12(b) | same | state `refused`, `problem` set, two rows | the problem sentence and the two rows | `[rule]` 09B §5.4; CV-6 |
| C2·C12(c) | same | state `ready`, no rows on Open | "No open conversations" | `[rule]` 09B §7 |
| C2·C13(a) | the mute button | settings `sound` true; click | `conversationsSettingsSave({sound: false})` called once; the button reads as muted | `[rule]` 09B §4, §9; CV-8 |
| C2·C13(b) | same | settings `sound` false | the button reads as muted on open; a click saves `{sound: true}` | `[rule]` 09B §4 |
| C2·C14(a) | `SessionsPage` | badges `[{session_id: 7, open: 2, waiting: false}]` | session 7's row shows "2 open"; other rows show no badge | `[rule]` 09B §6 |
| C2·C14(b) | same | `[{session_id: 7, open: 1, waiting: true}]` | session 7's row shows "Pricing is waiting" | `[rule]` 09B §6; CV-1 |
| C2·C15(a) | the panel open, then a `conversations_changed` event | the list mock now returns one more row | the new row is shown without closing the panel | `[rule]` 09B §5.1 |
| C2·C15(b) | `Header`, then a `conversations_changed` event | counts go from 1 to 2 | the button shows 2 | `[rule]` 09B §6; CV-1 |
| C2·C15(c) | click a row | — | the panel shows the back arrow and that row's SKU and topic; the back arrow returns to the list | `[rule]` 09B §7 |

### Delegated choices
Panel width, animation and exact styling (design handoff tokens; Pricing's inbox screenshot is the reference) · the wording of the time (reuse `lib/format.ts`).

### Notes
- Hazard: `App.tsx` returns early for Health and Settings; the panel, the context and (U3) the toasts must be inside all three branches, with the same `useConversations` instance.
- Hazard: the existing header tests render `Header` without a provider; give the context a default that reads nothing, or wrap those tests.
- The placeholder line of task 4 is removed by C3.

### Unit test commands
`cd apps/desktop && npx vitest run src/features/conversations src/features/sessions src/features/shell src/features/settings` · `npm run typecheck`

## Unit 3 — Toast, chime, Mac banner

**Unit id:** `C2·U3` · **Rows:** C2·C16–C19 · **Changes app files:** yes

### Goal
The window toasts, the chime, the Mac banner: the event bridge acts on the `chime` and `banner` flags C1 already computes.

### Read first
Master plan §4 (Event), §5 MC-C1.9, §6 (chime, banner), §7 (the notification plugin) · book `09B` §9 · `apps/desktop/src-tauri/src/lib.rs` (`impl EventSink for TauriEvents`, the plugin registrations near `.plugin(tauri_plugin_dialog::init())`, the window-focus handler, the inline event tests) · `apps/desktop/src-tauri/Cargo.toml` · `apps/desktop/src/features/shell/ArrivalToasts.tsx` and `apps/desktop/src/features/sessions/ResearchPill.tsx` (`WriterToasts`): the toast patterns · after adding the dependency, the installed `tauri-plugin-notification` source for the builder's method names.

### Files
New: `crates/infra/src/chime.rs`, `apps/desktop/src-tauri/src/alerts.rs`, `apps/desktop/src/features/conversations/ConversationToasts.tsx`, `apps/desktop/src/features/conversations/__tests__/c2_toasts.test.tsx`. Edited: `crates/infra/src/lib.rs`, `apps/desktop/src-tauri/{Cargo.toml, src/lib.rs}`, `Cargo.lock`, `apps/desktop/src/App.tsx`, `apps/desktop/src/features/settings/PricingSection.tsx` (one hint line under the banner switch), `docs/guide/{02_architecture.md, 04_change_impact.md}`.

### Core mechanisms (the only functions with unit tests)
`psm_infra::chime::wav() -> Vec<u8>` (the tone, generated) · `alerts::banners(toasts, more) -> Vec<String>` (the banner bodies).

### Ordered tasks
1. `chime::wav()`: a 16-bit mono PCM WAV, two soft sine partials with a decaying envelope, at most 0.6 s, peak at most a third of full scale, the same bytes on every call. `chime::play(dir)`: writes `chime.wav` into `dir` when it is absent or differs, then starts `/usr/bin/afplay <file>` without waiting; any failure is logged and swallowed.
2. `alerts.rs`: `trait Alerts {fn chime(&self); fn banner(&self, title: &str, body: &str);}`; `SystemAlerts` (chime → `chime::play(<data folder>)`; banner → the notification plugin's builder, title "Session Manager", body = the line); `banners(toasts, more)` = the toast lines in order, plus `and <more> more` when `more` > 0. `TauriEvents` gets an `Arc<dyn Alerts>` and a "window focused" reader.
3. The `ConversationsChanged` arm: forward `conversations_changed {toasts, more}` (C1); `chime` → `alerts.chime()` once; `banner` ∧ the main window is not focused → `alerts.banner` once per line of `banners`.
4. `tauri-plugin-notification = "2"` in `Cargo.toml`, registered in `run()`. Record in the handoff: the crate version resolved, the builder calls used, and that no capability entry was needed (Rust-side use).
5. `ConversationToasts` (rendered in `App.tsx` for every route): on `conversations_changed`, one sonner toast per entry of `toasts` (`id` `conversation-<id>`, the text, action **Open** → `openPanel(conversation_id)`), and one `and N more` toast when `more` > 0. Duration 8 s when `document.hasFocus()`; otherwise it stays until the window's next `focus` event and 10 s after.
6. `PricingSection`: under the banner switch, the line "If no banner appears, allow notifications for Session Manager in System Settings → Notifications."
7. Guide rows (the alerts seam, the chime file in the data folder, the plugin). Tests for the rows.

### Criteria (4 criteria, 14 rows)
Boundary: `ConversationToasts` in a UI test with `services/events.ts` mocked and sonner observed; `chime::wav` and `alerts::banners` (pure); the `TauriEvents` bridge in `apps/desktop/src-tauri/src/lib.rs` (`c2_tests`) with a recording `Alerts` and a settable focus reader.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C2·C16(a) | `ConversationToasts` | event `{toasts: [{conversation_id: "c1", text: "New question · BEYO-1880 — Condition"}], more: 0}` | one toast with that text and an **Open** action | `[rule]` 09B §9; CV-1 |
| C2·C16(b) | same | click **Open** | `openPanel("c1")` was called once | `[rule]` 09B §9 |
| C2·C16(c) | same | three toasts and `more` 2 | four toasts: the three texts and "and 2 more" | `[rule]` 09B §5.5 |
| C2·C16(d) | same | event with no toasts | no toast | `[rule]` 09B §5.5 |
| C2·C16(e) | same | the document is not focused when the event arrives; then the window gets focus | the toast is created without an end time; after the focus event it is given 10 s | `[rule]` master plan §6 (banner click) |
| C2·C17(a) | `chime::wav` | — | starts with `RIFF`, holds `WAVE`, `fmt ` and `data` chunks whose sizes match the byte length; 1 channel, 16 bits | 09B §9 (built into the app) |
| C2·C17(b) | same | — | lasts more than 0.05 s and at most 0.6 s; at least one sample is not zero; no sample is above a third of full scale | 09B §9 (short, soft) |
| C2·C17(c) | same | two calls | identical bytes | 09B §9 |
| C2·C18(a) | bridge | `ConversationsChanged {notices: [one], chime: true, banner: false}` | the recording alerts saw exactly one `chime` and no banner; the window received `conversations_changed` with one toast | 09B §9; CV-1 |
| C2·C18(b) | same | three notices, `chime: true` | exactly one `chime` | 09B §5.5 (one sound) |
| C2·C18(c) | same | one notice, `chime: false`, `banner: false` | no `chime`, no banner; the window still received the event | CV-8 |
| C2·C19(a) | bridge, window not focused | `banner: true`, one notice (Question, `BEYO-1880`, Condition) | one banner with body "New question · BEYO-1880 — Condition" | 09B §9 |
| C2·C19(b) | bridge, window focused | the same event | no banner | 09B §9 (when not focused) |
| C2·C19(c) | `alerts::banners` | three lines and `more` 2; one line and `more` 0 | the three lines then "and 2 more"; the one line | 09B §5.5 |

### Delegated choices
The tone's two frequencies and decay · the banner title · whether `afplay` is given a volume flag.

### Notes
- Hazard: no audio in the webview (a page may not play sound before a click, so a chime right after launch would be silent); the chime is native.
- Hazard: `chime::play` must never block the event bridge (start the player and return).
- The real banner and the real sound cannot be asserted by the suite: they are steps of the Manual try-out, recorded in the Review log when the owner or the orchestrator runs them (charter rule 1, environment exemption; the automated proxies are C2·C18 and C2·C19).
- The plugin is used from Rust only; if the installed version offers a click callback on macOS, record it under Assumptions and leave the behaviour as planned (the toast stays until focus).

### Unit test commands
`cargo test -p psm-infra --lib chime` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/conversations` · `npm run typecheck` · `cargo clippy -p psm-infra -p psm-desktop --all-targets -- -D warnings`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-core --lib mechanisms::conversation_rules` · `cargo test -p psm-testkit --test suite c2_conversations_lists::` · `cargo test -p psm-infra --lib chime` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/conversations src/features/sessions src/features/shell src/features/settings` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: no list, count or badge call reaches the provider or takes the lane; the panel and the toasts are mounted on every route with one state; the chime never blocks the bridge; no sound file was added to the repo. Notes go to `docs/implementation/backlog_conversations/C2_header_panel_notifications.md`.

## Manual try-out (the narrative is built from this; done by the owner on the installed app, never by a session)
1. In Pricing, ask a blocking Condition question on a product photographed on this Mac. Within about 10 seconds: a chime, a toast "New question · <SKU> — Condition", the header icon shows 1, and the session's row on the Sessions page shows "Pricing is waiting".
2. Click the icon: the panel lists the conversation under Open, with the question's first line. Open that session: the icon lists only its conversations.
3. Put Session Manager in the background and ask a second question: a Mac banner appears (the first time macOS asks whether to allow notifications). Click it: Session Manager comes forward and the toast with Open is still there.
4. Press the mute button in the panel and ask again: no chime; the toast still appears.

## Review log
- **2026-11-20, implementer (opus), implement r1.** Built the three units in order (checkpoints U1 `89809fe`, U2
  `e3fd424`, U3 `2076b3d`; closing commit with tracker, this entry, handoff). U1: `conversation_rules::{list_order,
  resolved_recent, first_line}` with unit tests; `ConversationsService::{list, counts, session_badges}` over the copy
  (never the lane, never a request); a new store method `ConversationStore::item_sessions` (each item's SKU with its
  latest session, the `session_for_item` rule) gives the exact SKU match; domain `ConversationScope/Tab/Row/List/
  Counts/SessionBadge`; commands `conversations_list`, `conversations_counts`, `conversations_session_badges`,
  bindings, wrappers. U2: `useConversations` (one instance in `App.tsx`, `ConversationsContext` in all three route
  branches), `ConversationsButton` (Header both branches, Health, Settings), `ConversationsPanel` (tabs, footer,
  hidden-state line, problem, empty lines, mute button, placeholder on select), `ConversationRow`, `SessionBadge` +
  `useSessionBadges` in `SessionsPage`/`SessionRow`, styles. U3: `psm_infra::chime::{wav, play}` (generated tone,
  `afplay` reaped on its own thread), `alerts.rs` (`Alerts`, `SystemAlerts`, `banners`), `TauriEvents` now a struct
  with alerts and a focus reader (an `AtomicBool` kept by the main window's `Focused` events), the
  `ConversationsChanged` arm chimes once and banners per line when unfocused; `tauri-plugin-notification` 2.8.1
  registered in `run()`; `ConversationToasts`; Health branch got its own `Toaster`; PricingSection hint; guide rows.
  Assumptions and deviations: handoff §Assumptions/§Deviations (session match by a store query instead of
  `store.items()` + `session_for_item`, the service has no `Store`; Unassigned counted as a session route; a toast's
  Open switches the list scope to all; a `blocking` field added to the row; tracker actor `opus`, not `codex`). No
  owner card.
