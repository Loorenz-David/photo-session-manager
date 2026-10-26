---
plan: P14_publish_update
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-26
actor: opus
---
# P14 re-review r2 (after fix 1): APPROVED_WITH_NOTES

Fix 1 resolves F1 as the correction asked. When the last tag is cleared in the form and the app had published tags
before, the next publish now removes those tags from the shop, keeps admin tags, and stores an empty base. The
perimeter is exactly the allowed files. The suite, lint and typecheck are green. No blocking item remains, so the
triage table is empty. The verdict is `APPROVED_WITH_NOTES` because the earlier rounds' notes for this phase stay
open in the backlog. This round adds no notes.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Review history
- First review (archived): F1 (tag base), F2 (condition read-back), F3 (C1(c) fixture). Card 1 was answered from the
  book (R33).
- Review r1 of the continuation: F2, F3 and the R33 tag base hold. F1 (B1): removing the *last* tag clears the field,
  and the tag step was skipped.
- Settled and not re-verified here: C1(a–e), C2(a–c), C2(d)(i), C3(a–c), C4(a–d), C5(a–d), C6(a), C7(a), and
  assumptions 1–5.

## Perimeter check
- Gate: tracker `IMPLEMENTED` with the fix-1 note; checkpoint `4e50bac` exists. Review tree `61130ff` = `4e50bac` + the
  runner's re-review prompt commit. `git status --porcelain` was clean.
- `git show --stat 4e50bac`: `crates/core/src/services/publish.rs` (+20/−5), `crates/testkit/tests/p14_publish_update.rs`
  (+12/−1). Both are files F1 allowed. Also the P14 tracker row, the plan's Review log and the fix handoff, all
  inside the fix role's lane. Nothing else changed. `61130ff` adds only the prompt file.
- L4 on `61130ff` (clean): `cargo test --workspace` 371 passed / 0 failed; `cargo clippy --workspace --all-targets --
  -D warnings` exit 0; `cargo fmt --all -- --check` exit 0; `cd apps/desktop && npm test` 79 passed; `npm run
  typecheck` exit 0.

## Findings resolved
### F1 (B1): resolved as corrected
- **Code** (`publish.rs` `update`, tag block): if `product.tags` is not in `frozen_attributes`, the step reads
  `item_link.tags_base`. A non-empty base runs `merge_tags(base, [], remote_now)`. `update_tags` is called only when
  the result differs, and then `set_item_tags_base(.., [])`. An empty or null base leaves both the tags and the base
  alone. This is exactly the review r1 correction and 07A MC-13 (R33): "a tag removed in the app is removed from the
  shop", and admin tags survive because only `base − local` is removed.
- **No new failure path:** `update` already requires the `item_link` at entry (`external`). The repeated
  `ok_or(UploadIncomplete)` cannot fire on a path that used to succeed.
- **Re-entrancy (H4):** a crash between `update_tags` and the base write leaves the base at `[a]` and the shop at
  `[z]`. On resume the merge result equals `remote_now`, so it returns `None` and stores base `[]`. No double write
  and no lost admin tag.
- **Test:** C2(d)(ii) now clears through `AttributesService::clear`, the form's path. It asserts shop tags `[]` and
  `tags_base == "[]"`. On the pre-fix code the same input left shop `["a"]` (r1 probe), so the row can fail on a
  wrong outcome (B2 satisfied).
- **Changed-input probe** (throwaway `crates/testkit/tests/zz_review_p14_r2_probe.rs`, run, then deleted):
  (1) local `[a]`, shop `[a,z]`, publish, clear, publish → shop `[z]`. A third publish sends no `update_tags`, and
  the item ends `step: Done`. (2) An item never tagged, shop `[z]`, publish → no `update_tags`, shop `[z]`,
  `tags_base` stays `NULL`. Both passed.
- **H1–H8 on the delta:** the tag write stays inside the confirmed run (H1). No RAW, pixel, UI-automation, Tauri or
  new platform type is involved (H2, H3, H5, H7, H8). No secrets and no live calls.
- **Fix assumption** ("absent value with empty or null base: untouched") matches the r1 correction and MC-13's
  never-clear rule. It is not a B1.

## Triage
| Item | Class | Reason |
|---|---|---|
| (none) | — | No blocking item remains after fix 1. |

## Remaining findings
None blocking. Seen in passing and not raised: the absent-value branch repeats the link/base read of the present-value
branch. That is a refactor and fails the over-engineering test, so no note was written. Open notes from earlier rounds
stay in `docs/implementation/backlog.md` (P14 rows: description re-sent every publish, duplicate tag tests, retouch
generation from job payload, H7 literals, MC-24 health message, no `done` assertion after a damage-report write).

## Carry-forward dispositions
| Note (backlog) | Destination |
|---|---|
| Description re-sent on every publish | P15/P16 tidy-up (backlog) |
| Duplicate `merge_tags` testkit tests | backlog (test hygiene) |
| Retouch generation read from job payload | P15/P16 (when the migration happens) |
| H7 `"shopify"` literals in core | plan/H7 (backlog) |
| MC-24 health message for missing condition field | P16 (settings/health) |
| No `done` assertion after damage-report write | backlog (test) |

## Lessons for the plans
- None new. The r1 lesson stands: when a row's input is a UI gesture, name the service call the gesture makes.

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P14_publish_update_review_r2.md` (new; this file)
- `docs/implementation/plans/P14_publish_update.md` (one Review log entry appended)
- `docs/implementation/master_plan.md` (P14 tracker row only → `APPROVED`)
- `docs/implementation/narratives/P14_publish_update.md` (new)
- Probe: `crates/testkit/tests/zz_review_p14_r2_probe.rs` (untracked) was created, run (2 tests) and deleted. No production
  or tracked test file was touched. `git status --porcelain` was empty after deletion. The only side effects were test-harness
  temp directories and the `target/` build cache. No live services, camera or Capture One were used. No commit was made.
