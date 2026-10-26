---
plan: P14_publish_update
role: implement
round: 1
state: DONE
date: 2026-10-26
actor: codex
---
# P14 implement handoff

## What was built
Publishing now sends changed, valid item fields to Shopify and checks their stored values. It preserves admin-added tags, updates the title and SKU without clearing a barcode, and sends a manual description as HTML. Selected condition photos become files in the Damage report field in photo order. A Capture One retouch sends a revisioned photo, replacing its gallery media in the same position; earlier condition files remain in Files. The confirmation summary includes image, unreviewed, removal and product-create notes. A missing configured condition field allows the rest of the publish to finish and explains the skipped photos in its notification. The MVP run ends after `update`.

## Tests → criterion rows
`crates/testkit/tests/p14_publish_update.rs` has one boundary test per row:

| Rows | Tests | Result |
|---|---|---|
| C1(a–e) | `c1a_changed_confirmed_attribute_only` through `c1e_readback_mismatch_fails_update_with_key` | 5 pass |
| C2(a–c) | `c2a_admin_added_tag_kept_without_write` through `c2c_local_addition_merges_and_persists_base` | 3 pass |
| C3(a, c, b) | `c3a_title_written_only_when_different`, `c3c_identity_sku_only_preserves_barcode_and_mismatch_fails`, `c3b_manual_description_html_and_absence` | 3 pass |
| C4(a–d) | `c4a_selected_condition_files_in_order` through `c4d_missing_condition_field_skips_files_and_notifies` | 4 pass |
| C5(a–d) | `c5a_retouch_prepares_revision_and_new_filename` through `c5d_crash_adopts_only_current_revision` | 4 pass |
| C6(a) | `c6a_summary_counts_exclusions_removal_and_create` | pass |
| C7(a) | `c7a_full_run_emits_prepare_upload_update_done_without_describe` | pass |

Core mechanism tests: `mc13_write_set_canonical_and_writable`, `mc13_tags_preserve_remote_additions_and_local_removals`, and `mc21_replacement_candidates_only_newer_ready_links` pass. Existing P13 boundary tests pass with the required new summary wording. No camera, Capture One process, Shopify service or network is needed by the suite.

## Assumptions
- The first tag merge uses an empty base if `item_links.tags_base` has no value. It preserves existing remote tags and adds local tags.
- The missing-field notification is appended when the confirmed condition selection has photos to skip.
- The adapter converts the confirmed manual description to HTML. The service reads the current HTML to avoid a write for a missing local description.
- The RAW generation captured by the export render tracking job is the generation recorded on the photo link. A later retouch remains dirty for the next publish.
- Attribute batches contain at most 25 values, from SHOPIFY-VERIFY-5.

## Deviations
No semantic deviation. The plan's expected-file list was extended where compilation and the required read paths demanded it: `domain/publish.rs` exposes link revision fields already in the schema; `ports/commerce.rs`, Shopify `adapter.rs` and `types.rs` read remote product tags/description and the configured condition key; `test_app.rs` supplies a configurable fake condition key; P13 tests use P14's confirmed summary wording. The store persists render-request generation in existing job payloads; no migration was changed. The row C4(c) test also checks that an explicit stored empty condition selection writes `[]`, as its plan note requires.

## Files changed
- `crates/core/src/domain/publish.rs`
- `crates/core/src/mechanisms/attribute_write.rs` (new), `mod.rs`, `publish.rs`
- `crates/core/src/ports/commerce.rs`, `store.rs`
- `crates/core/src/services/publish.rs`
- `crates/infra/src/sqlite/store.rs`
- `crates/integrations/src/commerce/shopify/adapter.rs`, `types.rs`
- `crates/testkit/src/fake_commerce.rs`, `test_app.rs`
- `crates/testkit/tests/p13_publish_runs.rs`, `p14_publish_update.rs` (new)
- `docs/implementation/master_plan.md`
- `docs/implementation/plans/P14_publish_update.md`
- `docs/implementation/handoffs/implementer/handoff_P14_publish_update_implement_r1.md`

## Checkpoint SHA
`HEAD` after this checkpoint commit (`git rev-parse HEAD` resolves it). The handoff is part of that commit, so it uses a symbolic reference rather than embedding its own object hash.

## Stamp
| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test p14_publish_update` | 21 passed |
| `cargo test --workspace` | pass, no failures |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `cd apps/desktop && npm run typecheck` | pass |
| `cd apps/desktop && npm test` | 79 passed |

The first Clippy close run found three formatting-style `collapsible_if` warnings in newly edited Rust. They were corrected before the green Clippy and final Rust suite stamp. No pre-existing failure remains.

## Manual try-out
1. Open a session with main and condition photos. Fill two Details fields and a manual description, then publish. In Shopify admin, check the changed metafields and HTML description. An admin-only field and an admin-added tag should remain.
2. In the Damage report picker, untick one condition photo and publish. The Shopify condition metafield should list only the selected files in condition order.
3. Retouch an already published photo in Capture One and wait for the adjusted preview. The session should show Ready · Publish. Publish again; the shop should show the new image in the same gallery position.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
