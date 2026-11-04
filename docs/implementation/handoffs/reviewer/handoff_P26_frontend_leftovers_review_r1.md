---
plan: P26_frontend_leftovers
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-04
actor: opus
---
# P26 review r1: APPROVED_WITH_NOTES

## Verified correct
- **Gate.** The tracker showed `IMPLEMENTED`. Checkpoint `791446e` (`CHECKPOINT (not approved): implement P26 frontend leftovers`) is in `git log`.
- **Perimeter.** `git show --stat 791446e` lists exactly the 7 declared paths: `SettingsPage.tsx`, `useWorkspaceState.ts`, the two new
  test files, the P26 tracker row, the P26 Review log and the implementer handoff. Nothing extra, nothing missing. No bindings change.
- **Suite (reviewer run on clean `033791e`, the checkpoint plus the review prompt commit).** `cargo test --workspace` 457 passed, 0 failed.
  `cargo clippy --workspace --all-targets -- -D warnings` clean. `cargo fmt --all -- --check` clean. `npm run typecheck` clean.
  `npm test` 20 files / 131 passed.
- **C1(a).** `SettingsPage.tsx:92` renders `Effective path: {settings.document_id || settings.default_document_id}` over the edited draft
  `settings`. `default_document_id` comes from `SettingsFile::default_document_id` (`crates/infra/src/settings_file.rs:62`), which is the
  same fallback `effective_document_id` uses (`:70`). So before a save, the line names the document the app will actually use (10C §Setting,
  ch12 §Settings). The `~/Pictures/Session Manager` literal and the `page.data?.document_id` branch are gone. The name line, the buttons
  and the save patch are unchanged, and P24 C5(a) still passes. The test asserts the exact `Effective path:` text, no `~/` anywhere, the picked
  folder after the second click, and `saveSettings` never called. It fails on the old literal and on an `effective_document_id` fallback
  (which would still read `/Users/me/Pictures/Other` after the click).
- **C2(a).** `run`'s `finally` decrements `commandsInFlight` first. At zero it clears `reloadWanted` and awaits one `reload()`. Otherwise
  it sets `reloadWanted` and does not reload. Case table checked by reading: in-order settle (reload only at the second), out-of-order settle
  (reload only at the last), a failing command (same path, toast kept), a single command (one reload, as before), an event during a command
  (deferred, then served by the closing reload). An event that arrives during the closing reload now reloads again, and the ticket keeps the
  newest answer. `undo` keeps its own reload. The test pins the call order `(1, 3)`, `(2, 2)` and the optimistic `[5, 2, 1]`. It serves a
  stale `original` before resolving `p1`, so an early reload would repaint `[1, 2, 5]`. It requires no new `getSessionView` call after
  `p1` and exactly one after `p2`. That count sits at the service boundary and the row requires it. Both "always reload" and "never reload"
  variants fail it.
- **Discrimination by variation.** I ran both new tests against `791446e~1` (the pre-change production) in a scratch `git worktree`. Both
  failed: C1(a) on the `~/` literal, and C2(a) with `expected [1, 2, 5] to deeply equal [5, 2, 1]` after `p1`. The worktree was then
  removed. No production file in the repository was touched.
- **Assumption.** "The last-settling command always reloads once and clears `reloadWanted`" is the plan's delegated choice. It is
  consistent with ch06 §Order and P19 R8, so it is not B1. It goes to the narrative.
- **H1–H8.** The change is frontend UI state only. There is no external write (H1), no RAW access (H2), no pixel work (H3), and no
  automation of other apps (H5). H6 is unaffected. No platform concept enters core (H7), and no business logic moves into Tauri or the UI
  (H8). No secrets and no live service calls.

## Findings
None blocking (B1–B4). No should-fix.

## Notes written to backlog
- **P26 (P22 origin):** `run` does not bump the `request` ticket when it applies the optimistic order. A reload already in flight when a
  drag starts (the initial load, or an event reload issued at zero) can still land and paint the pre-drag order until the command's
  own reload repaints it. The window is narrow. Fix: bump `request.current` in `run`.
- **P26:** `reloadWanted` is now write-only (nothing reads it). It is harmless dead state; remove it when the file is next touched.
- **P26 (test):** `p26_double_drag.test.tsx` lines 1–192 are a verbatim copy of the P22 harness. Extract a shared workspace test harness.
- **P26 (not verified live):** both rows were verified in jsdom only. The owner's try-out step is the only on-screen proof.
- Marked resolved: backlog rows "→ P26 · P22" (the closing reload replaces a second drag) and "P24 (UI)" (the `~/` detail line).

## Probe declaration
I edited no production file and ran no mutation probe (flow.md §4). The only variation was a scratch `git worktree` at `791446e~1` in a
temp directory, with `node_modules` symlinked and the two new test files copied in. It was removed with `git worktree remove --force`.
`git worktree list` shows only the main tree. There were no database, settings or Capture One side effects.

## Write perimeter
`docs/implementation/master_plan.md` (P26 tracker row only) · `docs/implementation/plans/P26_frontend_leftovers.md` (Review log entry) ·
`docs/implementation/backlog.md` (2 rows marked resolved, 4 rows appended) · `docs/implementation/narratives/P26_frontend_leftovers.md` (new) ·
this handoff. No commits (the runner commits).

## Lessons for the plans
- Rows that pin a "no reload yet" outcome work well when the stale answer differs from the optimistic order, as this row did by serving
  `original` before `p1`. Keep that pattern.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
