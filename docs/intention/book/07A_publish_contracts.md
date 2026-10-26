# 07A — Mechanism contracts: publish (confirmation, resume, gallery, attributes)

Written by the mechanism inventory (2026-10-25, round 1). Conventions as in
`05A_mechanism_contracts.md`. Chapter 07 is the authority for the publish semantics; chapter 09
for the integration interface. **Shopify shapes in this chapter are grounding-owed** (charter rule
17): the Shopify evidence doc does not exist yet. Every Shopify-owned shape below carries a
`SHOPIFY-VERIFY-n` id (list at the end); the phase that builds the adapter grounds each one
against the pinned API version before its criteria are final.

---

## MC-10 — Confirmation and the frozen run

Confirming (popover, or "Publish N confirmed") writes, in **one** SQLite transaction, one
`publish_runs` row (`confirmed_at`, `confirmed_summary` = the exact text shown) and one
`publish_items` row per session, holding everything the run may send, frozen:
- `create_item`, `title` (as today);
- `frozen_main`: photo ids of the session's `main` photos in position order;
- `frozen_condition`: photo ids of the `condition` photos in position order, minus the
  damage-report exclusions (MC-14);
- `frozen_attributes`: the canonical local value (05A MC-9) of every attribute that has one;
- `announced_removals`: the external media ids of gallery media the session does not own, as
  read by `item.get` when the confirmation surface was built; their count is the
  gallery-removal number shown (N1);
- `confirmed_change_seq`: the session's `change_seq` (05A MC-6).

Edits after confirmation (attributes, order) go out on the **next** publish **(ratify R-I14:
chapter 07's "set attribute values" sends the confirmed values, not values edited during the
run)**. Exception, safety
first **(ratify R-I6)**: at every step, a frozen photo that is now `deleted`, or no longer in
this session, is skipped as if absent from the frozen lists. No deleted photo ever leaves the
Mac (M3), even if the deletion happened after the confirmation.

**Mutation token.** Every adapter method that mutates the integration takes a `ConfirmedRun`
value (run id, `confirmed_at`). Only the publish service can build one, and only from a
persisted run with `confirmed_at` set. The adapter logs the run id with every mutation. There is
no other mutation path, so this is checked by the type system as well as by tests (charter
rule 11).

**Invariant (M17).** Every integration mutation carries a run id whose run was committed with
`confirmed_at` before the call; the photos, order, removals and attribute values a run sends
are the frozen ones, minus photos deleted or moved away since.

---

## MC-11 — Step machine, resume, idempotency keys

**Persistence.** `publish_items.step` is committed **before** the step's work starts and advanced
only after that step's own read-back succeeds. A failure stores `step = failed`,
`failed_step = <step>` and the reason. Retry sets `step = failed_step` and re-runs from there
(chapter 07). At startup every `jobs` row in `running` returns to `queued`, and every item whose
step is active is re-queued at its step. Steps may run more than once (at-least-once); the keys
below make that harmless. A remote call that times out has an **unknown** outcome: the next
attempt reads before it writes, and it never assumes the call failed.

**prepare.** If `create_pending`: first `item.search` by exact SKU (the adapter compares the
default variant SKU byte for byte after NFC). 0 matches → `item.create`, then `item_links` is
written before any other call; 1 match → adopt it into `item_links`; more than 1 → fail with
"several products carry SKU <sku>". Then re-verify RAW hashes (05A MC-3) and render the export
(05A MC-8, 10A MC-16).
*Grounding (fold-back 2026-10-26, ch16 entry 17):* this rule assumes `item.create` leaves a
product that carries the SKU and that a search right after finds it. SHOPIFY-VERIFY-7 and the
SHOPIFY-VERIFY-6 lag check test both. If create is not atomic or search lags, prepare needs a
recovery lookup that does not depend on the search index. The orchestrator amends this contract
before the adapter phase continues.

**photo_links key (ratify R-I4).** `(photo_id, integration, item_id, role)`, where `item_id` is
the item the link was created for. A link created for item A never satisfies item B: a photo
moved to another session is uploaded again for the new product, and its old media count as
"own" media of item A (MC-12), to be removed on A's next publish.

