---
plan: T4_template_retake_and_shift
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-16
actor: claude opus 5.5 (pipeline-reviewer)
---
# T4 — Retakes and bad shots · review r1

**Verdict: CHANGES_REQUESTED.** One blocking finding (B1, route production). After you deny a Condition photo, **Shift the rest back** is offered whenever a later Condition photo exists, and pressing it moves nothing. 10G §6.4 says the button disappears when there is nothing to shift. Everything else holds: the perimeter, the checks, all 14 rows and their tests, the pinned-test table, the removal of `container_filled`, the guide paths, and H1–H8.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
This is the first review of this phase. The implementer was Claude Opus 5.5. The phase was built as three units plus a closing checkpoint. There are no earlier rounds.

## Verified correct
- **Gate.** The plan's Tracker line read IMPLEMENTED. The checkpoints `2015c99` (U1), `332237d` (U2), `6e026d2` (U3) and `76a84a3` (closing) are all in `git log`.
- **Perimeter.** I ran `git show --stat` on each checkpoint. The committed paths match the handoff's "Files changed" list exactly. That list is the plan's per-unit files plus the four disclosed deviations:
  - `commands/items.rs`: the exhaustive error-to-code match.
  - `useWorkspaceState.ts`: the toast text removed, and the action runner for the button.
  - `WorkspacePage.tsx`: one prop.
  - `templateDrop.ts`: a comment only.

  Each deviation is required by a task. `templates.ts` landed in U2 instead of U3, which is harmless. Nothing is extra or missing. The tree was clean at the start and at the end of the review.
