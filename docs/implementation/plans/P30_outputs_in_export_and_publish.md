---
phase: P30_outputs_in_export_and_publish
title: Outputs in export and publishing — two files per split photo, upload names with -L/-R, links per slot with composition_rev, replacement and retirement, gallery and damage report per output, refusal while needs_review, masters released after publish
difficulty: high
projection: no
depends_on: [P29_master_and_output_render]
---
# P30 — Outputs in export and publishing: two files per split photo, upload names with -L/-R, links per slot with composition_rev, replacement and retirement, gallery and damage report per output, refusal while needs_review, masters released after publish

## Goal
Implement 10D MC-29 (invariant M35) and the publish half of MC-28's cache and master rules. A `normal` photo has one output (slot `single`);
a `split` photo has two (`left`, then `right`); positions stay per photo and outputs expand in photo order when a run stages its files: a
split at main place 3 writes `<key>_03_main.jpg` and `<key>_04_main.jpg` and later photos move down one; `item.json` lists the output
files in order (M15 per output). Uploads are named `<folder_key>-<uid12>.jpg` for `single`, `-L.jpg` / `-R.jpg` for the halves, and a
revision adds `-r<n>` after that. `photo_links` gains `slot` (its key becomes `(photo_id, integration, item_id, role, slot)`, migration 005)
and `composition_rev`, the revision its output was rendered from; the 07B MC-21 replacement also fires when the photo's composition revision
is greater than the link's; a link whose slot the photo no longer has (Split ↔ Normal) is **retired** — its media or file id joins the
item's `own` set and gallery reconciliation removes it. The gallery `D` is the ordered list of main outputs (the cover is main 01's first
output: the left half of a split); the damage report lists the selected condition photos' outputs in order. Publish confirmation refuses,
naming the photos, while any main or condition photo of the session is `needs_review`, and `prepare` fails the item the same way if that
happens after confirmation. A staging folder from an earlier attempt is reused only when its `manifest.json` keys equal the current
`(adjusted_gen, composition revision, output_format)` of every output, so a stale output is never uploaded. When a publish item reaches
`done` and the session is not dirty, the session's masters are deleted (MC-28 §Master). **Not in this phase:** any UI (the publish popover
keeps one thumbnail per photo; split tiles are P33), `composition_apply` (P34), the editor.

## Read first
Book: `10D_composition_and_framing.md` **§MC-29 in full** (Outputs, Export, Upload filename, Links, Gallery and cover, Damage report, Dirty,
M35), §MC-28 §Master (masters after publish), §Cache key ("Stale outputs are never … uploaded"), §MC-27 §Revisions (publish confirmation
refuses) · `05A` MC-8 §Outputs ("`NN` numbers outputs"), MC-6 · `07A` MC-11 (upload state machine, adoption by stem, "links are per output
slot"), MC-12 (`D` one per output; `own`), MC-14 (condition files per output) · `07B` MC-21 (replacement; "the same replacement fires when the
photo's composition revision is greater than the link's `composition_rev`") · `05_domain_model.md` §photo_links · `14` (M35; M15, M18, M19,
M21, M28 keep holding) · `01` (H1, H4, H7) · master plan §3, §6.4 (`CommerceAdapter::staged_uploads_create(run, stem, byte_size)`;
`Store` link methods with `slot`), §6.5 (`photo_links` after 005), §6.6 (`OutputSlot`; `UploadState::Retired`), §6.10, §7.5 (`FakeCommerce`
records uploaded hashes) · process `flow.md` §4, §6, §6A, §9 · plans `P29_master_and_output_render.md` (the export rework this phase builds on:
`output_names`, `manifest.json`, `master_ready`, `MasterFiles::remove_all`), `P28_composition_foundation.md` (`CompositionKind::slots`,
`save_composition` bumps `change_seq`), `P14_publish_update.md` C5 rows (the retouch replacement rows this phase generalises) · code (line
numbers at `d763492`; P29 shifts the services): `crates/core/src/services/publish.rs` (`confirm :206–291`; `run_item :318–399` — the `Done`
branch swaps the staging `:343–368`; `prepare :400–522`: the staging-reuse check `:440–455`, the per-photo link loop `:477–520` with
`replacement_candidates :503–505` and `export_sha256 :514`; `upload_stem :523–533`; `upload :534–666`: the stem `:581–589`, adoption
`:605–615`, `staged_uploads_create :619–625`, `product_create_media`/`file_create :631–641`; `update :667–905`: `desired :680–690`, `own
:693–713`, `reconcile::plan :715`, the condition ids `:750–769`), `crates/core/src/mechanisms/publish.rs` (`replacement_candidates :6–19`
+ its unit test `:62–82` — extended to the composition revision), `crates/core/src/mechanisms/upload_name.rs` (`matches :2–13`; add `stem`),
`crates/core/src/domain/publish.rs` (`PhotoLink :239–250`), `crates/core/src/ports/commerce.rs` (`staged_uploads_create :78–85`),
`crates/core/src/ports/store.rs` (`photo_link :185`, `upsert_photo_link :186`, `photo_links_for_item :187`, `export_retouch_gen :184`),
`crates/core/src/services/sessions.rs` (`view :121–145` — `upload_states` by `photo_link(photo, item, role)`), `crates/infra/src/sqlite/
store.rs` (`photo_link_row :285–297`, `map_photo_link :298–313`, `photo_link :1438–1444`, `upsert_photo_link :1445–1448`, `photo_links_for_item
:1449–1462`; `SqliteStore::open :27–38` — expose the migration list for C3(b)), `crates/infra/src/sqlite/migrations/002_domain.sql`
(`photo_links :61–76`), `crates/integrations/src/commerce/shopify/adapter.rs` (`upload_filename :95`, `staged_uploads_create` — takes the
stem now), `crates/integrations/src/lib.rs` (`PublishJob :864–889` — masters released after `done`) · fakes: `crates/testkit/src/fake_commerce.rs`
(`staged_uploads_create :308–340` records the filename; `upload :341` — record the SHA-256 of `bytes` per filename; `file_create :452–476`;
`product_reorder_media :387–417`; `list_media`), `crates/testkit/src/test_app.rs` · tests to reuse: `p13_publish_runs.rs` (`photo :30`,
`condition :56`, `publish :113`, `drain :107`, `c5c :713` gallery order, `c4e :682` moved photo), `p14_publish_update.rs` (`c4a :509`
condition files in order, `c5a–c5d :645–790` the retouch replacement shapes to copy for the composition revision), `p09_local_export.rs`
(`names`, `export`), `p12_shopify_adapter.rs` (every `staged_uploads_create(` call site — the new `stem` argument; grep).

