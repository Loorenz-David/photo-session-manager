---
plan: T1_templates_and_settings
role: implement
round: 1
state: DONE
date: 2026-11-16
actor: codex
---
# T1 implementer handoff

## What was built
A photographer can make one shooting template per item type in Settings, add example photos or empty containers, put containers in final and shooting order, set split/shape/zoom presets, and delete a template. Changes save on each action and survive an app restart. The examples appear in black and white while their JPEG copies keep their colour. Settings now has a side list of sections; existing controls retain the shared Save settings path and remain mounted when another section is shown. Template editing is local to this Mac and does not affect session photos yet.

The backend has migration 014, a core template service over the Store port, ten Tauri commands, generated bindings, and a service wrapper. The app data folder and asset scope include `templates/`. The guide records the new schema, flow and scope.

## Tests → criterion rows
| Row | Test |
|---|---|
| T1·C1(a) | `t1_templates::t1_c1a_create_and_lookup` |
| T1·C1(b) | `t1_templates::t1_c1b_duplicate_type` |
| T1·C1(c) | `t1_templates::t1_c1c_delete_template_and_examples` |
| T1·C2(a) | `t1_templates::t1_c2a_add_examples_and_empty_container` |
| T1·C2(b) | `t1_templates::t1_c2b_unreadable_does_not_add` |
| T1·C2(c) | `t1_templates::t1_c2c_final_order_does_not_change_shoot_order` |
| T1·C2(d) | `t1_templates::t1_c2d_rejects_bad_permutations` |
| T1·C2(e) | `t1_templates::t1_c2e_shooting_order_does_not_change_final_order` |
| T1·C2(f) | `t1_templates::t1_c2f_remove_renumbers_both_orders` |
| T1·C2(g) | `t1_templates::t1_c2g_presets_and_invalid_values` |
| T1·C2(h) | `t1_templates::t1_c2h_replace_example` |
| T1·C3(a) | `t1_templates::t1_c3a_restart_keeps_template` |
| T1·C3(b) | `t1_tests::t1_c3b_commands_reach_wired_app_and_bindings` |
| T1·C4(a) | `mechanisms::template::tests::t1_c4a_type_key` |
| T1·C5(a) | `t1_settings_drawer::t1_c5a_lists_sections_in_order_and_opens_capture_one` |
| T1·C5(b) | `t1_settings_drawer::t1_c5b_shows_only_the_selected_pane` |
| T1·C5(c) | `t1_settings_drawer::t1_c5c_saves_values_typed_across_two_sections_together` |
| T1·C5(d) | `t1_settings_drawer::t1_c5d_hides_the_shared_save_footer_for_templates` |
| T1·C6(a) | `t1_templates_editor::t1_c6a_creates_a_template_for_the_typed_type` |
| T1·C6(b) | `t1_templates_editor::t1_c6b_shows_final_order_bw_examples_and_shooting_numbers` |
| T1·C6(c) | `t1_templates_editor::t1_c6c_adds_chosen_photos_in_order_and_shows_last_result` |
| T1·C6(d) | `t1_templates_editor::t1_c6d_saves_split_then_zoom_without_a_save_button` |
| T1·C6(e) | `t1_templates_editor::t1_c6e_taps_shooting_order_and_saves_after_each_tap` |
| T1·C6(f) | `t1_templates_editor::t1_c6f_move_earlier_reorders_the_final_containers` |
| T1·C6(g) | `t1_templates_editor::t1_c6g_failed_write_shows_error_and_reloads_stored_presets` |
| T1·C6(h) | `t1_templates_editor::t1_c6h_confirms_deletion_and_returns_to_create` |

## Assumptions
- Template types use trimmed lowercase keys, while the original trimmed spelling is displayed, as the plan specifies.
- Example images are colour JPEG copies on disk and grayscale only in the view. Unsupported or unreadable files do not add a container.
- The editor fetches template data when the Shooting templates pane is selected. This keeps inactive panes mounted without launching template calls while other settings tests or pages are active.
- The existing template list joins local template types with Shopify suggestions; local editing still works without a shop connection.
- Delete template uses the platform's confirmation dialog. A cancelled dialog leaves it unchanged.
- File cleanup follows the database transaction. An OS file removal error after commit can leave an orphaned image copy or folder; the editor reloads the stored state on error.

## Deviations
- Unit 2 added the required `TemplatesSection.tsx` placeholder, though its path was listed as new in Unit 3. Unit 3 replaced it and touched `SettingsPage.tsx` to pass the active section state, keeping its pane mounted while deferring its service calls.
- Updated `crates/testkit/tests/suite/p01_storage_identity.rs`, beyond the named T1 test file, because its exact table list rejected the two new migration tables. The targeted 20-test module passes.
- Updated `docs/guide/02_architecture.md` and `docs/guide/04_change_impact.md` under flow §7A, which asks for guide changes when schema, commands, data paths or couplings change.
- No plan/intention semantic deviation; no live service or hardware used.

