# 09A — Shopify credentials and the condition-photos field (owner answers)

Owner decisions 2026-10-26 (chapter 15 R32, chapter 16 entry 18): **the app connects to Shopify
as a Dev Dashboard app with a client ID and client secret, not with a pasted access token; the
store is a settings value, and the test store is replaced by the real store by changing settings
only; the condition photos go into one product metafield whose `namespace.key` is a setting.**
This chapter is the authority on both; chapter 09 (Shopify adapter), 07A MC-13/MC-14 and
chapter 12 (health) point here.

## MC-23 — Shopify credentials and the access token

**Settings.** The `integration` block of `settings.json` (file mode 0600) holds
`{"kind":"shopify","shop_domain","api_version","client_id","client_secret","condition_photos_key"}`.
`shop_domain` is the store's `*.myshopify.com` domain. There is no stored access token.

**Access token.** The adapter obtains the Admin API access token with Shopify's client
credentials grant: `POST https://<shop_domain>/admin/oauth/access_token` with
`grant_type=client_credentials`, `client_id`, `client_secret`. The response carries the token and
its lifetime (reported as about 24 hours; the grounding probe records the actual fields). The
token is held **in memory only**: never written to disk, the database, logs, fixtures, commits or
handoffs, and never shown. It is fetched on first use, reused until shortly before it expires,
then fetched again. A request answered `401` fetches a new token and is retried **once**; a
second `401` is an error. The client secret is never sent anywhere except this token request.

**Errors shown to the photographer** (health screen and the failing operation's message):
- the grant is refused with `shop_not_permitted` → "This Shopify store is not in the app's
  organization";
- `app_not_installed` → "The Session Manager app is not installed on this store";
- invalid client ID or secret, or any other refusal of the grant → "Shopify credentials rejected".
Each leaves the app in its no-integration behaviour for publishing (export stays available, H6)
until the next successful token fetch.

**Store switch.** Nothing store-specific is built into the app: definitions, field keys,
products and media are always read from the configured store. Moving from the test store to the
real store is a settings change (`shop_domain`, `client_id`, `client_secret`); the next schema
refresh reads the new store's definitions. Links (`item_links`, `photo_links`) name the
integration, not the store; switching stores with links in place is not an MVP flow (the owner
switches before real publishing starts).

## MC-24 — The condition-photos field

**Setting.** `integration.condition_photos_key` is a `namespace.key` string, initial value
`custom.damage_report`, set by hand in `settings.json` for now (no settings UI in the MVP beyond
what chapter 12 lists).

**Mapping.** In `attributes.schema` the adapter maps **exactly** the definition whose
`namespace.key` equals `condition_photos_key` to the core type `media_list`, group `condition`.
That definition must be of Shopify type `list.file_reference` accepting images. Every other
file-list or file definition is a read-only `reference` (MC-13 table). Core and the form find
the condition-photos field as the one `media_list` definition; nothing in core names the key.

**Misconfiguration.** If no definition has that key, or its type is not `list.file_reference`,
the schema refresh still succeeds without a `media_list` field. Health shows "Condition photos
field `<key>` not found in the shop" (or "… is not a file list"), and the damage report is not
published: the `update` step skips the condition-photo writes and the run's notification says
so. Gallery photos and other attributes publish normally.

**Publishing** follows 07A MC-14 unchanged, with the role `attribute:<condition_photos_key>`.

**Invariant (M30).** With the integration configured, at most one integration definition has
core type `media_list`, and its key equals `condition_photos_key`; condition photos are written
only to that metafield; the access token and client secret never appear outside memory and the
settings file.
