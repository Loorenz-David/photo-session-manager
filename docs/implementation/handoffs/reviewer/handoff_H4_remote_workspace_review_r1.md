---
plan: H4_remote_workspace
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-23
actor: opus (pipeline-reviewer, Opus 5.5)
---
# H4_remote_workspace: review r1

First review of the milestone (3 units, 14 criteria, 69 rows). The process authority is flow.md: no mutation probes, blocking only for B1–B4.

**Gate.** The tracker read `IMPLEMENTED`. Checkpoints `16606c1` (U1), `e9dd489` (U2), `264133a` (U3) and `b19474a` (close) are all in `git log`. Gate passed.

## Verified correct

**Perimeter.** `git show --stat` of each checkpoint equals the handoff's per-unit "Files changed" list, with nothing extra and nothing missing. The handoff declared three files outside the plan's lists, each forced by the change: `crates/testkit/tests/suite/c3_conversations_answer.rs`, `crates/testkit/tests/suite/p01_storage_identity.rs` and `apps/desktop/src/features/sessions/usePictures.ts`. `Header.tsx` and the 12 existing App/Workspace UI test files named in U3 Notes are untouched. In `p62_retention_after_publish.rs` only `c1b` and `c4a` changed (`a.remote().set_keep_files`), along with `c3a` (view, then `run_pending`, then view; fetches compared as a set; the URL derived from the media id because `RemotePictureView` carries none) and `c3b` (confirmation accepted, `details_only` asserted). No other pre-existing navigation or UI test assertion changed.

**Checks (Review mode, run once on HEAD, whose code equals `b19474a`):** `h4_remote_workspace::` 18 passed · `h4_details_only_publish::` 18 passed · `p62_retention_after_publish::` 12 passed · `cargo test -p psm-desktop` 92 passed (incl. `h4_tests` 4) · vitest over workspace, shell, sessions, publish, conversations and pending: 364 passed · `npm run typecheck` clean · `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-desktop -p psm-testkit --all-targets -- -D warnings` clean · `cargo fmt --all -- --check` clean. The implementer's full-suite stamp (workspace 1334 + p01 fix, npm test 849) stands.

**Read-checks the plan's Review mode names:**
- **L1 (activation):** `SessionsService::activate` returns `Ok(())` for `Remote(_)` before `editing`/`health`/`store.session`/`log.step`/`set_target`. `claim_target(Some(id))` returns `Ok(false)` for `Remote(_)` before `store_target`. `session_activate` builds both service instances with `.with_remote(remote)`. The state source is `with_remote` when attached, else `remote_state(store.remote_input)` when commerce is set, else `Local` (H6).
- **L3 (announcing a failed read):** `run_pending` emits `SessionChanged` only when `check_failed` turns from false to true (`failed_before` is read before `ensure_record`, and `note_check` returns the insert result). A successful read clears it.
- **No downloads in reads:** `RemoteItems::view` reads the store and pushes onto the queue. Its new `View` downloads replace only that session's not-started ones, and gone pictures are never queued. `session_remote_get` calls only `view`. `session_remote_refresh` is a press (`refresh`, then `view`).
- **Details-only run:** `details_only` is decided in `preview` (an `Unknown` item is read first via `ensure_record`; a failed read is logged and previewed as today), with no `item_get` media read. `confirm` drops the P62 refusal and skips the in-transaction photo comparison for it (L4). It is persisted (migration 021 is an `ALTER TABLE`; the store reads and writes the column) and read from the stored row on every run. `prepare`/`upload` return at once (no export, no links, no P69 wait). `update` skips `update_gallery`. `update_details` removes the condition key and every media-list key from the write set and skips the condition block. `export_run_id` stays `None`, so no swap happens.
- **Capture target:** `useCaptureTarget` is still the only caller of `activateSession`/`deactivateSession` (grep over `src` outside the tests). It activates once per (target, becomes-activatable) and reads the remote state of `captureTarget(nav)`, keyed by session id, sharing one instance when the target is the session on screen. `VisitBar` hides Shoot here instead by the same `activatable` rule.

**Other production code read against the rows:** `SessionsService::view` gives a remote session no photos, and so no upload states or compositions. `link_preview` uses `remote::local_picture` (M-H4). The attribute guard is `item_is_remote` (cleared, or `Remote(_)` when commerce is set). `download` skips an existing file (L19). `cover_media()` covers `View`, so view pictures get the stale rule and the chunked URL refresh. `RemoteCover` gallery and condition counts come from the same single `remote_covers` query. In `list`, a remote row's counts and status input follow MC-H3.7 (`ready_session_ids` untouched), and the label lets waiting, publishing, failed, cancelled and description-failed labels show through, else `Archived`; archived rows keep it without `with_remote`. `WorkspacePage`'s gate replaces classify, reorder, move, copy, templateDrop, remove, templateBack, templateShiftBack and patchPhoto, and guards `enterCompose` and every `onMenu` action (so split/normal/compose/apply/deny/move/copy/assign never reach `setKind`, `setComposition` or a picker). `FormDialog` gets the session view's photos (L10). `setPreviewFocus` gets `null` for a negative id (L12). `remotePhotos` matches task 2, with a non-null rect and frame. `PublishPopover` shows the details-only line per item and `Publish details` only when every item is details-only (L21).

