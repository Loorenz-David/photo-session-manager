---
plan: P12_shopify_adapter
role: implement
round: 1
state: DONE
date: 2026-10-26
actor: claude-opus-5.5 (pipeline-implementer)
---
# P12 Shopify adapter — implement r1 handoff

**Gate:** the intention reads RATIFIED (2026-10-24), P10_attributes is APPROVED, and P12 was NOT_STARTED, so the gate passed.
`docs/intention/evidence/shopify_admin.md` did not exist, so task 0 ran with the owner's `~/.psm_shopify.env`. The
file was loaded with `set -a; . …; set +a`, and no value from it was ever printed, copied or committed.

## What was built
- **The shop has been grounded.** The probe ran every SHOPIFY-VERIFY question (SV-0…SV-7) against the owner's
  **test store** on API 2026-10, wrote each answer to `docs/intention/evidence/shopify_admin.md` with request,
  response and conclusion, and left 70 sanitized request/response fixtures. Everything it created (one draft
  product, four media, two files, six scratch definitions) is on the cleanup ledger and was deleted. One exception
  is listed below. **No stop case was hit:**
  - The create is atomic: one `productSet(synchronous: true)` call sets DRAFT, SKU and barcode.
  - A SKU search finds the product 1 s after create. Search is case-insensitive, so the adapter compares byte for byte.
  - Media and file filenames are readable from `image.url` once READY. Duplicates get `_<uuid>`.
  - `access.admin` is visible on every definition.
  - `metafieldsSet` is atomic per call: limit 25, failing entries named by `field[1]`.
- **The Shopify adapter exists as a library and is not used by the app yet** (P13 wires it). Item search by exact
  SKU, get, create a draft, change SKU/barcode (diff write plus read-back), the schema translated to core field
  types, value read and write (batched), staged upload, PUT upload, media create, poll, reorder, job poll, list,
  delete from the gallery, file create and poll, and title, description and tags writes.
- **Shopify login by client ID and secret** (`ClientCredentialsAuth`). The token is kept in memory and refreshed
  60 s before its 86 399 s lifetime ends. A 401 fetches a new token and retries once. Refusals map to the three
  09A messages.
- **Mutation token:** every mutating method takes `&ConfirmedRun`. Its only constructor,
  `ConfirmedRun::from_persisted`, reads `Store::confirmed_run`.
- **Settings:** `Settings::integration_settings()` gives a typed read of the `integration` block. An absent
  `condition_photos_key` reads as `custom.damage_report`, an absent `api_version` as `2026-10`, and the secret is redacted in `Debug`.
- **Test harness:** `FakeTransport` (replies keyed by operation and variables digest, with per-key sequences; an
  unmatched request panics; every request is recorded), a fixture loader, and `TestApp::confirmed_run()`.
- **Live adapter smoke** (beyond the probe). The real adapter's own documents were run once against the test store
  and cleaned up. Every method behaved as the fixtures say, including a file created from an IMAGE staged target.

## Tests → criterion rows
All rows are in `crates/testkit/tests/p12_shopify_adapter.rs` and run against `FakeTransport` with no network.
Each assertion has the shape its row specifies. The rows' illustrative literals (`K-1`, `K-2`, `123`) are replaced
by the captured probe values (Assumption A14).

