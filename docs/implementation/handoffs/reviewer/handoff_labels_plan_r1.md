---
plan: labels
role: plan
round: 1
state: OWNER_DECISIONS_PENDING
date: 2026-11-23
actor: Claude (pipeline-planner, implementation-planner doctrine; orchestrated)
---
# Price labels from Pricing — plans for L1–L3 (book 09C)

**Summary.** A short master plan (`master_plan_labels.md`) whose §5 resolves 09C's four contracts (MC-L1 the Labels
box, MC-L2 check / generate / download, MC-L3 the display copy and the marker, MC-L4 staying in step) and its
connection rule (ACC) to code-level rules against the code at `ae91079` and the Pricing integration package as read on
2026-11-23; three milestone plans in the units format of flow §10; the phase list. Gate: 09C's status header reads
**RATIFIED** (owner, 2026-11-23, changelog A33). The mechanism-inventory gate is replaced by §5 (recorded as a process
delta, as for the writer, conversations and archive tracks); 09C §9 (M-L1…M-L6) is the ledger the rows trace to.
Every cited existing path resolves (script; the only unresolved paths are the files these milestones create). The
Pricing shapes were read from `openapi-integration.json`, `API.md`, `LABELS.md` and `EVENTS_AND_WEBHOOKS.md` (files
only; nothing was called). One thing was probed on the dev Mac: turning a PDF page into a picture (below, point 1).

Counts (computed from the plan files by script): 3 milestones, 11 units, 50 criteria, 294 rows; every unit ≤ 8 criteria.
| Milestone | Units | Criteria | Rows | App files | Projection |
|---|---|---|---|---|---|
| L1_labels_provider_and_marker | 4 | 19 | 109 | yes | **yes** |
| L2_labels_box | 3 | 14 | 78 | yes | no |
| L3_generate_and_download | 4 | 17 | 107 | yes | **yes** |

Ledger coverage: M-L1 (L1·C13(f), L3·C8(c)) · M-L2 (L1·C7(a), L2·C3(d), L3·C1(g), L3·C3(c), L3·C3(g), L3·C5(e),
L3·C12(f)) · M-L3 (L1·C9(a), L3·C6(a), L3·C7(a)–(e), L3·C8(a), L3·C14(a)–(b)) · M-L4 (L3·C9, L3·C15(b), L3·C15(e)) ·
M-L5 (L2·C3(d), L2·C4(b), L3·C1(g), L3·C10(g)) · M-L6 (L1·C12(a), (c), (h), L2·C1(f), L2·C5(e), L2·C6(f), L3·C1(h),
L3·C11(q)). Every scenario of 09C §12 (1–12) is cited by at least one row. No ledger entry is unserved; no row traces
to nothing.

## ⚠ OWNER DECISIONS REQUIRED (1)

**Card 1 — When does the Labels box show its number?**
- **Question:** Should the Labels box show its number as soon as the Sessions list opens (yes), or only after you
  have gone into the box once (no)?
- **Story:** You start Session Manager in the morning. Pricing holds 12 priced products without a label. With "no",
  the Labels box shows no number until you click into it; you would have to open it to learn whether anything needs
  printing. With "yes" it reads 12 at once, next to Open and Published, and drops as labels are printed.
- **Branches:** **Yes** — one small extra question to Pricing each time the Sessions list opens and after a label
  changes (never more than one every 2 seconds). **No** — no extra question; the number stays empty until your first
  visit to the box after each launch.
- **Recommendation:** Yes: the number is what tells you to open the box.
- **On silence:** L1 is built (it does not depend on this); L2 does not start.
- **Trace:** 09C §5 "Place" against §12 scenario 1; MC-L1.6; rows L2·C5, L2·C7(d), L2·C10(a), L2·C14(c).

(Technical note for the coordinator: 09C §5 says the box's number is "from the last list answer; before any answer,
empty" and lists no read outside the box, while §1.1 and scenario 1 have the box showing the number before it is
opened. The L2 plan is written for **yes**; master plan §2A lists the four edits for **no**. `L2_labels_box.md`
carries `blocked_on: owner card 1`.)

## Write perimeter (this session)
Documents only, all new: `docs/implementation/master_plan_labels.md`, `docs/implementation/phases_labels.txt`,
`docs/implementation/plans/{L1_labels_provider_and_marker.md, L2_labels_box.md, L3_generate_and_download.md}`, this
handoff. No code, no migration, no tool-recorded state (the repo has no architecture graph). Not touched: `STATUS.md`,
`STATUS_labels.md`, book 09C, the Pricing repository (read only), any other plan or prompt. Scratch files (a test PDF
and its pictures, three small scripts) are in the session's scratchpad outside the repo.

