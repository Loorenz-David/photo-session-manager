---
plan: T2_template_placement
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-16
actor: claude opus 5.5 (pipeline-reviewer)
---
# T2 — Photos file themselves · review r1

**Verdict: CHANGES_REQUESTED.** One blocking finding (B1). Everything else holds.

## Review history
First review of this phase. Units 1–2 came from Codex. Unit 3 was started by Codex and finished by Opus. No earlier review rounds.

## Verified correct
- **Gate.** The tracker read IMPLEMENTED. Checkpoints `efcc86c` (U1), `0bff768` (U2), `fa8a484` (U3 partial), `018dc27` (U3) and `9005a11` (closing) are all in `git log`.
- **Perimeter.** I checked `git show --stat` for each checkpoint. The committed paths are exactly the handoff's declared whole-phase list, plus the two disclosed deviations (the nullable `slot`/`template_id` DTO typing and its regenerated bindings; the `p01_storage_identity` table list), plus the plan and handoff. Nothing is extra or missing. `crates/core/src/domain/template.rs` is listed under U1 in the plan but was edited in U2's commit, which is inside the phase perimeter.
- **Checks** (Review mode, run once on code tree `9005a11`, one cargo command at a time):
  - core: `cargo test -p psm-core mechanisms::shot_order` 10/10 and `mechanisms::template` 4/4
  - behaviour: `cargo test -p psm-testkit --test suite t2_template_placement::` 20/20
  - desktop: `cargo test -p psm-desktop` 28/28; `bindings.ts` was regenerated with no change
  - UI: `npx vitest run src/features/workspace` 14 files, 79 tests, including `t2_template_containers` 5/5
  - `cargo clippy -p psm-core -p psm-infra -p psm-desktop -p psm-testkit --all-targets -- -D warnings`, `cargo fmt --all -- --check` and `npm run typecheck`: all green
  - The implementer's full-suite stamp stands: Rust 921 after the p01 fix, UI 532.
- **P4, `shot_order::order`**, read step by step:
  1. The sort key is (time, shots before holds, photo id).
  2. Runs cover holds and counter shots only. A run ends at a gap over `FIFO_WINDOW_MS` or at a second prefix. A shot without a counter keeps its slot and does not break a run.
  3. The restart rule (+10000 to numbers more than 5000 below the largest) matches P4. Ties go by time, then id. The slot formula `min(prev + max(1, Δc), m − (k − j))` is correctly turned into 0-based form. The run's holds refill the slots left over.
  4. There is no panic path: assigned slots strictly increase and stay ≤ m − 1, and the holds left over equal m − k.
- **P5, P8.** `expected` maps shot index → the container with that `shoot_no`, and anything past n → Condition. `zoomed` keeps the centre. `preset_values` keeps turns, flip and straighten, uses orientation Named, and builds the rect from `default_rect` on the turned frame.
- **P1–P3 in the store.**
  - P1: `template_for_session` matches on `type_key` of the item's type. Unassigned (no item) gets no template.
  - P2: `shot_keys` takes `key_at` → `captured_at` → `created_at`, and `key_name` → the newest done, non-duplicate delivery's `file_key`. It covers every classification.
  - P3: `hold_times` takes exposures that are `expecting`, have no non-duplicate delivery of their own, and fall inside the window.
- **P6/P7, `place_arrival`.**
  - One transaction.
  - Skipped: photos placed by hand, Deleted photos and photos with no row.
  - Main is re-sorted by (no slot, slot, by hand, previous position). Template-placed Condition photos take the Condition positions in shot order.
  - Presets apply only when a photo's place changed, or to the new photo.
  - Mirror-tag jobs are queued on a classification change.
  - Events are emitted after the commit (`photo_changed` for changed photos other than the new one, then `session_changed`). There is no history step.
- **Review-mode reading checks.**
  - `adopt_frame_tx` changed only in its first-rect branch.
  - An untemplated session gets no `photo_places` row on any path: `photo_place_mark` checks P1 itself, and placement returns before any write.
  - A failed placement cannot fail an arrival: it runs in its own transaction, which rolls back, and `accept` logs a warning and goes on.