**Photo uid and upload filename.** Each `photos` row gets `uid` (UUID v4) at creation; it is
never reused. The filename sent with every upload (staged upload, media create, file create) is
`<folder_key>-<first 12 hex of uid>.jpg` (lowercase). The local export file keeps its
position-based name (05A MC-8); only the uploaded copy's name differs. Position-based names are
never used to identify remote media: they change with every reorder. A retouched photo is
re-uploaded under a revision suffix and replaces its media in place (07B MC-21).

**upload** (for each frozen main photo, then each frozen condition photo; role `gallery` or
`attribute:<key>`): state `none → exported → staged → uploaded → attached → ready`, each
transition committed after the remote effect is confirmed. A link already `ready` is skipped.
Before creating media or a file for a link that is not `attached` or `ready`, the adapter lists
the product's media (or the shop's files) and **adopts** an existing one whose filename matches
the photo's upload stem (Shopify may add a uniquifying suffix: SHOPIFY-VERIFY-2), setting
`attached`, then polls it to `ready`. This is the crash-recovery path between remote create and
local record. More than one match: adopt the first in gallery order; the others are own media and
are removed in `update`. **(ratify R-I10)** Chapter 09's "by filename and hash" is implemented as
by-filename only on the remote side; Shopify is not known to expose a content hash
(SHOPIFY-VERIFY-3). The SHA-256 of each uploaded export file is stored in the link
(`export_sha256`) for local checks.

**update** → MC-12 then MC-13. **describe** → skipped in the MVP. **done** → `published_at` =
finished time, `published_seq = confirmed_change_seq`, notification row.

**Invariant (M18).** Killing the process at any instant during a publish item, then resuming,
ends with the same product, media set, media order, file set and attribute values as an
uninterrupted run, with no duplicate product, media or file (M4 restated per key).

---

## MC-12 — Gallery reconciliation (update step, first half)

