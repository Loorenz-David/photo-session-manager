---
plan: P14_publish_update
role: implement
round: 1
state: DONE
date: 2026-10-26
actor: codex
---

# P14 publish update — implement continuation

## What was built

A tag added in Shopify survives another publish, while a tag removed in Session Manager is removed from Shopify. The app now remembers the local tag list after each successful update, even when the first publish needs no tag write. An empty local list can remove previously shared tags. The existing P14 attribute, condition-photo, identity, retouch, and summary behavior remains covered by its boundary tests.

## Tests → criterion rows

Each P14 row has one matching boundary test in `crates/testkit/tests/p14_publish_update.rs`:

| Criterion rows | Tests and outcome |
|---|---|
| C1(a–e) | `c1a_*` through `c1e_*`: confirmed attribute diff, no clear, invalid and read-only suppression, read-back failure |
| C2(a–d) | `c2a_*` through `c2d_*`: merge outcomes, local base, two publishes preserving an admin tag and applying a local removal |
| C3(a–c) | `c3a_*`, `c3b_*`, `c3c_*`: title, description, identity |
| C4(a–d) | `c4a_*` through `c4d_*`: condition files, republish, no-value case, missing field |
| C5(a–d) | `c5a_*` through `c5d_*`: retouch revision, replacement, condition file, crash adoption |
| C6(a), C7(a) | `c6a_*`, `c7a_*`: summary text and publish step sequence |

C2(c)'s corrected base assertion and the new C2(d) test failed against the starting code and pass after the fix. C2(d) runs through the real publish service and fake shop, asserting shop tags and the absence of an unnecessary tag mutation.

## Assumptions

- The base advances only when the frozen confirmation includes a valid local tag value. An absent local value remains untouched under MC-13's no-clear rule.
- The latest P14 Review log and corrected 07A MC-13 govern this continuation, although the launch wrapper labels it round 1.

## Deviations

None. Earlier F2 and F3 fixes were preserved. No live Shopify or camera call was needed for this correction.

## Files changed

- `crates/core/src/services/publish.rs` — persist local tags as the base after the merge.
- `crates/testkit/tests/p14_publish_update.rs` — corrected C2(c), added C2(d).
- `docs/implementation/master_plan.md` — P14 tracker to IMPLEMENTED.
- `docs/implementation/plans/P14_publish_update.md` — Review log.
- `docs/implementation/handoffs/implementer/handoff_P14_publish_update_implement_r1.md` — this handoff; the runner commits handoffs separately under flow §3 and §8.

## Checkpoint SHA

`87dda4a` — `CHECKPOINT (not approved): P14 tag base correction`.

## Stamp

| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test p14_publish_update c2` before production edit | Expected red: C2(c) stored `[y,x]` instead of `[x]`; C2(d) removed the admin tag on the second publish. |
| `cargo test -p psm-testkit --test p14_publish_update` | 22 passed. |
| `cargo test --workspace` | Passed, no failures. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed. |
| `cargo fmt --all -- --check` | Passed. |
| `npm test` in `apps/desktop` | 79 passed. |
| `npm run typecheck` in `apps/desktop` | Passed. |

## Manual try-out

1. With the app and the test Shopify store configured, add a tag in Session Manager and a different tag directly in Shopify admin. Publish the session twice. Both tags should remain after the second publish.
2. For a product whose tag appears in both places, publish once, clear the tag in Session Manager, and publish again. The tag should disappear from Shopify.
3. For the wider P14 behavior, fill a Details field and publish; leave another field empty that has an admin value. Check that the first is updated and the second is retained. Exclude one condition photo and confirm the damage report lists the others in order. Retouch a published photo in Capture One, then republish and check that its new version occupies the same gallery position.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
