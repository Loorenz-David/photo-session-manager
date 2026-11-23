---
plan: L2_labels_box
role: implement
round: 1
state: DONE
date: 2026-11-23
actor: claude-opus (pipeline-implementer)
---
# L2 — The Labels box · implement r1

## What was built
- **A Labels box in the sessions list.** After Open / Published / All there is a fourth box, **Labels**. Its number is
  how many priced products still lack a printed price label in Pricing. The number is asked once when the Sessions list
  opens, after you leave the box, and after a label changes elsewhere (at most once per 2 s).
- **Inside the box.** The toolbar shows the sub-filters Missing price tag (default) · Not printed · Printed · All with
  Pricing's counts, a search that goes through Pricing (SKU, title, product type), and the line "Products created since
  19 Nov 2026". The time filter, "To review", the Unassigned card and the time groups are hidden. Rows come 25 per page
  in Pricing's order, with **Previous / Next** and "Page 2 of 5".
- **Rows.** A product whose shop product is linked to an item on this Mac, or whose SKU equals a local item's SKU, looks
  and opens like that session: its name, SKU, picture, pills and ⋯. Any other product shows Pricing's title and its
  small shop picture (fetched in the background through the existing search-picture cache, `https://` only). Clicking
  it adopts the shop product (one Shopify read, nothing written) and opens it as a shop-only item. Every row shows the
  price as "8 900 kr" (or "No price"), the label state (as a pill and in the Label column) and when the product was
  created.
- **Selection.** Rows with a SKU can be ticked, and the header box ticks the page. "Select all N" pages Pricing's list
  to the end and selects every product with a SKU, so the bar may read "All 58 selected". "Generate labels for N" is on
  the bar but greyed out until L3. The selection survives page changes and refreshes; a new filter, a new search,
  leaving the box and Escape clear it.
- **Staying current.** A `labels_changed` event inside the box asks the page on screen again, with its own cursor. It
  runs at once, then at most once more per 2 s, and keeps your ticks and your page. A failed refresh keeps the rows on
  screen and shows the message with **Try again**. Labels off or denied shows the reason with **Open Settings →
  Pricing**.
- **Behind it:** `LabelsService::{list, summary, select_all, open_product}` (MC-L1.1–.8), `match_product`,
  `LabelStore::linked_items`, four commands (`labels_list`, `labels_summary`, `labels_select_all`, `labels_open`), and the
  pure page state `labelPages.ts::pageStep`. Nothing about a label row is written to the database: counts and the SKUs
  last seen live in memory only.

