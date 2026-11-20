---
plan: C2_header_panel_notifications
role: implement
round: 1
state: DONE
date: 2026-11-20
actor: claude (opus, pipeline-implementer)
---
# Handoff — C2_header_panel_notifications, implement r1

Gate: intention RATIFIED (2026-10-24; 09B RATIFIED 2026-11-20); `depends_on: [C1_connection_and_sync]`, whose Tracker
line reads APPROVED (2026-11-20); this plan's Tracker read NOT_STARTED. I worked the three units in order, with a
`CHECKPOINT (not approved):` commit after each unit's test commands passed, then ran the closing gate once.

## What was built
**For the photographer:**
- **A chat icon in the header on every page** (Sessions, a session, Settings, Health). It shows how many conversations
  have an unread pricer message. Inside a session the number counts only that session's conversations, and a small
  dot shows when other sessions have news.
- **A right-side panel** opens from the icon. It has an **Open** tab (unresolved) and a **Resolved** tab (resolved in
  the last 30 days). Open rows are sorted news first, then blocking, then latest activity. Each row shows the SKU and
  product title, the topic, a status dot, amber "Pricing is waiting", the newest message's first line and its time,
  an unread dot, and "Not on this Mac" when no session has that SKU.
- Inside a session the panel lists only that session's conversations, with the footer "N new in other sessions → All
  conversations".
- When Pricing is not connected, the panel says "Connect Pricing in Settings → Pricing" and has a button to Settings.
  A refused key shows its sentence above the list.
