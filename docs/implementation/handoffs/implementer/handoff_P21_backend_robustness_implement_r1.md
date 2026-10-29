---
plan: P21_backend_robustness
role: implement
round: 1
state: DONE
date: 2026-10-29
actor: codex
---

# P21 implementation handoff

## What was built

The app starts with damaged settings, an incomplete shop connection, or a failed backup and shows
the cause in Health. Saving repairs a damaged settings file, and simultaneous settings saves keep
all changes. Shop search reports outages; Health reads shop fields before judging the condition
field. Jobs with no handler stay queued. Publishing records plain `Published` text and skips an
unchanged description. An empty barcode clears the stored barcode.

## Tests → criterion rows

Each row has one boundary test with the exact outcome named in the plan:

| Row | Test |
|---|---|
| C1(a) | `p21_backend_robustness::c1a_damaged_settings_start_and_save_repairs_file` |
| C1(b) | `p21_backend_robustness::c1b_incomplete_integration_starts_in_export_mode` |
| C1(c) | `p21_backend_robustness::c1c_failed_backups_do_not_stop_start_or_publish` |
| C2(a) | `psm_desktop::p21_tests::c2a_concurrent_settings_saves_preserve_every_change` |
| C2(b) | `p21_backend_robustness::c2b_exact_secret_mask_only` |
| C2(c) | `p21_backend_robustness::c2c_integration_defaults_appear_on_read` |
| C3(a) | `p21_backend_robustness::c3a_handlerless_runner_claims_nothing` |
| C3(b) | `p21_backend_robustness::c3b_unknown_kind_retries_and_known_job_completes` |
| C4(a) | `p21_backend_robustness::c4a_shop_search_surfaces_provider_error_local_search_stays_local` |
| C5(a) | `p21_backend_robustness::c5a_publish_notification_is_plain_text_with_stored_time` |
| C5(b) | `p21_backend_robustness::c5b_unchanged_description_is_not_sent_twice` |
| C6(a) | `p21_backend_robustness::c6a_empty_barcode_clears_and_none_keeps` |
| C7(a) | `p21_backend_robustness::c7a_health_reads_field_before_judging_it` |
| C7(b) | `p21_backend_robustness::c7b_health_distinguishes_unread_from_stored_fields` |

No P21 core mechanism unit tests were added; the plan lists none. The pre-edit red baseline was
not captured; these tests were verified green on the finished tree.

## Assumptions

- A malformed `settings.json` uses complete in-memory defaults until the next save, preserving
  the damaged bytes for diagnosis.
- An incomplete integration block makes the app use Export mode while preserving the other
  settings. Health names the missing field.
- A failed schema refresh uses stored definitions if they exist. With no stored definitions,
  Health reports that shop fields have not been read yet.
- A backup failure remains visible through a settings runtime rebuild and clears on a later
  successful backup. A theme-only save clears a repaired settings warning.

## Deviations

- `crates/infra/src/paths.rs` was needed beyond the expected file list: its eager creation of
  `backup/` rejected a regular file before backup handling could report the failure.
- The inherited P13 search test had to change its old outage expectation; it now expects the
  provider error and separately checks local search. Its notification assertion also follows
  the new text.
- Regenerated bindings gained the command's documented empty-barcode rule. No signature changed.

## Files changed

Checkpoint code and tests: `apps/desktop/src-tauri/src/commands/items.rs`, `apps/desktop/src-tauri/src/lib.rs`,
`apps/desktop/src-tauri/src/state.rs`, `apps/desktop/src/services/bindings.ts`,
`crates/core/src/ports/commerce.rs`, `crates/core/src/services/{health,items,publish,settings}.rs`,
`crates/infra/src/{jobs,paths,settings_file}.rs`,
`crates/integrations/src/commerce/shopify/adapter.rs`, `crates/integrations/src/lib.rs`,
`crates/testkit/src/fake_commerce.rs`,
`crates/testkit/tests/{p13_publish_runs,p21_backend_robustness}.rs`.

Coordination artifacts: `docs/implementation/master_plan.md` (P21 tracker row),
`docs/implementation/plans/P21_backend_robustness.md` (Review log), and this handoff. These
remain for the runner to commit.

## Checkpoint SHA

`e33ddfbf6d4192f6979bb41ae63a76eb4f38aa07` — `CHECKPOINT (not approved): implement P21 backend robustness`.
Only the 17 code, binding and test files above are in the checkpoint.

## Stamp

| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test p21_backend_robustness` | First run: 12 passed, backup-start case failed before the `paths.rs` fix. The failing case passed on its targeted rerun; the final workspace run passed all 13. |
| `cargo test -p psm-desktop p21_tests` | 1 passed. |
| `cargo test -p psm-desktop export_bindings` | Passed; generated binding committed. |
| `cargo test --workspace` | Passed at the checkpoint code tree, including 14 P21 row tests. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed. |
| `cargo fmt --all -- --check` | Passed. |
| `cd apps/desktop && npm run typecheck` | Passed with the planned Node 22 path. |
| `cd apps/desktop && npm test` | 12 files, 114 tests passed. |

No camera, Capture One, or live Shopify service was required by the automated checks.

## Manual try-out

1. Quit the app, remove the final `}` from its `settings.json`, then relaunch. The app opens;
   Settings → Health shows `settings.json` as invalid. Save settings and retry Health: the row
   disappears and the file is valid again.
2. Quit, remove `client_secret` from the integration block, then relaunch. The header offers
   Export and Health names the incomplete integration. Enter the secret and save to restore
   publishing.
3. Save a preview recipe change, immediately save an export-folder change, then relaunch. Both
   values remain.
4. With a store configured, turn Wi-Fi off and search the shop from New session. It reports
   shop unavailable. Health says shop fields have not been read yet if no earlier schema is
   stored.
5. Publish a session twice without changing its description. The notification text is
   `Published`, and the log contains one `update_description` mutation. P22 adds the visible
   local time in the notification drawer.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