| Row | Test | What it asserts |
|---|---|---|
| C1(a) | `c1a_search_no_match_is_empty` | `[]` for the 0-match capture |
| C1(b) | `c1b_search_one_match` | one `RemoteItem` with the fixture's id, title, SKU, barcode, `thumbnail_url: None`, `media: []` |
| C1(c) | `c1c_search_two_products_returns_both` | both product ids (declared-synthetic second node) |
| C1(d) | `c1d_search_is_byte_exact_after_nfc` | `[]` for the other-case capture (Shopify returned the upper-case SKU) |
| C2(a) | `c2a_item_get_media_in_gallery_order` | 3 × `RemoteMedia{id, filename, Ready, position}` in gallery order, exact filenames |
| C3(x) | `c3x_set_identity_writes_only_the_sku` | exactly one `ItemSetIdentity`; its variables carry the new SKU and no `barcodes`; returns `{SKU-2, Some(4006381333931)}` |
| C3(a) | `c3a_create_draft_with_sku_and_barcode` | returns the id; recorded `ProductSet` variables == the SV-7 capture's; read-back fixture is DRAFT with SKU/barcode equal to the inputs |
| C3(b) | `c3b_confirmed_run_only_from_a_confirmed_row` | null `confirmed_at` → `NotConfirmed`; missing id → `NotConfirmed`; confirmed → `Ok(run_id, confirmed_at)` |
| C4(a) | `c4a_choice` | choice + writable + choices; also the severity-five clause (`custom.condition_psm_probe_cracks`, group condition) and "no group `product`" over the whole return |
| C4(b)–(r) | `c4b_text` … `c4r_url` (17 tests) | named core type and `writable`; (i)(j)(k) `validations.units` = SV-1 unit codes; (n) `reference`, false; (p) `{"min":"1","max":"5"}` |
| C4(s) | `c4s_configured_condition_key_is_media_list` | `custom.damage_report` → `media_list`, writable, group condition |
| C4(t) | `c4t_references_are_read_only` | file, metaobject, product, list-metaobject and list-product refs → `reference`, false |
| C4(u) | `c4u_access_forbidding_writes_makes_read_only` | declared synthetic access edit (`MERCHANT_READ_WRITE`) → integer, `writable = false` |
| C4(v) | `c4v_only_the_configured_file_list_is_media_list` | captured second images list → `reference`, false; the media_list keys are exactly `["custom.damage_report"]` |
| C4(w) | `c4w_missing_condition_key_has_no_media_list` | key `custom.nope` → no `media_list`; `custom.damage_report` `reference`, false; `Ok` |
| C5(a)–(p) | `c5a_choice` … `c5p_url` (16 tests) | `canonicalize(read(s))` == the stated canonical **and** canonical round trip through `to_platform_string`; byte equality `to_platform_string(c) == s` for choice, text, multi_choice, tags, textarea, integer, boolean, date, color, url |
| C5(q) | `c5q_write_batched_at_the_limit_without_credentials` | 30 values → 2 `MetafieldsSet` calls (25 + 5); no query or variables contains the token or the secret |
| C5(r) | `c5r_partial_failure_names_the_failing_entries` | batch 1 succeeds; batch 2 gets the SV-5 two-invalid reply → `AttributeVerification{keys: [entry 25, entry 27]}`; exactly 2 calls |
| C6(a) | `c6a_staged_upload_request` | `filename "k-1-ab12cd34ef56.jpg"`, `image/jpeg`, `IMAGE`, `PUT` |
| C6(b) | `c6b_upload_puts_with_parameters_as_headers` | PUT URL = the fixture URL; headers = the fixture parameters as a name→value map |
| C6(c) | `c6c_create_media_returns_the_new_media_id` | id from `newest` (SV-2); `originalSource` = resource URL |
| C6(d) | `c6d_poll_media_ready_failed_and_timeout` | returns after the 3rd poll; FAILED → `Provider("Image could not be processed")`; always PROCESSING → `Timeout` after 5 polls |
| C6(e) | `c6e_reorder_moves_every_id_zero_based` | one request, moves for all 3 ids, `newPosition` "0","1","2" |
| C6(f) | `c6f_poll_job_until_done` | returns on the 2nd poll |
| C6(g) | `c6g_list_media_keeps_exact_filenames` | original + uniquified filenames with positions 1, 2 |
| C6(h) | `c6h_delete_not_found_counts_as_deleted` | `FILE_DOES_NOT_EXIST` → `Ok(())`; request = `fileUpdate referencesToRemove` |
| C7(a) | `c7a_file_create_then_poll_ready` | returns the `MediaImage` id; `duplicateResolutionMode: APPEND_UUID`; poll PROCESSING → READY (2 polls) |
| C7(b) | `c7b_media_list_write_sends_file_ids` | value `["f1","f2"]` as a string, type `list.file_reference`, `custom`/`damage_report` |
| C8(a) | `c8a_description_html_escapes` | exact HTML |
| C8(b) | `c8b_update_tags_payload` | one `productUpdate` with `tags ["a","b"]` |
| C9(a) | `c9a_token_fetched_once_and_reused` | one form POST to the token URL with exactly the three fields; both calls carry T1 |
| C9(b) | `c9b_token_refreshed_before_expiry` | at lifetime − 59 s: a second token request; the second call carries T2 |
| C9(c) | `c9c_unauthorized_refetches_and_retries_once` | 401→200: 2 token requests, tokens [T1, T2], `Ok`; 401,401: `CredentialsRejected` after exactly 2 GraphQL attempts |
| C9(d) | `c9d_shop_not_permitted` | `ShopNotPermitted` + its 09A message; 0 GraphQL requests |
| C9(e) | `c9e_app_not_installed` | `AppNotInstalled` + message; 0 GraphQL requests |
| C9(f) | `c9f_wrong_secret_is_credentials_rejected` | `CredentialsRejected` + message; 0 GraphQL requests |
| C9(g) | `c9g_secret_and_tokens_never_leave_the_token_request` | over the (a)–(f) and C6(a)–(b) sequences: no GraphQL URL, query or variables, no PUT URL or header, no error `Display`/`Debug` and no `ShopifyConfig` `Debug` contains `SECRET-XYZ`, T1 or T2; T1 and T2 appear as the access-token argument |

