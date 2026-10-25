# 08 — Attributes and AI

## Schema (generic; owned by core)
An attribute definition has key, label, type, choices, validations, group and description.
Core types: `text`, `textarea`, `rich_text`, `integer`, `decimal`, `money`, `dimension`, `weight`,
`boolean`, `date`, `color`, `rating`, `url`, `tags`, `choice` (single), `multi_choice`,
`media_list` (photos from the session), `reference` (opaque; shown as "Edited in <platform>").
Integrations translate their own types into these (chapter 09). The form never reads a platform
type.

## Field kinds (how a definition renders; from the design handoff)
`choice` → pills when ≤ 4 short options, searchable dropdown otherwise, cards when options carry
descriptions, severity scale for the N/A · None · Minor · Moderate · Major set. `multi_choice` →
chips or multi-select dropdown. `dimension` → number + unit. `integer` → stepper. `media_list` →
picker over the session's condition photos, all selected by default. Unknown → "Edited in
Shopify admin". Exact thresholds in `evidence/design_handoff.md`.

## Groups and presets
Three form steps: Details, Condition, Product. Condition holds every definition whose key starts
with the integration's condition prefix plus the condition media list and condition report;
Product holds title, description, tags (never removable). **Customize fields** per item type and
step: reorder, remove, add from the integration's full definition list, default value per field.
Presets are local and pre-fill new sessions of that item type.

## Values and provenance
Each value carries provenance `human`, `ai` or `default`. A human edit makes the field
human-owned; AI never overwrites it. AI values carry confidence (high/medium/low), status
(suggested/accepted/cleared), reason and sources. "Accept all" accepts suggested values except
low-confidence ones. Values save locally as typed; the form never blocks publishing.

## AI agent (architecture now, agent post-MVP — decision C3)
The MVP ships the schema, provenance fields, pills, popover and research-panel UI states driven by
stored provenance, with no agent running. The agent, when added, operates only on the generic
schema: look at the main photo, search, read sources, set suggested values with confidence and
sources, write the description after publish. It must work with no integration configured.

## Identity fields
SKU and barcode are editable from the form header (SKU rules: 05B MC-22). Title is a Product-step field.
