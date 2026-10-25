---
phase: P11_form_ui
title: Product details form — steps, field kinds, customize fields, AI states (frontend)
difficulty: normal
projection: no
depends_on: [P10_attributes, P08_workspace_ui]
---
# P11 — Product details form: steps, field kinds, customize fields, AI states (frontend)

## Goal
Build the form dialog from the design: Details / Condition / Product steps with filled/total counts, the media
column, the `MetafieldInput` generator driven by core-typed definitions (field kinds and thresholds from the
design evidence), values saved as typed, the damage-report picker (exclusions), Customize fields mode per item type
and step, SKU/barcode header editor, tags input, description mode (manual / "AI agent writes it" placeholder), and
the AI pill / popover / research-panel **states** driven by stored provenance (no agent). Keys `D`, `Esc`.
**Not in this phase:** the batch queue strip and publish footer buttons (P15), the agent (post-MVP).

## Read first
master plan §6.7 (P10 commands), §6.9 · book `01`, `14`, `08` (all), `04` S-F · contracts `05A` MC-9 (severity set),
`07A` MC-14 · evidence `design_handoff.md` §Form, §Field kind map, §AI, §Keyboard · `claude_design/NOTES.md` §Screens
6–7, §Decisions (Metafields drive the form, Condition step, Customize fields, SKU and barcode, Tags, Description, AI
agent rules); `COMPONENTS.md` §Product details form, §MetafieldInput kinds → shadcn; screens `desktop/22–31`, `34`,
`narrow/03`; `claude_design/source/metafield-input.html` (reference for the generator; not transplanted).

## Files expected to change
`apps/desktop/src/features/form/{FormDialog.tsx, StepTabs.tsx, MediaColumn.tsx, MetafieldInput.tsx, kinds.ts (kindOf),
inputs/{TextInput, TextareaInput, ChoicePills, SelectInput, RadioCards, SeverityScale, MultiPills, MultiSelect, TagsInput,
IntInput, DecimalInput, DimensionInput, BooleanInput, DateInput, ColorInput, RatingInput, UrlInput, FilesPicker,
DescriptionInput, Unsupported}.tsx, CustomizeMode.tsx, IdentityEditor.tsx, AiPill.tsx, AiPopover.tsx, ResearchPanel.tsx,
useFormState.ts, __tests__/p11_form_ui.test.tsx}`, `apps/desktop/src/services/attributes.ts`,
`apps/desktop/src/features/workspace/WorkspacePage.tsx` (open form with `D` / header button).

## Core mechanisms (unit tests allowed)
- `features/form/kinds.ts: kindOf(def: Definition) -> Kind` (evidence §Field kind map, expressed over core types: `choice` → severity | radio | select | choice; `multi_choice` → multiselect | multi; `tags` → tags; …).

## Ordered tasks
1. `useFormState(itemId)`: `form_get`, local edits, debounced `attribute_values_set` (300 ms), re-fetch on blur.
2. `kindOf` and the input components (shadcn mapping per `COMPONENTS.md`).
3. Dialog: header (type, SKU | barcode editor → `item_update_identity`, Saved indicator, Customize fields, close),
   step tabs with counts, media column (cover 2×2 + smaller grid), footer (Save & close, prev/next step; Publish
   button slot filled by P15).
4. Customize mode: dnd-kit sortable list, remove, add via `Command` over all definitions, default value editor →
   `presets_set`; Title/Description/Tags without a remove control.
5. AI states: pill with 3/2/1 bars (low reads "Check"), popover (reason, sources, Looks right → `attribute_values_set`
   with provenance human of the same value, Clear → `attribute_ai_clear`, See research), research panel with the
   timeline from `ai_sources` (static), Accept all → `attribute_ai_accept_all`.
6. Damage-report picker: condition photos with selection; unselecting writes `{"excluded": […]}`.
7. Tests.

## Criteria (6 criteria, 30 rows; 28 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a–u) | `kindOf` (21 rows) | (a) text, no choices → `text` · (b) choice with the severity set → `severity` · (c) choice, one option > 32 chars → `radio` · (d) choice, 5 short options → `select` · (e) choice, 4 options joined > 48 chars → `select` · (f) choice, 4 short → `choice` · (g) tags → `tags` · (h) multi_choice, 5 options → `multiselect` · (i) multi_choice, 3 short → `multi` · (j) textarea and rich_text → `textarea` · (k) integer → `int` · (l) decimal → `decimal` · (m) dimension → `dimension`, weight → `weight` · (n) boolean → `boolean` · (o) date → `date` · (p) color → `color` · (q) rating → `rating` · (r) money → `money` · (s) url → `url` · (t) media_list → `files` · (u) reference → `unsupported` | the named kind | `[rule]` evidence §Field kind map (ch08 §Field kinds); (b) also MC-9 severity set |
| C2(a) | typing in a text field | fake timers | `attribute_values_set(item, key, "…")` called once after the debounce with the typed string | `[rule]` ch08 (values save as typed) |
| C2(b) | field whose value has `valid = false` | — | rendered with an "invalid" marker and the verbatim value; no `attribute_values_set` until the user edits | MC-9 (invalid shown, never written back unless edited) |
| C3(a) | step tabs | `form_get` with Details 3/8 filled, Condition 0/13, Product 1/3 | tabs read "Details 3/8", "Condition 0/13", "Product 1/3" | `[rule]` evidence §Form |
| C4(a) | Customize mode: drag k2 above k1, remove k3, add k9 with default "Oak" | — | `presets_set(type, step, [{key:k2},{key:k1},{key:k9, default:"Oak"}])` | `[rule]` ch08 §Customize fields |
| C4(b) | Customize mode on Product | — | title, description, tags rows have no remove control | `[rule]` ch08 (never removable) |
| C5(a) | `AiPill` | provenance ai, confidence high / medium / low | 3 / 2 / 1 bars; low reads "Check" | `[rule]` evidence §AI |
| C5(b) | popover actions | "Looks right" / "Clear" | `attribute_values_set(… provenance human, same value)` / `attribute_ai_clear(item, key)` | `[rule]` ch08 |
| C5(c) | field with provenance human | — | no pill rendered | `[rule]` ch08 (human edits win) |
| C6(a) | `FilesPicker` | condition [p1,p2,p3], excluded [] → user unticks p2 | `attribute_values_set(item, "custom.damage_report", {"excluded":[p2]})`; p1, p3 remain shown selected | MC-14 |

## Delegated choices
Debounce interval, how `See research` renders sources, media column thumbnails source (adjusted preview, else
original), tag suggestions source (none without an integration; shop tags arrive with P12's schema import as
`validations.suggestions` — optional), date picker component.

## Manual try-out
1. Open a session, press `D`: the form shows Details / Condition / Product. Without Shopify configured only the Product
   step has fields (Title, Description, Tags) plus SKU and barcode in the header; type a title → "Saved".
2. Customize fields → reorder, remove, add → close → the layout persists for that item type.
3. After P12 (Shopify configured) the Details and Condition steps show the shop's fields: severity scales, pills,
   dropdowns, dimension inputs, the damage-report picker over the condition photos.

## Notes
- AI pills only appear for values with provenance `ai`; nothing writes such values in the MVP, so the states are
  exercised by tests and by a dev fixture (delegated), never by production.
- Estimated size: ~32 files, ~2 800 lines.

## Review log
(empty)
