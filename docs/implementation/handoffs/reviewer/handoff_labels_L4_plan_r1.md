---
plan: L4_owner_corrections
role: plan
round: 1
state: OWNER_DECISIONS_PENDING
date: 2026-11-25
actor: Claude (pipeline-planner, implementation-planner doctrine; orchestrated)
---
# Price labels — plan for L4, the owner's corrections of 2026-11-24 (book 09C §14)

**Summary.** One milestone plan, `plans/L4_owner_corrections.md`, in the units format of flow §10: 3 units, 9 criteria,
51 rows (counted by script; every unit ≤ 8 criteria), `projection: no`, `depends_on: [L3_generate_and_download]`,
Tracker NOT_STARTED. Gate: 09C's status header reads **RATIFIED** (owner 2026-11-23, A33; owner corrections
2026-11-24, A34, §14); 10H is RATIFIED (A32) and carries §14.3's amendment in its §6. The mechanism-inventory gate is
replaced by `master_plan_labels.md` §5 / §5A, as for L1–L3. Code read at `99142a69`; every existing path the plan cites
resolves (script; the five unresolved paths are files L4 creates). Nothing was called or launched; the Pricing
package and one line of Pricing's source were read as files.

| Unit | Builds | Criteria | Rows |
|---|---|---|---|
| U1 | 09C §14.3: a shop-only row counts and filters as Published (`SessionsService::list`, so `page` and both commands) | 2 | 13 |
| U2 | 09C §14.1: notes with kind and address (adapter → core → command), a picture per checked product, the fixed-height dialog with pinned footer and product cards | 5 | 26 |
| U3 | 09C §14.2: the dialog saves and opens every file after a run, toast on full success, result view otherwise (frontend only) | 2 | 12 |

Totals of the track after L4 (script): 4 milestones, 14 units, 59 criteria, 345 rows.

## ⚠ OWNER DECISIONS REQUIRED (1)

**Card 2 — A product from the other Mac after you bring its photos back.**
- **Question:** After you bring back the photos of a product from the other Mac, should it stay under Published
  until a photo really changes (yes), or sit under Open from then on, as today (no)?
- **Story:** You open a sofa that was photographed on the other Mac and start to rotate a photo; the app brings its 8
  photos back to this Mac. Today the sofa then sits under Open, even if you cancel the rotation and change nothing,
  and stays there until you publish it from this Mac.
- **Branches:** **Yes** — it stays under Published until a photo changes; this needs extra work after L4 (the app
  must remember "published, as it came back"). **No** — nothing more is built; a brought-back product is under Open
  until you publish it here.
- **Recommendation:** No: photos only come back when you start a photo edit, so the product is about to change anyway.
- **On silence:** L4 is built (it does not depend on this); a brought-back product stays under Open as today, and
  nothing more is planned.
- **Trace:** 09C §14.3 second bullet against book 07 "Dirty" and 10H §7; MC-H3.7A; L4·U1 Notes (no row asserts it).

## Why the card (technical)
09C §14.3's second bullet says bring back and later edits "follow the published session's rules (07B / A10: still
Published, with its 'Changed' pill …)". Book 07 says a dirty published session **returns to ready / in progress**
(Open) with "Changed after publishing"; and a shop-only item was never published from this Mac, so after bring back it
is `Local` with `published_at` unset and `derive_status` answers Ready / InProgress. Keeping it Published would need a
published baseline stamped at bring back (`sessions.published_at` / `published_seq`), which reaches retention (10H
§7.7 "N days after its next publish"). That is not a small UI fix and the book's two sentences disagree, so L4 leaves a
brought-back item exactly as today and asserts nothing about it (see owner card 2). No L4 row changes with the answer.

## What was decided in planning (the narrative's "Choices made for you"; details in the plan's "Points recorded")
1. **§14.3 rule (MC-H3.7A):** a `Remote(ShopOnly)` row is `Published` whenever its label is `Archived`; when a publish
   of it shows through (running, held, failed, cancelled) the status is what `derive_status` gives. One place:
   `SessionsService::list` (`page` calls it; `sessions_page` and `sessions_list` both attach the remote items; the
   header, pickers and the workspace panel read `sessions_list`). `Unknown` (adopted, not read yet) and a shop
   product without pictures stay under Open. No frontend file changes; `SessionRow.tsx:20` and `SessionsPanel.tsx:15`
   follow the status.