Count: C1 4 · C2 1 · C3 3 · C4 23 · C5 18 · C6 8 · C7 2 · C8 2 · C9 7 = **68 tests, 68 rows**. Every test
discharges a row; there are no orphans.

**Core-mechanism unit tests** (plan list only): `type_map::tests::{mc13_to_core_table, mc13_from_platform_shapes,
mc13_to_platform_shapes}` and `adapter::tests::{mc11_upload_filename, mc13_description_html}`.
**Baseline:** I wrote the tests after the adapter code, so no red baseline was captured before the first
production edit (flow §4 has no baseline rule; stated so nobody assumes one).

## Assumptions
- **A1. Fixture store.** The C4 input is the test store's captured definition list (66) plus 6 scratch definitions
  (72), not the real store's 68. The owner made the test store the probe store, and the plan says fixtures are
  the test store's captures. Two consequences:
  - The plan's "(68)" and the `custom.condition_cracks` clause are absorbed. The clause is served by the scratch
    `custom.condition_psm_probe_cracks` (severity five, group condition).
  - The real store's `condition_*` fields and app-owned definitions were never captured. They are read from the
    configured store at schema refresh (09A MC-23).
- **A2. Absorbed literals (SV ids).** `expires_in` 86399 (SV-0); L = 25 and entries named by `field[1]`, with
  `elementIndex` null (SV-5); reorder base 0 and job `{done}` (SV-4); uniquifier `_<uuid v4>` (SV-2/3); unit codes
  `MILLIMETERS…YARDS`, `OUNCES…KILOGRAMS`, `MILLILITERS…IMPERIAL_GALLONS` (SV-1); decimal `12.50` → `12.5`; colour
  case kept; not-found = `FILE_DOES_NOT_EXIST`.
- **A3. Media delete.** `productDeleteMedia` is gone in 2026-10. `product_delete_media` uses
  `fileUpdate(referencesToRemove: [product])`, which removes the media from this product's gallery and leaves the
  file in Files. That is the narrowest effect that makes the gallery equal D. `fileDelete` would also remove the
  file from anything else that references it.
- **A4. Media create.** `productCreateMedia` is gone. `product_create_media` uses `productUpdate(media:)`, and the new
  id is `product.media(first: 1, reverse: true)`, the last media in gallery order (seen on all three creates).
- **A5. Writability.** Writable only when `access.admin = PUBLIC_READ_WRITE`. Shopify's own description of every
  other value excludes other apps.
- **A6. Transport runtime (L13).** reqwest 0.13 with `default-features = false` and features `json` and `rustls`;
  no `blocking`. `ReqwestTransport` owns a private worker thread with a current-thread Tokio runtime, and callers
  block on a std channel. The caller's thread never enters, starts or drops a runtime, so there is no panic in a
  Tauri async command. Requests time out after 30 s. Errors use `without_url()`, so signed URLs never reach messages.
- **A7. Transport signature.** `post_graphql(url, access_token, query, variables)` takes the endpoint URL so the
  transport is shop-agnostic. `HttpReply` gained `retry_after_s` for the delegated 429 policy.
