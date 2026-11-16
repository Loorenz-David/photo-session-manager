---
phase: T3_template_hand_changes
title: Your changes win, and there is a way back — a photo you move, re-classify or re-frame in a templated session is marked as differing from the template and is never moved back by the app; you can drop a photo onto a container (an empty one is filled, two placed photos swap); a quiet warning counts the differing photos and offers Back to template, which is one undoable step
difficulty: high
projection: no
depends_on: [T2_template_placement]
blocked_on: none (owner card 1 of `handoffs/reviewer/handoff_templates_plan_r1.md` is built as "swap" on silence)
---
# T3 — Your changes win, and there is a way back

**Tracker:** IMPLEMENTED · 2026-11-16 · claude sonnet (pipeline-implementer) · fix 1: B1 closed (undo/redo of a reorder restores the place row the drag changed); t3_template_hand 20/20, p39 24, t2 21, p02 25, p62 12, t1 12, core 96, clippy/fmt/typecheck green

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `10G` §6 rules 1, 2 and 5, by `master_plan_templates.md` §5 P7 and P9. (U1) "Differs" is computed and shown in the template view; undo of an older step keeps Main in container order. (U2) Drop onto a container: fill an empty one, swap two placed photos; one undo step. (U3) Back to template: place and presets of the differing photos, one undo step. (U4) The warning, the marks and the drop in the session view. **Not here:** a retake (a photo from outside the containers dropped onto a filled one: refused in this phase, built in T4); Deny's Shift the rest back (T4).

## How to work this plan (flow.md §10, master_plan_templates.md §3)
One runner phase made of 4 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `T3·C<n>(<row>)`; tests named after their row. Open details: the smallest option consistent with 10G and `master_plan_templates.md` §5, recorded under **Assumptions**.

| Unit | What it builds | Criteria | Rows | Changes |
|---|---|---|---|---|
| U1 | Differs in the view; order kept after an undo; a photo row can be deleted | 1 | 7 | Rust |
| U2 | Drop onto a container (fill, swap) as an undo step | 1 | 6 | Rust |
| U3 | Back to template as an undo step; commands | 1 | 7 | Rust, generated bindings |
| U4 | Warning, marks and drop in the session view | 1 | 4 | TypeScript |
| **Total** | | **4** | **24** | |

## Read first (whole phase)
`docs/process/flow.md` (§4, §4A, §6, §10) · `docs/implementation/master_plan_templates.md` (all; §5 P6–P9) · `docs/implementation/master_plan.md` §3, §7 · book `10G` §6, §8 (A5, A6) · book `05A` MC-30 (undo) · the T2 plan (the fixture Chair-3, what placement and the view do) · `crates/core/src/services/{template_placement.rs, history.rs, review.rs, composition.rs}` · then each unit's Read first.

## The fixture the rows use
Chair-3 of the T2 plan (K1, K2, K3; shot 1 → K2, shot 2 → K1, shot 3 → K3; K3 split + zoom 20). **S0** is session S after F1…F5 arrived in order: F2 in slot 1, F1 in slot 2, F3 in slot 3, F4 and F5 Condition at positions 1 and 2. "Frames adopted" means `adopt_frame` (6000×4000, camera) was called for each file after it arrived.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **Dropping a placed photo onto another filled container swaps the two** (owner card 1; on silence this is what is built). Both are then marked.
- **Dropping a photo from Condition onto a filled container is not possible yet** in this phase; T4 makes it the retake.
- **A photo you put into a container by hand keeps its own framing**; the container's split and zoom come with Back to template.
- **A re-framed photo counts as differing only when its container has presets.**
- **A denied photo is not "differing"** and Back to template never brings it back; undo does.
- **Back to template touches only the differing photos**, and uses the template as it is now.
- **A photo that was in the session before its template existed, or was moved in from another session, shows as differing**; Back to template files it by when it was shot. (This is how photos shot into Unassigned and then assigned get filed: one tap.)
- **A photo moved back by undoing a move returns without its container**; it shows as differing and Back to template files it.

## Unit 1 — Differs, and order after an undo

