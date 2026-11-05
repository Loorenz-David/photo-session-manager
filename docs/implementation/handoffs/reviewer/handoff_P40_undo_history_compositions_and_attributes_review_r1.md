---
plan: P40_undo_history_compositions_and_attributes
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-05
actor: opus
---
# Review P40 r1: APPROVED_WITH_NOTES

Composition and attribute steps are recorded and reversed as the plan states. All 20 rows have a boundary test that fails on a wrong
outcome. Suite, lint and typecheck are green, and the perimeter matches the declared files. Nothing blocks (no B1–B4). There are six
notes; the one that matters is probe-confirmed: undoing a re-frame of a `needs_review` photo restores the old rect as `ok`. It is routed
to P41 for an owner choice.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. The re-frame undo note below is a semantic choice the book's letter currently settles (MC-30: "a reversed composition is a saved
composition"). It is raised for the orchestrator to fold into P41 planning, as a card if it decides the owner should choose.

## Gate and evidence
- Tracker read `IMPLEMENTED`. Checkpoint `69d384274590b9e0b807e2ba0c78853d30943bc9` is in `git log`.
- `git show --stat 69d3842` lists 13 paths: the plan's 10 declared paths (9 changed + `p40_undo_history_values.rs`) plus
  `master_plan.md` (tracker) and the plan file (Review log). Nothing is extra or missing.
- One run on clean tree `57495ec` (code identical to `69d3842`): `cargo test --workspace --quiet` exit 0,
  `cargo clippy --workspace --all-targets -- -D warnings` exit 0, `cargo fmt --all -- --check` exit 0, `npm run typecheck` exit 0.

## Verified correct
- **Store.** `save_composition_in(conn, …)` is the single body shared by `Store::save_composition` (own transaction) and
  `StoreTx::save_composition`. Idempotence is values + `state ok` + `origin`. A save sets `revision + 1`, `state 'ok'` and
  `review_reason NULL`, and adds `change_seq + 1` only for a non-deleted photo. `StoreTx::composition` and `delete_attribute_value` are
  thin.
- **Composition recording.** `set` and `reset` keep `before = (values(), origin)` from the read that supplies the frame. They record only
  when `changed`, labelled `Compose NN` or `Compose photo`. `apply` records one step holding only the changed photos, in the first changed
  photo's session (D14). `composition_apply` converts every item first and then calls `apply` once (task 3, refused order D12).
- **Composition reversal.** Runs in one transaction under the operation lock. Missing photo → `that photo is gone`. Photo in another
  session → `already changed`. `NeedsReview`, no frame or a failed `validate` → the fixed reasons, all read inside the transaction (Hazards).
  A `StepCannotApply` rolls the whole transaction back, so multi-photo steps never apply partly. `PhotoChanged` (per changed photo) and one
  `SessionChanged` are emitted after the lock drops, and none when nothing changed (P39 D12). `sessions = [step.session_id]`,
  `active_photo_id None`.
- **Attribute recording.** `write` takes a caller-decided `action`, and `ai_suggest` passes `None`, so it records nothing (D2). A
  `same_content` no-op returns before `change` is set, so it records nothing (M16). `write_title` records `Edit Title` or `Clear Title` only
  when the title changed (D13). `accept_all` records the suggested → accepted rows when there are any. `clear` on a non-live field writes
  and records nothing. Recording happens after the commit, through `session_for_item`.
- **Attribute reversal.** Missing item → `that product is gone`. `Row(Some)` → `put_attribute_value` restores the row byte for byte,
  `updated_at` included. `Row(None)` → delete. `Title` → `update_item_title`, which does not touch `change_seq`. Each write is skipped when
  whole-row `==` holds (D15). `set_item_activity` runs once, and only when something was written. No photo or session events are emitted
  (D10).
- **Coalescing (D3).** Merges only when redo is empty and the newest step is an `Edit …` single-field `Attribute` step with the same item
  and key. It keeps the first `before` and the latest `after`. Clear and Accept all never merge in either direction. The unit test
  `mc30_attribute_steps_coalesce` covers one merge and three negatives on the real function.
- **Copy redo (task 8).** A `NotFound` on the source read becomes `that photo is gone`, so the step is dropped.
- **Wiring.** `CompositionService { history }` is set in `integrations/src/lib.rs`. `.with_history` is on both command constructors.
  `health.rs:36`, `lib.rs:586` and the 16 test call sites are untouched. `TestApp::attributes` attaches the history.
