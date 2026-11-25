---
plan: L4_owner_corrections
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-25
actor: opus (claude-opus-5.5, pipeline-reviewer)
---
# Handoff: L4_owner_corrections, review r1

**Review history.** First review of this milestone. Implement r1 used checkpoints U1 `9cda2589`, U2 `7600a70b`, U3
`5d25b5f3` and close `50517f44`. Process authority is `docs/process/flow.md`: no mutation probes, blocking only B1–B4.

**Gate.** Passed. The tracker read `IMPLEMENTED`, all four checkpoint SHAs are in `git log`, and `50517f44..HEAD`
touches only the runner's review prompt.

## Verified correct
- **Perimeter.** The commits hold exactly the handoff's Files changed: `git show --stat` per checkpoint gives U1 4
  paths, U2 23, U3 11 and close 2.
  - The deviations are disclosed and stay within the plan's delegated choices:
    - `LabelCard.tsx` is also edited in U3 (a `failure` prop);
    - `useLabelDialog.ts` is edited in U2 (task 8's `show`);
    - `mechanisms/labels.rs` is untouched.
  - No `crates/infra/src/sqlite/` file and no `crates/core/src/mechanisms/` file changed. So there is no migration and
    no column (M-L1), and `derive_status`, `row_label` and `ready_session_ids` are untouched.
  - U3 changed no Rust file.
- **Checks.** Each ran once, one cargo command at a time, and all were green:
  - adapter `conversations::beyo_pricing::labels`: 66;
  - suite `h4_details_only_publish::`: 28 (H4·C9(b)–(g) unchanged and passing);
  - suite `l4_labels_check_cards::`: 7;
  - `psm-desktop`: 124;
  - vitest `src/features/labels`: 12 files, 95 tests;
  - clippy on core, integrations, testkit and desktop with `-D warnings`; fmt; typecheck.
  - The implementer's workspace stamp (suite 1464, npm 948) stands.
- **U1 (L4·C1–C2, 13 rows).**
  - `list` now computes the label before the status, from the same `input`.
  - `row_status` gives `Published` only for `Remote(ShopOnly)` with label `Archived`. Every other row gets
    `derive_status`.
  - The counts, the filter and `SessionRow.status` all use that one value.
  - The P65 `WaitingForEditor` branch is unchanged in effect: it is guarded by `!cleared_or_remote`, as before.
  - The tests cover every case MC-H3.7A names:
    - (a)–(d), (i): Published;
    - (e)–(h): a running, held, failed or cancelled publish shows through;
    - (j), (k): `Unknown` and `Local` stay Open.
  - C2 drives the real command bodies through the Tauri mock IPC and compares the JSON.
- **U2 (L4·C3–C7, 26 rows).**
  - **Adapter.** `plan_note` maps exactly as MC-L2.15 says:
    - only `url_not_published` gets a link;
    - the link is the first `http(s)://` word, with trailing `.,;)` removed;
    - `ftp` gets no link.
  - **Core.** `check_pictures` works as follows:
    - it calls `match_items` once;
    - it runs one `SessionsService::list(All)`, and only when a product matched;
    - otherwise it uses the remembered `https://` `thumb_url` through `search_picture` (cached file or nothing);
    - any error gives an empty map plus a warn log.
  - **Remembered rows.** `seen` is filled by `list` and `select_all`, and emptied by `forget()` (Test connection and
    key change).
  - **Tests.** C4 asserts the exact picture map, plus the request lines, shop reads, image fetches and
    `mutation_ops()`.
    - C4(c) and C4(f) would fail if the check downloaded a picture or kept stale addresses.
  - **Frontend.** `LabelCard` follows 09C §14.1:
    - the pill order is No price → issues → not-published (+ QR link) → other;
    - the tones are amber, red and grey;
    - QR link toggles the address, which opens only through `ExternalLink` → `openUrl`.
  - **Layout.** The structure is scroll / preview / footer. The CSS gives the dialog a fixed height
    (`min(720px, calc(100vh - 4rem))`, `overflow: hidden`), the list `overflow-y: auto` and the footer a place outside
    the list. The thumbnail is a 1:1 box at the row thumbnail's 50 px width, and the title and group lines are `nowrap`
    with an ellipsis.
- **U3 (L4·C8–C9, 12 rows).**
  - `deliver` saves each artifact id at most once per dialog (`savedOnce` ref), in order, while the phase is `saving`.
    In that phase it cannot be closed: `leave` returns early, and Escape and outside clicks are prevented.
  - "Everything generated" is `fullyGenerated` ∧ no unresolved ∧ every file `opened`.
  - The toast is raised only in `deliver`. The three hosts (`LabelRow`, `LabelsBox`, `DeleteSession`) are unchanged.
  - Only a failed, retryable save is asked again (`retry`). A file saved but not opened is never asked again: C9(g)
    asserts one call after a wait.
  - `labelsPartial` results go through the same `deliver`, as C9(d) shows.
  - `doneTitle` and `generatedToast` match MC-L2.17's words.
- **Earlier tests changed: exactly the named ones.**
  - `h4_c9a` is replaced by `l4_c1a`.
  - `l1_c7c`'s note assertion and `l3_c11a`'s `picture_path: null` are updated.
  - `l3_c12b` and `l3_c14d` are removed, and `l3_label_done_dialog.test.tsx` is deleted.
  - Elsewhere only fixtures changed. The now-unused `PRICE`, `T50`, `result` and `labelsPartial` import were also
    removed, for `noUnusedLocals`, and no expectation changed.
- **Rows → tests.** All 51 rows have a test at the boundary the row names. Each asserts the exact outcome and would fail
  on a wrong one, so there is no B2. The over-engineering test was applied: the only internal setup is fixture SQL, an
  existing pattern, and no test asserts call counts of internal collaborators.
- **Reading checks of Review mode.** All hold:
  - both session commands still attach `.with_remote`;
  - `provider.run` is called only at `label_runs.rs:323`;
  - no `Download PDF` or `Done` remains in `features/labels`, except in the tests that assert their absence;
  - core names no provider code (H7): `url_not_published` and the sentence form live only in the adapter;
  - L4·C3's fixture sentence has the source form of "Rule 17";
  - fixtures are neutral (`TEST-…`, `shop.example`, a zeroed CDN path).
- **Assumptions (12).** None contradicts 09C §14, MC-L2.15–.17 or MC-H3.7A. All are put in plain words in the narrative.
- **H1–H8.** All hold:
  - no new external write (C4(g));
  - no RAW, pixel or UI-automation change;
  - each file is asked once (H4);
  - labels stay optional (H6);
  - H7 as above;
  - core has no Tauri dependency (H8).
- **Guide (§7A).** `02_architecture.md` (Rows MC-H3.7, Labels check / frontend) and `04_change_impact.md` (check item
  DTO row) are updated.

## Findings
None blocking (B1–B4: none). Two notes are below.

## Notes written to backlog
All are in `docs/implementation/backlog_labels/L4_owner_corrections.md`, the path the plan's Review mode names, as in
L1–L3.
- **UX:** a card for a product with no title (nullable in Pricing) shows an empty first line.
- **UX:** the result view's problem cards have a pointer cursor and a hover border, but clicking them does nothing.
- **Closed:** L3 r1's note on duplicate PDFs from Download PDF. L4 removed the button, and a saved but unopened file is
  never asked again.
- **Carried forward:** L2 r1's note "a key change leaves the Labels box rows on screen" (`key_changed` emits no
  `LabelsChanged`).

**Carry-forward dispositions**

| Note | Destination |
|---|---|
| Empty title line on a card | orchestrator → next labels phase or a small fix |
| Problem cards look clickable | orchestrator → next labels phase or a small fix |
| Key change leaves Labels box rows (from L2) | orchestrator (still open since L2) |

## Lessons for the plans
- L4's Review mode named its own backlog path (`backlog_labels/…`), but the generic review prompt says
  `docs/implementation/backlog.md`. The prompt template could read the plan's path when the plan names one.

## Probe declaration
I edited no production or test file, ran no probe, and changed no database or state. The only files I wrote are this
handoff, the plan's Tracker line and Review log entry, `backlog_labels/L4_owner_corrections.md` and
`narratives/L4_owner_corrections.md`.

## ⚠ OWNER DECISIONS REQUIRED (0)
This review raises no new card. The planner's existing **card 2** (master plan labels §2A: should a brought-back product
from the other Mac stay under Published) is still open. It is relayed verbatim in the narrative's "Needs you" so the
owner sees it. L4 does not depend on it, and on silence nothing more is built.
