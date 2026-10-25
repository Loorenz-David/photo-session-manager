---
plan: P07_shell_sessions_ui
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-25
actor: opus
---
# P07 shell, sessions list and new session — re-review r2 (after fix 1)

**Verdict: APPROVED_WITH_NOTES.** F1 (the only blocking finding from r1) is resolved using r1's first offered
correction. The fix stayed inside its allowed files. Suite, lint and typecheck are green. No blocking item remains,
so the triage table is empty. I added one note to the backlog: the book's S-A wording should absorb the
relaunch behaviour.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Review history

r1 settled all 19 criterion rows (C1–C6), assumptions 1–14, H1–H8 and the test set, and wrote 6 notes to the backlog.
Its one blocking finding was F1 (B1): after a relaunch the UI showed no open session while the backend still captured
into the last one. This round re-verifies only the F1 seam.

## Perimeter check

- Gate: tracker row `IMPLEMENTED` with fix note; checkpoint `7f1a0c0ce28c0a2f732118f0dad4d2a9982ae820` is in `git log`.
- `git show --stat 7f1a0c0` lists 11 paths. 8 are code or test files, and all are in F1's allowed list:
  `apps/desktop/src/App.tsx`, `apps/desktop/src/services/sessions.ts`, `apps/desktop/src/services/bindings.ts`
  (one generated line), `apps/desktop/src-tauri/src/commands/sessions.rs`, `apps/desktop/src-tauri/src/lib.rs`
  (one registration line), `crates/core/src/services/sessions.rs`, and both P07 test files. The other 3 are the
  fix handoff, the P07 tracker row only, and the plan's Review log. `editing_startup.rs` is untouched, as the fix
  declared. No finding.
- `git diff 7f1a0c0 HEAD`: only `prompts/reviewer/P07_shell_sessions_ui_review_r2.md`, so the code tree equals the
  checkpoint. `git status --porcelain` was clean at entry.
- Stamp. Authorization was recorded before the run: the prompt directs one run, and this is the approval gate.
  Code tree = `7f1a0c0`, Node v22.22.3, cargo from `~/.cargo/bin`:
  - `cargo test --workspace`: 182 passed, 0 failed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0.
  - `cargo fmt --all -- --check`: 0.
  - `npm run typecheck`: 0.
  - `npm test`: 19/19.

  These match the fix handoff's stamp.
- No production file was edited, not even temporarily (flow §4). No probes were run, no state was touched, and
  there is no archgraph delta.

## Findings resolved

### F1 (B1) — resolved

- **Backend.** `SessionsService::active()` returns `store.active_session_id()`. MC-2 arrival uses this same value
  to attribute frames (`services/arrival.rs`), so "what the UI opens" and "where captures land" come from one source.
- **Command.** `session_active` is thin. It reads the id, then calls `get(id)` and converts it to `SessionResponse`.
  It returns `Option<SessionResponse>` because `CommandId` is deserialize-only. That divergence is sound: same
  information, no new DTO.
- **Launch.** `useResumeActiveSession` runs once:
  - `navigate` is a `useCallback(…, [])`, so the effect does not re-run.
  - StrictMode's double run is guarded by `cancelled`.
  - It redirects only when the hash is still `""` or `#/` at resolve time, so a session or health route the user
    picked first is kept.
  - After the redirect, `useCaptureTarget(id)` calls `session_activate(id)` again, which only re-applies the same
    folder.
  - Nothing is written, so it cannot race `EditingStartup`'s background re-apply. That closes the race r1 named.
- **Leaving.** Going back to Sessions still calls `session_deactivate`, because `active.current` is set after
  activate resolves (S-E).
- **Odd timing.** The user may open session X and return to Sessions before the launch read resolves. The read then
  returns A or X, the app navigates there, and activates it. Screen and capture target still agree, so there is
  no wrong outcome.
- **Error path.** If the read fails, a toast appears and the list stays. Today this can only come from a store
  error, because sessions cannot be deleted (no delete path exists). Not reported.
- **Tests (B2).** One check per test:
  - F1(a): `activeSession → {id: 9}` makes the hash `#/session/9` and shows the view, with no deactivate call.
    Without the hook the hash stays `#/` and the test fails.
  - F1(b): with nothing active, the app stays on `#/` and shows the empty list.
  - The Rust test `f1_active_reports_the_stored_active_session` checks `None`, then the stored id.

  All tests sit at the services boundary, and none touches internals or fails the over-engineering test.
- **Divergence: the relaunch resumes the open session's view.** This differs from the literal S-A reading
  ("app opens on the Sessions list") only when a session was open at quit. It is sound and not B1:
  - r1 offered this exact correction.
  - S-A describes what the list shows, not a launch-route rule.
  - S-E and ch06 §arrival require the screen and the capture target to agree.
  - The list is one click away, and that click switches captures to Unassigned.

  I wrote a backlog note so the book can absorb it. The narrative tells the owner how to ask for the other
  option.

## Triage

| Item | Class | Reason |
|---|---|---|
| (none) | — | No blocking item remains after fix 1. |

## Remaining findings

- **Blocking:** none.
- **Notes (backlog):**
  1. P07 → book 04: the S-A wording vs relaunch resume (above). This is a recommendation for the intention, not a
     defect.
- r1's 6 backlog notes stand unchanged (P12 ×1, P08 ×2, P16 ×1, P08/P16 ×1, P07 ×1).

## Carry-forward dispositions

| Note | Destination |
|---|---|
| MC-22 proposal advance on shop hit; `origin ≠ local` provisional | P12 |
| Header reads type/SKU via `sessions_list(all)`, no refresh | P08 |
| Unassigned count includes deleted photos | P08 |
| Theme in `localStorage` | P16 |
| Raw backend error messages in toasts | P08 / P16 |
| Filter counts ignore time filter | P07 (accepted, documented in narrative) |
| S-A wording vs relaunch resume | book ch04 (orchestrator/owner) |

## Lessons for the plans

- When a fix can choose between two corrections that change user-visible launch behaviour, the finding should say
  which one the book prefers. Otherwise the narrative has to carry the choice to the owner after the fact. This
  time it was cheap, because both options were consistent with S-E.

## Write perimeter

- `docs/implementation/handoffs/reviewer/handoff_P07_shell_sessions_ui_review_r2.md` (this file).
- `docs/implementation/narratives/P07_shell_sessions_ui.md` (new).
- `docs/implementation/plans/P07_shell_sessions_ui.md` (Review log entry).
- `docs/implementation/master_plan.md` (P07 tracker row → `APPROVED` only).
- `docs/implementation/backlog.md` (1 row appended).

No code was edited, no probes were run, there is no archgraph delta, and nothing was committed.
