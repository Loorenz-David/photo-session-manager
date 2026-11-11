---
plan: P66_approve_on_arrival_and_session_order
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-11
actor: opus
---
# P66 review r1: APPROVED_WITH_NOTES

All 7 criterion rows hold at their boundary and each test fails on a wrong outcome. The 33 committed paths equal the declared
files. Touched Rust and UI suites, fmt and typecheck are green in the reviewer's run. Nothing blocks (B1–B4), and there are 3 backlog notes.

⚠ OWNER DECISIONS REQUIRED (0) — none.

## Gate and evidence
- Tracker `IMPLEMENTED`; checkpoint `bc9ba7b` present. Reviewed tree `3166dff`. Only runner doc commits follow the checkpoint, and `git diff bc9ba7b HEAD -- crates apps Cargo.*` is empty.
- Perimeter: `git show --stat bc9ba7b` lists 33 paths, the same set as the handoff's "Files changed".
- Reviewer runs (code tree = `bc9ba7b`):

| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test <file>` for p66, p01, p05, p17, p27, p39, p40, p44, p49, p61, p62, p64 | 6, 20, 33, 15, 11, 24, 18, 25, 21, 7, 12, 8 passed; 0 failed |
| `cargo fmt --all -- --check` | ok |
| `npm run typecheck` | ok |
| `vitest run` p66, p07, p08, p15, p18 + p19/p22/p26 UI files | 12 files, 87 passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | **not completed**: stalled twice at 0 % CPU inside `clippy-driver` (psm_desktop_lib, then psm_integrations on a desktop-less retry). Environment; killed. The implementer's exit 0 on `bc9ba7b` is consumed because the code tree is identical. |

- The full workspace was not re-run, per the plan's Review mode. The implementer's 928/0 stamp is consumed.

## Verified correct
- **C1(a).** Arrival inserts `main` at `COUNT(main)+1` inside the insert transaction, so photos keep arrival order. The green tag is queued on the RAW's first preview (`mirror_arrived`; it skips `unreviewed`). The test runs FTP, import, FTP and asserts `main` 1–3 in order, tag 4 on every variant, and `InProgress` → `Ready` once previews exist.
- **C1(b).** Old `unreviewed` rows are untouched (None position) and the new arrival is `main`/1.
- **C2(a).** Deny marks `photo_denials`. A restore through `set_photo_review` removes the mark while `files_deleted_at IS NULL`. Undo brings the photo back to `main`/2. Two days of storage ticks delete nothing, and the files are byte- and set-identical.
- **C2(b)(i)/(ii).** The window closes at `denied_at + 24 h`, or when `published_at > denied_at`. `published_at` is written only when a publish item completes (`complete_publish_item`). The delete runs under `History::operation` and re-checks that the photo is still deleted, marked and past its window. It also requires every other row on the RAW to be denied and past its window. Only the RAW and `previews|masters|outputs/<key[..2]>/<key>/*` are removed (thumbnails live under `previews/`). Each removal is logged `rule=denied`, the mark gets `files_deleted_at`, and the photo's undo/redo steps are dropped. A re-classify is refused afterwards. Publish's per-upload recheck skips `Deleted`, so a photo denied during a running publish is not uploaded. The tests check four things: nothing is deleted before the publish or at 23 h; the deleted set equals the denied RAW plus its files, each logged; the restored and other RAWs stay byte-identical, with 3 uploads; the undo top moves past the deny and a re-classify is refused.
- **C3(a).** The list orders and groups by `created_at`. The test opens A, edits B and captures into A, then asserts `[C,B,A]` under Open/All, with and without a query, and that A (created 40 days ago) is not "Today". The old `last_activity_at` order fails this test.
- **C3(b).** The UI test asserts the buttons `Deny`/`Move to Condition`. C, ⌫ and the swipes call the right service with the right id, the next photo in order opens, and the toast text is right. A on a main photo calls nothing, and Approve shows only on `unreviewed`. The list renders the delivered order, the row time is the creation time, and the "Today" filter uses the creation time.
- **C4(a).** See the evidence above. The implementer's full stamp is consumed.
- **Assumptions A1–A11.** None contradicts the book as amended by changelog 46. The owner layer lists them in plain words in the narrative.
- **H1–H8.** H2 is amended by changelog 46 for denied photos only. Deletion is limited and re-checked as above. No secrets and no live services in the suite; tests use temp folders.
- **Tests.** Every test in the two new files traces to a row. No implementation-coupled assertions.

## Findings
None blocking. No should-fix findings.

## Notes written to backlog
1. Deleting a denied photo's files drops whole undo/redo steps that name it, including multi-photo Classify/Move steps. The other photos in such a step lose their undo.
2. Book 06, book 01 H2 and the registry (§6.4 Store/StoreTx additions, §6.5 `photo_denials`/migration 012, §6.7 `SessionRowResponse.created_at`) still carry pre-P66 wording. The orchestrator owns both.
3. The green tag waits for Capture One's first adjusted preview (A2), not the instant of arrival. It is absent while Capture One is closed.

## Lessons for the plans
- A criterion that changes a hard constraint (H2) should name the book wording to update and who updates it, so the registry and book do not lag the code. Here that was left to "the orchestrator owns the book".

## Write perimeter
`docs/implementation/backlog.md` (3 lines appended); `docs/implementation/plans/P66_approve_on_arrival_and_session_order.md` (Review log entry);
`docs/implementation/master_plan.md` (P66 tracker row only); `docs/implementation/narratives/P66_approve_on_arrival_and_session_order.md` (new); this handoff.
No production or test file touched, no probes, no database state. Build artefacts under `target/` only.
