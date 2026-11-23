---
plan: H1_points_and_remote_record
role: implement
round: 1
state: DONE
date: 2026-11-23
actor: claude (opus, pipeline-implementer)
---
# Handoff — H1_points_and_remote_record, implement r1

**Gate:** the intention is RATIFIED (2026-10-24). `depends_on: []`. The plan's Tracker line read NOT_STARTED. I worked the two units in order. After each unit I ran its **Unit test commands** and made a `CHECKPOINT (not approved):` commit. Then I ran the closing gate once, at the end.

## What was built
**In product terms.** Nothing new shows on screen yet. The app now knows these things:
- **Shopify's points.** After every Shopify answer (also a "throttled" one), the app keeps Shopify's remaining points. It also estimates how many points have come back since then. Background work waits while fewer than a quarter of the points are left: the first read of a shop product, and the clean-up's check before it clears an item. Your own presses (Refresh from Shopify, and later publish and bring back) never wait.
- **The record of remote pictures.** For an item whose pictures are only on Shopify, the app records which pictures Shopify holds (gallery, then condition photos, in order, each with its address and size). This covers an archived item (cleared by the clean-up) and a shop product adopted from the search that was never photographed on this Mac.
  - For a shop product, the pictures are read once, the first time they are needed. Later opens use the record.
  - A product whose pictures Shopify is still processing is not recorded. It is asked again later.
  - **Refresh from Shopify** reads the pictures again, replaces the record and deletes cached picture files that no longer match. An archived item stays archived.
- **The clean-up.** It never clears a shop-only product (it has nothing on this Mac to clear). It never clears an item brought back after its last completed publish. It now also records each picture's size, so a later "Bring back" can say how many MB it will download.
- **Downloads.** The adapter can download a picture at full size. It can tell "this picture is gone from Shopify" (HTTP 403/404) from "the download failed". It can ask Shopify for up to 25 picture addresses in one request.

**In code terms.** U1 `9ec9baf`:
- core: `domain/remote.rs` (`CallBudget`) and `mechanisms/remote.rs` (`BUDGET_FLOOR`, `URL_REFRESH_CHUNK`, `must_wait`).
- port `CommerceAdapter::{budget, media_urls, fetch_original}`, with the defaults of master plan archive §4.
- `MediaImage.bytes`; `Error::ImageGone` → `image_gone`.
- `HttpTransport::download` → `Download {status, bytes}`; the inherent `ReqwestTransport::download` is renamed `get_file`.
- adapter: `ThrottleReading` in a `Mutex<Option<…>>`, written right after `with_token` returns, before any retry or error return, and never held across a request. Also `projected`, `media_urls` (`MEDIA_URLS`), `fetch_original`, a private `get`, and the size fields in `MEDIA_READBACK` and `media_readback_of`.
- fixtures: `h1_media_urls.json` and `h1_media_readback_sizes.json`.
- fakes: `FakeTransport::{on_download, downloads}` with `Recorded::Download`; `FakeCommerce::{set_media_bytes, set_budget}`; a `Processing` gallery media answers `url: None`.

U2 `74e952b`:
- storage: `020_remote_items.sql`; `ItemRetention.{restored_at, remote_checked_at}`; `ArchivedImage.bytes`; `Store::{set_remote_record, update_remote_urls, remote_input}`.
- core rule: `RemoteKind`, `RemoteState`, `RemoteInput`, `remote_state`.
- service: `services/remote.rs` with `RemoteItems::{new, state, ensure_record, refresh, queue_record, run_pending}` and `Pending`.
- app wiring: `App.remote`, `AppHandles.remote`, the `remote_background` thread with a bounded join, `TestApp::remote()`; `Error::NotRemote` → `not_remote`.
- retention: `RetentionInput.{shop_only, restored_at}`. `retention::archived_file_name` is used by `cached_path` and `refresh`. `verify` copies sizes. The scan's budget wait sets `scan_requested`.
- guide updates.