## Files changed
- Backend: `crates/infra/src/sqlite/migrations/014_shooting_templates.sql`, `crates/infra/src/sqlite/store.rs`, `crates/infra/src/paths.rs`, `crates/core/src/domain/template.rs`, `crates/core/src/domain/mod.rs`, `crates/core/src/mechanisms/template.rs`, `crates/core/src/mechanisms/mod.rs`, `crates/core/src/services/templates.rs`, `crates/core/src/services/mod.rs`, `crates/core/src/ports/store.rs`, `crates/core/src/error.rs`.
- Desktop edge and bindings: `apps/desktop/src-tauri/src/commands/templates.rs`, `apps/desktop/src-tauri/src/commands/mod.rs`, `apps/desktop/src-tauri/src/commands/items.rs`, `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/services/bindings.ts`, `apps/desktop/src/services/templates.ts`.
- UI: `apps/desktop/src/features/settings/SettingsNav.tsx`, `apps/desktop/src/features/settings/SettingsPage.tsx`, `apps/desktop/src/features/settings/TemplatesSection.tsx`, `apps/desktop/src/features/settings/TemplateExample.tsx`, `apps/desktop/src/styles/globals.css`.
- Tests: `crates/testkit/tests/suite/t1_templates.rs`, `crates/testkit/tests/suite/main.rs`, `crates/testkit/tests/suite/p01_storage_identity.rs`, `apps/desktop/src/features/settings/__tests__/t1_settings_drawer.test.tsx`, `apps/desktop/src/features/settings/__tests__/t1_templates_editor.test.tsx`.
- Documentation: `docs/guide/02_architecture.md`, `docs/guide/04_change_impact.md`, `docs/implementation/plans/T1_templates_and_settings.md`, this handoff.

## Checkpoint SHA
1. Unit 1: `c6a63a5` — storage, service, commands, bindings, 14 row tests.
2. Unit 2: `45d4867` — Settings section drawer and 4 row tests.
3. Unit 3: `5dda588` — template editor and 8 row tests.
The closing checkpoint contains the targeted correction, guide update, tracker, Review log and this handoff.

## Stamp
| Command | Result |
|---|---|
| `cargo test -p psm-core mechanisms::template` | 1/1 passed (Unit 1) |
| `cargo test -p psm-testkit --test suite t1_templates::` | 12/12 passed (Unit 1 and after the row assertion strengthening) |
| `cargo test -p psm-desktop` | 27/27 passed; generated bindings committed. The first run compiled before `export_bindings` wrote the file; the targeted wired test passed on rerun, then the full desktop crate passed. |
| `cargo clippy -p psm-core -p psm-infra -p psm-desktop --all-targets -- -D warnings` | passed after formatting/code fixes (Unit 1) |
| `npx vitest run src/features/settings src/features/shell src/features/pending` | 113/113 passed (Unit 2) |
| `npx vitest run src/features/settings` | 80/80 passed (Unit 3) |
| `cargo test --workspace` | one run: `psm-testkit` suite 900 passed, 1 failed (`p01_storage_identity::c1a` exact table set before migration 014); earlier workspace programs passed. The stale expectation was updated. |
| `cargo test -p psm-testkit --test suite p01_storage_identity::` | targeted rerun 20/20 passed after correction; full suite intentionally not repeated per flow §4A. |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed |
| `cargo fmt --all -- --check` | passed after applying formatter to the targeted test changes |
| `cd apps/desktop && npm run typecheck` | passed |
| `cd apps/desktop && npm test` | 83 files, 527/527 passed |

## Manual try-out
1. Start the app with scratch data: from `apps/desktop`, run `PSM_DATA_DIR=$HOME/psm-templates-dev npm run tauri -- dev`.
2. Open Settings and click through the side sections. The existing settings are there; edit two of the first four sections and press Save settings once.
3. Select Shooting templates, type `Dining chair`, and create it. Add several image files. Their examples appear in black and white.
4. Drag cards into final order, or use Move earlier/later. Set Split, Shape and Zoom on a card. Select Set shooting order and tap the cards in shutter order, then Done.
5. Quit and reopen the app with the same scratch data folder. The template, examples, orders and presets are still there.
6. Try to add a non-image file renamed `.jpg`; the editor shows the unreadable-image message and leaves the saved template intact.

## ⚠ OWNER DECISIONS REQUIRED (0)
none
