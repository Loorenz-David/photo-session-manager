---
plan: H1_points_and_remote_record
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-11-23
actor: Claude Opus 5.5 (pipeline-projectionist)
---
# Projection of H1_points_and_remote_record (round 0)

The plan can be built, and almost every check it describes can be written today with one clear expected result. Six checks need a short rewrite before the build session starts. Three of them could never fail as written. Two cannot be set up with the test tools the plan provides. One has an expected result that cannot happen. There are also three places where the plan contradicts the project's own rules: a log line that puts "shopify" in the core code, an error the plan uses but never adds, and test helpers that would sit unused until a later milestone. None of this needs you: every item is a paragraph-sized fix the orchestrator can make in the plan files, and the build session can start after that.

## ⚠ OWNER DECISIONS REQUIRED (0)

Nothing needs the owner: every ledger row is a plan amendment or a delegation the coordinator can grant.

## Decision ledger

| # | Decision point (where the artifacts stop determining the next step) | Classification | Proposed routing |
|---|---|---|---|
| L1 | **H1·C1(c) cannot fail.** Its input is the body of `sv0_graphql_401.json` (status 401). `ClientCredentialsAuth::with_token` (`auth.rs:112-125`) handles a 401 itself: it fetches a new token and calls again, and on a second 401 it returns `Err(CredentialsRejected)`. So `call_inner` never receives that reply, and the budget-read code never runs on it. The row stays green even if an implementation overwrote the reading with `None` when `extensions` is missing. That is exactly the defect the row (MC-H5.1, "an answer without it leaves the stored value as it was") is meant to catch. The setup also needs a second queued 401 reply, or the `FakeTransport` panics on an unmatched request. | plan gap | Amend H1·C1(c): input = "after (b), a query answered `200` with a body that has `data` and no `extensions` key (built by the test; `extensions` is optional in a GraphQL reply)"; outcome unchanged: `Some {1999.0, 2000.0}`. |
| L2 | **No row covers the hazard the plan calls most important**: reading the budget from a throttled or failed reply, before `call_inner`'s error returns (`adapter.rs:471-490`; Notes, 2nd hazard). Today only the Review-mode reading check covers it. No captured fixture in the repo has an `errors` body together with `extensions.cost`; the only error fixture, `sv0_graphql_401`, has no cost block. | free choice (coordinator's) | Either add a row H1·C1(f): a query answered (via `on_graphql_always`, since throttled replies are retried `poll.retries` times) with a `THROTTLED` errors body carrying `extensions.cost.throttleStatus {currentlyAvailable: 12, maximumAvailable: 2000.0, restoreRate: 0.0}` → `Err(Provider)` and `budget() = Some {12.0, 2000.0}`. That shape must first be grounded per rule 17 (shopify.dev rate-limits page; its `source` says schema-grounded). Or keep it a reading check only and record that choice in the plan. |
| L3 | **`Error::NotRemote` is used but never added.** H1·C10(c) expects `Err(NotRemote)`. No task creates the variant, and U2's Files omit `crates/core/src/error.rs` and `apps/desktop/src-tauri/src/commands/items.rs`. `impl From<Error> for CommandError` (`commands/items.rs:38`) is an exhaustive `match` with no `_` arm, so adding the variant without its code `not_remote` breaks the `psm-desktop` build (B4). | plan gap | Amend U2 task 4: "`Error::NotRemote` (`#[error(...)]`, core wording says *the shop*) and its command code `not_remote` (master plan §4 Errors)". Add both files to U2's Files. |
| L4 | **The `shopify.budget` log line puts a Shopify literal in core.** U1 task 5 says the log is "the caller's (U2)", and the callers are `RemoteItems::run_pending` and `StorageService::retention_scan`, both in `crates/core`. That contradicts H7, the guide's H7 row ("`"shopify"` literals … do not add more", `03_rules_and_principles.md:14`), master plan §4 ("No name of Shopify … in `crates/core`") and this plan's own Review-mode check. Whichever way the implementer goes, it is a silent deviation, and the reviewer could treat it as B3. | plan gap (home: master plan §4 Log lines) | Rename the core log line, e.g. `remote.budget_wait available= maximum=`, in master plan §4 Log lines, H1 U1 task 5, U2 task 4 and the Manual try-out note. |
| L5 | **H1·C9(f) cannot be set up with the fakes.** `FakeCommerce::media_readback` always gives every gallery media `url: Some(fake_image_url(..))` (`fake_commerce.rs:716-726`). Task 9 adds no way to make a gallery picture "still processing". | plan gap | Amend task 9 (or U2's fake edits, `fake_commerce.rs` is in both perimeters): e.g. "`media_readback` answers `url: None` for a gallery media whose `status` is `RemoteMediaStatus::Processing`" (no new setter), with H1·C9(f) as its caller. Also say that `set_media_bytes(id, n)` applies to gallery media and condition files alike. H1·C12(a) needs condition sizes, and `media_readback` builds condition entries from `values` + `files`. |
| L6 | **H1·C10(e)'s expected result cannot happen.** "Archived item" is defined as published through `FakeCommerce` and then cleared. The publish records mutations, so `mutation_ops()` is never empty after setup. H1·C12(b) has the same problem in a milder form: "no `media_readback` read was made" (the publish helper's reads already exist). | plan gap | Amend H1·C10(e) to: "`mutation_ops()` is unchanged by the `ensure_record` / `refresh` calls (compared with its value after the setup)". Amend H1·C12(b) to: "no `media_readback` read was recorded by the scan". |
| L7 | **H1·C9(d) (two threads, one read) cannot fail without a forced overlap.** If the first thread finishes before the second starts, the second re-reads a `checked` input even without the lock, and the row passes. | plan gap (fixture cell) | Amend H1·C9(d) input: "`FakeCommerce::set_hook` sleeps 100 ms on `"media_readback"`, so both calls are inside before either writes". Without the lock this produces two reads, so the row can fail. |
| L8 | **Refresh's cover pruning needs `cover_name`, which belongs to H2** (H2 task 1, row H2·C5(d)). MC-H3.3 says delete `remote/<item_id>/cover-*` "not the new cover's". H1 cannot compute "the new cover's" name without that function. | plan gap | Amend U2 task 4: "in H1, `refresh` deletes every `remote/<item_id>/cover-*`; H2 narrows it to keep `cover_name(new cover)` when it adds the function". Add that sentence to H2 task 1. H1·C10(d) is unaffected. |
| L9 | **The test-fake additions would be unused in H1.** Task 9 says "each addition has a caller in U2's tests". It does not: `media_urls`/`url_reads`, `fetch_original`/`original_fetches`, `set_image`, `image_faults`/`fail_image`, and `set_fetch_delay`/`max_fetches_in_flight` have no caller in any H1 row. Their first callers are in H2, H3 and H4. That breaks charter rule 4 (no dead scaffolding), and the plan's own claim is false. | plan gap (home: master plan §4 Tests "FakeCommerce grows (H1)") | Move each addition to the milestone whose rows first call it (keep `set_budget` and `set_media_bytes` in H1, plus L5's knob). Or keep them in H1 and record the exception with its reason in task 9. |
| L10 | **The full `RemoteItems::new` constructor causes unused-field warnings.** Arguments such as `editing`, `capture_root`, `history`, `search_cache_limit` (and `health` until H2) become private fields that H1 never reads. Rust then warns `dead_code`, and `cargo clippy … -D warnings` turns that warning into an error (B4). | plan gap | Amend U2 task 4: say how. For example: "fields first used by a later milestone carry `#[allow(dead_code)]` with a comment naming it", or "take only what H1 uses; later milestones extend the constructor". |
| L11 | **Retention's `shop_only` source.** Task 6 says to fill it "from `RemoteItems::state`". But `StorageService` holds no `RemoteItems` (constructor `storage.rs:308`, built separately at `lib.rs:1056`). | free choice | Delegate explicitly. Recommended: `remote_state(store.remote_input(session_id)?) == Remote(ShopOnly)` inside `StorageService`, needing no new wiring (retention runs only with an integration, so the H6 "no integration → Local" rule holds). The alternative is `with_remote(Arc<RemoteItems>)`. |
| L12 | **p62's test helpers are private.** `shop_app`, `item`, `published`, `clear_all` and `product_of` are private `fn`s in `p62_retention_after_publish.rs`, which is not in U2's Files. The row definitions of U2 depend on them. | free choice | Delegate: make them `pub(crate)` and add the p62 file to U2's Files (recommended: one copy), or copy them into `h1_remote_record.rs`. |
| L13 | **The P62 archived-cache file name** (`<role>-<NN>-<12 hex>.jpg`) exists only as the private `ArchiveService::cached_path` (`retention.rs:180`). `refresh`'s pruning needs it, and so does H1·C10(d)'s test. | free choice | Delegate: expose one name function in `services/retention.rs` and reuse it, rather than writing it a second time. |
| L14 | **H1·C9(e)/(f) error variants are not pinned** ("`Err`"). Under flow §5 B2, an `is_err()` assertion accepts several outcomes. | free choice | Pin (e) to the read's own error, which is `Err(Provider)` from the fake. For (f), delegate the variant explicitly (e.g. `Provider("… still processing")`) and assert it. |
| L15 | **Where `fileSize` is parsed.** `MediaNode.original_source` is a `UrlHolder {url}` (`types.rs:83-114`), which other queries share. | free choice | Delegate: add `file_size: Option<i64>` (serde `fileSize`) to `UrlHolder`, or give it a separate type. Either is fine. |
| L16 | **H1·C8(b)'s trace `charter rule 7`** is not in the trace vocabulary of master plan §3. | plan gap (minor) | Retrace to the 10F retention record kept (`M41`), or record `charter rule 7` as an allowed trace in §3. |

