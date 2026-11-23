---
plan: H3_bring_back
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-23
actor: opus (pipeline-reviewer, Claude Opus 5.5)
---
# H3 — Bring back: review, round 1

**Gate:** passed. The tracker read `IMPLEMENTED`. Checkpoints `fb8d34f` (U1), `c9dad23` (U2), `747a2c4` (U3) and `93d843d` (tracker and log) are all in `git log`. The working tree was clean at the start.

**Review history:** first review. Projection r0 settled L1–L16 before implementation. I did not reopen them, only checked the code against them.

## Verified correct
**Perimeter (step 1).** `git diff --stat fb8d34f^ 747a2c4` lists 26 files, exactly the handoff's declared production, test-kit, test and doc paths. `93d843d` touches only the plan file. Two plan-listed paths did not change (`crates/testkit/src/test_app.rs`, `apps/desktop/src/services/bindings.ts`). The handoff explains both (the `App::build` closure serves the fake shop; no specta type changed), so this is a note, not a finding. Deviation (a), the `h2_pages_and_search.rs` constructor call, is forced by the registry constructor and declared.

**Checks (step 2, plan Review mode, run once each on HEAD code = `747a2c4`).** All green:
- `cargo test -p psm-core --lib mechanisms::remote`: 25 passed.
- `h3_jpeg_photos::`: 10 passed.
- `h3_bring_back::`: 31 passed.
- `h3_publish_after_bring_back::`: 18 passed.
- `p62_retention_after_publish::`: 12 passed.
- `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-desktop -p psm-testkit --all-targets -- -D warnings`: clean.
- `cargo fmt --all -- --check`: clean.

The implementer's full-suite stamp (workspace, clippy, fmt, typecheck, npm test on `747a2c4` content) stands and nothing contradicts it.

**Production code, read against MC-H4 (step 3 and the Review-mode reading checks).**
- `Store::bring_back` (`crates/infra/src/sqlite/store.rs`) is **one transaction** implementing MC-H4.4 (a)–(e) item by item:
  - (a) Deletes the item's links for the session's photos and upserts `photo_places.released = 1` for every row. Each row then gets `deleted`, no position, the three preview paths nulled and `raw_missing`.
  - (b) Inserts one photo per file, gallery first, at the record position, with the canonical path, the SHA-256, `source import` and `captured_at = created_at = at`. The composition is `normal / original / named / 0 / none / 0 / (0,0,1,1) / default / rev 0`, `raw_state` is upserted by path, and the jobs are `original_preview` and `render_preview` with no `hash` job. In a templated session, main photo *k* gets `place_by_hand = 1` with `slot = k` only when a container has `final_no = k`.
  - (c) Each link is `gallery` or `link_role_condition`, slot `single`, `ready`, rev 0, with `retouch_gen` read back from `raw_state` inside the transaction (L16), `composition_rev 0` and no superseded ids.
  - (d) `item_retention`: `cleared_at` null, `restored_at` = at.
  - (e) No `change_seq` write.
- `RemoteItems::bring_back` (`crates/core/src/services/bring_back.rs`):
  - Runs the MC-H4.1 preconditions in the contract's order. An `Unknown` item is read first.
  - Names: `taken` uses the canonical root joined with the folder key. A store error is kept, ends the names and fails before any download (L9).
  - Downloads go through `.part`, then a rename. A body without `FF D8 FF` counts as a failed try. A stale URL triggers one `media_urls` per chunk, and the refreshed URL gets 1 + `DOWNLOAD_RETRIES` tries. A picture still gone after the refresh is stored as gone and fails with "no longer in the shop".
  - `Store::bring_back` runs **only after every file is on disk and canonicalized**, inside `History::retire_session`'s `act` (L8). After that come `sync_folder` (a failure gives `synced = false`), `BringBack {Done}` and `SessionChanged`.
  - Adapter calls are `media_urls`, `fetch_original`, `condition_photos_key`, `inspect_document` (through the uncached closure, L4) and `sync_folder`. None takes a `ConfirmedRun` (M-H1, H1).
  - The `Running` drop guard clears the mark on every exit, unwinding included. The failure path removes only the paths `write` noted (`.part` and final names of this attempt).
- `Store::archived_images` orders `role<>'gallery', position`, so the zip of pictures to names to paths is correct.
- `JpegSource` (`crates/infra/src/rawpreview.rs`): an undecodable file gives `Ok(None)`. A file of 1600 px or less is copied byte for byte; a larger one gets a Lanczos3 q80 resize, which never enlarges. The write is temp + rename. EXIF orientation is read through `kamadak-exif` and kept only for 1–8, and the frame is swapped for 5–8. The pixels are not rotated. `OriginalPreviewJob` chooses it for `jpg`/`jpeg` in any case. Pixel decoding stays in `crates/infra` (H3).
- The publish service was not changed: no `publish.rs` or `mechanisms/publish.rs` in the diff. MC-H4.7 holds on the existing code.
- The evidence doc holds **VERIFY-JPEG: PASS** (listed after sync, `find_image` answers, both recipes export, the master is at pixel size, the `.cos` appears about 15 s after processing).