## Files expected to change
Backend: `crates/infra/src/sqlite/migrations/005_photo_link_slots.sql` (new), `crates/infra/src/sqlite/store.rs`, `crates/core/src/domain/
publish.rs`, `crates/core/src/ports/{store.rs, commerce.rs}`, `crates/core/src/mechanisms/{publish.rs, upload_name.rs, naming.rs}`,
`crates/core/src/services/{publish.rs, export.rs, sessions.rs}`, `crates/core/src/error.rs` (if `NeedsReview` needs a `publish` variant —
Delegated), `crates/integrations/src/commerce/shopify/adapter.rs`, `crates/integrations/src/lib.rs`, `crates/testkit/src/{fake_commerce.rs,
test_app.rs}`.
Tests: `crates/testkit/tests/p30_outputs_in_export_and_publish.rs` (new: C1–C8), amendments to `p12_shopify_adapter.rs` (call sites) and
`p13`/`p14` where a `PhotoLink` literal gains `slot`/`composition_rev`.

## Core mechanisms (unit tests allowed)
`psm_core::mechanisms::publish::replacement_candidates` keeps its existing unit test, extended with the composition-revision clause (it is
already on the list of P13). Nothing else.

## Ordered tasks
0. **Registry first (master plan §6.10).** `PhotoLink { …, slot: OutputSlot, composition_rev: i64 }`; `upload_state` gains the value `retired`
   (`UploadState::Retired`); `Store::photo_link(photo_id, item_id, role, slot)`, `upsert_photo_link` (writes both new columns),
   `photo_links_for_item` (unchanged shape, rows carry the slot), `retire_photo_links(item_id, photo_id, role, keep: &[OutputSlot], now) ->
   Vec<PhotoLink>` (every link of that photo/item/role whose slot is not in `keep` and whose state is not already `retired` → `upload_state =
   'retired'`, returned); `CommerceAdapter::staged_uploads_create(run, stem: &str, byte_size)` (the adapter appends `.jpg`; core builds the stem);
   `mechanisms::upload_name::stem(folder_key, uid, slot, revision) -> String` = `<folder_key>-<uid12>` + (`-L` | `-R` | nothing) + (`-r<n>` when
   `n ≥ 1`); `replacement_candidates(links, retouch_gens, composition_revs)` → a `ready` link is replaced when its RAW's `retouch_gen` is newer
   **or** the photo's composition revision is greater than `link.composition_rev`.