2. **Card pictures (MC-L2.16):** computed in `check` from what is on this Mac: the matched session's row picture (one
   unpaged `list`), else the search-cache file of a `thumb_url` remembered from `list` / `select_all`. No new Pricing
   or Shopify call; nothing downloaded inside `check`; no live update of a card.
3. **Notes (MC-L2.15):** Pricing's note is `{code, message}` only; the address is inside the sentence. The adapter
   gives core `PlanNote {kind, message, link}`; "QR link" shows only with a link.
4. **§14.2 is frontend-only (MC-L2.17):** the dialog calls the existing `labels_download` once per file after a
   result; `LabelsService` and the run rules are untouched. "Everything generated" also requires that the check had
   no not-found product; the batch title counts them (`2 of 3`).

## Earlier tests L4 replaces (named in the plan per unit)
- `h4_c9a_a_shop_only_row_counts_its_record` (status `Ready`) → L4·C1(a).
- `l1_c7c_plan_reprint_and_nullable_template_and_title`: its note assertion (`notes == ["M"]`) → the `PlanNote` of L4·C3(b).
- `l3_c11a_labels_check_answers_the_groups`: expected JSON gains `picture_path: null` per item.
- `l3_c12b_one_section_per_template_with_a_line_per_product` → L4·C6(b), (c).
- `l3_c14d_leaving_after_a_failed_generate_shows_what_was_generated` → L4·C9(d).
- `l3_label_done_dialog.test.tsx` deleted: `l3_c15a`–`e`, `l3_c16a`–`e` (the done / Download PDF dialog) → L4·C8, L4·C9.
- `l3_c17e`: expectation kept, mocks only.

## Traps for the orchestrator
1. **Projection is off by choice.** The one outside shape (the `url_not_published` sentence) is grounded in Pricing's
   source, not in the package (`API.md` names the code only). The plan cites `plan.py:196-200` and the rule fails
   soft. If you prefer the charter's strict reading (a rule-17 fixture cannot waive projection), set `projection: yes`
   before the run; the plan needs no other change.
2. **The live check** of "QR link" is the first contact with a real note; if the pill never shows on a product that
   is not published, Pricing's sentence differs from the one read.
3. **jsdom cannot prove layout.** Fixed height, scrolling, one-line ellipsis and the 1:1 thumbnail are asserted by
   structure (L4·C6(a)) and by the reviewer reading the CSS (Review mode). The owner's try-out step 2–3 is the real proof.
4. **`master_plan_archive.md` was amended** (MC-H3.7A under MC-H3.7, log v1.4): the rule's home is the archive
   track's master plan. `docs/guide/02_architecture.md` :346 is an L4·U1 task, not done here.
5. **L4·C1 tests live in `h4_details_only_publish.rs`** (they need its helpers); the U1 filter is
   `--test suite h4_details_only_publish::`.
6. **`phases_labels.txt`** is still not read by `run_phase.sh all`; start with
   `PSM_IMPLEMENTER=claude tools/pipeline/run_phase.sh phase L4_owner_corrections`.
7. **Commit attribution.** The brief asked for a `Claude Opus 5.5` co-author line; this session runs on Fable 5.2 and
   its harness fixes the line to the model that wrote the commit, so the commit carries `Claude Fable 5.2`.
8. L2 r1's key-removal note and L3's two backlog notes are not taken by L4, except that "a second numbered copy after
   Preview could not open" can no longer happen (no retry is offered for a saved file).

## Write perimeter (this session)
- New: `docs/implementation/plans/L4_owner_corrections.md`, `docs/implementation/handoffs/reviewer/handoff_labels_L4_plan_r1.md`.
- Edited: `docs/implementation/master_plan_labels.md` (title v1.3, intro, §2 row and totals, §2A card 2, §3 L4 delta,
  §4 vocabulary / Words / Frontend / Tests, new §5A, §8 log v1.3), `docs/implementation/master_plan_archive.md`
  (MC-H3.7A, §8 log v1.4), `docs/implementation/phases_labels.txt` (L4 line).
- No code, no test, no `STATUS*.md`, no intention file, no other plan, no archgraph (the repo has none). One commit, not pushed.
