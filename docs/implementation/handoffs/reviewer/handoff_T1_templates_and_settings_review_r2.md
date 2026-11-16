---
plan: T1_templates_and_settings
role: review
round: 2
verdict: APPROVED
state: DONE
date: 2026-11-16
actor: opus
---
# T1 re-review r2 — APPROVED

Gate: tracker showed `IMPLEMENTED` with the fix-1 note, and fix checkpoint `a801f3a` exists. The orchestrator committed
it because the fix session's sandbox refused `git add`, as recorded in the fix handoff. The only blocking finding
from r1 (F1, no working delete confirmation) is resolved as its correction said. No blocking items remain, so there
is nothing to triage. Review history: r1 settled U1–U3, rows C1–C6(g), H1–H8 and the perimeter of `c6a63a5^..23a9114`.
None of that was re-verified.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Perimeter check
- `git show --stat a801f3a` lists `apps/desktop/src/features/settings/TemplatesSection.tsx` (+14/−2) and
  `apps/desktop/src/features/settings/__tests__/t1_templates_editor.test.tsx` (+6/−1). Both are in F1's allowed
  list. The optional files (`services/templates.ts`, `capabilities/default.json`, `globals.css`) were not touched.
- `git diff --stat 23a9114 4ca3773` adds only pipeline documents: prompts, the r1 and fix handoffs, the runner
  state file, the 7 backlog lines from r1, and this plan's tracker line and Review log lines (r1 and fix). No other
  production or test path changed.
- The fix handoff declares the same two code files plus its docs. Claim and tree agree.

## Findings resolved
**F1 (B1) — resolved.**
- `window.confirm` is removed. A search for `window.(confirm|alert|prompt)` over `apps/desktop/src` finds nothing.
- The Delete template button now only sets `deleteOpen`. The shadcn `Dialog` (Radix, portalled to the body; the
  same component `SettingsPage` uses for unsaved changes) shows "Delete shooting template?" with the item type.
- `deleteCurrent` → `deleteTemplate` is reachable only from the dialog's own Delete template button. Cancel, the X
  close button and Escape (Radix `onOpenChange`) only close the dialog.
- The Templates pane sits outside the settings `<form>`, and all the buttons are `type="button"`, so nothing submits
  Save settings.
- A rejected delete still goes through `failure` (alert + `getTemplate` reload).
- No plugin permission is needed, so the capability is unchanged. That is correct for this option.
- **C6(h)** now asserts the dialog text ("Delete the template for Dining chair?"), then Cancel → `deleteTemplate`
  not called and the editor still shown, then confirm inside the dialog → called exactly once with id 8 → Create
  template shown. A regression to delete-on-first-press would fail the "not called" line, and a confirm that did
  nothing would fail the `calledExactlyOnceWith`. The test observes the row at the UI boundary and would fail on a
  wrong outcome. It passes the over-engineering test: it drives only visible controls.
- Declared divergences: none. The fix used the first option of F1's correction.

Adversarial read of the changed seam, with nothing wrong found:
- The dialog's description reads `template?.item_type`, but it can only open while a template is shown.
- `setDeleteOpen(false)` runs before the delete, so the dialog cannot stay open over the Create view.
- While the dialog is open, Radix hides the outer content from assistive tech, so the two same-named buttons do
  not collide for a user.
- The dialog stays mounted in an inactive pane, but it is closed and cannot open without a click.

**Checks** (reviewer run, clean tree `4ca3773`, `git status --porcelain` empty, one command at a time):

| Command | Result |
|---|---|
| `cd apps/desktop && npx vitest run src/features/settings` (includes `t1_templates_editor`) | 18 files, 80/80 |
| `npm run typecheck` | green |
| `cargo fmt --all -- --check` | green |
| `cargo clippy --workspace --all-targets -- -D warnings` | green |

No Rust file changed in the fix, so no `cargo test` ran (no crate touched, no `--test suite` stem named by F1). The
implementer's full-suite stamp from r1 stands. No probe touched any file (flow §4: no mutation regime).

## Triage
No blocking items remain, so there is nothing to classify as MUST or DROP.

| Item | Class | Disposition |
|---|---|---|
| — | — | — |

## Remaining findings
None blocking. No new notes this round.

Carry-forward dispositions: the seven r1 notes stay open under T1 in `docs/implementation/backlog.md`:
1. Guide rule against `window.confirm` / `window.alert` (a candidate for the orchestrator's guide fold).
2. C2(c)/(e) check final order by sequence only.
3. C2(d) covers 2 of its 4 pairs.
4. The Zoom box saves per keystroke.
5. Enter in the Stock quantity box submits Save settings.
6. Raw `template_exists` / `no_template` codes.
7. Orphan files when cleanup after commit fails.

Destination: backlog. Items 4 and 6 are natural to pick up in T2–T4, which build on the editor.

## Lessons for the plans
- None new. The r1 lesson stands: plans asking for "a confirmation" should name the in-app `Dialog`. T3/T4 warnings
  should be planned that way.

## Write perimeter
- `docs/implementation/plans/T1_templates_and_settings.md`: the tracker line (→ APPROVED) and one Review log entry
  appended.
- `docs/implementation/narratives/T1_templates_and_settings.md` (new).
- This handoff.
- No production or test file was edited. No probe touched any file. No database or state outside the temporary test
  folders. Commits: none (the runner commits).
