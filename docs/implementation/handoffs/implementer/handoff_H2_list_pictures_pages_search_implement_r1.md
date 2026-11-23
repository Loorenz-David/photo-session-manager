---
plan: H2_list_pictures_pages_search
role: implement
round: 1
state: DONE
date: 2026-11-23
actor: claude-opus (pipeline-implementer; the prompt template names `codex`, this session ran on Claude Opus as master_plan_archive.md §3 says)
---
# H2 — A picture on every row, pages, and pictures in the search: implementer handoff r1

## What was built
- **No row shows a broken picture any more.** A sessions-list row shows this Mac's preview when its file is there,
  otherwise (for a product whose files were cleared, or a shop product already opened) its first Shopify picture, kept
  small on this Mac, otherwise a grey placeholder. The same rule feeds the paged list, the unpaged list the other
  screens use, the hover card, the workspace's Sessions panel and the item search.
- **Pictures load in the background.** When a page is shown, its missing covers are queued; a background step
  downloads them four at a time and tells the window (`picture_ready`), which swaps the placeholder for the picture
  without reloading the list. A picture address Shopify has replaced is asked for again in one batched read (only when
  the Shopify points allow it, never more than once per 10 minutes per picture) and the download retried once; a
  picture Shopify no longer has stays a placeholder and is reported once on Health ("1 picture is no longer in the
  shop: <SKU>"); Refresh from Shopify or deleting the session clears that.
- **Pages of 25.** The Sessions page asks for one page (`sessions_page`); the search, the filter tabs, the time filter
  and "To review" look through every session and go back to page 1; page numbers sit under the list (all up to 7,
  otherwise first, last and around the current one, with Previous / Next); group headers show the group's full count.
- **Pictures in the search.** New session and Assign product show each result's picture: a local result its session's
  row picture (its cover is fetched if missing), a shop result the shop's featured picture fetched at 160 px into a
  200 MB cache (oldest files go first); the first letter shows while a picture loads or when there is none.
- **Delete session** also removes the product's picture caches (`previews/remote/<item>/`, `previews/archived/<item>/`);
  no Shopify call.

Code: `mechanisms::remote::{cover_name, page_of, search_name, evict}` and constants; `RemoteItems::{row_pictures,
request_covers, item_pictures, item_picture, search_picture, forget_item}` and the reworked `run_pending`
(`services/remote.rs`); `Store::remote_covers`; `SessionsService::{with_remote, page}`; `ItemsService::search_shop_pictures`
/ `ShopResult`; `AppEvent::PictureReady` → `picture_ready`; commands `sessions_page`, `items_search` /
`items_search_shop` with `picture_path` / `session_id`; UI `usePictures`, `Pagination`, `ResultThumb`, `Thumb`
placeholder, `rangeStart`.

## Tests → criterion rows
| Rows | Test (file :: name) |
|---|---|
| H2·C1(a)–(g) | `crates/testkit/tests/suite/h2_row_pictures.rs` :: `h2_c1a_…` … `h2_c1g_…` |
| H2·C2(a)–(d) | same file :: `h2_c2a_…` … `h2_c2d_…` |
| H2·C3(a)–(d) | same file :: `h2_c3a_…` … `h2_c3d_…` |
| H2·C4(a)–(f) | same file :: `h2_c4a_…` … `h2_c4f_…` |
| H2·C5(a)–(c) | same file :: `h2_c5a_…`, `h2_c5b_…`, `h2_c5c_…` |
| H2·C5(d) | `crates/core/src/mechanisms/remote.rs` (inline) :: `h2_c5d_cover_name` |
| H2·C6(a)–(d) | same module :: `h2_c6a_…` … `h2_c6d_…` |
| H2·C7(a)–(h) | `crates/testkit/tests/suite/h2_pages_and_search.rs` :: `h2_c7a_…` … `h2_c7h_…` |
| H2·C8(a)–(b) | `apps/desktop/src-tauri/src/lib.rs` `h2_tests` :: `h2_c8a_…`, `h2_c8b_…` |
| H2·C9(a)–(g), C10(a)–(e) | `apps/desktop/src/features/sessions/__tests__/h2_list_pages_and_pictures.test.tsx` :: `h2_c9a_…` … `h2_c10e_…` |
| H2·C11(a)–(c) | `h2_tests` :: `h2_c11a_…`, `h2_c11b_…`, `h2_c11c_…` |
| H2·C12(a) | `crates/testkit/tests/suite/h1_shopify_reads.rs` :: `h2_c12a_the_search_reads_the_featured_picture` |
| H2·C12(b) | `h2_pages_and_search.rs` :: `h2_c12b_…` |
| H2·C12(c)–(e) | `h2_tests` :: `h2_c12c_…`, `h2_c12d_…`, `h2_c12e_…` |
| H2·C13(a)–(c), (e) | `mechanisms/remote.rs` (inline) :: `h2_c13a_…`, `h2_c13b_…`, `h2_c13c_…`, `h2_c13e_…` |
| H2·C13(d) | `h2_pages_and_search.rs` :: `h2_c13d_…` |
| H2·C14(a)–(c) | `apps/desktop/src/features/sessions/__tests__/h2_search_pictures.test.tsx` :: `h2_c14a_…`, `h2_c14b_…`, `h2_c14c_…` |

## Assumptions
1. **One Store read for the row picture** (plan Notes, "a `Store` method … is the implementer's choice"):
   `Store::remote_covers()` returns, per item session, the remote-state inputs, item id, SKU and the record's cover
   picture. The list record's single `COALESCE(adjusted, original)` cover path is split into
   `cover_adjusted_preview_path` / `cover_original_preview_path`, which MC-H1.1(i) needs (fall back to the original
   when the adjusted file is gone).
2. **`row_picture` lives on `RowPictures`** (`RemoteItems::row_pictures()` reads once, then
   `RowPictures::row_picture(session_id, adjusted, original)` per row), so `list` never reads the store per row
   (plan hazard). Without `with_remote`, `list` applies `remote::local_picture` (rules i and iii).
3. `PictureTarget` has `Cover` and `Search` only; H4 adds `Remote`.
4. **Search pictures are queued in their own list**, deduplicated, not replaced by `request_covers(Search, …)`: the
   frontend runs `items_search` and `items_search_shop` side by side, and one would otherwise drop the other's
   downloads. Covers keep MC-H1.3's per-source replacement.
5. `item_pictures(&[item_id])` (one read for a whole result list) beside the registry's `item_picture(item_id)`.
6. After a search picture is written its modified time is set to the app clock, so `evict`'s oldest-first follows the
   clock (production: real time); this makes C13(d) deterministic and keeps the just-written file newest.
7. Test set-up: H2·C1, C2 and C5(b) use the real "published then cleared" archived item; C3 and C4, which need up to 13
   archived items, make them as `a17` does (local item linked to a fake product, `mark_item_cleared` with its record):
   the same stored state without a publish per item.
8. `ItemDto.picture_path` / `session_id` are left out of the JSON when empty (`skip_serializing_if`, TS
   `picture_path?: string`, `session_id?: number`): other commands answer them absent, which the UI reads like null;
   this avoided touching ~25 TS fixtures.
9. `sessions_page` takes `created_from` and `only_ids` as `CommandId` (an integer number), like the other commands.
10. "To review" sends the waiting ids sorted ascending; its count comes from the research overview over every session.
11. A clamped answer (asked page 3, answered page 2) moves the page state to the answer's page, which reloads that page
    once more.
12. The gone map (MC-H1.5) is left by a record replacement through `ensure_record` / `refresh` and by Delete session;
    a 10F clearing of an item already remote does not happen in H2's flows (bring back is H3).
13. `FakeCommerce::media_urls` hands out `fake_refreshed_url(id, n)` (`…?v=n`) so a refreshed address differs from
    the stored one; `fail_image` faults are consumed one per `fetch_image` of that exact URL.
14. `AppEvent::HealthChanged` is emitted when the gone pictures change (as the storage service does).

## Deviations
- **Files outside the listed perimeter:** `crates/core/src/ports/store.rs` and `crates/infra/src/sqlite/store.rs`
  (assumption 1). TS fixtures got `cover_pending: false` (type only) in `c4_visit`, `a19_review_queue`,
  `p07_shell_sessions_ui`, `p08_workspace_ui`, `p66_approve_on_arrival_ui`. `apps/desktop/src/services/items.ts` is in
  the U4 list but needed no change (the type comes from `bindings.ts`).
- **Existing Sessions-page tests:** the plan's "their mock becomes `sessionsPage` with the same rows; no assertion
  changes" is done through a helper, `apps/desktop/src/features/sessions/__tests__/pageFromList.ts`, that answers
  the mocked `sessionsPage` from the test's existing `listSessions` mock (applying the time filter and `only_ids` as
  the backend does). No assertion changed. Converted: `p07`, `a19`, `c2_session_badges`, `w3_writer_triggers`, `p66`
  (listed) and `p18_pending_state`, `p15_publish_ui`, `c4_visit` (not listed; they render the Sessions page too —
  `c4_visit` showed up in the closing gate). `a10`, `a17`, `r11_arranged_tag` render rows directly and needed nothing.
