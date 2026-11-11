---
plan: P67_form_values_defaults_and_batch_step
role: implement
round: 1
state: DONE
date: 2026-11-11
actor: codex
---
# Handoff — P67 implement r1 (Claude Opus in the `codex` actor slot)

Gate: intention header RATIFIED; tracker P66 APPROVED, P67 NOT_STARTED → passed.

## What was built
- **The product form shows what is already on the Shopify product.** Opening the form of a linked item reads the product once
  (title, description, product type, tags, every metafield incl. condition fields) and shows the values with a small "from Shopify"
  label (also next to the Item type picker). While the read runs, the fields show a quiet loading skeleton (no blank fields that then
  jump). If the read fails, the form says `Couldn't read this product from Shopify` and stays editable with the local values. A value the
  photographer typed, picked or cleared here always wins over what the shop holds; reopening keeps it. Nothing is ever written to
  Shopify on open; publishing still writes only what differs (07A MC-13 unchanged).
- **Defaults and saved values per item type.** Every editable field (text, number, choice, multi-choice, rich text, tags, dimension …)
  has **Save as default** and **Save value**. A default fills that field for new items of the type and for linked items whose shop value is
  empty (marked "Default"; never over a human, AI or shop value). Saved values show as one-click picks under the field, newest first, each
  with × to remove; a pick fills the field as the photographer's own value. The default is stored in the type's step layout, so
  **Customize fields shows and edits the same default**. **Fill defaults** (form footer) fills every empty field that has a default, in one
  undoable step.
- **No automatic product type.** The form no longer focuses the Item type picker on open, and a type is saved only when the photographer
  confirms it (leaves the picker, presses Return, or Save & close). Cause: see Assumptions A5.
- **Batch step memory.** In batch publish, the next form opens on the step (Details / Condition / Product) and scroll position of the
  previous form; a single form opens on Details as before. The batch reads all its linked products once, together (one request each).

Mechanics: port `CommerceAdapter::product_values` (Shopify: one `ProductValues` GraphQL query; fake: one read op `product_values`);
`AttributesService::{read_integration_values, default_set, saved_values, saved_value_add, saved_value_remove, fill_defaults}`;
`Provenance::Integration`; `FormField.{default, saved}`, `Form.item_type_from_integration`; migration `013_form_values.sql`
(`attribute_saved_values`, `item_remote_titles`); commands `form_read_integration`, `attribute_default_set`, `attribute_saved_value_add`,
`attribute_saved_value_remove`, `attribute_fill_defaults`; UI in `FormDialog`, `MetafieldInput`, `ItemTypePicker`, `BatchQueue`.

## Tests → criterion rows
| Row | Test(s) | Boundary / assertion shape |
|---|---|---|
| C1(a) | `p67…rs::c1a_linked_item_form_shows_the_products_values_from_shopify`; `c1a_shopify_adapter_reads_the_form_values_in_one_request`; UI `p67_form_values_ui.test.tsx` "C1(a) …" | service + `form_get`: title, description, tags, 3 metafields (one a condition field) shown with provenance `integration`, type `Sofa` from the product, exactly one shop read op and no mutation; real adapter: one GraphQL request yields all values (condition-photos field excluded, HTML → paragraphs); UI: skeleton and no field while the read is pending, then values each labelled "from Shopify", type labelled |
| C1(b) | `c1b_local_edit_wins_on_reopen_and_a_failed_read_keeps_the_form_editable`; UI "C1(b) …" | edited field keeps the human value after a re-read while an untouched field follows the shop; failed read → `RemoteRead::Failed`, values kept, a later edit stored; UI: the exact sentence, field editable and saved |
| C2(a) | `c2a_save_as_default_fills_new_items_of_that_type_and_empty_linked_values` | Sofa new: `Oak` (default); Chair: none; linked Sofa empty: `Oak` (default); linked Sofa set: `Walnut` (integration) |
| C2(b) | `c2b_saved_values_most_recent_first_pick_remove_and_customize_shows_the_default`; UI "C2(b) …" | saved lists newest first for a choice and a text field; pick stored as human; removed value gone; `presets_get` (Customize) shows the default saved from the form; UI: chip order, pick → `setAttributeValue`, remove/save value/save default called with the item type |
| C2(c) | `c2c_fill_defaults_fills_only_the_empty_fields_that_have_a_default`; UI "C2(c) …" | 6 fields (5 empty, 3 with defaults, 1 human with a default) → returns 3, the 3 filled as default, 2 empty, human untouched; UI: button calls the service and the field shows the filled default |
| C3(a) | `c3a_new_item_without_a_type_keeps_an_empty_type_when_its_form_opens`; UI "C3(a) …" | open (read + form): type stays `""`, no value stored, no shop read; UI: picker empty and not focused, keys and a shown suggestion save nothing, leaving the picker saves the chosen type |
| C4(a) | UI "C4(a) …" | batch of 3: Product on form 1, Confirm & next → form 2 on Product; batch read called once with all ids; single form opens on Details |
| C5(a) | closing gate (Stamp) | all green |

