---
plan: T4_template_retake_and_shift
role: implement
round: 1
state: DONE
date: 2026-11-16
actor: claude opus 5.5 (pipeline-implementer)
---
# T4 — Retakes and bad shots · implement r1

## What was built
- **Retake onto a filled container (10G §6.3).** In a templated session, a photo that sits in no container (a Condition photo, an extra Main photo) dropped onto a filled container takes that container. The photo that was there is denied exactly like Deny: red, hidden from Main, its file deleted when the undo window closes, ⌘Z brings it back. It also leaves the shot count (`released`). The new photo gets the container's split, shape and zoom, and it takes over the old photo's place in the shooting order, so no warning appears and later arrivals still go to Condition. It is one undo step, "Retake container <n>". Two photos that both sit in containers still swap (owner card 1, unchanged). The temporary refusal "That container is filled" (`container_filled`) is gone.
- **Deny leaves the container empty (10G §6.4).** A denied photo still counts as a shot, so later photos stay where they are and the container stays empty.
- **Shift the rest back (10G §6.4).** When a denied photo has a later photo that the template placed, the session's template view says so (`can_shift_back`) and the photo panel shows a **Shift the rest back** button, also when nothing differs. Pressing it takes the denied photos out of the count and moves the template-placed photos to their places, with those places' framing. Photos placed by hand stay where they are. It is one undo step, "Shift the rest back". Undo brings the gap back, and a second undo brings the denied photo back. With nothing to shift the button is hidden and the command refuses with `nothing_to_shift`. New command `template_shift_back`, wrapper `templateShiftBack`, bindings regenerated.
- **The owner's story as one test.** `t4_c4a_the_owners_story` runs 10G §8 A1–A9 in order: template editor and restart, a new session, out-of-order arrival with held shutters, Condition after the last container, four Condition photos to Main, a wrong drag, Back to template, undo/redo, deny + Shift the rest back, a retake, and a session type without a template.
- **Agent guide current (flow.md §7A).** `docs/guide/README.md` (authority row and reading-order row for shooting templates), `01_product.md` (workflow row and vocabulary), `02_architecture.md` (the flow "Shooting templates" T1–T4, modules, commands, error codes, migrations 014/015, `<data>/templates/`, frontend files), `04_change_impact.md` (new §Shooting templates with six rows).
- Internal: the re-placing loop of `place_arrival` became `place_template_photos`, shared by `place_arrival` and `shift_back`. No behaviour change: t2 21/21, p66 6/6, full suite green.

## Tests → criterion rows
| Row | Test | Boundary / shape |
|---|---|---|
| T4·C1(a) | `crates/testkit/tests/suite/t4_template_retake_shift.rs::t4_c1a_retake_takes_the_container_and_denies_the_old_photo` | `drop_on_container`, stored photos, denial, row, composition, `session_view`: every outcome of the row, exact |
| T4·C1(b) | `…::t4_c1b_arrival_after_a_retake_goes_to_condition` | arrival: F6 Condition pos 2; F4 slot 3; F1, F2, F5 exact places |
| T4·C1(c) | `…::t4_c1c_retake_is_one_undo_step` | `HistoryService`: label; undo = full S0 snapshot (place, row incl. released/keys, composition, denial) plus the named fields; redo = full snapshot of (a) |
| T4·C1(d) | `…::t4_c1d_retake_of_a_retake_carries_the_key_on` | F4 Deleted + released; F5 Main slot 3, `key_name` `DSC00003.ARW`; differs false; count 0 |
| T4·C2(a) | `…::t4_c2a_deny_leaves_the_container_empty` | `ReviewService::delete`, `session_view`: F1 slot 2, F3 slot 3, no Main in slot 1, `can_shift_back` true |
| T4·C2(b) | `…::t4_c2b_arrival_after_a_deny_does_not_move_later_photos` | arrival: F4 Condition; F1, F3 unchanged |
| T4·C2(c) | `…::t4_c2c_shift_the_rest_back_moves_later_photos_back_one` | `shift_back`: F3 slot 1 normal/default; F4 Main slot 3 split rect (0.1, 0.2, 0.8, 0.6); no Condition; F2 released; `can_shift_back` false |
| T4·C2(d) | `…::t4_c2d_shift_is_one_undo_step_then_the_deny` | label; undo = full snapshot before the shift + the named fields; second undo → F2 Main slot 1 |
| T4·C2(e) | `…::t4_c2e_nothing_later_nothing_to_shift` + `apps/desktop/src-tauri/src/lib.rs::t3_tests::t4_c2g_shift_back_command_answers_no_template` (code string) | `can_shift_back` false; `Err(Error::NothingToShift)`; undo label unchanged; the UI code `nothing_to_shift` pinned through `CommandError::from` |
| T4·C2(f) | `…::t4_c2f_hand_placed_later_photos_are_not_shifted` | `can_shift_back` false |
| T4·C2(g) | `apps/desktop/src-tauri/src/lib.rs::t3_tests::t4_c2g_shift_back_command_answers_no_template` | wired command answers `no_template`; `bindings.ts` contains `template_shift_back` |
| T4·C3(a) | `apps/desktop/src/features/workspace/__tests__/t4_template_shift.test.tsx` "T4·C3(a) …" | PhotoPanel, services mocked: button shown with count 0; `templateShiftBack` called once with 7 |
| T4·C3(b) | same file, "T4·C3(b) …" | no Shift the rest back button when `can_shift_back` false |
| T4·C4(a) | `…::t4_c4a_the_owners_story` | one test, A1–A9 in order, each step asserted before the next |