1. **Migration `005_photo_link_slots.sql`** (SQLite cannot alter a unique constraint): `CREATE TABLE photo_links_v2 (id INTEGER PRIMARY KEY,
   photo_id INTEGER NOT NULL REFERENCES photos(id), integration TEXT NOT NULL, item_id INTEGER NOT NULL REFERENCES items(id), role TEXT NOT
   NULL, slot TEXT NOT NULL DEFAULT 'single', external_id TEXT, revision INTEGER NOT NULL DEFAULT 0, retouch_gen INTEGER NOT NULL DEFAULT 0,
   composition_rev INTEGER NOT NULL DEFAULT 0, superseded_ids TEXT, upload_state TEXT, export_sha256 TEXT, error TEXT, updated_at INTEGER NOT
   NULL, UNIQUE(photo_id, integration, item_id, role, slot)); INSERT INTO photo_links_v2 (id, photo_id, integration, item_id, role, slot,
   external_id, revision, retouch_gen, composition_rev, superseded_ids, upload_state, export_sha256, error, updated_at) SELECT id, photo_id,
   integration, item_id, role, 'single', external_id, revision, retouch_gen, 0, superseded_ids, upload_state, export_sha256, error, updated_at
   FROM photo_links; DROP TABLE photo_links; ALTER TABLE photo_links_v2 RENAME TO photo_links;`. `SqliteStore::migrations() -> Vec<M<'static>>`
   exposed so a test can stop at version 4 (C3(b)).
2. **Export numbering** is P29's `output_names` (already per output); this phase only proves it with a split (C1) and uses it in publish.
3. **Publish: confirmation and prepare gates.** `confirm`: before freezing, the current main photos and selected condition photos with
   `composition.state == needs_review` → `Err(NeedsReview { photos })` (labels in display order, `Main 02`, `Condition 01`) and nothing written;
   `publish_preview` is unchanged. `prepare`: the same check over the frozen (selected) lists → `Err(NeedsReview)`, so the item fails at `prepare`
   with `error` = the `needs_review` message (`Re-frame first: …`); Retry re-runs `prepare`.
4. **Prepare: staging reuse by manifest.** The existing-staging check becomes: `manifest.json` exists, `manifest.format_key == settings.output_format
   .format_key()`, and for every output `(photo, slot)` of the current selection (`output_names` over frozen main then condition) the entry under
   its file name has `adjusted_gen == store.adjusted_gen(raw)` and `composition_rev == store.composition(photo).revision`; otherwise discard the
   staging and `run_staged` again (which renders only what the output cache lacks).
5. **Prepare: links per output.** Loop over `output_names` (main → `gallery`, condition → `attribute:<key>`): `retire_photo_links(item, photo,
   role, comp.kind.slots())` first; then per `(photo, slot, name)`: `link = photo_link(photo, item, role, slot)` or a new `PhotoLink { slot,
   composition_rev: manifest.composition_rev, retouch_gen: export_retouch_gen(…) }`; replacement per `replacement_candidates` (retouch **or**
   composition): `revision += 1`, old `external_id` → `superseded_ids`, `upload_state none`, `retouch_gen`/`composition_rev` = the manifest's;
   `export_sha256` = the staged file's hash; upsert.
6. **Upload per output.** The loop walks `output_names`; `stem = upload_name::stem(folder_key, uid, slot, link.revision)`; adoption matches the
   stem; `staged_uploads_create(run, &stem, len)`; `file_create(run, url, &format!("{stem}.jpg"))`; `uploaded`/`total` count outputs
   (`PublishItem.total` is set at confirm from the output count). Retired links are skipped (not uploaded, not counted).
7. **Update per output.** `desired` = for each selected frozen main photo, for each slot of its **current** composition kind, the `ready` link's
   id (a missing or unready link → `UploadIncomplete` as today); `own` keeps collecting every link's `external_id` and `superseded_ids`
   (retired links included) plus filename matches; the condition metafield value = the selected condition photos' outputs' file ids in order.
   Gallery read-back must equal `desired` (cover = `desired[0]`).
