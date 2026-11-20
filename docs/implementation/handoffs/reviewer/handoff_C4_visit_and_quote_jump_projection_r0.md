---
plan: C4_visit_and_quote_jump
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-11-20
actor: pipeline-projectionist (Claude Opus 5.5)
---
# Projection — C4_visit_and_quote_jump (round 0)

The plan can be built, but not exactly as written: before implementation it needs a few paragraph-sized fixes. The most
important one protects the plan's main promise. If you open a session from a conversation while Settings has unsaved
changes, then press Discard or Save, today's code would carry on as an ordinary opening and move the camera's folder,
which is exactly what the visit must never do. The second fix is in the quote-chip half: the plan asks the
conversations service to check "does this product's form have this field", but the service has no way to see the
form today, so the plan must say how it gets that information. Nothing needs you: every point below is a planning or
wording fix the orchestrator can make, and implementation can start once they are folded in.

## ⚠ OWNER DECISIONS REQUIRED (0)

Nothing needs the owner. Every ledger row is a plan gap or a free choice the orchestrator can settle from 09B §7, MC-C3
and the code.

## Gate check

- Intention status header: **RATIFIED** (`docs/intention/intention.md:3`). ✓
- Plan frontmatter `projection: yes`; master plan §3 requires projection for C4 ("the capture folder must not move"). ✓
- `depends_on: [C3_conversation_and_answering]`: C3 is APPROVED (commit `9736e63`). ✓
- No upstream gate handoff for C4 in `OWNER_DECISIONS_PENDING`. ✓
- Inputs read: `docs/process/flow.md`, the master plan (`master_plan_conversations.md` §3, §4, §5 MC-C3, §6), book 09B §2, §3,
  §7, §10, §12, book 14 (M9), the plan, and the code the plan's Read-first lists name.

## Decision ledger

