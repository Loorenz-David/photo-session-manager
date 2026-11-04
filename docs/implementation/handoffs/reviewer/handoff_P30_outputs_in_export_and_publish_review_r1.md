---
plan: P30_outputs_in_export_and_publish
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-04
actor: opus (pipeline-reviewer)
---
# Review: P30 outputs in export and publish, round 1

**Verdict: CHANGES_REQUESTED**, one blocking finding (F1, B1, route `production`). The rest of the phase holds: the perimeter matches,
the suite is green, and all 20 rows have a boundary test that fails on a wrong outcome. F1 is a crash or poll-failure path the rows do
not reach. On it, a re-framed photo keeps its old framing on the shop permanently, and the link records it as current.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- Tracker row `IMPLEMENTED`. Checkpoint `a3d561f` is in `git log`.
- Tree for this review: clean `fd9c240` = `a3d561f` + the implementer handoff and the review prompt (docs only).

## Verified correct
- **Perimeter.** `git show --stat a3d561f` lists 23 paths, the same as the handoff's Files changed. Four of them are outside the plan's list
  (`examples/shopify_probe.rs`, `p01_storage_identity.rs`, `p15_publish_backend.rs`, `testkit/Cargo.toml` + `Cargo.lock`). All four are
  declared under Deviations (d) and forced by the signature changes. Plan-listed files with no change (`naming.rs`, `error.rs`,
  `integrations/src/lib.rs`, `test_app.rs`) are explained in the handoff.
- **Suite, run once (L4, tree `fd9c240`, clean porcelain).** `cargo test --workspace` exit 0 (no failures).
  `cargo clippy --workspace --all-targets -- -D warnings` clean. `cargo fmt --all -- --check` clean. `npm run typecheck` clean.
  `npm test` 132 passed.
- **Migration 005.** The table is recreated with `slot` (default `single`) and `composition_rev` (default 0), the rows are copied with the
  same `id`, and the table has `UNIQUE(photo_id, integration, item_id, role, slot)`. C3(b) migrates a real v4 database through
  `SqliteStore::open` and reads the row back field by field, plus the unique index columns.
- **Stem and matching.** `stem` = `<folder_key>-<uid12 lowercase>` + `-L`/`-R` + `-r<n>` only for n ≥ 1. `matches` accepts the exact name
  or `_<uuid v4>`, case-insensitively, so `K-uid` never matches `K-uid-L.jpg`. The adapter and the fake only append `.jpg`.
- **Replacement.** `replacement_candidates` applies to ready links only: retouch newer **or** composition revision greater. The unit test
  covers both clauses and a non-ready link.
- **Refusals.** `confirm` collects `needs_review` labels before anything is written. C7(a) checks the `publish_runs` count and that the
  fake recorded no mutations. `prepare` runs the same check before `item_search`/`item_create`. C7(b) asserts `failed_step Prepare` and the
  message, then Retry → `done`.
- **Staging reuse (C8(a)).** The manifest's format key, entry count, and per output the photo, slot, `adjusted_gen` and composition
  revision must all match, and the file must be present. A resumed `upload` whose staging is stale goes back to `prepare`. C8(a) compares
  the uploaded bytes' hash with the r1 output in the cache.
- **Per-output links, upload, update.** Retirement runs at `prepare` and changes only the state (the external id is kept). Upload walks
  `output_names`. `D` and the damage report come from the current slots. `own` = every gallery link's id and superseded ids, plus stems for
  every slot and revisions 0…n, so retired media are removed without `foreign_media_added`. The C2–C6 tests assert exact filenames, orders,
  ids and metafield values.
- **Masters.** Released in `run_item`'s `Done` branch only when `session.dirty == false`. The release skips RAWs that an export in flight
  or a pending master still needs, and only deletes under app storage (H2). C8(b) and C8(c) cover the clean and dirty sides.
- **Session view.** The badge shows the first slot's link (C4(c)). Every photo has a composition row (insert plus P28 backfill), so the new
  `composition(photo)` lookup cannot fail on a photo without one.
- **Deviations judged.** (a) C4(b)/(c): the plan's own key allows one `(a, single)` row, so reusing the retired row (first media id →
  `superseded_ids`) keeps the rows' intent, and the outcomes the owner can observe match the rows. (b) C3(d): under P28, a `set` that moves
  a composition from default to user origin is a change, so the row publishes a user composition first. Its observable outcome (no upload,
  no revision bump, not dirty) is what the row specifies. (c) C8(c): the photo changes during `upload`, before `update`, which is still a
  change during the run.
- **Assumptions.** 1, 2, 3, 4, 6 and 7 do not contradict the book: 2 follows 10D's and the plan's `K-…-L` form over 07A's "(lowercase)",
  see the backlog. Assumption 5 is the cause of F1.
- **H1–H8.** H1, H2, H3, H5, H6, H7 and H8 hold against the diff (no Shopify/Tauri concept added to core types beyond the existing link
  table; no RAW touched). H4 is broken on the F1 path.