## Reality-check findings

- **Paths.** Every existing path in both units' Files and Read-first lists resolves. `docs/implementation/backlog_archive/` does not exist yet; the reviewer creates it. Migrations `001…019` exist, so `020` is free (`store.rs:67-90`). Suite stems `a17_delete_session`, `p12_shopify_adapter`, `p27_retire_shape_on_arrival`, `p59_…` and `p62_…` are all in `main.rs`.
- **Cited lines** match the plan's claims: `call`/`call_inner` `adapter.rs:432-492`, `media_readback` :1681, `fetch_image` :1696 (only `get_bytes` caller, :1704), `media_readback_of` :1970-2010, `PollConfig` :158; `graphql.rs:32/34`; `transport.rs` trait :37-57, `MAX_DOWNLOAD_BYTES` :60, inherent `ReqwestTransport::download` :193, `get_bytes` :226; `ports/commerce.rs:213/216`; `MediaImage` `publish.rs:61`; `ItemRetention`/`ArchivedImage` `domain/storage.rs:45-62`; `retention_scan` :702, `clear_item` :827; store `item_retention` :3210, `mark_item_cleared` :3236, `archived_images` :3255; rewind test `p27…:346-372`; `storage_background` `lib.rs:1163`.
- **Other implementations of the changed traits.** `research-tool/src/accept.rs:143` (`StandInShop`) implements `CommerceAdapter`. The new port methods all have defaults, so it needs no edit. `HttpTransport` is implemented only by `ReqwestTransport` and `FakeTransport`. `MediaImage` is built at exactly the four sites the plan names (adapter ×2, fake ×2).
- **Name clash (harmless).** After task 3, `ReqwestTransport` has an inherent `download(String)` and a trait `download(&str)`. Inherent methods take priority, so this compiles, but it reads badly. Implementer's choice; renaming the inherent one is the obvious tidy-up.
- **The existing retention check now depends on unverified field names.** Task 7 adds `originalSource { fileSize }` and `GenericFile.originalFileSize` to the existing captured `MEDIA_READBACK`. If either name were wrong, Shopify would reject the whole query, and every 10F clearing verification would report "Shopify could not be read". `originalSource.fileSize` is backed by our own evidence (`shopify_admin.md:228`, SV-3). `GenericFile.originalFileSize` and the "`null` per unknown id" behaviour of `nodes(ids:)` are schema-grounded only; this session's web access was refused, so I could not re-read shopify.dev. **Route to the coordinator:** the live check after H4 must include a clearing verification (the readback), not only covers. No build carrying H1 should be installed before that check (the track already defers installing until the join).

