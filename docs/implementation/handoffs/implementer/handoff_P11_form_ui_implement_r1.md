---
plan: P11_form_ui
role: implement
round: 1
state: DONE
date: 2026-10-25
actor: opus
---
# Handoff — P11_form_ui, implement round 1

Gate: intention RATIFIED; P10_attributes and P08_workspace_ui APPROVED; P11 was NOT_STARTED. Passed.

## What was built
- **Product details form.** In a session that has an item, `D` (or the new "Details" button on the stage) opens a
  dialog. Its header shows the item type (or title), SKU | barcode with a pencil editor, a Saved indicator, a
  Research chip when AI values exist, Customize fields, and close. Below it are the Details / Condition / Product
  tabs with filled/total counts. On the left is the media column: cover 2×2 and the other set smaller; on Condition
  the condition photos lead. The right side holds the generated fields. The footer has the fill progress, Save &
  close, the previous/next step buttons, and an empty Publish slot for P15. Esc closes the dialog.
- **Fields from definitions.** `kindOf` maps each core-typed definition to one of 21 kinds, using the evidence
  §Field kind map with its exact thresholds. Each kind has an input: pills, severity scale, radio cards, searchable
  select, multi pills, multi-select, tags, stepper, decimal, money, dimension/weight, Yes/No, date, colour, stars,
  URL, damage-report picker, description (AI/manual), and "Edited in Shopify admin".
- **Saving.** Values save as typed. Each field is written 300 ms after its last edit (`attribute_values_set`). On
  blur the field is written and the form re-fetched. Edits still waiting are written when the form closes.
  Emptying a field clears it.
- **Invalid values (MC-9).** These are shown with an "Invalid" marker and the verbatim stored value. They are
  never written until the photographer edits the field.
- **Customize fields** (per item type and step). Rows can be dragged to reorder (dnd-kit) and removed, except
  Title, Description and Tags. Fields are added from every definition through a search list, and each field can
  carry a default value. The layout is saved with `presets_set` on Done, or when the mode is left another way.
- **AI states (no agent).**
  - A field with provenance `ai` shows a pill with 3/2/1 bars; low confidence reads "Check".
  - The popover shows confidence, value, reason and sources. "Looks right" writes the same value as a human edit,
    "Clear" calls `attribute_ai_clear`, and "See research" opens the panel.
  - The research panel is a static timeline from `ai_sources`. Focusing a field dims the other steps, and Accept all
    calls `attribute_ai_accept_all`.
  - Human-owned fields show no pill.
- **Damage report (MC-14).** It shows the session's condition photos in condition order, ticked per `form_get`'s
  `selected`. Unticking writes `{"excluded": [...]}`.

## Tests → criterion rows
File: `apps/desktop/src/features/form/__tests__/p11_form_ui.test.tsx` (vitest + jsdom; `@/services/attributes` mocked).
Each test asserts the row's exact outcome at its boundary.

| Row | Test | Assertion shape |
|---|---|---|
| C1(a)–(u) | `P11 C1 kindOf` `it.each`: 23 cases, one per row. (j) has textarea and rich_text; (m) has dimension and weight | `kindOf(def)` equals the named kind exactly (unit test on the listed core mechanism) |
| C2(a) | `C2(a) typing in a text field…` | Fake timers. Not called at 299 ms; called exactly once at 300 ms with `(7, "custom.maker", "Hans Wegner")` |
| C2(b) | `C2(b) an invalid stored value…` | The marker contains "Invalid" and the verbatim `Glossy!`. No write after 400 ms. The first write happens only after the user picks "Satin" |
| C3(a) | `C3(a) the step tabs read…` | Tabs whose accessible names are exactly "Details 3/8", "Condition 0/13", "Product 1/3" |
| C4(a) | `C4(a) Customize: drag k2 above k1…` | Real pointer drag (dnd-kit) on mocked rects, remove k3, add k9, default "Oak", Done. `setPreset` called once with `("Chair", "details", [{key:k2},{key:k1},{key:k9, default:"Oak"}])` |
| C4(b) | `C4(b) Customize on Product…` | Title, Description and Tags rows have a reorder handle and no Remove button. The added "Era" row does have one, so the test discriminates |
| C5(a) | `C5(a) AiPill shows 3 / 2 / 1 bars…` | Filled bars are 3/2/1 of 3. Text "AI", "AI", "Check" |
| C5(b) | `C5(b) popover…` | "Looks right" gives `setAttributeValue(7, scratch, "Minor")` (the human-edit command, same value). "Clear" gives `clearAttribute(7, "custom.era")`. Neither action triggers the other call |
| C5(c) | `C5(c) a field with provenance human…` | 0 pills in the human field, 1 in the AI field beside it |
| C6(a) | `C6(a) damage report…` | Checkboxes appear in order [p1,p2,p3]. Unticking p2 writes exactly `{excluded:[p2]}` once. Checked state becomes [true,false,true] |

Reverse direction: every test in the file maps to a row above, and there are no orphan tests.
Red baseline: **not captured.** I wrote the production code before the test file. At the first run, 3 tests failed,
and all three were test-side causes:
- the C1(e) fixture joined to exactly 48 characters instead of more than 48;
- a helper returned null in C5(b);
- jsdom has no PointerEvent (fixed with the polyfill P08 uses).

