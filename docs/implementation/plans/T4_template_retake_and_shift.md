---
phase: T4_template_retake_and_shift
title: Retakes and bad shots — a photo from outside the containers dropped onto a filled container takes its place and the old one is deleted (red, undoable); denying a photo leaves its container empty, and "Shift the rest back" moves the later photos back one container when you reshot at once; the owner's whole story is proven in one test, and the agent guide describes the feature
difficulty: high
projection: no
depends_on: [T3_template_hand_changes]
blocked_on: none (owner card 1 of `handoffs/reviewer/handoff_templates_plan_r1.md`: if the owner answered "always delete", the orchestrator writes that into this plan's Points before launch; unanswered, the plan stands as written)
---
# T4 — Retakes and bad shots

**Tracker:** IMPLEMENTED · 2026-11-16 · claude opus 5.5 (pipeline-implementer) · 3 units, 14 rows tested (11 Rust behaviour incl. the one-test owner's story, 1 wired command, 2 UI); closing gate green (Rust suite 952, UI 538, clippy, fmt, typecheck)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `10G` §6 rules 3, 4 and 5 and §8, by `master_plan_templates.md` §5 P10. (U1) The retake. (U2) Deny leaves the container empty; Shift the rest back. (U3) The button, the owner's story as one test, the guide. **Not here:** anything of 10G §7 (no image analysis, no automatic recovery).

## How to work this plan (flow.md §10, master_plan_templates.md §3)
One runner phase made of 3 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `T4·C<n>(<row>)`; tests named after their row. Open details: the smallest option consistent with 10G and `master_plan_templates.md` §5, recorded under **Assumptions**.

| Unit | What it builds | Criteria | Rows | Changes |
|---|---|---|---|---|
| U1 | The retake | 1 | 4 | Rust |
| U2 | Deny leaves the container empty; Shift the rest back | 1 | 7 | Rust, generated bindings |
| U3 | The button; the owner's story; the guide | 2 | 3 | TypeScript, one test, documents |
| **Total** | | **4** | **14** | |

## Read first (whole phase)
`docs/process/flow.md` (§4, §4A, §6, §7A, §10) · `docs/implementation/master_plan_templates.md` (all; §5 P2, P6–P10) · `docs/implementation/master_plan.md` §3, §7 · book `10G` §6, §8, §9 · the T2 and T3 plans (fixture Chair-3, state S0, `StepKind::Template`, `drop_on_container`) · `crates/core/src/services/{template_placement.rs, history.rs, review.rs (delete)}` · then each unit's Read first.

## The fixture the rows use
Chair-3 and S0 of the T2 and T3 plans (K1, K2, K3; shot 1 → K2, shot 2 → K1, shot 3 → K3; K3 split + zoom 20; S0: F2 slot 1, F1 slot 2, F3 slot 3, F4 and F5 Condition 1 and 2). "Frames adopted" = `adopt_frame` (6000×4000, camera) for each file after it arrived.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **A retake is a photo that is not in a container** (a Condition photo, an extra Main photo) dropped onto a filled container. It takes the container, gets its split, shape and zoom, and stands in for the shot it replaced, so no warning follows. Two photos that both sit in containers still swap (owner card 1).
- **The replaced photo is denied like any denied photo**: red, undoable, its file deleted when the undo window closes.
- **A denied photo keeps its container empty** and still counts as a shot, so later photos stay where they are, until you press Shift the rest back.
- **Shift the rest back shows only when a later photo, placed by the template, exists.** It moves those photos back by as many containers as there are denied shots before them; photos you placed by hand stay.
- **Undo order:** undoing Shift the rest back brings the gap back; undoing once more brings the denied photo back.

## Tests that pin what this plan changes
| Test | File | Pins | What happens, where |
|---|---|---|---|
| `t3_c2d_…` (a photo without a slot dropped on a filled container is refused `container_filled`) | `crates/testkit/tests/suite/t3_template_hand.rs` | the temporary refusal | U1: deleted; T4·C1(a) replaces it. The error code `container_filled` is removed (`error.rs`, `master_plan_templates.md` §4, the toast text in the UI) |
| T3·C4(d) (`dropAction`) | `apps/desktop/src/features/workspace/__tests__/t3_template_warning.test.tsx` | which drops are template drops | U1: one input changes: a photo **without** a slot over a filled container's tile is now a template drop on that container; a photo without a slot over a tile without a slot stays default. The row keeps its id |

## Unit 1 — The retake

**Unit id:** `T4·U1` · **Rows:** T4·C1

### Goal
The fourth case of `drop_on_container`: D has no slot and a Main photo O has slot n.

### Read first
book `10G` §6 rule 3, §9 answer 2 · `master_plan_templates.md` §5 P2, P8, P10 · `crates/core/src/services/review.rs` (`delete`: what a denial writes) · `crates/core/src/services/template_placement.rs` (`drop_on_container`).

### Files
Edited: `crates/core/src/services/template_placement.rs` · `crates/core/src/error.rs` · `crates/core/src/services/history.rs` (only if the step's state needs a field) · `apps/desktop/src/features/workspace/{templateDrop.ts, __tests__/t3_template_warning.test.tsx}` · `docs/implementation/master_plan_templates.md` §4 (remove `container_filled`) · `crates/testkit/tests/suite/{main.rs, t3_template_hand.rs}`. New: `crates/testkit/tests/suite/t4_template_retake_shift.rs`.

### Core mechanisms
None new.

### Ordered tasks
1. `drop_on_container`, D without a slot, O in slot n: in the one transaction O is denied exactly as `ReviewService::delete` denies (Deleted, no position, the denial row, the mirror tag) and its row gets `released = 1`; D becomes Main with slot n, `place_by_hand = 1`, `frame_by_hand = 0`, `key_at` and `key_name` = O's key (O's own overrides when it has them, else O's `captured_at` and camera file key), and gets container n's composition (P8). Orders (P7). One `StepKind::Template` step, label "Retake container <n>", restoring both photos (O's denial row is removed by the undo).
2. `dropAction`: a dragged photo without a slot over the tile of a Main photo that has a slot → a template drop on that slot.
3. Remove `container_filled`.
4. Tests for the rows; the table above.

### Criteria (1 criterion, 4 rows)
Boundary: `drop_on_container`, `HistoryService`, stored photos, compositions, `photo_denial`, `session_view`, on `TestApp`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| T4·C1(a) | `drop_on_container` | S0, frames adopted; drop F4 on 3 | F3 is Deleted, has a denial row, `released` 1. F4 is Main slot 3, `by_hand` true, `differs` false; its composition is kind split, origin user, rect (0.1, 0.2, 0.8, 0.6). Main positions F2 = 1, F1 = 2, F4 = 3; F5 Condition position 1; count 0; `can_shift_back` false | 10G §6.3, A8 |
| T4·C1(b) | arrival after a retake | then F6 arrives | F6 Condition position 2; F4 still slot 3; F1, F2, F5 unchanged | 10G §4.4, P10 |
| T4·C1(c) | `HistoryService::{state, undo, redo}` | after (a) | the undo label is "Retake container 3"; undo → exactly S0 with frames (F3 Main slot 3, no denial row, `released` 0; F4 Condition position 1, composition kind normal origin default, no key override; F5 position 2); redo → exactly the state of (a) | 10G §6.3 (undoable as today), §6.5 |
| T4·C1(d) | `drop_on_container` | after (a); drop F5 on 3 | F4 is Deleted and `released`; F5 is Main slot 3 with `key_name` `DSC00003.ARW`; `differs` false; count 0 | 10G §6.3, P10 |

### Delegated choices
How the key is copied · whether the denial is written by calling into `ReviewService` or by the same store calls.

### Notes
- Hazard: the denied photo must be `released`, or Shift the rest back (U2) would offer to shift after every retake.
- Hazard: the storage job deletes a denied photo's files when the undo window closes and drops history steps that name it (`History::retire_photo`); `names_photo` already covers `StepKind::Template` (T3).

### Unit test commands
`cargo test -p psm-testkit --test suite t4_template_retake_shift::` · `cargo test -p psm-testkit --test suite t3_template_hand::` · `cd apps/desktop && npx vitest run src/features/workspace/__tests__/t3_template_warning.test.tsx`

## Unit 2 — Deny leaves the container empty; Shift the rest back

**Unit id:** `T4·U2` · **Rows:** T4·C2

### Goal
A denied photo keeps its place in the count (T2 already counts it); the view says when shifting is possible; `shift_back` does it as one undo step.

### Read first
book `10G` §6 rule 4, §9 answer 1, §8 A7 · `master_plan_templates.md` §5 P2, P6, P10.

### Files
Edited: `crates/core/src/services/template_placement.rs` · `crates/core/src/error.rs` (`nothing_to_shift`) · `apps/desktop/src-tauri/src/{commands/templates.rs, lib.rs}` · `apps/desktop/src/services/{bindings.ts (generated), templates.ts}` · `crates/testkit/tests/suite/t4_template_retake_shift.rs`.

### Core mechanisms
None new.

### Ordered tasks
1. `session_view.can_shift_back`: true when the session is templated and has a Deleted photo that is not `released` and a photo that is not Deleted, has a row with `place_by_hand = 0`, and has a greater shot index than that Deleted photo.
2. `shift_back(session_id)`: not templated → `no_template`; `can_shift_back` false → `nothing_to_shift`. One transaction: every Deleted, not released photo of the session gets `released = 1` (a row is created when missing); then every photo with `place_by_hand = 0` that is not Deleted is set to its expected place with that place's composition (P6, P8) and the orders are renumbered (P7). One `StepKind::Template` step, label "Shift the rest back", naming every photo whose row, place or composition changed (the released ones included).
3. Command `template_shift_back(session_id)` (answers like `template_back`); wrapper; bindings.
4. Tests for the rows.

### Criteria (1 criterion, 7 rows)
Boundary: `ReviewService::delete`, `shift_back`, `HistoryService`, stored photos, compositions, `session_view`; the wired desktop command test.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| T4·C2(a) | `ReviewService::delete`, `session_view` | F1, F2, F3 arrive; F2 denied | F2 Deleted; F1 slot 2 and F3 slot 3 as before; no Main photo has slot 1; `can_shift_back` true | 10G §6.4 (leaves its container empty), A7 |
| T4·C2(b) | arrival | then F4 arrives | F4 is Condition; F1 and F3 unchanged | 10G §6.4 (later photos do not move), P2 |
| T4·C2(c) | `shift_back` | F1, F2, F3, F4 arrive (F3 in slot 3, F4 Condition), frames adopted; F2 denied; shift | F3 slot 1 with composition kind normal, origin default; F4 Main slot 3 with composition kind split, rect (0.1, 0.2, 0.8, 0.6); no Condition photo; F2 `released` 1; `can_shift_back` false | 10G §6.4 (Shift the rest back), A7 |
| T4·C2(d) | `HistoryService::{state, undo}` | after (c) | the undo label is "Shift the rest back"; undo → the state before the shift (F3 slot 3 split, F4 Condition position 1 with its default composition, F2 Deleted and not released, `can_shift_back` true); a second undo → F2 Main slot 1 | 10G §6.4 (one undoable step), §6.5 |
| T4·C2(e) | `session_view`, `shift_back` | S0; F5 denied | `can_shift_back` false; `shift_back` is refused, code `nothing_to_shift`; the undo label is still the denial's | 10G §6.4 (the button disappears when there is nothing to shift) |
| T4·C2(f) | `session_view` | F1, F2, F3 arrive; F3 classified Condition (by hand); F2 denied | `can_shift_back` false | 10G §6.4 (templated photos only), §6.1 |
| T4·C2(g) | the wired app, command `template_shift_back`; the generated bindings | a session with no template | the error code `no_template`; `bindings.ts` contains `template_shift_back` | 10G §4 |

### Delegated choices
Whether `shift_back` shares its re-placing with `place_arrival` · the message of `nothing_to_shift`.

### Notes
- T2 already counts denied photos (P2); row (b) proves it stays so.
- A denied photo beyond the last container needs no shift: every later shot is Condition either way, and row (e) says the button is not offered.

### Unit test commands
`cargo test -p psm-testkit --test suite t4_template_retake_shift::` · `cargo test -p psm-testkit --test suite p66_approve_on_arrival_and_session_order::` · `cargo test -p psm-desktop` · `cargo clippy -p psm-core -p psm-infra -p psm-desktop --all-targets -- -D warnings` · `cargo fmt --all -- --check`

## Unit 3 — The button, the owner's story, the guide

**Unit id:** `T4·U3` · **Rows:** T4·C3–C4

### Goal
The button in the session; 10G §8 as one sequential test; the agent guide current (flow.md §7A).

### Read first
book `10G` §8 (A1–A9, the acceptance story) · `apps/desktop/src/features/workspace/{PhotoPanel.tsx, TemplateWarning.tsx}` · `docs/guide/{README.md, 01_product.md, 02_architecture.md, 04_change_impact.md, 05_working.md}` (§Keeping this guide current).

### Files
Edited: `apps/desktop/src/features/workspace/{PhotoPanel.tsx, TemplateWarning.tsx, workspace.css}` · `apps/desktop/src/services/templates.ts` · `crates/testkit/tests/suite/t4_template_retake_shift.rs` · `docs/guide/{README.md, 01_product.md, 02_architecture.md, 04_change_impact.md}`. New: `apps/desktop/src/features/workspace/__tests__/t4_template_shift.test.tsx`.

### Core mechanisms
None.

### Ordered tasks
1. A button **Shift the rest back** in the photo panel (beside the warning's place, shown also when nothing differs) while the view's `can_shift_back` is true; it calls `template_shift_back` and applies the result like any review action.
2. The story test `t4_c4a_the_owners_story`, one test, in this order, each step asserted before the next:
   - **A1** make a template for "Dining chair" from four example images; put them in another final order; container 3: split + zoom 20; set the shooting order (shot 1 → container 2, shot 2 → container 1, shot 3 → container 3, shot 4 → container 4); `kill_and_restart`; the template reads back the same.
   - **A2** a new session of a "Dining chair" item: the view is templated with four containers and no photo.
   - **A3** three shutters reported; files 1, 3, 2 arrive in that order: after file 3, container 3 holds it and container 1 is empty; after file 2 the containers hold files 2, 1, 3; with frames adopted, file 3 is split with rect (0.1, 0.2, 0.8, 0.6).
   - **A4** file 4 fills container 4; files 5–8 arrive as Condition.
   - **A5** files 5–8 are classified Main: the count is 4; file 9 arrives as Condition and files 5–8 are still Main.
   - **A6** a second "Dining chair" session with files 1–4 placed: drop the photo of container 3 on container 2: count 2; Back to template: count 0 and the photo is in container 3; undo: count 2.  Redo.
   - **A7** in that session deny the photo in container 4: the container is empty, `can_shift_back` is false (nothing later); file 5 arrives as Condition; `can_shift_back` is true; shift: file 5 is in container 4.
   - **A8** file 6 arrives (Condition); drop it on container 1: the photo that was there is Deleted with a denial row; file 6 is in container 1; count 0.
   - **A9** a session of a type without a template: three files arrive; all Main at positions 1, 2, 3; no `photo_places` row.
3. The guide: `01_product.md` (the shooting template in the workflow and vocabulary; 10G out of the book-pending list if it is there) · `02_architecture.md` (a flow "Shooting templates": editor, placement on arrival, hand changes, the way back; the modules of `master_plan_templates.md` §4; migrations 014 and 015 and `<data>/templates/` in §Persistence; the commands; the frontend files) · `04_change_impact.md` (rows: a change to the shot order or the placement contract → `mechanisms/shot_order.rs`, `master_plan_templates.md` §5, `t2`; a new person action on a photo must mark it by hand → `template_placement.rs`, `t3`; deleting a photo row → `photo_places`; `adopt_frame_tx`'s first branch → `pending_zoom`; a new `TemplateView` field → bindings and `useTemplateView`) · `README.md` (authority row for 10G and `master_plan_templates.md`; reading-order row "Touch shooting templates").

### Criteria (2 criteria, 3 rows; C3 `[rule]`)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| T4·C3(a) | `PhotoPanel` (UI test, `services/templates` mocked) | a view with `can_shift_back` true and count 0; press Shift the rest back | the button is shown; `templateShiftBack` is called once with the session's id | `[rule]` 10G §6.4 |
| T4·C3(b) | same | a view with `can_shift_back` false | no Shift the rest back button | `[rule]` 10G §6.4 |
| T4·C4(a) | `TestApp`: the services and stored state named in task 2 | the story of task 2 | every step's stated outcome holds, in order | 10G §8 A1–A9 |

### Delegated choices
The button's look and exact place · wording of the guide.

### Notes
- The story test repeats outcomes other rows prove; it is kept because 10G §8 is the acceptance, and it must stay one test.
- In A7 container 4 is the last container; that is why the button appears only after file 5 arrived.
- Each session of the story uses its own camera file names (`arrival_photo` files a repeated name with the same content as a duplicate); "file 1" means that session's first file.
- The guide is documents only; no book chapter, no `STATUS.md`.

### Unit test commands
`cargo test -p psm-testkit --test suite t4_template_retake_shift::` · `cd apps/desktop && npx vitest run src/features/workspace` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time. A failure is fixed and only the failing files re-run.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-testkit --test suite t4_template_retake_shift::` · `cargo test -p psm-testkit --test suite t3_template_hand::` · `cargo test -p psm-testkit --test suite t2_template_placement::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/workspace src/features/settings` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check: the table "Tests that pin what this plan changes" was handled as written; `container_filled` is gone from code, bindings and UI; the guide rows name real paths. Notes go to `docs/implementation/backlog.md` under T4.

## Manual try-out (the narrative is built from this)
On scratch data, with the dining-chair template:
1. Shoot the chair. One angle came out badly: press Deny on it. Its container is empty and nothing else moves.
2. If you reshot it at once, the photos after it are each one container too far: press **Shift the rest back**. They move back one. ⌘Z brings the gap back.
3. If you reshot it at the end instead, the new photo is under Condition: drag it onto the container of the bad angle. The old photo turns red (deleted, ⌘Z brings it back), the new one sits in the container with its split and zoom, and no warning shows.
4. Run through your whole story once: template, new chair, shoot out of order, Condition after the last angle, four Condition photos to Main, a wrong drag and Back to template.

## Review log

### 2026-11-16 · implement r1 · claude opus 5.5 (pipeline-implementer)
**Built.** U1 (`2015c99`): the retake is the fourth case of `drop_on_container` (`template_placement.rs::retake`): the occupant is denied as `ReviewService::delete` denies (Deleted, removed from its group, `place_by_hand = 1`, slot kept, denial row, mirror tag) and gets `released = 1`; the dropped photo becomes Main in slot n, `place_by_hand = 1`, `frame_by_hand = 0`, takes the occupant's effective shot key (`shot_keys`: `key_at`/`key_name` overrides, else captured time and camera file key) and container n's presets; one `StepKind::Template` step "Retake container <n>". `Error::ContainerFilled` and the code `container_filled` are gone (core, `commands/items.rs` mapping, the UI toast text in `useWorkspaceState.ts`, `master_plan_templates.md` §4); `t3_c2d_…` deleted; T3·C4(d) gained the slot-less inputs. U2 (`332237d`): `TemplateView.can_shift_back` (`can_shift_back` in `template_placement.rs`), `shift_back` (one transaction, step "Shift the rest back" naming every photo whose state changed), `Error::NothingToShift` (`nothing_to_shift`), command `template_shift_back`, wrapper `templateShiftBack`, bindings regenerated. The re-placing loop of `place_arrival` is now `place_template_photos`, shared by `place_arrival` and `shift_back` (no behaviour change; t2 21/21, p66 6/6). U3 (`6e026d2`): Shift the rest back in `TemplateWarning` (shown while `can_shift_back`, also when nothing differs), wired through `PhotoPanel` → `WorkspacePage` → `useWorkspaceState.templateShiftBack` (the action runner + undo toast, like Back to template); the story test `t4_c4a_the_owners_story`; the guide (`README.md`, `01_product.md`, `02_architecture.md`, `04_change_impact.md`).

**Judgment calls.** (1) The retake copies the occupant's *effective* shot key (P2 `at` falls back to `created_at` when there is no `captured_at`), read through `shot_keys`, so a retake of a retake carries the original key on (row d). (2) The denial is written with the same store calls as `ReviewService::delete` (not by calling it: one transaction, one step). (3) P8 on a retake: presets are applied when the container has them; a container without presets resets the composition only if the person had not framed the photo (`frame_by_hand` was 0), so a person's framing is not lost. (4) `nothing_to_shift` message: "There is nothing to shift back". (5) The step of `shift_back` snapshots every photo of the session before and after and keeps those whose state differs. (6) `role="status"` moved from the warning's outer box to the "n photos differ" text, so a line showing only Shift the rest back is not announced as a warning (T3·C4(a)/(b) unchanged and green). (7) `dropAction` already treated any dragged photo over a placed tile as a template drop; task 2 needed no code change, only the test inputs and the comment.

**Deviations (perimeter).** Edited outside the units' file lists because the tasks require them: `apps/desktop/src-tauri/src/commands/items.rs` (the exhaustive `Error` → code match: remove `container_filled`, add `nothing_to_shift`), `apps/desktop/src/features/workspace/useWorkspaceState.ts` (U1: the `container_filled` toast text the plan's table says to remove; U3: `templateShiftBack` through the action runner, "applies the result like any review action"), `apps/desktop/src/features/workspace/WorkspacePage.tsx` (one prop passing that action to `PhotoPanel`), `apps/desktop/src/features/workspace/templateDrop.ts` (doc comment only).

**For the reviewer.** No red baseline was recorded before the production edits (flow.md has no Task 0; stated rather than reconstructed). `cargo test -p psm-desktop` fails once on a fresh command (`include_str!` of `bindings.ts` compiles before `export_bindings` rewrites it) and passes on the second run; the committed bindings are the regenerated ones (tree clean after the closing gate).

