---
plan: P24_capture_one_provisioning
role: implement
round: 1
state: DONE
date: 2026-11-03
actor: codex
---
# P24 implementer handoff

## What was built
Session Manager now uses its own Capture One session by default, creates it if missing, and opens a selected session alongside the photographer's other open documents. It launches Capture One when needed, creates or repairs the two disabled processing recipes, applies a listed next-capture style, installs hooks, and verifies its document is in front before routing captures. Settings has a `.cosessiondb` picker and a return-to-default action; Health names provisioning failures. Existing configured document folders remain supported.

## Tests → criterion rows
| Row | Boundary test | Result |
|---|---|---|
| C1(a) | `c1a_fresh_default_document_is_created_alongside_and_recipes_are_idempotent` | Pass |
| C1(b) | `c1b_drifted_recipe_repairs_only_dependent_fields` | Pass |
| C1(c) | `c1c_launch_installed_missing_and_bounded_timeout` | Pass |
| C1(d) | `c1d_automation_and_edition_fail_before_document` | Pass |
| C1(e) | `c1e_non_session_folder_and_closed_document` | Pass |
| C1(f) | `c1f_style_setting_applies_reports_missing_and_preserves_when_empty` | Pass |
| C2(a) | `c2a_activation_fronts_own_document_before_capture_folder` | Pass |
| C2(b) | `c2b_unverified_front_blocks_capture_then_retry_recovers` | Pass |
| C2(c) | `c2c_health_reasserts_front_only_while_active` | Pass |
| C3(a) | `c3a_document_setting_defaults_clears_validates_and_honours_existing` | Pass |
| C4(a) | `c4a_recipe_script_creates_repairs_only_drift_and_skips_exact` | Pass; task 0's fresh-recipe values govern changed fields |
| C4(b) | `c4b_document_front_and_style_scripts_use_grounded_dictionary_forms` | Pass; task 0's make form omits `kind` |
| C5(a) | UI `C5(a) Settings picks a session file…` | Pass |
| C5(b) | UI `C5(b) Health shows each provisioning failure…` | Pass |
| C6(a) | Optional live Capture One check | Deferred to reviewer/owner; suite is hardware-free |

Core mechanism test: `mc26_classifies_editor_failures` passes on installed-app and Automation-denied strings. Inherited p04, p16 and p17 tests were updated for provisioning and passed.

## Assumptions
- `AppPaths.home` reads HOME; `TestApp` overrides it with the temporary directory.
- A folder-named `.cosessiondb` wins when present; otherwise the first session file in name order is used.
- Opening a closed session first makes a uniquely parented `PSM Open Window` throwaway document under the app data directory, opens the target into that window, and removes the throwaway folder. This follows VERIFY-14.
- Empty style leaves Capture One's next-capture style untouched, including a style the photographer previously selected.

## Deviations
- C4(b) says `make_document` must include `withProperties.kind == 'session'`; task 0 and VERIFY-13 show that property throws. The script and test omit it. No book change is needed; this criterion cell should be corrected before review.
- C4(a) says all nine dependent fields appear in `changed` on creation. VERIFY-14 says a fresh recipe already has `enabled=false`, so the script writes and reports only the eight fields that differ. The test asserts that observed outcome. This criterion cell should be corrected before review.
- The inherited P17 test for a missing Capture root now expects the P24 repair to recreate it. P04 recipe and P16 closed-document fixtures were likewise updated for the new behavior.
- The optional C6(a) live run was not attempted in this Codex session. Task 0 was already run by the orchestrator and recorded in the evidence file; no code in this session sent Apple events or touched an owner document.

