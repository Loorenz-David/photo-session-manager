---
plan: archive
role: plan
round: 1
state: DONE
date: 2026-11-23
actor: Claude (pipeline-planner, implementation-planner doctrine; orchestrated)
---
# Archived and shop-only items editable — plans for H1–H4 (book 10H)

**Summary.** A short master plan (`master_plan_archive.md`) whose §5 resolves 10H's five contracts (MC-H1 row picture,
MC-H2 pages and search pictures, MC-H3 remote items in the workspace, MC-H4 bring back, MC-H5 the call budget) to
code-level rules against the code at `7f839cc`, four milestone plans in the units format of flow §10, and the phase
list. Gate: 10H's status header reads **RATIFIED** (owner, 2026-11-23, changelog A32). The mechanism-inventory gate is
replaced by §5 at the orchestrator's instruction (recorded as a process delta, as for the writer and conversations);
10H §9 (M-H1…M-H5) is the measurement ledger the rows trace to. Every cited existing path resolves (checked by
script; the only unresolved paths are the files these milestones create). The explorer's code facts were re-read in
the code; where a line had moved or a fact needed a correction it is in "Points" below. The Shopify shapes were read
from shopify.dev's Admin GraphQL 2026-10 pages (documentation search only; no store was called) and from the
repository's captured P12 fixtures.

Counts (computed from the plan files by script): 4 milestones, 12 units, 54 criteria, 250 rows; every unit ≤ 8 criteria.
| Milestone | Units | Criteria | Rows | App files | Projection |
|---|---|---|---|---|---|
| H1_points_and_remote_record | 2 | 12 | 55 | yes | **yes** |
| H2_list_pictures_pages_search | 4 | 14 | 67 | yes | no |
| H3_bring_back | 3 | 14 | 62 | yes | **yes** |
| H4_remote_workspace | 3 | 14 | 66 | yes | **yes** |

Ledger coverage: M-H1 (H1·C10(e), H2·C5(a), H3·C8(a), H4·C1(h), H4·C6) · M-H2 (H3·C6, H3·C8(b), H4·C2(f)) · M-H3
(H3·C10–C13) · M-H4 (H2·C1, H2·C10(b), H4·C3, H4·C5(a), H4·C10(c)) · M-H5 (H2·C5(b), H3·C8(c)). No ledger entry is
unserved; no row traces to nothing.

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner. 10H and its three answered cards determine every behaviour the plans build. The choices
planning had to make where the chapter is silent are provisional, listed in master plan §6 and carried in plain words
by each plan's "Points recorded by planning" for the narratives; the ones most worth a second look are points 1–6
below.

## Write perimeter (this session)
Documents only, all new: `docs/implementation/master_plan_archive.md`, `docs/implementation/phases_archive.txt`,
`docs/implementation/plans/{H1_points_and_remote_record.md, H2_list_pictures_pages_search.md, H3_bring_back.md,
H4_remote_workspace.md}`, this handoff. No code, no migration, no tool-recorded state (the repo has no architecture
graph). Not touched: `STATUS.md`, `STATUS_archive.md`, book 10H, any other plan or prompt.

## Points the orchestrator should know
1. **A details-only publish needed a contract the chapter only names** (10H §6 "Publish of details only"). Read in
   the code: a publish of an archived item would stage an export of files that are gone; and with empty photo lists
   `update` would reconcile the gallery against nothing and **delete every picture** (all are the app's own,
   `publish.rs:1733-1804`), and write an **empty damage-report list** (`publish.rs:1820-1842`). MC-H3.6 therefore
   persists `publish_items.details_only` at confirmation and a run skips both blocks. Projection on H4 should attack
   exactly this.
2. **Retention had two silent traps** (MC-H3.8, MC-H4.8): a shop-only item after a details-only publish is "published
   and clean" and would be "cleared" with zero verified images, wiping its picture record; and an archived item
   brought back and not yet edited is "published and clean and long past retention", so the next scan would clear it
   again within the hour. Both are closed in `eligibility` (H1·C11) with service rows in H3·C14 and H4·C8.
3. **Opening a remote item must not move the capture folder, and the first-open read is background work** (10H §8
   lists it as such). So the gate is in two places (MC-H3.4): the backend's `activate` does nothing for a remote
   session, and the frontend activates only once the state is known to be local. **Provisional choice:** when that
   first read fails (offline), the session activates as it does today and says so; the alternative sends new photos to
   another product silently.
