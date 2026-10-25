---
phase: P10_attributes
title: Attributes — definitions, canonical values, provenance, presets, damage-report selection
difficulty: high
projection: no
depends_on: [P09_local_export]
---
# P10 — Attributes: definitions, canonical values, provenance, presets, damage-report selection

## Goal
Implement the generic attribute model of chapter 08 over the schema: core-typed definitions imported from an
adapter (or empty without one), values stored only in canonical form with byte-equality semantics (MC-9, M16),
`valid = false` for ill-fitting remote values, provenance rules (human wins, accept-all skips low), presets per item
type and step (Customize fields), the damage-report selection as exclusions (MC-14), form data (`form_get`), and the
side effects on `last_activity_at` and `change_seq`. Also fills `item.json`'s `attributes` (MC-8). **Not in this
phase:** the form UI (P11), the Shopify type map and schema import (P12), writes to the platform (P14), any AI agent.

## Read first
master plan §6.5 (`attribute_*` tables), §6.6 (`CoreType`, `Provenance`, …), §6.7 (P10 commands) · book `01`, `14`,
`08` (all), `05` §attribute schema and values, `04` S-F · contracts `05A` MC-9 (table, invalid values, severity set),
MC-8 (item.json attributes), MC-6 (attribute edits do not change `change_seq`), MC-4 (`last_activity_at`); `07A`
MC-14 (selection semantics) · `claude_design/reference/SHOPIFY_PRODUCT_METAFIELDS.md` (the shape of real definitions —
used to build the core-typed test fixture, not imported here).