## Files changed
Checkpoint contains these paths:
- `apps/desktop/src-tauri/src/commands/items.rs`
- `apps/desktop/src-tauri/src/commands/sessions.rs`
- `apps/desktop/src-tauri/src/commands/settings.rs`
- `apps/desktop/src/features/health/HealthPage.tsx`
- `apps/desktop/src/features/pending/__tests__/p18_pending_state.test.tsx`
- `apps/desktop/src/features/settings/SettingsPage.tsx`
- `apps/desktop/src/features/settings/__tests__/p16_settings.test.tsx`
- `apps/desktop/src/features/settings/__tests__/p23_shape_on_arrival.test.tsx`
- `apps/desktop/src/features/settings/__tests__/p24_capture_one_provisioning.test.tsx`
- `apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx`
- `apps/desktop/src/features/shell/__tests__/p22_shell_tidy.test.tsx`
- `apps/desktop/src/services/bindings.ts`
- `apps/desktop/src/services/settings.ts`
- `crates/core/src/domain/editing.rs`
- `crates/core/src/error.rs`
- `crates/core/src/ports/editing.rs`
- `crates/core/src/services/camera.rs`
- `crates/core/src/services/editing_startup.rs`
- `crates/core/src/services/sessions.rs`
- `crates/core/src/services/settings.rs`
- `crates/infra/src/paths.rs`
- `crates/infra/src/settings_file.rs`
- `crates/integrations/src/editing/capture_one/adapter.rs`
- `crates/integrations/src/editing/capture_one/gate.rs`
- `crates/integrations/src/editing/capture_one/jxa.rs`
- `crates/integrations/src/editing/capture_one/jxa/editor_info.js`
- `crates/integrations/src/editing/capture_one/jxa/ensure_capture_style.js`
- `crates/integrations/src/editing/capture_one/jxa/ensure_recipe.js`
- `crates/integrations/src/editing/capture_one/jxa/front_document.js`
- `crates/integrations/src/editing/capture_one/jxa/make_document.js`
- `crates/integrations/src/editing/capture_one/jxa/open_document.js`
- `crates/integrations/src/editing/capture_one/jxa/set_recipe_props.js`
- `crates/integrations/src/editing/capture_one/mod.rs`
- `crates/integrations/src/editing/capture_one/provision.rs`
- `crates/integrations/src/editing/capture_one/recipes.rs`
- `crates/integrations/src/editing/capture_one/scripting.rs`
- `crates/integrations/src/lib.rs`
- `crates/testkit/src/fake_capture_one.rs`
- `crates/testkit/src/test_app.rs`
- `crates/testkit/tests/p04_capture_one_adapter.rs`
- `crates/testkit/tests/p16_operations.rs`
- `crates/testkit/tests/p17_real_run_fixes.rs`
- `crates/testkit/tests/p24_capture_one_provisioning.rs`
- `docs/implementation/master_plan.md`
- `docs/implementation/plans/P24_capture_one_provisioning.md`
- `docs/implementation/handoffs/implementer/handoff_P24_capture_one_provisioning_implement_r1.md` (this handoff, written after the checkpoint for the runner to commit)

## Checkpoint SHA
`1a4d277a170904bcdcec49a693f30528aeda0903` — `CHECKPOINT (not approved): implement P24 Capture One provisioning`. The checkpoint contains only the 45 paths above; the runner owns the handoff commit.

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace` | Pass, all Rust crates and integration tests including 12 P24 tests |
| `npm test` in `apps/desktop` | Pass, 129 UI tests including 2 P24 tests |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass after fixing two lint findings in this implementation |
| `cargo fmt --all -- --check` | Pass |
| `npm run typecheck` in `apps/desktop` | Pass |
| `git diff --cached --check` | Pass before checkpoint |

## Manual try-out
1. Quit Capture One. Open Session Manager → Settings. Choose **Use the app's own session** and Save if an older session is selected. Capture One should launch and show a new Session Manager session. Check that `PSM Preview` and `PSM Web` exist and are unticked; Health should show document, recipes and hooks green. If `PSM House` is absent, Health shows a Style error; clear the Style field and Save to clear it.
2. Bring another Capture One session to the front, then activate a Session Manager session. Its document should come to the front while the other remains open.
3. With a tethered camera, capture a frame. It should land in `~/Pictures/Session Manager/Capture/<item folder>` and appear in Session Manager.
4. Enable `PSM Web` and set its quality to 70 in Capture One. Press Health → Retry checks; it should become disabled and return to quality 90.
5. In Settings, use **Use another session…** to select an existing `.cosessiondb`; Save and check its folder appears. Use **Use the app's own session** and Save to return.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