## Tests → criterion rows (Task 0 map, both directions; every test discharges exactly the row it is named after)
Rust suite `crates/testkit/tests/suite/l2_labels_list.rs` (boundary `App.labels` on `TestApp::with_pricing()`):
| Row | Test | Shape as the row specifies? |
|---|---|---|
| L2·C1(a) | `l2_c1a_one_request_rows_newest_first_with_their_fields` | yes: exact request line list, row tuples in order, `missing_price_tag = 3`, `next_cursor None` |
| L2·C1(b) | `l2_c1b_pages_through_the_cursor` | yes: 25 + cursor; exact second request line with `cursor=o%3A25&limit=25`; 5 rows, None; 30 distinct ids |
| L2·C1(c) | `l2_c1c_the_search_is_trimmed_dropped_when_empty_and_cut` | yes: the three exact request lines |
| L2·C1(d) | `l2_c1d_work_from_is_passed_through` | yes |
| L2·C1(e) | `l2_c1e_a_session_marker_reads_the_list_state_without_a_status_request` | yes: state equal, no label request |
| L2·C1(f) | `l2_c1f_no_key_or_denied_answer_off_or_denied_without_a_request` | yes: `Off(NoKey)`, `Denied`, empty request lists |
| L2·C1(g) | `l2_c1g_provider_failures_map_to_their_errors` | yes: three errors, `access() = Denied` |
| L2·C2(a) | `l2_c2a_a_linked_session_is_the_rows_session` | yes: id, sku, title of the session |
| L2·C2(b) | `l2_c2b_a_local_item_with_the_same_sku_is_matched` | yes |
| L2·C2(c) | `l2_c2c_the_link_wins_over_the_sku` | yes |
| L2·C2(d) | `l2_c2d_an_unmatched_row_has_pricings_title_and_its_shop_id` | yes |
| L2·C2(e) | `l2_c2e_no_sku_or_a_sku_in_another_case_matches_nothing` | yes (both rows `None`) |
| L2·C2(f) | `l2_c2f_a_linked_item_without_a_session_gives_no_session` | yes |
| L2·C3(a) | `l2_c3a_a_matched_row_has_its_sessions_picture_while_the_file_exists` | yes: path, then None after deletion |
| L2·C3(b) | `l2_c3b_an_unmatched_rows_picture_comes_through_the_search_cache` | yes: None; file at `remote/search/<search_name>`; fetch `(url,160)`; exact `PictureReady` list; then `Some(file)` |
| L2·C3(c) | `l2_c3c_no_picture_without_an_https_address` | yes |
| L2·C3(d) | `l2_c3d_the_list_writes_nothing_to_the_shop_or_to_pricing` | yes |
| L2·C4(a) | `l2_c4a_a_linked_product_opens_its_session_without_a_shop_call` | yes (shop read log unchanged) |
| L2·C4(b) | `l2_c4b_an_unmatched_product_is_adopted_with_one_read_and_no_write` | yes: sku, origin Linked, link, reads delta = `["item_get"]`, no mutation |
| L2·C4(c) | `l2_c4c_opening_again_answers_the_same_session_and_the_row_is_matched` | yes |
| L2·C4(d) | `l2_c4d_a_linked_item_without_a_session_gets_one_without_a_shop_call` | yes |
| L2·C4(e) | `l2_c4e_a_shop_product_without_a_sku_creates_nothing` | yes (item and session counts unchanged) |
| L2·C4(f) | `l2_c4f_a_sku_collision_creates_nothing` | yes |
| L2·C4(g) | `l2_c4g_a_product_id_that_is_no_shop_id_is_invalid_without_a_shop_call` | yes |
| L2·C5(a) | `l2_c5a_a_refreshing_summary_asks_one_unsearched_row` | yes |
| L2·C5(b) | `l2_c5b_a_summary_without_refresh_never_asks` | yes |
| L2·C5(c) | `l2_c5c_a_fresh_count_is_not_asked_again` | yes (at 9 s: none; at 11 s: one) |
| L2·C5(d) | `l2_c5d_the_number_follows_the_search_until_a_refresh` | yes |
| L2·C5(e) | `l2_c5e_no_number_and_no_request_without_access` | yes |
| L2·C5(f) | `l2_c5f_test_connection_forgets_the_number` | yes |
| L2·C6(a) | `l2_c6a_select_all_pages_to_the_end_in_the_lists_order` | yes: exact two request lines; 620 ids in list order |
| L2·C6(b) | `l2_c6b_select_all_keeps_only_products_with_a_sku` | yes (exact `Selected` list) |
| L2·C6(c) | `l2_c6c_every_request_carries_the_filter_and_the_search` | yes |
| L2·C6(d) | `l2_c6d_a_repeated_cursor_is_a_bad_reply_not_an_endless_walk` | yes |
| L2·C6(e) | `l2_c6e_a_failed_page_is_the_error_and_no_partial_selection` | yes |
| L2·C6(f) | `l2_c6f_select_all_without_a_key_is_off_without_a_request` | yes |

Core mechanism unit tests (`crates/core/src/mechanisms/labels.rs`, `match_product`, U1 task 1): `match_by_link`,
`match_by_sku`, `the_link_wins_over_the_sku`, `two_items_linked_to_one_product_match_the_lowest_item_id`,
`no_sku_and_no_link_match_nothing`, `a_sku_differing_only_in_case_matches_nothing`.