4. **"Shop-only" is defined tightly** (MC-H3.1): an adopted item (`origin = linked`) with a shop link whose session
   has no photo row. Assigning Unassigned photos to it, or photos arriving in its folder, make it an ordinary session
   exactly as today (the publish then announces the removal of the shop's pictures, as it does now). 10H does not ask
   for more; say so if the owner expects those flows to bring the shop pictures back first.
5. **File names on a second bring back** (10H names `<SKU>-main-01.jpg`): `photos` has `UNIQUE(session_id, raw_path)`
   and the earlier rows are kept, so a product brought back, republished, cleared and brought back again cannot reuse
   the names. MC-H4.2 adds `-b2` to the whole set. The name uses the folder key (the SKU made safe for a file name).
6. **"About X MB"** needs sizes the 10F record does not have. From H1 on the readback asks Shopify for them
   (`originalSource.fileSize`); items cleared before H1 use 1.5 MB per photo (10F's measured output size) until a
   Refresh from Shopify. No extra request is made for the prompt (10H §8 lists the allowed calls).
7. **Row label and status of remote rows** (MC-H3.7): both kinds show "Archived in Shopify" unless their publish is
   running, held, failed or cancelled; a shop-only item's counts come from its Shopify pictures, so a never-published
   adopted product is "Ready" under Open (today it is "In progress, 0").
8. **No `implementer:` override.** The task allowed Fable for frontend-heavy phases; `docs/process/flow.md` log v1.3
   (2026-11-08) and `docs/guide/03_rules_and_principles.md` forbid it and are the process authority (same decision
   as the conversations plan). If the owner lifts it: H2 units 3–4 and H4 unit 3 are the frontend parts, but a
   milestone has one implementer.
9. **Explorer facts, corrected or sharpened:** the CDN download's failure today is the string `download: HTTP <n>`
   (`transport.rs:202`), so "404/403" needed a typed shape: H1 changes the transport seam to return the status.
   `adopt` creates no session (the dialog calls `createSession` after it). The search documents already select
   `featuredMedia`, but **every captured search fixture has `featuredMedia: null`** (the probe product had no
   picture), so the non-null form is schema-grounded only. The captured fixtures **do** carry `extensions.cost`
   (`sv4_job_poll_done.json`), so the budget shape is captured, with mixed integer and float numbers.
   `MediaImage.originalSource.url` is "valid only for a short period" (shopify.dev): the stored URL must stay
   `image.url`.
10. **A migration hazard every `ALTER TABLE … ADD COLUMN` now carries:** `p27_retire_shape_on_arrival::c4a_…` drops
    each added column before it rewinds `user_version`; a new column without its `DROP COLUMN` line there fails that
    test with "duplicate column name". Both migrations (020 in H1, 021 in H4) name it as a task.
11. **Existing tests that must change** (named in the plans, with how): `p62_…::c3a_…` (the view no longer downloads;
    files appear after the background step, H4·U1), `p62_…::c3b_…` (a publish confirmation of a cleared item is now
    accepted as details-only, H4·U2), the Sessions-page UI tests (their list mock becomes the paged call, H2·U3),
    `p62_archived_ui.test.tsx` (removed with `ArchivedView`, H4·U3), and the `App`-level UI tests gain a remote mock
    answering `local` (H4·U3). No other existing assertion may change.
12. **H3 has no screen and H1 has no screen.** The owner sees something after H2 (list, pages, search) and after H4
    (the workspace and bring back). The narratives of H1 and H3 say so.
13. **Commit trailer.** The task asked for `Co-Authored-By: Claude Opus 5.5`; this session runs on Fable 5.2, and the
    session's standing attribution rule names that model, so the commit carries `Co-Authored-By: Claude Fable 5.2`.
    Amend the trailer if the project wants the role's nominal model there instead.

## Risks
- **Capture One and JPEGs is unverified.** Nothing in the evidence doc says the dictionary lists, finds and processes
  a `.jpg` in a capture folder like a RAW, or when it writes a settings file for one. H3·U1 task 0 is a probe that
  records VERIFY-JPEG (or UNPROBED when Capture One is not open). If the probe says a JPEG cannot be processed with
  the `PSM Master` recipe, bring back still works but a changed photo could not be re-rendered: that would be an
  owner card at that point, not before.
- **All new Shopify shapes are schema-grounded, not captured** (`nodes(ids:)`, the sizes, the non-null
  `featuredMedia`). Projection on H1 should re-read the fixtures against shopify.dev; the orchestrator's live check
  after H4 is the first real contact. Whether the CDN really answers 403/404 for a replaced file is 10H's own
  assumption.
- **A publish after bring back still renders every photo** (a master and an output each, through Capture One), also
  the untouched ones; it uploads only the changed. Correct, but a 20-photo product costs 20 renders to change one.
  Not optimised here (the plans forbid it); a candidate for the backlog.
- **H3·U2 is the largest unit** (7 criteria, 34 rows): the all-or-none import has many observable facets. The unit
  boundary before U3 is a safe place to stop and resume.
- **The capture-target gate is frontend state** (as in C4): `useCaptureTarget` must wait for the remote state before
  activating. H4·C13 asserts whole call lists; the existing navigation tests must keep their expectations.
- **`photo_links` of the retired rows are deleted at bring back** so the new photos own the media ids alone; the
  superseded ids those links carried are dropped with them (after a completed publish none of that media exists any
  more).
- **Research and "Arranged" on a remote item** are untouched: they need local photos and behave as for a session
  without photos until bring back.

## Next
Tracker lines are all `NOT_STARTED`. Start with `PSM_IMPLEMENTER=claude tools/pipeline/run_phase.sh phase
H1_points_and_remote_record` (projection runs first).
