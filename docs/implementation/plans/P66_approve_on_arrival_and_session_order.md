---
phase: P66_approve_on_arrival_and_session_order
title: Photos are approved on arrival (the photographer marks Condition or Deny instead; Deny deletes the RAW after the undo window), and the session list stays in creation order, newest on top
difficulty: high
projection: no
depends_on: [P65_capture_sync_waiting_jobs_and_inventory]
---
# P66 — Approve on arrival, Deny deletes the RAW, stable session order (owner 2026-11-11, changelog 46 (1)(2))

Written by the orchestrator (Opus) on 2026-11-11 from the owner's first day at work with the P53–P64 build.
Owner: "when opening a session and then coming back to the session list the order has changed (last open appears at the top); I need this to remain in
the order of creation"; "all pictures that arrive should be approved, and I mark deny or condition instead … that way I can re-order them directly and if
perfect shooting I can publish directly; the approval of the picture blocks a lot of the flow."

## Goal
1. **Approved on arrival (06).** Every new photo joins its session as `main` at the end of Main (arrival order), mirrored green to Capture One. The review
   actions become **Condition** (C, swipe up) and **Deny** (⌫, swipe left); a condition photo keeps "Move to Main"; a denied photo can be restored by undo.
   The "next unreviewed photo opens" flow no longer applies to arrivals; after an action the next photo in order opens. Photos that are already
   `unreviewed` stay so and keep today's actions (A still approves them). `ready` (07) and publish (P64) keep working: a session whose photos all arrived
   approved is Ready as soon as previews exist.
2. **Deny deletes the RAW (01 H2 amended).** A denied photo is hidden and never published. Its RAW (and its app-owned derivatives: previews, master,
   thumbnail) are deleted when the undo window closes: when the session's next publish completes, or 24 h after the deny, whichever is first (run by
   the P61 storage job, logged `storage.cleared rule=denied path=… bytes=…`). Undo before that restores the photo fully. After deletion the photo row
   stays as a denied, file-gone entry (no undo; the undo stack entry is dropped). Never deletes a RAW of a photo that is not denied, and re-checks the
   photo is still denied at delete time. Capture One shows the variant as missing (10F Q4 precedent).
3. **Session list order (04).** The list is sorted by creation time, newest first, inside every group; time groups and the time filter use the creation
   time, not last activity. Opening, editing, publishing or capturing into a session never moves it.

## Read first
Changelog 46, 44 (P64 unreviewed → published), 43, 40–41 (10F, storage job); book 06, 07 §Status, 01 H2, 10F. Code: `crates/core/src/services/{arrival.rs,
photos.rs, review*.rs, storage.rs, sessions.rs}`, `crates/infra/src/sqlite/store.rs` (photos ORDER BY classification; sessions list), mirror
(`crates/integrations/src/editing/capture_one/mirror.rs`), `apps/desktop/src/features/{workspace, sessions}` (review keys/swipes, SessionsPage grouping by
`last_activity_at`). Tests: p08, p19, p22, p26 UI tests, p44, p61, p62, p64 testkit files.

## Files expected to change (write perimeter)
The files above as needed; `crates/testkit/tests/p66_approve_on_arrival_and_session_order.rs` (new); a workspace/sessions UI test (new). ≈ 18 paths, ≈ 500 lines.

