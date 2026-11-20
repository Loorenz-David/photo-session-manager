---
phase: C4_visit_and_quote_jump
title: Jump and back — "Open session" in a conversation opens that session as a visit, with the panel still open and the camera's new photos still going where they went before; a slim bar says so and offers "← Back" to where you were and "Shoot here instead"; in the session on screen, the quote's field chip opens the form at that field
difficulty: high
projection: done (r0 2026-11-20 AMENDMENTS_REQUIRED, amendments applied by the orchestrator; no new round)
depends_on: [C3_conversation_and_answering]
blocked_on: none
---
# C4 — Jump and back

**Tracker:** IMPLEMENTED (2026-11-20, claude (opus) implementer r1; 2 units, 46 rows: 31 new UI tests (15 rules, 15 App visit, 3 chip, 3 form — C3(d) as two), 6 suite, 5 adapter; full gate green: cargo 2082 passed, npm 784 passed)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `09B` §7 ("The jump and the way back", "Quote chip jump") by contract **MC-C3** (`docs/implementation/master_plan_conversations.md` §5). (U1) The visit: one state for route and visit, Open session, the bar, Back, Shoot here instead, and the proof that the capture folder does not move. (U2) The quote chip jump. **Not here:** any change to what a normal opening of a session does (it activates it exactly as today); remembering more of a session's screen than the app already remembers.

