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
### 2026-10-25 — implement r1 (opus, `pipeline-implementer`)
**Built.** `services/attributes.ts` (wrappers over the P10 commands plus `item_update_identity`; values go out as
JSON text of the core form). `features/form/`: `kindOf` (core mechanism), 20 input components, `MetafieldInput`
(label, AI pill, Default marker, invalid marker with the verbatim value, input by kind), `useFormState` (form_get,
drafts, per-field 300 ms debounce, flush + re-fetch on blur, pending edits written on close), `FormDialog` (header
with type, SKU | barcode editor, Saved indicator, Research chip, Customize fields, close; step tabs with counts;
media column; footer with fill progress, Save & close, prev/next, empty Publish slot for P15), `CustomizeMode`
(dnd-kit sortable rows, remove except Title/Description/Tags, add via `Command` over all definitions, default editor,
`presets_set` on Done or on leaving the mode), `AiPill`/`AiPopover`/`ResearchPanel` (static timeline from
`ai_sources`, focus dims other steps, Accept all). `WorkspacePage`: `D` and a "Details" button open the form when
the session has an item; workspace keys are off while it is open; Esc closes it (Radix dialog).
**Tests.** `__tests__/p11_form_ui.test.tsx`: 32 cases for the 30 rows (C1(j) and C1(m) each have two cases). No red
baseline was captured: the production code was written before the test file.
**Judgment calls.** (1) Debounce 300 ms per field; emptying a field calls `attribute_ai_clear` instead of writing
`""`/`[]`. (2) Decimal/int/dimension inputs send what was typed; a non-fitting entry is stored invalid by the backend
(MC-9) and shows the invalid marker after the blur re-fetch. (3) Title: no value bound to `items.title` (no command
writes it); the Title input shows the item's title as placeholder only (see backlog note P10 → P11, P14). (4) Description
opens in "AI agent writes it" (design default) unless a description is stored; that mode is a placeholder that writes
nothing. (5) Money currency: stored, else `validations.currency`, else `SEK`. Units: `validations.units`, else cm/mm/in,
kg/lb. Rating scale: `validations.min/max`, else 0–5. Tag suggestions: `validations.suggestions` (none today).
(6) Date: native `<input type=date>`. (7) Thumbnails: adjusted preview, else original (reuses the workspace `Thumb`).
(8) Media column shows unreviewed photos after Main (they publish in Main, N2). (9) Non-writable definitions render
"Edited in Shopify admin" (or "Edited in <integration>") whatever their core type. (10) Customize saves on Done and
when the mode is left by step change or closing the form, not on every change. (11) The form button sits in the
workspace stage beside "Sessions", not in `features/shell/Header.tsx` (outside the perimeter).
**Deviations.** None from the file list. Only existing shadcn components are used (no Tabs/RadioGroup/Sheet/Calendar
added to `components/ui/`, which is outside the perimeter); tabs, radio cards and the research panel are plain
elements with ARIA roles.
**Observations.** Local items are created with `item_type = ""` (P07), so Customize fields saves layouts for the
empty type until some phase lets the photographer set a type. `item_update_identity` treats `null` as "unchanged",
so the barcode cannot be cleared from the form.


### 2026-10-25 — review r1 (opus, `pipeline-reviewer`) — CHANGES_REQUESTED
**Gate.** Tracker `IMPLEMENTED`; checkpoint `a3ba576` in `git log`. `git show --stat a3ba576`: 36 paths = the declared
files changed (34 code paths incl. 20 inputs, `master_plan.md`, this plan). No extra or missing paths.
**Suite (tree `af028a2`, clean; code identical to `a3ba576`).** `npm run typecheck` 0 · `npm test` 77/77 (4 files) ·
`cargo test --workspace` 235 passed, 0 failed · `cargo clippy --workspace --all-targets -- -D warnings` 0 ·
`cargo fmt --all -- --check` 0.
**Verified correct (by reading code and test).** C1(a–u): `kindOf` follows evidence §Field kind map exactly (severity
before radio before crowded; `> 32`, `> 4`, `> 48`); each fixture has one sufficient cause (C1(e) joins to 53 chars
with every option ≤ 32; C1(b) would fall to `select` without the severity check). C2(a): per-field 300 ms debounce,
fake timers prove not-at-299/once-at-300 with the typed string. C2(b): invalid value shown with marker + verbatim
text; nothing written until the user picks a choice (a write-back on load would fire inside the 400 ms wait).
C3(a): tab names come from `form_get` counts (fixture counts differ from field counts). C4(a): real dnd-kit drag,
remove, add, default → `setPreset("Chair","details",[k2,k1,k9="Oak"])` once. C4(b): fixed keys lack Remove, Era has
it. C5(a): bars 3/2/1, "Check" on low. C5(b): Looks right → `attribute_values_set` (command is provenance Human,
`commands/attributes.rs:258`) with the stored value; Clear → `attribute_ai_clear`; each excludes the other.
C5(c): pill absent on human, present on AI neighbour. C6(a): picker lists condition photos only, in order; untick
writes exactly `{excluded:[p2]}` once. Emptying a field → `attribute_ai_clear`, which clears any live value incl.
human (`services/attributes.rs:319`). H1–H8, secrets, live services: no violation (form writes local only; works
without integration; no core/Tauri change).
**Findings.**
- **F1 (B1, route: production).** The Product step's Title edits `product.title`, a second copy, while the item's
  title lives in `items.title` (ch05 §items: `title`, owner U, "product title"; ch08 §Identity fields: "Title is a
  Product-step field"). Typed titles never reach the sessions list, the workspace header or `item.json`'s `title`;
  export then carries two different titles (implementer Assumption 3; P10 backlog note "bind before the form
  ships"). Correction: give the title one home, `items.title`: `form_get` presents `items.title` as the
  `product.title` field's value, and a human write or clear of `product.title` updates `items.title` (no separate
  stored row), with one behaviour test at the service boundary; drop the placeholder workaround. Files:
  `crates/core/src/services/attributes.rs`, `crates/core/src/ports/store.rs`, `crates/infra/src/sqlite/store.rs`,
  `crates/testkit/tests/p11_title_binding.rs` (new), `apps/desktop/src/features/form/FormDialog.tsx`,
  `apps/desktop/src/features/form/__tests__/p11_form_ui.test.tsx`, `apps/desktop/src/features/workspace/WorkspacePage.tsx`
  (reload after a title save, optional).
**Notes (backlog, 5).** Customize toggle re-fetches before the layout save lands; own invalid entry empties the
input; `item_type = ""` shares one layout; barcode cannot be cleared; C6(a) fixture order = id order.
**Lessons for the plans.** A backlog note addressed to the next phase ("P10 → P11: bind title before the form
ships") was not folded into P11's rows; notes routed to a named phase need a row or an explicit refusal at
compile time.
