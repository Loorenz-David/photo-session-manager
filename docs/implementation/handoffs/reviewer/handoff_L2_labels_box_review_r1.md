---
plan: L2_labels_box
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-23
actor: opus (pipeline-reviewer)
---
# L2 — The Labels box · review r1

**Verdict: APPROVED_WITH_NOTES.** Production matches MC-L1.1–.9 and 09C §5 (with card 1). Every one of the 78 criterion rows has a test at its boundary that fails on a wrong outcome. Review-mode checks are green. H1–H8 hold. No assumption contradicts the book. Four notes go to the milestone backlog, and two L1 backlog notes are resolved.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- The plan's Tracker line read `IMPLEMENTED` (2026-11-23).
- Checkpoints `e81b52d3`, `70f4d19b`, `80a720ca` and `3fe83622` are in `git log`.
- Intention: RATIFIED.
- Owner card 1 was answered 2026-11-23 ("yes"; `master_plan_labels.md` §2A).

## Verified correct
**Perimeter (step 1).**
- `git show --stat` per unit: U1 has 13 paths, U2 has 10, U3 has 12, and the close commit touches the plan only. These equal the handoff's Files changed.
- Paths outside the unit file lists were all disclosed as deviations:
  - `useSessionLabels.ts`, for U3 task 2.
  - Guide edits in U2 and U3, per flow §7A.
  - `l1_markers.test.tsx`, a mock line only, as U3 Files allows.
- No file under `features/sessions/__tests__` changed.

**Checks (step 2).** Each ran once, one cargo command at a time, on the clean tree `5f371d5f`:

| Check | Result |
|---|---|
| `cargo test -p psm-core --lib mechanisms::labels` | 14 passed |
| `cargo test -p psm-testkit --test suite l2_labels_list::` | 36 passed |
| `cargo test -p psm-desktop` | 103 passed (6 `l2_tests`) |
| `npx vitest run src/features/labels src/features/sessions` | 21 files, 117 passed |
| `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-testkit -p psm-desktop --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npm run typecheck` | exit 0 |

The implementer's full-suite stamp at `80a720ca` stands: later commits touch docs only. The p33 intermittent is listed in the guide.

**Rows (step 3), production first, then the test.**

- **C1 (MC-L1.1, .2, .5)**
  - `gate()` answers `Off` or `Denied` before any request.
  - `search_text` trims the query, drops it when empty, and cuts it to `QUERY_MAX` chars.
  - The cursor is passed through untouched.
  - `products()` takes `copy.start()` before the request, applies each row's state, and remembers the counts with `searched`.
  - Tests assert the exact request lines (including `cursor=o%3A25`), exact row tuples in order, and exact error variants. C1(e) can fail: if the copy were not applied, `session_labels(false)` would answer `None`.
- **C2 (MC-L1.3)**
  - `match_product`: link first, lowest item id, exact case-sensitive SKU.
  - `linked_items` is one SELECT, with the latest session as `MAX(s.id)`.
  - There is one `SessionsService::page(All, …, Some(&ids), 1)` call, skipped when there are no ids.
  - C2(c)'s fixture makes link-over-SKU the only reason for `sA`. C2(e) covers the null SKU and the case-different SKU.
- **C3 (MC-L1.4)**
  - `https://` ∧ `commerce_id` ∧ commerce present → `search_picture`, which only enqueues in memory. `list` never downloads.
  - C3(b) observes None, then the file, the fetch `(url,160)`, the exact `PictureReady`, and then `Some(file)`.
  - C3(c) observes no fetch for null or `http://` addresses.
  - C3(d) observes an empty `mutation_ops` and no `/labels/runs`.
- **C4 (MC-L1.8)**
  - Match → latest session, or `create`.
  - Otherwise `commerce_id_of` → `adopt` → `create`. `adopt` errors (no SKU, `SkuCollision`) return before any write.
  - Tests assert the shop read delta (`["item_get"]`), the item and session counts unchanged on error, and the same session on a second open.
- **C5 (MC-L1.6, card 1)**
  - A refresh request is made only when nothing is remembered ∨ the remembered answer was searched ∨ its age ≥ 10 s.
  - Not allowed → None, with no request. `connected()` and `key_changed()` forget the counts.
  - Tests use exact request lines and a 9 s / 11 s clock.
- **C6 (MC-L1.7)**
  - 500 per page. A repeated cursor, or more than 200 pages, → `BadReply`. Any error → that error, never a partial selection.
  - SKU-less rows are dropped.
  - C6(a) asserts all 620 ids in the list's order and both exact request lines.
- **C7 (commands)**
  - Thin bodies: one service call on `spawn_blocking`. The DTOs embed `SessionRowResponse` and L1's marker wire form.
  - C7(a) compares the whole JSON exactly. C7(b) compares the whole `labels_off` error exactly.