- **Tracker actor:** written as `claude-opus implementer`, not `codex`, since this session ran on Claude Opus.
- No other deviation; the six other `listSessions` callers are untouched.

## Files changed
Production: `crates/core/src/{domain/remote.rs, domain/session.rs, events.rs, mechanisms/remote.rs, ports/store.rs,
services/remote.rs, services/sessions.rs, services/items.rs}`, `crates/infra/src/sqlite/store.rs`,
`crates/integrations/src/{lib.rs, session_delete.rs}`, `apps/desktop/src-tauri/src/{lib.rs, commands/sessions.rs,
commands/items.rs}`, `apps/desktop/src/services/{bindings.ts (generated), events.ts, sessions.ts}`,
`apps/desktop/src/features/sessions/{SessionsPage.tsx, SessionRow.tsx, HoverPreview.tsx, GroupHeader.tsx,
NewSessionDialog.tsx, Pagination.tsx (new), usePictures.ts (new), ResultThumb.tsx (new)}`,
`apps/desktop/src/features/workspace/{SessionsPanel.tsx, PickerDialogs.tsx}`, `apps/desktop/src/lib/format.ts`,
`apps/desktop/src/styles/globals.css`.
Test support: `crates/testkit/src/fake_commerce.rs`, `crates/integrations/src/commerce/shopify/fixtures/h2_search_featured.json`
(new, schema-grounded), `apps/desktop/src/features/sessions/__tests__/pageFromList.ts` (new).
Tests: `crates/testkit/tests/suite/{h2_row_pictures.rs (new), h2_pages_and_search.rs (new), h1_shopify_reads.rs,
main.rs}`, `apps/desktop/src/features/sessions/__tests__/{h2_list_pages_and_pictures.test.tsx (new),
h2_search_pictures.test.tsx (new)}`, and the mock conversions above (8 files) plus the fixture-type files.
Docs: `docs/guide/02_architecture.md`, `docs/guide/04_change_impact.md`, the plan's Tracker line and Review log, this
handoff. No archgraph state; no other role's files.

