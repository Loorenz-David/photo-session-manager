---
plan: T2_template_placement
role: implement
round: 1
state: DONE
date: 2026-11-16
actor: codex (U1, U2, U3 partial) + claude opus 5.5 (U3 finish, closing gate, handoff)
---
# T2 — Photos file themselves · implement r1

## What was built
- **Photos file themselves.** In a session whose item type has a shooting template, each arriving photo goes to the container of its place in the shooting order, ordered by when it was shot (the camera's file number decides among files shot within five minutes of each other), not by when it arrived. A shutter the camera reported whose file has not arrived holds its container open for up to five minutes. Photos after the last container arrive as Condition.
- **Presets.** A container's split, shape and zoom are applied to the photo the template puts there; when the frame is not known yet, the zoom waits and is applied with the first frame.
- **Hand changes are kept.** Classify, reorder, delete and a Compose edit on a photo of a templated session mark it as placed by hand; later arrivals never move it or change its framing. A moved photo loses its place record. Arrival placement is not an undo step.
- **Session view.** The Main area shows the containers in final order: an empty one as a faded tile with its number, a filled one as the photo's tile today; then Main photos without a container and unreviewed ones. A small "Template" button in the Main heading opens the template: each container's number, black-and-white example, "Shot n" and presets in words. The view refreshes on every photo/session change of that session. A session whose type has no template (or a template with no container) looks and behaves exactly as before.
- Units: U1 `efcc86c` and U2 `0bff768` by Codex (the session was cut off by its usage limit after a partial U3, `fa8a484`); this session re-ran U1/U2's unit commands (green, nothing redone), finished U3 (`018dc27`), ran the closing gate and wrote this handoff.

## Tests → criterion rows
| Row | Test | Shape |
|---|---|---|
| T2·C1(a)–(j) | `crates/core/src/mechanisms/shot_order.rs` tests `t2_c1a_time_and_id`, `t2_c1b_counter_corrects_time`, `t2_c1c_gap_reserves_hold`, `t2_c1d_no_gap_ends_with_hold`, `t2_c1e_first_shot_late`, `t2_c1f_multiple_gaps`, `t2_c1g_counter_restart`, `t2_c1h_outside_window`, `t2_c1i_no_counter_keeps_its_slot`, `t2_c1j_prefix_starts_new_run` | exact entry order |
| T2·C2(a) | `mechanisms::template::tests::t2_c2a_shoot_to_final` | exact |
| T2·C2(b) | `…::t2_c2b_zoom_centred` | exact (1e-9) |
| T2·C2(c) | `…::t2_c2c_preset_values` | exact + `validate` |
| T2·C3(a)–(f) | `t2_template_placement::t2_c3a_shooting_order_files_final_order`, `t2_c3b_extras_become_condition`, `t2_c3c_other_type_untouched`, `t2_c3d_empty_template_untouched`, `t2_c3e_type_key_matches`, `t2_c3f_arrival_has_no_history` | stored photos / rows / history |
| T2·C4(a)–(e) | `t2_c4a_exposure_hold_keeps_gap`, `t2_c4b_late_file_replaces_slot_and_emits`, `t2_c4c_expired_hold_closes_gap`, `t2_c4d_existing_photo_counts_but_stays`, `t2_c4e_tether_uses_file_time` | stored photos, events |
| T2·C5(a)–(c) | `t2_c5a_preset_materialises_on_first_frame`, `t2_c5b_late_preset_refits_known_frame`, `t2_c5c_preset_removed_when_photo_becomes_condition` | compositions |
| T2·C6(a)–(b) | `t2_c6a_view_has_containers_and_photos`, `t2_c6b_no_template_view` | `session_view`, wired command + bindings |
| T2·C7(a)–(d) | `t2_c7a_manual_classify_stays`, `t2_c7b_manual_condition_stays`, `t2_c7c_move_loses_place_row`, `t2_c7d_compose_edit_survives_late_arrival` | stored photos, rows, compositions |
| T2·C8(a) | `features/workspace/__tests__/t2_template_containers.test.tsx` `t2_c8a_empty_containers_are_faded_numbered_placeholders` | cell order E1, P1, E3; `data-empty`; text; heading count 1 |
| T2·C8(b) | `t2_c8b_filled_containers_then_photos_without_a_container` | five tiles in exact order, no placeholder |
| T2·C8(c) | `t2_c8c_untemplated_session_looks_as_today` | three tiles, no placeholder, no button |
| T2·C8(d) | `t2_c8d_info_button_opens_the_template` | entries 1, 2, 3; entry 3 `[data-bw] img`, "Shot 3", "Split · Zoom 20 %" |
| T2·C8(e) | `t2_c8e_view_reloads_on_events_of_its_session_only` | `getTemplateView` calls exactly `[[7],[7]]` |
U1/U2 tests are Codex's; this session ran them, did not rewrite them. No test outside the rows was added (the p01 edit below updates an existing tripwire list).

## Assumptions
- Placement reads and writes inside one store transaction (a delegated choice).
- Placeholder: a faded, dashed tile with its number; the info button: a small outlined "Template" with an info icon at the right of the Main heading; the dialog lists number, black-and-white example, "Shot n", presets in words. The folded strip is unchanged.
- A Main photo whose slot names a container that no longer exists (template edited mid-session) is shown after the containers, with the slotless Main photos.

## Deviations
1. **View DTO typing (U2 file, typing only).** `TemplatePhotoViewDto.slot` and `TemplateViewDto.template_id` were generated as non-null `number` though Rust sends `null`; now `Option<specta_typescript::Number>` like every other optional id, bindings regenerated (`cargo test -p psm-desktop`). No runtime change.
2. **Schema tripwire outside the file list.** `crates/testkit/tests/suite/p01_storage_identity.rs` C1(a) lists every table and failed in the closing suite on the new `photo_places`; the table and its eight columns were added to its list (T1 did the same for its tables). Unit 2's commands did not include p01, so this showed only at the close.

## Files changed
This session (cycle-scoped):
- `apps/desktop/src/features/workspace/ContainerGrid.tsx` (rewritten from the partial)
- `apps/desktop/src/features/workspace/PhotoPanel.tsx` (info icon, import order)
- `apps/desktop/src/features/workspace/workspace.css` (button layout)
- `apps/desktop/src/features/workspace/__tests__/t2_template_containers.test.tsx` (new)
- `apps/desktop/src-tauri/src/commands/templates.rs`, `apps/desktop/src/services/bindings.ts` (generated) — deviation 1
- `crates/testkit/tests/suite/p01_storage_identity.rs` — deviation 2
- `docs/implementation/plans/T2_template_placement.md` (tracker line, Review log)
- this handoff

Whole phase (U1, U2 by Codex; U3 partial kept as is): `crates/core/src/mechanisms/{shot_order.rs, template.rs, mod.rs}`, `crates/core/src/{domain/template.rs, ports/store.rs, services/{arrival.rs, composition.rs, mod.rs, review.rs, template_placement.rs}}`, `crates/infra/src/sqlite/{store.rs, migrations/015_photo_places.sql}`, `crates/testkit/tests/suite/{main.rs, t2_template_placement.rs}`, `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/services/templates.ts`, `apps/desktop/src/features/workspace/{TemplateInfoDialog.tsx, useTemplateView.ts, WorkspacePage.tsx}`.
Mutation-probe files: none (regime off). Not mine and not committed: `docs/implementation/STATUS_templates.md` (modified in the worktree before this session).

## Checkpoint SHA
U1 `efcc86c` · U2 `0bff768` · U3 partial `fa8a484` · U3 `018dc27` · closing (p01, plan, handoff): the commit after `018dc27` titled "CHECKPOINT (not approved): T2 closing evidence and handoff".

## Stamp
- Resume check: `cargo test -p psm-core mechanisms::shot_order` 10/10 · `mechanisms::template` 4/4 · clippy psm-core · fmt · `--test suite t2_template_placement::` 20/20 · `p66…` 6/6 · `p28…` 20/20 · `p58…` 6/6 · `p44…` 25/25 · `cargo test -p psm-desktop` 28/28 · clippy core/infra/desktop — all green.
- U3: `npx vitest run src/features/workspace` 14 files / 79 tests + the new file 5/5 · `npm run typecheck` green.
- Closing gate (tree `018dc27` + p01 edit): `cargo test --workspace` 3 min 30 s → suite 920/921, the one failure `p01_storage_identity::c1a_open_creates_all_domain_columns` (deviation 2), fixed and re-run alone: `p01_storage_identity::` 20/20; every other binary green (core 96, testkit 233, 163, 37, 30, desktop 28, 10). `cargo clippy --workspace --all-targets -- -D warnings` green · `cargo fmt --all -- --check` green · `npm run typecheck` green · `npm test` 84 files / 532 tests green. Pre-existing failures: none seen.

## Manual try-out
On scratch data, with a template made in Settings → Shooting templates (say "Dining chair", five containers, the third with Split and Zoom 20):
1. Make a session for an item of type Dining chair and open it: the Main area shows five faded, numbered tiles and a small "Template" button. Press it: the black-and-white examples with their numbers, "Shot n" and presets.
2. Shoot the angles in your shooting order. Each photo lands in its own container, whatever order the files arrive in; the split angle's photo is split and zoomed.
3. Keep shooting after the last angle: those photos arrive under Condition.
4. Move a Condition photo to Main: it stays there when the next photo arrives.
5. Open a session whose type has no template: everything is as before.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
