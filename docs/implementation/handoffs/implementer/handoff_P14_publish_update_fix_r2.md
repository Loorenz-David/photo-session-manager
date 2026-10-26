---
plan: P14_publish_update
role: fix
round: 2
state: OWNER_DECISIONS_PENDING
date: 2026-10-26
actor: codex
---
# P14 fix r2 handoff

F2 and F3 are corrected and the full verification stamp is green. F1 remains unresolved because the
review correction requires the owner's tag-model choice. P14 remains `CHANGES_REQUESTED` until that
correction is made.

## ⚠ OWNER DECISIONS REQUIRED (1)

**Card 1: what should the app compare against when it merges product tags?**

- **Question:** Should the app remember the tags it last published, and merge against that (option A)?
- **Story:** You type the tag "oak" in the app; the shop already has "vintage", added in the admin. The first publish correctly sends "vintage, oak". You publish again a week later without touching tags, and "vintage" vanishes from the listing. Separately, if you delete a tag in the app, the shop keeps it forever.
- **Branches:** **A**: the app remembers its own last-published tags. Admin tags are never touched, and removals made in the app reach the shop. · **B**: the app's tag field mirrors the shop. Each publish copies the shop's tags into the app, so admin tags appear in your form and deleting one there removes it from the shop.
- **Recommendation:** A. It is the smallest change, keeps your fields exactly as you typed them, and never removes a tag you did not remove.
- **On silence:** The gate holds. F1 is not fixed and P14 stays unapproved.
- **Trace:** 07A MC-13 (Product tags), P14 C2(c), M20.

## Finding → what changed → files

### F1 (B1) — pending owner decision

The correction is conditional on the owner selecting the merge base. I did not change the production
tag behavior or C2(c) before that choice. The tracker remains `CHANGES_REQUESTED`.

**Files:** No F1 production or criterion change. The decision is recorded above.

### F2 (B1) — corrected

Added a dedicated condition-metafield read-back port and Shopify GraphQL query. Publish verification
uses it for `media_list` writes, while ordinary `attributes_read` keeps omitting the condition key.
The fake now omits that key from ordinary reads and returns it through the dedicated read-back, so
C4(a)'s existing boundary test exercises the adapter-shaped gap. Added a focused Shopify value parser
unit test.

**Files:** `crates/core/src/ports/commerce.rs`, `crates/core/src/services/publish.rs`,
`crates/integrations/src/commerce/shopify/adapter.rs`,
`crates/integrations/src/commerce/shopify/graphql.rs`, `crates/testkit/src/fake_commerce.rs`,
`crates/testkit/tests/p14_publish_update.rs`.

### F3 (B2) — corrected

Changed C1(c)'s stored value to valid JSON text `"Sweden"` on a writable text definition while
keeping `valid = false`; the test asserts that the remote value remains absent.

**Files:** `crates/testkit/tests/p14_publish_update.rs`.

## Divergences

F1's correction is not applied because its stated prerequisite—the owner choosing option A or B—has
not been met. This is carried as owner card 1. F2 and F3 follow their correction clauses.

## Files changed

- `crates/core/src/ports/commerce.rs`
- `crates/core/src/services/publish.rs`
- `crates/integrations/src/commerce/shopify/adapter.rs`
- `crates/integrations/src/commerce/shopify/graphql.rs`
- `crates/testkit/src/fake_commerce.rs`
- `crates/testkit/tests/p14_publish_update.rs`
- `docs/implementation/master_plan.md` (P14 tracker note; state remains `CHANGES_REQUESTED`)
- `docs/implementation/plans/P14_publish_update.md` (Review log)
- `docs/implementation/handoffs/implementer/handoff_P14_publish_update_fix_r2.md`

## Checkpoint SHA

`HEAD` after the checkpoint commit; this self-containing handoff is part of that commit, so
`git rev-parse HEAD` resolves the SHA.

## Stamp

| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test p14_publish_update` | 21 passed |
| `cargo test -p psm-integrations mc14_condition_file_ids_are_json_values` | 1 passed |
| `cargo test --workspace` | 370 Rust tests passed; no failures |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `cd apps/desktop && npm run typecheck` | pass |
| `cd apps/desktop && npm test` | 79 passed |
