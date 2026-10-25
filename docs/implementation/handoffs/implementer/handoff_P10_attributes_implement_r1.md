---
plan: P10_attributes
role: implement
round: 1
state: DONE
date: 2026-10-25
actor: codex (run on Claude Opus 5.5, per prompt)
---
# P10 attributes: implement r1 handoff

## What was built
An item can now **hold its product information**: the attribute values behind the form that P11 will draw.

- **Definitions.** The fields come from the connected shop (`import_definitions`, which replaces the whole set
  for that integration). With no shop connected, only the three built-in Product fields exist: **Title,
  Description, Tags**. They are always there and can never be removed. Shop fields of type `reference`
  (metaobjects, product links, single files) are always read-only.
- **Values are stored one way only** (MC-9). Every value is saved in its canonical form, for example `"1.50"` → `"1.5"`,
  `#AABBCC` → `#aabbcc`, `sek` → `SEK`, a repeated wood type kept once. Saving a value equal to the stored one writes nothing, so
  nothing gets marked as changed (M16). A value that does not fit its field, such as `"minor"` for a severity field or the
  number `1` for the shop's `"1"`…`"8"` choice, is kept exactly as received and marked invalid (`valid = 0`). It is
  shown, and it stays out of the export.
- **Provenance** (ch08). A value typed by the photographer is human-owned, and the AI can never overwrite it. An AI
  suggestion carries confidence, status, reason and sources. **Accept all** accepts every suggestion except
  low-confidence ones. **Clear** takes the value away from the item.
- **Customize fields (presets)** per item type and step: field order, which fields appear, and default values. A
  new session of that item type is pre-filled with the defaults (provenance `default`). The Product step always keeps
  title, description and tags.
- **The form** (`form_get`). It has three steps (Details, Condition, Product). Each step lists its fields in order (the
  preset if there is one, otherwise every field of the step) with value, provenance, a default flag, a severity flag and a
  `filled/total` count. The **damage report** shows the selected condition photos: all condition photos in condition
  order except the ones excluded. A condition photo added later is selected automatically (MC-14).
- **Side effects.** Every attribute write moves the session to "now" in the sessions list (`last_activity_at`,
  MC-4). It never marks a published session as changed (`change_seq` untouched, MC-6).
- **Export.** `item.json` now carries `attributes`: every valid value, keyed by field key (MC-8, M7).
- **Commands** (registry §6.7): `attributes_definitions_list`, `attribute_values_get`, `attribute_values_set`,
  `attribute_ai_accept_all`, `attribute_ai_clear`, `presets_get`, `presets_set`, `form_get`. Typed bindings were
  regenerated.

Code: `crates/core/src/domain/attribute.rs` (`Definition`, `Value`, `Preset`, `CoreType`, `AttributeGroup`,
`Provenance`, `AiConfidence`, `AiStatus`, form types), `crates/core/src/mechanisms/values.rs` (`canonicalize`,
`is_severity_set`, `selected_condition_photos`), `crates/core/src/services/attributes.rs` (`AttributesService`,
`prefill_defaults`, `builtin_definitions`). The store is extended through `StoreTx`, with no new migration because the P01
tables already had every column.

## Tests → criterion rows
All tests are in `crates/testkit/tests/p10_attributes.rs`. The boundary is the `AttributesService` call plus the stored
`attribute_values` row, read straight from SQLite. C5 also uses `form_get`, C7 the session row, and C8 the produced `item.json`.

