# 05B — SKU generation and SKU change (owner card 2)

Owner decision 2026-10-25 (chapter 15 R31, chapter 16 entry 16): **a new local item's SKU is
proposed by the app from a prefix and is editable before confirming; an item's SKU stays
editable after photos exist.** This chapter is the authority on both rules; chapter 05 (items),
05A MC-2 and MC-8, and 07A MC-11/MC-13 point here.

## MC-22 — SKU proposal, the stored folder key, and SKU change

**Proposal.** Setting `sku_prefix` (string, set once in Settings; empty or absent = no
proposal). In New session's "Create '…'" row the SKU box is pre-filled with
`<prefix>-<NNNN>`, where `NNNN` = 1 + the largest integer `n` among local items whose SKU matches
`^<prefix>-(\d+)$` exactly (case-sensitive), or 1 if none, zero-padded to at least 4 digits. The
box stays editable; the SKU sent is what the box holds at confirmation. When an integration is
configured and reachable, a proposed or typed SKU that `item.search` finds in the shop is shown
as "<SKU> is already in the shop" and the create action is disabled until it changes (the
proposal advances to the next free number). No prefix → the box starts empty and is required.

**Stored folder key.** `items.folder_key` is computed from the SKU (MC-8) at creation and
**follows the SKU while the item has no photo row in any of its sessions**. From the first photo
row on, it is **frozen**: it names the capture folder (MC-2), and that folder is never renamed
(captures already in it keep their paths; MC-1 identity is unchanged). Session attribution (MC-2
rule 2) and upload filenames (07A MC-11) use the stored `items.folder_key`, never a key
recomputed from the current SKU.

**SKU change.** An item's SKU may change at any time, including after photos and after
publishing. The new SKU must be unique and its `folder_key`, ignoring case, must not equal
another item's stored folder key or the folder key of another item's current SKU (R-I11 refusal
message). With an integration configured and reachable, a SKU found on another shop product is
refused ("<SKU> is already in the shop").

**Export.** The export folder and file names (MC-8) use `folder_key(current SKU)`, and
`item.json` carries the current SKU. An export under a previous SKU stays where it is, untouched.

**Publish.** `prepare`'s search by SKU (07A MC-11) uses the current SKU. For a linked item, the
`update` step writes the default variant's SKU and barcode when they differ from the remote
values, and reads them back (diff write, as 07A MC-13: never clears a value the app does not
hold). An identity edit does not make the session dirty; it goes out on the next publish, like
an attribute edit.

**Invariant (M29).** After any sequence of SKU edits, every capture of an item lands in, and is
attributed from, the one folder named by its stored key; the export folder and `item.json` use
the current SKU; after the next publish the shop product's variant SKU equals the current SKU.
No proposal is ever confirmed while the shop holds that SKU on another product (when reachable).