Desktop `apps/desktop/src-tauri/src/lib.rs` `l2_tests`: L2·C7(a) `l2_c7a_labels_list_answers_rows_counts_and_work_from`
(whole JSON exact; the matched row's `session` checked by id and SKU, then the rest exact) · C7(b)
`l2_c7b_off_and_denied_are_their_codes` (`labels_off` whole error exact; denied code and message) · C7(c)
`l2_c7c_an_unavailable_list_is_labels_list_unavailable` · C7(d) `l2_c7d_labels_summary_answers_the_number_or_null` ·
C7(e) `l2_c7e_labels_select_all_answers_the_products_with_a_sku` · C7(f) `l2_c7f_labels_open_answers_the_new_session`.

UI `features/labels/__tests__/l2_label_pages.test.ts`: C8(a)–(i) `l2_c8a_…` … `l2_c8i_…` (one test each).
`l2_label_words.test.ts`: C9(a)–(d). `l2_labels_box.test.tsx`: C10(a)–(g), C12(a)–(c), C14(a)–(d).
`l2_labels_rows.test.tsx`: C11(a)–(d). `l2_labels_selection.test.tsx`: C13(a)–(e). Each test asserts the exact calls
(`labelsList` / `labelsSummary` / `labelsSelectAll` / `labelsOpen` argument lists) and the rendered text the row names.
C14(b) uses fake timers: one call at 0 s, none until 1.999 s after the window opened, the second at 2 s, none after.
No row is without a test; no test discharges no row. Red baseline: **not captured** (tests were written beside the
code, unit by unit; flow.md has no baseline regime). Each test passed on its first green run after its code was
written.

## Assumptions
1. `open_product` matches by MC-L1.3 using the SKU the **last list answer** showed for the product, kept in memory and
   forgotten with the counts. The command carries only the product id. A row whose SKU matched a session opens that
   session directly on screen.
2. `summary(refresh)` passes a provider error through: the screen keeps the number it had. `summary` also applies its
   one row to the display copy (delegated).
3. A connection that is `Ready` but has no store answers `Off(Ready)`.
4. The fake Pricing's cursor is `o:<offset>`, so percent-encoding is visible (`cursor=o%3A25`).
5. The box's search text is separate from the sessions search and is kept across leaving and re-entering. The
   sub-filter starts on Missing price tag at each entry (delegated).
6. **Try again** asks the first page (MC-L1.9) and keeps the selection.
7. The 2-second refresh limit counts only `labels_changed` events. Entering the box does not start the 2-second window.
8. Leaving the box asks `labelsSummary({refresh: true})` once, so the number outside is the unsearched one again. The
   service sends no request when its count is fresh and unsearched.
9. The marker words show both as the row's pill and in the Label column (U3 task 4 names both).
10. A matched row's ⋯ menu is given the access `allowed`, because a list answer implies it. An empty page reads "No
    products here".

## Deviations
- `apps/desktop/src/features/labels/useSessionLabels.ts` changed (not in U3's file list). When no session ids are on
  screen it now forgets the ids it had, so a `labels_changed` event inside the Labels box asks no `labels_status` (U3
  task 2). Behaviour with session rows on screen is unchanged, and the L1 marker tests are green.
- `docs/guide/02_architecture.md` and `04_change_impact.md` were also updated in U2 (commands) and U3 (frontend), as
  flow §7A asks; only U1 lists them.
- `FakePricing::fail_matching_after(path, skip, n, status)` was added to the test kit (needed for L2·C6(e): the second
  page fails while the first passes). The plan's L2 list does not name it.
- Tracker actor `opus`: the session ran on Claude Opus; the prompt template says `codex`.
- `l1_markers.test.tsx` changed only its `@/services/labels` mock (gains `labelsSummary` → `{missing_price_tag: null}`),
  as U3 says. No existing assertion changed. The other existing tests that render `SessionsPage` do not mock
  `@/services/labels`; the page guards the call exactly as `useSessionLabels` does.

## Files changed
U1 (`e81b52d3`): `crates/core/src/{domain/label.rs, mechanisms/labels.rs, ports/labels.rs, services/labels.rs}`,
`crates/infra/src/sqlite/labels.rs`, `crates/integrations/src/{labels.rs, lib.rs}`,
`crates/testkit/src/{fake_pricing.rs, fake_commerce.rs}`, `crates/testkit/tests/suite/{main.rs, l2_labels_list.rs (new)}`,
`docs/guide/{02_architecture.md, 04_change_impact.md}`.
U2 (`70f4d19b`): `apps/desktop/src-tauri/src/{commands/labels.rs, lib.rs}`, `apps/desktop/src/services/{bindings.ts
(generated), labels.ts}`, `apps/desktop/src/features/labels/{labelWords.ts, labelPages.ts (new)}`,
`apps/desktop/src/features/labels/__tests__/{l2_label_pages.test.ts, l2_label_words.test.ts}` (new),
`docs/guide/{02_architecture.md, 04_change_impact.md}`.
U3 (`80a720ca`): `apps/desktop/src/features/labels/{LabelsBox.tsx, LabelRow.tsx, LabelsBatchBar.tsx}` (new),
`apps/desktop/src/features/labels/useSessionLabels.ts`, `apps/desktop/src/features/sessions/{SessionsPage.tsx,
Filters.tsx}`, `apps/desktop/src/styles/globals.css`, `apps/desktop/src/features/labels/__tests__/{l2_labels_box,
l2_labels_rows, l2_labels_selection}.test.tsx` (new), `apps/desktop/src/features/labels/__tests__/l1_markers.test.tsx`
(mock only), `docs/guide/02_architecture.md`.
Close (`3fe83622`): `docs/implementation/plans/L2_labels_box.md` (Tracker line, Review log).
This handoff: `docs/implementation/handoffs/implementer/handoff_L2_labels_box_implement_r1.md` (left for the runner).
Mutation probes: none (flow §4: no mutation regime). No architecture graph in this repo.

## Checkpoint SHA
U1 `e81b52d3` · U2 `70f4d19b` · U3 `80a720ca` · close `3fe83622` (tree clean after it, except this handoff).

## Stamp
Run once at the end, one command after another, at `80a720ca` (working tree = U3):
- `cargo test --workspace`: exit 0. suite 1395 passed, research engine 340, desktop 103, core 165, infra 139,
  integrations 270, others green; 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo fmt --all -- --check`: exit 0.
- `cd apps/desktop && npm run typecheck`: exit 0.
- `npm test`: 902/903. The one failure is `features/workspace/__tests__/p33_split_grid.test.tsx` C4(d), listed as known
  intermittent in `docs/guide/04_change_impact.md` §Fragile spots. Re-run alone: 7/7 green. No pre-existing failures
  otherwise.
Per-unit commands (all green): U1 `mechanisms::labels` 14, `l2_labels_list::` 36, `l1_labels_markers::` 15, `cargo check
--workspace`, clippy on four crates · U2 `cargo test -p psm-desktop` 103, `vitest src/features/labels` 27, typecheck,
desktop clippy · U3 `vitest src/features/labels src/features/sessions` 117, typecheck.

## Manual try-out
Needs a Pricing key with `labels:read`, on the installed app (no session launches it).
1. Open Sessions. Next to Open / Published / All there is a **Labels** box with the number of labels still to print.
2. Click it. You see priced products without a printed label, newest first, each with a picture, SKU, price ("8 900 kr")
   and "Missing price tag". Under the toolbar: "Products created since <date>".
3. Click **Printed**: printed products with "Label printed · <date>". Type a SKU in the search box: the list and the
   numbers follow.
4. With more than 25 products, press **Next** and **Previous**; the line reads "Page 2 of N".
5. Click a product you photographed on this Mac: its session opens. Click one from the other Mac: it opens as a
   product whose photos are on Shopify (nothing is written to Shopify).
6. Tick two rows: "2 selected". Tick the header box, then "Select all N": "All N selected". "Generate labels for N" is
   greyed out (it comes with L3).
7. Print a label in Pricing: within about ten seconds the product leaves Missing price tag and the number drops, with
   no sound.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
