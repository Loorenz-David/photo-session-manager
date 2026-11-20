---
phase: C4_visit_and_quote_jump
title: Jump and back — "Open session" in a conversation opens that session as a visit, with the panel still open and the camera's new photos still going where they went before; a slim bar says so and offers "← Back" to where you were and "Shoot here instead"; in the session on screen, the quote's field chip opens the form at that field
difficulty: high
projection: yes
depends_on: [C3_conversation_and_answering]
blocked_on: none
---
# C4 — Jump and back

**Tracker:** NOT_STARTED (2026-11-20, planner)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `09B` §7 ("The jump and the way back", "Quote chip jump") by contract **MC-C3** (`docs/implementation/master_plan_conversations.md` §5). (U1) The visit: one state for route and visit, Open session, the bar, Back, Shoot here instead, and the proof that the capture folder does not move. (U2) The quote chip jump. **Not here:** any change to what a normal opening of a session does (it activates it exactly as today); remembering more of a session's screen than the app already remembers.

## How to work this plan (flow.md §10, master_plan_conversations.md §3)
One runner phase of 2 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `C4·C<n>(<row>)`; tests named after their row (`c4_c1a_<slug>`). Open details: the smallest option consistent with 09B §7 and MC-C3, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_conversations/C4_visit_and_quote_jump.md`. **Never call Pricing or Shopify, never launch the built app** (master plan §3).

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The visit, the bar, Back, Shoot here instead | 8 | 30 | **yes** |
| U2 | The quote chip jump | 4 | 14 | **yes** |
| **Total** | | **12** | **44** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §10) · `docs/implementation/master_plan_conversations.md` §4 (Frontend), **§5 MC-C3**, §6 · book `09B` §2 (decision 3), §7, §10 (rule 4), §12 (CV-2) · book `04` S-B and S-E (what opening a session and returning to the list do to the capture folder) · `apps/desktop/src/App.tsx` (**all of it**: `useRoute`, `useCaptureTarget`, `useResumeActiveSession`, every `navigate` caller) · `apps/desktop/src/lib/routes.ts` · `apps/desktop/src/services/sessions.ts` (`activateSession`, `deactivateSession`, `activeSession`) · the C3 plan, Unit 2.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **A visit never moves the camera's folder:** new photos keep going to the session you were shooting (or to Unassigned if you came from the Sessions page) until you press "Shoot here instead".
- **Back returns to where you were before the first jump,** also after jumping through several conversations.
- **Opening the session you are shooting from a conversation ends the visit,** exactly like Back.
- **Any ordinary navigation ends the visit** and behaves as it always has (opening a session that way points the camera at it).
- **A visit does not survive closing the app:** at the next start Session Manager opens the session the camera points at, as before.
- **The field chip is a link only for fields this Mac's form has for that product;** otherwise it is plain text.

## Unit 1 — The visit, the bar, Back, Shoot here instead

**Unit id:** `C4·U1` · **Rows:** C4·C1–C8 · **Changes app files:** yes

### Goal
`visit.ts` (`visitStep`, `captureTarget`), its use in `App.tsx` in place of `useRoute`'s separate state, **Open session** in the conversation header, `VisitBar`.

### Read first
Master plan §5 **MC-C3** (every item) · `apps/desktop/src/App.tsx` · `apps/desktop/src/features/shell/{Header.tsx (every place that writes `window.location.hash`), PageHeader.tsx}` · `apps/desktop/src/lib/{routes.ts (`goTo`, the leave guard), keyboard.ts (`useBackKeys`)}` · `apps/desktop/src/features/conversations/{ConversationView.tsx, useConversations.ts}` · `apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx` and `apps/desktop/src/features/sessions/__tests__/a19_review_queue.test.tsx` (how `App` navigation is tested with mocked services) · `apps/desktop/src-tauri/src/commands/sessions.rs` (`session_activate`, `session_deactivate`, `session_active`, `session_get`) · `crates/testkit/tests/suite/p05_capture_arrival.rs` (how a test makes a capture arrive in the active session).

### Files
New: `apps/desktop/src/features/conversations/{visit.ts, VisitBar.tsx}`, `apps/desktop/src/features/conversations/__tests__/{c4_visit_rules.test.ts, c4_visit.test.tsx}`, `crates/testkit/tests/suite/c4_conversations_visit.rs`. Edited: `apps/desktop/src/App.tsx`, `apps/desktop/src/features/conversations/{ConversationView.tsx, useConversations.ts}`, `apps/desktop/src/styles/globals.css`, `crates/testkit/tests/suite/main.rs`, `docs/guide/{01_product.md, 02_architecture.md, 03_rules_and_principles.md, 04_change_impact.md}`.

### Core mechanisms (the only functions with unit tests)
`visitStep(state, action)` and `captureTarget(state)` in `features/conversations/visit.ts` (pure; MC-C3.1–3).

### Ordered tasks
1. `visit.ts`: the types and the two functions of MC-C3.1–3, exactly; `routeOf(hash)` and `hashOf(route)` for the four routes.
2. `App.tsx`: one `useReducer(visitStep)` replaces `useRoute`'s three `useState`s. `navigate(next)` and every `hashchange` dispatch `route-changed`; the hash is written from the new state after a step, and a `hashchange` that only confirms the state's route dispatches nothing (MC-C3.4). `useCaptureTarget(captureTarget(state))`; every other consumer (`WorkspacePage`, `Header`, `useHistory`, `useHistoryKeys`, the review queue, `useConversations`) keeps using the route's session id. `openSession(id)`, `back()`, `shootHere()` are offered through the conversations context; `openSession` goes through the leave guard of `lib/routes.ts` first (a Settings page with unsaved changes can keep the user, and then nothing happens).
3. **Open session** in the conversation header line: shown when the conversation's `session_id` is set and is not the session on screen; a click calls `openSession(session_id)`. The panel's open state and selected conversation are untouched (MC-C3.6).
4. `VisitBar` under the header, shown iff a visit exists (MC-C3.5): the sentence, **← Back to <origin label>**, **Shoot here instead**. The capture session's SKU comes from `activeSession()` (re-read when the bar appears and on `session_changed`), the visited and origin SKUs from the sessions list.
5. Backend proof (no production change expected): a suite test that the reads a visit performs leave the active session alone (rows C4·C8).
6. Guide: the visit (01, 02), CR4 with `visit.ts` and `useCaptureTarget` (03), a change-impact row "anything that navigates to a session, or reads the route's session to decide where captures go → `visit.ts`" (04).
7. Tests for the rows.

### Criteria (8 criteria, 30 rows)
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
| C4·C3(b) | `App` on the Sessions page, the same click | — | session 7 is shown; no `activateSession` and no `deactivateSession` call at all | CR4 |
| C4·C3(c) | after (a) | — | the panel is still open and shows the same conversation | 09B §7.1; MC-C3.6 |
| C4·C4(a) | `VisitBar` after C4·C3(a) | sessions 3 (`BEYO-3`) and 7 (`BEYO-7`); `activeSession()` answers session 3 | "Visiting BEYO-7: new photos still go to BEYO-3." with "← Back to BEYO-3" and "Shoot here instead" | `[rule]` 09B §7.2 |
| C4·C4(b) | `VisitBar` after C4·C3(b) | `activeSession()` answers none | "Visiting BEYO-7: new photos still go to Unassigned." with "← Back to Sessions" | `[rule]` 09B §7.2 |
| C4·C4(c) | `App` in session 3, no visit | — | no bar | `[rule]` 09B §7.2 |
| C4·C5(a) | after C4·C3(a), click **← Back** | — | session 3 is shown; the bar is gone; no `activateSession` or `deactivateSession` call since the first click | CR4; 09B §7.3 |
| C4·C5(b) | after C4·C3(b), click **← Back** | — | the Sessions page is shown; no call | 09B §7.3 |
| C4·C5(c) | in session 3: Open session → 7, Open session (another conversation) → 9, **← Back** | — | session 3 is shown; no call during the three steps | 09B §7.3 |
| C4·C6(a) | after C4·C3(a), click **Shoot here instead** | — | `activateSession(7)` called exactly once; the bar is gone; session 7 still shown | 09B §7.4 |
| C4·C7(a) | after C4·C3(a), go to the Sessions page by the header's back link | — | the Sessions page; the bar is gone; `deactivateSession` called once (as when leaving an active session today) | 09B §7.5 |
| C4·C7(b) | after C4·C3(b), a normal navigation to session 5 (`navigate(5)`, as the sessions list does) | — | `activateSession(5)` called once; the bar is gone | 09B §7.5 |
| C4·C7(c) | `App`, no visit: open session 5 from the Sessions page, then return to the Sessions page | — | `activateSession(5)` once, then `deactivateSession` once (unchanged behaviour) | 09B §7.5; CR4 |
| C4·C8(a) | `TestApp` | session A activated; then `session_get(B)` semantics (the session view read), the conversations `view` of a conversation of B and `attachable` of it | the active session is still A | CR4 |
| C4·C8(b) | same | after (a), a capture arrives | its photo row belongs to session A | CR4; CV-2; M9 |

### Delegated choices
The bar's styling (slim, under the header, design handoff tokens) · whether the reducer lives in a hook (`useNav`) or inline in `App.tsx`.

### Notes
- Hazard (the heart of MC-C3): route and visit change in **one** step. With two separate states there is a render where the route is the visited session and the visit is not yet set (or already cleared), and `useCaptureTarget`'s effect activates the wrong session: new photos land in the product the photographer only looked at. Rows C4·C3(a), C4·C5(a) and C4·C5(c) assert the **whole** call list for that reason.
- Hazard: many places write `window.location.hash` directly (`Header.tsx`, the notifications drawer, `SessionActionsMenu`, `PageHeader`'s back). They are all ordinary navigation: `hashchange` → `route-changed`. Do not convert them.
- Hazard: `useResumeActiveSession` navigates once at launch; it is ordinary navigation and no visit exists then.
- Hazard: the existing navigation tests (`p19_navigation`, `a19_review_queue`, `a11_header_review`, `w3_writer_triggers`) must pass unchanged; a change to one of them is a signal that normal navigation changed.
- `visit.capture` is what the frontend believes; the bar shows what the backend reports (`activeSession()`), so a disagreement is visible instead of silent.

### Unit test commands
`cd apps/desktop && npx vitest run src/features/conversations src/features/shell src/features/sessions` · `npm run typecheck` · `cargo test -p psm-testkit --test suite c4_conversations_visit::`

## Unit 2 — The quote chip jump

**Unit id:** `C4·U2` · **Rows:** C4·C9–C12 · **Changes app files:** yes

### Goal
`ConversationsProvider::attribute_key`, `jump_key` in the conversation view, the chip as a link, the form opening at that field.

### Read first
Book `09B` §7 ("Quote chip jump") · `API.md` (field notes on `quote.field`) · master plan §4 (Ports) · `crates/integrations/src/conversations/beyo_pricing/adapter.rs` · `crates/core/src/services/{conversations.rs, attributes.rs (the form's fields and layout for an item; the built-in `product.title` and `product.description` definitions)}` · `apps/desktop/src/features/form/{FormDialog.tsx (`position`, `showField`, `reveal`), MetafieldInput.tsx (`reveal`), StepTabs.tsx}` · `apps/desktop/src/features/workspace/WorkspacePage.tsx` (`detailsStep` → `FormDialog`) · `apps/desktop/src/App.tsx` (`detailsOpen`, `detailsStep`) · `apps/desktop/src/features/conversations/QuoteBlock.tsx`.

### Files
Edited: `crates/core/src/{ports/conversations.rs, services/conversations.rs, domain/conversation.rs}`, `crates/integrations/src/conversations/beyo_pricing/adapter.rs`, `crates/testkit/tests/suite/c4_conversations_visit.rs`, `apps/desktop/src-tauri/src/commands/conversations.rs`, `apps/desktop/src/services/{bindings.ts (generated), conversations.ts}`, `apps/desktop/src/App.tsx`, `apps/desktop/src/features/workspace/WorkspacePage.tsx`, `apps/desktop/src/features/form/FormDialog.tsx`, `apps/desktop/src/features/conversations/{QuoteBlock.tsx, MessageBubble.tsx, ConversationView.tsx, useConversations.ts}`. New: `apps/desktop/src/features/conversations/__tests__/c4_quote_chip.test.tsx`, `apps/desktop/src/features/form/__tests__/c4_form_field.test.tsx`.

### Core mechanisms (the only functions with unit tests)
`BeyoPricingAdapter::attribute_key` (pure; inline tests in `adapter.rs`).

### Ordered tasks
1. `attribute_key(quote_field)`: `title` → `product.title`; `description` → `product.description`; `condition` → `custom.condition_report_description`; `mf:<namespace>.<key>` (a non-empty namespace without a dot, then a dot, then a non-empty key) → `<namespace>.<key>`; anything else → none.
2. The view's messages gain `quote.jump_key: Option<String>` = `attribute_key(quote.field)` when the conversation's session is on this Mac **and** the form of that session's item (its layout, all steps, as `AttributesService` builds it) has a field with that key; otherwise none. Command DTO, bindings.
3. `QuoteBlock`: the chip is a button iff `jump_key` is set **and** the conversation's session is the session on screen; a click calls the context's `openField(jump_key)`.
4. `App.tsx`: `openField(key)` sets `detailsField` and opens Details; `detailsField` is cleared when the form closes and when the session changes (as `detailsStep`). `WorkspacePage` passes it to `FormDialog` as `field`. `FormDialog`: when `field` is set on open (or changes while open), go to the step that holds that key and reveal it (the existing `showField` behaviour: `setStep`, `setReveal`); a field that has no reveal affordance (the title and description boxes) gets the step and the keyboard focus.
5. Tests for the rows.

### Criteria (4 criteria, 14 rows)
Boundary: `attribute_key` (pure); `ConversationsService::view` on `TestApp::with_pricing()`; `QuoteBlock` in the conversation view, and `FormDialog`, rendered in UI tests with services mocked.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C4·C9(a) | `attribute_key` | `title` | `product.title` | 09B §7 (field mapping) |
| C4·C9(b) | same | `description` | `product.description` | 09B §7 |
| C4·C9(c) | same | `condition` | `custom.condition_report_description` | 09B §7 |
| C4·C9(d) | same | `mf:beyo.designer`; `mf:beyo_pricing.material` | `beyo.designer`; `beyo_pricing.material` | 09B §7 |
| C4·C9(e) | same | `mf:designer`; `mf:.x`; `mf:beyo.`; `price`; `` | none for each | 09B §7 (anything else: plain text) |
| C4·C10(a) | `view` | a conversation of a session on this Mac; its first message quotes `mf:custom.designer`; the item's form has a field `custom.designer` | that quote's `jump_key` is `custom.designer` | 09B §7 ("if this Mac has it") |
| C4·C10(b) | same | the quote's field is `mf:custom.unknown_field` (no such field in the form) | `jump_key` none | 09B §7 |
| C4·C10(c) | same | the same quote as (a) on a conversation not on this Mac | `jump_key` none | 09B §7 |
| C4·C10(d) | same | the quote's field is `title` | `jump_key` `product.title` | 09B §7 |
| C4·C11(a) | `QuoteBlock` in the view | `jump_key` `custom.designer`; the conversation's `session_id` 7; session 7 on screen | the chip is a button; a click calls `openField("custom.designer")` once | `[rule]` 09B §7 |
| C4·C11(b) | same | the same quote; session 3 on screen | the chip is plain text | `[rule]` 09B §7 (when the conversation's session is the one on screen) |
| C4·C11(c) | same | `jump_key` null | the chip is plain text | `[rule]` 09B §7 |
| C4·C12(a) | `FormDialog` | opened with `field` `custom.designer`, a Details-step field | the Details step is shown and that field's input received the reveal (it is scrolled to: `scrollIntoView` called on its box) | `[rule]` 09B §7 (opens the form at that field) |
| C4·C12(b) | same | opened with `field` `custom.condition_report_description`, a Condition-step field; then with a key the form does not have | the Condition step is shown with the field revealed; then the form opens as without `field` | `[rule]` 09B §7 |

### Delegated choices
How a field without reveal gets the focus · whether `field` rides in `FormPosition` or is its own prop.

### Notes
- Hazard (H7): the mapping from Pricing's field names lives in the adapter; core only passes the string through the port.
- Hazard: `jump_key` says the field exists for that product; the chip is a link only when that product's session is the one on screen (the frontend's half). While visiting a session the chip therefore works for the visited session's conversations.
- Hazard: `FormDialog` must not jump again on every re-render: react to a change of `field`, as `reveal` does with its counter.

### Unit test commands
`cargo test -p psm-integrations --lib conversations::` · `cargo test -p psm-testkit --test suite c4_conversations_visit::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/conversations src/features/form src/features/workspace` · `npm run typecheck` · `cargo clippy -p psm-core -p psm-integrations -p psm-desktop -p psm-testkit --all-targets -- -D warnings`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cd apps/desktop && npx vitest run src/features/conversations src/features/shell src/features/sessions src/features/form` · `cargo test -p psm-integrations --lib conversations::` · `cargo test -p psm-testkit --test suite c4_conversations_visit::` · `cargo test -p psm-desktop` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading `App.tsx`: route and visit are one state changed by `visitStep` only; `useCaptureTarget` receives `captureTarget(state)` and nothing else calls `activateSession` or `deactivateSession`; no pre-existing navigation test was weakened. Notes go to `docs/implementation/backlog_conversations/C4_visit_and_quote_jump.md`.

## Manual try-out (the narrative is built from this; done by the owner on the installed app, never by a session)
1. Open session A and keep shooting. Have the pricer ask a question about product B. The icon in A shows a dot; open the panel, press "1 new in other sessions → All conversations", open the question, press **Open session**.
2. Session B opens; the panel still shows the question; a bar says "Visiting <B>: new photos still go to <A>." Take a photo: it lands in A.
3. Press **← Back to <A>**: you are in A again. Jump once more and press **Shoot here instead**: the bar goes away and the next photo lands in B.
4. In B, open a question that quotes the Condition text: click the CONDITION chip and the form opens at that field.

## Review log
