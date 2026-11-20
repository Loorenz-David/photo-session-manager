---
plan: C4_visit_and_quote_jump
role: implement
round: 1
state: DONE
date: 2026-11-20
actor: claude (opus, pipeline-implementer)
---
# Handoff — C4_visit_and_quote_jump, implement r1

**Gate:** the intention is RATIFIED (2026-10-24). `depends_on: [C3_conversation_and_answering]`, and that plan's Tracker line reads APPROVED (2026-11-20, APPROVED_WITH_NOTES). This plan's Tracker line read NOT_STARTED. I worked the two units in order and made a `CHECKPOINT (not approved):` commit after each unit's test commands passed. Then I ran the closing gate once, at the end.

## What was built
**For the photographer:**
- **Open session** appears in a conversation's header line when the conversation belongs to a session other than the one on screen. Pressing it opens that session as a **visit**:
  - The panel stays open on the same conversation.
  - The camera's folder does not move. New photos keep going to the session being shot, or to Unassigned when the jump started on the Sessions page or in Settings.
- **The visit bar** sits as a slim strip under the header while a visit lasts. It reads "Visiting BEYO-7: new photos still go to BEYO-3." and offers two buttons:
  - The second SKU is the session the backend reports active, so a mismatch would be visible rather than silent.
  - **← Back to <origin>** returns to where the photographer was before the first jump: that session's SKU, "Sessions", "Settings", "Health", or "Unassigned". It does this also after jumping through several conversations.
  - **Shoot here instead** points the camera at the visited session, exactly like opening it normally, and the bar goes away.
