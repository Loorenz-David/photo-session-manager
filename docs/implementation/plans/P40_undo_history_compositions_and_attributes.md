---
phase: P40_undo_history_compositions_and_attributes
title: Undo history — composition saves (set, reset, Apply to selected as one step) and attribute edits (a field saved, Accept all, Clear, the title) as steps, reversed as new revisions and as restored value rows with their provenance, in one transaction each; a composition that now needs review cannot be undone; consecutive saves of one field coalesce
difficulty: high
projection: done
depends_on: [P39_undo_history_core]
---
# P40 — Undo history: composition and attribute steps

## Goal
The second backend half of R41 (05A MC-30, M36): `CompositionService` records every saved composition (`set`: a finished gesture, a control
change, Esc revert, Paste framing, Split/Normal; `reset`) as one step and `apply` (Apply composition to selected) as **one step for every
photo it changed**; `AttributesService` records every value the details form saves (`set_value`, `accept_all`, `clear`, the title) as one
step carrying the prior value **and provenance** of every field it changed. `HistoryService` reverses them: a reversed composition is a
**saved composition** (MC-27: a new revision, `state ok`, MC-6 `change_seq + 1`, `PhotoChanged` — so the next export or publish re-renders
its outputs and replaces its links, MC-28/MC-29) written for every photo of the step in one transaction; a reversed attribute step puts the
prior rows back byte for byte (or removes a row that did not exist) and sets `last_activity_at` without touching `change_seq`. A
composition step whose photo is now `needs_review` cannot apply (nothing is cropped silently, M33). Consecutive saves of the same field
coalesce into one step (the form writes as you type, ch08).
**Not in this phase:** the frontend (P41); the Edit menu (P41); any change to what `set`, `reset`, `apply`, `set_value`, `accept_all` or
`clear` store (P28, P34, P10 stand).

