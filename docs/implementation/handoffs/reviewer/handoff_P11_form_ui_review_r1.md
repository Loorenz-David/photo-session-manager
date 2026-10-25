---
plan: P11_form_ui
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-10-25
actor: opus
---
# Handoff: P11_form_ui, review round 1

**Summary.** All 30 criterion rows hold at their boundaries and their tests can fail on a wrong outcome. The
perimeter equals the declared files, and suite, lint and typecheck are green. There is **one blocking finding (B1)**:
the form's Title is stored as a second copy (`product.title`) instead of being the item's title (`items.title`).
Implementer Assumption 3 states this contradiction with ch05/ch08 itself. Five notes went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. The book answers F1 (ch05: the title has one home on the item), so it goes to the fix session.

## Gate
- Tracker row P11_form_ui was `IMPLEMENTED`. The checkpoint `a3ba576` exists in `git log`. Passed.
- `git show --stat a3ba576` lists 36 paths: 34 code paths (`FormDialog`, `StepTabs`, `MediaColumn`, `MetafieldInput`,
  `kinds.ts`, the 20 `inputs/*`, `CustomizeMode`, `IdentityEditor`, `AiPill`, `AiPopover`, `ResearchPanel`,
  `useFormState`, the test file, `services/attributes.ts`, `WorkspacePage.tsx`), plus `master_plan.md` (the P11 row
  only) and the plan file (Review log). This equals the handoff's declared files changed. Nothing is extra or
  missing.

## Evidence (one run, master plan §7.2 commands)
Tree: HEAD `af028a2`, `git status --porcelain` empty. Code is identical to `a3ba576`; the commits after it touch
docs only (handoff and prompt).

| Command | Result |
|---|---|
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | 4 files, 77/77 passed |
| `cargo test --workspace` | 235 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |

PATH used `~/.cargo/bin` and Node 22 (`~/.nvm/versions/node/v22.22.3/bin`), as §7.1 says.

## Verified correct
I read the production code and then the test for every row. No production file was edited, not even temporarily.
No test was re-run with a changed input, because reading settled every row.

| Row | Production | Test discriminates because |
|---|---|---|
| C1(a–u) | `kinds.ts` matches evidence §Field kind map in order: severity, then any option > 32 → radio, then > 4 options or joined > 48 → select, else pills. multi_choice works the same way; DIRECT covers the rest; reference and unknown → unsupported. | Each fixture has exactly one sufficient cause. In C1(e) the options join to 53 characters, there are 4 of them, and each is ≤ 32. C1(c) joins to exactly 48, so only the > 32 rule fires. C1(b) has 5 options, so without the severity check it would fall to `select`. |
| C2(a) | `useFormState.edit` keeps one timer per field (300 ms) and then `write` → `setAttributeValue(item, key, value)`. | With fake timers: no call at 299 ms, exactly one call at 300 ms, with the final typed string. |
| C2(b) | `MetafieldInput` shows the marker for `stored.valid === false`, unless the field was edited, with the verbatim text. `storedValue` hides invalid values, and nothing writes on load. | Any write-back on load would fire inside the 400 ms wait. The marker text and the verbatim value are both asserted. The first write happens only after the user picks "Satin". |
| C3(a) | `StepTabs` names come from `group.filled/total` of `form_get`. | The counts in the fixture (3/8, 0/13, 1/3) differ from the field counts (all 0), so a derived count would fail the test. |
| C4(a) | `CustomizeMode` uses `arrayMove` on drag end, filter on remove, append on add, and the default editor. Done → `setPreset(type, group, layout)`. | A real dnd-kit pointer drag on mocked rects, then the exact array is asserted, called once. |
| C4(b) | `onRemove` is null for `FIXED_KEYS`. | Title, Description and Tags have a reorder handle but no Remove control. Era does have one, so a blanket "no remove" would fail. |
| C5(a) | `CONFIDENCE_BARS` and "Check" when low. | The filled count is asserted for each confidence (3/2/1) out of 3 bars, plus the text. |
| C5(b) | Looks right → `setAttributeValue(item, key, parsed stored value)`. The command always writes provenance Human (`commands/attributes.rs:258–267`). Clear → `clearAttribute`. | The exact args are asserted, and each action is shown not to trigger the other call. |
| C5(c) | `showAi = !edited && provenance === 'ai'`. | The human field has 0 pills and the AI neighbour has 1, so the test is not vacuous. |
| C6(a) | `FilesPicker` shows `displayOrder(photos).condition`. Toggling writes `{excluded: ids − chosen}`. | The main photo is absent, the order is [p1,p2,p3], the payload is exactly `{excluded:[p2]}` and is written once, and the checked state is [T,F,T]. |

Other things verified by reading:
- Emptying a field calls `attribute_ai_clear`. This clears any live value, including a human one (`services/attributes.rs:319–328`), so emptying is a real clear.
- Edits still waiting are written when the form unmounts. The writes are not gated by `alive`.
- `presets_set` accepts `item_type = ""`.
- `D` is disabled while a dialog is open and inside text fields (`lib/keyboard.ts isTyping`).

