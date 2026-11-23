---
plan: L1_labels_provider_and_marker
role: fix
round: 2
state: DONE
date: 2026-11-23
actor: claude-sonnet-5.5 (pipeline-implementer)
---
# L1 fix 1: F1 (the session page reopened asks without refresh)

## Finding → what changed → files
- **F1 (B1, production).** MC-L3.7 / 09C §7: the session page asks `labels_status` with `refresh = true` when opened. Before: `Header` stays mounted across routes and `useSessionLabels` applied the Sessions page's id-list rule, so a reopen of the same session sent `refresh: false`.
  Change: `useSessionLabels(ids, answer, refreshOnOpen = false)`; with `refreshOnOpen` every run of the answer effect asks with `refresh: true`. `Header` passes `true` (its answer is the session id, so the effect runs once per open, and not at all while no session is open). The Sessions page passes nothing, so its rule (L1·C18(c)) is unchanged.
  `l1_c18f` now renders `Header` 5, then no session, then 5 again, and expects `[{[5], refresh: true}, {[5], refresh: true}]`.
  Files: `apps/desktop/src/features/labels/useSessionLabels.ts`, `apps/desktop/src/features/shell/Header.tsx`, `apps/desktop/src/features/labels/__tests__/l1_markers.test.tsx`.

## Divergences
One small one: the finding's correction offered "the hook forgets its last refreshing id list when its ids become null, or the header passes a per-open flag". I took the per-open flag, because forgetting on null would also change the Sessions page whenever its ids are null between two loads of the same page, which L1·C18(c) forbids. Not a deviation from the finding's text.

## Files changed
- `apps/desktop/src/features/labels/useSessionLabels.ts`
- `apps/desktop/src/features/shell/Header.tsx`
- `apps/desktop/src/features/labels/__tests__/l1_markers.test.tsx`
- `docs/implementation/plans/L1_labels_provider_and_marker.md` (Tracker line, Review log entry)
- this handoff
No Rust file, no shared code, no mutation probe, no production file touched temporarily. Test file touched: `l1_markers.test.tsx` (only `l1_c18f`).

## Checkpoint SHA
Recorded by the checkpoint commit whose subject starts `CHECKPOINT (not approved): L1 fix 1` (`git log --grep "L1 fix 1"`).

## Stamp
Tree: HEAD `71b685e` plus the perimeter above. Run once, one after the other:
- `npx vitest run src/features/{labels,sessions,shell,settings,workspace,publish,conversations,pending,compose}`: 104 files, 704 passed (before: 704; `l1_c18f` extended, no test added).
- `npm run typecheck`: 0 errors.
- `cargo fmt --all -- --check`: clean.
- Clippy and the Rust suites were not re-run: no Rust file changed and no shared code changed (flow §4A).
- The repo has no `npm run lint` script.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
