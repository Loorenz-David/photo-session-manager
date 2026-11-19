---
plan: W2_writer_engine
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-19
actor: claude-opus-5.5 (pipeline-reviewer)
---
# W2_writer_engine — review r1: APPROVED_WITH_NOTES

First review of a five-unit milestone. The gate passed: the Tracker line read IMPLEMENTED, and all five unit checkpoints plus the closing checkpoint (`e7481f3`, `66ac61b`, `7a2724a`, `c796d2b`, `5c6067c`, `a91d440`) are in `git log`. No B1–B4 finding. There are 9 notes, all in `docs/implementation/backlog_writer/W2_writer_engine.md`.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
**Perimeter.** `git diff --stat e7481f3^ a91d440` lists the same 34 paths the handoff declares: new `writer/{config,defaults,input,postcheck,run,safeguard,schema,writes}.rs`; new engine stems `w2_writer_run.rs`, `w2_writer_writes.rs`, `w2_safeguard.rs`; edited research files `service.rs, store.rs, domain.rs, config.rs, runs.rs, stages.rs, rules/{cost,run_rules}.rs, adapters/openai/schema.rs, api_impl.rs, export.rs, writes.rs, writer/{mod,api}.rs`; `tests/engine/{harness,main}.rs`; app files `integrations/src/lib.rs` (one line), `infra/src/sqlite/session_delete.rs`, `testkit/tests/suite/{p01_storage_identity,a17_delete_session}.rs`; `docs/guide/02_architecture.md`; the plan; the handoff. Nothing extra, nothing missing. Tree clean at `2ee34aa` when the checks ran.

