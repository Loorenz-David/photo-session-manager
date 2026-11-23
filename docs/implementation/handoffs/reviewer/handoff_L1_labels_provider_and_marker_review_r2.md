---
plan: L1_labels_provider_and_marker
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-23
actor: opus
---
# L1 re-review r2: APPROVED_WITH_NOTES

Fix 1 resolves F1 as its correction said: the session page now asks for its marker with `refresh: true` on every
open, a reopen of the same session included, and the Sessions page's id-list rule is unchanged. The perimeter is
exactly the three allowed files plus the plan's Tracker / Review log and the fix handoff. All checks green. No
blocking item remains; one new note (same family as r1 note 3) goes to the backlog.

## Review history
- **r1 (CHANGES_REQUESTED, 1 blocking).** Settled: U1–U4, rows C1–C19 except the reopen half of C18(f), H1–H8, the
  existing-test edits, orphan tests, the implementer's full-suite stamp. Open: F1 (B1, the session page reopened
  asked with `refresh: false`).
- **r2 (this round).** Scope: the fix checkpoint `c9618201` only; settled areas not re-verified.

## Perimeter check
- Gate: plan Tracker line `IMPLEMENTED (… fix 1 …)`; fix checkpoint `c9618201` ("CHECKPOINT (not approved): L1 fix 1, …")
  in `git log`. Review tree `6e9925a1` (= `c9618201` + the runner's re-review prompt commit, docs only);
  `git status --porcelain` clean at entry and exit.
- `git show --stat c9618201`: `apps/desktop/src/features/labels/useSessionLabels.ts`,
  `apps/desktop/src/features/shell/Header.tsx`, `apps/desktop/src/features/labels/__tests__/l1_markers.test.tsx`,
  `docs/implementation/plans/L1_labels_provider_and_marker.md` (Tracker line + Review log entry only),
  `docs/implementation/handoffs/implementer/handoff_L1_labels_provider_and_marker_fix_r2.md`. Exactly F1's allowed
  files. No undeclared path.
- `git diff --stat 167d95d2 HEAD -- '*.rs' 'Cargo.*'`: empty (no Rust changed since the r1 review tree).

## Checks (once each, one command at a time)
| Command | Result |
|---|---|
| `npx vitest run src/features/{labels,sessions,shell,settings,workspace,publish,conversations,pending,compose}` | 104 files, 704 passed |
| `npm run typecheck` | 0 errors |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-testkit -p psm-desktop --all-targets -- -D warnings` | clean |

No Rust suite run: the fix touched no crate and no Rust file changed since r1's green runs (flow §4A; r1's stamps stand).

## Findings resolved
- **F1 (B1) → resolved.** `useSessionLabels(ids, answer, refreshOnOpen = false)`; the answer effect asks with
  `refreshOnOpen || key !== lastRefreshed.current`. `Header` passes `true`; its `answer` is `labelled` (the session id,
  or `null` for no session / session 1), so the effect runs once per change of the open session and returns early on
  `null`. Walk: 5 → `ask([5], true)`; list (`null`, early return, no call); 5 → `ask([5], true)`. A switch 5 → 6 also
  asks with refresh. The only other caller, `SessionsPage` (`useSessionLabels(shownIds, list)`), takes the default,
  so L1·C18(c) is untouched (its test still green). The test `l1_c18f` rerenders 5 → undefined → 5 and asserts the
  exact call list `[{[5], true}, {[5], true}]` under `waitFor`; against the pre-fix code it would see
  `[{[5], true}, {[5], false}]` (r1 probe) and fail, and it also fails if the reopen makes no call. Over-engineering
  test passes: it asserts the calls crossing to the backend, nothing internal.
- **Divergence** (declared): the per-open flag, one of the two options the correction offered. Sound: resetting the
  last refreshing list on `null` would have touched the Sessions page's rule.

## Changed seam, read adversarially
- The labels-changed listener is unaffected by `refreshOnOpen` (it still asks with `event.access`), as MC-L3.7 says.
- On a reopen `shown.current` equals the key, so the previous marker stays visible until the fresh answer replaces
  it (rows never wait on the call). Consistent with "never delaying the rows".
- Seen in passing (note, not blocking): after the user leaves a session, the header's hook keeps `shownIds = [5]`
  and keeps listening, so a `labels_changed` while the list is on screen makes the header ask for session 5 too (with
  refresh when `access` is true). Nothing is rendered from it. Pre-existing since U4, same family as r1 note 3.

## Triage
| Item | Class | Why |
|---|---|---|
| (none blocking) | — | F1 resolved; nothing new reaches B1–B4 |
| Header hook re-asks for a closed session on `labels_changed` | DROP → note | one extra status call, no visible outcome; speculative robustness |

No MUST → `APPROVED_WITH_NOTES`.

## Remaining findings
None blocking. Notes (backlog `docs/implementation/backlog_labels/L1_labels_provider_and_marker.md`, the plan's
reviewer-notes path): r1 notes 1–4 still open; one r2 note added (above).

## Carry-forward dispositions
| Note | Destination |
|---|---|
| r1-1 markers stale after key removal / server change (ACC.3 gap) | orchestrator fold before L2 |
| r1-2 try/catch shaped by test mocks | L2·U3, L3·U4 |
| r1-3 hidden ids re-asked after a page empties | L2·U3 |
| r1-4 `p01` second `ADD COLUMN` tripwire | L2, L3 plans (Files) |
| r2-1 header hook keeps a closed session's ids | L2·U3 (with r1-3: clear shown ids when ids become null/empty) |

## Lessons for the plans
- A hook shared by a persistent element (the header) and a page carries two different "when opened" rules; name
  each caller's rule in the row (done now by the fix; L2/L3 rows that reuse the hook should say which rule applies).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Probe declaration
No probe, no temporary file, no production file touched, no database or state changed.

## Write perimeter (this session)
- `docs/implementation/plans/L1_labels_provider_and_marker.md` (Tracker line → APPROVED, Review log entry)
- `docs/implementation/backlog_labels/L1_labels_provider_and_marker.md` (one note appended)
- `docs/implementation/narratives/L1_labels_provider_and_marker.md` (new)
- this handoff
- No commits (the runner commits).
