---
phase: P40_undo_history_compositions_and_attributes
title: Undo history — composition saves (set, reset, Apply to selected as one step) and attribute edits (a field saved, Accept all, Clear, the title) as steps, reversed as new revisions and as restored value rows with their provenance, in one transaction each; a composition that now needs review cannot be undone; consecutive saves of one field coalesce
difficulty: high
projection: yes
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
- `crates/core/src/services/attributes.rs:205–235` (`write`: one read-modify-write per key; **`same_content` → no write**), `:237–268`
  (`write_title`: `items.title`, `set_item_activity`), `:270–314` (`set_value`: the title branch, the human-over-AI rule), `:343–370`
  (`ai_suggest`), `:372–395` (`accept_all`: one transaction over every suggested non-low value), `:397–412` (`clear`: `ai_status = Cleared`,
  the row stays, `is_live` false), `:66–75` (`same_content`).
- `crates/core/src/domain/attribute.rs:205–230` (`Value` fields: `item_id, key, value, valid, provenance, ai_confidence, ai_status, ai_reason,
  ai_sources, updated_at`; `is_live`).
- `crates/core/src/ports/store.rs:84–92` (`StoreTx::attribute_values`, `put_attribute_value`, `set_item_activity`), `:111–122`
  (`Store::composition`, `save_composition`), `:149` (`session_for_item`).
- `crates/infra/src/sqlite/store.rs:755–768` (`put_attribute_value` upsert, `set_item_activity`).
- `apps/desktop/src-tauri/src/commands/compositions.rs:189–216` (`composition_apply` loops `app.compositions.apply(vec![one])` per item — the
  step must still be **one**, see task 3), `commands/attributes.rs:203–219` (`integration()`, `with_service` → `AttributesService::new(store,
  clock, integration)`), `:259–305` (`attribute_values_set`, `attribute_ai_accept_all`, `attribute_ai_clear`).
- `crates/core/src/services/health.rs:36` and `crates/integrations/src/lib.rs:572` (the two other `AttributesService::new` callers: schema
  refresh — they pass `None` for the history).
- `crates/testkit/tests/p34_composition_apply.rs:11–60` (the rendered-photo fixture: `with_fake_document`, `add_photo`, `upsert_raw_state`,
  `fake.add_image`, `previews.request_initial`, `complete_render_with(SyntheticJpeg::rgb(2560, 1707, …))`, a valid square `rect {0.25, …, 0.5,
  1280/1707}`), `crates/testkit/tests/p29_master_and_output_render.rs` or `lib.rs:289–330` (`c8a`: `set_variant_geometry(rotation 3.0)` +
  `on_variant_adjusted` + `tick` + `complete_render_with` → `needs_review`), `crates/testkit/tests/p10_attributes.rs:1–60` (`app_with`, `attrs`,
  `item`, `INTEGRATION`, `shop_definitions()` keys such as `custom.color_furniture` (Text, label `Color`), `custom.condition_cracks` (severity
  choice), `:355–364` (an `ai_suggest` call)).

## Files expected to change
Core (changed): `crates/core/src/services/{history.rs, composition.rs, attributes.rs}`, `crates/core/src/ports/store.rs`.
Infra (changed): `crates/infra/src/sqlite/store.rs`. Integrations (changed): `crates/integrations/src/lib.rs` (`CompositionService { history }`,
the `AttributesService::new(…, None)` call). Core (changed): `crates/core/src/services/health.rs` (`None`).
Desktop (changed): `apps/desktop/src-tauri/src/commands/{attributes.rs, compositions.rs}`.
Testkit (changed): `crates/testkit/src/test_app.rs` (an `attributes()` helper with the history). Tests (new):
`crates/testkit/tests/p40_undo_history_values.rs`. Tests (changed): `crates/testkit/tests/p10_attributes.rs` (`attrs()` passes `None` or the
history — one line; assertions unchanged).
Estimated: 10 changed source paths, 1 new test, 1 adjusted test.

## Core mechanisms (unit tests allowed)
`history.rs` coalescing rule (`mc30_attribute_steps_coalesce`): recording an `Attribute` step for `(item, key)` when the newest undo step is an
`Attribute` step for the same `(item, key)` **and the redo list is empty** replaces that step's `after` and keeps its `before` (one step; the
label unchanged); any other newest step → a new step.