No test outside these rows was added. Core mechanisms list in the plan: none, so no unit tests.

## Assumptions
- **A1 (H7):** the stored provenance is `integration`, not `shopify`, because core tables never name a platform (H7). The UI label says "from Shopify".
- **A2 precedence:** human and AI values, and fields the photographer cleared, are never replaced by a read. A default or an earlier read is replaced
  by a non-empty shop value. This follows C2(a) "linked set: Shopify value kept" over Goal 1's "(human/ai/default) wins". A field the
  shop leaves empty gets the type's default. An earlier read whose shop value is now empty is removed.
- **A3 title:** the title still has one home (`items.title`). A read sets it only while the local title is empty or still equals the last-read
  title (`item_remote_titles`). It shows "from Shopify" while it equals the last-read title. The type follows the existing changelog-42
  rule (`item_type_base`).
- **A4:** Title (per item), the damage report (per session) and read-only `reference` fields (metaobject references: "Edited in Shopify admin",
  never written per MC-13) get no Save as default / Save value. Every other kind the form renders has them.
- **A5 cause of the automatic type:** on open, the dialog put focus in its first focusable control, the Item type picker (an
  `<input list>`). With focus there, WebKit's suggestion list takes the arrow and Return keys. Each suggestion it shows in the field is an
  `input` event, and the form saved any listed type on every `input` event (`onChange` → `commitType`). The fix removes both: no focus
  on open (`onOpenAutoFocus` prevented), and a type is saved only on blur, Return or Save & close. This explanation comes from reading
  the code. It was not reproduced in the real WebKit window.
- **A6:** an item with no stored link reads nothing (`not_linked`). A linked item while no integration is configured shows the failure sentence.
- **A7:** "scroll section" = the fields column's scroll position.
- **A8:** the batch shows its loading rows until the batch read is under way, so a form never starts a second read of its own.

## Deviations
- Test files changed only because a DTO gained fields (no behaviour change): UI `p11_form_ui.test.tsx`, `p22_form_tidy.test.tsx`,
  `p41_undo_ui.test.tsx`. The table list in `p01_storage_identity.rs` gained the 2 new tables.
- Production files outside the plan's named list (needed by the listed ones): `crates/core/src/domain/{attribute,publish}.rs`,
  `crates/core/src/ports/{commerce,store}.rs`, `crates/infra/src/sqlite/store.rs`, `crates/infra/src/sqlite/migrations/013_form_values.sql`
  (new), `apps/desktop/src-tauri/src/{commands/attributes.rs, lib.rs}`, `apps/desktop/src/services/bindings.ts` (regenerated),
  `crates/testkit/src/fake_commerce.rs`.
- `refreshItem` is no longer called by the form; the read replaces it, keeping one request per product.