## What the chapter left to planning, and how it is resolved
1. **The preview picture** (master plan §3, MC-L2.5; row L3·C4(c)). Probed on the dev Mac (macOS 26.5.2): `sips -s
   format png -Z 1200 <pdf> --out <png>` renders a 70 × 100 mm vector PDF page to a sharp 840 × 1200 PNG; without `-Z`
   it is 198 × 283; a non-PDF exits 13 and writes nothing. The PNG is written under `<data>/previews/labels/`, the
   only place besides `templates/` the webview may read, and shown on a white tile (it has an alpha channel). No new
   dependency. Not verified on the work Mac's macOS; a failed raster only replaces the picture by a sentence.
2. **Cursor pages** (MC-L1.9; rows L2·C8, L2·C12): a cursor stack in the frontend (`cursors[0]` none, `cursors[k+1]`
   = the `next_cursor` of page `k`), changed only by the pure `pageStep`; Previous re-asks the cursor already seen; a
   newer answer for a page drops the later cursors; an empty later page and a rejected cursor go back to page 1;
   "Page k of n" from the filter's count.
3. **The display copy and "labels changed"** (MC-L3.2, MC-L4): an in-memory map with a per-product event sequence, so
   **an answer that was in flight when an event arrived cannot overwrite the event** (unit-tested as a Core
   mechanism). One `LabelsChanged {product_ids, access}` per pull page, also when the page stops on an error (the
   cursor has already moved past those events).
4. **Chunks above 300 and keys** (MC-L2.1, .6, .7): the check is cut into chunks of 300 and merged; a run chunk gets
   its key once (`sm-` + UUID v4) and the service, not the screen, owns the keys, the retries (twice, at once, same
   key and same ids) and Try again; a finished chunk is never sent again. Runs start only through
   `generate(check_id)`: no call takes product ids for a run (M-L2 bound at the interface).
5. **The Labels folder naming rule** (MC-L2.13): the provider's filename is reduced to its last path component and
   cleaned, `.pdf` is ensured, a taken name gets ` (2)`, ` (3)` …, `.part` then rename; the folder is derived from
   `AppPaths.home` at one place.
6. **Unmatched rows** (MC-L1.3, .8): matched by the link's product id, else by exact SKU; opening an unmatched row
   runs the existing `adopt` (one shop read, no write) and creates the session (`adopt` alone creates none), then
   navigates; the row is matched from then on.
7. **Provider-neutral core** (master plan §4 Code layout): core says labels provider, label state, check, run,
   product, labels folder; the two id forms of a product (`gid://shopify/Product/<n>` ↔ `<n>`) live only in the
   adapter, behind the pure port methods `product_id_of` / `commerce_id_of`; the sentences that name Pricing come only
   from the adapter's `describe`.

## Traps given explicit rules (each has a row or a Notes hazard)
- **A refused label call must not kill conversations.** The existing `read_error` / `write_error` map 401/403 to
  `ProviderError::Refused`, which sets the connection to `refused`. Label calls answer their own `LabelsError` and
  never write the connection state (ACC.5; L1·C12(g)).
- **A PDF through the existing transport is silently damaged** (`send` decodes every body as JSON or lossy text). A
  binary `fetch` is added and used for preview, download and run (L1·U1 Notes; L1·C8(a), L1·C10(a)).
- **The key could be sent to another host by a download address.** The adapter sends only a path under the store's
  own `/labels/runs/` prefix, appended to the configured server; anything else is refused without a request
  (MC-L2.12; L1·C10(b)).
- **Whether the key may read labels** is stored (`conversation_sync.labels_access`, migration 022: the only thing
  09C stores, and not label state), so a key without the scope costs one call, not one per launch (ACC; L1·C12).
  The migration needs its `DROP COLUMN` line in the p27 rewind test.
- **`sessions_changed` fires while photos arrive;** asking Pricing for markers on each would be a loop. The marker
  read refreshes only when the page shows other sessions, and never within 10 s for the same products (MC-L3.3, .7;
  L1·C13(c), L1·C18(c)).
- **`connected()` after the lane is released**, or the pull and every reply wait behind a templates read (L1·U3
  Notes).
- **The fake shop's product ids are `product-<n>`**, which no label row can match; tests need `add_product_with_id`
  with a `gid://shopify/Product/<digits>` id (L1·U3 task 7).