## Ordered tasks
0. **Registry first (master plan §6.10).** `StepKind::Composition { session_id, photos: Vec<CompositionChange> }`, `CompositionChange { photo_id,
   before: (CompositionValues, Origin), after: (CompositionValues, Origin) }`; `StepKind::Attribute { session_id, item_id, fields: Vec<FieldChange>
   }`, `FieldChange { key, before: FieldState, after: FieldState }`, `FieldState::{ Row(Option<Value>), Title(String) }` (the whole `Value` row,
   `updated_at` included; `Row(None)` = no row). Labels: `Compose 03` (the photo's position; `Compose photo` when unpositioned) for every
   single-photo composition step incl. reset, Esc revert, Paste, Split/Normal; `Apply framing to <n> photos` (`… to 1 photo`); `Edit <label>`
   (the definition's `label`, e.g. `Edit Color`, `Edit Title`); `Accept all suggestions`; `Clear <label>`. `CompositionService` gains `history:
   Arc<History>`; `AttributesService::new(store, clock, integration, history: Option<&History>)`. **New** `StoreTx::composition(photo_id) ->
   Result<Composition>`, `StoreTx::save_composition(photo_id, &CompositionValues, Origin, now_ms) -> Result<(Composition, bool)>` (the body of
   `Store::save_composition` moved to a shared `fn save_composition_in(conn: &Connection, …)` used by both), `StoreTx::delete_attribute_value
   (item_id, key)`. Cannot-apply reasons: `that photo changed in Capture One — re-frame it first` (`needs_review`), `that photo has no frame`
   (`FrameUnknown`), `the framing no longer fits this photo` (`validate` fails), `that photo is gone`.
1. **Composition recording** (`composition.rs`): `set` and `reset` read the current composition (they already do, for the frame) and keep
   `before = (values(), origin)`; after `save` returns `changed == true`, record `Composition { photos: [one] }` in the photo's session with the
   label; `changed == false` records nothing. `apply(items)` records **one** step with every photo whose save changed (refused or unchanged photos
   are not in it); zero changed photos → nothing. A photo whose `values()` is `None` (no rect yet) cannot be `set` (FrameUnknown) and records
   nothing.
2. **Composition reversal** (`history.rs`): undo = for every `CompositionChange` inside one `store.transaction`, **under the `History` operation
   lock of P39 §Serialisation** (one undo or redo at a time per app): `tx.photo(photo_id)` (`NotFound` → `that photo is gone`; `session_id !=
   step.session_id` → `that photo already changed`, a later non-step move); `tx.composition(photo_id)`; `state == NeedsReview` → cannot apply
   (reason above); `frame()` `None` → cannot apply; `validate(&before.values, frame)` fails → cannot apply; `tx.save_composition(photo_id,
   &before.values, before.origin, now)` — when it answers `changed == false` for every photo (the stored values already equal the target) nothing
   was written and nothing is emitted (P39 D12: a reversal that changes nothing is not a photo change); after the commit emit `PhotoChanged` for
   every photo whose save changed and `SessionChanged` once (what `CompositionService::save` emits). Redo = the same with `after`. The saved row has
   `revision = previous + 1` (never `− 1`), `state ok`; `change_seq + 1` comes from `save_composition_in` as today. Only `StepCannotApply` drops the
   step; a SQLite or I/O error leaves both lists unchanged (P39 D11).
3. **`composition_apply` as one step** (`commands/compositions.rs:189–216`): the command converts every item first (collecting conversion
   refusals) and calls `app.compositions.apply(valid_items)` **once**, so the service records one step; the DTO is unchanged.
4. **Attribute recording** (`attributes.rs`): `write` already holds `stored` (the prior row) and the row it writes → after a real write record
   `Attribute { fields: [{ key, before: Row(stored), after: Row(Some(row)) }] }` in `session_for_item(item_id)`'s history (no session → not
   recorded); `write_title` records `{ key: product.title, before: Title(old), after: Title(new) }` when the title changed; `accept_all` records one
   step with every accepted row (`before` = suggested, `after` = accepted) when `count > 0`; `clear` records `{ before: Row(live), after: Row(cleared
   row) }` (nothing when there was nothing live). Recording after the commit; `same_content` no-ops record nothing. The coalescing rule of §Core
   mechanisms applies to `set_value` and `clear` single-field steps (never to `accept_all`).
5. **Attribute reversal** (`history.rs`): undo = one `store.transaction` under the same operation lock: `tx.item(item_id)` (`NotFound` → cannot
   apply, `that product is gone`); for every field: `Row(Some(v))` → `put_attribute_value(&v)` (byte-identical row, `updated_at` included);
   `Row(None)` → `delete_attribute_value(item_id, key)`; `Title(t)` → `update_item_title(item_id, &t, now)`; each write is skipped when the stored
   row (or title) already equals the target, and when every field was skipped `set_item_activity` is skipped too (P39 D12); otherwise
   `set_item_activity(item_id, now)` once. `change_seq` is never touched. Redo = the same with `after`. `active_photo_id` is `None`; `sessions` =
   the item's session. Only `StepCannotApply` drops the step (P39 D11).
6. **Wiring**: `CompositionService { history }` in `lib.rs:426`; `AttributesService::new(…, Some(app.history.as_ref()))` in `commands/attributes.rs`
   (`with_service` and `attributes_definitions_list`), `None` in `health.rs:36` and `lib.rs:572`; `TestApp::attributes(integration) ->
   AttributesService` with the history; `p10_attributes.rs` `attrs()` gets its fourth argument.
7. Tests: `p40_undo_history_values.rs` (C1–C7), `cargo test --workspace`, clippy, fmt; `npm run typecheck` (no binding change).

## Criteria (7 criteria, 19 rows; 0 `[rule]` rows)
Fixtures: **F-comp** = p34's rendered-photo fixture (`with_fake_document`; frame `2560 × 1707`; default square after the render = `rect
{0.166601562, 0, 0.666796875, 1}`, `origin default`, `revision 0`… read the actual stored row as `c0`); `V1` = p34's valid square `{0.25,
(1 − 1280/1707)/2, 0.5, 1280/1707}`; `V2` = `V1` with `x = 0.3`. **F-attr** = p10's `app_with(true)` with `item(a, "K")` → `(item, s)`; key `K1
= custom.color_furniture` (Text, label `Color`), `K2 = custom.condition_report` (Textarea). `comp(p)` = `a.composition(p)`; `row(item, key)` =
the stored `Value` read through `store.transaction(|tx| tx.attribute_values(item))` (cleared rows included); `vals(item)` =
`attributes.values(item)` (live only). `H = a.history()`, `C = a.app.compositions`, `A = a.attributes(Some(INTEGRATION))`.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | composition set undo | F-comp, photo `p` Main 1; `c0 = comp(p)`; `C.set(p, V1)` → `c1` (`revision 1`, `origin user`); `n = seq(s)`; `out = H.undo(s)` | `out.label == "Compose 01"`, `out.active_photo_id == None`; `c2 = comp(p)`: `c2.values() == c0.values()`, `c2.origin == c0.origin` (`default`), `c2.revision == 2`, `c2.state == Ok`; `seq(s) == n + 1`; `a.events` holds a `PhotoChanged { photo_id: p, session_id: s }` and a `SessionChanged { s }` after the undo | MC-30 ("the prior composition values, saved as a new revision (MC-27)"; "a reversed composition is a saved composition"); MC-6; M36 |
| C1(b) | composition redo | from C1(a); `H.redo(s)` | `comp(p).values() == Some(V1)`, `origin user`, `revision 3`; `H.state(s) == { Some("Compose 01"), None }` | M36 ("redoing them leaves them exactly as after") |
| C1(c) | reset is a step | F-comp; `C.set(p, V1)`; `C.reset(p)` → `origin default`, `revision 2`; `H.undo(s)` | `comp(p).values() == Some(V1)`, `origin user`, `revision 3`; `H.state(s) == { Some("Compose 01"), Some("Compose 01") }` (the set below, the reset in redo) | MC-30 (Reset is a composition save); MC-27 §Values (`origin`) |
| C1(d) | no change, no step | F-comp; `C.set(p, V1)`; `C.set(p, V1)` again | `comp(p).revision == 1`; `H.state(s).undo == Some("Compose 01")` and `H.undo(s)` once leaves `comp(p).values() == c0.values()` — i.e. exactly one step was recorded | MC-30 (a step is one owner action that changed something); MC-27 ("identical values are a no-op", master plan §6.10) |
| C2(a) | Apply as one step | F-comp with three rendered photos `p, q, r` Main 1–3; `before_q = comp(q)`, `before_r = comp(r)`; `C.apply(vec![(q, V1), (r, V2), (p, INVALID)])` (p refused `off_shape`); `out = H.undo(s)` | `H.state(s).undo` before the undo `== Some("Apply framing to 2 photos")`; after: `comp(q).values() == before_q.values()`, `comp(r).values() == before_r.values()`, both `revision 2`, `comp(p).revision` unchanged (never touched); one `H.state(s)` entry moved: `{ undo: None, redo: Some("Apply framing to 2 photos") }`; `out.label == "Apply framing to 2 photos"` | MC-30 ("Apply composition to selected → every touched photo's prior composition, as one step"); M36 |
| C2(b) | Apply redo | from C2(a); `H.redo(s)` | `comp(q).values() == Some(V1)`, `comp(r).values() == Some(V2)`, both `revision 3` | M36 |
| C2(c) | the command sends one step | mock webview (`p28_tests` pattern): `composition_apply {items: [{photoId: q, values: V1}, {photoId: r, values: V2}]}`; `history_get` | the DTO `applied == [q, r]`, `refused == []`; `history_get.undo == "Apply framing to 2 photos"` (one step, not two) | MC-30; master plan §6.7 |
| C3(a) | needs_review blocks the undo | F-comp; `C.set(p, V1)`; make `p` `needs_review` as `lib.rs c8a` does (`set_variant_geometry(rotation 3.0)`, `on_variant_adjusted`, `tick(20_000)`, `complete_render_with`); `rev = comp(p).revision`; `r = H.undo(s)` | `r` is `Err(StepCannotApply { direction: "undo", label: "Compose 01", reason: "that photo changed in Capture One — re-frame it first" })`; `comp(p).state == NeedsReview`, `comp(p).revision == rev`, `comp(p).values() == Some(V1)` (nothing written); `H.state(s).undo == None` (dropped) | MC-30 ("not partly applied: nothing changes, it is dropped"); MC-27 §Revisions ("Nothing is cropped silently"); M33, M36 |
| C4(a) | set over an AI suggestion, undo restores the row | F-attr; `A.ai_suggest(item, K1, json!("Teak"), AiConfidence::High, Some("label"), vec!["https://x".into()])`; `ai = row(item, K1)`; `clock.advance(1_000)`; `A.set_value(item, K1, json!("Oak"), Human)`; `last = session(s).last_activity_at`; `n = seq(s)`; `clock.advance(1_000)`; `out = H.undo(s)` | `out.label == "Edit Color"`, `out.active_photo_id == None`; `row(item, K1) == ai` **field for field** (`value "\"Teak\""`, `provenance Ai`, `ai_confidence High`, `ai_status Suggested`, `ai_reason`, `ai_sources`, `updated_at` = the suggestion's); `session(s).last_activity_at > last`; `seq(s) == n` (no photo change) | MC-30 ("the prior value and provenance of every field the action changed"); MC-4; MC-6 ("Attribute edits do not change `change_seq`"); M36 |
| C4(b) | first value, undo removes the row | F-attr; `A.set_value(item, K2, json!("Scuffed"), Human)`; `H.undo(s)` | `row(item, K2)` is `None` (no row at all, not a cleared one); `vals(item)` has no `K2`; `H.state(s) == { None, Some("Edit Condition report") }` | MC-30; M36 ("attribute values and provenance exactly as they were") |
| C4(c) | redo | from C4(a); `H.redo(s)` | `row(item, K1)` has `value "\"Oak\""`, `provenance Human`, `ai_status None`, `ai_confidence None` | M36 |
| C4(d) | the title | F-attr (`create_local("K", "Oak", …)` → title `Oak`); `A.set_value(item, PRODUCT_TITLE, json!("Oak table"), Human)`; `H.undo(s)`; `H.redo(s)` | after the undo `store.item(item).title == "Oak"` and `H.state(s).redo == Some("Edit Title")`; after the redo `title == "Oak table"`; `seq(s)` unchanged throughout | MC-30; ch05 (the title's one home); MC-6 |
| C4(e) | coalescing while typing | F-attr; `A.set_value(item, K1, json!("O"), Human)`; `A.set_value(item, K1, json!("Oa"), Human)`; `A.set_value(item, K1, json!("Oak"), Human)`; `H.undo(s)` | one step: `H.state(s).undo` before `== Some("Edit Color")` and after the single undo `row(item, K1) == None` and `H.state(s) == { None, Some("Edit Color") }` (the three saves were one step with the first `before`) | MC-30 §Not steps ("Typing inside a text field is the field's own undo"; "a field committed" is one step); ch08 ("Values save locally as typed") |
| C4(f) | no coalescing across fields | F-attr; `A.set_value(item, K1, json!("Oak"))`; `A.set_value(item, K2, json!("Scuffed"))`; `A.set_value(item, K1, json!("Teak"))`; `H.undo(s)` ×3 reading `vals` after each | after 1: `K1 == "Oak"`, `K2 == "Scuffed"`; after 2: `K1 == "Oak"`, no `K2`; after 3: no `K1`; `H.state(s).undo == None` | MC-30 (three owner actions, three steps); M36 |
| C5(a) | accept all | F-attr; `ai_suggest` `K1` High, `K2` Medium, `custom.condition_cracks` Low (`json!("Minor")`); `A.accept_all(item)` → `2`; `H.undo(s)` | `H.state(s).undo` before `== Some("Accept all suggestions")`; after: `row(item, K1).ai_status == Some(Suggested)`, `row(item, K2).ai_status == Some(Suggested)`, the Low row untouched (`Suggested` throughout); `H.state(s) == { None, Some("Accept all suggestions") }` | MC-30 ("Accept all"); M36 |
| C5(b) | clear | F-attr; `A.set_value(item, K1, json!("Oak"))`; `A.clear(item, K1)` → the row is `Cleared`, not live; `H.undo(s)` | `row(item, K1).ai_status == None` and `is_live`, `value "\"Oak\""`, `provenance Human`; `vals(item)` has `K1`; `H.state(s).redo == Some("Clear Color")`; a second `A.clear(item, K2)` on an empty field records nothing (`H.state(s).undo` unchanged) | MC-30 ("Clear"); M36 |
| C6(a) | one list across kinds | F-attr with one rendered Main photo `p` (F-comp fixture inside the same app); `R.classify(u, Main)` (`u` unreviewed → Main 2); `A.set_value(item, K1, json!("Oak"))`; `C.set(p, V1)`; `H.undo(s)` ×3 reading `H.state(s).undo` before each | `Some("Compose 01")` → `Some("Edit Color")` → `Some("Approve 02")` → `None`; after the three undos `comp(p).values() == c0.values()`, `vals(item)` has no `K1`, `state(s, Main) == [(p, 1)]` | MC-30 ("One history per session" — every kind in the same list, LIFO); M36 |
| C7(a) | events | F-attr; `A.set_value(item, K1, json!("Oak"))`; `H.undo(s)` | `a.events` holds `HistoryChanged { session_id: s, undo: Some("Edit Color"), redo: None }` then `HistoryChanged { s, None, Some("Edit Color") }` | MC-30 (header icons and menu follow the history) |
| C7(b) | command DTO for an attribute step | mock webview: `attribute_values_set {itemId, key: K1, value: "\"Oak\""}`; `history_undo {sessionId: s}` | `{"label":"Edit Color","direction":"undone","active_photo_id":null,"sessions":[{"id": s, …}]}`; `attribute_values_get {itemId}` afterwards has no `K1` | MC-30; master plan §6.7 |

## Delegated choices
Where `save_composition_in` lives (a free function in `store.rs`); whether `apply` validates all items before saving any (today it saves what
it can — keep that) or records partial success as the step (it does, by rule); how the composition label reads the photo's position
(`tx.photo`); whether `FieldState::Title` is a separate variant or `Row` with a synthetic row (the variant is recommended: the title is not a
row); the `reason` texts beyond those fixed in task 0; whether coalescing also requires the same `item_id` (it does — the key is `(item,
key)`).

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
- **Tests that change their expectation**: `p10_attributes.rs` `attrs()` only (a fourth argument). No assertion changes anywhere.
- **Projection** is on: the transactional composition path and the byte-identical attribute restore are the two places a plausible
  implementation can silently diverge (a `− 1` revision; a restore through `set_value` that re-canonicalises and loses `ai_*`).
- Registry additions made by the planner in master plan §6.4, §6.10. No intention-book change.
- Estimated size: ~12 paths, ~900 lines (≈ 400 core/infra, ≈ 500 tests).

## Review log

- **2026-11-05, planner carry-over from P39's projection r0** (`handoff_P39_undo_history_core_projection_r0.md` D10–D12, D7): tasks 2 and 5 now
  state that composition and attribute reversals run under the `History` operation lock (D10), that only `StepCannotApply` drops a step (D11),
  that a reversal whose target equals the stored state writes nothing and does not touch activity or `change_seq` (D12), and that a composition
  step checks the photo is still in the step's session (D7's analogue). No row change: C1(d) already pins "no change, no step"; the lock and the
  drop rule are P39's rows C2(e)/C4(c) over the shared `HistoryService`. P40's own projection remains on.