**Unit id:** `T3·U1` · **Rows:** T3·C1

### Goal
`session_view` reports which photos differ (P9). Undoing or redoing any older step in a templated session leaves Main in container order (P7). A photo row with a place row can be deleted.

### Read first
`master_plan_templates.md` §5 P7, P9 · `crates/core/src/services/template_placement.rs` (`session_view`, the hand-change hooks T2 added) · `crates/core/src/services/history.rs` (`HistoryService::reverse`, `apply`) · `crates/infra/src/sqlite/store.rs` (`delete_photo_row`).

### Files
Edited: `crates/core/src/services/{template_placement.rs, history.rs}` · `crates/core/src/ports/store.rs` and `crates/infra/src/sqlite/store.rs` (as needed; `delete_photo_row`) · `crates/testkit/tests/suite/main.rs` (`mod t3_template_hand;`). New: `crates/testkit/tests/suite/t3_template_hand.rs`.

### Core mechanisms
None new.

### Ordered tasks
1. `session_view`: `differs` per photo by P9 (the expected place comes from the same shot order placement uses, read at the moment of the call); the count is the number of differing photos.
2. `HistoryService::reverse`: inside the transaction, after a step was applied, Main is renumbered by P7 in every touched session that is templated.
3. `delete_photo_row` also deletes the photo's `photo_places` row.
4. Tests for the rows.

### Criteria (1 criterion, 7 rows)
Boundary: `session_view`, stored photos, on `TestApp`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| T3·C1(a) | `session_view(S)` | S0; F4 classified Main | F4: no slot, `by_hand` true, `differs` true; every other photo `differs` false; count 1 | 10G §6.2, A5 |
| T3·C1(b) | stored photos, `session_view` | S0; F2 classified Condition, then Main | F2 Main slot 1; Main positions F2 = 1, F1 = 2, F3 = 3; F2 `by_hand` true, `differs` false; count 0 | 10G §6.2 (the warning ends when the photos match again) |
| T3·C1(c) | `session_view` | S0, frames adopted; `CompositionService::set` gives F3 a valid normal composition; then the same for F1 | after the first: F3 `differs` true, count 1. After the second: F1 `differs` false, count still 1 | 10G §6.1 (compose edit), P9 |
| T3·C1(d) | `session_view` of both sessions | S0; F1 moved to S2 (type "Chair", empty) | S: no photo has slot 2, count 0. S2: F1 `by_hand` true, `differs` true, count 1 | 10G §6.1 (move to another session) |
| T3·C1(e) | same | S0; F1 copied to S2 | S: F1 slot 2, `by_hand` false, count 0. S2: the copy `by_hand` true, `differs` true | 10G §6.1 (copy) |
| T3·C1(f) | `Store::transaction` → `delete_photo_row` | a photo that has a `photo_places` row | the call succeeds; neither the photo row nor the place row exists | 10G §6.5 (undo of a copy stays possible) |
| T3·C1(g) | `HistoryService::undo` | F1 arrives (slot 2); F1 classified Condition; F2 and F3 arrive (slots 1 and 3); undo | F1 Main slot 2; Main positions F2 = 1, F1 = 2, F3 = 3 | 10G §6.5, P7 |

### Delegated choices
Whether `differs` reuses placement's reader or its own · where the renumbering helper lives.

### Notes
- Hazard: a test that passes because nothing differs proves nothing; rows (a), (c), (d) each assert a `true`.
- The copy has no place row (no row = placed by hand); nothing is written for it.

### Unit test commands
`cargo test -p psm-testkit --test suite t3_template_hand::` · `cargo test -p psm-testkit --test suite p39_undo_history_core::` · `cargo test -p psm-testkit --test suite t2_template_placement::`

## Unit 2 — Drop onto a container

**Unit id:** `T3·U2` · **Rows:** T3·C2

### Goal
`template_placement::drop_on_container(photo_id, container_no)`: the person puts a photo into a container. One transaction, one undo step.

