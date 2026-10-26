---
plan: P13_publish_runs
role: implement
round: 1
state: DONE
date: 2026-10-26
actor: codex
---
# P13 publish runs — implement r1 handoff

The gate passed: the intention is RATIFIED, P09 and P12 are APPROVED, and P13 was not APPROVED when work began.

## What was built

- A photographer can confirm a publish preview that freezes the chosen Main and Condition photos, valid item details, change sequence, and announced shop removals. Every shop mutation in the run receives the persisted confirmation token.
- Publishing creates or adopts the shop product, verifies the RAW files, renders a staged local export, uploads each eligible photo, mirrors the shop gallery in Main order, then records publication time and a notification. Failed steps can be retried; active work resumes at app startup. Upload recovery adopts shop media by the exact photo filename pattern to avoid duplicates.
- Shopify settings now construct the commerce adapter. Startup refreshes shop definitions and their links atomically. New-session search includes unlinked shop products; choosing one adopts it locally. SKU creation, identity edits, and next-number suggestions check reachable shop products.
- Tauri commands expose preview, confirm, retry and notifications, with progress events and generated TypeScript bindings. The New-session dialog labels shop hits “in the shop”. P15 supplies the Publish popover; it is outside this phase.
- Read-only Shopify title-search and file-list probes are recorded in `docs/intention/evidence/shopify_admin.md`. No live shop mutation was made in this phase.

## Tests → criterion rows

Each row has one boundary test. Rust tests are in `crates/testkit/tests/p13_publish_runs.rs`; C9(f) is in `apps/desktop/src/features/sessions/__tests__/p13_publish_runs.test.tsx`.

| Row | Boundary test and exact outcome checked |
|---|---|
| C1(a) | `c1a_confirm_freezes_lists_and_excludes_unreviewed`: one frozen run/item, with only eligible Main and Condition photos |
| C1(b) | `c1b_confirm_freezes_only_canonical_valid_values`: only canonical valid details frozen |
| C1(c) | `c1c_preview_removals_and_exact_confirmation_text`: foreign media announced; summary stored byte for byte |
| C1(d) | `c1d_confirm_stores_current_change_sequence_and_time`: confirmed sequence and clock time persisted |
| C2(a) | `c2a_every_remote_mutation_carries_a_previously_confirmed_run`: mutations carry an already confirmed run |
| C3(a) | `c3a_prepare_creates_product_then_persists_link_before_next_remote_effect`: created product linked before the next remote effect |
| C3(b) | `c3b_prepare_adopts_one_exact_sku_match_without_create`: one exact SKU match adopted without create |
| C3(c) | `c3c_prepare_fails_when_two_products_carry_sku`: ambiguous SKU fails prepare |
| C3(d) | `c3d_changed_raw_blocks_prepare_before_upload`: changed RAW fails before upload |
| C3(e) | `c3e_prepare_exports_to_staging_and_records_exact_file_hash`: staged exports and exact SHA-256 per link |
| C4(a) | `c4a_resume_after_staging_effect_has_one_media_per_photo`: restart after staging effect finishes with one media per photo |
| C4(b) | `c4b_resume_adopts_media_created_before_local_record`: remote media adopted without another create |
| C4(c) | `c4c_resume_polls_attached_media_without_creating_again`: attached media polled to ready without duplicate |
| C4(d) | `c4d_ready_link_skips_staging_on_next_publish`: ready link skips staging |
| C4(e) | `c4e_moved_photo_uploads_for_new_item_without_changing_old_link`: link ownership stays scoped to item |
| C5(a) | `c5a_foreign_media_blocks_reconciliation`: foreign media blocks deletion |
| C5(b) | `c5b_reconciliation_deletes_owned_and_announced_and_moves_all`: exact delete and move plan |
| C5(c) | `c5c_update_mirrors_main_order_and_keeps_condition_out_of_gallery`: exact Main gallery order and cover |
| C5(d) | `c5d_wrong_order_readback_fails_at_update`: wrong read-back fails update |
| C5(e) | `c5e_update_refuses_unready_desired_media_before_deletion`: incomplete upload blocks deletion |
| C5(f) | `c5f_not_found_delete_reply_counts_as_removed`: missing media deletion is accepted |
| C6(a) | `c6a_deleted_after_confirmation_is_not_uploaded_or_in_gallery`: deleted photo skipped and notification still recorded |
| C6(b) | `c6b_moved_after_confirmation_is_skipped`: moved photo skipped |
| C7(a) | `c7a_done_records_publish_and_notification`: publish timestamp, sequence, status and notification |
| C7(b) | `c7b_photo_change_during_run_remains_dirty_after_done`: later change remains dirty |
| C7(c) | `c7c_failed_upload_updates_activity_and_status`: failure activity and status |
| C8(a) | `c8a_retry_upload_does_not_rerun_prepare`: retry resumes upload only |
| C8(b) | `c8b_restart_requeues_running_upload_job_and_completes`: running job requeued and completes once |
| C9(a) | `c9a_settings_build_shopify_adapter_with_default_condition_key_or_none`: configured adapter and absent integration |
| C9(b) | `c9b_schema_refresh_replaces_definitions_and_links_atomically`: refresh replacement and rollback on failure |
| C9(c) | `c9c_search_merges_local_and_unlinked_shop_hits_and_survives_outage`: merged hits and local-only outage result |
| C9(d) | `c9d_adopt_shop_hit_creates_linked_item_then_session`: adoption and session open |
| C9(e) | `c9e_shop_sku_collision_blocks_create_and_foreign_edit_but_allows_own_or_outage`: SKU collision policy |
| C9(f) | `NewSessionDialog` UI test: shop label, adoption call, then session open |
| C9(g) | `c9g_next_sku_skips_remote_candidates_then_falls_back_on_outage`: remote candidate skip and outage fallback |