## Criteria decidability (per row: test writable now with one exact outcome? fixture producible?)

| Rows | Writable now | Fixture producible | Note |
|---|---|---|---|
| C1(a), (b), (d), (e) | yes | yes: captured `sv4_job_poll_done` with `restoreRate` edited; integer form read with `as_f64` | `restoreRate 0` makes them independent of time |
| C1(c) | **no** (cannot fail) | the reply never reaches `call_inner` | L1 |
| C2(a–d), C3(a–d) | yes | pure | — |
| C4(a–e) | yes | `replay` keys on op `MediaUrls` + variables `{ids}`; (b), (d), (e) are replies built by the test | schema-grounded shapes, see reality checks |
| C5(a–d) | yes | `fileSize` grounded in SV-3; `originalFileSize` schema-grounded only | fixture request variables `{id, namespace, key}`, or `replay_any` |
| C6(a–f) | yes | `Download` is our own type (task 3) | — |
| C7(a–g), C11(a–c) | yes | pure | — |
| C8(a) | yes | store call | — |
| C8(b) | yes | `SqliteStore::migrations().take(19)` (as `p30…:594`) or the p27 rewind style; insert an `items` row first (FK) | trace L16 |
| C9(a–c), (e), (g) | yes | adopt + push `RemoteMedia` / `values` / `files` into `FakeCommerce.state` | (e) error pin L14 |
| C9(d) | **no** (cannot fail) | needs a forced overlap | L7 |
| C9(f) | **no** | the fake cannot produce a URL-less gallery picture | L5, L14 |
| C10(a), (b), (d) | yes | (d): the test computes the P62 name with `sha2` | L8, L13 |
| C10(c) | yes, once L3 is done | — | L3 |
| C10(e) | **no** (expected result impossible) | — | L6 |
| C12(a) | yes, once L5 says condition sizes come from `set_media_bytes` | — | L5 |
| C12(b) | yes with relative wording | `ITEMS_PER_SCAN = 5`, no `verify_after` set on a wait | L6 |
| C12(c) | yes | `clear_all`'s ticks cover `BATCH_PAUSE` (2 s) | — |