### Read first
book `10G` §6 rules 1, 3, 5 · `crates/core/src/services/history.rs` (`Step`, `StepKind`, `names_photo`, `apply`, `check_photo`, `cannot`; how `Composition` and `Classify` steps restore state) · `crates/core/src/services/review.rs` (how a service records a step after its transaction).

### Files
Edited: `crates/core/src/services/{template_placement.rs, history.rs}` · `crates/core/src/error.rs` (`container_filled`) · `crates/core/src/domain/template.rs` · store files as needed · `crates/testkit/tests/suite/t3_template_hand.rs`.

### Core mechanisms
None new.

### Ordered tasks
1. `StepKind::Template { session_id, photos }`: per photo its state before and after: classification, position, the whole `photo_places` row (or none), the composition values with origin, and whether it has a denial row. `names_photo` covers it. Undo writes `before`, redo `after`, for every photo of the step, then renumbers Main by P7 and puts the step's Condition photos back at their recorded positions (Condition stays dense). A photo whose classification or slot is not what the step left → `StepCannotApply` ("that photo already changed"), nothing written. Events as other steps (`photo_changed` per photo, `session_changed`).
2. `drop_on_container(photo_id, n)`. Refusals: the session is not templated → `no_template`; n is not a container number of the template → `invalid_value`; the photo is Deleted or in another session → `invalid_value`. Cases, the photo being D:
   - D is the only thing asked for (D is Main with slot n) → nothing changes, no step;
   - no Main photo has slot n → D becomes Main with slot n;
   - a Main photo O has slot n (the one with the lowest position when there are several) and D is Main with a slot → D and O exchange slots;
   - a Main photo has slot n and D has no slot (Condition, unreviewed, or Main after the containers) → `container_filled`, nothing changes (T4 replaces this case).
   Every photo it moves gets `place_by_hand = 1`. Compositions are not touched. Main and Condition are renumbered (P7), the session touched, mirror tags queued for a changed classification. One step, label "Move to container <n>".
3. Tests for the rows.

### Criteria (1 criterion, 6 rows)
Boundary: `drop_on_container` and `HistoryService` on `TestApp`; stored photos, `session_view`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| T3·C2(a) | `drop_on_container` | F1 arrives (slot 2); F1 classified Condition; drop F1 on 3 | F1 Main slot 3, position 1, `by_hand` true, `differs` true | 10G §6.1 (drag to another place) |
| T3·C2(b) | same | S0; drop F3 on 2 | F3 slot 2, F1 slot 3; Main positions F2 = 1, F3 = 2, F1 = 3; F3 and F1 `by_hand` true and `differs` true; count 2 | 10G §6.1, §6.2, A6 (owner card 1 = swap) |
| T3·C2(c) | same, `HistoryService::state` | S0; drop F3 on 3 | nothing changed; no undo step | 10G §6.5 |
| T3·C2(d) | same | S0; drop F4 (Condition) on 1 | refused, code `container_filled`; S0 unchanged; no undo step | 10G §6.3 (built in T4) |
| T3·C2(e) | same | S0: drop F3 on 0; drop F3 on 4; in a session with no template: drop its photo on 1 | `invalid_value`; `invalid_value`; `no_template` | 10G §4 (templated sessions only) |
| T3·C2(f) | `HistoryService::{state, undo, redo}` | after (b) | the undo label is "Move to container 2"; undo → exactly S0 (slots, positions, `by_hand` false for F3 and F1, count 0); redo → exactly the state of (b) | 10G §6.5, MC-30 |

### Delegated choices
The shape of the step's per-photo state · helper names.

### Notes
- Hazard: recording the step inside the store transaction deadlocks nothing today, but follow `ReviewService`: collect the step in the closure, record it after the commit.
- `container_filled` is temporary by design; do not build UI text around it beyond a toast.

### Unit test commands
`cargo test -p psm-testkit --test suite t3_template_hand::` · `cargo test -p psm-testkit --test suite p39_undo_history_core::` · `cargo test -p psm-core services::history`

## Unit 3 — Back to template

**Unit id:** `T3·U3` · **Rows:** T3·C3

### Goal
`template_placement::back_to_template(session_id)` re-applies the template to the differing photos, place and presets, as one undo step; the two commands of this phase.

