---
plan: C4_visit_and_quote_jump
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-20
actor: opus (pipeline-reviewer)
---
# Review: C4_visit_and_quote_jump, round 1

**Verdict: APPROVED_WITH_NOTES.** 0 blocking findings, 3 notes (backlog), 0 owner cards. Tracker → APPROVED; Review log entry and narrative written.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- Tracker line read `IMPLEMENTED (2026-11-20 …)`.
- Checkpoints `816fdc4` (U1), `4955b51` (U2) and close `1385000` are in `git log`.
- Process authority: `docs/process/flow.md`. That means no mutation probes, no production edits, blocking only for B1–B4.

## Verified correct
**Perimeter.**
- `git show --stat 816fdc4` gives 16 paths and `4955b51` gives 23. Both equal the handoff's declared lists exactly.
- The disclosed deviations stay inside the plan's file sets:
  - `globals.css` was also edited in U2. It is in U1's list.
  - `services/conversations.ts` was not edited. Its types come from bindings.

**Runs** (this review, one cargo command at a time; code tree = `4955b51`, HEAD `1dacb59` adds documents only):

| Command | Result |
|---|---|
| `npx vitest run src/features/conversations src/features/shell src/features/sessions src/features/settings src/features/form` | 64 files / 402 passed |
| `cargo test -p psm-integrations --lib conversations::` | 33 passed |
| `cargo test -p psm-testkit --test suite c4_conversations_visit::` | 6/6 |
| `cargo test -p psm-desktop` | 80 passed |
| `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-desktop -p psm-testkit --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` | clean |

The implementer's full-suite stamp (cargo 2082, npm 784) stands. Nothing above contradicts it.

**Review-mode checks (by reading `App.tsx` and the diff).**
- Route and visit are one `useReducer(visitStep)` state in `useNav`.
- Every change goes through `step`: it computes `visitStep(ref, action)`, writes the hash if it differs, sets the ref, then dispatches. The hash therefore equals the route when the page renders (MC-C3.4).
- The `hashchange` handler reads `window.location.hash` and ignores a confirmation.
- `navigate`, `openSession`, `visitBack` and `shootHere` keep a stable identity through `useCallback` on a stable `step`. No hash is written at mount.
- `useCaptureTarget(captureTarget(nav))` is the only caller of `activateSession` / `deactivateSession` under `apps/desktop/src` (grep, tests excluded).
- `SettingsPage` stores `{target, proceed}`. Discard calls `proceed()`, Save calls it after a successful save, Cancel drops it. Neither writes a hash.
- `jump_key` exists only in the view-only `ConversationView.jump_keys` and `QuoteDto`. The persisted `Quote` / `Message` are unchanged, and C4·C10(a) reads the stored `conversation_messages.quote` JSON.
- No pre-existing test was weakened. `git diff --stat 816fdc4^ 4955b51` over `__tests__` and `crates/testkit/tests` shows only the new C4 files and one `mod` line in `suite/main.rs`.

**Production against authority.**
- `visitStep` matches MC-C3.3 case by case:
  - `open-session` to the route's own session: unchanged.
  - A first jump records origin, capture and visited.
  - `open-session(visit.capture)` behaves as `back`.
  - A further jump keeps origin and capture.
  - `back` and `shoot-here` follow MC-C3.3, and do nothing when there is no visit.
  - `route-changed` to the visited session: unchanged. Any other route ends the visit.
- `captureTarget` = MC-C3.2.
- `VisitBar` (MC-C3.5):
  - Shown iff `visit ≠ null`.
  - The capture session comes from `activeSession()` and its SKU from the `listSessions` rows. Null or id 1 reads "Unassigned".
  - Origin labels: SKU, Sessions, Settings, Health.
- Panel open state and selection live outside the route (MC-C3.6). C4·C3(c) observes this.
- `attribute_key` sits in the Pricing adapter with exactly the 09B §7 mapping (H7). Core passes the string through the port.
- `attributes::form_keys` = `layout_keys` ∩ `active_definitions`, one copy of the layout rules.
  - The infra read is one deferred (read-only) transaction.
  - It returns an empty set when the session has no item.
  - `integration` comes from `settings.integration.kind`, the same opaque string `active_definitions` already takes.