## Files expected to change
`crates/core/src/domain/attribute.rs` (new: `Definition`, `Value`, `Preset`), `crates/core/src/mechanisms/values.rs`
(new: canonical forms), `crates/core/src/services/attributes.rs` (new), `crates/core/src/services/export.rs` (values into
item.json), Store trait + impl (definitions, presets, values), `apps/desktop/src-tauri/src/commands/attributes.rs` (new:
`attributes_definitions_list`, `attribute_values_get`, `attribute_values_set`, `attribute_ai_accept_all`, `attribute_ai_clear`,
`presets_get`, `presets_set`, `form_get`), bindings, `crates/testkit/src/fixtures/definitions.rs` (new: a core-typed
definition set modelled on the shop's 68 definitions, incl. the severity fields and `custom.damage_report`),
`crates/testkit/tests/p10_attributes.rs` (new).

## Core mechanisms (unit tests allowed)
- `mechanisms::values::canonicalize(core_type, &Definition, serde_json::Value) -> Result<CanonicalJson, Invalid>` (MC-9 table).
- `mechanisms::values::is_severity_set(choices) -> bool`.
- `mechanisms::values::selected_condition_photos(condition_in_order, excluded) -> Vec<PhotoId>` (MC-14).

## Ordered tasks
1. `Definition{integration, key, label, core_type, choices, validations, group, description, writable}`;
   `AttributesService::import_definitions(integration, defs)` (replace set for that integration);
   `definitions(integration)`. Without an integration the definition set is empty except the three built-in Product
   fields `product.title`, `product.description`, `product.tags` (always present, never removable).
2. `canonicalize` per MC-9 (canonical JSON = `serde_json` with sorted object keys, no whitespace); `set_value(item,
   key, raw_json, provenance)`: canonical → store (`valid = 1`), equal canonical → no write; non-fitting → store
   verbatim `valid = 0`; `ai_suggest(item, key, value, confidence, reason, sources)` refused on `human` values;
   `accept_all(item)` (suggested, not low → accepted); `clear(item, key)`; `values(item)`.
3. Presets: `presets_get(item_type, group)`, `presets_set(layout)`; `form_get(item_id)` → three groups with ordered
   fields (preset layout, else all definitions of the group), each with definition, value, provenance, `is_default`,
   and per-group `filled/total`; new sessions of a type pre-fill defaults (provenance `default`).
4. Damage report: `selected_condition_photos`; `form_get` includes the selected ids for `media_list` fields.
5. Side effects: `set_value` sets `last_activity_at`, never touches `change_seq`. `item.json` `attributes` = every
   valid value keyed by `key`.
6. Commands, bindings, tests.

## Criteria (8 criteria, 31 rows; 7 `[rule]` rows)
| Row | Boundary | Input | Exact outcome (stored canonical JSON unless noted) | Trace |
|---|---|---|---|---|
| C1(a) | `set_value` text | `" Hello  "` (NFD `é` inside) | `" Hello"` with NFC; trailing whitespace trimmed only | M16, MC-9 |
| C1(b) | textarea | `"a \n"` | `"a"` | MC-9 |
| C1(c) | url | `"https://x.se/ "` | `"https://x.se/"` | MC-9 |
| C1(d) | color | `"#AABBCC"` / `"red"` | `"#aabbcc"` / invalid (`valid = 0`, stored verbatim) | MC-9 |
| C1(e) | rich_text | `"a\n\n\nb"` | `"a\n\nb"` | MC-9 |
| C1(f) | integer | `12` / `12.0` / `"12"` | `12` / invalid / invalid | MC-9 |
| C1(g) | decimal | `"1.50"` / `"2.0"` / `"-0.10"` / `"1e3"` / `1.5` (number) | `"1.5"` / `"2"` / `"-0.1"` / invalid / invalid (never a float) | MC-9 |
| C1(h) | money | `{"amount":"10.50","currency":"sek"}` | `{"amount":"10.5","currency":"SEK"}` | MC-9 |
| C1(i) | dimension (units `cm, mm, in`) | `{"value":"120.0","unit":"cm"}` / unit `"ft"` | `{"unit":"cm","value":"120"}` / invalid | MC-9 |
| C1(j) | boolean | `true` / `"true"` | `true` / invalid | MC-9 |
| C1(k) | date | `"2026-10-25"` / `"2026-10-25T10:00:00Z"` | as is / invalid | MC-9 |
| C1(l) | rating (min 1, max 5) | `{"value":"4","min":"1.0","max":"5.0"}` | `{"max":"5","min":"1","value":"4"}` | MC-9 |
| C1(m) | choice (`1`…`8`) | `"1"` / `1` / `"9"` | `"1"` / invalid / invalid | M16 (numeric-looking choices stay strings) |
| C1(n) | multi_choice / tags | `["Oak","Teak","Oak"]` | `["Oak","Teak"]` (de-duplicated, order kept) | MC-9 |
| C1(o) | media_list | `{"excluded":[3,1]}` | `{"excluded":[3,1]}` | MC-9, MC-14 |
| C1(p) | reference | any JSON | stored verbatim, `valid = 1`, `writable = false` on the definition | MC-9 (opaque) |
| C2(a) | `set_value` choice | `"minor"` for a severity field | `valid = 0`, value `"minor"` verbatim | MC-9 (invalid stored, shown) |
| C2(b) | then `set_value` `"Minor"` with provenance human | — | `valid = 1`, value `"Minor"` | MC-9 |
| C3(a) | `set_value` twice with the same canonical value (`"1.50"` then `"1.5"`) | `ManualClock` advanced between calls | `updated_at` and `last_activity_at` unchanged after the second call | M16 (equal → no write) |
| C4(a) | `set_value(provenance human)` | — | `provenance = human` | `[rule]` ch08 §Values and provenance |
| C4(b) | `ai_suggest` on a human value | — | refused; value and provenance unchanged | `[rule]` ch08 (human wins) |
| C4(c) | `ai_suggest` on an empty field | confidence `medium` | `provenance = ai`, `ai_status = suggested`, `ai_confidence = medium` | `[rule]` ch08 |
| C4(d) | `accept_all` | suggested high, medium, low | high and medium → `accepted`; low stays `suggested` | `[rule]` ch08 (Accept all skips low) |
| C4(e) | `clear(key)` | an ai value | `ai_status = cleared`, value removed from `values(item)` | `[rule]` ch08 |
| C5(a) | `selected_condition_photos` via `form_get` | condition [p1,p2,p3], exclusions `[p2]` | selected `[p1, p3]` | M21 (selection) |
| C5(b) | same after a new condition photo p4 | — | `[p1, p3, p4]` (selected by default) | M21 |
| C5(c) | same after p2 → main | exclusions still `[p2]` | `[p1, p3, p4]`; stored value unchanged | MC-14 (ids no longer condition are ignored) |
| C6(a) | `presets_set(type "Sideboard", details, [{key:k1, default:"Oak"}, {key:k2}])` then `form_get` on a new Sideboard item | — | Details fields in order k1, k2; k1 value `"Oak"` with `provenance = default`, `is_default = true` | `[rule]` ch08 §Groups and presets |
| C6(b) | `presets_set(product, layout without product.title)` | — | `Err(invalid_value)`; Product group always lists title, description, tags | `[rule]` ch08 (never removable) |
| C7(a) | `set_value` | `ManualClock` at t; session `change_seq = n` | `last_activity_at = t`; `change_seq = n` (unchanged) | MC-4, M13 (attribute edits do not dirty) |
| C8(a) | `export_run` after values set | `custom.years = "1950-1960s"`, an invalid value on another key | `item.json` `attributes` is exactly `{"custom.years": "1950-1960s"}` (invalid omitted) | MC-8, M7 |

## Delegated choices
Whether canonical JSON uses `serde_json::Map` with `preserve_order` off (sorted) or a BTreeMap — sorted keys either
way; the shape of `FormGroup` DTOs; whether built-in Product fields are stored as definitions with
`integration = "core"` or synthesized; debounce lives in the UI, not here.

## Manual try-out
No form UI yet (P11). The owner can wait for P11; the reviewer can show `sqlite3 … 'select key, value, valid,
provenance from attribute_values;'` after setting a value through the dev tools if the implementer exposes one.

## Notes
- Without an integration the form has only Product fields (title, description, tags) plus SKU/barcode; the shop's
  68 definitions arrive via P12.
- Estimated size: ~9 files, ~1 600 lines.

## Review log
- **2026-10-25 implement r1 (opus).** Built: `domain::attribute` (`CoreType`, `AttributeGroup`, `Provenance`,
  `AiConfidence`, `AiStatus`, `Definition`, `Value`, `Preset`, form types), `mechanisms::values` (`canonicalize`,
  `is_severity_set`, `selected_condition_photos`, unit tests `mc9_*`/`mc14_*`), `services::attributes`
  (`AttributesService`: import/definitions, set_value, ai_suggest, accept_all, clear, values, presets_get/set,
  form_get; `prefill_defaults`), StoreTx methods + SQLite impl (no new migration), item.json `attributes` in the
  export, 8 Tauri commands + bindings, fixture `shop_definitions()` (68 shop definitions in core types + 6 `test.*`),
  `tests/p10_attributes.rs` (31 tests, one per row). Assumptions (handoff): built-in Product fields synthesized
  (`integration = "core"`, description = `rich_text`); url trimmed like text (row C1(c)); decimals also drop
  redundant leading zeros and `-0`; `clear` keeps the row with `ai_status = cleared`; `is_default` = provenance
  `default`; `filled` counts valid values; every attribute write (incl. AI) sets `last_activity_at`; values cross
  the command boundary as JSON text. Deviations: pre-fill hooked into `ItemsService::create_local` and
  `SessionsService::create` (one line each), `serde_json` added to `psm-desktop`, tracker actor `opus` (not `codex`).

### 2026-10-25 — review r1 (opus, `pipeline-reviewer`): **APPROVED_WITH_NOTES**
**Gate.** Tracker `IMPLEMENTED`; checkpoint `51b99f6` in `git log`. HEAD `eed32c2` adds only the implement handoff and
review prompt; `git status` clean. **Perimeter:** `git show --stat 51b99f6` lists 22 paths, equal to the handoff's
declared list (6 new, 14 modified, 2 docs). Outside the plan's expected files: `items.rs`/`sessions.rs` (one pre-fill
call each, needed by C6(a)), `src-tauri/Cargo.toml` + `Cargo.lock` (`serde_json`), module wiring. All declared.
**Stamp** on that tree, all exit 0: `cargo test --workspace` (P10 31/31), `cargo clippy --workspace --all-targets -- -D
warnings`, `cargo fmt --all -- --check`, `npm run typecheck`, `npm test` (45/45). Bindings regenerated by the suite with
no diff.
**Rows.** C1(a)–(p): `canonicalize` implements each MC-9 form; each test asserts the exact stored `(value, valid)` read
from SQLite, invalid cases verbatim with `valid = 0`. C2(a)/(b): severity choice is case-sensitive; human re-edit gives
`valid = 1`, `human`. C3(a): `write` skips a row with the same content, and the test asserts that `updated_at` and
`last_activity_at` stay unchanged after the clock advances. C4(a)–(e): human ownership, refusal with the row unchanged,
`ai/suggested/medium`, accept-all skips low, cleared row absent from `values`. C5(a)–(c): `form_get` selection = condition
photos sorted by `(position, id)` minus exclusions; the stored value is untouched. C6(a)/(b): preset order + pre-fill
`default`/`is_default`; Product layout without title refused, form keeps all three. C7(a): `set_item_activity` updates
only `last_activity_at`. C8(a): export snapshot keeps valid live values; `item.json` `attributes` exact.
**Variation (scratch test, deleted, tree clean after):** a condition reorder (p3 → position 1) selects `[p3,p1,p2]`;
with no integration the form totals are 0/0/3 and a shop key is refused; a preset default `"Wegner "` pre-fills
canonical `"Wegner"` only for items of that type, AI may replace a default, and a human value then refuses AI.
**H1–H8:** no platform write, file, pixel or UI-automation path added. Core has no platform names (only `product.*`
built-ins). Commands are thin DTO wrappers. Everything runs with `integration = None`.
**Assumptions:** none contradicts the book in a way users can see. Assumption 2 (`product.title` is separate from
`items.title`) and assumption 3 (url trimmed, which follows row C1(c) over MC-9's table) are backlog notes. The rest go
to the narrative.
**Notes (backlog, 7):** title binding before P11/P14; MC-9 vs C1(c) url trim (fold upstream); human write to a
`reference` field accepted; C1(p) `writable` comes from the fixture; C5 id order = position order (probe confirms
position); damage report exported as row ids; tags not NFC-normalized.
**Lessons for plans:** selection/order rows need a reorder step in the input (second phase running); a plan row that
restates a contract table must quote it, not paraphrase it (C1(c)); the plan named `product.title` without saying
whether it is `items.title`.
