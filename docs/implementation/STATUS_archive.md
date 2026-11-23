# STATUS — archived and shop-only items editable (book 10H)

Running log of the 10H track on branch `feature/agentic`. The main tracker `STATUS.md` is not edited by this track.

- **2026-11-23:** owner request: archived items show a broken picture in the sessions list and a read-only page; they
  must open in the normal workspace, be editable (rearrange, rotate, framing, Capture One) by downloading the
  pictures from Shopify, and be republished. Also: pages in the sessions list so a load spends no Shopify points, and
  pictures on search results.
  - Explorer map of the archive code (clearing, guards, list cover, archived view, search, publish media, photo
    pipeline, Shopify cost handling). Root cause of the "?": clearing deletes previews but the photo rows keep the
    paths, and the list never reads `archived_images`.
  - Plan presented to the owner. Cards: products from the other Mac get the same flow; editing from the published
    JPEG is fine; 25 sessions per page with page numbers.
  - Chapter `10H_archived_items_editable.md` RATIFIED (`7f839cc`, changelog A32; amends 10F's read-only rule and Q3).
  - Planner started (`pipeline-planner`): `master_plan_archive.md`, `phases_archive.txt`, plans `H…`.
  - **Plan written** (`f40a43b`): H1 points and remote record (projection), H2 list pictures, pages and search, H3 bring back (projection), H4 remote workspace (projection). 12 units, 54 criteria, 250 rows, 0 owner cards. Data-loss traps given rules: details-only publish of an archived item must not delete its Shopify pictures; a brought-back item is not cleared again before its next publish; clean-up ignores items with no local files. Capture One with JPEGs is unverified: H3 starts with a probe.
  - **Chain H1→H4 started** in the background (`scratchpad/archive_chain.sh`, log `scratchpad/archive_chain.log`).
  - **2026-11-23:** H1, H2, H3 APPROVED (narratives in `narratives/H1…H3`). H4 projection r0 amendments applied (`094d25c`); the H4 implementer stopped at 11:32 (usage limit) with **uncommitted partial edits** (14 modified files, new `commands/remote.rs` and `h4_remote_workspace.rs`), state NEEDS_ORCHESTRATOR "implementer state MISSING". Next: let a fresh implementer continue from the partial edits (or save them as a patch and discard), then `PSM_IMPLEMENTER=claude tools/pipeline/run_phase.sh phase H4_remote_workspace`; then the read-only live check on the real shop, release build, install, PR.
  - **2026-11-23 13:03: H4 APPROVED** (restarted clean; the stopped run's partial edits saved as `scratchpad/h4_partial.patch` and in "Session Manager old (2026-11-12)/h4_partial_2026-11-23.patch", then discarded). **All four milestones approved.**
  - **Live check on the real shop (read-only, orchestrator):** `MEDIA_READBACK` with `originalSource.fileSize` / `GenericFile.originalFileSize` accepted, no errors (14 gallery media, all with url and fileSize); `extensions.cost.throttleStatus` present (`currentlyAvailable` int, `maximumAvailable` float); `nodes(ids:)` returns url and size per known id and `null` for an unknown id (cost 3); the title search returns `featuredMedia` urls (20 of 20); a full-size download answers `200 image/jpeg`, first bytes `ff d8 ff`, ~570 KB. Capture One JPEG probe: PASS (H3, evidence VERIFY-JPEG). Not exercised live: a bring back and a republish on a real archived product (the owner's first use).