### Read first
book `10G` §6 rule 2, §8 A6 · `master_plan_templates.md` §5 P5–P9.

### Files
Edited: `crates/core/src/services/template_placement.rs` · `apps/desktop/src-tauri/src/{commands/templates.rs, lib.rs}` · `apps/desktop/src/services/{bindings.ts (generated), templates.ts}` · `crates/testkit/tests/suite/t3_template_hand.rs`.

### Core mechanisms
None new.

### Ordered tasks
1. `back_to_template(session_id)`: not templated → `no_template`. One transaction: every photo that differs (P9) gets a row with `place_by_hand = 0` and `frame_by_hand = 0`, goes to its expected place, and gets that place's composition (P8: the container's presets when it has any; unchanged when the container has none and the photo only differed by place; a photo going to Condition keeps its composition). Then orders (P7), touch, mirror tags. One `StepKind::Template` step, label "Back to template", naming every photo it changed. Nothing differs → nothing written, no step. Deleted photos are never touched.
2. Commands `template_drop(photo_id, container_no)` and `template_back(session_id)`; both answer the sessions touched like the review commands (`ReviewResponse`); wrappers; bindings.
3. Tests for the rows.

### Criteria (1 criterion, 7 rows)
Boundary: `back_to_template`, `HistoryService`, stored photos and compositions, `session_view`; the wired desktop command test.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| T3·C3(a) | `back_to_template(S)` | the state of T3·C2(b) | F3 slot 3, F1 slot 2; Main positions F2 = 1, F1 = 2, F3 = 3; `by_hand` false for both; count 0 | 10G §6.2, A6 |
| T3·C3(b) | same, compositions | S0, frames adopted; F3 given another valid composition by `CompositionService::set`; back | F3's composition is kind split, origin user, rect (0.1, 0.2, 0.8, 0.6); its row has `frame_by_hand` 0; count 0 | 10G §6.2 (place and presets) |
| T3·C3(c) | same | S0; F4 classified Main; back | F4 Condition position 1, F5 position 2; Main has three photos; count 0 | 10G §6.2, §4.4 |
| T3·C3(d) | `HistoryService::{state, undo, redo}` | after (a) | the undo label is "Back to template"; undo → exactly the state of T3·C2(b) (F3 slot 2, F1 slot 3, both `by_hand` true, count 2); redo → count 0 and the state of (a) | 10G §6.2 (one undoable step), A6 (undo restores the hand placement) |
| T3·C3(e) | `back_to_template`, `HistoryService::state` | S0; F1 denied (`ReviewService::delete`); back | F1 is still Deleted and has its denial row; the count was 0 before and is 0 after; the undo label is still the denial's | 10G §6.4 (Deny is not undone by the app) |
| T3·C3(f) | `session_view`, `back_to_template` | a "Stool" session without a template; G1, G2 arrive; then a template for "Stool" with two containers, shooting order K2, K1 | before: both `by_hand` true and `differs` true, count 2. After back: G1 slot 2, G2 slot 1; Main positions G2 = 1, G1 = 2; both rows `place_by_hand` 0; count 0 | 10G §6.2, P6 |
| T3·C3(g) | the wired app, commands `template_back`, `template_drop`; the generated bindings | a session with no template: `template_back`, then `template_drop` for its photo and container 1 | both answer the error code `no_template`; `bindings.ts` contains `template_drop` and `template_back` | 10G §4 |

### Delegated choices
The DTO of the two commands beyond "the touched sessions" · whether `back_to_template` shares code with `place_arrival`.

### Notes
- Back to template can leave a photo placed by hand that does not differ; that is intended (it matches).
- Hazard: the snapshot for undo must be taken before anything is written, compositions included.

### Unit test commands
`cargo test -p psm-testkit --test suite t3_template_hand::` · `cargo test -p psm-desktop` · `cargo clippy -p psm-core -p psm-infra -p psm-desktop --all-targets -- -D warnings` · `cargo fmt --all -- --check`

## Unit 4 — Warning, marks and drop in the session view

**Unit id:** `T3·U4` · **Rows:** T3·C4