## Tests → criterion rows
| Row | Test | Shape |
|---|---|---|
| H1·C1(a) | `h1_shopify_reads::h1_c1a_no_call_no_budget` | `budget() == None` |
| H1·C1(b) | `h1_c1b_budget_read_from_a_captured_answer` | `Some {1999, 2000}` exactly |
| H1·C1(c) | `h1_c1c_an_answer_without_extensions_keeps_the_reading` | unchanged `Some {1999, 2000}` |
| H1·C1(d) | `h1_c1d_integer_numbers_are_read` | `Some {954, 1000}` |
| H1·C1(e) | `h1_c1e_the_newest_reading_wins` | `Some {500, 2000}` |
| H1·C1(f) | `h1_c1f_a_throttled_answer_is_read` | `Err(Provider)` and `Some {12, 2000}` |
| H1·C2(a–d) | inline `adapter::tests::h1_c2a…h1_c2d` | exact `f64` per input |
| H1·C3(a–d) | inline `mechanisms::remote::tests::h1_c3a…h1_c3d` | exact bool per input |
| H1·C4(a) | `h1_c4a_urls_sizes_and_a_missing_media` | full `Vec` equality; one request with the three ids in order |
| H1·C4(b) | `h1_c4b_twenty_six_ids_take_two_requests` | 26 ids in order; requests of 25 and 1 |
| H1·C4(c) | `h1_c4c_no_ids_no_request` | `[]`, no request at all |
| H1·C4(d) | `h1_c4d_a_media_still_processing_has_no_url` | `[Some {id, None, None}]` |
| H1·C4(e) | `h1_c4e_a_short_answer_is_refused` | `Err(Provider)` containing `1 of 2` |
| H1·C5(a) | `h1_c5a_gallery_carries_sizes` | full gallery equality |
| H1·C5(b) | `h1_c5b_condition_files_carry_sizes_in_the_value_order` | full condition equality (three entries) |
| H1·C5(c) | `h1_c5c_original_source_fallback_keeps_its_size` | the original URL and size |
| H1·C5(d) | `h1_c5d_a_missing_product_is_not_found` | `Err(NotFound)` |
| H1·C6(a) | `h1_c6a_fetch_image_asks_for_the_width` | bytes; the download list is exactly `[…a.jpg?width=320]` |
| H1·C6(b) | `h1_c6b_fetch_original_asks_for_the_url_unchanged` | bytes; the download list is exactly `[url]` |
| H1·C6(c) | `h1_c6c_fetch_image_404_is_gone` | `Err(ImageGone)` |
| H1·C6(d) | `h1_c6d_fetch_original_403_is_gone` | `Err(ImageGone)` |
| H1·C6(e) | `h1_c6e_another_status_is_a_provider_error_without_the_url` | `Err(Provider)` whose text has no host and no file name |
| H1·C6(f) | `h1_c6f_fetch_original_refuses_http` | `Err(InvalidData)`; no request recorded |
| H1·C7(a–g) | inline `mechanisms::remote::tests::h1_c7a…h1_c7g` (one test per row; (g) checks both inputs) | exact state |
| H1·C8(a) | `h1_remote_record::h1_c8a_record_round_trip` | full record equality, gallery first; `(77, None, None)` |
| H1·C8(b) | `h1_c8b_an_older_database_keeps_its_retention_record` | `(true, Some(5), None, None)`; two pictures with `bytes: None` |
| H1·C9(a) | `h1_c9a_first_open_records_the_shop_pictures` | full record; checked = now; `ShopOnly`; 1 read; 1 `SessionChanged` |
| H1·C9(b) | `h1_c9b_a_product_without_pictures_is_local` | empty record; checked set; `Local`; 1 read |
| H1·C9(c) | `h1_c9c_later_opens_use_the_record` | read count 1 after two calls |
| H1·C9(d) | `h1_c9d_two_callers_make_one_read` | 1 read under a 100 ms hook; full record |
| H1·C9(e) | `h1_c9e_an_unreachable_shop_writes_nothing` | `Err(Provider)`; nothing written; `Unknown` |
| H1·C9(f) | `h1_c9f_a_picture_still_processing_writes_nothing` | `Err(Provider)` containing `still processing`; nothing written; `Unknown` |
| H1·C9(g) | `h1_c9g_the_queued_read_waits_for_the_budget` | `Waiting` with 0 reads, then `Worked`, 1 read, `ShopOnly` |
| H1·C10(a) | `h1_c10a_refresh_replaces_the_record` | record ids equal the reordered shop; `cleared_at` equal; 1 read |
| H1·C10(b) | `h1_c10b_a_press_never_waits_for_the_budget` | the same, with the budget `{100, 1000}` set after setup |
| H1·C10(c) | `h1_c10c_a_local_item_is_not_remote` | `Err(NotRemote)`; 0 reads |
| H1·C10(d) | `h1_c10d_refresh_deletes_stale_cache_files` | `(false, false, true)` |
| H1·C10(e) | `h1_c10e_reading_and_refreshing_make_no_shop_change` | `mutation_ops()` equal to the value after setup |
| H1·C11(a–c) | inline `mechanisms::retention::tests::h1_c11a…h1_c11c` | `Never` / `Never` / `Due` |
| H1·C12(a) | `h1_c12a_the_clearing_records_picture_sizes` | the recorded sizes equal the fake's sizes, in order |
| H1·C12(b), (c) | `h1_c12bc_the_clearing_waits_while_the_budget_is_low` | (b): neither cleared, 0 reads, no `retention_not_cleared`; (c): both cleared |

