---
plan: P08_workspace_ui
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-25
actor: opus
---
# P08 workspace UI: re-review r2 (after fix 1)

**Summary.** Fix 1 resolves F1 exactly as the correction asked. A `photo_added` event for the open session now sets the
active photo through `setActiveId`, so the stage, A/C/⌫ and ←/→ agree on the new capture. The new test C9 can fail on the
old behaviour. The perimeter is clean, and suite, lint and typecheck are green. No blocking item remains, so the triage
has no MUST and no DROP. One new note (milliseconds-long ordering windows in the same handler) goes to the backlog.
Verdict: **APPROVED_WITH_NOTES**.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
- **r1** (CHANGES_REQUESTED) settled all 22 criterion rows plus candidate C0, H1–H8, the asset-protocol scope, the
  assumptions and the r1 stamp. One blocking finding: F1 (B1).
- **This round** re-verifies only the changed seam: `onWorkspaceEvents` → `setActiveId`/`reload` in `useWorkspaceState.ts`,
  plus the new test.

## Gate
- The tracker showed `P08_workspace_ui | IMPLEMENTED | … | sonnet | fix 1: F1 resolved …`.
- The fix checkpoint `545d619` (`CHECKPOINT (not approved): P08 fix 1 …`, parent `2aa4ac2`) exists.
- HEAD `8762dfd` differs from it only by the re-review prompt.
- `git status --porcelain` was clean at entry.

## Perimeter check
`git show --stat 545d619` lists 5 paths, and they equal the fix handoff's "Files changed" list:
| Path | Allowed by |
|---|---|
| `apps/desktop/src/features/workspace/useWorkspaceState.ts` (+2/−2) | F1 files list |
| `apps/desktop/src/features/workspace/__tests__/p08_workspace_ui.test.tsx` (+16) | F1 files list |
| `docs/implementation/master_plan.md` (P08 row only) | F1 files list (tracker row) |
| `docs/implementation/plans/P08_workspace_ui.md` (Review log append only) | F1 files list (Review log) |
| `docs/implementation/handoffs/implementer/handoff_P08_workspace_ui_fix_r2.md` (new) | F1 files list (fix handoff) |

Nothing is outside the perimeter. The fix declares no probe files: its red check used `git stash` on the one production file,
and the committed diff shows no residue.

## Stamp (L4, run once as the prompt directs)
Tree: `8762dfd`, clean (the content of `545d619` plus the prompt doc). All commands exited 0:
| Command | Result |
|---|---|
| `cargo test --workspace` | 183 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npm run typecheck` (Node 22.22.3) | exit 0 |
| `npm test` | 2 files, 43 passed (the P08 file has 24: 23 + C9) |

## Findings resolved
**F1 (B1, production): resolved as corrected. No divergence.**
- **Production.** `useWorkspaceState.ts:104` is now `if (event.kind === 'photo_added') setActiveId(event.photo_id);`, and
  `setActiveId` is in the effect deps (it is a stable `useCallback([])`, so the listener does not resubscribe).
- **Trace through `reload`** (lines 73–92). After the view lands, `activeRef.current` is the new id and the photo is visible,
  so the default rule does not override it. The rendered `activeId` state was already set. Both r1 failure scenarios now
  resolve:
  - (a) Empty session (ref `null`) → `photo_added{1}`: state and ref become 1, the reload keeps it, and the stage shows photo 1.
  - (b) Photo 1 active → `photo_added{2}`: state and ref become 2, and `active`, the review keys and `step` all read 2.
- **Rapid arrivals.** `photo_added{2}` then `{3}`: the request ticket drops the stale reload, and 3 is active and visible.
- **Unknown photos.** An id the view does not contain falls back to `defaultActive`.
- **Test C9** (`p08_workspace_ui.test.tsx:281–295`). Main photo 1 is active. The real workspace handler delivers
  `photo_added{2, SESSION}`, then `activeTile()` must be 2, then `c` must call `classifyPhoto` exactly once with `(2,'condition')`.
  - It can fail. On the old line, the state stays 1, so the first `waitFor` times out. Even without that assertion,
    `c` would call `classifyPhoto(1,…)`.
  - It observes only the DOM attribute and service-mock arguments, so it passes the over-engineering test.
  - It is declared in the Review log as candidate criterion C9 and traces to ch04 S-C, so it is not an orphan.
  - Case (a) is not tested separately. It runs through the same line, and an extra test would not trace to a new row.
- **Seam neighbours.** `photo_added` has one emitter, `ArrivalService::accept` (`arrival.rs:278`), which serves both captures
  and imports. So an import also moves the stage to the photo that arrived last. That matches ch06 §Import ("follow the same
  path as captures"). `ArrivalToasts` and the P07 sessions list use separate subscriptions and are untouched.

## Triage (flow.md §5)
| Item | Class | Reason |
|---|---|---|
| none | — | F1 is resolved, and no new blocking item was found. |

No MUST, so the verdict is `APPROVED_WITH_NOTES`. The four r1 notes are already in `backlog.md`, and one new note was added
(below).

## Remaining findings
**Note N1 → `backlog.md`, not blocking.** The `photo_added` handler sets the active id before the reload that brings the
photo into `view`. Three windows follow from that ordering:
- **Reload in flight.** Until the reload resolves, `active` is `null`, so `Stage` renders "Ready for the first photo"
  (`Stage.tsx:36`).
- **Reload fails.** The stage stays empty until the next event.
- **Review command in flight.** A command (`run`) in flight when the capture arrives applies its precomputed `next`
  afterwards and replaces the new capture.

Each window lasts as long as a local SQLite round trip. `classify` enqueues the mirror tag and does not await Capture One.
S-C's "within about a second" holds, so this is not B1, and closing it would be speculative robustness. Suggested later
change: set the active photo after the reload resolves (or in it), and drop a precomputed `next` once a newer arrival has been seen.

## Lessons for the plans (coordinator)
- The r1 lesson stands. UI plans that subscribe to arrival events need a row for "live event → what the stage shows". C9 is
  that row for P08 and should be folded into the plan's criteria (or refused with a reason), together with candidate C0.

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P08_workspace_ui_review_r2.md` (this file, new)
- `docs/implementation/narratives/P08_workspace_ui.md` (new)
- `docs/implementation/master_plan.md` (P08 tracker row only → APPROVED)
- `docs/implementation/plans/P08_workspace_ui.md` (Review log entry appended)
- `docs/implementation/backlog.md` (1 row appended)
- No probes and no production or test edits. Logs are outside the repo: `/tmp/p08r2_{cargo,clippy,fmt,tsc,vitest}.log`.
- No commits (the runner commits). No architecture-graph delta.
