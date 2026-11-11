---
plan: P66_approve_on_arrival_and_session_order
role: implement
round: 1
state: DONE
date: 2026-11-11
actor: codex
---
# P66 implement r1 — handoff

Run by Claude Opus (pipeline-implementer agent) in the `codex` actor slot. All 7 rows have a passing test at their boundary;
the Rust suite, clippy, fmt, typecheck and the UI suite are green (stamp below; one known intermittent UI test).

⚠ OWNER DECISIONS REQUIRED (0) — none.

## What was built
- **Photos arrive approved.** Every new photo (camera over FTP or cable, Capture One tether, import, and Unassigned too) joins its
  session as **Main, at the end of Main, in arrival order**. Capture One gets the green tag as soon as it has delivered the photo's
  first adjusted preview. A session whose photos all arrived is **Ready** as soon as previews exist. Photos that were already
  "unreviewed" before this change stay that way and can still be approved with A.
- **Condition or Deny instead of Approve.** In the workspace: **C** / swipe up → Condition, **⌫** / swipe left / ⋯ "Deny" → Deny.
  The round buttons are Deny and Condition (Move to Main on a condition photo). Approve shows only on a leftover unreviewed photo.
  After an action the **next photo in order** opens. The toast reads "Photo denied · Undo ⌘Z".
- **Deny deletes the RAW after the undo window.** A denied photo is hidden and never published. Undo restores it fully while the
  window is open. The window closes when the **session's next publish completes**, or **24 h after the deny**, whichever comes first.
  Then the storage job (P61) deletes the RAW and the app's own files of it (previews, thumbnail, masters, outputs) and logs
  `storage.cleared rule=denied path=… bytes=…`. The photo row stays as a denied, file-gone entry, its undo steps are dropped, and it
  can no longer be restored. At delete time the photo is read again under the undo-history lock. Before deleting, the job checks that
  the photo is still denied and its window has closed, and that every other photo on the same RAW is also denied with its window closed.
  A RAW that a photo still uses is never touched.
- **Sessions list in creation order.** Newest created on top. The time groups and the time filter use the creation time. The column
  reads "Created ↓" and each row shows when the session was created. Opening, editing, capturing into or publishing a session never
  moves it.

## Tests → criterion rows
| Row | Test (boundary) | Asserts the row's exact outcome? |
|---|---|---|
| C1(a) | `p66…::c1a_three_arrivals_are_main_in_arrival_order_mirrored_green_and_ready_once_previews_exist`: FTP, import, FTP into an active session (stored photos, fake Capture One tags, session list status) | yes: `[(DSC00001, main, 1), (IMPORTED, main, 2), (DSC00003, main, 3)]`; tag 4 (green) on all three variants; `InProgress` before previews, `Ready` after |
| C1(b) | `p66…::c1b_photos_already_unreviewed_stay_unreviewed_and_a_new_arrival_is_main` (stored photos) | yes: the two old photos are `unreviewed`/no position; the new one is `main`/1 |
| C2(a) | `p66…::c2a_deny_then_undo_within_the_window_restores_the_photo_and_never_touches_the_raw` (review + history services, files on disk, storage job) | yes: back at `main`/2 among 1–3; RAW bytes and the app's files unchanged after 2 days of storage ticks; no `denied` deletion |
| C2(b)(i) | `p66…::c2b_i_deny_deletes_the_raw_when_the_sessions_next_publish_completes` (fake shop, storage job, log, disk) | yes: nothing deleted before the publish; publish Done with 3 uploads (the denied photo not among them); deleted set == the denied RAW + its previews; each logged `rule=denied`; the restored and non-denied RAWs byte-identical |
| C2(b)(ii) | `p66…::c2b_ii_deny_deletes_the_raw_24_hours_after_the_deny_without_a_publish` (storage job, log, disk, history) | yes: nothing at 23 h; at 24 h + 1 min the deleted set == denied RAW + its files, logged; restored and other photo untouched; undo top moves from `Delete 01` to `Approve 03` (deny step dropped), the photo stays `deleted`, re-classify refused |
| C3(a) | `p66…::c3a_sessions_stay_in_creation_order_newest_first_and_are_grouped_by_creation_time` (sessions list service, Open/All, with and without a query) | yes: `[C, B, A]` after opening A, editing B and capturing into A; C and B "Today", A (created 40 days ago) not |
| C3(b) | `p66_approve_on_arrival_ui.test.tsx` (workspace + sessions page, mocked services; real pointer drags on the swipe layer): 6 tests | yes: buttons `['Deny','Move to Condition']` on an arrived photo; C → `classifyPhoto(1,'condition')` and photo 2 opens; ⌫ → `deletePhoto(2)`, the toast text, photo 3 opens; swipe left → `deletePhoto(1)`; swipe up → condition; A on a main photo calls nothing; an unreviewed photo shows Approve and A → `classifyPhoto(9,'main')`; the list renders C, B, A, row time = creation time, the "Today" filter by creation time hides A |
| C4(a) | the full stamp below | — |

Every test in the two new files maps to a row above. No unit tests (the plan lists no core mechanisms).

## Assumptions
1. Unassigned arrivals are `main` too (book: "every new photo").
2. The green tag is written once Capture One has delivered the photo's first adjusted preview, not at the instant of arrival. Queuing
   it at arrival made the tag job poll Capture One by itself and script a stopped Capture One (p49 C2(d) caught it).
3. Photos deleted **before** P66 have no deny mark and keep their RAW (H2 as it was). Only denies made from now on are deleted.
4. Capture One's own files for a denied RAW (`CaptureOne/Settings*/…cos`, its cache) and `.duplicates/` copies are not deleted. The
   plan names "RAW + app-owned derivatives", and Capture One shows the variant as missing.