8. **Masters after publish.** In `PublishJob::run` (or `run_item`'s `Done` branch): when the item reached `done` in this call and
   `store.session(session_id).dirty == false` → `export.release_masters(session_id)` (P29's `MasterFiles::remove_all` for each RAW of the
   session's non-deleted photos); dirty → kept.
9. **Session view.** `upload_states[i]` = the state of the photo's **first slot** link (`single` or `left`), so the tile badge keeps meaning
   "this photo's upload" (P33 may widen it).
10. **Fake.** `FakeCommerce::upload` records `uploaded_sha[filename] = sha256(bytes)` (filename from the `StagedTarget`); `TestApp::fake_shop()`
    exposes it. Tests, `cargo test --workspace`, clippy, fmt; no bindings change (no command changed); `npm test`/typecheck unaffected.

## Criteria (8 criteria, 20 rows; 0 `[rule]` rows)
Fixtures: `K` as p13/p14 (`with_fake_shop`, `condition(&a)` defines `custom.damage_report`), photos through P29's `photo` helper (each with a
2560×1707 preview so compositions have a frame); `split(a, id)` = `composition_set(id, {split, 1:1, named, 0, none, 0, rect = default_rect(split,
…, 2560×1707)})` (P28 C1 case 10); `publish(a, s)` = p13's `publish` + `drain` (masters complete with P29's default `SyntheticMaster`; `tick` runs
`render_output`); `uid12(p)` = the first 12 hex of the photo's uid; `media_names()` = the fake shop's gallery filenames in order.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `with_fake_document()` (no shop); `K` main `[a, b, c]`, condition `[d]`; `split(a)`, `split(d)`; `a.app.export.run(s)` + complete | a local export with a split main at place 1 and a split condition | `names(K/)` = sorted `[K_01_main.jpg, K_02_main.jpg, K_03_main.jpg, K_04_main.jpg, K_01_condition.jpg, K_02_condition.jpg, item.json, manifest.json]`; `item.json`: `main == ["K_01_main.jpg","K_02_main.jpg","K_03_main.jpg","K_04_main.jpg"]`, `condition == ["K_01_condition.jpg","K_02_condition.jpg"]`; `manifest.outputs["K_01_main.jpg"].slot == "left"`, `["K_02_main.jpg"].slot == "right"`, `["K_03_main.jpg"] == {photo b, "single"}` | MC-29 §Export ("a split at main place 3 writes `_03` (left) and `_04` (right) and later photos move down one"); M15 per output |
| C1(b) | as (a) but `b` is deleted between `run` and the last `render_output` completion (P09 c-row shape: `review().delete(b)` mid-run) | a photo leaving mid-run | files renumbered: `K_01_main` (a left), `K_02_main` (a right), `K_03_main` (c); no `K_04_main.jpg`; `item.json` lists three | MC-8 (deleted photos never export); MC-29 §Export |
| C2(a) | `with_fake_shop`; `K` main `[a, b]`, `split(a)`; `publish(a, s)` | upload filenames | the fake's staged filenames, in order: `K-<uid12(a)>-L.jpg`, `K-<uid12(a)>-R.jpg`, `K-<uid12(b)>.jpg`; `media_names()` in that order (the fake keeps gallery order = creation order before reorder) | MC-29 §Upload filename; MC-11 |
| C2(b) | after (a): `composition_set(a, <different valid split rect>)` (revision 2); `publish` again | a revision of a split photo | staged filenames of the second run: `K-<uid12(a)>-L-r1.jpg`, `K-<uid12(a)>-R-r1.jpg` only (b unchanged, skipped as `ready`); the gallery holds exactly three media: `…-L-r1`, `…-R-r1`, `K-<uid12(b)>` in that order | MC-29 §Upload filename ("a revision adds `-r<n>` after that (`…-L-r2.jpg`)"); MC-21 |
| C2(c) | `K` condition `[d]`, `split(d)`; `condition(&a)`; `publish` | condition files | `file_create` filenames `K-<uid12(d)>-L.jpg`, `K-<uid12(d)>-R.jpg`; the metafield `custom.damage_report` value equals `[file(d-L), file(d-R)]` in order | MC-29 §Damage report; MC-14; M21 |
| C3(a) | after C2(a): `photo_links_for_item(K)` | links per slot | rows for `a`: `(gallery, left, ready, composition_rev 1, revision 0)`, `(gallery, right, ready, composition_rev 1, revision 0)`; for `b`: `(gallery, single, ready, composition_rev 0)`; `photo_link(a, K, "gallery", Left)` returns the left row; no row for `(a, single)` | MC-29 §Links (key with slot; `composition_rev` recorded) |
| C3(b) | a temp DB: `SqliteStore::migrations()[..4]` applied with `to_version(4)`; insert an `items` row, a `photos` row and a `photo_links` row (`role gallery`, `external_id "m1"`, `upload_state ready`, `revision 2`, `retouch_gen 1`); then `SqliteStore::open` on that file | migration 005 over existing data | the row reads back with `slot == "single"`, `composition_rev == 0`, `external_id "m1"`, `upload_state ready`, `revision 2`, `retouch_gen 1`, same `id`; `PRAGMA index_list` shows a unique index over `(photo_id, integration, item_id, role, slot)` | MC-29 §Links ("its key becomes (photo_id, integration, item_id, role, slot)"); charter rule 7 |
| C3(c) | `K` main `[a, b]` published (normal); `composition_set(a, <valid different rect>)` (revision 1); `publish` | replacement on composition revision | `a`'s link: `revision 1`, `composition_rev 1`, `superseded_ids == [old media id]`, `upload_state ready`; the gallery has exactly one media for `a`, at position 1 (same place), named `K-<uid12(a)>-r1`; `b` untouched (`revision 0`, no new upload) | MC-29 §Links ("the replacement rule also fires when the photo's composition revision is greater than the link's"); MC-21; M35 ("every uploaded file was rendered from the photo's current composition revision") |
| C3(d) | `K` main `[a]` published; `composition_set(a, <identical values>)` (no revision change); `publish` | a no-op save | no new upload (`staged filenames` unchanged), link `revision 0`, `composition_rev 0`, session not dirty before the publish (`dirty == false`, so the Publish button would not even be offered — the row runs `confirm` directly) | MC-29 §Dirty ("Opening and closing the editor without a change is not" a change) |
| C4(a) | `K` main `[a, b]` published normal; `split(a)`; `publish` | Normal → Split | the `(a, single)` link has `upload_state == "retired"` and keeps its `external_id`; the gallery no longer contains that media id (deleted by reconciliation, no `foreign_media_added`); new links `(a, left)`, `(a, right)` are `ready`; gallery order `[a-L, a-R, b]` | MC-29 §Links ("A link whose slot the photo no longer has … is retired: its media … joins the item's `own` set and is removed by gallery reconciliation") |
| C4(b) | after (a): `composition_set(a, {normal, 1:1, …, default square rect})`; `publish` | Split → Normal | `left`/`right` links `retired`, their media gone from the gallery; a new `(a, single)` link `ready`, uploaded as `K-<uid12(a)>.jpg` **with no `-r` suffix** (a new slot starts at revision 0); gallery `[a, b]` | MC-29 §Links; MC-11 |
| C4(c) | after (b): `photo_links_for_item(K)` | the retired rows | the three retired rows still exist (`retired`, external ids kept) — they are the `own` evidence for any later reconciliation; `SessionsService::view(s).upload_states[a] == Some("ready")` (the first slot's link) | MC-12 (`own` = any gallery link for this item) |
| C5(a) | `K` main `[a, b]`, `split(a)`; `publish` | gallery and cover | `media_names()` == `[K-<uid12(a)>-L, K-<uid12(a)>-R, K-<uid12(b)>]` (uniquifier-free in the fake); the product's first media (cover) is the `-L` file | MC-29 §Gallery and cover ("`D` is the ordered list of main outputs; the cover is main 01's first output (the left half of a split)"); M35; M19 |
| C5(b) | after (a): `review().reorder(b, 1)`; `publish` | a reorder | `media_names()` == `[K-<uid12(b)>, K-<uid12(a)>-L, K-<uid12(a)>-R]` (halves stay adjacent, Left first) | M35 ("a split's halves adjacent, Left first") |
| C6(a) | `K` condition `[d, e]`, `split(d)`; `condition(&a)`; `publish` | the damage report | metafield value == `[file(d-L), file(d-R), file(e)]`; none of the three is in the gallery | MC-29 §Damage report ("a split condition photo contributes Left then Right"); M21; M3 |
| C7(a) | `K` main `[a, b]`, `b` `needs_review` (P28 C5(a) shape); `publish.confirm(&[s], "…")` | confirmation while a photo needs review | `Err(NeedsReview { photos: ["Main 02"] })`; `SELECT COUNT(*) FROM publish_runs` unchanged; the fake shop recorded no mutation | MC-27 §Revisions ("publish confirmation … refuse while any of the session's main or condition photos needs it (naming them)"); M33; H1 |
| C7(b) | `K` main `[a, b]`; `confirm` (ok); **before** the first `tick`, make `b` `needs_review` (an incompatible preview); `drain` | needs_review after confirmation | the publish item is `failed` at `prepare` with `error` containing `Re-frame first: Main 02`; no remote mutation; then `composition_set(b, <valid>)` (confirms) and `publish.retry(item)` + `drain` → `done`, gallery `[a, b]` | MC-27 §Revisions; MC-11 (retry re-runs the failed step) |
| C8(a) | `K` main `[a]`; `fake_shop().fail_before_effect("product_create_media")`; `publish` → item `failed` at `upload`; clear the fault; `composition_set(a, <valid different rect>)` (revision 1); `publish.retry(item)` + `drain` | a stale staging from a failed attempt | the item reaches `done`; the uploaded file's SHA-256 (`fake_shop().uploaded_sha["K-<uid12(a)>.jpg"]`) equals `sha256(outputs/…/<uid12(a)>-single-g<g>-r1-jpeg-2048-q90.jpg)` and the link's `export_sha256`; the old `.K.partial-<old run>` staging is gone and a new one was rendered (manifest `composition_rev 1`) | MC-28 §Cache key ("Stale outputs are never … uploaded"); M35; M18 |
| C8(b) | `K` main `[a]` with a frame; `publish` to `done`; session not dirty | masters after publish | `masters/<pk(a)[0..2]>/<pk(a)>/` does not exist; `outputs/…` for `a` exists | MC-28 §Master ("deleted once its publish … completes and the session is not dirty") |
| C8(c) | `K` main `[a, b]`; `confirm`; run `prepare` and `upload` (drain until the item is at `update`), then `review().reorder(b, 1)` (a photo change during the run), then finish | a dirty session at `done` | item `done`; `session.dirty == true`; `masters/…/<pk(a)>/master-<g>.tif` still exists | MC-28 §Master ("… and the session is not dirty"); MC-6 (R-I13) |

## Delegated choices
Whether `NeedsReview` at `prepare` reuses `Error::NeedsReview` or a publish-specific message (the `error` text must contain `Re-frame first:`
and the labels); whether `retire_photo_links` runs at `prepare` only or also at `upload` (prepare is required); how `output_names` carries the
`Group`; whether the `uploaded`/`total` counts are stored per output (required) and shown per output in the progress event (`total` =
outputs); the `FakeCommerce` field name for uploaded hashes; whether `SqliteStore::migrations()` is `pub` or `pub(crate)` with a test hook.

## Manual try-out
1. Open a session with a few main photos, publish to the test store (Shopify configured): the gallery shows the square compositions at 2048
   px, in order; the cover is main 01. Capture One rendered one TIFF per photo during the publish; afterwards the `masters/` folder in the
   data directory is empty for that session.
2. Change nothing and publish again: nothing is uploaded (the popover says what would change; the run completes at once).
3. In the database set one photo's composition to split (`sqlite3 … "update compositions set kind='split', revision=revision+1, x=0, y=0.1251,
   w=1, h=0.7499 where photo_id=<id>"` — the editor arrives in P31–P33) and publish: that photo becomes two square media, Left then Right,
   adjacent, and later photos move down; the export folder shows `_0n` and `_0n+1` for it.
4. Set it back to normal the same way and publish: the two halves leave the gallery, one media returns in the same place.
5. Rotate a photo 90° in Capture One (needs review): Publish is refused with "Re-frame first: Main 0n"; Export likewise.

## Notes
- **Retired links stay as rows.** They are the memory that a media id once belonged to this item (`own`); deleting the row would make the
  media `foreign` at the next reconciliation and block the publish (R-I5). A retired link is never uploaded and never in `D`.
- **`total` counts outputs**, so the progress bar's "Uploading n of N" and the `upload_state` badge keep their meaning; the badge shows the first
  slot's state until P33 shows two.
- **Why `confirm` and `prepare` both check `needs_review`.** Confirmation is the owner-facing refusal the book names; `prepare` is the safety net
  for a Capture One change between confirmation and the run (the frozen list is otherwise sent as it was).
- **The manifest check replaces "all files exist".** A staging left by a failed attempt before a composition change would otherwise pass the old
  check and upload a stale rendering — the exact M35 defect ("old framing left on the shop").
- Registry additions made by the planner in master plan §6.4–§6.6, §6.10, §7.5. No intention-book change.
- Estimated size: ~16 files, ~1 100 lines (≈ 450 publish/export/store, ≈ 100 migration + adapter, ≈ 550 tests).

## Review log

### 2026-11-04 — implement r1 (claude-opus, Claude fallback for the Codex implementer)
**Built.** Migration `005_photo_link_slots.sql` (table recreated, rows kept as `single`/`composition_rev 0`, UNIQUE over the five
columns) and `SqliteStore::migrations()` (pub). `PhotoLink { slot, composition_rev }`; `Store::photo_link(.., slot)`,
`upsert_photo_link` (both columns), `retire_photo_links`. `CommerceAdapter::staged_uploads_create(run, stem, byte_size)`: the Shopify
adapter and the fake only append `.jpg`. `upload_name::stem(folder_key, uid, slot, revision)`. `replacement_candidates(links, retouch_gens,
composition_revs)` (unit test extended + one new case). Publish: `confirm` refuses with `NeedsReview` (labels) before writing, `total` =
outputs; `prepare` checks `needs_review` first (before any remote write), reuses a staging only when its `manifest.json` matches the
current outputs (photo, slot, `adjusted_gen`, composition revision, format key, entry count), retires links of slots the photo no longer has,
and writes one link per output; `upload` walks the outputs (stem per slot and revision); `update` builds `D` and the damage report per
output, `own` includes retired links and stems per slot/revision. Masters released in `run_item`'s `Done` branch when the session is not
dirty (`ExportService::release_masters(session_id)`). Session view: first slot's link. Fake: `staged_filenames`, `uploaded_sha`,
`media_poll_entered`. Tests: `p30_outputs_in_export_and_publish.rs` (20 rows); signature amendments in p01 (schema columns), p12, p13,
p14, p15.

**Delegated choices.** `NeedsReview` reuses `Error::NeedsReview` (message `Re-frame first: Main 02`). Retirement runs at `prepare` only.
`output_names` carries the group as the loop's group (helper `outputs`). `total`/`uploaded` count outputs, also in the progress event.
Fake field `uploaded_sha` (+ `uploaded_sha(filename)`). `migrations()` is `pub`.

**Assumptions.** (1) `UploadState::Retired` is the string `"retired"`: `upload_state` is a `String` throughout the code (the registry's
enum was never introduced). (2) Stem case: the stored folder key as is, the uid hex lowercase, `-L`/`-R` uppercase as 10D writes them;
`upload_name::matches` now ignores case so names the adapter lowercased before 10D are still recognised (folder keys are unique ignoring
case, 05B). (3) A publish item resumed at `upload` whose staging no longer matches the current outputs goes back to `prepare` (C8(a): Retry
re-runs the failed `upload` step, and the stale staging must not be uploaded). (4) A known export run that failed fails the item with
`export failed: <reason>` instead of waiting forever (the P29 backlog's stuck-run note); Retry starts a fresh export. (5) Links not yet
`ready` take the staged file's `retouch_gen`/`composition_rev` at `prepare` (they record what they will upload). (6) Adoption by stem
skips ids in the link's `superseded_ids`. (7) The fake's `fail_before_effect` on `staged_uploads_create`, `product_create_media` and
`file_create` now fails before anything is created (before, the media was pushed and then the error returned).

**Deviations.** (a) C4(b)/(c): the plan's own UNIQUE key allows one `(a, single)` row, so the returning `single` slot reuses its retired
row: state back to `none`, old media id moved to `superseded_ids` (still `own`), revision kept (0 → `K-<uid12>.jpg`, as C4(b) asks).
After C4(b) there are therefore **two** retired rows (left, right) plus the first single media id in the single row's `superseded_ids`,
not "three retired rows"; C4(c) asserts that. (b) C3(d): `CompositionService::set` with identical values on a **default-origin**
composition is a change under P28 (origin default → user, revision +1), and `reset` recomputes the default rect on the master frame; the
row therefore publishes a user composition first (revision 1) and re-saves it identically: no revision change, no upload, link
`composition_rev 1` (not 0). Note for P31: an editor that opens and closes without a change must not call `set` on a default composition.
(c) C8(c): the photo change happens while `upload` holds at its first media poll (a worker tick cannot be stopped between `upload` and
`update`). (d) Files outside the listed perimeter, all forced by the signature changes: `crates/integrations/examples/shopify_probe.rs`
(two call sites), `crates/testkit/tests/p01_storage_identity.rs` (schema column list), `p15_publish_backend.rs` (`PhotoLink` literals),
`crates/testkit/Cargo.toml` + `Cargo.lock` (`rusqlite_migration` dev-dependency for C3(b)). The adapter's `upload_filename` and its unit
test were removed (the name is core's `stem` now, covered by C2).

**Observation.** A link that crashed between media create and record and whose photo's composition then changed would adopt the stale
media by stem at the same revision (the replacement rule only fires on `ready` links). Not reachable by the plan's rows; candidate backlog
note.

### 2026-11-04 — review r1 (opus): CHANGES_REQUESTED
**Gate.** Tracker `IMPLEMENTED`; checkpoint `a3d561f` in `git log`. Perimeter: `git show --stat a3d561f` = the 23 paths the handoff
declares (the 4 outside the plan's list are declared under Deviations (d), forced by the signature changes). Suite/lint/typecheck run once
on clean `fd9c240` (= `a3d561f` + two docs files): `cargo test --workspace` exit 0, clippy `-D warnings` clean, `fmt --check` clean,
`npm run typecheck` clean, `npm test` 132 passed.

**Verified correct.** Migration 005 (table recreated, rows kept as `single`/`composition_rev 0`, same `id`, UNIQUE over the five
columns) and C3(b)'s test over a real v4 database. `upload_name::stem` (`-L`/`-R`, `-r<n>` only for n ≥ 1) and `matches` (exact or
`_<uuid v4>` uniquifier; a `-L` suffix can't pass as a uniquifier). `replacement_candidates` retouch **or** composition clause, ready
links only. `confirm` checks `needs_review` before any write and sets `total` = outputs; `prepare` checks it before any remote call.
`staging_current`: format key, entry count, per-output photo/slot/`adjusted_gen`/composition revision, file present. Retirement at
`prepare` (state only, external id kept), `own` = every gallery link's id + superseded ids + stems for every slot and revision 0…n;
`D` and the damage report per current slot; session view on the first slot. Masters released in `run_item`'s `Done` branch only when not
dirty, skipping RAWs an export in flight or a pending master still needs. Every row's test observes its outcome at its boundary and
would fail on a wrong outcome (C3(d) and C4(c) as amended by the implementer's deviations (a) and (b), whose outcomes are the rows'
intent under the plan's own UNIQUE key and P28's origin rule). H1–H8 hold except as F1 says for H4.

**F1 — B1, route production.** `prepare` stamps a non-`ready` link with the staged file's `composition_rev`/`retouch_gen` (assumption 5),
including a link in `staged`/`uploaded`/`attached` whose remote media was already created from the **previous** rendering. `upload` then
adopts that media (by stem when `external_id` is `None`, or polls the recorded id when `attached`) and marks it `ready`. The shop keeps
the old framing, while the link claims the current revision and the new hash, so no later publish replaces it. Reviewer probe (scratch
worktree at `a3d561f`, removed afterwards): `K` main `[a]`, `fail_after_effect("product_create_media")`, kill, `kill_and_restart`,
`other_square(a)` (revision 1), drain → `done`; link `ready`, `composition_rev 1`, `export_sha256` = sha(r1 output) `b904…`; the gallery's
only media `K-<uid12>.jpg` was uploaded with sha `a114…` (revision-0 bytes); staged filenames `[K-<uid12>.jpg]` (no new upload);
publish again → nothing uploaded, `dirty` cleared. Authority: 10D M35 ("every uploaded file was rendered from the photo's current
composition revision"), MC-28 §Cache key ("Stale outputs are never … uploaded"), 01 H4, plan C8(a) intent (Notes: "the exact M35 defect").
Before P30 the link kept its old `retouch_gen`, so the next publish replaced it; assumption 5 removed that self-healing.
**Correction.** At `prepare`, a link not `ready` whose state shows remote media may exist (`staged`, `uploaded`, `attached`) and whose
recorded `composition_rev` or `retouch_gen` is older than the staged file's is replaced like a ready one: old `external_id` →
`superseded_ids`, `revision += 1`, state `none`. The stale media then matches an `own` stem/superseded id and reconciliation removes it.
One test in `p30_outputs_in_export_and_publish.rs` of the probe's shape: after the resume, the gallery holds exactly one media for `a`,
whose `uploaded_sha` equals the r1 output's hash. **Files:** `crates/core/src/services/publish.rs`, optionally
`crates/core/src/mechanisms/publish.rs` (+ its unit test), `crates/testkit/tests/p30_outputs_in_export_and_publish.rs`.

**Notes → backlog (5):** stem keeps the folder key's case against 07A's "(lowercase)"; Normal↔Split change between `upload` and
`update` fails `update` with `UploadIncomplete` on every retry; first publish after migration 005 re-uploads photos whose composition
revision is ≥ 1; identical `set` on a default composition bumps the revision (P31); C4(c) "three retired rows" is impossible under the
plan's own key (plan lesson).