## Trace verification (both directions)

Rows to entries: every trace cell resolves to an MC-H item, M-H1, a 10H/10F section, M41 or H1 that says what the row asserts. The one exception is H1·C8(b) (L16). Entries to rows: MC-H5.1–5.5 (C1, C2, C3, C9(g), C10(b), C12(b,c), C4(b,c), C9(c)); MC-H3.1 (C7); MC-H3.2 (C8(a), C9); MC-H3.3 (C10); MC-H3.8 / MC-H4.8 (C11); MC-H1.4 adapter half (C4, C6); MC-H4.1 sizes (C5(a), C12(a)); M-H1 (C10(e)). The URL-refresh consumer of MC-H5.3 is H2's, not H1's. No entry this phase claims is left unserved.

## Note for H4's projection (not an H1 item)

Retention's exclusion is keyed on `Remote(ShopOnly)`. An adopted, photo-less item that is still `Unknown` (never read) is not excluded. Today such a session can never be Ready (`status.rs:33`), so this matters only if H4's details-only publish can complete for an `Unknown` item. MC-H3.6 says an `Unknown` item "is confirmed as today", so H4's projection should confirm that path cannot reach "published and clean" with zero photos.

## Write perimeter of this session

Documents: this file only. Code: none. Tool-recorded state: none (no archgraph in this repo). Commits: none.