## Checkpoint SHA
U1 `5e1a0d6` · U2 `a54ad20` · U3 `b9cb01f` · U4 `ba13aed` · close (c4_visit mock, tracker, Review log, handoff): the
commit after these (`CHECKPOINT (not approved): H2 close …`).

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace` | green (suite 1240 passed, desktop 88, all crates 0 failed) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cd apps/desktop && npm run typecheck` | clean |
| `npm test` | first run: 5 failed in `c4_visit.test.tsx` (its Sessions-page mock lacked `sessionsPage`); fixed as above; re-run of that file: 15 passed. Others: 122 files passed. |

Per unit: U1 `h2_row_pictures::` 24 passed, `p03_` 28, `a17_` 8, `mechanisms::remote` and `psm-desktop` green, clippy
clean; U2 `h2_pages_and_search::` 8, desktop 82, typecheck clean; U3 vitest sessions/workspace/shell 191 passed; U4
`h2_pages_and_search::` 10, `h1_shopify_reads::` 22, `p18_item_search::` 1, desktop 88, vitest sessions/workspace 157,
clippy clean.

## Manual try-out
1. Open Sessions. Rows of products marked "Archived in Shopify" first show a grey placeholder and, a moment later,
   their first picture. No row shows a broken "?".
2. With more than 25 sessions, page numbers appear under the list. Go to page 2; type part of a title that is on page
   1: it is found and the list is back on page 1.
3. Hover a row: the large card shows the same picture.
4. Press N and type part of a product's name: every result has a picture (this Mac's or the shop's), or its first
   letter while the picture loads. The same in Assign product.
5. Quit and reopen: the pictures are there at once (they are kept on this Mac).
6. Health shows "1 picture is no longer in the shop: <SKU>" only if a product's first picture was deleted in Shopify;
   Refresh from Shopify on that product clears it.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