**Hard constraints H1–H8, secrets, live services.** No violation found.
- The form calls only local commands (`attribute_*`, `presets_set`, `item_update_identity`), so nothing reaches a platform (H1).
- There are no file operations or pixel work (H2, H3). Thumbnails reuse the existing previews.
- Writes are idempotent, because the backend skips a value equal to the stored one (H4).
- The Product step works with no integration (H6).
- No core, Tauri or Rust file changed (H7, H8).
- The tests mock `@/services/attributes` and touch no network.

## Findings

### F1: B1, route: production. The form's Title is not the item's title
- **What is wrong.** The Product step's Title field reads and writes the attribute `product.title`. The item's title lives in `items.title`, and the field only shows it as a grey placeholder. Nothing writes `items.title` from the UI.
  - A title typed in the form never reaches the surfaces that read `items.title`: the sessions list, the workspace header and the dialog's own heading fallback.
  - The export's `item.json` then carries two titles: `"title"` from `items` and `attributes."product.title"` with the typed text.
  - P14's "title if different" (C3(a)) has no single local title to compare.
- **Authority.** ch05 §items gives `title` (owner U/I/AI, "product title") one home on the item. ch08 §Identity fields says "Title is a Product-step field", so the Product Title *is* that field. The implementer's Assumption 3 records the contradiction ("the book already answers it … ch05: one home, `items.title`"). The P10 backlog note says the binding is needed "before the form ships".
- **Correction.** Give the title one home, `items.title`:
  - `form_get` presents `items.title` as the `product.title` field's value (human, valid).
  - A human `attribute_values_set` / `attribute_ai_clear` on `product.title` updates `items.title` (and activity) instead of storing a separate row.
  - Add one behaviour test at the service boundary: set the Title, then `form_get`, and `session_get`'s item both show it.
  - Drop the placeholder workaround in the form.
- **Files the fix may touch:**
  - `crates/core/src/services/attributes.rs`
  - `crates/core/src/ports/store.rs` and `crates/infra/src/sqlite/store.rs` (a title setter, if none exists)
  - `crates/testkit/tests/p11_title_binding.rs` (new)
  - `apps/desktop/src/features/form/FormDialog.tsx`
  - `apps/desktop/src/features/form/__tests__/p11_form_ui.test.tsx`
  - optionally `apps/desktop/src/features/workspace/WorkspacePage.tsx`, to reload the view after a Title save so the header follows.
- **Out of scope for the fix:** P10's existing tests must stay green. If one of them pins `product.title` as a stored row, the fix updates that test with a one-line reason under Deviations.

No other blocking findings. No test fails B2: each row's test observes the row's outcome at its boundary and would fail on a wrong one.

## Assumptions (flow §6), assessed
- **Assumption 3 (Title not bound).** Contradicts ch05/ch08, so it is filed as **F1**.
- **Assumptions 1, 2, 4–11.** Consistent with the book. Reported to the owner in plain words:
  - Debounce 300 ms; emptying a field clears it.
  - Number entries are sent as typed; a bad entry is marked invalid.
  - The description opens in "AI agent writes it" (placeholder) unless one is stored.
  - Currency, unit and rating fallbacks (SEK, cm/mm/in, kg/lb, 0–5).
  - Native date picker.
  - Thumbnails use the adjusted preview, else the original.
  - Unreviewed photos are listed after Main in the media column.
  - Read-only fields show "Edited in Shopify admin".
  - Customize saves on Done or when the mode is left.
  - The "Details" button sits on the stage, not in the app header.
- **Assumption 12 (tracker actor `opus`).** Process only.

## Notes written to backlog (5)
1. P11: leaving Customize with the header toggle (not Done) re-fetches the form before the unawaited `presets_set` lands, so the form can briefly show the old order.
2. P11: when the photographer's own entry is stored invalid (e.g. `12,5` in a decimal field), the blur re-fetch empties the input. The text survives only in the Invalid marker.
3. P11 → P16 / item-type phase: `item_type = ""` for local items means one Customize layout is shared by all local items.
4. P11 → P16: the barcode cannot be cleared (`null` = unchanged).
5. P11 (test): C6(a) creates the photos in id order equal to position order. Plan lesson: order rows need a reorder in the fixture.

## Lessons for the plans
- A backlog note routed to a named next phase ("P10 → P11: bind the title before the form ships") was not folded into P11's rows or tasks. The implementer had to choose between leaving the perimeter and contradicting the book. When the coordinator compiles a phase, every note addressed to that phase should become a row/task or an explicit recorded refusal.

## Write perimeter of this session
- `docs/implementation/master_plan.md`: P11 tracker row only, now `CHANGES_REQUESTED`.
- `docs/implementation/plans/P11_form_ui.md`: Review log entry "review r1" appended.
- `docs/implementation/backlog.md`: 5 rows appended.
- This handoff (new).
- No production or test file was edited. No probes were made. No database or app state was touched. Logs went to `/tmp/p11_*.log` only (outside the repo). No narrative was written, because the phase is not approved.
