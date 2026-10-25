---
plan: intention
role: inventory
round: 1
state: OWNER_DECISIONS_PENDING
date: 2026-10-25
actor: Claude Opus 5.5 (pipeline-inventory, mechanism-inventory doctrine)
---
# Mechanism inventory of the ratified intention — round 1

**Summary.** Gate passed (intention RATIFIED 2026-10-24). I found 20 load-bearing mechanisms and
gave each a contract (MC-1…MC-20) in three lettered book chapters, 05A, 07A and 10A. Their
invariants are registered as M8–M27 in chapter 14. I ran 10 Capture One VERIFY probes
against the real app (5 PASS, 5 PARTIAL) and added 2 new items, both UNPROBED. Every probe
that needs a capture is UNPROBED, because the camera did not release the shutter on any of 9
scripted captures. The probes found four silent-failure facts that the book's design would have
hit:
1. hooks block Capture One while they run;
2. the processing hook reports the wrong output path under the default `add suffix` recipes;
3. a failed capture-folder set leaves a garbage read-back;
4. a colour-tag write fires the adjusted hook, and the hook carries no arguments.

I resolved 17 contradictions or ambiguities by contract (R-I1…R-I17, listed for ratification).
One question needs the owner (card I1). I made no material semantic change, so the header stays
RATIFIED. The exit gate is met: every silent-failure mechanism has a contract. The planner can
start. Card I1 only affects the re-publish behaviour of the publishing phase.

## ⚠ OWNER DECISIONS REQUIRED (1)

**I1 — Retouch after publishing.**
- **Question:** When you retouch an already-published photo in Capture One, should the session
  need publishing again, and the next publish replace that photo on Shopify?
- **Story:** You publish the oak sideboard on Monday. On Tuesday you heal a dust spot on Main 03
  in Capture One. Session Manager's preview updates, but the session still says Published, and
  even a second publish leaves Monday's image on Shopify, because that photo was already
  uploaded. The listing shows the dust spot until a customer mentions it.
- **Branches:** *Yes*: the retouch puts the session back to "Ready · Publish", and the next
  publish uploads the new version into the same gallery place and removes the old one. *No*:
  Shopify keeps the first version, and to change it you delete the photo and capture it again.
- **Recommendation:** Yes, because the live preview already promises that what you see is
  what goes out.
- **On silence:** the gate holds. The ratified rule (no replacement) is what gets built until
  you answer.
- **Trace:** 07 Dirty; 07A MC-11; 05A MC-6; 10A MC-17.

## Gate check
`docs/intention/intention.md` header: `Status: RATIFIED (owner David Loorenz, 2026-10-24;
chapter 16 entry 13)`. Passed. Read: `docs/process/flow.md`, the whole book (01–16), both
evidence docs, the sdef sections the probes needed, `claude_design/reference/SHOPIFY_PRODUCT_METAFIELDS.md`.

## Inventory (ranked by silent-failure risk)
| # | Mechanism | Risk | Contract | Status |
|---|---|---|---|---|
| 1 | Capture arrival: candidate filter, stability, session attribution, dedupe | High: lost, duplicate or misfiled captures, all silent | MC-2 (05A), M9 | contracted; capture-done argv unprobed (VERIFY-1/12) |
| 2 | Hook delivery from Capture One to the app | High: a blocking hook freezes Capture One; events lost when app closed | MC-15 (10A), M22 | contracted from probes |
| 3 | Capture-folder switch per session (D1) | High: failed set leaves garbage folder; captures go astray | MC-18 (10A), M25 | contracted from probes |
| 4 | Path canonicalization / RAW identity | High: NFD/NFC, `/private`, case spellings create duplicate rows | MC-1 (05A), M8 | contracted |
| 5 | Publish resume and idempotency keys (item create, uploads, link scope) | High: duplicate products/media, cross-product media leak after a move | MC-11 (07A), M18 | contracted; Shopify grounding owed (SHOPIFY-VERIFY-2/3/6) |
| 6 | Gallery reconciliation (N1 mirror) | High: misorder, deleting media nobody confirmed | MC-12 (07A), M19 | contracted; SHOPIFY-VERIFY-4 |
| 7 | Confirmation freezing and mutation token (H1, M5) | High: unconfirmed writes, deleted photos leaking | MC-10 (07A), M17 | contracted |
| 8 | Render jobs: output path and job correlation | High: wrong or old file recorded as preview or export | MC-16 (10A), M23 | contracted from probes |
| 9 | Live-preview staleness and trigger | High: preview stuck on an old edit, cached image never refreshed | MC-17 (10A), M24 | contracted; UI drag rate unprobed (VERIFY-9) |
| 10 | Dirty flag | High: change during publish lost, session shows Published while platform differs | MC-6 (05A), M13 | contracted; card I1 may add an event |
| 11 | Attribute write set and Shopify type map | High: clobbering admin edits, type drift, string choices coerced | MC-13 (07A), M20 | contracted; SHOPIFY-VERIFY-1/5 |
| 12 | Damage-report selection storage | Medium-high: new condition photos silently excluded | MC-14 (07A), M21 | contracted |
| 13 | SKU folder key and export naming | Medium-high: case-insensitive folder merge, stale export files | MC-8 (05A), M15 | contracted |
| 14 | Attribute canonical value forms and equality | Medium: drift, phantom writes | MC-9 (05A), M16 | contracted |
| 15 | Document/variant addressing and colour-tag mirror | Medium: tags on wrong/no variant, stale tag after rapid classify | MC-19 (10A), M26 | contracted; VERIFY-4 colours and VERIFY-11 open |
| 16 | Position math, move/copy, undo | Medium (mostly visible) | MC-5 (05A), M12 | contracted |
| 17 | Status derivation and row label | Medium: hidden republish failure | MC-7 (05A), M14 | contracted; order verified total |
| 18 | RAW hash | Medium: hash of a half-written file = false alarm forever | MC-3 (05A), M10 | contracted |
| 19 | Time: storage, captured_at, grouping, timers | Medium: DST-day grouping, camera clock | MC-4 (05A), M11 | contracted |
| 20 | Original preview extraction | Medium: wrong stream, orientation | MC-20 (10A), M27 | contracted from RAW inspection |
Not contracted (failure is visible at once, or the rule already sits verbatim in the evidence):
field-kind thresholds (`evidence/design_handoff.md`), swipe distances, toast timing, progress
bar fractions (defined in MC-7), backup rotation, settings file permissions.