- `QuoteBlock` renders a button iff `jump_key` is set and the conversation's `session_id` equals the context's `sessionId` (the route's session, i.e. the visited one during a visit).
- `FormDialog`:
  - It renders `FormBody` only while open, so the `jumped` ref resets on each open.
  - Each new `n` sets the step and the reveal through the existing reveal.
  - A field the form has makes the A19 landing return early.
  - An unknown key changes nothing.

**Rows → tests (each observes its row's outcome at the row's boundary and fails on a wrong outcome).**
- C4·C1(a)–(l), C4·C2(a)–(c): `toEqual` on the whole state, `toBe` on the target.
- C4·C3(a)–(d), C4·C5(a)–(c), C4·C6(a), C4·C7(a)–(c):
  - `App` is rendered with mocked services.
  - `captureCalls()` asserts the whole ordered activate/deactivate list after the hash events settle.
  - So a split-state render that activated the visited session, even briefly, turns C3(a), C5(a) and C5(c) red.
  - C3(b) also asserts the header's "Back to Sessions" link (the hash was written before render).
  - C3(d) runs through the real `SettingsPage` dirty state and its dialog.
  - C7(b) goes through the real notifications drawer.
- C4·C4(a)–(c): the exact bar sentence and button names, and the bar's absence.
- C4·C8(a)/(b): `TestApp` reads then `sessions().active()`. A hook-delivered capture's `photos.session_id` = A.
- C4·C9(a)–(e): exact outputs, including all five "none" inputs.
- C4·C10(a)–(d): exact `jump_keys` entry or none. (b) and (c) separate the form condition from the on-this-Mac condition.
- C4·C11(a)–(c): button vs plain text; `openField` called once with the key.
- C4·C12(a)–(c): the shown step region, plus `scrollIntoView` contexts equal to exactly the requested field.

**Changed-input probe** (a throwaway test file under `features/conversations/__tests__/`, run once, then deleted; `git status` clean afterwards). These cases are outside the rows and all held:
- `route-changed(session 3)` while visiting 7 from 3: visit ended, capture still 3, so no call.
- Health as origin, then `back`: returns to Health.
- `shoot-here`, then `open-session(3)`: a new visit with capture = 7.
- A visit from Unassigned (session 1): capture 1, and `open-session(1)` ends it.

**Assumptions 1–10.** None contradicts 09B §7 or MC-C3. They are reported to the owner in the narrative's "Choices made for you".

**Hard constraints H1–H8.**
- H1: no external write is added. The chip and the visit only read.
- H2–H6: not touched.
- H7: Pricing field names and Shopify keys stay in the adapter; core sees opaque strings.
- H8: the command only maps the DTO; the logic is in `ConversationsService::view`.
- No live service is used: the tests run on the fakes.

**Guide (flow §7A).** `docs/guide/01–04` are updated for the visit, `attribute_key`, the form-keys read and CR4's enforcement.

## Findings
None blocking.

## Notes written to backlog
File: `docs/implementation/backlog_conversations/C4_visit_and_quote_jump.md`. This is the plan's Review mode location and matches C1–C3. `docs/implementation/backlog.md` is not used for conversations phases.
- **N1 (UX):** a failed `activeSession()` read leaves the bar's capture name blank until the next `session_changed`. This hides the very disagreement MC-C3.5 is meant to show.
- **N2 (duplication):** Unassigned's id `1` is hard-coded a third time (`VisitBar.tsx`).
- **N3 (test reach):** C4·C12(c) does not exercise `FormBody`'s landing skip, because the effect order already wins. The row's outcome is asserted. The implementer disclosed this.

**Carry-forward:** N1–N3 → the conversations backlog, for the orchestrator's next conversations touch (no named destination phase; C4 is the last planned C-phase in `master_plan_conversations.md`).

## Lessons for the plans
- U2's Files listed `services/conversations.ts`, but generated bindings already carry new DTO fields there. Future plans can drop service-wrapper files when only a DTO field is added.
- The prompt template names `docs/implementation/backlog.md` for notes, while the conversations plans name `backlog_conversations/<phase>.md`. Align the review prompt template for the C track.

## Write perimeter (this session)
- `docs/implementation/plans/C4_visit_and_quote_jump.md`: the Tracker line and one Review log entry.
- `docs/implementation/backlog_conversations/C4_visit_and_quote_jump.md` (new).
- `docs/implementation/narratives/C4_visit_and_quote_jump.md` (new).
- This handoff.
- No production or test file was changed. The probe file `apps/desktop/src/features/conversations/__tests__/zz_review_probe.test.ts` was created, run and deleted.
- No database or Capture One state was touched. No commits.
