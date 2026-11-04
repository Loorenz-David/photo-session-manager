---
plan: P30_outputs_in_export_and_publish
role: fix
round: 2
state: DONE
date: 2026-11-04
actor: sonnet (pipeline-implementer)
---
# Fix 1 — P30 outputs in export and publish

## Finding → what changed → files
**F1 (B1, production): a crashed or poll-failed upload followed by a re-frame left the old framing on the shop, recorded as current.**
- `prepare` (`services/publish.rs`) now treats a link in `staged`/`uploaded`/`attached` whose recorded `composition_rev` or
  `retouch_gen` is older than the staged file's as a replacement, the same branch as a ready link: old `external_id` → `superseded_ids`,
  `revision += 1`, state `none`. The new stem no longer adopts the stale media; it matches an own stem or superseded id, so gallery
  reconciliation removes it.
- New test `c8a_a_media_created_before_a_crash_is_replaced_when_the_photo_is_reframed` (the reviewer probe's shape): media created, process
  killed, restart, re-frame, drain → item `done`, exactly one media for `a`, its `uploaded_sha` and the link's `export_sha256` equal the
  r1 output's hash, link `external_id` is that media.
- Files: `crates/core/src/services/publish.rs`, `crates/testkit/tests/p30_outputs_in_export_and_publish.rs`.
  `mechanisms/publish.rs` not touched (the rule is in `prepare`; `replacement_candidates` stays ready-only).

## Divergences
- The existing test `c8a_a_stale_staging_from_a_failed_attempt_is_rendered_again` (failure *before* the effect, link state `uploaded`, no
  media on the shop) now also takes the replacement path, because `prepare` cannot tell whether media exists. Its file is uploaded as
  `K-<uid12>-r1.jpg`, not `K-<uid12>.jpg`. I changed only the filename key in that test's `uploaded_sha` assertion; the hash, link and
  cleanup assertions are unchanged. Plan row C8(a) names the unsuffixed file; this follows the finding's correction (`revision += 1`)
  as written. Candidate plan amendment: C8(a) expects `-r1` (or the row names both fault shapes, per the reviewer's lesson).

## Files changed
- `crates/core/src/services/publish.rs`
- `crates/testkit/tests/p30_outputs_in_export_and_publish.rs`
- `docs/implementation/master_plan.md` (P30 tracker row only → `IMPLEMENTED`, fix 1)
- `docs/implementation/plans/P30_outputs_in_export_and_publish.md` (Review log entry appended)
- This handoff.
No mutation probes; no production code edited outside the list.

## Checkpoint SHA
`c96cbd9` (`CHECKPOINT (not approved): fix 1 P30 outputs in export and publish`)

## Stamp
Tree `c96cbd9`, clean porcelain before the handoff. `cargo test --workspace`: 529 passed, 0 failed (P30 file 21/21). `cargo clippy
--workspace --all-targets -- -D warnings` clean. `cargo fmt --all -- --check` clean. `npm run typecheck` clean, `npm test` 132 passed
(run from `apps/desktop`).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
