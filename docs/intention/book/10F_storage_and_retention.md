# 10F — Storage and retention

**Status: DRAFT (owner decisions 2026-11-09; not planned, not implemented).** The owner: "we can leave the intention ready but we will not implement
it; after the current implementation we move to the installation on the work PC." The open questions at the end must be answered before planning.
Changelog entry 40. Evidence: P52 runs 1–2 (`evidence/sony_camera_lab.md`) and the orchestrator's measurements of 2026-11-09 (below).

## Why
Nothing in the app frees space today except masters after publish (10D MC-28). Measured on the dev Mac with the A7R V (2026-11-09):

| File | Size per photo | Who writes it | Kept today |
|---|---|---|---|
| RAW in the capture folder | ~47 MB | camera (FTP or Save-to-PC) | forever (H2) |
| Identical second copy in `Capture/.duplicates/` | ~47 MB | the second delivery path | forever (MC-33). **On the cable every photo has one:** the camera saves to the Mac over USB *and* its Auto FTP uploads the same file over Wi-Fi |
| Capture One's preview export in `Output/_session_manager/` | ~0.7 MB | Capture One (`PSM Preview` recipe, overwrite) | forever, although the app has copied it into its cache |
| App cache: camera preview, adjusted preview, thumbnail | ~2.8 MB | the app | forever (only the newest adjusted generation is kept) |
| Master TIFF | 78–360 MB | Capture One (`PSM Master`), only when publishing/exporting | deleted after publish (MC-28) |
| Output JPEG | ~1–2 MB | the app (MC-28) | kept (the deliverable when there is no integration) |

A 200-photo day ≈ 10 GB on Wi-Fi, ≈ 20 GB on the cable, almost all RAWs and duplicates. A disk filled up on 2026-11-08 (an export loop, fixed in P56).

## What the photographer gets (owner decisions 2026-11-09)
1. **Duplicates go after 24 hours.** A copy in `.duplicates/` whose SHA-256 equals the filed photo's is deleted 24 h after it was diverted. A same-name
   file with a different hash is never a duplicate (it is a new photo, MC-33). *Amends MC-33 "never deleted".*
2. **Preview exports are not kept twice.** Capture One's export in `Output/_session_manager/` is deleted as soon as the app has copied it into its
   cache. In the cache, a new adjustment replaces the previous preview (one generation per photo, as today).
3. **Published items are cleared after N days; Shopify becomes the archive.** A setting `storage.retention_days_after_publish` (default **4**,
   Settings → Storage). N days after an item was **published and not dirty**, the app deletes that item's local files: its RAWs, their Capture One
   sidecars/cache entries for those files, its duplicates, its masters, its outputs and its previews. From then on the app shows that item's photos from
   **Shopify** (the published images, downloaded on demand and cached small); "if I need to do something with the picture we download it from Shopify".
   *Amends H2 ("RAW files are never … deleted by Session Manager") for published items past retention.* With no commerce integration (H6) nothing is
   cleared by this rule.
4. **A storage budget with a removal order.** Settings → Storage shows free space and space used by each kind, and a **minimum free space** the
   photographer can raise or lower (default proposed **20 GB**). When free space falls below it, the app removes, in this order, until it is above the
   limit again: (1) duplicates already verified (even before 24 h), (2) preview exports already copied, (3) masters not needed by a running
   publish/export, (4) local files of published items past retention, then of published items not yet past retention, **oldest publish first**.
   It never removes anything of an item that is not published (or is dirty, publishing, or failed); if that is not enough, it warns in the header and
   on Health ("Disk almost full: N GB free; nothing more can be cleared safely").
5. **Self-repair of Capture One's recipes before use.** The 2026-11-08 TIFF previews came from the `PSM Preview` recipe having been changed by hand in
   Capture One (owner: "I manually changed the export to TIFF when I was experimenting"). The app repaired recipes only at start-up. It checks (and
   repairs) its recipes before each render batch, and treats a completed export in the wrong format as a recipe fault (repair + re-render once), never
   as a preview.
6. **Never slows the app.** All clearing runs as low-priority background work, in small batches, paused while a capture, an upload, a render or a publish
   is running.

## Rules
- Clearing deletes permanently (moving to the Trash frees no space); every deletion is logged with the file, its size and the rule that removed it.
- Before clearing an item (rule 3/4), the app verifies that Shopify still holds every published image of that item (read back media ids and URLs);
  if not, it clears nothing for that item and reports it.
- After clearing, the item is **read-only** in the app (no reorder, retouch, recomposition or republish of its photos); its attributes stay editable
  only where they do not need images (open question Q3).
- Capture One keeps its catalogue entries for cleared files; they show as missing there (open question Q4).

## Open questions (answer before planning)
- **Q1** Which photos of a published item are cleared: all (main, condition, hidden/"deleted", never reviewed), or only those that were published?
  Proposed: all — after publish, unpublished photos of that item have no use.
- **Q2** Should the app keep a per-item "keep files" pin that exempts it from rules 3–4? Proposed: yes.
- **Q3** After clearing, is anything editable that would need a re-upload (e.g. reorder on Shopify)? Proposed: no; reorder/retouch would need a new
  photo session.
- **Q4** Remove the cleared images from the Capture One session too (via its dictionary, H5), or leave them as missing entries? Proposed: leave them.
- **Q5** Default minimum free space: 20 GB?

## Invariant (proposed M41)
No file of an item that is not published-and-clean is ever deleted by storage rules, and no RAW is deleted before Shopify has been verified to hold every
published image of its item.