### Goal
The person sees that photos differ, can go back in one tap, and can drag a photo onto a container.

### Read first
book `10G` §6 rule 2 · `apps/desktop/src/features/workspace/{PhotoPanel.tsx, ContainerGrid.tsx, PhotoTile.tsx, WorkspacePage.tsx (the function that handles a drop, `onDragEnd`), useWorkspaceState.ts (how an action's result updates the photos), useTemplateView.ts}` · `apps/desktop/src/services/templates.ts`.

### Files
New: `apps/desktop/src/features/workspace/{TemplateWarning.tsx, templateDrop.ts}` · `apps/desktop/src/features/workspace/__tests__/t3_template_warning.test.tsx`. Edited: `PhotoPanel.tsx`, `ContainerGrid.tsx`, `WorkspacePage.tsx`, `useWorkspaceState.ts` (if the action runner is there), `workspace.css`, `apps/desktop/src/services/templates.ts`.

### Core mechanisms
`templateDrop.ts::dropAction` (pure).

### Ordered tasks
1. `TemplateWarning`: shown in the photo panel above the Main section when the view's count is above 0: `role="status"`, the text "<n> photo differs from the template" / "<n> photos differ from the template", and a button **Back to template** that calls `template_back` and applies the result like any review action (the usual undo toast and ⌘Z work, as for other steps). Unobtrusive: one line, no dialog.
2. A differing photo's tile carries `data-differs` and a small mark.
3. `dropAction(photoId, overId, view, photos)` (pure): a template drop when the view is templated and `overId` is a container placeholder (`container:<n>`) → container n, or a Main photo's tile that has a slot → that slot; otherwise the default (today's reorder / classify paths). Placeholders become drop targets (`useDroppable`, id `container:<n>`). The page's drop handler asks `dropAction` first and calls `template_drop` for a template drop; a refusal is shown as the usual error toast.
4. Untemplated sessions: nothing changes.

### Criteria (1 criterion, 4 rows, all `[rule]`)
Boundary: `PhotoPanel` in a UI test (`services/templates` mocked); the pure `dropAction`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| T3·C4(a) | `PhotoPanel` | a view with count 2; press Back to template | a status line "2 photos differ from the template"; `templateBack` is called once with the session's id | `[rule]` 10G §6.2 |
| T3·C4(b) | same | a view with count 0 | no status line and no Back to template button | `[rule]` 10G §6.2 |
| T3·C4(c) | same | a view where the photo in slot 2 differs and the photo in slot 1 does not | the first photo's tile has `data-differs`; the second has not | `[rule]` 10G §6.2 (a small mark on each differing photo) |
| T3·C4(d) | `dropAction` | a templated view (three containers; photo 11 in slot 1, photo 12 in slot 3, photo 13 Main without a slot); dragged photo 12 over: `container:2`; `photo:11`; `photo:13`; `group:condition`. Then an untemplated view, over `photo:11` | container 2; container 1; default; default. Untemplated: default | `[rule]` 10G §6.1, A6, A9 |

### Delegated choices
The mark's look · the warning's exact placement inside the panel · the toast text for `container_filled` ("That container is filled").

### Notes
- Dragging within Condition, and between Main extras, stays today's behaviour.

