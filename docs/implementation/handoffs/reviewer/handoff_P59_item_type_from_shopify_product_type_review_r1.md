---
plan: P59_item_type_from_shopify_product_type
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-09
actor: opus
---
# P59 review r1: CHANGES_REQUESTED

Production is right on every criterion row except one regression. Opening an existing linked item in New Session now depends on a successful Shopify read (F1, B1/H6). Separately, C2(a)'s test asserts the fake's own sorting instead of the adapter's (F2, B2). Both fixes are small and fit one fix cycle.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Gate and evidence

- Tracker: `IMPLEMENTED`. Checkpoint `c69c4824895e80ab07777978a248074194743a29` is in `git log`.
- `git show --stat c69c482`: 41 paths, **identical** to the handoff's declared list (machine diff, empty). Deviations (migration 009, `item_type_base`, mock updates) are declared.
- Tree: HEAD `c87c0df` = checkpoint + the review prompt only; `git status --porcelain` clean before and after this review.
- Stamp (one run, master plan §7.2): `cargo test --workspace` 863 passed / 0 failed · `cargo clippy --workspace --all-targets -- -D warnings` clean · `cargo fmt --all -- --check` clean · `npm run typecheck` clean · `npm test` 59 files / 421 tests passed. **Not B4.**

## Verified correct

- **C1(a)**: `productType` is read on all three adapter paths: VARIANT_SEARCH (`ProductRef`, `rename_all = camelCase`), TITLE_SEARCH (json `productType`) and PRODUCT_READ (`ProductNode`, explicit rename). `search_shop` and `adopt` carry it into `item_type`. The test drives the real adapter through `FakeTransport` and the service through the fake shop.
- **C1(b)**: `ItemsService::refresh` fills an empty linked type and follows later shop changes. A local edit survives a refresh because the base differs from the local value. The test covers all three transitions. The form calls refresh on mount for linked items and ignores errors there.
- **C3(a)**: create sends `productType` only when the type is non-empty. Update goes through `productUpdate` inside the confirmed run, followed by a read-back check. The type is frozen at confirmation (`publish_items.frozen_item_type`), so a retry does not write twice (H4). Tests cover the adapter inputs (create, update, empty omitted) and the service: a post-confirm edit is not sent, the next publish sends `Chairs`, and an empty type stays unset. H1 holds: `update_item_type` is called only from the publish job.
- **C4(a)**: `resolve_preset` returns the type's own preset, else the `''` preset, else none. It is used by `prefill_defaults`, `presets_get` (relabelled, so saving creates the type's own preset) and form ordering. The test covers the fallback, the type's own save, a second type keeping the default, and a type change keeping a human value. Core mechanism: unit-level is allowed, and this test is at service level anyway.
- **C2(c)**: free text appears when the integration is absent. A failed fetch shows the exact sentence, and the `no_integration` code suppresses it. `itemCreate` receives the typed text. Both branches are tested.
- **C2(b)**: Create is gated on a listed type and `itemCreate` receives the chosen type. The listing clause is not asserted; see note.
- **Adapter type list (reviewer probe, scratch test, removed):** with two `productTypes` pages (`Lamps, Chairs` / `"", "  ", Lamps, Benches`), `ShopifyAdapter::item_types` returns `["Benches","Chairs","Lamps"]` and sends `after: "c1"` on page 2. The production behaviour is correct.
- **Hard constraints:** H1 holds (the only writes are inside the publish job). H4 holds (frozen type, read-back, base updated after success). H6 is broken on one path, F1. H7 holds: `productType` appears only in `crates/integrations/.../shopify/`, and the base is a link-table column. H8 holds (the three new commands are thin). H2, H3 and H5 are not touched.
- **Assumptions:** zero shop types → free text; a local edit wins until the confirmed publish; a type change replaces only default-provenance values; the header commits a listed option immediately and free text on blur. **None contradicts the book.** They go into the narrative at approval.

## Findings