- **Rows.** Each test observes its row's outcome at the row's boundary and fails on a wrong one:
  - C1(a): fails on a `− 1` or unchanged revision, a missing `change_seq` step, or a missing event.
  - C1(c): pins origin `user` and both labels.
  - C1(d): a second recorded step would leave V1 after one undo.
  - C2(a): p stays at revision 0, and the step label is the 2-photo one.
  - C2(c): a looping command would read `… to 1 photo`.
  - C3(a): pins the full reason, the unchanged revision and values, and the drop to `Approve 01`.
  - C4(a): compares the whole `Value`.
  - C4(b): checks the row is absent, not merely cleared.
  - C4(e): without the merge, undo would leave `"Oa"`.
  - C4(f): a merge past another field would empty K1 on the first undo.
  - C5(a): checks every row is back to `Suggested`.
  - C5(b): checks two steps.
  - C6(a): checks LIFO order across kinds.
  - C7(a): checks the exact `HistoryChanged` pair and that no photo or session event fires.
  - C7(b): checks the DTO shape and that the field is gone.
  - C8(a): checks the reason, the drop and `NothingToRedo`.
- **Assumptions.** (1) The nested Apply item is `photo_id`, matching `CompositionApplyItemDto` and the generated `bindings.ts`, so the
  plan's `photoId` is a fixture slip. (2) Apply keeps partial success and records only the changed photos. (3) Schema refresh records
  nothing and AI suggestions are not steps. All three follow tasks 1–4. None contradicts the intention.
- **H1–H8.** No external write (H1). No RAW touched (H2). No pixel work (H3). Reversals are single transactions (H4). No UI automation
  (H5). Nothing commerce-dependent (H6). No platform concept in core (H7). Commands stay thin, and `composition_apply` only converts and
  delegates (H8). No secret and no live service involved.

## Findings
None blocking.

## Notes written to backlog
1. **P40 → P41 (by probe).** Undoing the re-frame of a `needs_review` photo restores the old rect with `state ok`, so the "Re-frame"
   state does not return. This follows MC-30's letter but strains M33's "until confirmed". Probe: set V1 → flag (rotation 3°) → set V2 →
   undo gives `Ok("Compose photo")`, values V1, `state Ok`, revision 3. Remedies: restore `needs_review` on undo, or refuse the undo. Owner
   choice before P41.
2. **P40.** Undo of a cross-session Apply (D14) is always refused by the reversal's session check. No screen produces such an Apply.
3. **P40.** `apply` uses `.expect("saved photo")` on a read after the save, and `record_one` can return `Err` after a committed save.
   Both need a narrow race.
4. **P40.** A coalesced step whose final value equals its starting value is a visible no-op on ⌘Z. Cosmetic.
5. **P40 (tests).** `c4f` lacks the K1 check after the second undo, `c5b` lacks the redo-empty check before its undo, and `c2a` compares
   against p's default rather than `before_q`/`before_r` (equal in this fixture). The rows' outcomes are still pinned.
6. **P40 (plan).** C2(c)'s `photoId` fixture is wrong for a nested DTO field. Planners should copy nested names from `bindings.ts`.

## Lessons for the plans
- Treat composition `state` as part of what a step restores, or rule it out explicitly. "Before = (values, origin)" was chosen for
  `save_composition`'s idempotence check. Nobody asked what undoing a `needs_review` confirmation should mean. P41's plan, or a book
  amendment, should decide it (note 1).
- Nested wire field names in fixtures come from the generated bindings. Only top-level command arguments are camelCase.

## Write perimeter
- `docs/implementation/master_plan.md`: P40 tracker row → `APPROVED`.
- `docs/implementation/plans/P40_undo_history_compositions_and_attributes.md`: Review log entry.
- `docs/implementation/backlog.md`: 6 rows appended.
- `docs/implementation/narratives/P40_undo_history_compositions_and_attributes.md`: new.
- This handoff: new.
- Probe declaration: `crates/testkit/tests/zz_p40_review_probe.rs` was created as a copy of the P40 test helpers plus one probe test,
  run once with `cargo test -p psm-testkit --test zz_p40_review_probe`, and deleted. `git status` afterwards showed only the documents
  above. No production file was touched, and no database or state outside the test's temporary directory.
- No commits made (the runner commits).

## Carry-forward dispositions
| Note | Destination |
|---|---|
| 1 re-frame undo restores `ok` | P41 planning (orchestrator decides whether it becomes an owner card) |
| 2 cross-session Apply undo | backlog, next touch of `CompositionService::apply` / P41 if it routes grid Split/Normal through Apply |
| 3 `expect` after save | backlog, next touch of `composition.rs` |
| 4 no-op coalesced step | backlog, P41 (notice wording) or later |
| 5 secondary asserts | backlog, next touch of `p40_undo_history_values.rs` |
| 6 plan fixture naming | planner lesson |