## Files changed
Declared write perimeter (this session, all in the checkpoint commit):
- `apps/desktop/src-tauri/src/commands/attributes.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src/features/form/FormDialog.tsx`
- `apps/desktop/src/features/form/ItemTypePicker.tsx`
- `apps/desktop/src/features/form/MetafieldInput.tsx`
- `apps/desktop/src/features/form/__tests__/p11_form_ui.test.tsx`
- `apps/desktop/src/features/form/__tests__/p22_form_tidy.test.tsx`
- `apps/desktop/src/features/form/useFormState.ts`
- `apps/desktop/src/features/history/__tests__/p41_undo_ui.test.tsx`
- `apps/desktop/src/features/publish/BatchQueue.tsx`
- `apps/desktop/src/services/attributes.ts`
- `apps/desktop/src/services/bindings.ts`
- `crates/core/src/domain/attribute.rs`
- `crates/core/src/domain/publish.rs`
- `crates/core/src/ports/commerce.rs`
- `crates/core/src/ports/store.rs`
- `crates/core/src/services/attributes.rs`
- `crates/infra/src/sqlite/store.rs`
- `crates/integrations/src/commerce/shopify/adapter.rs`
- `crates/integrations/src/commerce/shopify/graphql.rs`
- `crates/testkit/src/fake_commerce.rs`
- `crates/testkit/tests/p01_storage_identity.rs`
- `docs/implementation/master_plan.md`
- `docs/implementation/plans/P67_form_values_defaults_and_batch_step.md`
- `apps/desktop/src/features/form/__tests__/p67_form_values_ui.test.tsx`
- `crates/infra/src/sqlite/migrations/013_form_values.sql`
- `crates/testkit/tests/p67_form_values_defaults_and_batch_step.rs`
- `docs/implementation/handoffs/implementer/handoff_P67_form_values_defaults_and_batch_step_implement_r1.md`

No mutation probes were run (flow §4: regime off); no file was changed and reverted.

## Checkpoint SHA
`a4ee3aec5a79a9b25cd41a19b7da5aad885de566` (27 paths; this handoff is left for the runner to commit)

## Stamp
Tree: the checkpoint commit below (code identical to the tree the gate ran on; only docs changed after).
| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no warnings |
| `cargo test --workspace` | 54 test binaries ok, then `p61_storage_cleanup_and_budget::c1a_a_verified_duplicate_goes_24_hours_after_it_was_diverted` failed once under the full-suite load (camera/FTP storage path, untouched by P67) and cargo stopped there. Re-run alone: `cargo test -p psm-testkit --test p61_storage_cleanup_and_budget` → 7 passed. The binaries after it were run with `cargo test --no-fail-fast -p psm-testkit --test p62… --test p67…` → p62 12, p63 4, p64 8, p65 8, p66 6, p67 7 passed; `cargo test --no-fail-fast -p psm-desktop` → 20 passed (incl. `export_bindings`, committed bindings equal the regenerated file) |
| `npm run typecheck` | exit 0 |
| `npm test` | 67 files, 459 tests passed (`p33_split_grid` did not flake this run) |

Pre-existing/intermittent: the p61 c1a failure above (timing under load; it passes alone; P67 changes nothing on that path).

## Manual try-out
1. `cd apps/desktop && npm run tauri -- dev` (Shopify connected in Settings).
2. Open a session whose item was found in the shop and open its product form: the fields shimmer for a moment, then show the product's
   title, description, tags, type and metafields, each marked "from Shopify". Change one field, close, reopen: your value stays.
3. In any field press **Save value** twice with different values: they appear as picks under the field, newest first; click one to fill the
   field, × to remove one. Press **Save as default** on a field, then open **Customize fields**: the same default is shown there.
4. Create a new session of that item type: the field starts with the default. On an older item press **Fill defaults**: only empty fields fill.
5. Create a new item without choosing a type and open its form: the Item type stays empty until you pick one and leave the picker.
6. Select three sessions and start a batch publish; on the first form go to **Product** and press **Confirm & next**: the second form opens
   on Product.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