- **Checks** (the plan's Review mode, each run once, one cargo command at a time, on the clean tree `e8cc68a`):
  - `t4_template_retake_shift::`: 11/11
  - `t3_template_hand::`: 19/19
  - `t2_template_placement::`: 21/21
  - `cargo test -p psm-desktop`: 30/30
  - `cargo clippy -p psm-core -p psm-infra -p psm-desktop -p psm-testkit --all-targets -- -D warnings`: ok
  - `cargo fmt --all -- --check`: ok
  - `npm run typecheck`: ok
  - `npx vitest run src/features/workspace src/features/settings`: 34 files, 165 tests

  The implementer's full-suite stamp stands (Rust behaviour suite 952, UI 538).
- **U1, the retake.** In `drop_on_container`, the match `(Some(o), _)` with `own.is_none()` calls `retake`.
  - The old photo is denied with the same writes as `ReviewService::delete`: removed from its ordered group, `set_photo_review(Deleted, None)`, `place_by_hand = 1` with the slot kept (identical to `photo_place_mark(id, true, false, None)`), `set_photo_denied(at)` and the mirror tag. It also gets `released = 1`.
  - The dropped photo goes to the end of Main (`reclassify`) and gets a row with slot n, `place_by_hand = 1` and `frame_by_hand = 0`. Its `key_at` and `key_name` are the old photo's *effective* key from `shot_keys`, which is the P2 fallback chain.
  - Container n's presets are applied through `apply_frame`. Main is renumbered and the session is touched, all in one transaction.
  - It is one `StepKind::Template` step, "Retake container <n>", over both photos. `photo_state` includes `denied_at`, so undo removes the denial row (row c, full-state equality).
  - The occupant search reads slots of Main photos only, so a denied photo's kept slot never counts as an occupant.
  - Two photos that both sit in containers still swap.
- **U1, keys and differs.** The dropped photo takes the old photo's shot index, so `differs` is false (by hand, at its expected container, `frame_by_hand` 0). Later arrivals still go to Condition (row b). A retake of a retake carries `DSC00003.ARW` on (row d).
- **`container_filled` is gone.** It no longer appears in `error.rs`, `commands/items.rs`, `bindings.ts`, `useWorkspaceState.ts` or `master_plan_templates.md` §4. The only mentions left are the historical T3 narrative and the T3 backlog note. `t3_c2d_…` is deleted. T3·C4(d) keeps its id and gains the two inputs the plan's table names: a slotless photo over a filled tile is a template drop, and over a slotless tile it is the default drop.
- **U2, Deny leaves the container empty.** P2 counts a denied photo that is not released, so later photos keep their places (rows a and b).
- **U2, `shift_back`.** In order:
  1. `ensure_session_not_cleared`, then `no_template`, then `nothing_to_shift`.
  2. Every Deleted photo that is not released is released (its row is created if missing).
  3. `place_template_photos` re-places the photos that have `place_by_hand = 0` and are not Deleted, with P8 frames, and orders Condition and Main (P7).
  4. The step "Shift the rest back" names every photo whose `photo_state` changed, found by a before/after snapshot of the whole session.

  `place_template_photos` is the old loop of `place_arrival`, unchanged, with `photo_id` replaced by `arrived: Option<i64>` (t2 21/21). Undo restores the gap. A second undo restores the denied photo (row d, full-state equality).
- **U2, `can_shift_back`.** It returns true when some photo that is not Deleted, is placed by the template and has a shot index greater than the smallest index of an unreleased Deleted photo exists. That is exactly the plan's task 1 rule. See the finding for where that rule disagrees with 10G.
- **U2, the command.** `template_shift_back` goes through `hand_action`, like `template_back`, and is registered in `specta_builder`. The binding is regenerated, and the wrapper is `templateShiftBack`. Row g runs the wired mock app (`no_template`) and checks that `bindings.ts` contains the command. The code string `nothing_to_shift` is pinned through `CommandError::from`.
- **U3, the button.** `TemplateWarning` shows Shift the rest back while `can_shift_back`, including when `count` is 0. `PhotoPanel` passes `onTemplateShiftBack`, and the page's version runs the action runner with an undo toast. `useTemplateView` re-reads on `history_changed` and on photo and session events, so the button appears after a Deny and disappears after the shift.
- **U3, the story.** `t4_c4a_the_owners_story` is one test that runs A1–A9 in order, each asserted before the next. It covers A1 (create, reorder, presets, shooting order, `kill_and_restart`, read back equal), A3 (held shutters, 1-3-2 arrival, split rect), A7 (button false with nothing later, true after file 5, shift fills container 4) and A8 (retake, red, count 0).
- **U3, the guide.** Every path named in the new guide rows exists: `mechanisms/{shot_order.rs, template.rs}`, `services/{template_placement.rs, templates.rs, history.rs}`, migrations `014_shooting_templates.sql` and `015_photo_places.sql`, `features/settings/{SettingsNav, TemplatesSection, TemplateExample}.tsx`, `features/workspace/{ContainerGrid, TemplateInfoDialog, TemplateWarning, useTemplateView, templateDrop}`, `t3_c1f_photo_row_with_place_row_can_be_deleted`, and `commands/templates.rs`.
- **Tests (over-engineering test applied).** Every row has one behavioural test at its boundary that asserts the exact outcome. None uses call counts, private helpers or internal mocks. The UI mock of `services/templates` is the boundary that row C3 names. None of these tests could pass on a wrong outcome.
- **Assumptions.** None contradicts the intention:
  1. The effective key follows P2/P10.
  2. Writing the denial with the same store calls is an allowed delegated choice.
  3. Keeping a hand framing when the container has no presets follows P8.
  4. The `nothing_to_shift` message is a delegated choice.
  5. The step includes every changed photo, as the plan requires.
  6. Moving `role="status"` keeps the T3 rows green.
  7. `dropAction` needed no code change, which is correct.
  8. The handoff records the actor that actually ran.
- **H1–H8.** No external write (H1). The retake's file removal is the existing Deny undo-window path of 10F (H2). No pixels (H3). Each action is one transaction and re-running it is refused or makes no change (H4). No UI automation (H5). No commerce (H6). The camera file key is the P2 core mechanism that already exists (H7). Commands are thin over `psm_core` services (H8).

## Findings

### T4-R1-B1 · blocking (B1) · route: production
**What is wrong.** `can_shift_back` is true, and `shift_back` goes ahead, when the denied photo is beyond the last container (a Condition shot) and a later Condition photo placed by the template exists. Pressing the button releases the denied photo and records an undo step "Shift the rest back", but no photo moves. The button stays until it is pressed.

**Evidence** (a probe test added to the T4 test file, run, and then reverted; the checksum matched afterwards). S0 (3 containers; F4 and F5 are Condition). Deny F4 (Condition 1). Result: `can_shift_back = true`. `shift_back` returns Ok. The places of F1, F2, F3 and F5 are identical before and after. The undo label is "Shift the rest back". The same happens whenever every later template-placed photo stays where it is after the release, for example when the photos in between were placed by hand.

**Authority.**
- Book 10G §6.4: "moves every later templated photo back one container … **The button disappears when there is nothing to shift.**"
- The plan's own U2 Notes: "A denied photo beyond the last container needs no shift … the button is not offered."

The plan's task 1 rule ("a later photo placed by the template exists") is wider than both. The intention wins.

**Correction.** `can_shift_back` is true only when releasing the session's unreleased denied photos would change the expected place (classification or slot) of at least one photo that is not Deleted and was placed by the template. Otherwise it is false, and `shift_back` answers `nothing_to_shift` with no write and no step. Add a test at the row (e) boundary for this case: S0, F4 denied → `can_shift_back` false, `shift_back` refused with `nothing_to_shift`, and the undo label is still the denial's. Rows C2(a), C2(c), C2(f) and A7 keep their outcomes under this rule.

**Files the fix may touch.** `crates/core/src/services/template_placement.rs` · `crates/testkit/tests/suite/t4_template_retake_shift.rs`.

## Notes written to backlog
1. **Book wording (10G §4.2 vs §6.4).** §4.2 counts only shots "not deleted by a hand action described in §6 rule 4". But rule 4's Deny must keep counting for its container to stay empty. P2 and P10 settle this in code, but the sentence in the book says the opposite.
2. **T4·C3(a) uses `PhotoPanel`'s fallback, as T3's N4 did.** The page's path (`useWorkspaceState.templateShiftBack` → action runner → "Shifted the rest back · Undo ⌘Z") has no UI test.
3. **`cargo test -p psm-desktop` fails once after a new command** (`include_str!` of `bindings.ts` is compiled before `export_bindings` rewrites it), as the implementer disclosed. Every phase that adds a command pays one extra run.
4. **T3's N2 and N6, routed "→ T4", are still open.** T4 did not touch the reorder paths: a slot number that names no container, and the undo of a no-op reorder keeping the place mark.

## Carry-forward dispositions
| Note | Destination |
|---|---|
| 1 (10G §4.2 wording) | orchestrator: book fold |
| 2 (page path of Shift the rest back untested) | backlog: next workspace phase |
| 3 (`psm-desktop` first-run failure) | backlog: process/tooling |
| 4 (T3 N2/N6) | backlog: next phase that touches reorder in templated sessions |

## Lessons for the plans
- A rule for "when a button shows" should be written as "when pressing it would change something", or the plan's Notes should name the edge cases it excludes with a row. Here task 1's rule and its own Note disagreed, and the row that the Note cited, (e), only covered the last-photo case. The full case table for "nothing to shift" has three cases: nothing later; later but only hand-placed photos; later but every later template-placed photo stays where it is after the release (beyond the last container).

## Mutation-probe declaration
- `crates/testkit/tests/suite/t4_template_retake_shift.rs`: one probe test appended, run once, reverted with `git checkout`. `shasum` matched afterwards. No production file was touched. No database or state outside the test's temp directory was changed.

## Files written by this review
- `docs/implementation/handoffs/reviewer/handoff_T4_template_retake_and_shift_review_r1.md` (this file)
- `docs/implementation/plans/T4_template_retake_and_shift.md` (Tracker line; Review log entry)
- `docs/implementation/backlog.md` (four T4 notes appended)
