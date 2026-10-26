---
plan: P12_shopify_adapter
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-26
actor: claude-opus-5.5 (pipeline-reviewer)
---
# P12 Shopify adapter — review r1

**Verdict: APPROVED_WITH_NOTES.** All 68 rows hold at their boundaries. The perimeter equals the declared files.
Suite, lint and typecheck are green. HEAD contains no credential. There are no blocking findings (B1–B4) and 8
notes in the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- The tracker showed `P12_shopify_adapter IMPLEMENTED`, and checkpoint `bd4218f` is in `git log`. Passed.

## Verified correct
- **Perimeter.** `git show --stat bd4218f` lists exactly the handoff's declared files: 31 paths plus 70 fixture
  JSONs. The one path outside the plan's list, `apps/desktop/src-tauri/src/commands/items.rs` (five match arms), is
  disclosed and required by the exhaustive `Error → CommandError` match. The working tree was clean at start.
- **Stamp (own run, tree `bd4218f` plus docs commits).** `cargo test --workspace` 309 passed, 0 failed (the P12
  file 68). `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`,
  `npm run typecheck` and `npm test` (78) all exit 0.
- **Rows.** For each row I read the production path, then the test.
  - C1(a)–(d): `item_search` filters the returned variants by NFC byte equality. Shopify's search is
    case-insensitive (SV-6), so C1(d)'s other-case capture would return an item if the filter were dropped. It
    also dedupes by product id.
  - C2(a): media come back in gallery order, with 1-based positions, `Ready` and the exact filenames.
  - C3(x): only the differing field is written, and a `None` barcode is never sent. The return value is the
    mutation's read-back: if the adapter returned its inputs, the barcode would be `None` and the test would fail.
  - C3(a): the recorded `ProductSet` variables equal the SV-7 capture, including `status: DRAFT`, SKU and
    `barcodes`.
  - C3(b): `ConfirmedRun` has private fields and one constructor. The tree has no other `ConfirmedRun {…}`
    literal (only the probe example and `TestApp` call `from_persisted`). `Store::confirmed_run` flattens a null
    `confirmed_at` to `None`.
  - C4(a)–(w): `to_core` is the MC-13 table. `writable` = type-writable ∧ `access.admin == PUBLIC_READ_WRITE`.
    `media_list` applies only when the key equals `condition_photos_key` and the type is `list.file_reference`.
    `condition` group rules, unit codes and rating bounds were checked, and no definition has group `product`.
    C4(t) keys were checked against the fixture types: file, metaobject, product, list-metaobject and
    list-product references. C4(u) is a single declared synthetic edit, which the test asserts by counting it.
  - C5(a)–(p): canonical value plus round trip, and byte identity exactly for the types the row lists (choice,
    text, multi_choice, tags, textarea, integer, boolean, date, color, url).
  - C5(q): 25 + 5, and no token or secret in any query or variables.
  - C5(r): the in-batch index from `field[1]` maps to global keys 25 and 27, and the adapter stops after the
    failing batch.
  - C6(a)–(h): as listed in the implementer table. C6(d) checks 3 polls, the fixture's FAILED message, and a
    Timeout after exactly 5 polls. C6(h) checks that `FILE_DOES_NOT_EXIST` is treated as deleted, and the request
    shape.
  - C7(a)(b), C8(a)(b).
  - C9(a)–(g): refresh happens when `now ≥ obtained + lifetime − 60 s`, so C9(b) at −59 s discriminates. A 401
    gets one refetch and one retry, and a second 401 clears the token and returns `CredentialsRejected`. Grant
    refusal codes map to the three 09A messages, verbatim against 09A MC-23. The secret travels only in
    `post_form`. `ShopifyConfig`, `IntegrationSettings` and `ClientCredentialsAuth` `Debug` redact the secret.
  - `FakeTransport` panics on an unmatched request, so an extra or renamed mutation cannot pass silently.
  - No test fails the over-engineering test. None asserts call counts of internals or mocks an internal
    collaborator; counts are of recorded requests at the transport boundary, as the rows specify.
- **Hard constraints.**
  - H1: every mutating port method takes `&ConfirmedRun`. The non-mutating ones (search, get, schema, read,
    polls, list) do not need it.
  - H4: a mutation is retried only on 401/429/`THROTTLED`, where Shopify did not execute it, and never after a 5xx
    or a timeout (MC-11).
  - H6: nothing is wired into `App`.
  - H7: the `publish.rs` types are platform-neutral (see the note on the error strings).
  - H8: business logic has no Tauri, and `items.rs` is only the error mapping.
  - H2, H3, H5: not touched.
  - Secrets and no-live-services: the suite is network-free, and the probe is an example, not a test. With the
    owner's env loaded, `git grep` of HEAD for the shop domain, shop handle, client id and secret found 0 files
    each (counted, values never printed). Fixtures hold no `shpat_`-style tokens except the probe's fake
    wrong-token literal.
- **Evidence doc.** SV-0…SV-7 each have request, response and conclusion. The SV-6 lag check was found at 1.0,
  5.7 and 30.6 s. SV-5 is atomic per call. The access scopes granted cover L20. The cleanup ledger lists one
  undeletable leftover, which is disclosed.
- **Assumptions A1–A21.** None contradicts the book.
  - A3 removes media from the gallery with `fileUpdate referencesToRemove`, which satisfies M19 (the gallery
    equals D) and leaves the file in Files, as MC-14 already does for condition files.
  - A11 reflects that MC-14's local value is exclusions.
  - The rest are runtime or literal choices inside the delegated set. They are told to the owner in the narrative.

## Findings
None blocking (no B1–B4).

## Notes written to backlog
1. P12 → P14: `attributes_read` skips the condition-photos key. P14's MC-13/M21 read-back of the damage report
   needs a read of that metafield's file ids.
2. P12 → P13: the new media id is the "last gallery media" of the `productUpdate(media:)` reply (A4). Adoption
   should confirm the filename stem before recording `attached`.
3. P12 → P13: `integration_settings()` defaults have no row or test. P13's wiring row should observe them.
4. P12 → P11, P14: only units and the rating bounds reach core `validations`. Shopify's other limits surface only
   at publish.
5. P12: writability `PUBLIC_READ_WRITE` only. Definitions this app owns would read as read-only (not reachable in
   the MVP).
6. P12 → plan/H7: core `error.rs` carries Shopify-named display strings, as directed by the plan and 09A.
7. P12: test-store leftover definition `…related_products_display`.
8. P12 (process): an extra live smoke on the test store (A21), disclosed and cleaned up.

## Lessons for the plans
- The rows' illustrative literals (`K-1`, `123`), the "(68)" count and `custom.condition_cracks` came from the real
  store's reference, while the probe store was the test store. A live-store plan should name its literals as
  "the probe's captured values" from the start.
- A plan that authorises live writes "once" should say whether a post-implementation smoke of the adapter's own
  documents is allowed.
- A typed settings parse the plan asks for should carry a criterion row (or be named as P13's).

## Write perimeter (this session)
- `docs/implementation/master_plan.md`: the P12 tracker row only (IMPLEMENTED → APPROVED).
- `docs/implementation/plans/P12_shopify_adapter.md`: Review log entry.
- `docs/implementation/backlog.md`: 8 rows appended.
- `docs/implementation/narratives/P12_shopify_adapter.md`: new.
- This handoff.
- No production or test code was touched. There were no mutation probes (flow §4). Build artifacts are only in
  `target/` (ignored). No remote state was touched, and the test store was not contacted. No commits.