## Criteria (5 criteria, 7 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | approved on arrival | 3 photos arrive in an active session (FTP, import) | all `main`, positions 1–3 in arrival order, mirrored green; session Ready once previews exist | `[rule]` changelog 46 (2) |
| C1(b) | existing unreviewed untouched | a session with 2 `unreviewed` photos from before; a new arrival | the 2 stay `unreviewed`; the new one is `main` | `[rule]` changelog 46 (2) |
| C2(a) | deny then undo | deny a photo, undo within the window | photo back at its position; RAW never touched | `[rule]` changelog 46 (2); MC-30 |
| C2(b) | deny deletes after the window | deny; (i) session publish completes; (ii) 24 h pass (fake clock) without publish | RAW + derivatives deleted, `rule=denied` logged; never published; a non-denied photo's RAW untouched; a photo restored before the delete is kept | `[rule]` changelog 46 (2); H2 amended |
| C3(a) | session order stable | sessions A, B, C created in that order; open A, edit B, capture into A | list order C, B, A in every view; time groups by creation time | `[rule]` changelog 46 (1) |
| C3(b) | UI | workspace keys/swipes | C → condition, ⌫/swipe left → Deny, no approve step for arrivals; the list test as C3(a) | `[rule]` changelog 46 |
| C4(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (known intermittent `p33_split_grid` aside), clippy, fmt, typecheck green | `[rule]` master plan §7.2 |

## Notes
- **Hard rules:** delete only RAWs of denied photos, only after the window, only with a re-check; tests use temp folders; the suite never needs Capture
  One, the camera or a shop. Default implementer (Codex; Claude when Codex is exhausted). Update book 06/01 wording only if the implementer is asked to
  (orchestrator owns the book).

## Review mode (owner 2026-11-10: fast; kept)
- **Implementer:** high effort; run the full closing gate once (`cargo test --workspace`, clippy, fmt, typecheck, `npm test`) and put the stamp in the handoff.
- **Reviewer (time-boxed, still correct):** do not re-run `cargo test --workspace`. Run this phase's new test file(s), the test files of the crates and features the diff touches, clippy, fmt, typecheck and the touched UI test files; trust the implementer's full-suite stamp unless it is missing or contradicts what you see. Read every changed production path against the rows; only B1–B4 findings block. Aim to finish in about 15 minutes.

## Review log

### 2026-11-11 — implement r1 (Claude Opus, `codex` actor slot)
**Built.** (1) Arrival (`ArrivalService::accept` → `Store::arrival_insert(…, Classification::Main)`) stores every new photo
(capture, FTP, import, Unassigned too) as `main` at the end of Main; the green tag is queued when Capture One delivers the RAW's
first adjusted preview (`PreviewsService::mirror_arrived`). Photos already `unreviewed` are untouched; A still approves them.
(2) Deny = the existing delete path plus a deny mark (`photo_denials`, migration 012; set by `ReviewService::delete` and by a redone
deny, removed by any restore through `set_photo_review`). The storage job (P61) looks at pending denials every batch pause and, when the
window has closed (session `published_at` > deny time, or 24 h — `DENY_UNDO_WINDOW_MS`), re-reads the photo under the undo-history lock
(`History::retire_photo`), checks every other photo on that RAW is denied with its window closed too, deletes the RAW and the RAW's
`previews/`, `masters/`, `outputs/` folders' files (rule `denied`, logged `storage.cleared rule=denied path=… bytes=…`), marks the
denial `files_deleted_at`, and drops every undo/redo step naming the photo. A file-gone denied photo cannot be re-classified.
(3) Sessions list: `group_by_activity` is fed `created_at` (order newest-created first, groups by creation time); `SessionRow` /
`SessionRowResponse` gain `created_at`; the UI time filter, the column header ("Created ↓") and the row time use it. UI: Deny (⌫, swipe
left, ⋯ menu "Deny"), toast "Photo denied · Undo ⌘Z", Approve button only on an unreviewed photo, after an action the next photo in the
display order opens.

**Assumptions.** A1 Unassigned arrivals are `main` too ("every new photo"). A2 the green tag is mirrored after the first adjusted preview,
not at the instant of arrival (a tag job at arrival polled Capture One on its own and scripted a stopped Capture One — p49 C2(d)). A3
photos deleted before P66 (no deny mark) keep their RAW (H2 as it was); only denies made from P66 on are deleted. A4 Capture One's own
files of a denied RAW (`CaptureOne/Settings*/…cos`, cache) and `.duplicates/` copies are not deleted (the row names RAW + app-owned
derivatives; Capture One shows the variant as missing). A5 a RAW shared with another photo row (Copy to session) is deleted only once
every row on it is denied with its window closed. A6 the history labels keep the registry's wording (`Delete 03`), the buttons say Deny.
A7 the session row's time shows the creation time, matching the order and the groups. A8 the window closes on a publish completing
*after* the deny (strictly later `published_at`); a publish already running when the photo was denied also closes it when it completes.
A9 pinned items ("keep files") are not exempt: Deny is the photographer's explicit choice.

**Deviations.** Test files outside the new ones, changed because the behaviour they asserted changed: p01 (table list +`photo_denials`),
p05 c5a/c8a (arrival is `main`), p62 (its hidden-photo fixtures now represent photos hidden before P66, by removing the deny mark), UI p07,
p08 (toast text, `created_at`), p15, p18 (fixture `created_at`). Production files outside the listed ones: `crates/core/src/domain/`
`{session,storage}.rs`, `crates/core/src/ports/store.rs`, `crates/core/src/services/{history,previews}.rs`, `crates/integrations/src/lib.rs`,
`apps/desktop/src-tauri/src/commands/sessions.rs`, `apps/desktop/src/services/{bindings,photos}.ts`, migration 012.