### Unit test commands
`cd apps/desktop && npx vitest run src/features/workspace` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time. A failure is fixed and only the failing files re-run.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-testkit --test suite t3_template_hand::` · `cargo test -p psm-testkit --test suite p39_undo_history_core::` · `cargo test -p psm-testkit --test suite t2_template_placement::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/workspace` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: no path changes a photo placed by hand except `back_to_template` and the person's own action; the undo snapshot includes compositions; untemplated sessions take none of the new paths. Notes go to `docs/implementation/backlog.md` under T3.

## Manual try-out (the narrative is built from this)
On scratch data, in a dining-chair session with all containers filled and a few Condition photos:
1. Move four Condition photos to Main: a line says "4 photos differ from the template". Ignore it; shoot another photo: nothing moves back.
2. On another chair, drag the split photo from place 3 onto place 2: the two swap, both get a small mark, the line says 2 photos differ.
3. Press Back to template: both return, the line goes away. Press ⌘Z: your swap is back. ⌘⇧Z: the template again.
4. Re-frame the split photo in Compose: it is marked; Back to template gives it the split and zoom again.
5. Drop a Condition photo onto an empty container: it is placed there.

## Review log

### 2026-11-16 · implement r1 · claude-opus
**Built.** U1 (`8057056`): `session_view` reports `differs` per photo and the count by P9, using the same shot order as placement read at the call (`session_view_at(store, now, id)`; `session_view` = system clock); `HistoryService::reverse` renumbers Main by P7 in every touched templated session; `delete_photo_row` deletes the `photo_places` row. U2 (`3d7ea16`): `StepKind::Template` (per photo: classification, position, whole place row, full composition, denial time; undo/redo check session, classification and slot, restore, keep ordered groups dense); `drop_on_container` (fill, swap, no-op, `container_filled`, `invalid_value`, `no_template`), label "Move to container <n>"; error `ContainerFilled` → code `container_filled`. U3 (`8db8fe4`): `back_to_template` (differing photos only, P8 presets for containers that have them, Condition in shot order, label "Back to template"); commands `template_drop`, `template_back` (answer `ReviewResponse`), wrappers `templateDrop`, `templateBack`, bindings. U4 (`f4e9432`): `TemplateWarning` (status line + Back to template through the workspace action runner, undo toast), `data-differs` + amber dot on every differing tile, `templateDrop.ts::dropAction`, droppable placeholders, the page drop handler asks `dropAction` first; toast "That container is filled".

**Assumptions.** (1) P7's Main key is applied as (has no slot, slot, previous position): the "placed by hand" key and T2 fix 1's "hand-placed photos keep their absolute position" are replaced by this single rule, because the fixed-position rule contradicts T3·C1(g) and T3·C2(b) (backlog T2 r2 note) while the slot rule keeps T2's dragged-order test green (a drag writes the dragged photo's position and slot first; ties keep that order). (2) A photo keeps its slot in its row when it is classified or denied (the slot is the container it was last in); only a Main photo is given a new slot by a reorder (a Condition reorder no longer writes its Condition position into the slot, backlog T2 r1 note). This is what makes T3·C1(b) and T3·C1(g) come back to their container. (3) Undo/redo of a reorder in a templated Main writes the slot of the position it restores (mirror of what the drag wrote). (4) A swap also exchanges the two photos' positions. (5) Back to template keeps the composition (and any pending zoom) of a photo going to a container without presets or to Condition. (6) The template view also re-reads on `history_changed` (a review action emits no photo event, so the warning would not update after classify/compose otherwise).

**Deviations.** Outside the per-unit file lists, each needed: `apps/desktop/src-tauri/src/commands/items.rs` (exhaustive error-code match for `ContainerFilled`), `crates/core/src/ports/store.rs` (`photo_denied_at` on `StoreTx`, for the step snapshot), `apps/desktop/src/features/workspace/PhotoTile.tsx` (`TemplateDiffersContext`, `data-differs`, mark), `useTemplateView.ts` (assumption 6). `crates/core/src/domain/template.rs` was not needed (the step state lives in `history.rs`).

**Known edge (not a row).** A container photo dragged (reorder) behind the extra Main photos gets the slot number of its position, so it sorts before the extras rather than after them; the template drop replaces drags onto placed tiles, and it differs, so Back to template restores it.

**Stamp.** `cargo test --workspace` ok (1540 tests, 3 min 48 s) · `cargo clippy --workspace --all-targets -- -D warnings` ok (after one test-file type alias; T3 stem re-run ok) · `cargo fmt --all -- --check` ok · `npm run typecheck` ok · `npm test` ok (85 files, 536 tests).


### 2026-11-16 · review r1 · opus (pipeline-reviewer) · CHANGES_REQUESTED
**Verified correct.**
- Perimeter: `8057056^..ab51a43` = the 24 declared paths.
- Checks green, run once each on `58f285f`:
  - `t3_template_hand::` 19, `p39_undo_history_core::` 24, `t2_template_placement::` 21, `psm-desktop` 29
  - clippy (core/infra/desktop/testkit), fmt
  - vitest workspace 15 files / 83 tests, typecheck
  - The implementer's full-suite stamp stands.
- U1: `differs` matches P9 (Deleted and released never; by hand or no row vs the expected place from the shared `shot_indexes`; frame by hand only in its expected container with presets). `reverse` renumbers templated Main. `delete_photo_row` removes `photo_places`.
- U2:
  - `drop_on_container` refusals: `no_template`, `invalid_value` ×2, `container_filled`.
  - Cases: no-op, fill, swap with positions.
  - `StepKind::Template` snapshot: classification, position, row, full composition, denial. Pre-checks run before any write.
- U3: `back_to_template` acts only on differing photos, current template, presets only where the container has them, Condition in shot order, one step. Commands are thin; bindings regenerated.
- U4: warning, marks, pure `dropAction`, droppable placeholders.
- Rows: all 24 tests observe their row's outcome at the row's boundary and fail on a wrong one. No orphans.
- Review-mode reads hold: hand-placed photos change only by Back or the person's action; the snapshot includes compositions; untemplated sessions take no new path.
- H1–H8 hold. No assumption contradicts the book.

**Findings.**
- **T3-R1-B1 (B1, route production).** Undo/redo of a `Reorder` in a templated Main writes `slot = <restored position>` (assumption 3), and `reverse` now renumbers by (no slot, slot, position). An extra Main photo (no slot) dragged onto another extra therefore gets a slot on undo and keeps sorting first: the undo is a visible no-op.
  - Probe: S0, F4 and F5 → Main (4, 5); `reorder(F5, 4)`; undo → F4 5, F5 4, slots (None, 5).
  - It is a regression: T2's `reverse` restored it.
  - Authority: 05A MC-30 (reorder: undo restores the prior position), 10G §6.5, U1 goal.
  - Fix: restore the slot the photo had before the drag on undo, and the drag's slot on redo (record it in the step). Add one test at `HistoryService::undo`/`redo` with the probe input.
  - Files: `services/{history.rs, review.rs}`, the store port/impl only if a slot-clearing write is needed, `t3_template_hand.rs`.

**Notes → backlog (T3).** N1: P7 in `master_plan_templates.md` is stale against assumption 1 (fold upstream) · N2: a reorder still writes non-container slot numbers · N3: `container_filled` string not seen by a command-level test · N4: the page's Back to template path (undo toast) has no UI test · N5: guide update carried to T4.

**Probes.** Two scratch tests appended to `t3_template_hand.rs`, run, and removed (`git checkout`, checksum identical). No production code edited.

### 2026-11-16 · fix 1 · claude sonnet (pipeline-implementer) · IMPLEMENTED
- **T3-R1-B1 fixed.** `StepKind::Reorder` now carries `place_before` and `place_after`, each (slot, placed by hand), read from the photo's place row in `ReviewService::reorder` around the drag's `photo_place_mark`. `apply` in `history.rs` writes `place_before` on undo and `place_after` on redo through `photo_place_row`/`photo_place_put`, in place of the old `photo_place_mark(…, Some(target))`. An extra Main photo that had no slot has none after undo, so `renumber_main` leaves the positions `reorder` restored.
- **Judgment call.** Undo also restores `place_by_hand` as it was before the drag (the finding says "as it was before the drag"); an untemplated session writes no row, so nothing is written there. No store port change was needed.
- **Test.** `t3_c1g_undo_of_a_reorder_among_extra_main_photos_restores_the_order` (probe input: F4, F5 → Main; `reorder(F5, 4)`; undo → F4 4, F5 5; redo → F5 4, F4 5).
- **Runs.** `t3_template_hand::` 20, `p39_undo_history_core::` 24, `t2_template_placement::` 21, `p02_review_order::` 25, `p62_retention` 12, `t1_templates::` 12, `cargo test -p psm-core` 96, clippy (core/infra/desktop/testkit), fmt, typecheck. The full suite was not run: the change is local to the Reorder step.