## Probes
| VERIFY | Result | Evidence (in `evidence/capture_one_16.8.6.md`) |
|---|---|---|
| 1 hook arguments | PARTIAL | processing done `[job id, RAW, [outputs]]` (13 calls); batch done one path; adjusted: no args; capture done UNPROBED |
| 2 hooks set at runtime | PASS | fire immediately when set by script to a `.scpt` path; read back via `/private`; `""` clears; **hooks are synchronous: 6 s hook delayed a query 5.1 s** |
| 3 capture folder must exist? | PASS | must be created first; failed set reads back `<caller cwd>/Capture`; restored |
| 4 colour-tag integers | PARTIAL | r/w and persisted (`Color_Tag_Index`); fires adjusted hook; colours not visually checked (no screen access) |
| 5 process output path | PASS | job id returned; `root folder type` not settable; argv 3 wrong under `add suffix`, right under `overwrite`; output events true |
| 6 batched tag read cost | PARTIAL | N=1: 13 ms in-script, 110–130 ms per osascript; N≥50 unprobed |
| 7 document addressing | PARTIAL | `byId(<folder path>)` and `byName` work; two documents unprobed |
| 8 hook → app channel | PASS (processing) | spool file + background curl nudge: 0.56–0.67 s, no blocking; curl inline 30–35 ms but blocks; capture latency unprobed |
| 9 adjusted hook rate, sidecar | PARTIAL | once per scripted change, 130–165 ms, no args; in-hook primary-variant query 30–55 ms; `.cos` only at autosave 9–15 s later; UI drag rate unprobed |
| 10 preview render latency | PASS | 0.52–0.72 s, 10 runs, 1600 px JPEG |
| 11 (new) variant lookup across browsed folders | UNPROBED | needs captures in two subfolders |
| 12 (new) scripted capture | UNPROBED, blocked | 9 releases logged by Capture One, no frame written; likely focus priority |

Commands to run each UNPROBED item later are written under its VERIFY entry in the evidence.

## What I added to the book
- `book/05A_mechanism_contracts.md`: MC-1…MC-9 (identity, arrival, hash, time, order, dirty,
  status, naming, value forms).
- `book/07A_publish_contracts.md`: MC-10…MC-14 (frozen run, resume keys, gallery
  reconciliation, attribute writes plus type map, damage report) and SHOPIFY-VERIFY-1…6, the
  grounding owed before the adapter phase's criteria are final (charter rule 17).
- `book/10A_capture_one_contracts.md`: MC-15…MC-20 (hooks, renders, live preview, folder
  switch, addressing and tags, original preview).
- `book/14_measurement_ledger.md`: M8–M27, one per contract.
- `book/15_decisions.md`: open card I1.
- `book/16_changelog.md`: entry 14.
- `intention.md`: map rows for 05A, 07A and 10A, plus one read-order sentence. Status header
  unchanged.
- `evidence/capture_one_16.8.6.md`: results under VERIFY-1…10, new VERIFY-11/12, other
  observations, probe perimeter.

