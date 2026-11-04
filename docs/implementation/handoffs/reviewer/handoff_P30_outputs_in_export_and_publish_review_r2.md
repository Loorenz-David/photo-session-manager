---
plan: P30_outputs_in_export_and_publish
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-04
actor: opus (pipeline-reviewer)
---
# Re-review: P30 outputs in export and publish, round 2 (after fix 1)

**Verdict: APPROVED_WITH_NOTES.** F1, the only blocking finding from round 1, is resolved as its correction asked. The fix stays inside the
allowed files and the suite is green. The one declared divergence (C8(a) now expects the `-r1` name) is sound. No blocking item remains, so
there is nothing to triage. I wrote two new backlog notes, set the tracker to `APPROVED` and wrote the narrative.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
Round 1 settled the perimeter of `a3d561f`, migration 005, stem and matching, the replacement candidates, the refusals, staging reuse,
per-output links, upload and update, the release of masters, the session view, deviations (a)–(d), assumptions 1–4 and 6–7, and H1–H8
except H4 on the F1 path. None of that is re-verified here. This round covers the fix's perimeter, the changed `prepare` seam and its
dependents in `upload`/`update`, and one suite run.

## Gate
- Tracker row was `IMPLEMENTED` with a fix-1 note. Checkpoint `c96cbd9` is in `git log`.
- Review tree: clean `c54508c`. That is `c96cbd9` plus the fix handoff and this round's prompt, docs only.

## Perimeter check
`git show --stat c96cbd9`:
| File | Allowed by F1? |
|---|---|
| `crates/core/src/services/publish.rs` (+7/−1) | yes |
| `crates/testkit/tests/p30_outputs_in_export_and_publish.rs` (+36/−1) | yes |
| `docs/implementation/master_plan.md` (P30 row only) | yes (own tracker row) |
| `docs/implementation/plans/P30_outputs_in_export_and_publish.md` (Review log append) | yes |

`crates/core/src/mechanisms/publish.rs` was optional and is untouched. `git diff --stat c96cbd9 c54508c` shows only the fix handoff and the
r2 prompt. The perimeter matches the fix handoff's Files changed exactly.

## Suite
The prompt directs one run of suite, lint and typecheck on the review tree, so this is that run (L4, clean `c54508c`, code identical to the
fix stamp's `c96cbd9`).
- `cargo test --workspace`: 529 passed, 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --all -- --check`: clean.
- `npm run typecheck` (apps/desktop): clean.
- `npm test`: 132 passed.

## Findings resolved
### F1: resolved as corrected
- **Change.** In `prepare`, `replaced` now also holds when the link is in `staged`, `uploaded` or `attached` and its `composition_rev` is
  below the staged file's, or its `retouch_gen` is below `export_gen`. The existing branch then moves `external_id` → `superseded_ids`,
  sets `revision += 1` and state `none`. That is the correction word for word.
- **Covered states, read against `upload`.** `staged` is written before `commerce.upload`, `uploaded` before
  `product_create_media`/`file_create`, and `attached` after them. Every state in which remote media can exist is therefore in the set.
  `exported` comes before `staged_uploads_create` only, and that call creates no media. `none` and `retired` are handled elsewhere.
- **Downstream.** After the bump, adoption looks for the `-r1` stem only and skips superseded ids, so the stale media is never adopted
  (07B MC-21: "matches the stem of the link's current revision only"). `update`'s `own` includes stems for revisions 0…`revision` and the
  superseded ids, so reconciliation deletes the stale gallery media. For the condition role, the damage report points at the new file and
  the old one stays unreferenced in Files, as 07B MC-21 §Condition photos says.
- **No double bump.** A replaced link is stamped with the staged values and left in `none`. If `prepare` runs again on the same staging
  (crash after `prepare`), the values are equal and `none` is outside the set, so the revision stays at 1 and the link moves to
  `exported`.
- **Test.** `c8a_a_media_created_before_a_crash_is_replaced_when_the_photo_is_reframed` has the probe's shape: fault after
  `product_create_media`, crash, restart, re-frame, drain. It asserts the item is `done`, there is exactly one media for `a`, that media's
  `uploaded_sha` equals the r1 output's hash, and the link's `export_sha256` and `external_id` match. Under the r1 code, the round-1 probe
  showed the only media carrying the revision-0 hash (`a114…` ≠ `b904…`), so the `uploaded_sha` assertion fails on the defect. The
  assertions are on outcomes at the boundary (the fake shop's media and the stored link), with no internal coupling.

### Declared divergence: C8(a) expects `K-<uid12>-r1.jpg`. Sound.
The existing row (failure before the effect, link left `uploaded`, no media on the shop) now takes the replacement path, because
`prepare` cannot tell whether media exists. This follows F1's correction as written, and it is the safe side the book's naming allows. Only
the filename key changed. The hash, the link and the old-staging checks are unchanged, so the row still fails if a stale rendering is
uploaded. The plan row's literal name goes to the backlog as a plan lesson.

## Triage
No blocking items remain after fix 1.
| Item | Class | Disposition |
|---|---|---|
| (none) | — | — |

## Remaining findings
None blocking. Two notes appended to `docs/implementation/backlog.md`:
1. **Plan lesson.** C8(a)'s filename key should read `K-<uid12(a)>-r1.jpg` after fix 1.
2. **Code reading, not reproduced.** The new trigger fires only on an older `composition_rev`/`retouch_gen`. If the output size or quality
   changes in Settings between a crashed upload and its resume, the old-format media is adopted and marked `ready` with the new hash.
   The framing is right and only the size/quality is old. Ready links are not replaced on a format change either, and the book does not
   ask for that. It is a book decision.

## Carry-forward dispositions
| Note | Destination |
|---|---|
| R1 notes 1–5 (07A case, Normal↔Split mid-run retry, migrated `composition_rev 0`, identical `set` on default, C4(c) count) | backlog (as routed in r1: 07A amendment, P31, plan lesson) |
| R2 note 1 (C8(a) name) | backlog → coordinator plan fold |
| R2 note 2 (format change not a replacement trigger) | backlog → orchestrator/book (decide whether a format change re-uploads) |

## Lessons for the plans
- When a correction changes the revision rule, rows that name literal upload filenames have to be re-derived. C8(a) named `K-…jpg`, and
  the correction made that `-r1`.

## Write perimeter (this session)
- `docs/implementation/master_plan.md`: P30 tracker row only (→ `APPROVED`).
- `docs/implementation/plans/P30_outputs_in_export_and_publish.md`: Review log entry appended.
- `docs/implementation/backlog.md`: 2 rows appended.
- `docs/implementation/narratives/P30_outputs_in_export_and_publish.md`: new.
- This handoff.
- No probes, no production or test edits, no worktree. Build cache only in `target/`. No commits. No archgraph state.
