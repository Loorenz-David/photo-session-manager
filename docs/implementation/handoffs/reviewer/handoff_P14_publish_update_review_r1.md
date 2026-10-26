---
plan: P14_publish_update
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-10-26
actor: opus
---
# P14 review r1 (continuation): CHANGES_REQUESTED

This review covers the continuation attempt: the fix-r2 tree (`ca203c0`, F2 and F3 of the archived first review,
never reviewed until now) plus the tag-base correction (`87dda4a`, F1 under R33). F2 and F3 hold. The tag base is
right for every path that stores a list. One blocking defect is left. Removing the **last** tag in the form clears
the field instead of storing `[]`, so the tag step never runs and the tag stays on the shop. The book answers the
correction (07A MC-13, R33), so no owner card is needed.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Review history
- Archived first review (`archive/P14_publish_update_20261026-071423/`): 19/21 rows held. F1 (tag base), F2
  (condition read-back on the real adapter), F3 (C1(c) fixture) blocked. Card 1 was answered from the book, option A
  (R33). C2(c) was amended and C2(d) added.
- Settled there, not re-verified here except where the deltas touch them: C1(a,b,d,e), C2(a,b), C3(a–c), C4(a–d),
  C5(a–d), C6(a), C7(a), assumptions 1–5, H1–H8.

## Verified correct
- **Gate:** tracker `IMPLEMENTED`; checkpoint `87dda4a` is in `git log`. Review tree `931cb88` = checkpoint + the
  runner's handoff commit `59bd0ba` + review prompt. `git status --porcelain` was clean before the review.
- **Perimeter:** `git show --stat 87dda4a` = `publish.rs`, `p14_publish_update.rs`, `master_plan.md`, plan file,
  exactly the declared files. The handoff file is committed by the runner (`59bd0ba`), as declared. Fix r2
  (`535dd2a..ca203c0`) = its 6 declared code files + docs.
- **L4 stamp (tree `931cb88`, clean):** `cargo test --workspace` 371 passed / 0 failed; `cargo clippy --workspace
  --all-targets -- -D warnings` exit 0; `cargo fmt --all -- --check` exit 0; `npm test` 79 passed; `npm run
  typecheck` exit 0.
- **F2 resolved:** a new port read `condition_photos_readback` (platform-neutral name, H7). The Shopify adapter reads
  `product.metafield(namespace, key){value}` for the configured key and parses the JSON id list (unit test
  `mc14_condition_file_ids_are_json_values`). `update` uses it only for the `media_list` key and compares exactly.
  The fake now omits the condition key from `attributes_read`, as the adapter does. A missing remote value counts as
  failed. (Guard weakness: see the backlog note.)
- **F3 resolved:** C1(c) stores JSON `"Sweden"` with `valid = false` on a writable text definition, against a shop
  without the key. If `frozen_attributes` dropped its `valid` filter, the value would be written and the test would
  fail.
- **Tag base (R33):** `set_item_tags_base(local_tags)` runs after the merge, whether or not a write happened, and
  only after `update_tags` succeeds. C2(c) asserts base `["x"]`. C2(d)(i) fails on the old code: the second publish
  computed `[x]` and wrote it. It passes now (`[y,x]`, no second tag write). A crash between `update_tags` and the base
  write re-merges to `None` on resume (H4). Reviewer probe: local `[a,b]`, shop `[z]`, publish, then local `[a]`,
  publish → shop `[z,a]` (local removal applied, admin tag kept).
- **Assumptions:** (2) "continuation labelled round 1" is process only. (1) "the base advances only for a frozen local
  tag value; an absent value stays untouched" is the cause of F1, because in the real app an emptied tag field *is*
  an absent value.
- **H1–H8 on the deltas:** writes stay inside a confirmed run (H1). Nothing touches a RAW (H2) or pixels (H3).
  Re-entrant as noted (H4). No UI automation (H5). The new port method is optional to the workflow (H6). No new
  platform type in core (H7). No Tauri dependency (H8). No secrets, no live calls.

## Findings

### F1 (B1, blocking): `route: production`
- **What:** the form's save path (`useFormState.write`) calls `clearAttribute` when a value is empty, and an empty tag
  list counts as empty (`isEmptyValue`). A cleared row is not live, so `frozen_attributes` has no `product.tags`. The
  whole tag block in `update` is then skipped. No merge runs and the base is not updated, so the shop keeps every tag
  the app had published. Reviewer probe (throwaway test file, deleted): local `["a"]`, shop `["a"]`, publish (no tag
  write, base `["a"]`), `AttributesService::clear(item, "product.tags")`, publish → shop tags `["a"]`, item `done`.
  The C2(d)(ii) test models "clear local tags" as `set_value([])`, an input the form never sends, so the suite stays
  green.
- **Authority:** 07A MC-13 Product tags (R33): "After every successful `update` the base becomes the current local
  list … a tag removed in the app is removed from the shop"; plan C2(d)(ii) ("clear local tags, publish → shop
  `[]`"); orchestrator's R33 instruction ("an empty local list after a non-empty base counts as a value"); flow §5 B1.
- **Correction:** in `update`, when `product.tags` is absent from the frozen values and the stored `tags_base` is
  non-empty, run the merge with local `[]`, write if the result differs, and store base `[]`. With an absent value
  and an empty or null base, leave everything untouched. Only tags the app itself held are removed, never admin tags.
  In C2(d)(ii), clear the field through `AttributesService::clear`, the form's path, instead of `set_value([])`.
- **Files:** `crates/core/src/services/publish.rs`, `crates/testkit/tests/p14_publish_update.rs`.

## Notes written to backlog
1. P14 (test, fix r2): no P14 test asserts that a publish which writes the damage report ends `done`. The F2 fix is
   verified by reading only, and a regression to `attributes_read` would stay green.

## Lessons for the plans
- When a row's input is a UI gesture ("clear local tags"), the plan should name the service call that gesture makes.
  Here the form clears the field rather than storing `[]`, and the test picked the other path.

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P14_publish_update_review_r1.md` (new; this file)
- `docs/implementation/plans/P14_publish_update.md` (one Review log entry appended)
- `docs/implementation/master_plan.md` (P14 tracker row only → `CHANGES_REQUESTED`)
- `docs/implementation/backlog.md` (1 row appended)
- Probe: `crates/testkit/tests/zz_review_p14_probe.rs` (untracked) was created twice, run (the first attempt did not
  compile; the second ran 2 probes) and deleted. No production or tracked test file was touched, and `git status` showed only
  the three docs above afterwards. The only side effects were temporary directories from the test harness and the
  `target/` build cache. No live services, camera or Capture One were used. No commit was made.
