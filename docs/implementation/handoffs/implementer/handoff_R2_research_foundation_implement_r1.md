---
plan: R2_research_foundation
role: implement
round: 1
state: DONE
date: 2026-11-12
actor: codex
---

# R2 research foundation — implement r1

## What was built

The shared research foundation is ready for the engine and app tracks. Name comparison and value sameness are pure core functions. Existing values and publish items now carry the frozen research fields; SQLite adds them idempotently after numbered migrations. The app has an unwired research API and eight commands that report `research_unavailable` until the engine is connected. A reduced research JPEG can be made from an adjusted preview using the existing composition pixel rule, with metadata removed. No research run or publish hold is produced yet.

## Tests → criterion rows

Every row has one input/outcome test at its named boundary. Core names below are inline unit tests; `research::r2_foundation` is the testkit suite module; `r2_research_command_tests` invokes the desktop command handler.

| Row | Test |
|---|---|
| A03·C1(a) | `mechanisms::name_key::tests::a03_c1a_punctuation` |
| A03·C1(b) | `mechanisms::name_key::tests::a03_c1b_diacritics_and_spaces` |
| A03·C1(c) | `mechanisms::name_key::tests::a03_c1c_nfkc` |
| A03·C1(d) | `mechanisms::name_key::tests::a03_c1d_full_fold` |
| A03·C1(e) | `mechanisms::name_key::tests::a03_c1e_symbols` |
| A03·C1(f) | `mechanisms::name_key::tests::a03_c1f_no_aliases` |
| A03·C1(g) | `mechanisms::name_key::tests::a03_c1g_empty` |
| A03·C7(a) | `mechanisms::sameness::tests::a03_c7a_text` |
| A03·C7(b) | `mechanisms::sameness::tests::a03_c7b_choice` |
| A03·C7(c) | `mechanisms::sameness::tests::a03_c7c_multi_choice` |
| A03·C7(d) | `mechanisms::sameness::tests::a03_c7d_dimension` |
| A03·C7(e) | `mechanisms::sameness::tests::a03_c7e_boolean_integer` |
| A07·C7(a) | `research::r2_foundation::a07_c7a_schema_step_preserves_rows_and_is_idempotent` |
| A09·C5(a) | `research::r2_foundation::a09_c5a_held_status_and_unresolved_label` |
| A09·C5(b) | `research::r2_foundation::a09_c5b_held_session_is_publishing_in_list` |
| F·C1(a) | `r2_research_command_tests::f_c1a_unwired_commands_return_research_unavailable` |
| F·C2(a) | `research::r2_foundation::f_c2a_research_transaction_commits_and_rolls_back_both_views` |
| F·C3(a) | `research::r2_foundation::f_c3a_public_listing_default_and_fake_values` |
| A10·C1(a) | `research::r2_foundation::a10_c1a_full_frame_reduced_with_corners` |
| A10·C1(b) | `research::r2_foundation::a10_c1b_turn_and_cut_right_half_of_source` |
| A10·C1(c) | `research::r2_foundation::a10_c1c_small_preview_never_upscaled` |
| A10·C1(d) | `research::r2_foundation::a10_c1d_large_preview_resized_to_2048` |
| A10·C2(a) | `research::r2_foundation::a10_c2a_exif_and_gps_removed` |

## Assumptions

- U1 pins `unicode-casefold 0.2.0` and `unicode-general-category 1.1.0`. Decimal measurements use checked `i128` products; an overflow is unequal.
- U2 stores the three publish JSON maps as `BTreeMap<String, Json>` and starts them as `{}`. `StatusInput.unresolved_conflicts` is the count of pending keys without entries in `resolved_attributes`. The API's stage, state and trigger display fields are strings until the engine supplies its own enums. The `research` settings slot stays absent when unset, preserving existing settings output.
- U3 calls the established composition output renderer with the adjusted preview, JPEG quality 85 and the requested long edge, then strips APP0–APP15 and COM segments without a second encode. Its temporary render file is removed before return; only the requested destination remains. Under MC-27 a quarter turn maps the source's right half to the turned frame's bottom half.

## Deviations

- Existing schema/value test fixtures were updated for `ai_prior` and `research_feedback`; the desktop crate gained a test-only `rusqlite` dependency for the command no-write check. These paths are named by U2's fixture allowance.
- `npm run typecheck` initially found no `tsc` because this worktree had no `node_modules`; `npm ci` restored the local dependencies and the closing typecheck passed. No package manifest or lockfile was changed by npm.
- No numbered migration was changed. No hardware or live provider was used.