**Checks** (Review mode, each run once on clean `2ee34aa`):
| Command | Result |
|---|---|
| `cargo test -p psm-research` | lib green, engine 328 passed, doctest 1 passed, 0 failed (1 min 18 s) |
| `cargo test -p psm-testkit --test suite -- p01_storage_identity:: a17_delete_session::` | 28 passed |
| `cargo clippy -p psm-research -p psm-infra -p psm-integrations -p psm-testkit --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | exit 0 |
The implementer's full-suite stamp (`cargo test --workspace`, workspace clippy, fmt) stands. Nothing contradicts it.

**Rows, by reading the production code and then its test:**
- **U1 (C1–C4).**
  - `WriterSettings::default()` serialises to master plan §4's list field by field.
  - `from_slot` merges valid keys over the defaults and skips a key with the wrong shape (C2(a)).
  - `validate` errors start with the field's name (C2(b), five cases plus the default accepted).
  - `INSTRUCTION_EN` and `INSTRUCTION_SV` were compared mechanically against the blockquotes of §9.1 and §9.2: byte-equal.
  - `EXAMPLES_EN` and `EXAMPLES_SV` were read against evidence §8: equal, with exactly §9.4's changes.
  - `visible_keys` matches WC2: the saved list ∩ layout in layout order, else Details+Condition minus money/media_list/reference, and the writer's own keys are never visible.
  - `input::build` matches WC2. `value` is null only for Empty/Dismissed. `fact` is false for Empty, Dismissed and Unchecked-low. `listings` take Match first, then Similar, in stage 3 order, at most 8, with verdicts from stage 4. `answers` are the answered findings by `resolved_at`. `rejected` is `cleared_set`, deduplicated canonically. `texts` = `requestable_texts`. `shop_name` is omitted when blank.
  - Each C4 test asserts the exact JSON of its key.
- **U2 (C5–C11).**
  - `check` implements WC5 items 1–10:
    - characters are scalar values; words are tokens with a letter or digit;
    - banned words match on word boundaries, phrases as substrings, and the iconic/ikonisk/unik exemption needs a fact;
    - em dash, an en dash unless digits stand on both sides, and " - ";
    - "!" and "?";
    - the English patterns are matched per sentence, with "later" checked by position;
    - use-marker sentences are counted;
    - damage terms come from `damage_terms`, else from the label words (≥ 4 letters, `es`/`s` stripped), mapped for Swedish;
    - the report must start with the grade, mapped for Swedish;
    - stability words;
    - names by `N` substring over the facts and notes; standalone 18/19/20xx years; model tokens, excepting the dimension letters and the Dimensions/Mått line.
  - All 36 tests assert the exact list of hit codes on an otherwise clean text.
  - C11(a) runs the four shipped examples with no hit.
- **U3 (C12–C15).**
  - `request_decision` applies WC3 1–6 in order. `writer_request` builds its input in one transaction. Research-busy excludes only `browse` and `write`. `writer_block` checks integration → key → price for (resolved provider, `writer.model`) → requestable texts.
  - Start: tier 0 is `manual`/`write`/`look`. `may_start` returns Yes for `write` after the blocked/running/day-limit checks, so there is no source wait. A `write` run is blocked by `writer_block`.
  - The start transaction stores the `writer` snapshot (resolved instruction, examples, origin moved from `writer_origin`, settings) and `stages.write`, with `input_snapshot` = `writer_input`.
  - `schema_for(Stage::Write)` derives `requested` from the request's input.
  - `next_call` treats `Write` like `Decide`.
  - `Stage::Write` = 8; it is not in `ALL`, and `parse` resolves it.
  - Tests: C12(a)–(h) assert the answer and the write runs; C13(b) and (c) assert the run state and the `jobs` rows; C15(a) and (b) assert the snapshot.
- **U4 (C16–C20).**
  - `writer::run::call` is the only builder of a writer request, and it sets `images` and `listing_images` to `Vec::new()` (Review-mode check: no builder can attach an image).
  - `texts_of` keeps only the requested names. The retry asks only for the texts with hits, naming them in both the instructions and the input. After a retry, every text with a hit is `low` with "Check: …". When the retry is Skipped, `NotRechecked` is added and the run ends `stopped_at_cap`.
  - `write_text` refuses any key outside `allowed`, which is the snapshot's requested names mapped to keys (Review-mode check: the write path cannot reach a key outside the requestable texts). It reads `stored_state`, then `cleared_set`, then `decide_write`, unchanged. It runs the same-content check, writes `ai` + `suggested` with `ai_sources` [], and never writes `items.title`.
  - Writes, steps, findings and the `result` output commit in one `research_transaction`. A run whose `result` is committed returns straight away on resume.
  - Tests: C16(b) and C18(a)–(i) assert rows, priors and step text; C20(a) asserts a single request and the same end state as an uninterrupted run; C20(b) asserts `updated_at` unchanged and outcomes "unchanged".
- **U5 (C21–C25).**
  - `filter` applies WC6 1–6. Keys are cut to layout keys. A type in mode off is dropped. Subject = K, or {tf} for text types. A finding is dropped when a subject key is `reviewed`. `field_vs_photos` needs all keys undecided. Field types need K ≠ ∅. Identity is `type|K|tf`, and a question's is `question|K||N(text)`. Findings already resolved or repeated in the run are dropped. The cap is applied in type order, then identity, then questions.
  - `store_findings` reads the key states inside the `result` transaction, calls `filter`, supersedes the item's open rows, then INSERTs the survivors. This is the only INSERT into `writer_findings` in production code (Review-mode check: `filter` runs before any insert).
  - Answer sets `answered` with a timestamp, then `writer_request(Answer)`. Dismiss writes no run. The counts cover all five types.
  - Export: `writes.json` is the `result` unit for a `write` run and null for a `look` run.
  - `session_delete` deletes the session's `writer_findings` when the table exists. p01 lists the table.
  - C22–C25 assert the stored rows, call rows, costs, day spend (`first_failing` flips at 0.014 / 0.014001), no outbound row, the export files, and that nothing dangles after delete.

**Assumptions** (handoff 1–18). None contradicts 08C (HW1–HW7) or WC2–WC7. Assumption 8 (`stopped_at_cap` counts as the automatic writing) is WC3.4's own list. Assumption 7 (findings come only from the first answer) fits 08C §5.3–§5.4: the rewrite is a text fix. All 18 are reported in plain words in the narrative.

**Deviations.**
- 1 (C18(f)) follows WC4.4 and 08B MC-A1, which define rich-text sameness as canonical equality, over the row's wording. That is a plan defect, recorded as a note and a lesson.
- 2 (table created from U1 in `store::TABLES`, run by `schema::ensure`), 3 (dispatch in `stages::execute`), 4 (`same_content` made `pub(crate)`, the shared helper the plan allows) and 5 (guide update) are sound.

**Hard constraints.**
- H1: no platform write; the title is never written to `items.title`.
- H2 and H3: no file or pixel work.
- H4: resume and the second application are idempotent (C20(a), (b)).
- H5: no UI automation of other apps.
- H6 and HW7: no integration → refused with a reason, and nothing else changes.
- H7: the writer table holds no platform concept. The "shopify" title-base literal lives in the research crate, not in core; recorded as a note.
- H8: the engine has no Tauri dependency.
- No secrets, no live services.

**Review history.** Round 1; no earlier rounds.

## Findings
None blocking (B1–B4). No finding carries a route.

## Notes written to backlog
`docs/implementation/backlog_writer/W2_writer_engine.md`. The plan's Review mode and master_plan_writer §3 name this track's backlog path, which W1 used too. The prompt's generic `backlog.md` was not used.

| Note | Destination |
|---|---|
| Text findings raised for a draft that was refused (typed text, no feedback → not `reviewed`) | planner (WC6) before W3 |
| WC5.10(iii) reads "A 1 cm …" at a sentence start as a model number | planner (WC5.10) before W3 |
| C18(f) row vs rich-text sameness (case-only variants of a dismissed description are suggested again) | planner/owner, only if wanted |
| "shopify" literal in three places (raw literal in `run.rs`) | W3 |
| Guide 04 row "research runs list" still says only `browse` (carried from W1) | W3 (guide task) |
| An answer while research is busy queues nothing; W3 should show the reason | W3 |
| Listings from a failed latest research run (assumption 4) | planner (WC2), low |
| `w2_a_writer_run_waits_for_no_photo_source` is outside Core mechanisms | none (recorded) |
| C21(b) first half has two sufficient causes | none (recorded) |

## Lessons for the plans
1. A row's example input must respect the contract's own sameness table. C18(f) wrote an `N`-match against a `rich_text` field.
2. WC6 should state whether a finding about a text needs that text to have been written.
3. A regex contract over free prose needs a sentence-start row. WC5.10(iii) catches "A 1 cm".

## Write perimeter
- `docs/implementation/plans/W2_writer_engine.md`: the Tracker line, and the Review log entry "review r1".
- `docs/implementation/backlog_writer/W2_writer_engine.md`: new.
- `docs/implementation/narratives/W2_writer_engine.md`: new.
- This handoff.
- Scratch outside the repo: `/tmp/w2rev/verb.py`, the instruction byte-compare script.
- No production or test file was touched. No probes; flow §4 has no mutation regime. No database or tool state.
- `docs/implementation/master_plan_writer.md`, `plans/W5_seo_publish.md` and `plans/W6_restoration_choices.md` were modified on disk by another actor during this session (W6 owner card 1 answered). They are not mine and were left untouched.
