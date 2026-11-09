---
plan: P58_fast_compose_on_camera_preview
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-09
actor: opus (pipeline-reviewer, Opus 5.5)
---

# P58 re-review r2: APPROVED_WITH_NOTES

Both r1 blocking findings are resolved as their corrections said:
- F1: the camera-preview branch of `canCompose` now requires `frame_source === 'camera'`.
- F2: C3(b) now delivers its two arrivals through the capture hook.

The suite is green apart from two load-sensitive timeouts:
- `p33_split_grid` C4(d), which the plan names as a known intermittent test;
- `p43_ftp_receiver` c7b, which only failed in a killed first run.

Both pass when run alone. Reading the changed seam surfaced one new B1-class item (R2-1): for a *matching* photo, an export master that lands first turns Compose off until Capture One's preview arrives. Triaged **DROP** (§Triage) and written to the backlog. Tracker → `APPROVED`; narrative written.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
- **r1 (CHANGES_REQUESTED)** settled everything except F1 and F2:
  - extractor and frame decision;
  - camera frame only on a match;
  - swap keeps the composition;
  - C1(a), C1(c), C1(d) and C2(a);
  - render queue and focus chain;
  - H1–H8;
  - assumptions 1–8.
- **r2 (this round)** covers only the fix delta and its seam.

## Perimeter check
- Gate: the tracker row was `IMPLEMENTED` with a fix note, and checkpoint `7549fc8` exists.
- `git show --stat 7549fc8` lists 17 paths. Every code and test path is in r1's allowed lists:
  - **F1:**
    - `commands/compositions.rs`
    - `bindings.ts`
    - `services/compositions.ts`
    - compose `harness.tsx`
    - the P58 UI test
    - fixtures that must now name the field: `machine.test.ts`, `p11_form_ui`, `p19_navigation`, `p08_workspace_ui`, `p19_drop`, `p20_drop_continuity`, `p22_workspace_tidy`, `p26_double_drag`. Each is a one-line `frame_source` addition, which the DTO type forces.
  - **F2:** `crates/testkit/tests/p58_fast_compose_on_camera_preview.rs`.
  - **Docs:** the fix handoff, the master-plan tracker row and the plan Review log.
  - Nothing outside the perimeter.
- HEAD `38494af` differs from `7549fc8` only in `docs/` (the handoff frontmatter and the r2 prompt). `git status --porcelain` was clean at review entry.