5. A RAW shared with a copied photo row (Copy to session…) is deleted only when every row on it is denied with its window closed.
6. Undo-history labels keep the registry wording (`Delete 03`); buttons, menu, swipe stamp and toast say "Deny".
7. Each session row shows its creation time, so it agrees with the order and the groups ("Created ↓").
8. The window closes on a publish that completes **after** the deny (`published_at` strictly later). That includes a publish that was
   already running when the photo was denied.
9. A "keep files" pin does not protect a denied photo: Deny is the photographer's explicit choice.
10. The storage job checks pending denials every batch pause (2 s), not only on its 5-minute scan, so a completed publish deletes the
   denied RAWs within seconds.
11. When a session opens, the first leftover unreviewed photo is still shown first (unchanged default). A main photo is shown first
   only when nothing is unreviewed.

## Deviations
- Production files beyond the plan's list (all needed for the rows): `crates/core/src/domain/{session,storage}.rs`,
  `crates/core/src/ports/store.rs`, `crates/core/src/services/{history,previews}.rs`, `crates/integrations/src/lib.rs` (storage gets
  the history), `apps/desktop/src-tauri/src/commands/sessions.rs` (`created_at` on the row DTO), `apps/desktop/src/services/{bindings,photos}.ts`,
  new migration `crates/infra/src/sqlite/migrations/012_photo_denials.sql` (table `photo_denials(photo_id, denied_at, files_deleted_at)`).
- Existing tests changed because the behaviour they asserted changed: p01 (table list gains `photo_denials`), p05 c5a/c8a (an arrival is
  `main`/1), p62 (its hidden-photo fixtures now represent photos hidden before P66, by removing the deny mark, so P62's Q1 coverage
  stays), UI p08 (toast text; `created_at` in a fixture), p07/p15/p18 (`created_at` in fixtures).
- **For the orchestrator (registry and book; not edited by me):** master plan §6.5 gains the table `photo_denials` (migration 012).
  §6.4 `Store` gains `pending_denials`, `photo_denial`, `mark_denial_files_deleted`, and `arrival_insert` takes a `Classification`.
  `StoreTx` gains `set_photo_denied`. §6.7 `SessionRowResponse` gains `created_at`. `StorageService::with_history` and
  `History::retire_photo` are new. `DENY_UNDO_WINDOW_MS` = 24 h, rule `denied`. Book 06 (Arrival, Classification, "next unreviewed")
  and 01 H2 still carry the old wording.

## Files changed
`apps/desktop/src-tauri/src/commands/sessions.rs`, `apps/desktop/src/features/sessions/{SessionRow,SessionsPage,TimeFilter}.tsx`,
`apps/desktop/src/features/workspace/{PhotoMenu,ReviewButtons,SwipeLayer}.tsx`, `apps/desktop/src/features/workspace/useWorkspaceState.ts`,
`apps/desktop/src/services/{bindings,photos}.ts`, `crates/core/src/domain/{session,storage}.rs`, `crates/core/src/ports/store.rs`,
`crates/core/src/services/{arrival,history,previews,review,sessions,storage}.rs`, `crates/infra/src/sqlite/store.rs`,
`crates/infra/src/sqlite/migrations/012_photo_denials.sql` (new), `crates/integrations/src/lib.rs`;
tests: `crates/testkit/tests/p66_approve_on_arrival_and_session_order.rs` (new), `apps/desktop/src/features/workspace/__tests__/p66_approve_on_arrival_ui.test.tsx` (new),
`crates/testkit/tests/{p01_storage_identity,p05_capture_arrival,p62_retention_after_publish}.rs`,
`apps/desktop/src/features/{sessions/__tests__/p07_shell_sessions_ui,workspace/__tests__/p08_workspace_ui,publish/__tests__/p15_publish_ui,pending/__tests__/p18_pending_state}.test.tsx`;
docs: `docs/implementation/master_plan.md` (P66 tracker row only), `docs/implementation/plans/P66_approve_on_arrival_and_session_order.md` (Review log).
No temporary probe edits outside these.

## Checkpoint SHA
`bc9ba7b` (tree clean after the commit; this handoff is left for the runner to commit).

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace --no-fail-fast` (content of `bc9ba7b`) | exit 0, 928 passed, 0 failed |
| `cargo test -p psm-testkit --test p66_approve_on_arrival_and_session_order` | 6/6 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npm run typecheck` | exit 0 |
| `npm test` | 66 files, 452/453; the one failure is the known intermittent `p33_split_grid` C4(d), which passed 3/3 when run alone |
| first full run, before the fixes (for the record) | 18 failures: expected arrival/table changes (p01, p05, p27), the mirror-at-arrival problem (p49, p17; fixed by assumption 2), p62 fixtures (fixed as above), p43 c7b (load) |

## Manual try-out
1. Open the app and open a session (or create one with N); make it the active session.
2. Shoot 3 photos (camera or Capture One's capture). Each appears on the stage and in the panel as Main 01, 02, 03 straight away.
   In Capture One each gets the green tag once its preview is ready.
3. With a photo on stage press **C** (or swipe up). It moves to Condition and the next photo opens.
4. Press **⌫** (or swipe left) on another photo. "Photo denied · Undo ⌘Z" appears. Press ⌘Z: the photo comes back in its place.
5. Deny it again and publish the session (or wait 24 h). Within a few seconds after the publish completes, the RAW is gone from the
   item's capture folder, and Capture One shows that variant as missing. ⌘Z no longer brings it back. The published product has only
   the other photos.
6. Go back to the sessions list: sessions stay in the order you created them (newest on top), whatever you opened, edited or
   published since. The time filter (sliders icon) filters by creation day.

⚠ OWNER DECISIONS REQUIRED (0) — none.