| Row | Test |
|---|---|
| C1(a) | `c1a_text_nfc_trailing_whitespace_trimmed` |
| C1(b) | `c1b_textarea_trailing_whitespace_trimmed` |
| C1(c) | `c1c_url_trailing_whitespace_trimmed` |
| C1(d) | `c1d_color_lowercase_or_invalid` |
| C1(e) | `c1e_rich_text_one_blank_line_between_paragraphs` |
| C1(f) | `c1f_integer_only_json_integers` |
| C1(g) | `c1g_decimal_strings_never_floats` |
| C1(h) | `c1h_money_decimal_amount_upper_currency` |
| C1(i) | `c1i_dimension_listed_units_only` |
| C1(j) | `c1j_boolean_only_json_booleans` |
| C1(k) | `c1k_date_only_dates` |
| C1(l) | `c1l_rating_decimal_strings_sorted_keys` |
| C1(m) | `c1m_numeric_looking_choices_stay_strings` |
| C1(n) | `c1n_multi_choice_and_tags_deduplicated_in_order` (both multi_choice and tags) |
| C1(o) | `c1o_media_list_stores_exclusions` |
| C1(p) | `c1p_reference_opaque_valid_never_writable` |
| C2(a) | `c2a_choice_outside_the_set_stored_invalid_verbatim` |
| C2(b) | `c2b_human_edit_to_a_choice_makes_it_valid` |
| C3(a) | `c3a_equal_canonical_value_is_not_written` |
| C4(a) | `c4a_human_edit_is_human_owned` |
| C4(b) | `c4b_ai_never_overwrites_a_human_value` |
| C4(c) | `c4c_ai_suggestion_on_an_empty_field` |
| C4(d) | `c4d_accept_all_skips_low_confidence` |
| C4(e) | `c4e_clear_removes_the_value` |
| C5(a) | `c5a_selection_is_condition_order_minus_exclusions` |
| C5(b) | `c5b_new_condition_photo_selected_by_default` |
| C5(c) | `c5c_exclusion_of_a_photo_no_longer_condition_is_ignored` |
| C6(a) | `c6a_preset_orders_details_and_prefills_defaults` |
| C6(b) | `c6b_product_fields_never_removable` |
| C7(a) | `c7a_attribute_edit_sets_activity_and_keeps_change_seq` |
| C8(a) | `c8a_item_json_attributes_hold_every_valid_value` |

Core-mechanism unit tests (plan **Core mechanisms** list only) are in `crates/core/src/mechanisms/values.rs`:
`mc9_text_nfc_trailing_whitespace_only`, `mc9_rich_text_one_blank_line_between_paragraphs`,
`mc9_color_lowercase_hex_only`, `mc9_integer_is_a_json_integer`, `mc9_decimal_forms`,
`mc9_money_dimension_rating_objects`, `mc9_boolean_date`, `mc9_choices_are_exact_strings`,
`mc9_media_list_and_reference`, `mc9_severity_set`, `mc14_selection_is_condition_order_minus_exclusions`.

## Assumptions
1. **Built-in Product fields are synthesized**, not stored. They use `integration = "core"`; `product.title` is `text`,
   `product.description` is `rich_text` (paragraphs, which matches MC-13's paragraph-to-`<p>` rule), and `product.tags` is
   `tags`. Importing definitions under the name `core`, or a key defined twice, is refused.
2. **`product.title` is an attribute value like any other.** It does not write `items.title`. The item's title from
   creation stays what the sessions list and `item.json` `title` show. Deciding which source feeds the form's title field
   is left to P11/P14.
3. **url is trimmed of trailing whitespace like text.** Row C1(c) says so. MC-9's table names only text and textarea for
   trimming, and I read the plan's row as the narrower statement.
4. **Decimal canonical form also drops redundant leading zeros** (`007.25` → `7.25`) and writes `-0` as `0`, so equal
   numbers are byte-equal. Exponents, `+`, `.5` and `1.` are invalid.
5. **Units.** dimension/weight units come from `validations.units` on the definition. Without such a list any non-empty
   unit is accepted. A rating's value is not range-checked against min/max. Currency must be three ASCII letters.
6. **Tags and multi_choice strings are not NFC-normalized.** They are only de-duplicated (first occurrence kept), and
   multi_choice elements must be in `choices`.
7. **An invalid value is stored as the received JSON, compact** (`serde_json` text of the input).
8. **`clear` keeps the row** with `ai_status = cleared` (for human values too). It disappears from `values()`, `form_get`
   and `item.json`. A later human `set_value` revives it.
9. **Human ownership.** `ai_suggest` is refused on any row whose provenance is `human`, cleared or not, so the human's last
   act on a field wins. `set_value` with a non-human provenance over a live human value is refused too.
   Refusals are `invalid_value`.
10. **A human typing the same value an AI suggested is a write** (provenance becomes `human`). "Equal → no write" compares the
    full stored content: value, validity, provenance and AI fields.
11. **Every attribute write sets `last_activity_at`** of the item's sessions: set, AI suggestion, accept all, clear (MC-4
    "any attribute edit"). Pre-filled defaults do not, because they are written in the session's own creation transaction.
12. **`is_default` = the value's provenance is `default`** (an untouched preset default). **`filled`** counts fields holding
    a valid value; `total` counts the step's fields. Preset keys whose definition no longer exists are skipped in the form.