## Contradictions resolved by contract (for owner ratification)
No product sentence was changed; each is cited in its contract as **(ratify R-In)**.
| ID | Book tension | Resolution | Consequence the owner may care about |
|---|---|---|---|
| R-I1 | "active session at arrival" vs events processed seconds later; Unassigned with per-SKU folders | session decided by the RAW's capture folder; `_Unassigned` folder when no session is active | a frame shot just before switching sessions stays with the old session |
| R-I2 | ch06 "unreviewed joins at end of Main" vs ch05 position null | position null; shown after Main; approve appends | none visible |
| R-I3 | ch05 "reassignable until published" vs S-K moving out of published sessions | moving out allowed; published sessions are not targets | as S-K describes |
| R-I4 | photo_links had no product scope | key includes item_id | a moved photo is uploaded again for its new product |
| R-I5 | N1 mirror vs H1 for media added on Shopify after confirming | update fails, deletes nothing unannounced | occasional "publish again to review" |
| R-I6 | M3 "order at confirmation" vs "no deleted photo leaves the Mac" | deleted/moved-away photos skipped mid-run | gallery may be one shorter than confirmed |
| R-I7 | status `failed` only if never published vs label "Publish failed" | label shows any failed latest item | failed republish is visible in the list |
| R-I8 | "set attribute values" vs never replacing platform data | diff write, never clears | admin-only fields untouched |
| R-I9 | M3 "every condition photo in the attribute" vs pickable field | selection governs; stored as exclusions | deselected photos are not published |
| R-I10 | "by filename and hash" | remote match by upload filename only | uploaded file names are `<sku>-<id>.jpg`, not `_01_main` |
| R-I11 | SKU unique (case-sensitive) vs case-insensitive folders | colliding SKUs refused | rare refusal message |
| R-I12 | move/copy onto a session that already has the RAW | refused | — |
| R-I13 | `done` sets dirty false even after mid-publish edits | change sequence | session stays Ready after mid-publish edits |
| R-I14 | update step sends values current at step time vs H1 | frozen at confirmation | edits during a run go out next time |
| R-I15 | sidecar fallback "under the watcher" | startup and focus only (sidecar lags 9–15 s and changes on our own tag writes) | — |
| R-I16 | startup scan of "the active capture folder" | every per-SKU folder | — |
| R-I17 | design maps date_time → date input | read-only in MVP (shop has none) | — |

## Notes for the planner and coordinator
- Phases touching MC-1/2/15/18 (capture arrival), MC-10/11/12/13 (publish) and MC-16/17
  (preview) are silent-failure phases: projection is mandatory for them under the charter
  (rule 6), whatever `flow.md`'s default says. This is for the coordinator to settle.
- The packaging spike (ch11) should re-run VERIFY-1, 8 (capture latency) and 12 once the
  camera releases, and VERIFY-11. The owner's 10-second checks for VERIFY-4 colours and the
  VERIFY-9 slider rate are in the evidence. They do not block planning: the contracts already
  handle both outcomes (the 10A MC-19 fallback, and in-hook identity for MC-17).
- The adapter phase must ground SHOPIFY-VERIFY-1…6 once the token arrives, before its criteria
  freeze.
- 07A introduces columns the planner must carry into migrations: `photos.uid`;
  `photo_links.item_id` and `export_sha256`; `sessions.change_seq` and `published_seq`; on
  `publish_items`: `failed_step`, `frozen_main`, `frozen_condition`, `frozen_attributes`,
  `announced_removals`, `confirmed_change_seq`. Plus `attribute_values.valid` and photo
  `orientation`. 05A makes `dirty` derived; the stored boolean in ch05 becomes this derivation.

## Write perimeter
- Repo, created: `docs/intention/book/05A_mechanism_contracts.md`,
  `docs/intention/book/07A_publish_contracts.md`, `docs/intention/book/10A_capture_one_contracts.md`,
  this handoff.
- Repo, edited: `docs/intention/intention.md` (map rows, one sentence),
  `docs/intention/book/14_measurement_ledger.md`, `docs/intention/book/15_decisions.md`,
  `docs/intention/book/16_changelog.md`, `docs/intention/evidence/capture_one_16.8.6.md`.
- No code, no archgraph, no commits.
- Outside the repo: `/tmp/psm_probe/`, `/tmp/psm_probe_tools/`, `/tmp/psm_probe_state.js`
  (probe scripts and logs). In the Capture One document `~/Pictures/session test 1/`: probe
  renders in `Output/` and `Output/psm_probe_sub/` with their `CaptureOne/Settings1680/*.cos`,
  and the empty folder `Capture/PSMPROBE-0001/`. Capture One state changed and restored: hook
  properties (null again), capture folder (original), colour tag (0), exposure (0), temporary
  recipe "PSM probe preview" (deleted). Capture One was brought to the front once
  (`activate`). The RAW's SHA-256 was unchanged before and after.