- Clicking a row shows a placeholder (back arrow, SKU, title, topic, "Opening conversations arrives with the next
  step") until C3 brings the real conversation view.
- **A mute button** in the panel. It is the same setting as the sound switch in Settings → Pricing.
- **Sessions list badges:** "N open", or amber "Pricing is waiting" when one of the open conversations is blocking.
- **When a question or a pricer reply arrives:**
  - a toast "New question · SKU — topic" / "New reply · SKU" with **Open**, at most three, then "and N more";
  - one soft chime that the app makes itself;
  - a Mac banner with the same words while the window is in the background.
  - A toast that arrives while the window is in the background stays until the window is focused again, then 10 s
    more.
- Settings → Pricing has a new hint under the banner switch about allowing notifications in System Settings.

**In code:**
- Rules: `conversation_rules::{list_order, resolved_recent, first_line}`.
- Service: `ConversationsService::{list, counts, session_badges}`. They read only the local copy: no lane, no
  provider request.
- Store and domain: a new store method `ConversationStore::item_sessions` (SQLite), plus domain types `ConversationScope`,
  `ConversationTab`, `ConversationRow`, `ConversationList`, `ConversationCounts`, `SessionBadge`.
- Commands: `conversations_list`, `conversations_counts`, `conversations_session_badges`, with regenerated bindings
  and wrappers.
- Frontend: `features/conversations/` (`useConversations` + `ConversationsContext`, `ConversationsButton`,
  `ConversationsPanel`, `ConversationRow`, `SessionBadge` + `useSessionBadges`, `ConversationToasts`).
- Chime: `crates/infra/src/chime.rs`.
- Banner: `apps/desktop/src-tauri/src/alerts.rs`.
- Bridge: `TauriEvents` now carries an `Arc<dyn Alerts>` and a focus reader.
- Plugin: `tauri-plugin-notification`, registered in `run()`.

## Tests → criterion rows
All 66 rows have exactly one test, named after its row.
- **U1 (28 rows)**
  - C2·C2(a–d), C2·C3(a), C2·C4(a): unit tests in `crates/core/src/mechanisms/conversation_rules.rs`, named
    `c2_c2a_…`, `c2_c2b_…`, `c2_c2c_…`, `c2_c2d_…`, `c2_c3a_…`, `c2_c4a_…`.
  - C2·C1(a–d), C2·C2(e), C2·C3(b), C2·C4(b–d), C2·C5(a–d), C2·C6(a–d): `crates/testkit/tests/suite/c2_conversations_lists.rs`
    (17 tests). They run on `TestApp::with_pricing()`, with the conversations brought in by `tick` from `FakePricing`.
  - C2·C7(a–e): `c2_tests::c2_c7*` in `apps/desktop/src-tauri/src/lib.rs`, at the command layer of the wired app.
- **U2 (24 rows)**
  - C2·C8(a–d), C2·C15(b): `features/conversations/__tests__/c2_header_icon.test.tsx`. C2·C8(d) renders `App` on
    `#/settings` and on `#/health`, as one parametrised test.
  - C2·C9(a–c), C2·C10(a–d), C2·C11(a–c), C2·C12(a–c), C2·C13(a–b), C2·C15(a), C2·C15(c): `c2_panel_list.test.tsx`.
  - C2·C14(a–b): `features/sessions/__tests__/c2_session_badges.test.tsx`.
- **U3 (14 rows)**
  - C2·C16(a–e): `features/conversations/__tests__/c2_toasts.test.tsx`, with sonner's `toast` observed.
  - C2·C17(a–c): unit tests in `crates/infra/src/chime.rs`.
  - C2·C18(a–c), C2·C19(a–b): `c2_tests::c2_c18*` and `c2_c19*`. These use a recording `Alerts` and a settable focus
    flag, and check the window event with `listen_any`.
  - C2·C19(c): `alerts::tests::c2_c19c_…`.

## Assumptions
1. **Session match by one store query.** The service only has `ConversationStore`, not the app `Store`, and the
   wiring in `crates/integrations/src/conversations/mod.rs` is outside the perimeter. So instead of the plan's
   `store.items()` + `session_for_item`, a new port method `ConversationStore::item_sessions()` returns each item's SKU
   with its latest session. The rule is the same as `session_for_item`: `ORDER BY id DESC LIMIT 1`. The exact
   (case-sensitive) comparison happens in Rust. The Unassigned session has no item, so it never matches.
2. **Unassigned counts as a session route.** The plan says `conversationsCounts(sessionId ?? null)`, so on the
   Unassigned page the count is 0, the dot shows news elsewhere, and the panel is scoped to session 1 (always empty,
   with the footer).
3. **A toast's Open** opens the panel on that conversation and switches the list scope to all. The conversation may
   belong to another session, and the back arrow should lead to a list that holds it.
4. **`blocking` added to the list row** (domain and DTO), so `list_order` can sort on it. C3's header line
   "· Blocking" will need it too.
5. **`first_line` of a message with no non-empty line** gives an empty string, so `last_line` is none.
6. **Focus for the banner rule** comes from an `AtomicBool`. It is set from `window.is_focused()` at setup, then
   updated on every `WindowEvent::Focused`. This avoids calling into the main thread from the bridge.
7. **Chime:**
   - 44.1 kHz, 0.45 s, partials 880 Hz (0.2) and 1320 Hz (0.1), a 6 ms rise, then a decay of e^(−9t);
   - peak about 0.3 of full scale, under the plan's limit of a third;
   - no volume flag for `afplay`;
   - the player process is reaped on its own thread, so the bridge never waits on it.
8. **Banner title:** "Session Manager".
9. **Plugin:**
   - `tauri-plugin-notification` resolved to **2.8.1**;
   - builder calls `app.notification().builder().title(..).body(..).show()` through `NotificationExt`;
   - **no capability entry was needed**, because it is used from Rust only.
   - This version does offer `Notification::on_action`, which reports a `tap` "as far as the notification server
     reports it". As the plan says, I left the behaviour as planned: the toast stays until the window gets the focus.
10. **The Health route got its own `Toaster`**, so conversation toasts show on every route. Before, Health had none.
11. **Listener subscriptions are guarded** (`Promise.resolve().then(...)`), the same pattern as `useResearchOverview`.
    Existing UI tests that mock `services/events` without `onConversationsChanged` keep passing, and the page never
    breaks if a listener is missing.
12. **The "and N more" toast** has the fixed id `conversation-more` and no Open action.

## Deviations
- U1 added `item_sessions` to the `ConversationStore` port. The files are in the perimeter; the method is new and not
  named in the plan (assumption 1).
- `TauriEvents` changed from a tuple struct to a struct with `app`, `alerts` and `focused`. The four existing test
  construction sites use `TauriEvents::silent(handle)`, which has no alerts. Same file, inside the perimeter.
- Tracker actor is written as `opus`, not `codex` as the prompt template says. I am Claude Opus, and C1's line set
  the same precedent.
- `features/sessions/__tests__/c2_session_badges.test.tsx` and the `useSessionBadges` hook live where the plan lists
  them. The hook is exported from `SessionBadge.tsx` rather than from a separate file.

## Files changed
**U1:**
- `crates/core/src/{domain/conversation.rs, mechanisms/conversation_rules.rs, ports/conversations.rs, services/conversations.rs}`
- `crates/infra/src/sqlite/conversations.rs`
- `apps/desktop/src-tauri/src/{commands/conversations.rs, lib.rs}`
- `apps/desktop/src/services/{bindings.ts (generated), conversations.ts}`
- `crates/testkit/tests/suite/main.rs`, plus the new file `crates/testkit/tests/suite/c2_conversations_lists.rs`

**U2:**
- New: `apps/desktop/src/features/conversations/{useConversations.ts, ConversationsButton.tsx, ConversationsPanel.tsx,
  ConversationRow.tsx, SessionBadge.tsx}`
- New tests: `apps/desktop/src/features/conversations/__tests__/{c2_header_icon.test.tsx, c2_panel_list.test.tsx}`,
  `apps/desktop/src/features/sessions/__tests__/c2_session_badges.test.tsx`
- Edited: `apps/desktop/src/App.tsx`, `apps/desktop/src/features/shell/Header.tsx`,
  `apps/desktop/src/features/health/HealthPage.tsx`, `apps/desktop/src/features/settings/SettingsPage.tsx`,
  `apps/desktop/src/features/sessions/{SessionsPage.tsx, SessionRow.tsx}`, `apps/desktop/src/styles/globals.css`

**U3:**
- New: `crates/infra/src/chime.rs`, `apps/desktop/src-tauri/src/alerts.rs`,
  `apps/desktop/src/features/conversations/ConversationToasts.tsx`,
  `apps/desktop/src/features/conversations/__tests__/c2_toasts.test.tsx`
- Edited: `crates/infra/src/lib.rs`, `apps/desktop/src-tauri/{Cargo.toml, src/lib.rs}`, `Cargo.lock`,
  `apps/desktop/src/App.tsx`, `apps/desktop/src/features/settings/PricingSection.tsx`,
  `docs/guide/{02_architecture.md, 04_change_impact.md}`

**Close:** the Tracker line and Review log in `docs/implementation/plans/C2_header_panel_notifications.md`, and this
handoff.

No sound file was added to the repo. No mutation-probe files, and no archgraph state.

## Checkpoint SHA
U1 `89809fe` · U2 `e3fd424` · U3 `2076b3d` · close: the `CHECKPOINT (not approved): C2 close` commit that holds this
handoff.

## Stamp
Closing gate, run once, one command after the other:

| Command | Result |
|---|---|
| `cargo test --workspace` | exit 0; 2035 passed, 0 failed (suite 1119/0, desktop 79/0) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `npm test` | 113 files, 709 tests passed |

No pre-existing failures. Per-unit commands were run before each checkpoint (results in the Review log).

## Manual try-out
For the owner on the installed app, never by a session. These are the plan's steps:
1. In Pricing, ask a blocking Condition question about a product photographed on this Mac. Within about 10 s you
   should get a chime, a toast "New question · SKU — Condition", the header icon showing 1, and the session's row on
   the Sessions page showing "Pricing is waiting".
2. Click the icon. The panel lists the conversation under Open, with the question's first line. Open that session:
   the panel now lists only its conversations.
3. Put Session Manager in the background and ask a second question. A Mac banner appears; the first time, macOS asks
   whether to allow notifications. Click it: Session Manager comes forward and the toast with Open is still there.
4. Press the mute button in the panel and ask again. There is no chime, but the toast still appears.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
