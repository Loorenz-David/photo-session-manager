# 02 — Product direction

Session Manager is a photography-workflow desktop application for professional photographers.
It is developed and tested inside a vintage-furniture company, and is intended to be offered later
to auction houses, retailers and other organisations with structured photography needs.

**Principle: MVP scope, production-oriented architecture.** The MVP implements only what the first
customer needs, on boundaries that let the following be added without restructuring:
- more platform integrations (auction systems, other e-commerce, inventory, custom APIs);
- more imaging providers (today Capture One; later other tether or edit tools);
- the AI research agent that fills item attributes;
- commercial packaging: signed installer, updates, licensing.

**What the MVP does not build:** a plugin ecosystem, licensing or subscriptions, auto-update,
hypothetical adapters, multi-user, cloud sync.

**Vocabulary (core terms, used in code and docs).**

| Core term | Meaning | Shopify word it replaces |
|---|---|---|
| Item | the thing being photographed; has an SKU, optional barcode, a type and attributes | Product |
| Session | one item's photography job: its photos, order, groups and item data | — |
| Photo | one captured or imported RAW inside a session, with its previews | Image / Media |
| Attribute | a typed field on an item, defined by a schema | Metafield |
| Integration | an external platform connected through an adapter with declared capabilities | the Shopify app |
| Publish | sending a completed session to an integration | Publish to Shopify |
| Imaging provider | the tether and editing tool behind capture and adjustment | Capture One |

"Capture One session" is always written **Capture One document** in this book to avoid a clash
with our Session.