- **A8. Retry policy.** Up to 3 retries for HTTP 429 or `THROTTLED` (any operation; the request was not executed)
  and for 5xx on queries only. Mutations are never retried after a 5xx or a timeout (MC-11). The wait is
  `Retry-After` or `retry_wait_ms` (default 2 s), capped.
- **A9. Polling (L19).** `PollConfig{interval_ms: 1000, attempts: 120}` by default; tests use 0 ms × 5. Exhaustion is `Error::Timeout`.
- **A10. `RemoteMedia`.** `filename: Option<String>`, `None` while Shopify has not exposed it (PROCESSING);
  `position` 1-based; `status` is a core enum `RemoteMediaStatus{Uploaded, Processing, Ready, Failed}`.
- **A11. `attributes_read`.** It returns the raw core-shaped JSON for every product metafield except the configured
  condition key (that value is local exclusions, MC-14). A value core cannot represent is returned as the
  platform's own JSON (a rich-text tree with other nodes or marks) or `{"verbatim": s}`; `canonicalize` rejects
  either, so the service stores it `valid = false` (L5).
- **A12. `attributes_write`.** It stops at the first failing batch and maps keys from `field[1]` (or
  `elementIndex`). An error it cannot map names every key of that batch.
- **A13. Test tokens.** `T1-psm-access-token` and `T2-psm-access-token` instead of bare `T1`/`T2`, so the absence
  check cannot hit fixture text (the fixture date `2026-10-26T10:00:00Z` contains "T1").
- **A14. Row literals.** C1, C3(a) and C3(x) use the captured probe values (`PSM-PROBE-SKU-1/2`, barcode
  `4006381333931`, `PSM-PROBE-NO-SUCH-SKU`) in place of `K-1`/`K-2`/`123`, so every fixture stays a real capture.
- **A15. `staged_uploads_create(run, folder_key, uid, revision, byte_size)`** computes the MC-11/MC-21 filename
  itself and always uses `resource: IMAGE`. The live smoke confirmed `fileCreate` works from an IMAGE target.
- **A16. Groups.** `condition` covers namespace `custom` with key prefix `condition_`, the configured key, and
  `custom.condition_report`. Core `validations` carries only `units` (dimension, weight, volume) and the rating
  bounds; every other Shopify validation stays in `DefinitionLink.platform_meta`
  (`{id, namespace, key, validations, access}`).
- **A17. Thumbnails.** `item_search` takes the thumbnail from `featuredMedia.preview.image.url`; `item_get` from the
  first gallery image.
- **A18. Error `Display`.** The three 09A errors display the photographer messages.
  `attribute_verification` / `not_confirmed` follow the existing `code: detail` style.
- **A19. Settings.** An absent `api_version` reads as `2026-10` (master plan §6.8 default).
- **A20. Tracker actor.** The tracker row says `opus`, not the prompt's `codex`, because Opus ran this session (flow §2).
- **A21. Extra live check.** Beyond the plan's single probe pass, I ran one live smoke of the adapter itself on the
  test store (one product, two media, one file; all deleted). Recorded in the evidence doc.

## Deviations
- `apps/desktop/src-tauri/src/commands/items.rs` (outside the perimeter) gained five arms in the `Error →
  CommandError` match (`attribute_verification` with `data.keys`, `not_confirmed`, `shop_not_permitted`,
  `app_not_installed`, `credentials_rejected`). The match is exhaustive, so the new error variants would not compile without them.
