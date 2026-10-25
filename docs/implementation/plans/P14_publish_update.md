---
phase: P14_publish_update
title: Publish update — attribute writes, title/tags/description, damage report, retouch replacement
difficulty: high
projection: no
depends_on: [P13_publish_runs]
---
# P14 — Publish update: attribute writes, title/tags/description, damage report, retouch replacement

## Goal
Finish the `update` step: write exactly the changed, valid, writable confirmed attribute values and verify them by
read-back (MC-13, M20), three-way-merge tags, write title and the manual description, upload the selected condition
photos as files and set the damage-report attribute (MC-14, M21), and replace retouched photos at publish (MC-21,
M28). Also the complete confirmation summary text (unreviewed note, removal count, create note). **Not in this
phase:** UI (P15); the AI description (post-MVP).

## Read first
master plan §6.5 (`item_links.tags_base`, `photo_links.revision/retouch_gen/superseded_ids`) · book `01`, `14`, `07`
§Publish steps 3–4, `07B` (all), `08` §Values and provenance · contracts `07A` MC-13 (write set, tags, title,
description), MC-14, MC-12 (own set includes `superseded_ids`); `07B` MC-21 (Replacement, Upload filename, Gallery,
Condition photos); `05A` MC-9 (equality) · evidence `shopify_admin.md` SV-1, SV-5.

## Files expected to change
`crates/core/src/services/publish.rs` (update step: attribute half; prepare: replacement pass), `crates/core/src/mechanisms/
{attribute_write.rs (new: write set, tags merge), reconcile.rs (own set with superseded ids)}`, Store trait + impl
(`tags_base`, link revision fields), `crates/testkit/src/fake_commerce.rs` (metafields, tags, files),
`crates/testkit/tests/p14_publish_update.rs` (new).

## Core mechanisms (unit tests allowed)
- `mechanisms::attribute_write::write_set(frozen, definitions, remote) -> Vec<(key, canonical)>` (MC-13 write set).
- `mechanisms::attribute_write::merge_tags(base, local, remote_now) -> Option<Vec<String>>` (MC-13: `(remote_now − (base − local)) ∪ (local − base)`, `None` if equal to `remote_now`).
- `mechanisms::publish::replacement_candidates(frozen_links, raw_retouch_gens) -> Vec<LinkId>` (MC-21).

## Ordered tasks
1. Update step, second half: `attributes_read` → `write_set` → batched `attributes_write` → read back and compare
   canonically (fail `attribute_verification{keys}` on any difference); title if different; description HTML when a
   manual description exists; tags via `merge_tags` with `item_links.tags_base` updated to the written set.
2. Damage report: selected condition photos (MC-14) uploaded as files (role `attribute:custom.damage_report`, same
   link state machine and adoption rule), then the attribute set to the file ids in condition order.
3. Replacement pass in `prepare` (before upload): for each frozen photo whose link (this item, this role) is `ready`
   and `raw_state.retouch_gen > link.retouch_gen`: `revision += 1`, `upload_state = none`, previous id pushed to
   `superseded_ids`, `link.retouch_gen = raw retouch_gen`; filename `<key>-<uid12>-r<n>.jpg`; adoption matches the
   current revision stem only; `own` for reconciliation includes `superseded_ids`.
3b. Identity write in `update` (05B MC-22): default-variant SKU/barcode diff against the remote, never clearing.
4. `publish_summary` completed (counts and notes); tests.