13. **Fields without a preset** follow definition order: built-ins first, then the integration's import order.
14. **Presets** accept any active definition in any step, but no duplicate keys. Defaults must fit their definition and
    are stored in canonical form. Without a stored preset, `presets_get` returns the implicit layout (every field of the
    step, no defaults).
15. **Values cross the command boundary as JSON text** (`value: string`, `default: string | null`) instead of a JSON type in the
    bindings. `attribute_values_set` always records provenance `human`. `ai_suggest` has no command (agent post-MVP).
16. **Active definitions in the app** = those of `settings.integration.kind`. Until P12 imports any, the app shows only
    Title, Description and Tags.

## Deviations
- **Pre-fill hook outside the listed files.** `crates/core/src/services/items.rs` (`create_local`) and
  `crates/core/src/services/sessions.rs` (`create`) each gained one call to `attributes::prefill_defaults` inside their
  existing transaction. Without it, "new sessions of a type pre-fill defaults" (task 3, row C6(a)) has no place to happen.
- **`serde_json` added to `apps/desktop/src-tauri/Cargo.toml`** (a workspace dependency already in the lock). The
  command layer needs it to parse the JSON text of a value. `Cargo.lock` changed by one line.
- **Tracker actor.** The prompt asked for actor `codex`. This session ran on Claude Opus 5.5, and flow.md §2 (owner,
  2026-10-25) names Opus as the implementer, so the tracker row says `opus`. The frontmatter follows P09's precedent.
- No new migration. No other files.

## Files changed
Code and tests (checkpoint commit):
- new: `crates/core/src/domain/attribute.rs`, `crates/core/src/mechanisms/values.rs`,
  `crates/core/src/services/attributes.rs`, `apps/desktop/src-tauri/src/commands/attributes.rs`,
  `crates/testkit/src/fixtures/definitions.rs`, `crates/testkit/tests/p10_attributes.rs`
- modified: `crates/core/src/domain/mod.rs`, `crates/core/src/mechanisms/mod.rs`, `crates/core/src/services/mod.rs`,
  `crates/core/src/ports/store.rs`, `crates/core/src/services/export.rs`, `crates/core/src/services/items.rs`,
  `crates/core/src/services/sessions.rs`, `crates/infra/src/sqlite/store.rs`, `crates/testkit/src/fixtures/mod.rs`,
  `apps/desktop/src-tauri/src/commands/mod.rs`, `apps/desktop/src-tauri/src/lib.rs`,
  `apps/desktop/src-tauri/Cargo.toml`, `Cargo.lock`, `apps/desktop/src/services/bindings.ts` (generated)
- docs: `docs/implementation/master_plan.md` (P10 tracker row only), `docs/implementation/plans/P10_attributes.md`
  (Review log)

Not committed by me (the runner commits it): this handoff.

## Checkpoint SHA
`51b99f6`: `CHECKPOINT (not approved): P10 attributes — canonical values, provenance, presets, damage-report selection`

## Stamp
| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test p10_attributes` | 31 passed |
| `cargo test -p psm-core values::tests` | 11 passed |
| `cargo test --workspace` | all green (every binary `ok`, 0 failed) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo test -p psm-desktop export_bindings` | bindings regenerated; the committed file equals the regenerated one |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | 3 files, 45 tests passed |
No pre-existing failures. Note for the runner: in this shell `cargo` lives in `~/.cargo/bin`, which is not on the
default `PATH`. I ran with `PATH="$HOME/.cargo/bin:$PATH"`.

## Manual try-out
There is no form screen yet (P11). To see values stored:
1. `cd apps/desktop && PSM_DATA_DIR=/tmp/psm-p10 npm run tauri -- dev`, then create a session for a new item
   (for example SKU `P10-1`).
2. Open the web inspector (right-click → Inspect) and in the console run:
   `await window.__TAURI_INTERNALS__.invoke('attribute_values_set', {itemId: 1, key: 'product.tags', value: '["Oak","Teak","Oak"]'})`
   and `await window.__TAURI_INTERNALS__.invoke('form_get', {itemId: 1})`. The form shows the Product step with Tags filled
   (`["Oak","Teak"]`), and `filled: 1, total: 3`.
3. In Terminal: `sqlite3 /tmp/psm-p10/session_manager.db 'select key, value, valid, provenance from attribute_values;'`
   shows `product.tags|["Oak","Teak"]|1|human`.
4. With photos reviewed, press **Export**. The item's `item.json` in the export folder now has
   `"attributes": {"product.tags": ["Oak", "Teak"]}`.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
