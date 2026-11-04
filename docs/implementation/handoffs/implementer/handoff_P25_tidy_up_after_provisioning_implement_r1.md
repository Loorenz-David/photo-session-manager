---
plan: P25_tidy_up_after_provisioning
role: implement
round: 1
state: DONE
date: 2026-11-04
actor: codex
---

# P25 implement handoff

## What was built

Capture One's failed session open now closes the app's temporary window and reports the failure. The front-document warning clears when no Session Manager session is active, while activation still refuses to capture into the wrong document. Settings exposes the fixed path of the app's own Capture One session. A fresh setup leaves the next-capture Style empty; an explicit saved style remains. A newly captured RAW with no recorded hash is hashed during export or publish preparation. Startup logging begins before the app opens. Health checks leave unknown crop-ratio jobs failed until Capture One lists the ratio. Shopify media creation waits for a filename and identifies the created media by that name, including a Shopify UUID suffix and gallery fallback.

## Tests → criterion rows

| Row | Boundary test and exact outcome |
|---|---|
| C1(a) | `p25_tidy_up_after_provisioning::c1a_open_document_closes_only_its_throwaway_on_silent_refusal`: three Node harness modes assert the ordered `make`, `open`, and conditional throwaway `close` calls. |
| C2(a) | `p25::c2a_failed_activation_lapses_when_no_session_is_active`: failed activation, then check clears the warning, routes `_Unassigned`, and camera status/capture recover. |
| C2(b) | `p25::c2b_deactivation_clears_front_warning`: active warning, immediate deactivation clear, `_Unassigned`, and no later front switch. |
| C3(a) | `p25::c3a_default_document_path_stays_constant_across_settings_changes`: returned and fetched settings retain the fixed default through explicit and cleared document choices. |
| C4(a) | `p25::c4a_fresh_settings_leave_next_capture_style_alone`: fresh file and DTO contain empty style, healthy hooks, no style write. |
| C4(b) | `p25::c4b_existing_style_name_is_preserved_and_applied`: explicit style survives restart and is applied once without file rewrite; missing key defaults empty. |
| C5(a) | `p25::c5a_first_verification_fills_missing_hash_then_detects_change`: first hash persists and returns Ok; changed and missing RAWs return their exact integrity states. |
| C5(b) | `p25::c5b_publish_of_recent_unhashed_photo_passes_prepare`: publish has no failed step, persists the first hash, and reaches staged upload. |
| C5(c) | `p25::c5c_export_of_recent_unhashed_photo_writes_image`: local export writes the named main JPEG and persists the first hash. |
| C6(a) | Desktop `p25_tests::c6a_startup_marker_is_written_before_app_open`: a fake-backed open writes exactly one log file containing `app open` and the data directory. |
| C7(a) | `p25::c7a_unknown_shape_does_not_requeue_until_listed`: two checks retain the failed job; listing the ratio queues it and the next tick applies it. |
| C8(a) | `p12_shopify_adapter::c8a_created_media_is_found_by_filename_not_gallery_position`: four captured-fixture variants assert matching candidate, foreign newest/gallery fallback, UUID uniquifier, and missing-name error. |

Backfills: P24 `c1a`, `c1b`, `c1e`, `c3a` gained the planned absence/preservation assertions; `c1f` explicitly saves `PSM House`; P17 `c1b` checks the old recipe is untouched. P00's default-style expectation is now empty. P12 `c6c` supplies its required MediaPoll reply. Six frontend Settings fixtures include `default_document_id`.

## Assumptions

- Chose `Settings::default_document_id` for the fixed app path and `psm_core::mechanisms::upload_name::matches` for the one filename rule shared by publish and Shopify.
- Required the Shopify suffix to parse as UUID version 4, as MC-11 specifies.
- Left real-start shape requeue unchanged; only Health checks exclude `crop_ratio_unknown` while the setting is still unknown.
- The desktop logging test is the only test in that binary that installs the process-global tracing subscriber.

## Deviations

- Added `apps/desktop/src-tauri/Cargo.toml` and `Cargo.lock` outside the plan's expected file list because `open_logged` names the existing `tracing-appender` guard in its public return type.
- Updated the P24 Node harness to provide `name()` on its simulated document, which the revised script reads.
- No live Capture One or Shopify probe ran. The automated suite used fake adapters and captured Shopify fixtures only.

## Files changed

Checkpoint: `Cargo.lock`; `apps/desktop/src-tauri/Cargo.toml`, `src/commands/settings.rs`, `src/lib.rs`; six Settings test fixtures under `apps/desktop/src/features/{pending,settings,shell}/__tests__/`; `apps/desktop/src/services/bindings.ts`; `crates/core/src/mechanisms/{mod.rs,upload_name.rs}`, `src/ports/store.rs`, `src/services/{editing_startup,integrity,publish,sessions}.rs`; `crates/infra/src/{settings_file,sqlite/store}.rs`; `crates/integrations/src/{lib,commerce/shopify/adapter,editing/capture_one/jxa/open_document.js}`; `crates/testkit/tests/{p00_spike,p12_shopify_adapter,p17_real_run_fixes,p24_capture_one_provisioning,p25_tidy_up_after_provisioning}.rs`; `docs/implementation/{master_plan.md,plans/P25_tidy_up_after_provisioning.md}`. This handoff is written after the checkpoint for the runner to commit as its own artifact.

## Checkpoint SHA

`b2f2ce8ac738a64eda43224c8735e30281428f04` — `CHECKPOINT (not approved): implement P25 tidy-up after provisioning`.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace --quiet` | Exit 0; all Rust workspace tests pass, including 12 P25 boundary rows and the P24/P17 backfills. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0. |
| `cargo fmt --all -- --check` | Exit 0. |
| `cargo test -p psm-desktop export_bindings` | Exit 0; generated binding includes `default_document_id`. |
| `npm run typecheck` | Exit 0. |
| `npm test` | Exit 0; 18 files, 129 tests. |
| `git diff --check` | Exit 0. |

## Manual try-out

1. Back up the current `settings.json`, launch with a fresh one, and confirm the Style field is empty and Health has no Style error. Restore the saved file; its explicit style still appears.
2. Select a scratch `.cosessiondb` file that Capture One cannot open. Save and retry Health: it reports that file could not open, and Capture One's Window menu has no `PSM Open Window`. Select the app's own session again.
3. Activate a Session Manager session, put another Capture One document in front, and run Health checks: the app's document returns to the front. Deactivate, front the other document again, and retry: no front warning or forced switch remains; the camera is connected.
4. Shoot and export immediately; repeat with Publish on a ready session. Neither should claim the new RAW changed before its hash job ran.
5. After launch, inspect `~/Library/Application Support/com.loorenz.sessionmanager/logs/psm.<today>.log` for `app open` followed by timed provider calls.
6. Set `crop_on_arrival` to `Squircle`, shoot, and revisit Health: the ratio warning persists without new crop attempts. Choose a listed ratio and retry; the waiting photo gets shaped.
7. Publish new photos and check the log: a media create is followed by MediaPoll before the media link is treated as attached.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