| # | Decision point | Classification | Proposed routing |
|---|---|---|---|
| L1 | **Open session from Settings with unsaved changes** (U1 task 2). The Settings guard is asynchronous. It returns `false`, opens "Save your changes?", and then Discard/Save write `window.location.hash = target` themselves (`features/settings/SettingsPage.tsx:81-87`, `:226-239`). That later hash write is a `hashchange`, so it becomes `route-changed(session 7)`: no visit, `captureTarget` = 7, `activateSession(7)`. **The capture folder moves**, breaking 09B §10 rule 4 and the MC-C3 invariant. Also, `lib/routes.ts` cannot ask the guard without writing the hash: `leaveGuard` is module-private (`routes.ts:18`), and `goTo` writes the hash (`routes.ts:25-28`), which would itself turn the jump into an ordinary navigation. Neither `lib/routes.ts` nor `SettingsPage.tsx` is in U1's Files. | **plan gap** (silent failure on CR4) | Amend U1. Add `apps/desktop/src/lib/routes.ts` and `apps/desktop/src/features/settings/SettingsPage.tsx` to Files. State the mechanism: the guard is given the continuation as well as the target (e.g. `LeaveGuard = (target, proceed) => boolean`). `goTo`'s continuation writes the hash. `openSession`'s continuation dispatches `open-session(B)`. Settings' Discard/Save call the stored continuation instead of writing the hash. Add row **C4·C3(d)**: `App` on Settings with unsaved changes, the panel on a conversation of session 7, click **Open session** → the "Save your changes?" dialog, Settings still shown. **Discard** → session 7 shown with the bar; no `activateSession` or `deactivateSession` call at all. (Cancel → Settings stays, no call.) Trace CR4; 09B §10 rule 4; MC-C3 invariant. |
| L2 | **When the hash is written, relative to the render** (U1 task 2, MC-C3.4). `PageHeader` reads `window.location.hash` **during render** to build its back link and Back keys (`features/shell/PageHeader.tsx:14-16`). Today `navigate` writes the hash before `setState` (`App.tsx:48-53`). "The hash is written from the new state after a step" can be read as a `useEffect`, which runs after the render. Then a visit opened from the Sessions page renders the workspace with hash `#/`: `parentRoute('#/')` = null, so there is no "← Sessions" link and no ⌘[. Back to a Health origin renders a "Sessions" back link instead of "Settings". The confirming `hashchange` dispatches nothing, so only an unrelated re-render repairs it. Related: `useResumeActiveSession` re-runs whenever `navigate` changes identity (`App.tsx:85-97`). A `navigate` that closes over the state would re-run it at `#/`, and the app would send the user into the active session after every return to the Sessions page. | **plan gap** | Amend U1 task 2: "`window.location.hash` already equals `hashOf(next.route)` when `next` renders. Compute `next = visitStep(current, action)` from a ref of the current state, write the hash, then dispatch. The `hashchange` handler reads `window.location.hash` when it runs (never `event.newURL`), so a late event from an earlier step is only a confirmation. `navigate`, `openSession`, `back`, `shootHere` keep a stable identity. No hash is written at mount; an unknown hash routes to Sessions, as today." Extend **C4·C3(b)**'s outcome with "and the header's back link to Sessions is shown". That is the user-visible form of this defect, and it costs no new criterion. |
| L3 | **How `view` learns "the item's form has a field with that key"** (U2 task 2). `ConversationsService` holds no `Store`; the registry says so explicitly (`master_plan_conversations.md` §4, the `ConversationStore` paragraph: "the service holds no `Store`"). It also holds no integration name. The form's fields depend on the integration: `active_definitions(tx, integration)` adds the integration's definitions only when a name is given (`crates/core/src/services/attributes.rs:131-140`). `custom.condition_report_description` and every `custom.*` key are integration definitions, not built-ins (`attributes.rs:87-127`). Commands read the integration name from settings (`apps/desktop/src-tauri/src/commands/attributes.rs:256-263`). So task 2 cannot be done inside `view` with what the service has, and U2's Files do not contain the seam. "As `AttributesService` builds it" also needs pinning. `form_get` drops preset keys whose definition is not active (`attributes.rs:1555-1566`); `layout_keys` keeps them (`attributes.rs:228-251`). | **plan gap** (blocks C4·C10(a),(b),(d)) | Amend U2 task 2 and Files with one named seam. Recommended, same shape as C3's added read (projection C3 L2): (1) a new `ConversationStore` read, "the field keys of the form of session S's item for integration I", implemented in `crates/infra/src/sqlite/conversations.rs` with `form_get` semantics (active definitions only); (2) `ServiceParts` / `ConversationsService::new` gain `integration: Option<String>`, filled in `crates/integrations/src/lib.rs:1179-1187` from the same settings integration block that gives `shop`. Add `crates/infra/src/sqlite/conversations.rs`, `crates/integrations/src/conversations/mod.rs` and `crates/integrations/src/lib.rs` to Files. `TestApp::with_pricing()` already writes `kind: "shopify"` (`crates/testkit/src/test_app.rs:595-601`), so C4·C10's fixture is "import a `custom.designer` definition for `shopify` (in the Details step), then create the session's item". Alternative: `view` takes the keys from its caller. That changes the `view` signature and its callers (`c3_conversations_answer.rs:403,455,624,628`, `commands/conversations.rs`) and moves work into a command, so it is not recommended. |
| L4 | **Where `jump_key` lives** (U2 task 2; Files list `domain/conversation.rs`). `ConversationView.messages` is `Vec<Message>` (`crates/core/src/domain/conversation.rs:448`). The domain `Quote` (`:127-131`) is **persisted** as JSON in the local copy (`crates/infra/src/sqlite/conversations.rs:142`, `:280`). A `jump_key` field on the domain `Quote` would write a view-time derivation into the copy (`null` at apply time, possibly wrong if anything ever writes it). | **plan gap** | Amend U2 task 2: `jump_key` is computed in `view` only and never stored. Either a view-only message/quote type that `ConversationViewDto` maps from, or a domain field the store never writes (`#[serde(skip)]`, plus a store round-trip unchanged). The planner names one. (The fake's `Quote` literal at `crates/testkit/src/fake_pricing.rs:246` is the **wire** type, so it is unaffected either way.) |
| L5 | **A chip jump on an item whose AI suggestions wait** (U2 task 4, C4·C12). `WorkspacePage` always passes `reviewFirst` and passes no `position` when `detailsStep` is null (`features/workspace/WorkspacePage.tsx:419`, `:499`). `FormBody` then moves to AI review once, after the form loads, when suggestions wait (`features/form/FormDialog.tsx:252-273`). A field jump would set the step and then be overridden: the form opens on AI review, not at the field. C4·C12(b)'s second half, "then with a key the form does not have → the form opens as without `field`", is also two-valued. It means Details, or AI review when suggestions wait, and it does not say whether the dialog was closed and reopened or the key changed while open. | **plan gap** (C4·C12(b) not decidable as written) | Amend U2 task 4: "a set `field` counts as a position for the A19 landing (no AI-review landing)". Amend C4·C12(b): "closed, then reopened with `field` `custom.unknown`, on an item with no waiting suggestion → the Details step, nothing revealed". Optionally add **C4·C12(c)**: an item with one waiting suggestion, opened with `field` `custom.designer` → the Details step with that field revealed, not AI review. |
| L6 | **"A field that has no reveal affordance (the title and description boxes)"** (U2 task 4, Delegated choices) is false. Title and description render through `MetafieldInput` like every field (`FormDialog.tsx:488-513`; `MetafieldInput.tsx:154` picks the textarea for the description), and `reveal` scrolls the box of every field (`MetafieldInput.tsx:149-151`). Separately, `reveal` also opens the AI popover of a field that holds an AI value (`MetafieldInput.tsx:166` `openSignal={reveal}`, `AiPopover.tsx:150-152`), so a chip jump to an AI-valued field opens that popover. | **plan gap** (wrong fact) + **free choice** | Delete the sentence and its delegated choice ("How a field without reveal gets the focus"). Delegate in writing: "a chip jump uses the existing reveal unchanged, including opening the AI popover of an AI-valued field" (smallest option). |
| L7 | **C4·C7(b)'s trigger.** "A normal navigation to session 5 (`navigate(5)`, as the sessions list does)" runs while visiting session 7, where the sessions list is not on screen. The `navigate` callers reachable from a workspace are the review queue's "Next to review" (`App.tsx:134-142`), the Unassigned arrival toast (session 1 only, `ArrivalToasts.tsx:30`) and Assign product (Unassigned only, `WorkspacePage.tsx:330`). The test author has to invent the trigger. | **plan gap** (C4·C7(b) not decidable as written) | Amend the input to a named affordance: "the notifications drawer's **Open** for session 5 (an ordinary hash navigation, MC-C3.3)". Or keep `navigate` and name "**Next to review** with session 5 next in the queue (the `a19_review_queue` mocks)". The orchestrator picks one; the outcome is unchanged. |
| L8 | **The capture session's SKU "comes from `activeSession()`"** (U1 task 4, MC-C3.5). `activeSession()` returns `SessionResponse`, which has no `sku` (`apps/desktop/src/services/bindings.ts:2122-2139`). | **plan gap** (wording) | Amend U1 task 4: "the id from `activeSession()` (null → `Unassigned`), its SKU from the sessions-list row with that id". C4·C4(a)'s fixture then mocks `listSessions` with BEYO-3 and BEYO-7 rows and `activeSession()` returning `{id: 3, …}`. |
| L9 | **Origin label when the origin is the Unassigned workspace** (session 1). MC-C3.1 makes `capture` = 1 there, which is consistent: `useCaptureTarget` already treats 1 as "not activated" (`App.tsx:64`). MC-C3.5's "the origin session's SKU" is undefined for it, because session 1 has no item and no list row (`Header.tsx:50`, `:191-194`; the list carries `unassigned_count`). | **free choice** | Delegate in writing: "← Back to Unassigned". The same rule covers a session whose list row has not loaded yet: show the label empty until it arrives. |
| L10 | **Does the panel's list scope survive a jump?** MC-C3.6 lists "scope" among the state that lives outside the route. `useConversations.ts:89` resets the scope to the session on screen whenever the route's session changes (C2: "All conversations lasts until the route changes"). The plan does not say whether to keep that reset. | **free choice** | Delegate in writing: "the list scope keeps following the session on screen, as today; only the open state and the selected conversation are carried through a jump (C4·C3(c))". The coordinator may align MC-C3.6's wording in the master plan. |
| L11 | **Name collision in the context.** `ConversationsState.back` already exists and means "back to the list" (`useConversations.ts:28`, `:105`). Task 2 offers the visit's `back()` through the same context. | **free choice** | Amend U1 task 2 to a non-colliding name (e.g. `visitBack`) or delegate the name explicitly. |
| L12 | **Clicking the same chip twice.** "React to a change of `field`, as `reveal` does with its counter" can mean a change of value (a second click on the same chip while the form is open does nothing) or a counter per `openField` call (every click jumps). | **free choice** | Delegate: "every `openField` call jumps (a counter, like `reveal`)". |
| L13 | **U2 has no guide task**, but it adds a port method and a command DTO field. `docs/guide/02_architecture.md:413-414` lists the `ConversationsProvider` methods, and with L3(a) a `ConversationStore` read as well. Flow §7A requires the guide to change in the same change. | **plan gap** (minor) | Add to U2: task "Guide: `attribute_key` and the form-keys read in 02; a change-impact row 'Pricing's quote field names → `BeyoPricingAdapter::attribute_key`' in 04". Add `docs/guide/{02_architecture.md, 04_change_impact.md}` to U2's Files. |

## Reality checks

- Every path in both units' Files exists or is marked new: `globals.css`, `suite/main.rs` (has `mod c3_conversations_answer;`,
  line 13), the four guide files, `p19_navigation`, `a19_review_queue`, `a11_header_review`, `w3_writer_triggers`,
  `p05_capture_arrival.rs`, `StepTabs.tsx`, `backlog_conversations/`. ✓ **Missing from Files:** `lib/routes.ts` and
  `SettingsPage.tsx` (L1); the seam files of L3; the guide files of L13.
- Crate names in the test commands resolve: `psm-integrations`, `psm-testkit`, `psm-desktop`, `psm-core`; npm scripts
  `typecheck` and `test` exist (`apps/desktop/package.json:9-10`). ✓
- Single `ConversationsProvider` implementor (`beyo_pricing/adapter.rs:254`), so adding `attribute_key` breaks no other
  impl. ✓
- `useCaptureTarget` (`App.tsx:61-77`) is the only frontend caller of `activateSession`/`deactivateSession` (grep over
  `apps/desktop/src`, non-test). The plan's Review-mode check "nothing else calls them" holds today. ✓
- Hash writers the plan lists as ordinary navigation are confirmed: `Header.tsx:170,179,212,231,247`,
  `NotificationsDrawer.tsx:14`, `PageHeader` via `goTo`, `ConversationsPanel.tsx:82`, and **also** `SettingsPage.tsx:232,238`
  (the guard continuation, L1), which the plan's hazard list does not name.
- 09B §7 field mapping matches the Pricing package: `API.md:86-88` names `title`, `description`, `condition`,
  `mf:beyo_pricing.material`, `mf:beyo.designer`; `Quote.field` is a free string (`openapi-integration.json` `Quote`). ✓
- Row counts derived: U1 12+3+3+3+3+1+3+2 = 30, U2 5+4+3+2 = 14, total 44 over 12 criteria, matching the plan's table. ✓
- Trace `M9` (C4·C8(b)) resolves to book 14 M9 ("in the session given by its capture folder"). ✓

## Criteria decidability (both questions per row)

| Rows | Test writable now, one exact outcome? | Fixture producible? |
|---|---|---|
| C4·C1(a)–(l), C4·C2(a)–(c) | Yes: pure `visitStep`/`captureTarget`, each case has one result (MC-C3.3 covers every action × visit/no-visit). | Ours. |
| C4·C3(a),(c) | Yes: mocked services, exact call list since the click. | Ours (mocked `conversationGet` with `session_id: 7`). |
| C4·C3(b) | Yes; add the back-link clause (L2). | Ours. |
| C4·C3(d) (proposed) | Yes once L1's mechanism is named. | Ours (Settings dirty state as in existing P63 tests). |
| C4·C4(a) | After L8's wording fix. | Ours. |
| C4·C4(b),(c), C4·C5(a)–(c), C4·C6(a), C4·C7(a),(c) | Yes. C4·C5(c) reaches the session-9 conversation through the "All conversations" footer, because the scope follows the route (L10). | Ours. |
| C4·C7(b) | **No**: the trigger is not reachable as written (L7). | — |
| C4·C8(a),(b) | Yes: `SessionsService::view(B)`, `conversations.view`/`attachable`, active session read; capture as in `p05`. It is a regression guard with no production change expected. | `TestApp::with_pricing()` + `FakePricing.ask`. ✓ |
| C4·C9(a)–(e) | Yes, pure. | Grounded in `API.md:86-88` (rule 17). ✓ |
| C4·C10(c) | Yes (`session_id` none → none). | `FakePricing.change_silently` edits the wire message's quote (`fake_pricing.rs:449`; `IntegrationCase.messages`). ✓ |
| C4·C10(a),(b),(d) | **No, until L3** names the seam and the integration (the "form has `custom.designer`" fixture depends on it). | Quote: ✓ as (c). Form field: needs an imported `shopify` definition (L3). |
| C4·C11(a)–(c) | Yes: the context's `openField` mocked, `session_id` vs the session on screen. | Ours. |
| C4·C12(a) | Yes, with a fixture without waiting suggestions (L5). `scrollIntoView` is observable (`MetafieldInput.tsx:150`). | Ours. |
| C4·C12(b) | **No**: second half two-valued (L5). | — |

**Trace, both directions.** Every row's trace cell resolves (09B §7.1–7.5, quote chip; MC-C3.1–3.3, 3.5, 3.6; CR4; CV-2;
M9). MC-C3.4 ("one step, one render") is served only by the call lists of C4·C3(a)/C4·C5. Its hash/render half has no
row until L2's clause is added to C4·C3(b). 09B §7.3's "as it was" is explicitly scoped out by the plan's Goal. That is
consistent, not unserved.