## Assumptions

- One queued `publish_step` job advances one item through one step. The next step gets a new queued job.
- P09's staging folder uses its export job id. The staged files are swapped to the local item folder at publish completion.
- Notification text uses the clock's UTC millisecond value; progress is emitted on each state change without throttling.
- Title search uses Shopify prefix terms with local substring filtering. File recovery pages the file list because the probed filename search index lagged.
- An unavailable shop leaves local search and the local next-SKU proposal usable, as the plan specifies.

## Deviations

- P01's exact schema check rejected an added `publish_items` column. The export job id instead lives in the existing app-state `settings` table until publish completion; no migration changed.
- Publish job registration uses the existing `JobRunner` in `App`; `infra/jobs.rs` needed no edit. The existing sessions query already reads publish-item status, so `sessions.rs` needed no edit.
- Required support edits touched DTO, error, event, module-registration, generated-binding and older UI fixture files. The UI boundary test is a dedicated frontend test file.
- P14's attribute and item-description writes and P15's Publish popover remain subsequent planned work.

## Files changed

- Desktop commands and wiring: `apps/desktop/src-tauri/src/commands/{items,mod,publish}.rs`, `apps/desktop/src-tauri/src/lib.rs`; `apps/desktop/src/services/{bindings,items}.ts`; `apps/desktop/src/features/sessions/NewSessionDialog.tsx`.
- Desktop tests: `apps/desktop/src/features/{form/__tests__/p11_form_ui,sessions/__tests__/p07_shell_sessions_ui,sessions/__tests__/p13_publish_runs,workspace/__tests__/p08_workspace_ui}.test.tsx`.
- Core: `crates/core/src/domain/{item,publish}.rs`, `error.rs`, `events.rs`, `mechanisms/{mod,naming,publish,reconcile}.rs`, `ports/{commerce,store}.rs`, `services/{attributes,export,items,mod,publish}.rs`.
- Infrastructure and integration: `crates/infra/src/sqlite/store.rs`, `crates/integrations/src/commerce/shopify/{adapter,graphql}.rs`, `crates/integrations/src/lib.rs`.
- Testkit: `crates/testkit/src/{fake_commerce,lib,test_app}.rs`, `crates/testkit/tests/p13_publish_runs.rs`.
- Process/evidence: `docs/implementation/{master_plan.md,plans/P13_publish_runs.md}`, `docs/intention/evidence/shopify_admin.md`.
- This handoff is left for the runner to commit, per flow §8.

## Checkpoint SHA

`3b034dd7a56ce154ec4f49aa5caeae73b2835dfe` — `CHECKPOINT (not approved): implement P13 publish runs`.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | PASS: 343 Rust tests, including 34 P13 boundary tests; doc tests pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cd apps/desktop && npm run typecheck` | PASS |
| `cd apps/desktop && npm test` | PASS: 79 tests, including C9(f) |
| `git diff --check` | PASS |

## Manual try-out

1. With the test Shopify integration configured, open New Session and search for a title that exists only in the shop. The result says “in the shop”; select it and verify a session opens for that item.
2. To exercise publishing without the P15 popover, run `cargo test -p psm-testkit --test p13_publish_runs c7a_done_records_publish_and_notification -- --exact`. The fake shop receives an ordered Main gallery and the local item records publication plus a notification.
3. Run `cargo test -p psm-testkit --test p13_publish_runs c8b_restart_requeues_running_upload_job_and_completes -- --exact` to see restart and upload recovery checked without hardware or a live shop.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
