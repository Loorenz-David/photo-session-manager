---
plan: P11_form_ui
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-26
actor: opus
---
# Handoff: P11_form_ui, re-review round 2 (after fix 1)

**Summary.** The one blocking finding from round 1 (F1, B1: the form's Title was a second copy) is resolved as its
correction said. The fix touched only files F1 allowed. Suite, lint and typecheck are green. No blocking item remains,
so the triage table is empty and the verdict is `APPROVED_WITH_NOTES`. Three new notes went to the backlog.

**Review history.** Round 1 settled all 30 criterion rows (C1–C6) at their boundaries, plus H1–H8 and assumptions
1, 2 and 4–11. Those are not re-verified here. This round covers only the F1 seam: the title write path, `form_get`'s
title presentation, and the form's reload after a Title save.

## Gate
- The tracker showed P11_form_ui `IMPLEMENTED` with a fix-1 note. Passed.
- Fix checkpoint `4fbc0f9` exists in `git log`. Passed.

## Perimeter check
`git show --stat 4fbc0f9` lists 8 paths:

| Path | Allowed by F1? |
|---|---|
| `apps/desktop/src/features/form/FormDialog.tsx` | yes |
| `apps/desktop/src/features/form/__tests__/p11_form_ui.test.tsx` | yes |
| `crates/core/src/ports/store.rs` | yes (title setter) |
| `crates/core/src/services/attributes.rs` | yes |
| `crates/infra/src/sqlite/store.rs` | yes (title setter) |
| `crates/testkit/tests/p11_title_binding.rs` (new) | yes |
| `docs/implementation/master_plan.md` (P11 row only, 1 line) | yes (process) |
| `docs/implementation/plans/P11_form_ui.md` (Review log "fix 1") | yes (process) |

The commits after it (`f7c2f86`, `9b29c4b`) add only the fix handoff and this round's prompt. The optional
`WorkspacePage.tsx` was not touched. Nothing is extra, so there is no perimeter finding.

## Evidence (one run)
Tree: HEAD `9b29c4b`, `git status --porcelain` empty. PATH used `~/.cargo/bin` and Node 22.

| Command | Result |
|---|---|
| `cargo test --workspace` | 236 passed, 0 failed (235 + 1 new) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | 4 files, 78/78 passed |

## Findings resolved
**F1 (B1, production): the form's Title is not the item's title → RESOLVED.** I checked each part of the
correction by reading the code:
- **`form_get` presents `items.title` as the `product.title` value.** `title_value(&item)` returns a human, valid
  value, and an empty or whitespace-only title is no value. It replaces the lookup in the values map only for this
  key, so the `filled` count follows it.
- **A human set updates `items.title` and stores no row.** `set_value` on `product.title`:
  - refuses a non-human provenance;
  - canonicalizes through the built-in Text definition, which trims, and refuses a non-text value;
  - then `write_title` writes `items.title` + `updated_at` only when the title differs (H4) and sets the sessions'
    `last_activity_at` (MC-4: "any attribute edit");
  - never touches `change_seq` (MC-6: "attribute edits do not change change_seq").
  The SQL in `update_item_title` names only `title` and `updated_at`.
- **Clear empties `items.title`.** This is consistent with the rest of the code: `create_local` accepts an empty
  title, `item_json` leaves out an empty title, and the list search reads `items.title`.
- **The placeholder workaround is gone.** After a Title blur or Save & close, the form calls `onItemChanged`, which
  is `state.reload()` in `WorkspacePage`, so the workspace view re-fetches.
- **The service test discriminates.** `p11_title_binding.rs` asserts every outcome on its own:
  - `session_get`'s item title;
  - the item row;
  - `form_get`'s field (and that it is valid and human);
  - no `product.title` row exists;
  - `last_activity_at` = T0 + 60 s;
  - after clear, an empty item title and no form value.
  A write to a value row instead, a missing activity update or a placeholder-only display would each fail it.
- **The form test discriminates.** It checks the field value (not a placeholder), the exact
  `setAttributeValue(item, 'product.title', 'Oak dining chair')`, and exactly one `onItemChanged`. It passes the
  over-engineering test: it observes props and calls at the component boundary.

**Judgment calls in the fix (four, all sound).**
1. Refusing non-human title writes. The book gives the title owners U/I/AI. The AI writer is not in the MVP, and
   `ai_suggest` has no command caller, so this refusal and leaving `ai_suggest` untouched reach nobody today. Noted
   for P12/P14 and the AI phase.
2. No preset default for the title. This protects the one home.
3. Refusing a non-text value. `items.title` cannot hold an invalid copy, and the form only sends strings.
4. No migration of earlier `product.title` rows. Acceptable: none can come from the shipped UI. Noted.

## Triage (flow.md §5)
| Item | Class | Why |
|---|---|---|
| — | — | No blocking item remains after fix 1. |

There is no MUST item, so the verdict is `APPROVED_WITH_NOTES`. Notes go to the backlog.

## Remaining findings
None blocking. I read the changed seam adversarially, and three things I saw are notes (appended to
`docs/implementation/backlog.md`):
1. **P11 (fix 1): Escape-close leaves the item card stale.** Change the Title, then close the form with Escape
   while the field still has focus, or after the debounced save without a blur. The title is written by the
   unmount flush or by the debounce, but `onItemChanged` is never called, so the workspace's item card shows the old
   title until the next workspace reload. The stored title, list and export are correct. Not B1: the boundary
   outcome is right and the display catches up on the next refresh.
2. **P11 (fix 1) → AI phase / P14: hidden rows.** `ai_suggest` on `product.title` would store a row the form
   ignores, and a `product.title` row from before the fix would still show up in `item.json` `attributes`. Neither
   is reachable today.
3. **P11 (fix 1) → P12, P14: title home.** The title now lives only in `items.title`, and a non-human set on
   `product.title` is refused. P12's import must seed `items.title`, and P14's "title if different" compares against
   it. This resolves the earlier "P10 → P11, P14" title note for the form.

## Carry-forward dispositions
| Note | Destination |
|---|---|
| r1 notes 1–5 (Customize toggle re-fetch, invalid entry empties input, shared layout for `item_type = ""`, barcode cannot be cleared, C6(a) fixture order) | backlog: P11 / P16 / item-type phase, as written in r1 |
| r2 note 1 (Escape-close stale card) | backlog: P11 follow-up (any later UI phase, e.g. P15) |
| r2 note 2 (hidden AI/legacy title rows) | backlog: AI phase / P14 |
| r2 note 3 (title home for import/publish) | backlog: P12, P14 |

## Lessons for the plans
- When a fix moves a field's single home (here, the title moves from an attribute row to `items.title`), the next
  phases that read or write that field (P12 import, P14 publish) should get the new home written into their rows
  when the coordinator compiles them. Backlog note 3 carries this.

## Mutation-probe declaration
No probes. Per flow.md §4, no production or test file was edited, not even temporarily. No database or app state
was touched. Logs went to `/tmp/p11r2_*.log` only, outside the repo.

## Write perimeter of this session
- `docs/implementation/master_plan.md`: P11 tracker row only, now `APPROVED`.
- `docs/implementation/plans/P11_form_ui.md`: Review log entry "re-review r2" appended.
- `docs/implementation/backlog.md`: 3 rows appended.
- `docs/implementation/narratives/P11_form_ui.md` (new).
- This handoff (new).
- Nothing committed. The runner commits these files.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