**Rows.** All 69 rows (72 tests, counting C13(c) ×2 and C13(e) ×3) trace to a test at the row's boundary, and each asserts the exact outcome. No orphan tests. Spot judgments:
- H4·C2(c) runs through the command on `with_camera_and_shop()`, whose direct camera adapter has `depends_on_editing_provider: false`. Without the `claim_target` guard, `active_session_id` would become A and the test would fail.
- H4·C2(f)'s `arrival_photo` ignores its session argument and files by the active target, so the test discriminates.
- H4·C1(i) would count 2 if every failure emitted.
- H4·C6(a)/(c)/(d) assert the full media-mutation set (6 op kinds) is empty for the run.
- H4·C7(a) asserts the written keys are exactly `[custom.material]`.
- H4·C13 asserts the whole ordered activate/deactivate list.
- H4·C9(c)'s fixture sets `dirty` through the store, because an attribute edit never moves `change_seq` (attributes MC-4/MC-6, verified in the module header). The row's outcome holds (plan lesson, backlog).
- H4·C5(d) counts exactly the fake readback's own `item_get`.

**Assumptions 1–8.** None contradicts 10H §6–§7 or MC-H3:
1. A failed `getRemote` is treated as `local`. This is safe for the capture folder, because the backend's MC-H3.4 guard absorbs the activation; the UX gap is a note.
2. `check_failed` per session.
3. Cleared still refuses media lists.
4. String composition defaults.
5. Log line.
6. Refresh in the header ⋯ menu only.
7. "0 of N" before the first event.
8. `details_only` omitted from the JSON when false.

All are reported in plain words in the narrative.

**H1–H8.**
- H1: a details-only publish still runs under a confirmed run; view, refresh and downloads call only read methods (C1(h)).
- H2: no RAW is touched.
- H3: no new pixel code.
- H4: the flag is read from the persisted row on every resume (C6(b), C6(c)).
- H5: Capture One is not called for a remote item.
- H6: without commerce, every state is `Local`.
- H7: no new platform literal in core.
- H8: commands are thin `spawn_blocking`; core has no Tauri use.

The guide was updated (01, 02, 03, 04) for the new commands, the details-only run, the capture-target gate and `features/workspace/remote/`.

## Findings
None blocking (no B1–B4).

## Notes written to backlog
Six notes are in `docs/implementation/backlog_archive/H4_remote_workspace.md` (the plan's Review mode destination), with a pointer line in `docs/implementation/backlog.md`:
1. **Import on a remote product's workspace:** the panel's Import and a Finder drop go to the capture target (the previous session or Unassigned), while the overlay says "into this session". The gate does not list import.
2. **Assumption 1's UX gap:** a failed `getRemote` opens an empty ordinary workspace with no note and no Try again.
3. **Plan lesson (C9(c)):** an attribute edit never makes a session dirty, so "dirty after a details edit" rows are reachable only by data.
4. **Publish edge:** `confirm` recomputes the preview, so a bring back between a details-only preview and Confirm freezes a media publish. It is harmless today, because brought-back photos are linked as uploaded (M-H3).
5. **Plan hygiene:** U1 listed `integrations/src/lib.rs` (changed in U2) and U2 listed `services/storage.rs` (unchanged). The three forced out-of-plan files are recorded.
6. **Carry-forward of H3's notes routed to H4:**
   - second press: the UI path is closed by the dialog;
   - arrival during a bring back: closed by design, because a remote session is never the target before bring back;
   - `leave_gone(item_id)?` after the commit: still open, with no later archive-track phase to take it.

**Carry-forward dispositions:**

| Note | Destination |
|---|---|
| 1 (import on a remote product) | the next phase touching the workspace or import (orchestrator's choice) |
| 2 (failed `getRemote` message) | same as 1 |
| 3 (plan lesson) | planner / orchestrator |
| 4 (publish edge) | backlog, revisit if bring back's link writing changes |
| 5 (plan hygiene) | none needed |
| 6c (`leave_gone`) | the next phase that changes `services/bring_back.rs` |

The H1 note about a condition reference with no file node goes to the orchestrator's live check after H4, already in the plan's Manual try-out.

**Lessons for the plans:**
- A row whose input is "an edited attribute (dirty)" contradicts attributes MC-6. Write it as "a dirty session (by a photo change)", or drop "dirty".
- The U3 gate list enumerates edits of existing photos but not import. 10H §6 is silent on import into a remote product; the book or the next plan should say where it goes.

**Probe declaration.** No production or test file was edited, even temporarily. No test was re-run with a changed input. No database or state outside the test temp dirs was touched. My write perimeter:
- `docs/implementation/backlog_archive/H4_remote_workspace.md` (new);
- `docs/implementation/backlog.md` (one appended line);
- `docs/implementation/plans/H4_remote_workspace.md` (Tracker line and one Review log entry);
- `docs/implementation/narratives/H4_remote_workspace.md` (new);
- this handoff.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