## Assumptions
1. Debounce is 300 ms per field. Emptying a field calls `attribute_ai_clear`; it never writes `""` or `[]` (ch08 "save as typed"; MC-8 "absent values omitted").
2. Number-like inputs send what was typed. A whole number goes out as a JSON integer. Anything that does not fit is stored invalid by the backend, and its marker appears after the blur re-fetch (MC-9).
3. **Title is not bound to `items.title`.** No command writes `items.title`, and the backend is outside this perimeter. The Title input shows the item's title as a placeholder only and saves to `product.title`. This is backlog note "P10 → P11, P14". The orchestrator should route the binding (seed and write-through, or drop `product.title`) to P14 or a fix, before list, export and publish can show a title the form did not.
4. The description opens in "AI agent writes it", the design's default, unless a description is already stored. Without the agent that mode is a placeholder that writes nothing. "Write manually" saves as typed.
5. Some fallbacks when the definition says nothing:
   - money currency: the stored currency, else `validations.currency`, else `SEK`;
   - units: `validations.units`, else cm/mm/in and kg/lb;
   - rating scale: `validations.min/max`, else 0–5;
   - tag suggestions: `validations.suggestions`, which today means none.
6. The date picker is the native `<input type=date>`.
7. Thumbnails use the adjusted preview, else the camera original (the workspace's `Thumb`).
8. The media column lists unreviewed photos after Main, because they publish in Main (N2).
9. A non-writable definition renders "Edited in Shopify admin" (or "Edited in <integration>"), whatever its core type.
10. Customize saves on Done, and also when the mode is left by changing step or closing the form. It does not save on every change.
11. "Header button" became a "Details" button on the workspace stage, under the Sessions toggle. The app header (`features/shell/Header.tsx`) is outside the perimeter.
12. Tracker actor: `opus`, not `codex` as the prompt template said. Opus ran this session (flow §2, owner change 2026-10-25).

## Deviations
None from the file list. Only shadcn components that already exist are used: tabs, radio cards and the research panel are plain elements with ARIA roles, because adding `components/ui/*` would leave the perimeter. `updateIdentity` sits in `services/attributes.ts`; `services/items.ts` is not in the perimeter.

## Files changed
- `apps/desktop/src/services/attributes.ts` (new)
- `apps/desktop/src/features/form/` (new): `FormDialog.tsx`, `StepTabs.tsx`, `MediaColumn.tsx`, `MetafieldInput.tsx`, `kinds.ts`, `CustomizeMode.tsx`, `IdentityEditor.tsx`, `AiPill.tsx`, `AiPopover.tsx`, `ResearchPanel.tsx`, `useFormState.ts`, `inputs/{TextInput, TextareaInput, ChoicePills, SelectInput, RadioCards, SeverityScale, MultiPills, MultiSelect, TagsInput, IntInput, DecimalInput, DimensionInput, BooleanInput, DateInput, ColorInput, RatingInput, UrlInput, FilesPicker, DescriptionInput, Unsupported}.tsx`, `__tests__/p11_form_ui.test.tsx`
- `apps/desktop/src/features/workspace/WorkspacePage.tsx` (open the form with `D` / Details button; workspace keys are off while it is open)
- `docs/implementation/master_plan.md` (P11 tracker row only), `docs/implementation/plans/P11_form_ui.md` (Review log)
- This handoff (left for the runner to commit).
- Files touched only temporarily: `CustomizeMode.tsx` got debug `console.log` lines while I diagnosed the drag test. I restored it from a copy before the commit, and the committed file has no debug lines.

## Checkpoint SHA
`a3ba576`. The tree was clean after the commit except this handoff.

## Stamp
| Command | Result |
|---|---|
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | 4 files, 77 tests passed (32 new) |
| `cargo test --workspace` | 22 test binaries, 235 passed, 0 failed (no Rust changes; `export_bindings` left `bindings.ts` byte-identical) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npx vite build` (extra check) | built; only the existing chunk-size warning |

No failures existed before this phase. Environment note: `cargo` was not on the session's PATH, so I used `PATH=$HOME/.cargo/bin:$PATH`. npm ran under Node 22 per master plan §7.1.

## Manual try-out
1. `cd apps/desktop && npm run tauri -- dev`, then open a session that has an item.
2. Press `D`, or click "Details" at the top right of the stage. The form shows Details / Condition / Product. Without Shopify configured only Product has fields: Title, Description and Tags. SKU and barcode are in the header; click them to edit.
3. Type a title. The header shows "Saving…" and then "Saved". Close and reopen the form: the title is still there.
4. Description: switch to "Write manually" and type; it saves the same way. Tags: type a tag and press Enter.
5. Click Customize fields on Product. Title, Description and Tags have no remove control. Drag them to reorder, then press Done. Reopen the form: the order is kept for every item of that type.
6. After P12 (Shopify configured), Details and Condition show the shop's fields, and Condition shows the damage-report picker over the condition photos. Untick a photo to exclude it.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. One item for the orchestrator to route, not for the owner: the Title binding in Assumption 3. The book already answers it (ch05: one home, `items.title`), but it needs backend work outside P11.