## Criteria (7 criteria, 20 rows; 0 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `update` | frozen `{years: "1950-1960s", country: "Sweden"}`; remote `{years: "1950-1960s", country: "Denmark", designer: "X"}` | fake `metafieldsSet` carries exactly `country` | M20 |
| C1(b) | `update` | key with no local value, remote has a value | never written, remote unchanged | M20 (never clears) |
| C1(c) | `update` | a local value with `valid = 0` | not written | MC-13 |
| C1(d) | `update` | a `reference` / `date_time` definition with a local value | not written (`writable = false`) | MC-13 (R-I17) |
| C1(e) | `update` | fake read-back returns a different value for `country` | item `failed` at `update`, `attribute_verification{keys: ["custom.country"]}` | MC-13 (read back) |
| C2(a) | `merge_tags` | base `[a,b]`, local `[a,b]`, remote `[a,b,c]` (admin added c) | `None` (no write); c kept | MC-13 (tags) |
| C2(b) | `merge_tags` | base `[a,b]`, local `[a]`, remote `[a,b,c]` | `Some([a,c])` (b removed locally, c kept) | MC-13 |
| C2(c) | `merge_tags` | base `[]`, local `[x]`, remote `[y]` | `Some([y,x])`; after the write `tags_base = [y,x]` | MC-13 |
| C3(a) | `update` | title local "Oak" vs remote "Oak" / "Teak" | no title write / `productUpdate` with `title: "Oak"` | MC-13 (title) |
| C3(c) | `update` | linked item, local SKU `K-2` / barcode none; remote `K-1` / `123` | `item_set_identity` sends `sku: K-2` only (barcode not cleared); read-back mismatch → `failed` at `update` | MC-22 (05B), M29 |
| C3(b) | `update` | manual description `"a\n\nb"` / none | `descriptionHtml = "<p>a</p><p>b</p>"` / no description field in the request | MC-13 (description) |
| C4(a) | `update` | condition [d, e, f], exclusions `[e]` | files uploaded for d and f (`role = attribute:custom.damage_report`), attribute value `["<fd>","<ff>"]` in that order | M21 |
| C4(b) | re-publish after reclassifying f to main and adding g | — | attribute `["<fd>","<fg>"]`; `ff` still exists in the fake's Files (not deleted) | M21, MC-14 |
| C4(c) | `update` | `frozen_condition` empty | attribute set to `[]`? **No**: no write when there is no local value and no selection — attribute untouched | M20 (never clears) — see Notes |
| C5(a) | `prepare` replacement | link for photo p `ready` with `retouch_gen 0`, `external_id m1`; raw `retouch_gen 1` | link `revision = 1`, `upload_state = none`, `superseded_ids = [m1]`, `retouch_gen = 1`; upload filename `<key>-<uid12>-r1.jpg` | M28 |
| C5(b) | `update` after C5(a) | p was main 02 | new media at gallery index 1; `m1` deleted (own); exactly one media for p | M28, M19 |
| C5(c) | replacement of a condition photo | — | new file id in the damage-report attribute; old file remains in Files | M28, MC-21 |
| C5(d) | `upload` adoption after `fail_after_effect(product_create_media)` on a revision-1 upload | shop has `…jpg` (old) and `…-r1.jpg` (new) | adopts `…-r1.jpg` only | MC-21 (current revision stem) |
| C6(a) | `publish_preview` → `publish_summary` | 3 main, 1 condition, 2 unreviewed, 1 announced removal, local item | text contains "Publish 4 images", "2 unreviewed photos will not be published", "1 photo will be removed from the gallery", "The product will be created on Shopify" (exact strings fixed in `publish_summary`) | MC-10 (summary = text shown), R19 |
| C7(a) | full run | — | step sequence recorded by `publish_progress` events is `prepare → upload → update → done` (no `describe`) | ch07 §Publish step 4 (MVP skip) via MC-11 |

## Delegated choices
Exact wording of the summary strings (fixed once in `publish_summary`, then asserted), batch size for file uploads,
whether tags are written in the same `productUpdate` as the title.

## Manual try-out
1. Fill a few Details fields, publish: in the Shopify admin the metafields show your values; a field you left empty
   that had an admin value is untouched; a tag added in the admin survives.
2. Untick one condition photo in the Damage report, publish: the metafield lists the others, in condition order.
3. Retouch a published photo in Capture One (heal a spot), wait for the preview to refresh: the session reads
   "Ready · Publish"; publish: the shop shows the new version in the same gallery position.

## Notes
- C4(c): with no selected condition photos and no stored `media_list` value there is nothing confirmed to write, so
  the attribute is untouched (MC-13 "keys with no local value are never written"). With a stored exclusion value and
  zero selected photos, `[]` **is** the confirmed value and is written. The row asserts the first case; the second is
  C4(a)'s rule with an empty list and is covered by `write_set` unit tests.
- Estimated size: ~6 files, ~1 400 lines.

## Review log
(empty)