Counts: 56 rows → 55 tests (suite: `h1_shopify_reads::` 21, `h1_remote_record::` 16; inline: 4 + 4 + 7 + 3). Every test in the two new files maps to a row. There are no orphan tests.

## Assumptions
1. **The budget wait is logged once per waiting spell.** The line is `remote.budget_wait available= maximum=`. Both background steps (`run_pending` and the storage scan) log it when a wait starts, not on every 250 ms tick.
2. **The storage scan checks the budget before each item's verification.** While it must wait, it stops and requests the scan again. The items left are counted as freed for the low-space warning, as the existing `ITEMS_PER_SCAN` stop already does.
3. **`run_pending` checks the budget before each queued read.** It answers `Worked` if at least one read ran, else `Waiting`. Reads it did not make stay queued. Without an integration it drops the queue and answers `Idle`.
4. `ensure_record` and `refresh` return the state afterwards (`Result<RemoteState>`).
5. `refresh` without an integration answers `NotRemote` (with no integration every item is local, H6).
6. Texts:
   - `NotRemote`: `This item's photos are on this Mac, not in the shop`.
   - The still-processing error: `a picture of this item is still processing in the shop`.
   - The record log line: `remote.record item=<item id> kind=<state> pictures=<n>`.
7. The budget reading lives in the adapter struct (`ThrottleReading`). `fileSize` is parsed on `UrlHolder` for gallery nodes. Condition references and `nodes(ids:)` are read from the JSON directly, as the existing condition code does.
8. `RemoteState` gets a hand-written serde impl, so its wire form is `local` / `unknown` / `archived` / `shop_only` (registry). No command sends it yet.
9. Only the two constants H1 reads were added (`BUDGET_FLOOR`, `URL_REFRESH_CHUNK`). The others arrive with the milestone that uses them.
10. `remote_input` uses `psm_core::services::retention::INTEGRATION`, so no second `"shopify"` literal was added.
11. Test inputs:
    - H1·C8(a) hands the store the condition picture first, so "gallery first" is something the store does, not something the input already had.
    - H1·C12(b) runs the full `clear_all` (many scans) with the low budget. That is stronger than "one scan". (c) follows (b) in the same test because the row says "after (b)".
12. `TestApp` variants that set `run_startup` (only the camera-script ones) also start `remote_background`. Without a shop it stays idle.

## Deviations
- `crates/testkit/tests/suite/p12_shopify_adapter.rs` gains one match arm (`Recorded::Form { .. } | Recorded::Download { .. }`). Its exhaustive match over `Recorded` does not compile once task 3 adds the variant. This file is outside the plan's file list.
- `crates/testkit/tests/suite/p01_storage_identity.rs` gains the three new columns in its expected schema. It is the inherited tripwire that lists every column, and it failed in the closing gate as it should. This file is outside the plan's file list.
- **No red baseline was captured.** I wrote the tests after the production code. flow.md does not ask for a baseline, and I am not reconstructing one.
- Actor: the prompt template says `codex`. This session ran on Claude Opus (`PSM_IMPLEMENTER=claude`), and the tracker and this handoff say so.
- `update_remote_urls` is built (U2 task 2) but no H1 row calls it. Its first caller and row are in H2.

