# 07 — Session states and publishing

## Status (derived; complete order, first match wins)
1. `publishing` — a publish item for this session is in `prepare`, `upload`, `update` or `describe`.
2. `failed` — the latest publish item is `failed` and the session has never been published.
3. `published` — `published_at` set and not dirty.
4. `ready` — at least one `main` photo, and every non-deleted photo is reviewed and has its
   adjusted preview.
5. `in_progress` — otherwise (including an empty session).
List rows show, in this order of precedence: Publishing n%, Publish failed (+ Retry), Description
failed, Published, Ready · Publish, In progress (+ "n to review"). The Open filter is everything
except `published`.

## Dirty
Any photo change (add, classify, reorder, move, delete, undo, or a Capture One retouch of a
reviewed photo: 07B) sets `dirty`; a published session
therefore returns to `ready` or `in_progress`. Attribute edits alone do not set dirty (they are
sent on the next publish regardless).

## Completion without an integration (H6, M7)
With no integration configured the header action is **Export**. It writes the session's main and
condition photos through the provider's web recipe into
`<export folder>/<SKU>/<SKU>_<NN>_<main|condition>.jpg` in order, plus `item.json` with the
attribute values. The export is also step one of every publish.

## Publish (integration configured)
- Entry points: the session header Publish popover, "Ready · Publish" on a row, or batch
  "Review & publish N" → form queue → "Publish N confirmed".
- The popover is the **confirmation surface** (H1): main thumbnails in order, condition
  thumbnails, product-details fill count, the note "n unreviewed photos will not be published"
  (N2: unreviewed photos are excluded from every publish and export), the gallery-removal count
  (N1), and the create-item
  note for local items. Confirming writes `publish_runs` + `publish_items` with the exact summary.
- Steps per item, each a resumable job (H4, M4):
  1. `prepare` — create the item on the integration if `create_pending`; export files (above).
  2. `upload` — for each main photo, then each condition photo: skip if a `photo_links` row is
     already `ready`; otherwise stage, upload, attach, poll to ready. Progress "Uploading n of N".
  3. `update` — make the gallery exactly the session's Main list in order (N1 mirror: add
     missing, reorder, remove platform media the session does not own), set attribute values
     including the condition-photo attribute, verify by reading back.
  4. `describe` — post-MVP: agent writes the description. MVP: skipped unless a manual
     description exists, which is sent in `update`.
  5. `done` — `published_at`, `dirty = false`, notification row "Published · time".
- Publishing never blocks the workspace; rows show live progress.

## Failure and retry
A failed step marks the item `failed` with the reason; photos that uploaded keep their links.
Retry re-runs from the failed step only. No step deletes or replaces integration data except the
gallery removals announced on the confirmation popover (N1).