### F1: B1 (H6), route: production. A failed shop refresh blocks starting a session on a linked item
- **What is wrong:** `NewSessionDialog` `openAction` now runs `item.origin === 'linked' ? await refreshItem(item.id) : item`, and a rejection aborts the open (error shown, no session). `ItemsService::refresh` returns `NoIntegration` when no commerce integration is configured, and a provider error when the shop is unreachable or the token is invalid. Before P59 this path opened the local item directly.
- **Reproduced:** in a scratch dialog test (removed), a local linked item `LAMP-1` with `refreshItem` rejecting `{code:'no_integration'}` led to `refreshItem(7)` being called and `createSession` **never** called. Control: the same test with `refreshItem` resolving opens the session.
- **Authority:** 01 H6 ("the core photography workflow runs with no commerce integration configured"). Plan Goal 1 and Delegated choices: refresh reuses existing read paths, which here means best effort.
- **Correction:** a failed refresh must not block opening. Fall back to the local item (as `FormDialog` already does). Add the rejecting-refresh case to the P59 UI test (session opens).
- **Files the fix may touch:** `apps/desktop/src/features/sessions/NewSessionDialog.tsx`, `apps/desktop/src/features/sessions/__tests__/p59_item_type_picker.test.tsx`. Optionally `crates/core/src/services/items.rs`, so `refresh` returns the local item when no integration is configured.

### F2: B2, route: production (the implementer owns tests in this project). C2(a)'s test asserts the fixture
- **What is wrong:** `p59_tests::c2a_item_types_list_returns_sorted_distinct_shop_types` (`apps/desktop/src-tauri/src/lib.rs`) goes command → service → `FakeCommerce::item_types`. The fake itself sorts, dedupes and drops `""` (`crates/testkit/src/fake_commerce.rs`). The production code that must do this, `ShopifyAdapter::item_types`, is never exercised, and the command and service add no filtering. The row's input (`Lamps, Chairs, "", Lamps`) exists to test the filtering, so the assertion matches the fake's own logic and cannot fail on a wrong production outcome.
- **Authority:** plan C2(a) row and §The contract ("adapter: the shop's distinct product types … empty strings dropped"); flow §5 B2.
- **Correction:** make the row's test fail on a wrong production outcome. Either drive `ShopifyAdapter::item_types` through `FakeTransport` with `productTypes` nodes `Lamps, Chairs, "", Lamps` (as C1(a) and C3(a) already do), or move sort/dedupe/blank-drop into `ItemsService::item_types` and have the fake return the raw list.
- **Files the fix may touch:** `crates/testkit/tests/p59_item_type_from_shopify_product_type.rs`, `apps/desktop/src-tauri/src/lib.rs` (`p59_tests`), `crates/testkit/src/fake_commerce.rs`, optionally `crates/core/src/services/items.rs`.
- **Triage hint for the re-review:** the production behaviour is already verified correct (probe above), so this finding is about the test only.

## Notes written to backlog

1. C2(b): no assertion reads the offered options (`<datalist>`).
2. A NULL `item_type_base` lets the first refresh overwrite a local edit (refresh uses `is_none_or`; publish treats NULL as untouched only when the local type is empty).
3. Items created in the app and then published (origin `local`) never refresh on form open.
4. The form header commits free text while the type list is still loading (`types === null`).
5. The picker is a native `<datalist>`, not the form's searchable select; check it on the work Mac.

## Lessons for the plans

- When a row's input exists to test filtering or ordering that the contract assigns to the adapter, the plan should name the adapter (through the fake transport) as the boundary. "Fake shop" let the filtering live in the fake.
- Any new call added to an "open item" path should state in the plan that it is best effort (H6).

## Write perimeter (this session)

- `docs/implementation/master_plan.md`: P59 tracker row only (→ `CHANGES_REQUESTED`).
- `docs/implementation/plans/P59_item_type_from_shopify_product_type.md`: Review log entry appended.
- `docs/implementation/backlog.md`: five P59 notes appended.
- `docs/implementation/handoffs/reviewer/handoff_P59_item_type_from_shopify_product_type_review_r1.md`: this file.
- **Scratch probe files (created, run, deleted; `git status` clean afterwards):** `apps/desktop/src/features/sessions/__tests__/zz_reviewer_probe_p59.test.tsx`, `crates/testkit/tests/zz_reviewer_probe_p59.rs`. Copies remain in `/tmp`, outside the repo. No production code touched. No database or settings side effects (tests use temp dirs). Build artefacts under `target/` only.
- No commits made.
