---
phase: A20_research_publish_ui
title: The publish review tells you about unchecked suggestions — how many weak ones will be left out, and which fields have a suggestion that replaced a preset or Shopify value, with Keep previous / Use suggestion / Edit right there; a waiting item says "Waiting for you"
difficulty: normal
projection: no
depends_on: [A09_research_publish_hold, A18_research_panel_ui]
---
# A20 — Publish popover and form: withheld line, conflicts, "Waiting for you" (08A §7; 08B MC-A10, MC-A12; MA8, MA17)

## Goal
Show what A09 computes: in the confirmation popover the line "n low-confidence AI suggestions will not be
published" and, per item, its conflicts as a warning with the three actions; pass the shown set to the
confirmation and rebuild on refusal; the session row label "Waiting for you · n conflicts"; conflict marks and the
same three actions in the form. **Confirm is never disabled by a conflict.** **Not in this phase:** any change to
the rules (A09), the panel (A18).

## Read first
`master_plan_research.md` §6.6 · book `08A` §7, `08B` **MC-A10** (the popover's content and texts; what is shown and
re-checked; status and label), MC-A12 (actions) · evidence `design_handoff.md` · code
`apps/desktop/src/features/publish/{PublishPopover.tsx, usePublish.ts, BatchQueue.tsx}`,
`apps/desktop/src/features/sessions/StatusPill.tsx`, `apps/desktop/src/features/form/{FormDialog.tsx, AiPopover.tsx,
MetafieldInput.tsx}`, `apps/desktop/src/services/{publish.ts, attributes.ts, bindings.ts}`, A09's preview fields.

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `apps/desktop/src/features/publish/ConflictList.tsx`,
`apps/desktop/src/features/publish/__tests__/a20_research_publish.test.tsx`,
`apps/desktop/src/features/form/__tests__/a20_conflicts_in_form.test.tsx`.
Edited: `apps/desktop/src/features/publish/{PublishPopover.tsx, usePublish.ts}`,
`apps/desktop/src/features/sessions/StatusPill.tsx`, `apps/desktop/src/features/form/{FormDialog.tsx, AiPopover.tsx}`,
`apps/desktop/src/services/{publish.ts, attributes.ts}`, own tracker row. No Rust change is expected; one found
necessary is declared under Deviations.

## Core mechanisms
None.

## Ordered tasks
1. Popover: when an item's `n_withheld` > 0, the line `n low-confidence AI suggestions will not be published`.
2. Popover: for each item with conflicts, a warning block listing per conflict the field label, the previous value
   and where it came from (`preset default` / `from Shopify`), the suggested value and its confidence, the buttons
   **Keep previous**, **Use suggestion**, **Edit**, and the line `This item waits until you resolve n conflicts`.
   A resolution calls `attribute_conflict_resolve` and re-reads the preview.
3. Confirm passes the `n_withheld` and conflict keys shown for each item. On the refusal `AI suggestions changed
   while you were confirming` the popover shows that sentence and rebuilds from a new preview.
4. Row label `Waiting for you · n conflicts` for `RowLabel::WaitingForYou(n)`.
5. Form: a field in conflict is marked and its popover offers the same three actions in place of Looks right /
   Clear; resolving there has the same effect.

## Criteria (5 criteria, 9 rows)
Boundary: UI tests with the services mocked (`vi.mock`), as `master_plan.md` §6.9.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | popover | preview with `n_withheld` 2 for one item; 0 for another | `2 low-confidence AI suggestions will not be published` once, under the first item | MA8 |
| C2(a) | popover | an item with one conflict: label `Designer`, prior `Unknown` from the preset, suggestion `Hans J. Wegner` at medium | the block shows the label, `Unknown`, `preset default`, `Hans J. Wegner`, medium confidence, the three buttons and `This item waits until you resolve 1 conflict`; Confirm is enabled | MA17 |
| C2(b) | popover | a conflict whose prior came from the shop | `from Shopify` | MC-A10 |
| C3(a) | resolve in the popover | click **Keep previous** | `attribute_conflict_resolve(item, key, keep_previous)` called; the preview is read again; the block is gone when the new preview has no conflict | MC-A12 |
| C3(b) | confirm | preview showed `n_withheld` 1 and conflict keys `{designer}` | `publish_confirm` receives exactly that shown set for the session | MA17 (R-IA22) |
| C3(c) | refusal | confirm rejects with `AI suggestions changed while you were confirming` | the sentence is shown; the preview is requested again; the popover stays open | MA17 |
| C4(a) | session row | label `WaitingForYou(2)` | `Waiting for you · 2 conflicts` | M14 |
| C5(a) | form | a field in conflict | marked; its popover shows Keep previous / Use suggestion / Edit and the previous value with its origin; **Edit** puts the field in edit mode and the typed value is saved as a human edit | MC-A12 |
| C5(b) | form header | two plain suggestions and one conflict | `3 to review` | MC-A10 |

## Delegated choices
Visual treatment of the warning within the design tokens · singular/plural wording (`1 conflict` / `n conflicts`)
· where Edit opens (the form field, focused). Refine at prompt time against `claude_design/`.

## Manual try-out
(With research producing suggestions, A15–A19 in place, on scratch data and the test shop:)
1. Give an item type a preset default for a field research fills (Customize fields), let research suggest another value.
2. Press Publish: the popover lists the conflict with both values. Press Confirm without resolving.
3. The session row says "Waiting for you · 1 conflict"; another session published at the same time goes through.
4. Open the form, choose Keep previous or Use suggestion: the item starts publishing on its own.
5. In the shop, the field holds what you chose.

## Notes
- No `implementer: fable`.
- Review mode: `npx vitest run src/features/publish src/features/form src/features/sessions`, typecheck.

## Review log