## Files changed
U1 (`9ec9baf`):
- `apps/desktop/src-tauri/src/commands/items.rs`
- core: `crates/core/src/{domain/mod.rs, domain/publish.rs, domain/remote.rs (new), error.rs, mechanisms/mod.rs, mechanisms/remote.rs (new), ports/commerce.rs}`
- adapter: `crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs, transport.rs, types.rs, fixtures/h1_media_urls.json (new), fixtures/h1_media_readback_sizes.json (new)}`
- testkit: `crates/testkit/src/{fake_commerce.rs, fake_transport.rs}`
- tests: `crates/testkit/tests/suite/{main.rs, h1_shopify_reads.rs (new), p12_shopify_adapter.rs}`

U2 (`74e952b`):
- `apps/desktop/src-tauri/src/commands/items.rs`
- core: `crates/core/src/{domain/remote.rs, domain/storage.rs, error.rs, mechanisms/remote.rs, mechanisms/retention.rs, ports/store.rs, services/mod.rs, services/remote.rs (new), services/retention.rs, services/storage.rs}`
- infra: `crates/infra/src/sqlite/{store.rs, migrations/020_remote_items.sql (new)}`
- app: `crates/integrations/src/lib.rs`
- testkit: `crates/testkit/src/test_app.rs`
- tests: `crates/testkit/tests/suite/{main.rs, h1_remote_record.rs (new), p27_retire_shape_on_arrival.rs, p62_retention_after_publish.rs}`
- guide: `docs/guide/{02_architecture.md, 04_change_impact.md}`

Closing commit: `crates/testkit/tests/suite/p01_storage_identity.rs`, `docs/implementation/plans/H1_points_and_remote_record.md` (Tracker line and Review log), and this handoff.

No probe touched any other file. No mutation regime (flow §4). No architecture graph in this repo. No bindings change: `bindings.ts` is unchanged after `cargo test -p psm-desktop`.

## Checkpoint SHA
- U1: `9ec9baf` — CHECKPOINT (not approved): H1·U1 Shopify points, URL read, sizes, typed downloads
- U2: `74e952b` — CHECKPOINT (not approved): H1·U2 record of remote pictures, first-open read, refresh, retention exclusions
- Close: the commit that adds this file (subject `CHECKPOINT (not approved): H1 close — …`).

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace` | every test program ran. Suite 1204/1205 on the first run: `p01_storage_identity::c1a_open_creates_all_domain_columns` failed (the schema tripwire; see Deviations). After the fix, `cargo test -p psm-testkit --test suite p01_storage_identity::` 20/20. The other programs were green: core 137, desktop lib 340, research engine 270, infra 80, integrations 77, and the rest. The doc-tests were skipped after the failure; `cargo test --workspace --doc` then ran green. |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `npm test` | 121 files, 806 tests passed |

Unit commands (all green): U1 `mechanisms::remote` 4, `commerce::` 9, `h1_shopify_reads::` 21, `p12_shopify_adapter::` 73, `p62_retention_after_publish::` 12, `cargo check -p psm-desktop`, clippy on core/integrations/testkit. U2 `mechanisms::remote` 11, `mechanisms::retention` 6, `h1_remote_record::` 16, `p62_…` 12, `p27_…` 11, `a17_delete_session::` 8, `cargo check -p psm-desktop`, clippy on core/infra/integrations/testkit.

## Manual try-out
There is nothing to try on screen in this milestone. The next milestone's sessions list shows what this one prepared.

For the orchestrator's live check after H4 (from the plan):
- The log line `remote.budget_wait` appears only when a background step waited.
- The live check must include one 10F clearing verification. The readback now asks Shopify for `originalSource.fileSize` and `GenericFile.originalFileSize`. Those names, and `nodes(ids:)`, are schema-grounded only. A wrong name would make every clearing report "Shopify could not be read".
- Install no build carrying H1 before that check.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
