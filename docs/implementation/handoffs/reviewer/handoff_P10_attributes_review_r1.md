---
plan: P10_attributes
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-25
actor: opus
---
# P10 attributes: review r1

**Verdict: APPROVED_WITH_NOTES.** The production code satisfies all 31 criterion rows at their boundaries, and each
row's test can fail on a wrong outcome. Nothing blocks under flow §5 (B1–B4). 7 notes went to the backlog. The most
important one is for P11/P14: the built-in Title field is not yet the item's title.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. The title binding (note 1) is answered by the book (ch05 `items.title` is the product title; ch08 "Title is a
Product-step field"). The orchestrator should fold it into P11/P14, not ask the owner.

## Verified correct
- **Gate.** Tracker row `IMPLEMENTED`. Checkpoint `51b99f6` is in `git log`. HEAD `eed32c2` adds only the implement
  handoff and review prompt, and `git status --porcelain` was clean at entry and after my probes.
- **Perimeter.** `git show --stat 51b99f6` lists 22 paths. They equal the handoff's "Files changed" list exactly: 6 new,
  14 modified, 2 docs. The paths outside the plan's expected files are declared deviations: one `prefill_defaults` call
  in each of `ItemsService::create_local` and `SessionsService::create`, both inside the existing creation transaction
  (needed for C6(a)), plus `serde_json` in `psm-desktop` (1 lockfile line). The rest is module wiring.
- **Stamp (B4)** on the checkpoint tree with the master plan §7.2 commands, all exit 0: `cargo test --workspace` (P10
  31/31, every other target green), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --
  --check`, `npm run typecheck`, `npm test` (3 files, 45/45). The bindings regenerated during the suite left no diff.
- **C1(a)–(p)** `mechanisms::values::canonicalize` implements the MC-9 table for each type:
  - text, textarea and url: NFC plus `trim_end`.
  - color: `#` plus 6 hex digits, lowercased.
  - rich_text: paragraphs joined by exactly one blank line.
  - integer: `is_i64` only. decimal: a strict `-?\d+(\.\d+)?` parse with trailing zeros dropped.
  - money: exact keys, 3-letter currency upper-cased. dimension: the definition's units.
  - boolean: JSON booleans only. date: `YYYY-MM-DD` that parses.
  - rating: three decimal strings.
  - choice: exact, case-sensitive membership. multi_choice and tags: first-kept de-duplication.
  - media_list: an integer exclusion list. reference: opaque.

  Objects get sorted keys and compact JSON. Each test reads the stored `(value, valid)` straight from SQLite and
  compares it exactly. Invalid inputs are asserted verbatim with `valid = 0`, so a wrong form, a coercion (`1` for
  `"1"`, a float decimal) or a wrong validity flag reddens the row.
- **C2(a)/(b)** `"minor"` is invalid for the severity field and stored verbatim. A human `"Minor"` then gives `valid = 1`
  and `human`.
- **C3(a)** `write` compares the full stored content (`same_content`) and returns before `put_attribute_value` and
  `set_item_activity`. The test advances the clock and asserts that `updated_at` and `last_activity_at` both stay at T0,
  so a write would fail it.
- **C4(a)–(e)** `ai_suggest` refuses any row whose provenance is `human`, and the test asserts both the `InvalidValue`
  and the unchanged row. A suggestion on an empty field gives `ai/suggested/medium`. `accept_all` accepts only rows that
  are `ai` + `suggested` + not `low` (the test checks all three statuses). `clear` sets `cleared`, and `values()`,
  `form_get` and the export all filter the row out.
- **C5(a)–(c)** `form_get` takes the item's session's condition photos sorted by `(position, id)`, minus the valid
  exclusions (`selected_condition_photos`). A later condition photo is selected. An excluded id that became main is
  ignored, and the stored value is asserted unchanged.
- **C6(a)/(b)** The preset layout drives order. `prefill_defaults` writes canonical defaults with provenance `default`
  for missing keys, and `is_default` is derived from that provenance. A Product layout without title, description or
  tags is refused before any write, and the form still lists all three.
- **C7(a)** `set_item_activity` runs `UPDATE sessions SET last_activity_at` only. The test asserts `(t, n)` with `n > 0`
  from a real classify.
