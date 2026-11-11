---
phase: P67_form_values_defaults_and_batch_step
title: The product form shows what is already on the Shopify product, every field has a default and saved values per item type, the form never picks a product type on its own, and batch forms open on the step you were on
difficulty: high
projection: no
depends_on: [P66_approve_on_arrival_and_session_order]
---
# P67 — Form values from Shopify, defaults and saved values, no auto type, batch step memory (owner 2026-11-11, changelog 46 (3)–(6))

Written by the orchestrator (Opus) on 2026-11-11. Owner: "for the shopify product form (details) I don't see the already added info on that product, the form
always comes blank"; "on the fields that are added I should be able to place predetermined values so that I can fill up quickly with already saved values,
this goes for all the customizable fields"; "about the default values on the form is important as I just spent 5 minutes … going one after the other placing
the same values"; "when I open the detail form for some reason the product type gets autoselected"; "when doing batches for publish … if I was on page 3 I
would like the next form to remember this".

## Goal
1. **Shopify values in the form (08 Values).** Opening the form of an item linked to a Shopify product reads the product once (title, description, product
   type, tags, and the values of every metafield definition the form shows, condition fields included) and shows them. Provenance `shopify` (read-only
   label "from Shopify"); a local edit (human/ai/default) wins; reopening keeps local edits. While reading: fields show a quiet loading state, never
   blank-then-jump; read failure → `Couldn't read this product from Shopify` and the form stays editable with local values. Publishing sends what the form
   shows (no change to 07A write rules beyond that). Batched reads for a batch of forms (≤ 1 request per product, cached for the form session).
2. **Defaults and saved values (08 Groups and presets), per item type.** In every customizable field (all kinds the form renders: text, number, choice,
   multi-choice, metaobject reference, rich text, tags) the photographer can (a) **Save as default** — fills that field for new items of that type and for
   linked items whose Shopify value is empty (provenance `default`, never overwriting human/ai/shopify values); (b) **Save value** — adds the current
   value to the field's saved values; the field shows saved values as one-click picks (most recent first, removable). Also managed in Customize fields
   (the existing "default value per field" there must work and be the same data). Stored locally per (item type, field key). One action
   **Fill defaults** on the form fills every empty field that has a default.
3. **No automatic product type.** The form never selects a product type the photographer did not choose: an item without a type shows the picker empty;
   a linked item shows Shopify's type; a new item shows the type chosen at session creation, if any. Find and remove the cause (write it in the handoff).
4. **Batch step memory.** In batch publish, each next form opens on the step (Details / Condition / Product) and scroll section the photographer was on in
   the previous form; a single form opened outside a batch opens as today.

