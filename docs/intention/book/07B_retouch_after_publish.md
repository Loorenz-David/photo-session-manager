# 07B — Retouch after publishing (owner decision I1)

Owner decision 2026-10-25 (chapter 15 R30, chapter 16 entry 15): **a Capture One retouch of a
published photo makes the session need publishing again, and the next publish replaces that
photo on Shopify in the same gallery place.** What the live preview shows is what goes out.
This chapter is the authority on the rule; 05A MC-6, 07A MC-11/MC-12/MC-14 and 10A MC-17 point
here.

## MC-21 — Retouch generation, dirty, and replacement at publish

**Retouch event.** Per RAW, `retouch_gen` (integer, starts 0). An adjusted event for a known RAW
(10A MC-17, same matching) is a **retouch** unless it falls inside the RAW's own-write window,
`now ≤ last_own_write + 30 s` (10A MC-19), which is our own colour-tag write echoing back. A
retouch does, in one transaction: `retouch_gen += 1`, and `change_seq += 1` (05A MC-6) on every
session that holds a non-deleted `main` or `condition` row of that RAW. The MC-17 sidecar
fallback (startup and focus) produces adjusted events under the same rule, so a retouch made
inside an own-write window is caught at the next focus or startup.

**Dirty.** A retouch is a photo change for MC-6: a published session holding the photo becomes
dirty and its row returns to Ready · Publish (or In progress). A retouch while a publish runs
keeps the session dirty after it (R-I13 unchanged).

**Recording what was sent.** Every `photo_links` row carries `revision` (integer, starts 0) and
`retouch_gen` (the RAW's `retouch_gen` read when that link's export render was requested, as
MC-17 records `g`).

**Replacement (prepare step, before upload).** For each frozen photo whose link for this item
and role is `ready` and whose RAW `retouch_gen` is greater than the link's: in one transaction,
`revision += 1`, `state = none`, and the previous media or file id is kept as
`superseded_ids` on the link. The upload step then treats the link as new (07A MC-11).

**Upload filename.** Revision 0: `<folder_key>-<first 12 hex of uid>.jpg` (unchanged). Revision
n ≥ 1: `<folder_key>-<first 12 hex of uid>-r<n>.jpg`. Adoption after a crash (MC-11) matches the
stem of the link's **current** revision only, so an old revision is never adopted as the new one.

**Gallery.** The new media id takes the photo's place in `D` (07A MC-12). The superseded media
is `own` (its id is in the link's `superseded_ids`, and its filename matches an upload stem of
this item), so step 3 deletes it and step 4 puts the new one at the same position.

**Condition photos.** The damage-report attribute is set to the current links' file ids (07A
MC-14), so it points at the new file. The superseded file stays in the shop's Files, unreferenced,
like files of deselected photos.

**Invariant (M28).** After a retouch of a photo in a published session, the session is dirty;
after the next publish completes, the product shows exactly one media for that photo, at the
same gallery position, rendered from a request made after the retouch, and the damage-report
attribute references only current revisions. A colour-tag write by the app alone never makes a
session dirty.