Inputs, read at step time: `D` = the media ids of the `ready` gallery links (this item) of the
filtered `frozen_main`, in order; `current` = the product's gallery media ids in order;
`own` = media ids of any gallery link for this item (any photo, including photos since moved,
deleted or reclassified, and the link's `superseded_ids`: 07B MC-21) plus media whose filename matches an upload stem of this item;
`announced` = `announced_removals`.

1. Every element of `D` exists and is `READY` (else the upload step was not complete: fail).
2. `foreign` = `current − D − own − announced`. If non-empty, **fail before any deletion** with
   "the product gained n photos after you confirmed; publish again to review the removal"
   **(ratify R-I5: H1 wins over the N1 mirror for media nobody confirmed removing)**.
3. Delete `(current − D) ∩ (own ∪ announced)`. A not-found reply counts as deleted.
4. Reorder: one reorder call with a move for **every** id in `D`, `newPosition` = index in `D`
   (0-based, i.e. main position − 1; SHOPIFY-VERIFY-4), then poll its job to completion.
5. Read back: the gallery id list must equal `D` exactly, else fail "gallery verification".

**Invariant (M19).** After `update` succeeds, the product gallery is exactly `D` in order (cover
= main 01); no media outside `own ∪ announced` was deleted; a condition photo is never in the
gallery.

---

## MC-13 — Attribute writes (update step, second half) and the Shopify type map

*Writability (fold-back 2026-10-26, ch16 entry 17):* a definition is writable when its type is
writable in the type map below **and** its definition `access` permits writes by this app
(SHOPIFY-VERIFY-1 records access). Other apps' namespaces may be read-only for our token.

**Write set (ratify R-I8).** Read the item's current values (`attributes.read`) at the start of
the write. Write exactly the keys in `frozen_attributes` whose definition is writable, whose
local value is valid, and whose canonical value differs from the canonical remote value. Keys
with no local value are never written; nothing on the platform is ever cleared. Writes are
batched (SHOPIFY-VERIFY-5 gives the batch limit), then read back and compared canonically; any
difference fails the step and names the keys.

**Product tags** (a product field, not a metafield) use a three-way merge so tags added in the
Shopify admin are not lost: with `base` = the app's own tag list as of the last successful
`update` of this product (stored per link; empty before the first), write
`(remote_now − (base − local)) ∪ (local − base)`, only if that differs from `remote_now`. After
every successful `update` the base becomes the current local list, whether or not a write
happened. So a tag added in the admin is never removed, and a tag removed in the app is removed
from the shop. (Corrected 2026-10-26, R33: the earlier base "the remote tags when last read"
deleted admin tags on the second publish.)
**Title** is written when it differs. **Description** (MVP: only a manual one) is sent as HTML:
each blank-line-separated paragraph becomes `<p>…</p>` with `& < > "` escaped.

**Type map** (definition → core type → value sent; value shapes SHOPIFY-VERIFY-1):
| Shopify type | Core type | Value sent (string) |
|---|---|---|
| `single_line_text_field` with choices | `choice` | the choice string |
| `single_line_text_field` | `text` | the string |
| `list.single_line_text_field` with choices / without | `multi_choice` / `tags` | JSON array of strings |
| `multi_line_text_field` | `textarea` | the string |
| `rich_text_field` | `rich_text` | JSON `root` → `paragraph` → `text` nodes; a remote value with any other node or mark makes the field read-only |
| `number_integer` | `integer` | decimal digits |
| `number_decimal` | `decimal` | the decimal string |
| `dimension`, `weight`, `volume` | `dimension` / `weight` / `dimension` | JSON `{"value": number, "unit": code}` |
| `boolean` | `boolean` | `true` / `false` |
| `date` | `date` | `YYYY-MM-DD` |
| `date_time` | `reference` (read-only in the MVP; the shop has none) **(ratify R-I17)** | never sent |
| `color` | `color` | `#rrggbb` |
| `rating` | `rating` | JSON `{"value", "scale_min", "scale_max"}` as decimal strings |
| `money` | `money` | JSON `{"amount", "currency_code"}` |
| `url` | `url` | the URL |
| `list.file_reference` limited to images, **only** the definition named by `condition_photos_key` | `media_list` | MC-14, 09A MC-24 |
| `file_reference`, `metaobject_reference`, `product_reference`, their `list.` forms, anything else | `reference` | never sent |

**Invariant (M20).** A publish writes only keys whose confirmed local value differs from the
platform value; it never clears or overwrites a key the photographer has no local value for; a
round trip of any supported type through read, canonical form and write is the identity.

---

## MC-14 — Condition photos and the damage-report attribute

The `media_list` value stores **exclusions**, not selections: `{"excluded": [photo ids]}`. The
selected set is the session's condition photos in condition order minus the excluded ids, so a
condition photo added later is selected by default (chapter 08 "all selected by default"). Ids
that are no longer condition photos are ignored. **(ratify R-I9)** M3's "every condition photo
is in the condition attribute" is evaluated over this selected set; with no exclusions they are
equal.

Publishing: each selected photo is uploaded as a file (role `attribute:<key>`, where `<key>` is the
configured `condition_photos_key`, 09A MC-24; MC-11 rules),
then the attribute is set to the JSON array of their file ids in order. Files for photos no
longer selected stay in the shop's Files (not deleted, not on the product).

**Invariant (M21).** After `update`, the condition attribute lists exactly the selected condition
photos in condition order; an unexcluded condition photo is never missing from it.

---

## Grounding owed (Shopify, before the adapter phase's criteria are final)
- **SHOPIFY-VERIFY-0** client credentials token request and response fields (token, lifetime, scope), the
  wrong-secret reply and a GraphQL 401 (09A MC-23; added 2026-10-26, ch16 entry 18).
- **SHOPIFY-VERIFY-1** metafield value string formats per type above, for the pinned version.
- **SHOPIFY-VERIFY-2** filename of a media image and of a file after upload: where it is
  readable, and the exact uniquifier added on a name collision.
- **SHOPIFY-VERIFY-3** whether any content hash or checksum is exposed for media or files.
- **SHOPIFY-VERIFY-4** `productReorderMedia` position base and job polling shape.
- **SHOPIFY-VERIFY-5** `metafieldsSet` batch limit and partial-failure shape.
- **SHOPIFY-VERIFY-6** search by exact SKU: query syntax, whether matching is case-sensitive,
  and whether a product created seconds earlier is found (immediately, +5 s, +30 s).
- **SHOPIFY-VERIFY-7** the create mutation that sets status DRAFT plus the default variant's SKU
  and barcode in one call, if one exists, and the mutation that writes SKU/barcode later
  (05B MC-22) with its read-back.
- SHOPIFY-VERIFY-1 also records each definition's `access`; SHOPIFY-VERIFY-4 also records the
  media-delete request and its not-found reply (MC-12 step 3).
(SHOPIFY-VERIFY-6 extension, -7 and the access/delete additions: fold-back 2026-10-26 from the
adapter phase's projection, ch16 entry 17.)