## How to work this plan (flow.md §10, master_plan_conversations.md §3)
One runner phase of 2 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `C4·C<n>(<row>)`; tests named after their row (`c4_c1a_<slug>`). Open details: the smallest option consistent with 09B §7 and MC-C3, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_conversations/C4_visit_and_quote_jump.md`. **Never call Pricing or Shopify, never launch the built app** (master plan §3).

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The visit, the bar, Back, Shoot here instead | 8 | 31 | **yes** |
| U2 | The quote chip jump | 4 | 15 | **yes** |
| **Total** | | **12** | **46** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §10) · `docs/implementation/master_plan_conversations.md` §4 (Frontend), **§5 MC-C3**, §6 · book `09B` §2 (decision 3), §7, §10 (rule 4), §12 (CV-2) · book `04` S-B and S-E (what opening a session and returning to the list do to the capture folder) · `apps/desktop/src/App.tsx` (**all of it**: `useRoute`, `useCaptureTarget`, `useResumeActiveSession`, every `navigate` caller) · `apps/desktop/src/lib/routes.ts` · `apps/desktop/src/services/sessions.ts` (`activateSession`, `deactivateSession`, `activeSession`) · the C3 plan, Unit 2.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **A visit never moves the camera's folder:** new photos keep going to the session you were shooting (or to Unassigned if you came from the Sessions page) until you press "Shoot here instead".
- **Back returns to where you were before the first jump,** also after jumping through several conversations.
- **From Settings with unsaved changes, Open session asks "Save your changes?" first;** Discard or Save then opens the session as a visit (the camera's folder still does not move), Cancel keeps you in Settings.
- **Opening the session you are shooting from a conversation ends the visit,** exactly like Back.
- **Any ordinary navigation ends the visit** and behaves as it always has (opening a session that way points the camera at it).
- **A visit does not survive closing the app:** at the next start Session Manager opens the session the camera points at, as before.
- **The field chip is a link only for fields this Mac's form has for that product;** otherwise it is plain text.

## Unit 1 — The visit, the bar, Back, Shoot here instead

**Unit id:** `C4·U1` · **Rows:** C4·C1–C8 · **Changes app files:** yes

### Goal
`visit.ts` (`visitStep`, `captureTarget`), its use in `App.tsx` in place of `useRoute`'s separate state, **Open session** in the conversation header, `VisitBar`.

### Read first
Master plan §5 **MC-C3** (every item) · `apps/desktop/src/App.tsx` · `apps/desktop/src/features/shell/{Header.tsx (every place that writes `window.location.hash`), PageHeader.tsx}` · `apps/desktop/src/lib/{routes.ts (`goTo`, the leave guard), keyboard.ts (`useBackKeys`)}` · `apps/desktop/src/features/settings/SettingsPage.tsx` (the leave guard it installs, :81-87, and the "Save your changes?" dialog's Discard / Save, :226-239) and `features/settings/__tests__/p63_settings_save_and_style.test.tsx` · `apps/desktop/src/features/conversations/{ConversationView.tsx, useConversations.ts}` · `apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx` and `apps/desktop/src/features/sessions/__tests__/a19_review_queue.test.tsx` (how `App` navigation is tested with mocked services) · `apps/desktop/src-tauri/src/commands/sessions.rs` (`session_activate`, `session_deactivate`, `session_active`, `session_get`) · `crates/testkit/tests/suite/p05_capture_arrival.rs` (how a test makes a capture arrive in the active session).

### Files
New: `apps/desktop/src/features/conversations/{visit.ts, VisitBar.tsx}`, `apps/desktop/src/features/conversations/__tests__/{c4_visit_rules.test.ts, c4_visit.test.tsx}`, `crates/testkit/tests/suite/c4_conversations_visit.rs`. Edited: `apps/desktop/src/App.tsx`, `apps/desktop/src/lib/routes.ts`, `apps/desktop/src/features/settings/SettingsPage.tsx`, `apps/desktop/src/features/conversations/{ConversationView.tsx, useConversations.ts}`, `apps/desktop/src/styles/globals.css`, `crates/testkit/tests/suite/main.rs`, `docs/guide/{01_product.md, 02_architecture.md, 03_rules_and_principles.md, 04_change_impact.md}`.

### Core mechanisms (the only functions with unit tests)
`visitStep(state, action)` and `captureTarget(state)` in `features/conversations/visit.ts` (pure; MC-C3.1–3).

### Ordered tasks
1. `visit.ts`: the types and the two functions of MC-C3.1–3, exactly; `routeOf(hash)` and `hashOf(route)` for the four routes.
2. `App.tsx`: one `useReducer(visitStep)` replaces `useRoute`'s three `useState`s. `navigate(next)` and every `hashchange` dispatch `route-changed`; a `hashchange` that only confirms the state's route dispatches nothing (MC-C3.4). **Hash and render (projection L2):** `window.location.hash` already equals `hashOf(next.route)` when `next` renders. A step computes `next = visitStep(current, action)` from a ref of the current state, writes the hash, then dispatches. The `hashchange` handler reads `window.location.hash` when it runs (never `event.newURL`), so a late event from an earlier step is only a confirmation. `navigate`, `openSession`, `visitBack`, `shootHere` keep a stable identity (so `useResumeActiveSession` does not re-run). No hash is written at mount; an unknown hash routes to Sessions, as today. `useCaptureTarget(captureTarget(state))`; every other consumer (`WorkspacePage`, `Header`, `useHistory`, `useHistoryKeys`, the review queue, `useConversations`) keeps using the route's session id. `openSession(id)`, `visitBack()` (named so because `ConversationsState.back` already means "back to the list", projection L11), `shootHere()` are offered through the conversations context. **The leave guard (projection L1):** `lib/routes.ts`'s guard gets the continuation as well as the target, `LeaveGuard = (target: string, proceed: () => void) => boolean`, and `lib/routes.ts` exports `leaveThen(target, proceed)`: no guard, or the guard returns true → `proceed()` now; the guard returns false → nothing now (the guard keeps `proceed`). `goTo(hash)` = `leaveThen(hash, () => { window.location.hash = hash; })`. `openSession(id)` = `leaveThen(hashOf(session id), () => step(open-session(id)))`, so it never writes the hash before the guard has agreed. `SettingsPage` stores the target and `proceed` when its guard refuses; Discard and Save (after a successful save) remove the guard and call the stored `proceed` instead of writing `window.location.hash`; Cancel drops it. The continuation steps from the ref's state at the time it runs (route Settings, no visit), so the visit's origin is Settings and its capture is Unassigned.
3. **Open session** in the conversation header line: shown when the conversation's `session_id` is set and is not the session on screen; a click calls `openSession(session_id)`. The panel's open state and selected conversation are untouched (MC-C3.6).
4. `VisitBar` under the header, shown iff a visit exists (MC-C3.5): the sentence, **← Back to <origin label>**, **Shoot here instead**. The capture session is the id from `activeSession()` (re-read when the bar appears and on `session_changed`; null → `Unassigned`), its SKU from the sessions-list row with that id (projection L8: `SessionResponse` has no `sku`); the visited and origin SKUs from the sessions list. An origin that is the Unassigned workspace (session 1, no item and no list row) is labelled "← Back to Unassigned"; a session whose list row has not loaded yet shows an empty label until it arrives (projection L9).
5. Backend proof (no production change expected): a suite test that the reads a visit performs leave the active session alone (rows C4·C8).
6. Guide: the visit (01, 02), CR4 with `visit.ts` and `useCaptureTarget` (03), a change-impact row "anything that navigates to a session, or reads the route's session to decide where captures go → `visit.ts`" (04).
7. Tests for the rows.

### Criteria (8 criteria, 31 rows)
Boundary: `visitStep` / `captureTarget` (pure); `App` rendered in UI tests with `services/sessions.ts`, `services/conversations.ts` and the other services mocked (outcome = the route shown, the bar, and the exact list of `activateSession` / `deactivateSession` calls); `TestApp` for C4·C8.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C4·C1(a) | `visitStep` | route session 3, no visit; `open-session(7)` | route session 7; visit `{origin: session 3, capture: 3, visited: 7}` | 09B §7.1; MC-C3.3 |
| C4·C1(b) | same | route sessions, no visit; `open-session(7)` | route session 7; visit `{origin: sessions, capture: undefined, visited: 7}` | 09B §7.1 |
| C4·C1(c) | same | route settings, no visit; `open-session(7)` | route session 7; visit `{origin: settings, capture: undefined, visited: 7}` | 09B §7.3 |
| C4·C1(d) | same | visiting 7 from session 3; `open-session(9)` | route session 9; visit `{origin: session 3, capture: 3, visited: 9}` | 09B §7.3 (jumping again keeps the origin) |
| C4·C1(e) | same | visiting 7 from session 3; `open-session(3)` | route session 3; visit null | MC-C3.3 |
| C4·C1(f) | same | visiting 9 from session 3 (after two jumps); `back` | route session 3; visit null | 09B §7.3 |
| C4·C1(g) | same | visiting 7 from sessions; `back` | route sessions; visit null | 09B §7.3 |
| C4·C1(h) | same | visiting 7 from session 3; `shoot-here` | route session 7; visit null | 09B §7.4 |
| C4·C1(i) | same | visiting 7 from session 3; `route-changed(session 7)` | unchanged | MC-C3.3 |
| C4·C1(j) | same | visiting 7 from session 3; `route-changed(sessions)` | route sessions; visit null | 09B §7.5 |
| C4·C1(k) | same | visiting 7 from session 3; `route-changed(session 5)` | route session 5; visit null | 09B §7.5 |
| C4·C1(l) | same | route session 3, no visit; `back`; `shoot-here`; `open-session(3)` | unchanged for each | MC-C3.3 |
| C4·C2(a) | `captureTarget` | visiting 7 from session 3 | 3 | MC-C3.2 |
| C4·C2(b) | same | visiting 7 from sessions | undefined | MC-C3.2 |
| C4·C2(c) | same | no visit, route session 5; no visit, route sessions; no visit, route settings | 5; undefined; undefined | MC-C3.2 (today's behaviour) |
| C4·C3(a) | `App` in session 3 (`activateSession(3)` already made), the panel open on a conversation of session 7; click **Open session** | — | the workspace of session 7 is shown; since the click neither `activateSession` nor `deactivateSession` was called | CR4; 09B §7.1; CV-2 |
| C4·C3(b) | `App` on the Sessions page, the same click | — | session 7 is shown, and the header's back link to Sessions is shown; no `activateSession` and no `deactivateSession` call at all | CR4; MC-C3.4 |
| C4·C3(c) | after (a) | — | the panel is still open and shows the same conversation | 09B §7.1; MC-C3.6 |
| C4·C3(d) | `App` on Settings with unsaved changes (dirty as in `p63_settings_save_and_style`), the panel on a conversation of session 7; click **Open session** | — | the "Save your changes?" dialog, Settings still shown; then **Discard** → session 7 is shown with the bar; no `activateSession` or `deactivateSession` call at all (and, on a second run, **Cancel** → Settings stays, no call) | CR4; 09B §10 rule 4; MC-C3 invariant |
| C4·C4(a) | `VisitBar` after C4·C3(a) | `listSessions` mocked with rows 3 (`BEYO-3`) and 7 (`BEYO-7`); `activeSession()` answers `{id: 3, …}` | "Visiting BEYO-7: new photos still go to BEYO-3." with "← Back to BEYO-3" and "Shoot here instead" | `[rule]` 09B §7.2 |
| C4·C4(b) | `VisitBar` after C4·C3(b) | `activeSession()` answers none | "Visiting BEYO-7: new photos still go to Unassigned." with "← Back to Sessions" | `[rule]` 09B §7.2 |
| C4·C4(c) | `App` in session 3, no visit | — | no bar | `[rule]` 09B §7.2 |
| C4·C5(a) | after C4·C3(a), click **← Back** | — | session 3 is shown; the bar is gone; no `activateSession` or `deactivateSession` call since the first click | CR4; 09B §7.3 |
| C4·C5(b) | after C4·C3(b), click **← Back** | — | the Sessions page is shown; no call | 09B §7.3 |
| C4·C5(c) | in session 3: Open session → 7, Open session (another conversation) → 9, **← Back** | — | session 3 is shown; no call during the three steps | 09B §7.3 |
| C4·C6(a) | after C4·C3(a), click **Shoot here instead** | — | `activateSession(7)` called exactly once; the bar is gone; session 7 still shown | 09B §7.4 |
| C4·C7(a) | after C4·C3(a), go to the Sessions page by the header's back link | — | the Sessions page; the bar is gone; `deactivateSession` called once (as when leaving an active session today) | 09B §7.5 |
| C4·C7(b) | after C4·C3(b), the notifications drawer's **Open** for session 5 (an ordinary hash navigation, MC-C3.3; publish notifications mocked as in `a11_header_review`) | — | `activateSession(5)` called once; the bar is gone | 09B §7.5 |
| C4·C7(c) | `App`, no visit: open session 5 from the Sessions page, then return to the Sessions page | — | `activateSession(5)` once, then `deactivateSession` once (unchanged behaviour) | 09B §7.5; CR4 |
| C4·C8(a) | `TestApp` | session A activated; then `session_get(B)` semantics (the session view read), the conversations `view` of a conversation of B and `attachable` of it | the active session is still A | CR4 |
| C4·C8(b) | same | after (a), a capture arrives | its photo row belongs to session A | CR4; CV-2; M9 |

### Delegated choices
The bar's styling (slim, under the header, design handoff tokens) · whether the reducer lives in a hook (`useNav`) or inline in `App.tsx`. Decided in writing (projection): the origin label of the Unassigned workspace is "Unassigned" (L9); the panel's list scope keeps following the session on screen, as today (`useConversations.ts:89`); only the open state and the selected conversation are carried through a jump (C4·C3(c), L10).

### Notes
- Hazard (the heart of MC-C3): route and visit change in **one** step. With two separate states there is a render where the route is the visited session and the visit is not yet set (or already cleared), and `useCaptureTarget`'s effect activates the wrong session: new photos land in the product the photographer only looked at. Rows C4·C3(a), C4·C5(a) and C4·C5(c) assert the **whole** call list for that reason.
- Hazard: many places write `window.location.hash` directly (`Header.tsx`, the notifications drawer, `SessionActionsMenu`, `PageHeader`'s back). They are all ordinary navigation: `hashchange` → `route-changed`. Do not convert them. The one exception is `SettingsPage`'s Discard / Save (`:232`, `:238`): they write the hash today and must call the stored continuation instead (task 2), or an Open session from a dirty Settings page becomes an ordinary opening and moves the capture folder (C4·C3(d)).
- Hazard: `PageHeader` reads `window.location.hash` during render (`PageHeader.tsx:14-16`); a hash written in an effect after the render leaves a visit without its back link and Back keys (C4·C3(b)).
- Hazard: `useResumeActiveSession` navigates once at launch; it is ordinary navigation and no visit exists then.
- Hazard: the existing navigation tests (`p19_navigation`, `a19_review_queue`, `a11_header_review`, `w3_writer_triggers`) and the Settings leave-guard tests (`p63_settings_save_and_style`, `p68_setup_polish`) must pass unchanged; a change to one of them is a signal that normal navigation changed.
- `visit.capture` is what the frontend believes; the bar shows what the backend reports (`activeSession()`), so a disagreement is visible instead of silent.

### Unit test commands
`cd apps/desktop && npx vitest run src/features/conversations src/features/shell src/features/sessions src/features/settings` · `npm run typecheck` · `cargo test -p psm-testkit --test suite c4_conversations_visit::`

## Unit 2 — The quote chip jump

**Unit id:** `C4·U2` · **Rows:** C4·C9–C12 · **Changes app files:** yes

### Goal
`ConversationsProvider::attribute_key`, the form-keys read and the integration name the service needs for it, `jump_key` in the conversation view (never stored), the chip as a link, the form opening at that field.

### Read first
Book `09B` §7 ("Quote chip jump") · `API.md` (field notes on `quote.field`) · master plan §4 (Ports, Settings, Service: constructor) · `crates/integrations/src/conversations/beyo_pricing/adapter.rs` · `crates/core/src/services/{conversations.rs, attributes.rs (`active_definitions` :131-140, the integration definitions :87-127, `layout_keys` :228-251, `form_get` :1515-1566; the built-in `product.title` and `product.description` definitions)}` · `crates/infra/src/sqlite/conversations.rs` (`session_links`, the C3 read, as the model for the new one) · `crates/integrations/src/conversations/mod.rs` (`ServiceParts`, `build_service`) and `crates/integrations/src/lib.rs:1179-1187` (where `shop` is filled from the settings integration block) · `crates/testkit/src/test_app.rs` (`with_pricing`, :595-601) · `apps/desktop/src/features/form/AiPopover.tsx` (`openSignal`) · `apps/desktop/src/features/form/{FormDialog.tsx (`position`, `showField`, `reveal`), MetafieldInput.tsx (`reveal`), StepTabs.tsx}` · `apps/desktop/src/features/workspace/WorkspacePage.tsx` (`detailsStep` → `FormDialog`) · `apps/desktop/src/App.tsx` (`detailsOpen`, `detailsStep`) · `apps/desktop/src/features/conversations/QuoteBlock.tsx`.

### Files
Edited: `crates/core/src/{ports/conversations.rs, services/conversations.rs, services/attributes.rs, domain/conversation.rs}`, `crates/infra/src/sqlite/conversations.rs`, `crates/integrations/src/conversations/{mod.rs, beyo_pricing/adapter.rs}`, `crates/integrations/src/lib.rs`, `crates/testkit/tests/suite/c4_conversations_visit.rs`, `apps/desktop/src-tauri/src/commands/conversations.rs`, `apps/desktop/src/services/{bindings.ts (generated), conversations.ts}`, `apps/desktop/src/App.tsx`, `apps/desktop/src/features/workspace/WorkspacePage.tsx`, `apps/desktop/src/features/form/FormDialog.tsx`, `apps/desktop/src/features/conversations/{QuoteBlock.tsx, MessageBubble.tsx, ConversationView.tsx, useConversations.ts}`, `docs/guide/{02_architecture.md, 04_change_impact.md}`. New: `apps/desktop/src/features/conversations/__tests__/c4_quote_chip.test.tsx`, `apps/desktop/src/features/form/__tests__/c4_form_field.test.tsx`.

### Core mechanisms (the only functions with unit tests)
`BeyoPricingAdapter::attribute_key` (pure; inline tests in `adapter.rs`).

### Ordered tasks
1. `attribute_key(quote_field)`: `title` → `product.title`; `description` → `product.description`; `condition` → `custom.condition_report_description`; `mf:<namespace>.<key>` (a non-empty namespace without a dot, then a dot, then a non-empty key) → `<namespace>.<key>`; anything else → none.
2. **The form's keys (projection L3, road (a)).** (i) `pub fn form_keys(tx: &dyn StoreTx, item_type: &str, integration: Option<&str>) -> Result<BTreeSet<String>>` in `services/attributes.rs` beside `layout_keys`: the keys of `layout_keys` whose definition is active (`active_definitions(tx, integration)`), i.e. exactly the fields `form_get` shows over all three steps (a preset key without an active definition is not shown, so it gets no link); one copy of the layout rules, no second one in infra. (ii) One new `ConversationStore` read, `form_keys(session_id, integration: Option<&str>) -> Result<BTreeSet<String>>`: the field keys of the form of session S's item for integration I (empty when the session has no item, e.g. Unassigned), implemented in `crates/infra/src/sqlite/conversations.rs` in one read transaction that finds the item and calls (i). (iii) `ServiceParts` and `ConversationsService::new` gain `integration: Option<String>`, filled in `crates/integrations/src/lib.rs` from the same settings integration block that gives `shop` (`integration.as_ref().map(|i| i.kind.clone())`; an integration change already rebuilds the App). `TestApp::with_pricing()` already writes `kind: "shopify"`.
   **`jump_key` (projection L4).** `view` computes, per message, `jump_key = attribute_key(quote.field)` when the conversation's session is on this Mac **and** `form_keys(session, integration)` contains that key; otherwise none. It is computed in `view` only and **never stored**: `ConversationView` gains a view-only `jump_keys: BTreeMap<String, String>` (message id → key, only the messages that have one); the persisted domain `Quote` and `Message` are unchanged (the local copy's JSON stays as C1 wrote it), and `ConversationViewDto` maps the map into each message DTO's `quote.jump_key: Option<String>`. Command DTO, bindings.
3. `QuoteBlock`: the chip is a button iff `jump_key` is set **and** the conversation's session is the session on screen; a click calls the context's `openField(jump_key)`.
4. `App.tsx`: `openField(key)` sets `detailsField` and opens Details; `detailsField` is cleared when the form closes and when the session changes (as `detailsStep`). `WorkspacePage` passes it to `FormDialog` as `field`. `FormDialog`: when `field` is set on open, or on every later `openField` call while open (a counter beside the key, like `reveal`; projection L12: every chip click jumps, also a second click on the same chip), go to the step that holds that key and reveal it (the existing `showField` behaviour: `setStep`, `setReveal`). A set `field` counts as a position for the A19 landing: the form does not move to AI review when suggestions wait (projection L5; `FormDialog.tsx:252-273`). A key the form does not have opens the form as without `field`. The chip jump uses the existing reveal unchanged, including opening the AI popover of an AI-valued field (projection L6; title and description render through `MetafieldInput` like every field and are revealed the same way).
5. Guide (projection L13): `attribute_key` and the form-keys read in `02_architecture.md` (beside the `ConversationsProvider` methods, :413-414); a change-impact row "Pricing's quote field names → `BeyoPricingAdapter::attribute_key`" in `04_change_impact.md`.
6. Tests for the rows.

### Criteria (4 criteria, 15 rows)
Boundary: `attribute_key` (pure); `ConversationsService::view` on `TestApp::with_pricing()`; `QuoteBlock` in the conversation view, and `FormDialog`, rendered in UI tests with services mocked.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C4·C9(a) | `attribute_key` | `title` | `product.title` | 09B §7 (field mapping) |
| C4·C9(b) | same | `description` | `product.description` | 09B §7 |
| C4·C9(c) | same | `condition` | `custom.condition_report_description` | 09B §7 |
| C4·C9(d) | same | `mf:beyo.designer`; `mf:beyo_pricing.material` | `beyo.designer`; `beyo_pricing.material` | 09B §7 |
| C4·C9(e) | same | `mf:designer`; `mf:.x`; `mf:beyo.`; `price`; `` | none for each | 09B §7 (anything else: plain text) |
| C4·C10(a) | `view` | `TestApp::with_pricing()`; a `custom.designer` definition imported for `shopify` (in the Details step), then the session's item created; a conversation of that session on this Mac; its first message quotes `mf:custom.designer` | that quote's `jump_key` is `custom.designer` | 09B §7 ("if this Mac has it") |
| C4·C10(b) | same | the quote's field is `mf:custom.unknown_field` (no such field in the form) | `jump_key` none | 09B §7 |
| C4·C10(c) | same | the same quote as (a) on a conversation not on this Mac | `jump_key` none | 09B §7 |
| C4·C10(d) | same | the quote's field is `title` | `jump_key` `product.title` | 09B §7 |
| C4·C11(a) | `QuoteBlock` in the view | `jump_key` `custom.designer`; the conversation's `session_id` 7; session 7 on screen | the chip is a button; a click calls `openField("custom.designer")` once | `[rule]` 09B §7 |
| C4·C11(b) | same | the same quote; session 3 on screen | the chip is plain text | `[rule]` 09B §7 (when the conversation's session is the one on screen) |
| C4·C11(c) | same | `jump_key` null | the chip is plain text | `[rule]` 09B §7 |
| C4·C12(a) | `FormDialog` | an item with no waiting suggestion; opened with `field` `custom.designer`, a Details-step field | the Details step is shown and that field's input received the reveal (it is scrolled to: `scrollIntoView` called on its box) | `[rule]` 09B §7 (opens the form at that field) |
| C4·C12(b) | same | an item with no waiting suggestion; opened with `field` `custom.condition_report_description`, a Condition-step field; then closed, and reopened with `field` `custom.unknown` | the Condition step is shown with the field revealed; then the Details step, nothing revealed | `[rule]` 09B §7 |
| C4·C12(c) | same | an item with one waiting AI suggestion; opened with `field` `custom.designer` | the Details step with that field revealed, not AI review | `[rule]` 09B §7; A19 landing |

### Delegated choices
Whether `field` rides in `FormPosition` or is its own prop. Decided in writing (projection): a chip jump uses the existing reveal unchanged, including opening the AI popover of an AI-valued field (L6); every `openField` call jumps (a counter, like `reveal`; L12).

### Notes
- Hazard (H7): the mapping from Pricing's field names lives in the adapter; core only passes the string through the port.
- Hazard: `jump_key` says the field exists for that product; the chip is a link only when that product's session is the one on screen (the frontend's half). While visiting a session the chip therefore works for the visited session's conversations.
- Hazard: `FormDialog` must not jump again on every re-render: react to a new `openField` call (its counter), as `reveal` does with its counter; a re-render with the same counter does nothing.
- Hazard: `jump_key` is a view-time derivation; it must never reach the local copy (`conversation_messages.quote` JSON). A store round-trip of a quoted message is unchanged by U2.

### Unit test commands
`cargo test -p psm-integrations --lib conversations::` · `cargo test -p psm-testkit --test suite c4_conversations_visit::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/conversations src/features/form src/features/workspace` · `npm run typecheck` · `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-desktop -p psm-testkit --all-targets -- -D warnings`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cd apps/desktop && npx vitest run src/features/conversations src/features/shell src/features/sessions src/features/settings src/features/form` · `cargo test -p psm-integrations --lib conversations::` · `cargo test -p psm-testkit --test suite c4_conversations_visit::` · `cargo test -p psm-desktop` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading `App.tsx`: route and visit are one state changed by `visitStep` only; `useCaptureTarget` receives `captureTarget(state)` and nothing else calls `activateSession` or `deactivateSession`; no pre-existing navigation or Settings leave-guard test was weakened; `SettingsPage`'s Discard / Save call the stored continuation and write no hash themselves; `jump_key` is not written by the store. Notes go to `docs/implementation/backlog_conversations/C4_visit_and_quote_jump.md`.