## Files changed

- `Cargo.lock`
- `Cargo.toml`
- `apps/desktop/src-tauri/Cargo.toml`
- `apps/desktop/src-tauri/src/commands/attributes.rs`
- `apps/desktop/src-tauri/src/commands/mod.rs`
- `apps/desktop/src-tauri/src/commands/research.rs`
- `apps/desktop/src-tauri/src/commands/sessions.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src/features/form/__tests__/p11_form_ui.test.tsx`
- `apps/desktop/src/features/form/__tests__/p22_form_tidy.test.tsx`
- `apps/desktop/src/features/form/__tests__/p67_form_values_ui.test.tsx`
- `apps/desktop/src/services/bindings.ts`
- `apps/desktop/src/services/events.ts`
- `apps/desktop/src/services/research.ts`
- `crates/core/Cargo.toml`
- `crates/core/src/domain/attribute.rs`
- `crates/core/src/domain/mod.rs`
- `crates/core/src/domain/publish.rs`
- `crates/core/src/domain/research.rs`
- `crates/core/src/domain/session.rs`
- `crates/core/src/events.rs`
- `crates/core/src/mechanisms/mod.rs`
- `crates/core/src/mechanisms/name_key.rs`
- `crates/core/src/mechanisms/sameness.rs`
- `crates/core/src/mechanisms/status.rs`
- `crates/core/src/ports/commerce.rs`
- `crates/core/src/ports/images.rs`
- `crates/core/src/ports/store.rs`
- `crates/core/src/services/attributes.rs`
- `crates/core/src/services/publish.rs`
- `crates/core/src/services/sessions.rs`
- `crates/infra/src/imaging/mod.rs`
- `crates/infra/src/imaging/research.rs`
- `crates/infra/src/settings_file.rs`
- `crates/infra/src/sqlite/ensure.rs`
- `crates/infra/src/sqlite/mod.rs`
- `crates/infra/src/sqlite/store.rs`
- `crates/integrations/Cargo.toml`
- `crates/integrations/src/lib.rs`
- `crates/research/Cargo.toml`
- `crates/research/src/api.rs`
- `crates/research/src/lib.rs`
- `crates/testkit/src/fake_commerce.rs`
- `crates/testkit/tests/suite/main.rs`
- `crates/testkit/tests/suite/p01_storage_identity.rs`
- `crates/testkit/tests/suite/p03_session_state_list.rs`
- `crates/testkit/tests/suite/p14_publish_update.rs`
- `crates/testkit/tests/suite/research/mod.rs`
- `crates/testkit/tests/suite/research/r2_foundation.rs`
- `docs/implementation/master_plan_research.md`
- `docs/implementation/plans/R2_research_foundation.md`
- `docs/implementation/handoffs/implementer/handoff_R2_research_foundation_implement_r1.md`

## Checkpoint SHA

1. U1 `290fcf3` — name key and sameness.
2. U2 `879b2f0` — storage seam and frozen shapes.
3. U3 `184e61c` — research copy renderer.
4. Closing artifacts: checkpoint commit containing this handoff and the R2 tracker/Review log (SHA is the commit that contains this file).

## Stamp

The full gate ran once on clean code checkpoint `184e61c`, before the tracker and handoff were written.

| Command | Result |
|---|---|
| `cargo test --workspace` | PASS: 822 testkit suite tests and all crate/desktop unit tests; no failures |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cd apps/desktop && npm run typecheck` | PASS |
| `cd apps/desktop && npm test` | PASS: 70 files, 466 tests |

Per-unit commands passed: U1 core name-key and sameness tests, clippy and fmt; U2 `research::r2_foundation`, P01, P03, core status, desktop bindings, desktop command test, typecheck, workspace check and the three touched UI test files (43 tests); U3 `research::r2_foundation`, P28 and P29. U2's first P01 run found its expected schema fixture missing the new table; it was updated and the targeted P01 rerun passed. U3's first test compile found a missing `GenericImageView` import in the new test; it was fixed and the targeted module passed.

## Manual try-out

1. On scratch data, run `cd apps/desktop && PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev`.
2. Open an item form and a publish review. Existing, default and “from Shopify” values should display as before; publishing an ordinary item should still work.
3. There is no new research panel behavior yet. The commands return `research_unavailable` until the engine is wired in R7.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