## Read first
Changelog 46, 42 (item type from Shopify), 45 (stock in the Product step); book 08, 07A, 09/09A. Code: `apps/desktop/src/features/form/*` (FormDialog,
CustomizeMode, ItemTypePicker, StepTabs, useFormState, MetafieldInput, inputs/*), `apps/desktop/src/features/publish/BatchQueue.tsx`,
`apps/desktop/src/services/attributes.ts`, `crates/core/src/services/attributes*.rs`, `crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs}`
(product + metafield reads), store (presets). Tests: form UI tests, p59, p64, p65 testkit files.

## Files expected to change (write perimeter)
The files above as needed; `crates/testkit/tests/p67_form_values_defaults_and_batch_step.rs` (new); form UI tests (new). ≈ 22 paths, ≈ 800 lines.

## Criteria (5 criteria, 8 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | Shopify values shown | linked item; fake product has title, description, type, tags, 3 metafields | form shows all of them labelled from Shopify; no blank flash | `[rule]` changelog 46 (3) |
| C1(b) | local edit wins; read failure | edit one field, reopen; product read fails | edited value kept; failure sentence; form editable | `[rule]` changelog 46 (3) |
| C2(a) | default per item type | Save as default on a field for type Sofa; new Sofa item; new Chair item; linked Sofa with that Shopify value empty / set | Sofa new: filled (default); Chair: empty; linked empty: filled; linked set: Shopify value kept | `[rule]` changelog 46 (4) |
| C2(b) | saved values | Save value twice on a choice field and a text field; pick one; remove one | picks listed most recent first; picking fills the field (human); removed gone; Customize shows the same default | `[rule]` changelog 46 (4) |
| C2(c) | Fill defaults | form with 5 empty fields, 3 with defaults, 1 human-filled | the 3 filled, the human value untouched | `[rule]` changelog 46 (4) |
| C3(a) | no automatic type | new item without type; open form | picker empty, nothing saved; cause in the handoff | `[rule]` changelog 46 (5) |
| C4(a) | batch step memory | batch of 3; on form 1 go to Product, Next | form 2 opens on Product; single form outside a batch opens on Details | `[rule]` changelog 46 (6) |
| C5(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (known intermittent `p33_split_grid` aside), clippy, fmt, typecheck green | `[rule]` master plan §7.2 |

## Notes
- **Hard rules:** no real Shopify in tests; never write to Shopify on form open; no photos deleted. Default implementer (Codex; Claude when Codex is exhausted).

## Review mode (owner 2026-11-10: fast; kept)
- **Implementer:** high effort; run the full closing gate once (`cargo test --workspace`, clippy, fmt, typecheck, `npm test`) and put the stamp in the handoff.
- **Reviewer (time-boxed, still correct):** do not re-run `cargo test --workspace`. Run this phase's new test file(s), the test files of the crates and features the diff touches, clippy, fmt, typecheck and the touched UI test files; trust the implementer's full-suite stamp unless it is missing or contradicts what you see. Read every changed production path against the rows; only B1–B4 findings block. Aim to finish in about 15 minutes.

## Review log

### 2026-11-11 — implement r1 (Claude Opus, `codex` actor slot)
**Built.** (1) Shopify values in the form: new port method `CommerceAdapter::product_values` (Shopify: one GraphQL query
`ProductValues` — title, productType, tags, descriptionHtml, metafields first 250; only a product with more than 250 metafields pages on
with `MetafieldsRead`); `AttributesService::read_integration_values` stores them as values with the new provenance `integration`; the
command `form_read_integration(item_ids)` reads a batch (one request per product). The form of a linked item shows a skeleton until the
read answers, then the fields labelled "from Shopify" (title, description, tags, metafields incl. condition fields, item type next to the
picker); a failed read shows `Couldn't read this product from Shopify` and the local values stay editable. The batch reads every linked
item of the queue once, together, and each form waits for its own result (the form's own `refreshItem` call is gone: one request per
product). Nothing is written to Shopify. (2) Defaults and saved values per item type: "Save as default" (stored in the item type's step
layout, so Customize fields shows and edits the same default), "Save value" (new table `attribute_saved_values`, migration 013), saved values
as one-click picks under the field (most recent first, × removes), "Fill defaults" in the footer (one undoable step). (3) No automatic
type: see Assumptions A5. (4) Batch step memory: `FormDialog` takes `position`/`onPosition`; `BatchQueue` keeps the last step and scroll and
opens the next form there; a single form opens on Details.

**Assumptions.** A1 (H7) the stored provenance is `integration`, not `shopify`: core tables never name a platform (H7); the UI label says
"from Shopify". A2 precedence: human and AI values, and fields the photographer cleared, are never replaced by a read; a `default` or an
earlier read is replaced by a non-empty product value (C2(a) "linked set: Shopify value kept" wins over Goal 1's "(human/ai/default) wins");
a field the product leaves empty gets the type's default; an earlier read whose product value is now empty is removed. A3 the title has one
home (`items.title`): the read sets it only while the local title is empty or still the last-read title (new table `item_remote_titles`);
the title shows "from Shopify" while it equals the last-read title. The type follows the changelog-42 rule already used by refresh. A4 no
"Save as default"/"Save value" on Title (a per-item value), on the damage report (per session) and on `reference` fields (metaobject
references are read-only "Edited in Shopify admin", never written, MC-13); every other kind the form renders has them. A5 the cause of
the automatic type: on open the dialog put focus in its first focusable control, the Item type picker (an `<input list>`); with focus there
WebKit's suggestion list takes arrow/Return keys and each suggestion it shows in the field is an `input` event, and the form saved any
listed type on every `input` event (`onChange` → `commitType`). Removed both: the form no longer focuses the picker on open
(`onOpenAutoFocus` prevented) and a type is saved only on leaving the picker or Return (or Save & close). A6 a linked item with no stored
link (`not_linked`) reads nothing; a linked item with no integration configured shows the failure sentence. A7 scroll memory is the
fields column's `scrollTop`.

**Deviations.** Test files changed because a DTO gained fields (no behaviour change): UI `p11_form_ui`, `p22_form_tidy`, `p41_undo_ui`
fixtures (`default`, `saved`, `item_type_from_integration`); `p01_storage_identity` table list (+2 tables). Production files outside the
listed ones: `crates/core/src/domain/{attribute,publish}.rs`, `crates/core/src/ports/{commerce,store}.rs`,
`crates/infra/src/sqlite/{store.rs, migrations/013_form_values.sql}`, `apps/desktop/src-tauri/src/{commands/attributes.rs, lib.rs}`,
`apps/desktop/src/services/bindings.ts` (regenerated), `crates/testkit/src/fake_commerce.rs`.
