# 09 — Integrations

## Shape
Commerce and publishing is one of three integration families; camera and editing providers are
chapter 10. Same rule: `core → service → interface → adapter`, adapters declare capabilities,
nothing platform-specific in core. One commerce adapter (Shopify) in the MVP. No plugin loader,
no registry beyond a settings key.

## Capabilities (declared by the adapter; the UI hides what is absent)
| Capability | Core operation |
|---|---|
| `item.search` | find items by SKU, barcode or title |
| `item.get` | item identity + existing media list with positions |
| `item.create` | create a draft item from SKU, barcode, title |
| `attributes.schema` | definitions translated to core types (chapter 08) |
| `attributes.read` | current values for an item |
| `attributes.write` | write values |
| `media.publish` | upload, attach, order gallery photos |
| `media.attribute_publish` | upload photos for a `media_list` attribute |
| `description.publish` | write the description |
A session can complete without any of them (H6).

## Shopify adapter (MVP)
Admin GraphQL API, version pinned in settings (2026-10 unless the store's evidence says
otherwise). Mapping:
- Item ↔ Product (SKU and barcode on the default variant); `item.create` makes a DRAFT product.
- `attributes.schema` ← product metafield definitions; type map in `evidence/design_handoff.md`.
  `metaobject_reference` and `product_reference` become `reference` (read-only in the form).
- `media.publish`: `stagedUploadsCreate` → upload → `productUpdate` media → poll `READY` →
  `productReorderMedia` → poll job.
- `media.attribute_publish`: `fileCreate` → poll → `metafieldsSet` on the `media_list` attribute.
  The condition photos attribute for the first customer is `custom.damage_report`
  (`list.file_reference`, images only), per the shop's definitions on 2026-10-24.
- Every write is preceded by a read that checks whether it already happened: by stored external
  ID, then by filename and hash (H4). Tests never touch the live store (fixtures captured once
  from the dev store).

## What stays out of core
Shopify IDs (in `item_links` / `photo_links`), metafield namespaces, GraphQL, media states.