- `crates/integrations/Cargo.toml` gained `tokio` (workspace) and `unicode-normalization` alongside `reqwest`, plus
  the dev-dependency `rusqlite` (the probe's smoke builds a `ConfirmedRun` from a scratch database).
- `master_plan.md` §6.2 was **not** edited with the reqwest features (only my tracker row is mine). The features are
  recorded in A6 for the coordinator to fold in.
- `WebFetch` and `WebSearch` were refused by the session's permissions ("Claude requested permissions to use
  WebFetch, but you haven't granted it yet"). I did not need them: Shopify's own enum descriptions came from GraphQL
  introspection, and the refusal codes come from 09A MC-23. Hence not BLOCKED.
- **Test-store leftover:** the standard definition `shopify--discovery--product_recommendation.related_products_display`
  ("Related products settings") was auto-created by the SV-1 foreign-namespace write. `metafieldDefinitionDelete`
  answers `ACCESS_DENIED` (owned by Shopify Search & Discovery). It has no values. The equally auto-created
  `reviews.rating` was deleted.
- `docs/implementation/phases.txt` shows as modified in the working tree. That is not my change, and it is not in my commit.

**For the coordinator to fold into P13/P14.**
- Adopt media only after READY: media that is still PROCESSING exposes no filename.
- Do not use `files(query: "filename:…")` for recovery: its index lagged after files were READY.
- Shopify reads `&quot;` back as `"` in `descriptionHtml`, so P14's read-back must normalise.
- Re-check definition `access` on the real store when switching stores.
- The `items.rs` match will need arms for any further error variants.

## Files changed
**Production and tests (commit `bd4218f`).**
- New:
  - core: `crates/core/src/domain/publish.rs`, `crates/core/src/ports/commerce.rs`.
  - integrations: `crates/integrations/src/commerce/{mod.rs, shopify/{mod.rs, adapter.rs, auth.rs, errors.rs, graphql.rs,
    transport.rs, type_map.rs, types.rs, fixtures/*.json (70)}}`, `crates/integrations/examples/shopify_probe.rs`.
  - testkit: `crates/testkit/src/{fake_transport.rs, fixtures/shopify.rs}`, `crates/testkit/tests/p12_shopify_adapter.rs`.
  - docs: `docs/intention/evidence/shopify_admin.md`.
- Changed:
  - core: `crates/core/src/{error.rs, ports/mod.rs, ports/store.rs, domain/mod.rs}`.
  - infra: `crates/infra/src/{sqlite/store.rs, settings_file.rs}`.
  - integrations: `crates/integrations/{Cargo.toml, src/lib.rs}`, `Cargo.lock`.
  - testkit: `crates/testkit/src/{lib.rs, fixtures/mod.rs, test_app.rs}`.
  - desktop (deviation): `apps/desktop/src-tauri/src/commands/items.rs`.
  - docs: `docs/implementation/master_plan.md` (P12 tracker row only), `docs/implementation/plans/P12_shopify_adapter.md`
    (Review log entry).

**Not committed, outside the repo or ignored.** Raw redacted captures and the ledger are in `target/shopify_probe/`
(gitignored); scratch queries and the curation script are in `/tmp`.

**Tool-recorded state.** No architecture graph is used in this repo (master plan §8).
**Mutation-probe files:** none (no mutation regime, flow §4).
**Remote state touched:** the owner's test store. Everything was created and deleted per the ledger, except the one
leftover definition above.

## Checkpoint SHA
`bd4218f` — `CHECKPOINT (not approved): P12 Shopify adapter — grounding probe (SV-0…7), GraphQL adapter, type map, token auth, fixtures, 68 behaviour tests`.
The working tree is clean except `docs/implementation/phases.txt`, which is not mine.

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace` | 309 passed, 0 failed: 236 before this phase + 68 P12 rows + 5 unit tests |
| `cargo test -p psm-testkit --test p12_shopify_adapter` | 68 passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 (fmt applied) |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | 4 files, 78 passed |
| `git grep` of HEAD for the shop domain, the shop handle, the client ID and the secret | 0 files each |

There are no pre-existing failures.

## Manual try-out
1. Read `docs/intention/evidence/shopify_admin.md`: one section per SV-0…SV-7 with request, response and conclusion,
   a section on the adapter's live smoke, and the cleanup ledger.
2. In the test store's Shopify admin:
   - Search products for `PSM-PROBE`: nothing is found.
   - Settings → Custom data → Products has no `psm_probe_*` definitions.
   - You will see one definition the probe caused but could not delete: "Related products settings"
     (`shopify--discovery--product_recommendation.related_products_display`). Delete it there if you like; nothing uses it.
3. Run `cargo test -p psm-testkit --test p12_shopify_adapter`: 68 pass with no network.
4. Optional: re-run the live smoke with
   `set -a; . ~/.psm_shopify.env; set +a; cargo run -p psm-integrations --example shopify_probe -- adapter_smoke`.
   It creates and then deletes one draft product, two photos and one file on the test store.
5. The app does not use the shop yet. Health, New-session search and the form's shop fields come with the next phase.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