**Tests (step 3).** Each of the 64 rows has exactly one test at its boundary, and each test asserts the row's exact outcome on the real services. There are no orphan tests. Tests I checked for discrimination by reading:
- C4(a): exact file set, and bytes per name compared with the media.
- C4(i): the exact event sequence.
- C6(a) and C6(f): exactly 3 fetches, and before/after equality of view, rows and `cleared_at`.
- C8(b): the exact op list during the call.
- C9(b) and C9(c): fail without the released and by-hand marks (the implementer's reverted probe confirms this, and the code path through `gaps`/`main_positions` agrees).
- C10(a): op names verified against `fake_commerce.rs` (`staged_uploads_create`, `upload`, `product_create_media`, `file_create`, `product_delete_media` all exist there), so the zero counts are not vacuous.
- C14(b): the exact record after re-clearing.

Deviation (d) on C8(a), "no mutation during each bring back", is the row's intent: the archived fixtures really published first. No test fails the over-engineering test.

**Assumptions (step 4).** None of the 11 contradicts 10H §7 or MC-H4:
- (1) Emitting `Failed` for every failure after the mark goes beyond the contract and stays consistent with it.
- (2) Matches MC-H1.4.
- (3) Matches MC-H4.3.
- (4) Is MC-H4.3's own leftover rule.
- (6) Is L9.
- (7) Is MC-H4.4(b)'s "main photo *k*".
- (8) Fits the H3·C4(i) row.

All of them are in the narrative in plain words.

**Hard constraints (step 5).**
- H1: no mutating call.
- H2: deletes only this attempt's own files, and the exception was added to guide 03.
- H3: decoding stays in infra.
- H4: C6(e) resumes idempotently after a kill.
- H5: Capture One only through the adapter.
- H6: `NoIntegration` without commerce (C7(b)).
- H7: no platform words in core; the `'shopify'` literal in infra's link insert follows the existing pattern (note).
- H8: the Tauri arm and error codes are thin.

No secrets, and no live services in the suite.

**Guide (§7A).** `01_product.md`, `02_architecture.md` (JPEG path, bring-back sequence), `03_rules_and_principles.md` (H2 exception, M-H invariants table) and `04_change_impact.md` rows were all updated in the same change.

## Findings
None blocking (B1–B4). No finding carries a route.

## Notes written to backlog
These are in `docs/implementation/backlog_archive/H3_bring_back.md` (the plan's Review mode), with a one-line pointer in `docs/implementation/backlog.md`:
1. **→ H4:** `leave_gone(item_id)?` runs after the commit. If it errored, the failure path would delete the files just imported and emit `Failed`. Today it can only fail on a poisoned mutex. Post-commit steps should log, never return.
2. **→ H4:** the remote-state check comes before the running mark. A second press inside a microsecond window could run a second full bring back of a now-local item. H4 should disable the press while one runs, or check the state again after taking the mark.
3. **→ book 10H §7.1:** a file the owner placed by hand under a target name is indistinguishable from a leftover, so it is overwritten, and removed on failure.
4. **→ H4:** `Store::bring_back` retires every row of the session, so a RAW filed into the session during the downloads would be retired too (its file stays on disk). This cannot happen before H4's activation.
5. **H7 family:** one more `'shopify'` literal in infra's `photo_links` insert.
6. **Plan hygiene:** `test_app.rs` and `bindings.ts` were listed in U2's Files and did not need to change.

**Carry-forward dispositions:**
| Note | Destination |
|---|---|
| 1, 2, 4 | H4 (command, activation, button state) |
| 3 | orchestrator / book 10H §7.1 |
| 5 | standing "H7 family" debt in `backlog.md` |
| 6 | none (informational) |

## Lessons for the plans
- U2's Files list named two paths "edited" that the design did not need to touch (`test_app.rs`, `bindings.ts` "generated"). Next time, mark a path "only if needed" so a perimeter check does not read it as missing.
- MC-H4.5 lists the post-commit steps but does not say they must never fail the call. A one-line rule ("after the transaction every error is logged, never returned") would have prevented note 1 by construction. Worth adding to MC-H4.5 before H4 adds more post-commit work (activation).

## Mutation-probe declaration
No probe. flow.md §4 turns the mutation regime off, and this review edited no production or test file. No database or app state was touched outside the test runs' temporary directories.

## Files written by this review
- `docs/implementation/handoffs/reviewer/handoff_H3_bring_back_review_r1.md` (this file)
- `docs/implementation/narratives/H3_bring_back.md`
- `docs/implementation/backlog_archive/H3_bring_back.md` (new)
- `docs/implementation/backlog.md` (one appended line)
- `docs/implementation/plans/H3_bring_back.md` (Tracker line → APPROVED; Review log entry appended)

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