## Findings

### F1 — B1 · route: production · a crashed or poll-failed upload followed by a re-frame leaves the old framing on the shop, recorded as current
- **What is wrong.** `prepare` writes the staged file's `composition_rev`/`retouch_gen` onto every link that is not `ready` (assumption 5,
  `services/publish.rs` in `prepare`, the `if link.upload_state != "ready"` block). That includes links in `staged`/`uploaded`/`attached`,
  whose remote media was already created from the **previous** staging. `upload` then reuses that media. With `external_id = None` it is
  adopted by its unchanged stem. With `attached` the recorded id is polled. The link ends up `ready` with the new revision and the new
  hash, while the shop holds the old rendering. No later publish replaces it, because the link now claims the current revision.
- **Reproduced (reviewer probe in a scratch git worktree at `a3d561f`, test added there only, worktree removed).** Setup: `K` main `[a]`,
  `fail_after_effect("product_create_media")`, confirm, drain (killed after the media was created), `kill_and_restart`,
  `other_square(a)` (composition revision 1), drain. Result: item `done`. The link is `ready`, `revision 0`, `composition_rev 1`,
  `export_sha256` = sha(r1 output) = `b904…`. The gallery's only media is `K-<uid12>.jpg`, uploaded with sha `a114…` (the revision-0
  bytes). Staged filenames are `[K-<uid12>.jpg]`, so nothing was re-uploaded. `dirty == true`. Publishing again → `done`, nothing
  uploaded, `dirty == false`.
- **Why it is P30's.** Before this phase, a non-ready link kept its old `retouch_gen`, so the next publish saw the newer generation and
  replaced the media. Stamping at `prepare` removed that self-healing and extends the hole to composition revisions. The plan's C8(a)
  exists for exactly this class (Notes: "a stale rendering — the exact M35 defect").
- **Authority.** 10D MC-29 M35 ("every uploaded file was rendered from the photo's current composition revision"). MC-28 §Cache key
  ("Stale outputs are never … uploaded"). 01 H4 (resuming after a crash yields the same end state). Plan C8(a) and Notes.
- **Correction.** At `prepare`, treat a link that is not `ready`, whose state shows remote media may exist (`staged`, `uploaded`,
  `attached`), and whose recorded `composition_rev` or `retouch_gen` is older than the staged file's, as a replacement: old `external_id`
  → `superseded_ids`, `revision += 1`, state `none`. The stale media then matches an `own` stem or superseded id, and gallery
  reconciliation removes it. Add one test in `p30_outputs_in_export_and_publish.rs` with the probe's shape. It asserts that after the
  resume the gallery holds exactly one media for `a`, and that its `uploaded_sha` equals the r1 output's hash.
- **Files the fix may touch.** `crates/core/src/services/publish.rs`; optionally `crates/core/src/mechanisms/publish.rs` (and its existing
  unit test) if the rule lives in `replacement_candidates`; `crates/testkit/tests/p30_outputs_in_export_and_publish.rs`.

## Notes written to backlog
1. P30 → book 07A: the stem keeps the folder key's case, against 07A's "(lowercase)". The book contradicts itself. Amend 07A to 10D's form.
2. P30: a Normal ↔ Split change between `upload` and `update` makes `update` fail with `UploadIncomplete` on every Retry. A new publish
   recovers.
3. P30: links migrated by 005 carry `composition_rev 0`, so the first publish after the update re-uploads (`-r<n>`) photos whose
   composition revision is ≥ 1, once.
4. P30 → P31: an identical `set` on a default-origin composition bumps the revision and makes the session dirty. The editor must not call
   `set` when nothing moved.
5. P30 plan lesson: C4(c)'s "three retired rows" is impossible under the plan's own UNIQUE key.

## Lessons for the plans
- A row on staging reuse should name both fault shapes, failure **before** the remote effect and failure **after** it (media created, not
  recorded or not ready). C8(a) named only the first, so the implementer's fake change (assumption 7) made the second unreachable from
  the rows.
- Rows that count database rows must be derived from the key the plan itself defines (C4(c)).

## Write perimeter (this session)
- `docs/implementation/master_plan.md`: P30 tracker row only (→ `CHANGES_REQUESTED`).
- `docs/implementation/plans/P30_outputs_in_export_and_publish.md`: Review log entry appended.
- `docs/implementation/backlog.md`: 5 rows appended.
- This handoff.
- Probe: a scratch `git worktree` at `/tmp/p30_probe` (detached at `a3d561f`). One scratch test was appended to its copy of
  `crates/testkit/tests/p30_outputs_in_export_and_publish.rs`, then the worktree was removed with `git worktree remove --force` and
  `git worktree prune`. No file in the main working tree was touched by the probe, and no production code was edited anywhere. The probe
  built into the shared `target/` directory (build cache only). The main tree's `git status --porcelain` was empty after removal.
- No narrative (verdict is not an approval). No commits.