- **No test may open Preview or write into the owner's Documents:** `FakeLabelFiles` in every `TestApp`; `TestApp`'s
  home is its temporary folder (master plan §3 Rule 9, §4 Files).
- **Picture addresses come from Pricing and are fetched by the shop adapter:** only `https://` ones are handed on
  (MC-L1.4; L2·C3(c)).
- **Select all must end:** a repeated cursor or more than 200 pages is an error, and an error never leaves a
  partial selection (MC-L1.7; L2·C6(d), (e)).
- **`workFrom` is a date, not a time:** formatted from its text parts (L2·C9(d)).
- **The preview PNG is transparent:** shown on white in both themes (L3·C13(a)).
- **"2 of 3 labels generated".** 09C §6.2 sends only the products that passed the check, while §6.3 and scenario 4
  count three with one failure. Resolved in MC-L2.9: the result counts the check's refused products as failed and
  lists them with their reasons. No owner call needed (both texts hold).

## Points the orchestrator should know
1. **Products are always addressed by product id; SKU addressing is not built.** Every Labels row carries the id and
   every other entry point is a session with a shop link (the entry is disabled without one), so 09C's "else SKU" has
   no caller (master plan §6). Say so if the owner expects labels for unpublished local items by SKU.
2. **L1 shows the ⋯ entry only in its disabled states** (09C §13 "disabled states only until L3"); the enabled entry
   arrives in L3 and replaces one L1 test expectation (L1·C19(c)); likewise the batch button is on the bar but
   disabled in L2 (L2·C13(a) → L3·C17(d)). Both replacements are named in the L3 plan.
3. **Marker precedence** (MC-L3.5) is a planning choice where 09C's four cases overlap: printed, then a reset reason,
   then "No price yet", then "Missing price tag"; a product Pricing does not list (`hasPrice` null in the status
   answer) gets a marker only when printed or reset.
4. **A batch that fails half-way** (chunk 2 of 2 unreachable after retries) can be left: the dialog then shows what
   chunk 1 generated, with the rest "not generated" and still selected (MC-L2.10; L3·C14(d), L3·C15(e)). 09C does not
   describe this case.
5. **No pause between the two automatic retries** (the request's own time-out spaces them): a dropped connection
   usually ends at Try again. Cheap to change if the owner wants a pause.
6. **The live check after L3 should stay read-only** (list, status, templates, one check, one preview): a run marks
   products Printed in the real Pricing, and 09C §13 gives the first Generate to the owner. Unverified until then:
   what Pricing answers for a stale cursor, and the PDF bodies.
7. **macOS will ask once for the Documents folder** when the first PDF is saved (the app is not sandboxed, but
   Documents is protected); it is in L3's try-out and narrative points.
8. **Fixture values are neutral** (`TEST-1802`, "Test table", "Label 70×100"), keeping the package's shapes; the ids
   reuse those already in the C1 fixtures. The package's own examples (BEYO-… SKUs, a template name) are not copied.
9. **Commit trailer.** The task asked for `Co-Authored-By: Claude Opus 5.5`; this session runs on Fable 5.2 and the
   session's standing attribution rule names that model, so the commit carries `Co-Authored-By: Claude Fable 5.2`
   (as the archive planning commit did). Amend it if the project wants the role's nominal model there.

## Risks
- **L1 changes the pull** (`ConversationsService::pull`) and `connect` / `key_set`: small branches, but in the code
  that keeps conversations in step. Projection on L1 should attack the early returns of `pull` (delivery on every way
  out) and the existing C1 suite must stay green (`c1_conversations_sync::` is in L1·U3's commands).
- **All label shapes are schema-grounded, not captured.** Projection on L1 should re-read the fixtures against
  `openapi-integration.json`; the orchestrator's live check is their first real contact.
- **L3·U4 is the largest UI unit** (6 criteria, 32 rows, every sentence of 09C §6 verbatim); the unit boundary before
  it is a safe place to stop and resume.
- **The preview depends on `sips` behaving the same on the work Mac.** If it does not, the dialog shows "The label
  preview can't be shown." and everything else works; that would be a follow-up, not a blocker.
- **Large "Select all" batches** are synchronous per chunk (Pricing renders up to 300 labels per call, 60 s time-out
  each); the dialog stays busy and cannot be closed meanwhile, as 09C §6.2 asks.

## Next
Tracker lines are all `NOT_STARTED`. Start with `PSM_IMPLEMENTER=claude tools/pipeline/run_phase.sh phase
L1_labels_provider_and_marker` (projection runs first). L2 waits for L1's approval **and** owner card 1.