## Read first
Book: `05A_mechanism_contracts.md` MC-30 (the table rows "composition saved", "Apply composition to selected", "attribute value saved";
§Rules "a reversed composition is a saved composition (new revision, outputs re-rendered, MC-28)"; §Not steps "Typing inside a text field is
the field's own undo"), MC-6 ("Attribute edits do not change `change_seq`"; "A saved composition change (revision + 1) is a photo change"),
MC-9 (canonical values; `valid`), MC-4 (`last_activity_at`: "any attribute edit") · `10D_composition_and_framing.md` MC-27 (§Values incl.
`origin`, `revision`, `state`; §Validation; §Revisions: `needs_review` — "Nothing is cropped silently"), MC-28 §Cache key, MC-29 §Links
and §Dirty, and the bullet "Undo and redo (⌘Z, ⌘⇧Z) work in the editor … Esc still reverts … and that revert is itself one step" ·
`08_attributes_and_ai.md` §Values and provenance ("Values save locally as typed") · `14_measurement_ledger.md` M13, M16, M33, M36 · plan
`P39_undo_history_core.md` (§The step model, §Reversal rules, Notes) · plans `P28_composition_foundation.md` (`save_composition`, the
geometry fixture), `P34_composition_polish.md` (`composition_apply`), `P10_attributes.md` (`set_value`, `ai_suggest`, `accept_all`, `clear`,
the title's one home) · master plan §6.4, §6.10 (P39–P41 block), §7.5.
Code at `3e4b760` (+ P39's additions):
- `crates/core/src/services/composition.rs:19–33` (`apply`: a loop of `set`, partial refusals), `:34–45` (`set`: frame, `validate`, `save`),
  `:46–66` (`reset`: `Origin::Default`), `:67–84` (`save`: `store.save_composition(photo_id, values, origin, now)` → `(Composition, changed)`;
  `PhotoChanged` + `SessionChanged` when changed).
- `crates/infra/src/sqlite/store.rs:975–1000` (`Store::save_composition`: its **own** transaction; the idempotence check compares values,
  `state == Ok` **and** `origin`; `revision + 1`, `state = 'ok'`, `review_reason NULL`; `change_seq + 1` only for a non-deleted photo), `:156–165`
  (`composition(conn, photo_id)` — the connection-level reader both paths can share), `:106–155` (`composition_row`).
- `crates/core/src/domain/composition.rs:204–236` (`CompositionValues`, `Composition`, `frame()`, `values()`), `crates/core/src/mechanisms/
  composition.rs` (`validate`).
- `crates/core/src/services/attributes.rs:229–255` (`write`: one read-modify-write per key; **`same_content` → no write**; shared by
  `set_value`, `ai_suggest` and `clear`, so recording is decided by the caller, D2), `:260–286` (`write_title`: `items.title`,
  `set_item_activity`), `:292–339` (`set_value`: the title branch, the human-over-AI rule), `:343–374` (`ai_suggest` — **records nothing**; no
  production caller), `:378–401` (`accept_all`: one transaction over every suggested non-low value), `:405–418` (`clear`: `ai_status = Cleared`,
  the row stays, `is_live` false), `:79–87` (`same_content`), `:36–55` (the built-in `product.description`, label `Description`, always active).
- `crates/core/src/services/history.rs` (P39, shipped): `StepKind::{Composition, Attribute}`, `CompositionChange`, `FieldChange`, `FieldState::{Row,
  Title}` already exist (`:63–101`); `apply()` holds the placeholder `InvalidData("history step belongs to P40")` (`:560–562`); `History::record`
  has no coalescing yet (`:136–149`); the operation lock and the drop rule live in `HistoryService::reverse` (`:191–246`); the Copy redo branch
  reads each source row with a bare `tx.photo(id)?` (backlog "P39 → P40": `NotFound` instead of `StepCannotApply`, task 8).
- `crates/core/src/domain/attribute.rs:205–230` (`Value` fields: `item_id, key, value, valid, provenance, ai_confidence, ai_status, ai_reason,
  ai_sources, updated_at`; `is_live`).
- `crates/core/src/ports/store.rs:94–100` (`StoreTx::attribute_values`, `put_attribute_value`, `set_item_activity`), the `Store::composition`,
  `save_composition` and `session_for_item` entries below them.
- `crates/infra/src/sqlite/store.rs:755–768` (`put_attribute_value` upsert, `set_item_activity`).
- `apps/desktop/src-tauri/src/commands/compositions.rs:189–216` (`composition_apply` loops `app.compositions.apply(vec![one])` per item — the
  step must still be **one**, see task 3), `commands/attributes.rs:203–219` (`integration()`, `with_service` → `AttributesService::new(store,
  clock, integration)` — gains `.with_history(app.history.as_ref())`, D4), `:259–305` (`attribute_values_set`, `attribute_ai_accept_all`,
  `attribute_ai_clear`); `apps/desktop/src-tauri/src/lib.rs` (`p28_tests`, `p39_tests`: the mock-webview pattern — the two command rows of this
  phase live there as `mod p40_tests`, D5; `commands` is private to the desktop crate and testkit cannot depend on it).
- `crates/core/src/services/health.rs:36` and `crates/integrations/src/lib.rs:586` (the two schema-refresh callers of `AttributesService::new` —
  **untouched**: without `with_history` nothing records), `lib.rs:440` (`CompositionService {` — gains `history`; `history` is built at `:436`, before
  it).
- `crates/testkit/tests/p34_composition_apply.rs:11–60` (the rendered-photo fixture: `with_fake_document`, `add_photo` (**unreviewed**),
  `upsert_raw_state`, `fake.add_image`, `previews.request_initial`, `complete_render_with(SyntheticJpeg::rgb(2560, 1707, …))`, the valid square
  `V1 = rect {0.25, (1 − 1280/1707)/2, 0.5, 1280/1707}` and `invalid` = `V1` with `rect {0.2, 0, 0.5, 1}`, refused `off_shape` `:55–63, :75`),
  `crates/testkit/tests/p29_master_and_output_render.rs:1007–1020` (`flag(a, p)`: `set_variant_geometry(rotation 3.0)` + `on_variant_adjusted` +
  `tick` + `complete_render_with` → `needs_review`), `crates/testkit/tests/p10_attributes.rs:1–60` (`app_with`, `attrs`, `item`, `INTEGRATION`,
  `shop_definitions()` keys such as `custom.color_furniture` (Text, label `Color`), `custom.condition_report` (Textarea, label `Condition report`),
  `custom.condition_cracks` (severity choice), `:355–364` (an `ai_suggest` call)); `apps/desktop/src-tauri/src/lib.rs:51–70` (`p39_tests::setup`:
  the mock app's photos become Main through `a.review().classify`, which records `Approve 0n`).
- The 16 three-argument `AttributesService::new` call sites (`p11_title_binding.rs:15`, `p13_publish_runs.rs` ×8, `p14_publish_update.rs` ×3,
  `p21_backend_robustness.rs` ×2, `p30_outputs_in_export_and_publish.rs:205`, `p10_attributes.rs:41`, `health.rs:36`, `lib.rs:586`,
  `commands/attributes.rs:218, 231`) — **all untouched** by D4's `with_history` route.

## Files expected to change
Core (changed): `crates/core/src/services/{history.rs, composition.rs, attributes.rs}`, `crates/core/src/ports/store.rs`.
Infra (changed): `crates/infra/src/sqlite/store.rs`. Integrations (changed): `crates/integrations/src/lib.rs` (`CompositionService { history }` only).
Desktop (changed): `apps/desktop/src-tauri/src/commands/{attributes.rs, compositions.rs}`, `apps/desktop/src-tauri/src/lib.rs` (`mod p40_tests`:
rows C2(c), C7(b)).
Testkit (changed): `crates/testkit/src/test_app.rs` (an `attributes(integration)` helper that attaches the history). Tests (new):
`crates/testkit/tests/p40_undo_history_values.rs`. Tests (changed): none (`AttributesService::new` keeps its three arguments, D4).
Estimated: 9 changed source paths (one carrying the two command rows), 1 new test — 10 paths.

## Core mechanisms (unit tests allowed)
`history.rs` coalescing rule (`mc30_attribute_steps_coalesce`, D3): recording a **`set_value`** `Attribute` step for `(item, key)` when the
newest undo step is itself a **`set_value`** `Attribute` step for the same `(item, key)` **and the redo list is empty** replaces that step's
`after` and keeps its `before` (one step; the label unchanged). Clear and Accept all steps never merge and nothing merges onto them (MC-30's
table lists "a field committed, Accept all, Clear" as three owner actions); any other newest step → a new step. The unit test carries the
negative cases: `Edit Color` then `Clear Color` → two steps; `Accept all suggestions` (one field) then `set_value` of that key → two steps.
Discriminator (free choice, no registry change): a single-field `Attribute` step whose label is `Edit <label>` is a `set_value` step.

## Ordered tasks
0. **Registry first (master plan §6.10).** `StepKind::Composition { session_id, photos: Vec<CompositionChange> }`, `CompositionChange { photo_id,
   before: (CompositionValues, Origin), after: (CompositionValues, Origin) }`; `StepKind::Attribute { session_id, item_id, fields: Vec<FieldChange>
   }`, `FieldChange { key, before: FieldState, after: FieldState }`, `FieldState::{ Row(Option<Value>), Title(String) }` (the whole `Value` row,
   `updated_at` included; `Row(None)` = no row). Labels: `Compose 03` (the photo's position; `Compose photo` when unpositioned) for every
   single-photo composition step incl. reset, Esc revert, Paste, Split/Normal; `Apply framing to <n> photos` (`… to 1 photo`); `Edit <label>`
   (the definition's `label`, e.g. `Edit Color`, `Edit Title`); `Accept all suggestions`; `Clear <label>` (`clear(product.title)` → a `Title` step
   `Clear Title`, D13). The step types already exist from P39 (`history.rs:63–101`): this phase adds the `StoreTx` methods, the labels and the
   wiring. `CompositionService` gains `history: Arc<History>`; `AttributesService` keeps `new(store, clock, integration)` and gains
   `with_history(self, &'a History) -> Self` (the `SessionsService::with_editing` pattern, D4) — without it nothing records. **New**
   `StoreTx::composition(photo_id) -> Result<Composition>`, `StoreTx::save_composition(photo_id, &CompositionValues, Origin, now_ms) ->
   Result<(Composition, bool)>` (the body of `Store::save_composition` moved to a shared `fn save_composition_in(conn: &Connection, …)` used by
   both), `StoreTx::delete_attribute_value(item_id, key)`. Cannot-apply reasons: `that photo changed in Capture One — re-frame it first`
   (`needs_review`), `that photo has no frame` (`FrameUnknown`), `the framing no longer fits this photo` (`validate` fails), `that photo is gone`
   (also for a copy redo whose source row is gone, task 8), `that product is gone`.
1. **Composition recording** (`composition.rs`): `set` and `reset` read the current composition (they already do, for the frame) and keep
   `before = (values(), origin)`; after `save` returns `changed == true`, record `Composition { photos: [one] }` in the photo's session with the
   label; `changed == false` records nothing. `apply(items)` records **one** step with every photo whose save changed (refused or unchanged photos
   are not in it) in **the first changed photo's session** (D14; the UI applies within one session's selection); zero changed photos → nothing. A
   photo whose `values()` is `None` (no rect yet) cannot be `set` (FrameUnknown) and records nothing.
2. **Composition reversal** (`history.rs`): undo = for every `CompositionChange` inside one `store.transaction`, **under the `History` operation
   lock of P39 §Serialisation** (one undo or redo at a time per app): `tx.photo(photo_id)` (`NotFound` → `that photo is gone`; `session_id !=
   step.session_id` → `that photo already changed`, a later non-step move); `tx.composition(photo_id)`; `state == NeedsReview` → cannot apply
   (reason above); `frame()` `None` → cannot apply; `validate(&before.values, frame)` fails → cannot apply; `tx.save_composition(photo_id,
   &before.values, before.origin, now)` — when it answers `changed == false` for every photo (the stored values already equal the target) nothing
   was written and nothing is emitted (P39 D12: a reversal that changes nothing is not a photo change); after the commit **and after the operation
   lock is released**, emit through `History`'s sink (the app's `EventHub`, same module, D10) `PhotoChanged` for every photo whose save changed and
   `SessionChanged` once (what `CompositionService::save` emits). `HistoryOutcome.sessions = [step.session_id]` (also when nothing was written),
   `active_photo_id = None` (D8). Redo = the same with `after`. The saved row has `revision = previous + 1` (never `− 1`), `state ok`; `change_seq +
   1` **per photo saved** comes from `save_composition_in` as today (a two-photo Apply undo adds 2, like `apply` itself; D11). Only `StepCannotApply`
   drops the step; a SQLite or I/O error leaves both lists unchanged (P39 D11).
3. **`composition_apply` as one step** (`commands/compositions.rs:189–216`): the command converts every item first (collecting conversion
   refusals) and calls `app.compositions.apply(valid_items)` **once**, so the service records one step; the DTO is unchanged; `refused` lists the
   conversion refusals first, then the service's, each in input order (D12 of the P40 projection).
4. **Attribute recording** (`attributes.rs`): recording is decided by the **caller** of `write`, never by `write` itself (`write` gains a
   "record as `<label>` / do not record" input, D2): `set_value` records `Attribute { fields: [{ key, before: Row(stored), after: Row(Some(row)) }] }`
   labelled `Edit <label>` in `session_for_item(item_id)`'s history (no session → not recorded); **`ai_suggest` records nothing** (not an owner
   action; no production caller); `write_title` records `{ key: product.title, before: Title(old), after: Title(new) }` labelled `Edit Title` when the
   title changed, and `clear(product.title)` → `Clear Title` (D13); `accept_all` records one step `Accept all suggestions` with every accepted row
   (`before` = suggested, `after` = accepted) when `count > 0`; `clear` records `{ before: Row(live), after: Row(cleared row) }` labelled `Clear
   <label>` (nothing when there was nothing live). Recording after the commit; `same_content` no-ops record nothing. The coalescing rule of §Core
   mechanisms applies to **`set_value` steps only** (D3): Clear and Accept all are always their own steps and are never merged onto.
5. **Attribute reversal** (`history.rs`): undo = one `store.transaction` under the same operation lock: `tx.item(item_id)` (`NotFound` → cannot
   apply, `that product is gone`); for every field: `Row(Some(v))` → `put_attribute_value(&v)` (byte-identical row, `updated_at` included);
   `Row(None)` → `delete_attribute_value(item_id, key)`; `Title(t)` → `update_item_title(item_id, &t, now)`; each write is skipped when the stored
   row already equals the target by **whole-row `==`** (`Value: PartialEq`, `updated_at` included; a `Title` by string, D15), and when every field
   was skipped `set_item_activity` is skipped too (P39 D12); otherwise `set_item_activity(item_id, now)` once. `change_seq` is never touched.
   Attribute reversals emit **no** `PhotoChanged` / `SessionChanged` (attribute writes emit nothing today; `HistoryChanged` covers the UI, D10).
   Redo = the same with `after`. `active_photo_id` is `None`; `sessions` = the item's session. Only `StepCannotApply` drops the step (P39 D11).
6. **Wiring**: `CompositionService { history }` in `lib.rs:440`; `.with_history(app.history.as_ref())` on the two `AttributesService::new` calls in
   `commands/attributes.rs:218, 231`; `health.rs:36`, `lib.rs:586` and every test's `AttributesService::new(…)` untouched (D4);
   `TestApp::attributes(integration: Option<&str>) -> AttributesService<'_>` = `new(store, clock, integration).with_history(app.history)`.
7. Tests: `p40_undo_history_values.rs` (C1(a)–(d), C2(a)–(b), C3, C4, C5, C6, C7(a), C8) and `apps/desktop/src-tauri/src/lib.rs` `mod p40_tests`
   (C2(c), C7(b): the mock-webview rows, D5); `cargo test --workspace`, clippy, fmt; `npm run typecheck` (no binding change).
8. **Copy redo with its source row gone** (`history.rs` Copy redo branch, backlog "P39 → P40", D9): the bare `tx.photo(source_id)?` becomes a
   `NotFound` → `StepCannotApply { direction: "redo", label, reason: "that photo is gone" }`, so the step is dropped instead of staying in the redo
   list forever.

## Criteria (8 criteria, 20 rows; 0 `[rule]` rows)
Fixtures: **F-comp** = p34's rendered-photo fixture (`with_fake_document`; photos are added **unreviewed**; frame `2560 × 1707`; default square
after the render = `rect {0.166601562, 0, 0.666796875, 1}`, `origin default`, `revision 0` — read the actual stored row as `c0`); where a row
needs `p` at Main 1 it says `R.classify(p, Main)`, **which records `Approve 01`** (the only route to a position); `V1` = p34's valid square `{0.25,
(1 − 1280/1707)/2, 0.5, 1280/1707}`; `V2` = `V1` with `x = 0.3`; `INVALID` = p34's `invalid` (`V1` with `rect {0.2, 0, 0.5, 1}`, refused
`off_shape`, D7). **F-attr** = p10's `app_with(true)` with `item(a, "K")` → `(item, s)`; key `K1 = custom.color_furniture` (Text, label `Color`),
`K2 = custom.condition_report` (Textarea, label `Condition report`). Test-local helpers (D16): `comp(p)` = `a.composition(p)`; `row(item, key)` =
the stored `Value` read through `store.transaction(|tx| tx.attribute_values(item))` (cleared rows included); `vals(item)` =
`attributes.values(item)` (live only); `seq(s)` = `store.session(s).session.change_seq`; `session(s)` = `store.session(s).session`; `state(s,
class)` as p02; `u` = a second photo added unreviewed in `s`. `H = a.history()`, `C = a.app.compositions`, `A = a.attributes(Some(INTEGRATION))`,
`R = a.review()`. Every `A.set_value` below passes `Provenance::Human` where the argument is omitted.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | composition set undo | F-comp, `R.classify(p, Main)` (Main 1; records `Approve 01`); `c0 = comp(p)`; `C.set(p, V1)` → `c1` (`revision 1`, `origin user`); `n = seq(s)`; `out = H.undo(s)` | `out.label == "Compose 01"`, `out.active_photo_id == None`, `out.sessions` names `s` only; `c2 = comp(p)`: `c2.values() == c0.values()`, `c2.origin == c0.origin` (`default`), `c2.revision == 2`, `c2.state == Ok`; `seq(s) == n + 1`; `a.events` holds a `PhotoChanged { photo_id: p, session_id: s }` and a `SessionChanged { s }` after the undo | MC-30 ("the prior composition values, saved as a new revision (MC-27)"; "a reversed composition is a saved composition"); MC-6; M36 |
| C1(b) | composition redo | from C1(a); `H.redo(s)` | `comp(p).values() == Some(V1)`, `origin user`, `revision 3`; `H.state(s) == { Some("Compose 01"), None }` | M36 ("redoing them leaves them exactly as after") |
| C1(c) | reset is a step | F-comp; `C.set(p, V1)`; `C.reset(p)` → `origin default`, `revision 2`; `H.undo(s)` | `comp(p).values() == Some(V1)`, `origin user`, `revision 3`; `H.state(s) == { Some("Compose photo"), Some("Compose photo") }` (the set below, the reset in redo) | MC-30 (Reset is a composition save); MC-27 §Values (`origin`) |
| C1(d) | no change, no step | F-comp; `C.set(p, V1)`; `C.set(p, V1)` again | `comp(p).revision == 1`; `H.state(s).undo == Some("Compose photo")` and `H.undo(s)` once leaves `comp(p).values() == c0.values()` — i.e. exactly one step was recorded | MC-30 (a step is one owner action that changed something); MC-27 ("identical values are a no-op", master plan §6.10) |
| C2(a) | Apply as one step | F-comp with three rendered photos `p, q, r`, all **unreviewed** (no classify step; the label does not depend on positions); `before_q = comp(q)`, `before_r = comp(r)`; `C.apply(vec![(q, V1), (r, V2), (p, INVALID)])` (p refused `off_shape`); `out = H.undo(s)` | `H.state(s).undo` before the undo `== Some("Apply framing to 2 photos")`; after: `comp(q).values() == before_q.values()`, `comp(r).values() == before_r.values()`, both `revision 2`, `comp(p).revision` unchanged (never touched); one `H.state(s)` entry moved: `{ undo: None, redo: Some("Apply framing to 2 photos") }`; `out.label == "Apply framing to 2 photos"` | MC-30 ("Apply composition to selected → every touched photo's prior composition, as one step"); M36 |
| C2(b) | Apply redo | from C2(a); `H.redo(s)` | `comp(q).values() == Some(V1)`, `comp(r).values() == Some(V2)`, both `revision 3` | M36 |
| C2(c) | the command sends one step | mock webview (`p28_tests` pattern): `composition_apply {items: [{photoId: q, values: V1}, {photoId: r, values: V2}]}`; `history_get` | the DTO `applied == [q, r]`, `refused == []`; `history_get.undo == "Apply framing to 2 photos"` (one step, not two) | MC-30; master plan §6.7 |
| C3(a) | needs_review blocks the undo | F-comp; `R.classify(p, Main)` (records `Approve 01`); `C.set(p, V1)`; make `p` `needs_review` with p29's `flag(&a, &p)` (`set_variant_geometry(rotation 3.0)`, `on_variant_adjusted`, `tick`, `complete_render_with`); `rev = comp(p).revision`; `r = H.undo(s)` | `r` is `Err(StepCannotApply { direction: "undo", label: "Compose 01", reason: "that photo changed in Capture One — re-frame it first" })`; `comp(p).state == NeedsReview`, `comp(p).revision == rev`, `comp(p).values() == Some(V1)` (nothing written); the compose step is dropped and the step below is next: `H.state(s).undo == Some("Approve 01")` | MC-30 ("not partly applied: nothing changes, it is dropped"); MC-27 §Revisions ("Nothing is cropped silently"); M33, M36 |
| C4(a) | set over an AI suggestion, undo restores the row | F-attr; `A.ai_suggest(item, K1, json!("Teak"), AiConfidence::High, Some("label"), vec!["https://x".into()])`; `ai = row(item, K1)`; `clock.advance(1_000)`; `A.set_value(item, K1, json!("Oak"), Human)`; `last = session(s).last_activity_at`; `n = seq(s)`; `clock.advance(1_000)`; `out = H.undo(s)` | `out.label == "Edit Color"`, `out.active_photo_id == None`; `row(item, K1) == ai` **field for field** (`value "\"Teak\""`, `provenance Ai`, `ai_confidence High`, `ai_status Suggested`, `ai_reason`, `ai_sources`, `updated_at` = the suggestion's); `session(s).last_activity_at > last`; `seq(s) == n` (no photo change) | MC-30 ("the prior value and provenance of every field the action changed"); MC-4; MC-6 ("Attribute edits do not change `change_seq`"); M36 |
| C4(b) | first value, undo removes the row | F-attr; `A.set_value(item, K2, json!("Scuffed"), Human)`; `H.undo(s)` | `row(item, K2)` is `None` (no row at all, not a cleared one); `vals(item)` has no `K2`; `H.state(s) == { None, Some("Edit Condition report") }` | MC-30; M36 ("attribute values and provenance exactly as they were") |
| C4(c) | redo | from C4(a); `H.redo(s)` | `row(item, K1)` has `value "\"Oak\""`, `provenance Human`, `ai_status None`, `ai_confidence None` | M36 |
| C4(d) | the title | F-attr (`create_local("K", "Oak", …)` → title `Oak`); `A.set_value(item, PRODUCT_TITLE, json!("Oak table"), Human)`; `H.undo(s)`; `H.redo(s)` | after the undo `store.item(item).title == "Oak"` and `H.state(s).redo == Some("Edit Title")`; after the redo `title == "Oak table"`; `seq(s)` unchanged throughout | MC-30; ch05 (the title's one home); MC-6 |
| C4(e) | coalescing while typing | F-attr; `A.set_value(item, K1, json!("O"), Human)`; `A.set_value(item, K1, json!("Oa"), Human)`; `A.set_value(item, K1, json!("Oak"), Human)`; `H.undo(s)` | one step: `H.state(s).undo` before `== Some("Edit Color")` and after the single undo `row(item, K1) == None` and `H.state(s) == { None, Some("Edit Color") }` (the three saves were one step with the first `before`) | MC-30 §Not steps ("Typing inside a text field is the field's own undo"; "a field committed" is one step); ch08 ("Values save locally as typed") |
| C4(f) | no coalescing across fields | F-attr; `A.set_value(item, K1, json!("Oak"))`; `A.set_value(item, K2, json!("Scuffed"))`; `A.set_value(item, K1, json!("Teak"))`; `H.undo(s)` ×3 reading `vals` after each | after 1: `K1 == "Oak"`, `K2 == "Scuffed"`; after 2: `K1 == "Oak"`, no `K2`; after 3: no `K1`; `H.state(s).undo == None` | MC-30 (three owner actions, three steps); M36 |
| C5(a) | accept all | F-attr; `ai_suggest` `K1` High, `K2` Medium, `custom.condition_cracks` Low (`json!("Minor")`); `A.accept_all(item)` → `2`; `H.undo(s)` | `H.state(s).undo` before `== Some("Accept all suggestions")`; after: `row(item, K1).ai_status == Some(Suggested)`, `row(item, K2).ai_status == Some(Suggested)`, the Low row untouched (`Suggested` throughout); `H.state(s) == { None, Some("Accept all suggestions") }` | MC-30 ("Accept all"); M36 |
| C5(b) | clear | F-attr; `A.set_value(item, K1, json!("Oak"))`; `A.clear(item, K1)` → the row is `Cleared`, not live; `H.state(s)` before the undo `== { Some("Clear Color"), None }` (two steps, no merge: D3); `H.undo(s)` | `row(item, K1).ai_status == None` and `is_live`, `value "\"Oak\""`, `provenance Human`; `vals(item)` has `K1`; `H.state(s) == { Some("Edit Color"), Some("Clear Color") }`; a second `A.clear(item, K2)` on an empty field records nothing (`H.state(s).undo` unchanged) | MC-30 ("Clear"); M16 (a write that changes nothing is no write and no step); M36 |
| C6(a) | one list across kinds | F-attr with one rendered photo `p` (F-comp fixture inside the same app) made Main 1 by `R.classify(p, Main)` (records `Approve 01`); `R.classify(u, Main)` (`u` unreviewed → Main 2, `Approve 02`); `A.set_value(item, K1, json!("Oak"))`; `C.set(p, V1)`; `H.undo(s)` ×3 reading `H.state(s).undo` before each and after the last | `Some("Compose 01")` → `Some("Edit Color")` → `Some("Approve 02")` → `Some("Approve 01")`; after the three undos `comp(p).values() == c0.values()`, `vals(item)` has no `K1`, `state(s, Main) == [(p, 1)]` | MC-30 ("One history per session" — every kind in the same list, LIFO); M36 |
| C7(a) | events | F-attr; `A.set_value(item, K1, json!("Oak"))`; `H.undo(s)` | `a.events` holds `HistoryChanged { session_id: s, undo: Some("Edit Color"), redo: None }` then `HistoryChanged { s, None, Some("Edit Color") }`, and **no** `PhotoChanged` / `SessionChanged` from the attribute reversal (D10) | MC-30 (header icons and menu follow the history) |
| C7(b) | command DTO for an attribute step (`lib.rs` `mod p40_tests`, mock webview; no integration configured, so the built-in field is used, D6) | `attribute_values_set {itemId, key: "product.description", value: "\"Scuffed top\""}`; `history_undo {sessionId: s}` | `{"label":"Edit Description","direction":"undone","active_photo_id":null,"sessions":[{"id": s, …}]}`; `attribute_values_get {itemId}` afterwards has no `product.description` | MC-30; master plan §6.7 |
| C8(a) | copy redo with its source row gone (backlog "P39 → P40") | three sessions `r, s, t` with p02's helpers: `main(r, [ra])`; `R.copy_photos(&[ra], s)` (step in `r`; copy `cs`); `R.copy_photos(&[cs], t)` (step in `s`); `H.undo(s)` (removes `t`'s copy); `H.undo(r)` (removes `cs`); `out = H.redo(s)` | `out` is `Err(StepCannotApply { direction: "redo", label: "Copy to t", reason: "that photo is gone" })`; `state(t, Main) == []`; the step is dropped: `H.state(s) == { None, None }`; a second `H.redo(s)` → `Err(NothingToRedo)` | MC-30 ("A step that can no longer apply … is dropped from the list, and the app says why"); M36 |

## Delegated choices
Where `save_composition_in` lives (a free function in `store.rs`); whether `apply` validates all items before saving any (today it saves what
it can — keep that) or records partial success as the step (it does, by rule); how the composition label reads the photo's position
(`tx.photo`); the `reason` texts beyond those fixed in task 0; the shape of `write`'s "record as" input (an `Option<&str>` label or a small
enum). Settled by the projection and written into tasks/Notes, not free any more: `FieldState::Title` is the variant P39 shipped; coalescing is
`(item, key)` for `set_value` steps only (D3); the composition reversal's `sessions`/`active_photo_id` (D8); events through `History`'s sink,
none for attribute reversals (D10); `change_seq` per photo (D11); `refused` order (D12); `Clear Title` (D13); the first changed photo's session
for a cross-session `apply` (D14); whole-row equality (D15); the test-local helpers (D16).

## Manual try-out
(The frontend is wired in P41; these steps check the backend through the running app's old ⌘Z.)
1. Open a photo in the editor (R), drag it, press Return; press ⌘Z: the framing is back where it was and the tile redraws. ⌘Z again undoes
   whatever came before (an approval, say). Press ⌘⇧Z — not wired yet (P41); instead re-open the editor and see the tile unchanged.
2. Select three photos, copy a framing (⌥⌘C on one), Apply composition to selected from the ⋯ menu; press ⌘Z once: all three are back.
3. Open Details (D), type a colour, close the form, press ⌘Z: the field is empty again (one undo for the whole typed word).
4. In Capture One, straighten a photo whose framing you changed; wait for "Re-frame"; press ⌘Z: the app says "Can't undo Compose 01: that
   photo changed in Capture One — re-frame it first" and the next ⌘Z undoes the step before.

## Notes
- **Planner readings (reversible; for the narrative's "Choices made for you")**:
  (1) Typing into one field with pauses makes one undo step: successive saves of the same field merge until something else is done (another
  field, a photo action, an undo). Editing a field, then another, then the first again gives three steps.
  (2) Undoing a composition on a photo that Capture One has since changed (the amber "Re-frame" state) is refused with a notice, so a stale
  framing is never confirmed by accident; the step is dropped and the next ⌘Z continues.
  (3) Undo of a composition writes a new revision (the row's revision only grows), so outputs and shop links refresh at the next export or
  publish exactly as after any framing change.
  (4) Undoing an attribute edit restores the field byte for byte, including the AI suggestion's confidence, status and sources, and removes a
  row that did not exist before; the session's activity time is refreshed, its dirty state is not (attribute edits never dirty a session).
  (5) Apply composition to selected is one step even when some photos were refused; only the photos that changed are in it.
  (6) A Split/Normal from the grid on several photos is one step per photo until P41 routes it through Apply (P41 Notes).
  (7) Steps of every kind share one list per session, newest first: ⌘Z after a framing change, a field edit and an approval undoes them in
  that order.
  (8) Only your own saves of a field merge into one step; a Clear or an Accept all is always its own step, and an AI suggestion is never a step
  (it is not something you did). Clearing the title is its own step, "Clear Title".
  (9) Undoing a framing refreshes the stage and tiles (the app's photo-changed signal); undoing a field edit refreshes only the undo icons — the
  details form re-reads its values when it opens or a field loses focus, as today.
  (10) "Apply to selected" undone across two photos counts as two photo changes for the session's dirty state, as applying it did.
  (11) When an apply mixes photos from two sessions (the screens never do), the step lives in the first changed photo's session.
  (12) Redoing a copy whose original photo has since been removed is refused with "that photo is gone" and the step is dropped (backlog item
  routed here from P39).
- **Delegations written down (projection D10–D16)**: composition reversals emit `PhotoChanged` per changed photo and one `SessionChanged` through
  `History`'s `EventSink` after the lock is released; attribute reversals emit nothing but `HistoryChanged` (D10). `change_seq` grows by one per
  photo saved in a composition reversal (D11). `composition_apply.refused` = conversion refusals first, then the service's, each in input order
  (D12). `clear(product.title)` records a `Title` step `Clear Title` when the title changed (D13). A cross-session `apply` records in the first
  changed photo's session (D14). "Already equals the target" = whole-row `==` for a `Value`, string `==` for a title (D15). Test helpers are
  test-local as the fixture paragraph defines them; `Provenance::Human` wherever a row omits it (D16).
- **Mechanism-check findings this plan builds on** (handoff §Mechanism check): `Store::save_composition` is **not** inside `StoreTx` and
  `apply` is a per-photo loop (not atomic today) — the step reversal needs a `StoreTx::save_composition`, hence task 0's shared body;
  `save_composition`'s idempotence compares `origin` too, so `before` carries `(values, origin)`; output re-rendering is **not** triggered by a
  save — the cache key `(adjusted_gen, revision, output_format)` (MC-28) and `photo_links.composition_rev` (MC-29) make a new revision stale, and
  the next export/publish run re-renders (`export.rs:466`) and replaces (P30 F1): a reversed composition therefore re-renders by being a new
  revision, nothing more is needed; `AttributesService::write` already reads the prior row inside its transaction; the title lives in
  `items.title` (no value row), so its step carries a string; `clear` keeps a non-live row, so "no row before" and "cleared row before" are
  different `before`s (`Row(None)` vs `Row(Some(cleared))`) — both restorable; `set_item_activity` is the only activity writer for attributes.
- **Hazards**: the `needs_review` guard must be read **inside** the reversal transaction (a render can flag the photo between the check and the
  write); `put_attribute_value` is an upsert keyed `(item_id, key)` — restoring a row writes its old `updated_at` back on purpose (byte
  identity, M36); `composition_apply` must call `apply` once (task 3) or the step splits per photo; `apply` emits one `PhotoChanged` per photo
  as today. `accept_all` must not coalesce. The `FrameUnknown` and `validate` guards keep M33's "the core refuses any rect outside the image".
- **Tests that change their expectation**: none (`AttributesService::new` keeps its signature, D4). No assertion changes anywhere.
- **Projection** is on: the transactional composition path and the byte-identical attribute restore are the two places a plausible
  implementation can silently diverge (a `− 1` revision; a restore through `set_value` that re-canonicalises and loses `ai_*`).
- Registry additions made by the planner in master plan §6.4, §6.10. No intention-book change.
- Estimated size: ~10 paths (9 changed, 1 new test), ~950 lines (≈ 400 core/infra/commands, ≈ 550 tests).

## Review log

- **2026-11-05, review r1 (opus): APPROVED_WITH_NOTES.** Gate: tracker `IMPLEMENTED`, checkpoint `69d3842` in `git log`. Perimeter: `git show --stat` = the 10
  declared paths + tracker + plan (13), nothing extra or missing. Rust suite, clippy `-D warnings`, fmt, `npm run typecheck` green (one run, clean tree
  `57495ec`). Verified correct, reading the code and then its test: `save_composition_in` is shared by `Store::save_composition` and `StoreTx::save_composition`
  (revision + 1, `state ok`, `change_seq + 1` per non-deleted photo); a composition reversal reads `needs_review`, frame and `validate` **inside** the
  transaction, under the operation lock, and emits `PhotoChanged` per changed photo + one `SessionChanged` after the lock is released (C1(a) red on
  `− 1` revision or a missing event; C3(a) pins reason text, unchanged revision and values, and the drop); `apply` records one step of only the changed
  photos (C2(a): p untouched at revision 0); `composition_apply` calls `apply` once (C2(c) would read `… to 1 photo` if it looped); attribute reversal
  restores whole rows with `put_attribute_value` / `delete_attribute_value` and skips a write when whole-row `==` (C4(a) compares the full `Value`,
  `updated_at` included); `ai_suggest` records nothing (`write(…, None, …)`); coalescing only on `Edit …` single-field steps with the same item and key and
  redo empty (C4(e) would leave `"Oa"` without it; C4(f) fails if a later same-key save merged past another field; C5(b) pins Clear as its own step);
  attribute reversals emit only `HistoryChanged` (C7(a)); copy-redo with source gone → `that photo is gone`, dropped (C8(a)). Assumptions: nested
  `photo_id` = `CompositionApplyItemDto` and `bindings.ts` (plan fixture slip, not a deviation in behaviour); partial-success Apply and no recording on
  schema refresh are as tasks 1–4 direct. None contradicts the book. H1–H8: no external write, no RAW touched, no pixel work, transactional reversals
  (H4), no platform concept in core, commands stay thin (H8). Notes (backlog, 6): **probe-confirmed** undo of a re-frame on a `needs_review` photo
  restores the old rect with `state ok` (MC-30's letter vs M33's "until confirmed"; owner choice before P41); cross-session Apply undo always refused by the
  session check; `expect("saved photo")` in `apply`; coalesced no-op step; three missing secondary asserts (c4f, c5b, c2a); C2(c) `photoId` plan slip.
  Probe file `crates/testkit/tests/zz_p40_review_probe.rs` created, run, deleted; no production file touched.

- **2026-11-05, implement r1 (codex).** Built composition set/reset/Apply recording, transactional composition undo/redo with new revisions and post-commit photo/session events; attribute set/title/clear/accept-all recording, exact stored-row reversal and same-field save coalescing; command wiring, one-call Apply, and the missing-source copy-redo refusal. Added 20 boundary tests, one coalescing mechanism unit test. **Assumptions:** the existing `composition_apply` nested item DTO uses `photo_id` (as its Rust serde and generated TypeScript binding require), though C2(c)'s fixture spells `photoId`; retained the DTO and used `photo_id` in the command test. `CompositionService::apply` keeps the existing partial-success behavior and records only successful changes, as tasks 1–3 direct. **Deviations:** C2(c)'s request fixture spelling only; no product behavior or file-perimeter deviation. The test app attaches history through its new `attributes` helper; existing three-argument attribute constructors remain untouched. Live camera and Capture One were not needed for the automated tests.

- **2026-11-05, orchestrator amendments after projection r0** (`handoffs/reviewer/handoff_P40_undo_history_compositions_and_attributes_projection_r0
  .md`, folded by the planner; `projection: yes` → `done`). D1 → F-comp states that a position needs `R.classify`, which records `Approve 01`;
  C1(a), C3(a), C6(a) carry it (C3(a) and C6(a) end on `Some("Approve 01")`), C2(a) keeps its photos unreviewed. D2 → task 4: `ai_suggest` records
  nothing; recording is the caller's decision (`write` gets a "record as" input). D3 (orchestrator's call) → §Core mechanisms and task 4: only a
  `set_value` save merges, onto a newest `set_value` step of the same `(item, key)` with redo empty; Clear and Accept all are always their own
  steps; negative cases in the unit test; C5(b) pins the two steps. D4 (orchestrator's call: the smaller route) → `AttributesService::with_history
  (&History)`, `new` unchanged; the 16 call sites, health.rs, lib.rs:586 and p10 untouched; Files/task 0/task 6 updated. D5 → `lib.rs` `mod
  p40_tests` hosts C2(c) and C7(b); Files and task 7. D6 (orchestrator's call) → C7(b) uses `product.description` (`Edit Description`). D7 →
  `INVALID` defined. D8 → task 2: `sessions = [step.session_id]`, `active_photo_id = None`. D9 (orchestrator's call: fold in) → task 8 + row C8(a);
  the backlog row is marked routed. D10–D16 → written into tasks 2–5 and Notes ("Delegations written down"). Reality checks: line citations
  refreshed (`attributes.rs`, `lib.rs:440/586`, `ports/store.rs`), `flag()` cited for C3(a), M16 now cited by C5(b). Criteria 7 → 8, rows 19 → 20
  (C8(a) added). Nothing declined. Carried to P41: nothing to change — the labels (`Edit Description`, `Clear Title`) and the refusal text come from
  the backend and P41's rows use literal label strings from mocked services; noted in P41's Review log.

- **2026-11-05, planner carry-over from P39's projection r0** (`handoff_P39_undo_history_core_projection_r0.md` D10–D12, D7): tasks 2 and 5 now
  state that composition and attribute reversals run under the `History` operation lock (D10), that only `StepCannotApply` drops a step (D11),
  that a reversal whose target equals the stored state writes nothing and does not touch activity or `change_seq` (D12), and that a composition
  step checks the photo is still in the step's session (D7's analogue). No row change: C1(d) already pins "no change, no step"; the lock and the
  drop rule are P39's rows C2(e)/C4(c) over the shared `HistoryService`. P40's own projection remains on.