## Write perimeter

This file only. No plan, intention, master plan, prompt, code or test was edited; no commit; no archgraph in this repo.

## Appendix (non-authoritative, discard): the skeleton this projection walked

`visit.ts`: `type Route = {kind: 'sessions'|'session'|'settings'|'health'; sessionId?: number}`; `type Visit = {origin:
Route; capture: number|undefined; visited: number}`; `type Nav = {route: Route; visit: Visit|null}`; `type VisitAction =
{type:'open-session', id} | {type:'back'} | {type:'shoot-here'} | {type:'route-changed', route}`; `visitStep(s, a): Nav`
per MC-C3.3; `captureTarget(s) = s.visit ? s.visit.capture : s.route.kind==='session' ? s.route.sessionId : undefined`;
`routeOf(hash)`/`hashOf(route)`. `App.tsx`: `useReducer(visitStep, routeOf(location.hash))`, a ref of state, a
`step(action)` that writes the hash and then dispatches (L2), the `hashchange` handler, and
`useCaptureTarget(captureTarget(state))`. The conversations context gains `openSession`/visit-back/`shootHere`/`openField`.
`VisitBar` sits between `Header` and `WorkspacePage`. U2: `attribute_key` in the adapter; the form-keys read (L3);
`jump_key` view-only (L4); `QuoteBlock` button iff `jump_key && session_id === ctx.sessionId`; `detailsField` in `App`
→ `WorkspacePage` → `FormDialog field`.