- **Criterion rows.** All 38 rows (C1(a)–C8(e)) have a test that observes the row's outcome at its boundary and would fail on a wrong one. C6(b)'s test of the wired command and the bindings is `apps/desktop/src-tauri/src/lib.rs::t2_c6b_session_view_command_and_bindings` (the handoff's map names the core test instead; see the notes). C8(e) counts calls on the mocked read service, which is the boundary the row names.
- **UI.**
  - `ContainerGrid` shows the containers in final order. A filled container shows its Main photos as today's tiles. An empty one shows a single placeholder with `data-container`/`data-empty` and its number. After the containers come Main photos with no slot or with a slot that matches no container, then the unreviewed photos.
  - Placeholders are not drag items, not counted (the heading uses `order.main.length`) and not photos.
  - The info button appears only when the view is templated. The dialog lists number, black-and-white example, "Shot n" and presets in words.
  - `useTemplateView` reloads only for its own session's three event kinds, and a failed read leaves the view untemplated.
- **Hard constraints H1–H8 hold.** No external write and no RAW touched. No new device concept in core: it reuses the existing ledger tables and `seq_of`. The command is thin over `template_placement::session_view`.
- **Assumptions.** All three (one transaction, the placeholder/button/dialog look, an orphan slot shown after the containers) are within 10G §5 and the delegated choices. None contradicts the intention.

## Findings

### B1-1 · blocking · route: production
**What is wrong.** In a templated session, dragging a Main photo to another place is undone, or lands one place off.
- `ReviewService::reorder` calls `photo_place_mark(photo, place = true, …, slot = Some(<target position>))`, then `renumber_main`.
- `renumber_main` sorts Main by P7: (no slot, slot, **placed by hand**, previous position). A template-placed photo therefore wins a shared slot.
- So a photo dragged onto a place a template photo holds is sorted back behind that photo.
- The undo list still gets a step for the move, although the photo did not move there.

**Probe.** A temporary test appended to `t2_template_placement.rs`, run once, then removed with `git checkout`; the file's checksum matched before and after.
- Chair-3, F1/F2/F3 arrived, giving positions F2 = 1, F1 = 2, F3 = 3.
- `reorder(F1, 1)`: F1 is **still at position 2** (slot 1, by hand). F2 stays at 1.
- Then `reorder(F3, 1)`: F3 lands at **position 2**, F1 at 3, F2 stays at 1.
- Untemplated control: `reorder(G2, 1)` moves G2 to 1.

**Authority.**
- 10G §6.1: "drag to another place … marks it placed by hand. The app never reverts it."
- The plan's recorded point: "In this phase a hand change is kept and nothing more."
- Plan U2 task 6 (reorder sets `place_by_hand`).
- This is the export/publish order, so it is a boundary the user observes.
- T3 does not remove the path. T3's `dropAction` sends a drag onto a slotless photo to today's reorder. With the same code, dragging F1 onto the slotless photo at position 4 gives it `slot = 4`, which is no container, and it lands at position 3.

**Correction.**
- Required outcome: after a reorder in a templated session, the Main order is the one the person made, and later arrivals do not move that photo relative to it.
- P7's sort key cannot put a hand-placed photo ahead of a template-placed one, so no slot value fixes this under P7 as written. The orchestrator first settles P7's rule for a hand-reordered photo. Suggested rule: a Main photo with a row and `place_by_hand = 1` keeps the position the person gave it, and template-placed and no-row Main photos fill the other positions in P7 order. With this rule C4(d), C7(a) and C7(c) keep their outcomes.
- Add one behaviour test for the reorder outcome.

**Files the fix may touch:**
- `crates/core/src/services/template_placement.rs` (`renumber_main` and the Main sort in `place_arrival`)
- `crates/core/src/services/review.rs` (`reorder`)
- `crates/infra/src/sqlite/store.rs` (`photo_place_mark`), only if the slot write changes
- `crates/testkit/tests/suite/t2_template_placement.rs`

No other finding blocks.

## Notes written to backlog
Five, under T2 in `docs/implementation/backlog.md`:
1. C5(a) asserts F1's kind and origin but not its rect.
2. A crash, or a failed placement, between the arrival insert and placement leaves the photo with no row. P6 then treats it as placed by hand for good (H4-adjacent).
3. A reorder inside Condition writes the Condition position into `slot`. It is harmless now, but T3's "differs" may misread it.
4. The handoff maps C6(b) to the core test; the real boundary test is in `src-tauri/src/lib.rs`.
5. The agent guide is not yet updated for `photo_places` and the placement service (T4 owns it).

## Lessons for the plans
- **L1 (P7 vs 10G §6.1).** P7's tie rule (template-placed before hand-placed on a shared slot) was written for "a hand-placed photo sits in a container another shot expects". It also decides where a hand-reordered photo goes, and the plan's task 6 asked for a renumber after every reorder. The contract needs one sentence saying where a photo placed by hand through reorder sits in Main. T3's drop (fill, swap) and T4's shift also renumber Main, so the same sentence protects them.
- **L2.** The plan's Unit 2 test commands did not include `p01_storage_identity::`, which lists every table, so the new table showed up only at the closing gate. A plan that adds a table could name p01 in its unit commands.

## Probe declaration
- Touched: `crates/testkit/tests/suite/t2_template_placement.rs` (one test appended, run once, restored by `git checkout`; `shasum` db342aed… matched before and after).
- No production file was edited. No database or app state outside the test's temporary folders.
- My write perimeter:
  - this handoff
  - the Tracker line and Review log entry of `docs/implementation/plans/T2_template_placement.md`
  - five lines appended to `docs/implementation/backlog.md`

## ⚠ OWNER DECISIONS REQUIRED (0)
None. The book answers the outcome (10G §6.1: a hand change is never reverted). The P7 wording is a planning matter for the orchestrator (L1).