## Manual try-out (the narrative is built from this; done by the owner on the installed app, never by a session)
1. Open session A and keep shooting. Have the pricer ask a question about product B. The icon in A shows a dot; open the panel, press "1 new in other sessions → All conversations", open the question, press **Open session**.
2. Session B opens; the panel still shows the question; a bar says "Visiting <B>: new photos still go to <A>." Take a photo: it lands in A.
3. Press **← Back to <A>**: you are in A again. Jump once more and press **Shoot here instead**: the bar goes away and the next photo lands in B.
4. In B, open a question that quotes the Condition text: click the CONDITION chip and the form opens at that field.

## Review log

- **2026-11-20, orchestrator: projection r0 amendments applied** (`handoffs/reviewer/handoff_C4_visit_and_quote_jump_projection_r0.md`, AMENDMENTS_REQUIRED, 0 owner cards; master plan v1.4). L1 the leave guard receives the continuation (`LeaveGuard = (target, proceed) => boolean`, `leaveThen(target, proceed)` in `lib/routes.ts`; `goTo`'s continuation writes the hash, `openSession`'s dispatches `open-session`; `SettingsPage`'s Discard / Save call the stored continuation instead of writing the hash); `lib/routes.ts` and `SettingsPage.tsx` added to U1 Files; row **C4·C3(d)** added (dirty Settings → dialog; Discard → session 7 with the bar, no activate/deactivate call; Cancel → Settings stays) (U1 task 2; master plan MC-C3.3). L2 the hash is written before the dispatch from a ref of the state, the handler reads the hash when it runs, stable callback identities, no hash at mount (U1 task 2; MC-C3.4); C4·C3(b) also asserts the header's back link to Sessions. L3 road (a), as recommended: core `form_keys` beside `layout_keys` (active definitions only, `form_get` semantics, one copy of the layout rules), one `ConversationStore` read `form_keys(session_id, integration)` in `crates/infra/src/sqlite/conversations.rs`, `integration: Option<String>` in `ServiceParts` / `ConversationsService::new` from `settings.integration.kind` (`crates/integrations/src/lib.rs`); C4·C10(a)'s fixture imports a `custom.designer` definition for `shopify` (master plan §4 Ports, Settings, constructor). Not road (b): it changes `view`'s signature and callers and moves work into a command. L4 `jump_key` view-only: `ConversationView.jump_keys` (message id → key), mapped by `ConversationViewDto` into `quote.jump_key`; the persisted `Quote` / `Message` are unchanged. Chosen over a `#[serde(skip)]` domain field because it leaves the stored type untouched, so the copy cannot ever carry it (smallest surface; master plan §4 Commands). L5 a set `field` counts as a position for the A19 landing (U2 task 4); C4·C12(b) now "closed, then reopened with `custom.unknown`, no waiting suggestion → Details, nothing revealed"; the optional row **C4·C12(c)** added (one waiting suggestion, `field` `custom.designer` → Details with the field revealed, not AI review), because otherwise the L5 rule has no row. L6 the false "no reveal affordance" sentence and its delegated choice deleted; delegated: the chip jump uses the existing reveal unchanged, including the AI popover. L7 C4·C7(b)'s trigger = the notifications drawer's **Open** for session 5 (the first option; an ordinary hash navigation through `hashchange`, the path MC-C3.3 lists, which is the one that must end the visit). L8 the capture session = `activeSession()`'s id, its SKU from the sessions-list row (U1 task 4; MC-C3.5); C4·C4(a)'s fixture mocks `listSessions` and `activeSession()` = `{id: 3, …}`. L9 delegated: "← Back to Unassigned"; a not-yet-loaded row shows an empty label (MC-C3.5). L10 delegated: the list scope keeps following the session on screen (MC-C3.6 aligned). L11 the visit's back is `visitBack` in the context. L12 delegated: every `openField` call jumps (a counter). L13 U2 task 5 (guide 02, 04) and the guide files in U2 Files. Also: U1 Read first names `SettingsPage.tsx` and `p63`; the Settings leave-guard tests join the must-pass-unchanged list; U1's vitest command and Review mode include `src/features/settings`; U2 clippy includes `psm-infra`. Counts: U1 8 criteria / 31 rows, U2 4 / 15; total 12 / 46 (recounted from the four plan files: 12 units, 73 criteria, 299 rows). Outcomes otherwise unchanged, so no new projection round (`projection: done`).

- **2026-11-20, implementer (claude opus) r1: IMPLEMENTED.** U1 `816fdc4`, U2 `4955b51`. *Built:* `visit.ts` (`visitStep`, `captureTarget`, `routeOf`, `hashOf`) as the one route+visit state in `App.tsx` (`useNav`: next state from a ref, hash written, then dispatch; `hashchange` reads the hash and ignores a confirmation; stable callbacks; `useCaptureTarget(captureTarget(state))`); `lib/routes.ts` `LeaveGuard = (target, proceed)` + `leaveThen`, `SettingsPage` Discard / Save call the stored `proceed`; **Open session** in the header line; `VisitBar` (capture SKU from `activeSession()` + the sessions list, re-read on `session_changed`); suite proof C4·C8. U2: `BeyoPricingAdapter::attribute_key`; core `attributes::form_keys` (= `layout_keys` ∩ active definitions); `ConversationStore::form_keys` (infra, one read transaction); `integration` in `ServiceParts` / `ConversationsService::new` from `settings.integration.kind`; view-only `ConversationView.jump_keys` → `QuoteDto.jump_key`; `QuoteBlock` button iff `jump_key` and the conversation's session is on screen; `openField` → `detailsField {key, n}` → `FormDialog` `field` (each new `n` jumps via the existing `setStep` + `setReveal`; a field the form has skips the A19 landing). Guide 01–04 updated. *Assumptions:* (1) the bar's capture label is empty until `activeSession()` answers, null or session 1 → "Unassigned"; (2) the bar renders only in the sessions/session branch (a visit always has a session route); (3) every "unchanged" step returns the same state object; (4) a step writes the hash only when the hash does not already represent the next route; `navigate` still bypasses the leave guard as before; (5) `jump_key` is skipped on the wire when none (`skip_serializing_if`, as P69's `masters_ready`), so the TS field is optional and C3's fixtures stay valid; (6) a `field` key the form lacks behaves as no `field` (A19 landing still applies); (7) the C4·C8(b) capture is a hook-delivered RAW in the fake Capture One's current capture folder; (8) C4·C10's definition comes from `TestApp::research_fields`; (9) styling: `.visit-bar` sticky under the header with a light `--warn` tint, `.quote-chip-link` underlined; (10) `mf:` splits at the first dot. *Deviations:* `globals.css` also changed in U2 (one rule; it is in U1's list); `services/conversations.ts` not edited (types come from bindings); actor recorded as claude (opus), not codex. *For the reviewer:* with the pre-C4 `useCaptureTarget(sessionId)` restored temporarily, 8 of 15 `c4_visit` tests go red (restored); C4·C12(c) stays green without the landing skip because the jump effect runs after the landing effect, so the skip only guards against reordering. No pre-existing navigation or leave-guard test was edited.

