---
plan: P14_publish_update
role: fix
round: 2
state: DONE
date: 2026-10-26
actor: codex
---

# P14 publish update — fix 1 handoff

## Finding → what changed → files

### F1 (B1) — resolved

When `product.tags` is absent from the frozen values, `update` now checks the saved `tags_base`. A non-empty base
means the form cleared the last app-owned tag: the service runs the merge with local `[]`, writes the result only
when it differs from Shopify, then stores base `[]`. Admin tags remain because the merge removes only tags held in
the app's previous base. An absent value with an empty or null base leaves tags and base untouched.

C2(d)(ii) now invokes `AttributesService::clear` to follow the form's actual save path. It asserts both that the
previously published tag is removed from Shopify and that the stored base becomes `[]`.

**Files:** `crates/core/src/services/publish.rs`, `crates/testkit/tests/p14_publish_update.rs`.

## Divergences

None. The correction follows F1 as written.

## Files changed

- `crates/core/src/services/publish.rs`
- `crates/testkit/tests/p14_publish_update.rs`
- `docs/implementation/master_plan.md` (P14 tracker → `IMPLEMENTED`, note: fix 1)
- `docs/implementation/plans/P14_publish_update.md` (Review log)
- `docs/implementation/handoffs/implementer/handoff_P14_publish_update_fix_r2.md` (this handoff)

## Checkpoint SHA

This handoff is included in the single checkpoint commit. Resolve its SHA with `git rev-parse HEAD` after commit;
the final session handoff line reports the resolved value.

## Stamp

| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test p14_publish_update c2d_republish_preserves_admin_tags_and_applies_local_removal` | 1 passed |
| `cargo test --workspace` | 371 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `cd apps/desktop && npm test` | 79 passed |
| `cd apps/desktop && npm run typecheck` | pass |

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