- **Ordinary navigation ends the visit and behaves as before.** This covers the Sessions back link, the notifications drawer's Open, the Sessions list, review navigation and the Back keys. Opening a session that way points the camera at it, as today.
- **Opening the session being shot** from a conversation ends the visit, exactly like Back.
- **From Settings with unsaved changes,** Open session first asks "Save your changes?":
  - Discard or Save opens the session as a visit (the camera's folder still does not move).
  - Cancel stays in Settings.
- **The quote chip** (for example "CONDITION" or "DESIGNER") is a link when two things hold: this Mac's form for that product has the field, and the conversation's session is the one on screen. A click opens Details on the step holding that field, scrolls to it and opens its AI popover if the field holds a suggestion.
  - Every click jumps, also a second click on the same chip.
  - A field the form has wins over the "land on AI review" rule.
  - In any other case the chip stays plain text.

**In the code:**
- `features/conversations/visit.ts`: the types `Route`, `Visit`, `VisitState`, `VisitAction`, plus `visitStep`, `captureTarget`, `routeOf`, `hashOf`, `sameRoute`, `routeSession` and `sessionRoute` (MC-C3.1–3, verbatim).
- `App.tsx`: `useNav` puts route and visit in one `useReducer(visitStep)`, which replaces `useRoute`'s three states (MC-C3.4).
  - A step computes the next state from a ref, writes the hash, then dispatches. So the hash already holds the new route when the page renders.
  - The `hashchange` handler reads `window.location.hash` when it runs. If the hash only confirms the current route, it does nothing; anything else is `route-changed`.
  - `navigate`, `openSession`, `visitBack` and `shootHere` keep a stable identity. No hash is written at mount.
  - `useCaptureTarget(captureTarget(state))` is still the only caller of `activateSession` / `deactivateSession`.
  - `openField` sets `detailsField = {key, n}` and opens Details. The field is cleared when the form closes and when the session changes.
- `lib/routes.ts`: `LeaveGuard = (target, proceed) => boolean` and `leaveThen(target, proceed)`. `goTo` is now `leaveThen(hash, write hash)`. `SettingsPage` stores `{target, proceed}` when its guard refuses; Discard and Save call `proceed()` and no longer write the hash themselves.
- `useConversations(sessionId, jump?)`: the context also carries `visit`, `openSession`, `visitBack`, `shootHere` and `openField`. The second argument is optional, so C2/C3 harnesses still work.
- `VisitBar.tsx` reads `activeSession()` and the sessions list when the bar appears and on `session_changed` (`onSessionsChanged`).
- **Core:**
  - `services::attributes::form_keys(tx, item_type, integration)` = `layout_keys` restricted to active definitions.
  - `ConversationStore::form_keys(session_id, integration)`, implemented in `crates/infra/src/sqlite/conversations.rs` as one read transaction over core's `form_keys`; it returns empty when the session has no item.
  - `ConversationsProvider::attribute_key` (pure).
  - `ConversationsService::new(…, shop, integration, …)`.
  - `view` fills the view-only `ConversationView.jump_keys` (message id → key). It is never stored: the persisted `Quote`/`Message` types are unchanged.
- **Adapter:** `BeyoPricingAdapter::attribute_key` (title / description / condition / `mf:<ns>.<key>`).
- **Wiring:** `ServiceParts.integration` is filled from `settings.integration.kind` in `crates/integrations/src/lib.rs`.
- **Command:** `QuoteDto.jump_key` (absent when none), mapped from `jump_keys` in `ConversationViewDto`. `bindings.ts` was regenerated.
- **Frontend for the chip:**
  - `QuoteBlock`: the chip is a button iff `jump_key` is set and the conversation's `session_id` equals the session on screen.
  - `MessageBubble` and `ConversationView` pass the conversation's session down.
  - `WorkspacePage` passes `detailsField` to `FormDialog` as `field`.
  - In `FormDialog`, a new `field` request (its `n`) moves to the field's step and reveals it, using the existing `setStep` + `setReveal`. A field the form has skips the A19 landing.
- **Guide:**
  - 01: the C4 paragraph.
  - 02: the visit, `attribute_key` and the form-keys read, the chip.
  - 03: CR4's enforcement.
  - 04: rows for navigation/capture target, Pricing's quote field names, and the form's layout rules.

## Tests → criterion rows
| Rows | Test (boundary) |
|---|---|
| C4·C1(a)–(l) | `features/conversations/__tests__/c4_visit_rules.test.ts` `c4_c1a_…` – `c4_c1l_…` (`visitStep`, pure) |
| C4·C2(a)–(c) | same file `c4_c2a_…` – `c4_c2c_…` (`captureTarget`, pure) |
| C4·C3(a), (b), (c) | `features/conversations/__tests__/c4_visit.test.tsx` `c4_c3a_…`, `c4_c3b_…`, `c4_c3c_…` (`App`; whole activate/deactivate call list, in order) |
| C4·C3(d) | same file `c4_c3d_…discard…`, `c4_c3d_…cancel…` |
| C4·C4(a), (b), (c) | same file `c4_c4a_…`, `c4_c4b_…`, `c4_c4c_…` |
| C4·C5(a), (b), (c) | same file `c4_c5a_…`, `c4_c5b_…`, `c4_c5c_…` |
| C4·C6(a) | same file `c4_c6a_…` (`['activate 7']` exactly) |
| C4·C7(a), (b), (c) | same file `c4_c7a_…` (`['deactivate']`), `c4_c7b_…` (the real `NotificationsDrawer`'s Open for session 5 → `['activate 5']`), `c4_c7c_…` (`['activate 5', 'deactivate']`) |
| C4·C8(a), (b) | `crates/testkit/tests/suite/c4_conversations_visit.rs` `c4_c8a_…`, `c4_c8b_…` (`TestApp::with_pricing()`; the capture is written into the fake Capture One's current capture folder and delivered by the `capture_done` hook) |
| C4·C9(a)–(e) | `crates/integrations/src/conversations/beyo_pricing/adapter.rs` tests `c4_c9a_…` – `c4_c9e_…` (core mechanism, pure) |
| C4·C10(a)–(d) | `c4_conversations_visit.rs` `c4_c10a_…` – `c4_c10d_…` (`ConversationsService::view`; (a) also checks the stored `conversation_messages.quote` JSON carries no jump key) |
| C4·C11(a)–(c) | `features/conversations/__tests__/c4_quote_chip.test.tsx` `c4_c11a_…` – `c4_c11c_…` (`QuoteBlock` inside `ConversationView`, real `useConversations` state) |
| C4·C12(a)–(c) | `features/form/__tests__/c4_form_field.test.tsx` `c4_c12a_…` – `c4_c12c_…` (`FormDialog`; the reveal is observed as `scrollIntoView` on the field's `data-field` box) |

**Self-checks:**
- I temporarily changed `useCaptureTarget(captureTarget(nav))` back to `useCaptureTarget(sessionId)` (the pre-C4 behaviour). 8 of the 15 `c4_visit` tests went red: C3(a), C3(b), C3(d), C5(a–c), C7(a), C7(b). I restored the line.
- I also removed C4·C12(c)'s landing skip in `FormDialog`, and the test stayed green. The jump effect is declared after the landing effect, so the field already wins when the two run in the same commit. The skip guards against a reordering of the two effects (projection L5). The behaviour the row names is asserted either way.
- Files touched by these probes, all restored, no net change: `apps/desktop/src/App.tsx`, `apps/desktop/src/features/form/FormDialog.tsx`.

The pre-existing navigation and leave-guard tests pass unchanged: `p19_navigation`, `a19_review_queue`, `a11_header_review`, `w3_writer_triggers`, `p63_settings_save_and_style` and `p68_setup_polish`. None of them was edited.

## Assumptions
1. **The capture session in the bar** is shown as an empty label until `activeSession()` answers; `null` reads "Unassigned". Session id 1 also reads "Unassigned". The bar re-reads on `onSessionsChanged` (which listens to `session_changed`, among other events).
2. **The bar sits only in the main branch of `App`** (sessions/session routes). A visit always has a session route (`visit ≠ null` ⇒ route = the visited session), so it is never needed on Settings or Health.
3. **`route-changed` to the route already shown, with no visit, returns the same state.** No re-render, no capture call. `visitStep` returns the identical state object for every "unchanged" case.
4. **The hash is written only when it does not already represent the next route** (`routeOf(hash) ≠ next.route`). This keeps an unknown hash or `''` untouched for the Sessions route and avoids extra `hashchange` events. Ordinary navigation (`navigate`) still goes through `route-changed`. It still does not ask the leave guard, as before C4.
5. **`jump_key` travels on the wire only when set.** `#[serde(skip_serializing_if = "Option::is_none")]`, the same pattern as P69's `masters_ready`, so the TypeScript type is `jump_key?: string`. This keeps C3's `QuoteBlock` test fixture valid without editing it.
6. **A `field` whose key the form does not have** leaves the form as without `field`: no jump, and the A19 AI-review landing still applies. C4·C12(b)'s case has no waiting suggestion.
7. **The C4·C8 capture is a hook-delivered RAW in the folder the fake Capture One currently captures into** (`fake.state.captures`), not a camera-sidecar capture. This proves the folder still points at A after the visit's reads.
8. **C4·C10's `custom.designer` definition** is imported with the existing `TestApp::research_fields` helper (a Details-step text definition for `shopify`).
9. **The chip button** has the hover title "Open the form at this field" and an underline (`.quote-chip-link`). The visit bar uses a light `--warn` tint over the page background and is sticky under the 56 px header (delegated styling).
10. **`attribute_key`'s `mf:` rule** splits at the first dot. So `mf:a.b.c` → `a.b.c` (namespace `a`, key `b.c`). The plan only requires a non-empty namespace without a dot and a non-empty key.

## Deviations
- **One file outside U2's listed files: `apps/desktop/src/styles/globals.css`** (the `.quote-chip-link` rule, one line). It is in U1's file list.
- **`apps/desktop/src/services/conversations.ts` was not edited.** Its types come from `bindings.ts`, so `jump_key` reached the frontend without a change there.
- **The prompt asks for actor `codex` in the tracker.** This session ran on Claude Opus (`PSM_IMPLEMENTER=claude`, master plan §3), so the tracker and this handoff say `claude (opus)`.

## Files changed
**U1 (816fdc4):**
- `apps/desktop/src/App.tsx`
- `apps/desktop/src/lib/routes.ts`
- `apps/desktop/src/features/settings/SettingsPage.tsx`
- `apps/desktop/src/features/conversations/{ConversationView.tsx, useConversations.ts, visit.ts (new), VisitBar.tsx (new)}`
- `apps/desktop/src/features/conversations/__tests__/{c4_visit_rules.test.ts, c4_visit.test.tsx}` (new)
- `apps/desktop/src/styles/globals.css`
- `crates/testkit/tests/suite/{main.rs, c4_conversations_visit.rs (new)}`
- `docs/guide/{01_product.md, 02_architecture.md, 03_rules_and_principles.md, 04_change_impact.md}`

**U2 (4955b51):**
- `crates/core/src/{domain/conversation.rs, ports/conversations.rs, services/attributes.rs, services/conversations.rs}`
- `crates/infra/src/sqlite/conversations.rs`
- `crates/integrations/src/{lib.rs, conversations/mod.rs, conversations/beyo_pricing/adapter.rs}`
- `crates/testkit/tests/suite/c4_conversations_visit.rs`
- `apps/desktop/src-tauri/src/commands/conversations.rs`
- `apps/desktop/src/services/bindings.ts` (generated)
- `apps/desktop/src/App.tsx`
- `apps/desktop/src/features/workspace/WorkspacePage.tsx`
- `apps/desktop/src/features/form/FormDialog.tsx`
- `apps/desktop/src/features/conversations/{QuoteBlock.tsx, MessageBubble.tsx, ConversationView.tsx, useConversations.ts}`
- `apps/desktop/src/styles/globals.css`
- `apps/desktop/src/features/conversations/__tests__/c4_quote_chip.test.tsx` (new)
- `apps/desktop/src/features/form/__tests__/c4_form_field.test.tsx` (new)
- `docs/guide/{02_architecture.md, 04_change_impact.md}`

**Close:** `docs/implementation/plans/C4_visit_and_quote_jump.md` (the Tracker line and a Review log entry), and this handoff.

**Tool-recorded state:** none (no architecture graph in this project).

## Checkpoint SHA
1. U1: `816fdc4` — CHECKPOINT (not approved): C4·U1 the visit, the bar, Back, Shoot here instead
2. U2: `4955b51` — CHECKPOINT (not approved): C4·U2 the quote chip jump
3. Close: STAMP_CLOSE_SHA (the tracker, the Review log and this handoff)

## Stamp
Tree: `4955b51` (U2 checkpoint), plus the close commit's documents only. Run once, one command after the other:
| Command | Result |
|---|---|
| `cargo test --workspace` | green: 20 test binaries, 2082 passed, 0 failed (suite 1156 passed, 767.8 s) |
| `cargo clippy --workspace --all-targets -- -D warnings` | green, no warnings |
| `cargo fmt --all -- --check` | green |
| `cd apps/desktop && npm run typecheck` | green |
| `npm test` | green: 119 files, 784 tests |

Pre-existing failures: none seen. Unit commands, run before each checkpoint:
- **U1:** vitest `conversations shell sessions settings` 46 files / 293 tests; typecheck; `c4_conversations_visit::` 2/2.
- **U2:** `psm-integrations --lib conversations::` 33 passed; `c4_conversations_visit::` 6/6; `psm-desktop` 80 passed; vitest `conversations form workspace` 41 files / 296 tests; typecheck; clippy on the five crates clean.

## Manual try-out
On the installed app. Sessions never launch it.
1. Open session A and keep shooting. Have the pricer ask a question about product B (session B on this Mac).
2. Open the conversations panel, press "N new in other sessions → All conversations", open the question and press **Open session**.
3. Session B opens, and the panel still shows the question. The bar reads "Visiting <B>: new photos still go to <A>." Take a photo: it lands in A.
4. Press **← Back to <A>**: you are in A again, and the bar is gone.
5. Jump to B once more and press **Shoot here instead**: the bar goes away, and the next photo lands in B.
6. In B, open a question that quotes the Condition text (or a field this Mac's form has). Click the chip (for example CONDITION): Details opens on that step, scrolled to the field.
7. Optional: in Settings, change a field without saving, open a conversation of another session and press **Open session**. "Save your changes?" appears. Discard opens the session with the bar, and the camera's folder does not move.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