Pinned tests changed as the plan's table says: `t3_c2d_condition_photo_onto_filled_container_refused` deleted (replaced by T4·C1(a)); T3·C4(d) keeps its id and gained the inputs "a photo without a slot over a filled tile → template drop" and "over a tile without a slot → default".

## Assumptions
1. The retake copies the old photo's **effective** shot key as `shot_keys` reads it (P2: `key_at`, else `captured_at`, else `created_at`; `key_name`, else the camera file key). This is how a second retake carries the original key on (row d).
2. The retake writes the denial with the same store calls as `ReviewService::delete` and does not call that service, so the retake stays one transaction and one undo step. The old photo keeps its slot in its row, as a Deny does.
3. Framing on a retake (P8): container presets are applied when the container has them. A container without presets resets the composition only when the person had not framed the photo, so a person's own framing is never lost.
4. `nothing_to_shift` message: "There is nothing to shift back".
5. The "Shift the rest back" step includes every photo of the session whose stored state (place, row, composition, denial) changed, found by comparing before and after.
6. The button sits on the warning line, right-aligned, styled like Back to template. The toast after it reads "Shifted the rest back · Undo ⌘Z", like Back to template's. `role="status"` now sits on the "n photos differ" text, so a line that shows only the button is not announced as a warning.
7. `dropAction` already treated any dragged photo over a placed tile as a template drop, so U1 task 2 needed no code change. Only the test inputs and the doc comment changed.
8. The tracker and handoff actor is the real one (claude opus 5.5). The prompt said `codex`, but earlier Claude-run T-phases recorded their real actor.

## Deviations
Four files were edited outside the units' listed perimeters because the tasks require them:
- `apps/desktop/src-tauri/src/commands/items.rs`: the exhaustive `Error` → code match (`container_filled` removed, `nothing_to_shift` added). It would not compile otherwise.
- `apps/desktop/src/features/workspace/useWorkspaceState.ts`: U1 removes the `container_filled` toast text, which the plan's table requires. U3 adds `templateShiftBack` through the action runner with an undo toast ("applies the result like any review action").
- `apps/desktop/src/features/workspace/WorkspacePage.tsx`: one prop (`onTemplateShiftBack`).
- `apps/desktop/src/features/workspace/templateDrop.ts`: doc comment only.

No red baseline was recorded before the production edits. flow.md has no Task 0, and none is reconstructed here.

## Files changed
- U1 `2015c99`: `crates/core/src/services/template_placement.rs`, `crates/core/src/error.rs`, `apps/desktop/src-tauri/src/commands/items.rs`, `apps/desktop/src/features/workspace/{templateDrop.ts, useWorkspaceState.ts, __tests__/t3_template_warning.test.tsx}`, `docs/implementation/master_plan_templates.md` (§4), `crates/testkit/tests/suite/{main.rs, t3_template_hand.rs, t4_template_retake_shift.rs (new)}`
- U2 `332237d`: `crates/core/src/services/template_placement.rs`, `crates/core/src/error.rs`, `apps/desktop/src-tauri/src/{commands/templates.rs, commands/items.rs, lib.rs}`, `apps/desktop/src/services/{bindings.ts (generated), templates.ts}`, `crates/testkit/tests/suite/t4_template_retake_shift.rs`
- U3 `6e026d2`: `apps/desktop/src/features/workspace/{PhotoPanel.tsx, TemplateWarning.tsx, workspace.css, useWorkspaceState.ts, WorkspacePage.tsx, __tests__/t4_template_shift.test.tsx (new)}`, `crates/testkit/tests/suite/t4_template_retake_shift.rs`, `docs/guide/{README.md, 01_product.md, 02_architecture.md, 04_change_impact.md}`
- Closing commit: `docs/implementation/plans/T4_template_retake_and_shift.md` (Tracker line, Review log), this handoff.
- Tool-recorded state: none (no architecture graph in this project). Mutation-probe files: none (regime off).

## Checkpoint SHA
U1 `2015c99` · U2 `332237d` · U3 `6e026d2` · closing (tracker, Review log, handoff): the `CHECKPOINT (not approved): T4 closing …` commit that follows `6e026d2`.

## Stamp
Tree `6e026d2` (code), closing gate run once, in order:
| Command | Result |
|---|---|
| `cargo test --workspace` | exit 0; behaviour suite 952 passed, core 96, desktop 30, all others green; 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no warnings |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `npm test` | 86 files, 538 tests passed |

Unit commands run along the way: t4 11/11, t3 19/19, t2 21/21, p66 6/6, `cargo test -p psm-desktop` 30/30 (the first run after adding the command fails once because `include_str!` reads `bindings.ts` before `export_bindings` rewrites it; the second run passes), clippy on core/infra/desktop, `npx vitest run src/features/workspace` 85/85. No pre-existing failures.

## Manual try-out
On scratch data (`PSM_DATA_DIR` pointing at a fresh folder), with a "Dining chair" template of three or four containers, container 3 set to split + zoom:
1. Open a new Dining chair session and shoot the chair, one shot per angle. Press Deny (⌫) on one angle. Its container goes empty and nothing else moves.
2. Shoot that angle again at once. The new photo fills the next container and every later photo is one container too far. A **Shift the rest back** button appears on the line above the containers. Press it: the photos move back one container. ⌘Z brings the gap back, and ⌘Z again brings the denied photo back.
3. Or reshoot the bad angle at the end instead. It arrives under Condition. Drag it onto the bad angle's container. The old photo turns red (deleted; ⌘Z brings it back), the new one sits in the container with that container's split and zoom, and no warning shows.
4. Run the whole story once: template, new chair, shoot out of order, Condition after the last angle, four Condition photos to Main (the warning counts 4), a wrong drag, Back to template.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
