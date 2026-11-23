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
