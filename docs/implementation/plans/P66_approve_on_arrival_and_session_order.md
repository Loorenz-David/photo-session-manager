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