- **C8(a)** The export snapshot keeps rows that are valid and live, and `item_json` receives them. The test asserts that
  `attributes` equals exactly `{"custom.years": …}`, with the invalid `1` omitted.
- **Variation (reviewer scratch test, run once and deleted; tree clean afterwards).**
  - Condition order follows position: p3 moved to condition position 1 → `[p3, p1, p2]`.
  - With `integration = None`, the form totals are 0/0/3 and a `custom.*` key is refused.
  - A preset default `"Wegner "` pre-fills as canonical `"Wegner"`, only on a new item of that type.
  - AI may replace a default, and once a human sets the value, AI is refused.
- **Hard constraints H1–H8.**
  - H1, H2, H3, H5: no platform write, file move or delete, pixel work, or UI automation was added.
  - H4: every write is one transaction, and an equal value is not written.
  - H6: everything runs with no integration.
  - H7: core and infra additions name no platform. The only keys are the generic `product.*` built-ins, and the
    condition grouping arrives on the definition.
  - H8: the commands are DTO/JSON-text wrappers over `AttributesService`. The integration kind is read from settings
    the same way P09's `integration_get` does.
  - No secrets and no live services in the tests.
- **Assumptions (handoff §Assumptions).** None contradicts the book at a boundary users can observe today.
  - 2 (`product.title` ≠ `items.title`): it diverges from ch05/ch08's single title, but no user can reach it before
    P11. Backlog note 1, routed to P11/P14.
  - 3 (url trimmed): this follows plan row C1(c) over MC-9's literal table. It is benign because canonicalization is
    applied on both sides of every comparison (M16/M20 hold). It is a plan/book disagreement, not a production defect,
    so it is backlog note 2 and goes upstream.
  - 1, 4–16: consistent with the book. They are in the narrative in plain words.

## Findings
None blocking (B1–B4). No orphan tests: the 31 boundary tests map one-to-one to rows, and the 11 unit tests in
`values.rs` cover only the plan's three Core mechanisms.

## Notes written to backlog
1. **P10 → P11, P14:** `product.title` neither reads nor writes `items.title`. The form shows an empty Title for an
   item created as "Oak", and once P11 edits it, `item.json` could carry two different titles. Bind it to `items.title`
   before the form ships.
2. **P10 → plan/book:** C1(c) trims url, but MC-9 lists only text and textarea. Fold one way (recommended: add url to
   MC-9's trimmed types).
3. **P10 → P11, P14:** a human write to a non-writable `reference` field is accepted and stored `valid = 1`. P11 must
   render it read-only and P14 must write only writable keys.
4. **P10 (test):** C1(p)'s `writable = false` comes from the fixture, not from `import_definitions`. The production
   code forces it (verified by reading). P12's import row should feed a reference marked writable.
5. **P10 (test):** C5 uses photos whose id order equals their position order. The reviewer probe confirms position
   order. Plan lesson.
6. **P10 → P14:** `item.json` exports the damage report as local photo row ids (this conforms to MC-8, but the ids mean
   nothing outside the database).
7. **P10:** tags and multi-choice values are not NFC-normalized (assumption 6), so a rare duplicate tag is possible.

## Lessons for the plans
- Order and selection rows need a reorder in their input. This is the second phase in a row (P09, P10) whose order
  rows cannot tell id order from position order.
- A row that restates a contract table should quote the table. C1(c) paraphrased MC-9 and widened it to url.
- A built-in field that shadows an existing column (`product.title` / `items.title`) must name its home in the plan.

## Write perimeter (this session)
- `docs/implementation/master_plan.md`: P10 tracker row only (`IMPLEMENTED` → `APPROVED`).
- `docs/implementation/plans/P10_attributes.md`: Review log entry appended.
- `docs/implementation/backlog.md`: 7 rows appended.
- `docs/implementation/narratives/P10_attributes.md`: new.
- This handoff: new.
- Probe file `crates/testkit/tests/zz_reviewer_probe_p10.rs`: created, run, deleted. It was never committed, and
  `git status` showed no residue. No production code was touched. Build artefacts under `target/` only.
- No commits (the runner commits).