- **C8, C9 (pure functions)**
  - `pageStep` per MC-L1.9: a stale index is ignored, an empty later page goes back to the initial state, and later cursors are dropped.
  - `pageLabel` = `max(k+1, ceil(n/25), 1)`.
  - `formatPrice` groups digits with U+2009 and ends with U+00A0 `kr`.
  - `workFromLine` is built from the text's parts, not from a `Date` (safe in any time zone).
  - Exact equality throughout.
- **C10–C14 (screen)**
  - `Filters` has a fourth item. `SessionsPage` hides the time filter, "To review", the Unassigned card, the table, the pager and the batch bar in the box. It keeps `sessionsPage` for the box it left.
  - `useSessionLabels(null)` asks no status.
  - `LabelsBox` asks the first page on enter, filter, search and Try again. A refresh asks `askCursor` of the page shown, keeps rows, page and selection, and shows the error above the rows.
  - The throttle is leading, plus one folded trailing call per 2 s. `labels_rejected` and an empty later page go back to page 1 once.
  - The selection is a map. Select all replaces it with exactly the answer. It is cleared on filter, search, unmount (leaving) and Escape.
  - Generate is disabled.
  - Tests assert exact `labelsList`, `labelsSummary`, `labelsSelectAll` and `labelsOpen` call lists and the rendered text. C14(b) checks 0 s, 1.999 s and 2 s with fake timers.

**Review-mode reading checks.**
- `list`, `summary`, `select_all` and `open_product` never call `run`, `plan`, `preview` or `download`.
- `open_product` calls only `adopt` and `create`.
- The list never downloads.
- Only `https://` addresses reach the shop adapter.
- Nothing about a label row is written to the DB. `linked_items` is a read, and the picture queue is in memory.
- The added core lines name no platform (grep for shopify, pricing and `gid:` in the core diff: none).
- The existing Sessions-page tests changed only the one mock.

**Hard constraints (step 5).**
- H1: opening a row reads Shopify and writes only locally. The list writes nothing to Shopify or Pricing.
- H4: a second open answers the same session.
- H6: without commerce there is no picture, and `open_product` passes the adopt error through.
- H7: core carries the `commerce_id` from the provider port and no platform names.
- H8: commands are thin and the logic lives in `LabelsService`.
- H2, H3 and H5 are untouched.
- Secrets and live services: none. Tests use `FakePricing` and `FakeCommerce`.

**Assumptions (step 4).** I read all 10 against 09C §5 and MC-L1. None contradicts them.
- Assumption 7 (entering the box does not open the 2 s window) is a reading of MC-L1.9 that allows at most one extra immediate request after entering.
- Assumption 1 (SKU from the last list answer) is MC-L1.8's rule 3 given an id-only command. Its rare edge is a note.
- All 10 are in the narrative.

## Findings
None blocking (B1–B4: none).

## Notes written to backlog
These are in `docs/implementation/backlog_labels/L2_labels_box.md`, the milestone's reviewer-notes file named by the plan and by `master_plan_labels.md`.
1. **The Labels box keeps its rows after the key goes.** `key_changed()` still emits no `LabelsChanged`. This is the open L1 r1 ACC.3 note, now also visible in the box. Route: orchestrator / L3.
2. **Invisible spaces in a test.** `l2_label_words.test.ts` writes U+2009 and U+00A0 as invisible literal characters, against the plan's U2 note to use escapes.
3. **A C10(b) sub-check that cannot fail.** The "no To review" check cannot fail in its fixture: nothing waits for review, so the button would be absent in Open too.
4. **`open_product`'s SKU match depends on memory.** It uses the SKUs seen since the last connect, so a Test connection while the box is open can turn a SKU-only match into `SkuCollision`.
5. **Resolved:** L1 r1 note 3 and L1 r2 note 5 (stale `useSessionLabels` ids) are closed by this phase's `useSessionLabels.ts` change.

## Lessons for the plans
- The prompt template names `docs/implementation/backlog.md` for notes. The labels milestone uses `backlog_labels/<milestone>.md`. The reviewer prompt for milestone tracks should name the milestone file.
- The L1 ACC.3 note ("fold into ACC.3 before L2") was not folded. L3's plan should either take it or record its refusal.

## Write perimeter (this session)
- `docs/implementation/plans/L2_labels_box.md`: the Tracker line → APPROVED, and a Review log entry appended.
- `docs/implementation/backlog_labels/L2_labels_box.md`: new.
- `docs/implementation/narratives/L2_labels_box.md`: new.
- This handoff.

No production or test file was edited. There were no mutation probes (flow §4), no DB or state side effects, and no commits (the runner commits). There is no architecture graph in this repo.