## Evidence (tree `38494af`, clean; code identical to `7549fc8`)
| Command | Result |
|---|---|
| `cargo test --workspace --no-fail-fast` | 858 passed, 0 failed (57 result lines) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | 418/419. Red: `p33_split_grid` C4(d) (expected `Main 4–5 of 6`, got the strip's previous text). This is the intermittent test plan row C4(a) names; `npx vitest run …p33_split_grid.test.tsx` → 7/7. |
| First full run (killed at its 600 s limit, load ≈ 4) | `p43_ftp_receiver` c7b failed on "condition did not settle within 15s". `cargo test -p psm-testkit --test p43_ftp_receiver` alone → 26/26. The fix touches nothing on the FTP path. The full re-take above was green. |

Why there were two full runs: the first was killed before it finished, so it was not a valid stamp. The re-take is not a repeat run on a tree that already had a stamp.

## Findings resolved
### F1 (B1): resolved as corrected
- `CompositionDto.frame_source = c.fingerprint.map(|f| f.source.as_str())`. This is `camera`, `preview` or `master`, and null exactly when there is no frame (frame and fingerprint are written together in `adopt_frame_tx`).
- `canCompose = rect ∧ frame ∧ (adjusted ∨ (original ∧ frame_source == 'camera'))`.
- Every Compose entry goes through it:
  - stage button and hint (`Stage.tsx`);
  - photo menu, including Normal/Split and Compose… (`PhotoMenu.tsx`);
  - R / double-click (`WorkspacePage.enterCompose`);
  - the editor's own guard and switch-photo (`WorkspacePage` effect, `ComposeMode`);
  - grid Normal/Split (`setKind`).
- `composePreview`, and through it `editorPicture` for the editor stage, output previews, split previews and loupe, calls `canCompose`.
- So the r1 reproduction (non-matching original, frame from `master`, no adjusted preview) now leaves Compose disabled with the waiting sentence.
- The new C1(b) UI case asserts exactly that. It fails if `frame_source` is ignored, because the r1 tree's rule returns `true` for that input; the implementer also showed it red by flipping the input to `camera`.
- The existing C1(a), C1(c) and C1(d) fixtures name `camera` and still pass.
- The shared harness default `preview` keeps every pre-P58 compose test on Capture One's preview.

### F2 (B2): resolved as corrected
- The `captured` helper writes a matching synthetic RAW into the capture folder and delivers `capture_done` through the hook. It then lists Capture One's variant and ticks, so the first request comes from the production path (`request_initial` / `wait_for_variant`).
- If captures were not marked as arrivals, `new-1` and `new-2` would queue behind `old-2…old-10`, and the first assertion (`new-2`) would fail.
- A broken newest-first order would fail the first assertion too.
- The backlog fixture still uses direct `request_render(…, false)`, which is correct: it is the input, not the subject.

## Triage (flow.md §5)
| Item | Class | Why |
|---|---|---|
| R2-1: an export master that lands first moves a matching photo's frame source `camera` → `master` and turns Compose off until the preview arrives | **DROP** → backlog | Not a wrong outcome: the user sees the pre-P58 waiting state with a true sentence. Nothing wrong is displayed, saved or exported; the stage and tile keep the composed camera picture, because they read frame + values, not `canCompose`. It heals when the preview lands, and the stage photo is rendered next (focus). The real fix needs a new persisted fact ("camera preview matches") across extractor, store and DTO, which is beyond a fix-round correction for a transient loss of a speed-up. |

No MUST items, so the verdict is **APPROVED_WITH_NOTES**.

## Remaining findings
### R2-1 (B1-class, DROP; route: production): an export before the first preview turns Compose off for a matching photo
- **What happens:**
  - A matching (a7R V) photo has a `camera` frame and no adjusted preview, and the photographer exports.
  - Masters bypass the one-at-a-time preview lane, so the master can land before Capture One's first preview.
  - `adopt_frame_tx`'s provisional branch (`store.rs`, "Capture One's frame replaces the camera preview's") rewrites `frame_source` to `master`, while `adjusted_preview_path` is still null.
  - `canCompose` is then false. Compose, Normal/Split and R show `Waiting for Capture One's full-frame preview`, and an editor open on that photo is closed by the `WorkspacePage` effect. Saves already queued survive; only a drag in progress is lost.
- **Before the fix:** the same state was composable, with the camera preview over the master's same-aspect frame.
- **Authority:** plan C1(a); changelog 39 ("composable when it matches the camera frame").
- **Correction for later:** have the MC-20 extractor persist "camera preview matches" on the photo, and gate on that rather than on the composition's current frame source.
- Written to `docs/implementation/backlog.md`.

### Seen in passing, not a P58 finding
- When a non-matching camera JPEG sits under a frame taken from a master, the stage and tile still draw that JPEG in the composed layout before the adjusted preview exists.
- This is the same `StageImage` code as before P58 (checked against `3b1351a^`), and it is display only. Not recorded.

## Lessons for the plans
- The r1 correction ("expose the frame source") named a **mutable** property as the signal for a **fixed** fact (does this photo's camera preview match). A correction that gates a feature on stored state should ask which later writers change that state. Here an export master does.

## Carry-forward dispositions
| Note | Destination |
|---|---|
| R2-1 (persist "camera preview matches") | backlog → orchestrator schedules (candidate: the next composition or preview phase) |
| r1 notes 1–4 (one preview lane timing, in-memory arrival and focus, orientation mismatch → review, registry and plan drift) | already in the backlog; the hardware items go to the next camera run, registry drift to the orchestrator |

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P58_fast_compose_on_camera_preview_review_r2.md` (this file)
- `docs/implementation/plans/P58_fast_compose_on_camera_preview.md` (Review log entry appended)
- `docs/implementation/master_plan.md` (tracker row P58 only → `APPROVED`)
- `docs/implementation/backlog.md` (1 note appended)
- `docs/implementation/narratives/P58_fast_compose_on_camera_preview.md` (new)

Other effects:
- **Production code and tests:** none touched, and no probes.
- **Scratch outside the repo:** suite logs `/tmp/p58r2_*.log`.
- **Commits:** none.
