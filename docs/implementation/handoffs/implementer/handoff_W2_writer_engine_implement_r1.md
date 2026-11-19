---
plan: W2_writer_engine
role: implement
round: 1
state: DONE
date: 2026-11-19
actor: claude-opus-5.5 (pipeline-implementer)
---
# W2_writer_engine — implement r1

Five units built in order, each with its checkpoint; closing gate stamp below. Gate checked first:
the intention reads RATIFIED, `W1_product_note` is APPROVED, W2 was NOT_STARTED.

## What was built
The whole publication writer now exists inside the research engine, proven with its fakes. Nothing
in the app runs it yet (W3 wires the triggers, commands and panels).

- **Writer settings and the shipped writing guide (U1).** `WriterSettings` with the master plan's
  defaults (automatic writing on, English, `gpt-6.1-sol` at `medium`, the seven lengths, the five
  safeguard modes, 3 questions, no shop name, the condition report field). It reads a damaged
  slot tolerantly and refuses bad values with a message naming the field. The English and Swedish
  instructions, title formulas, dimensions lines and examples A–D are shipped exactly as master
  plan §9 states them. A saved instruction or example list replaces the shipped one.
- **What the writer sees (U1).** By default, every Details and Condition field except prices,
  media and references (or the owner's saved list, in layout order). Each field comes with its
  value, the owner's decision on it and whether it may be stated as a fact. The writer also gets
  the note as the owner's decisions rewrite it, the titles of matching listings (Match first, at
  most 8), the owner's earlier answers, the texts the owner dismissed (so it writes differently),
  the texts to write, the language, the lengths and the shop name when one is set.
- **The check of every text (U2).** Lengths, about 45 banned words and phrases in both languages
  plus the owner's own, dashes, "!" and "?", the common AI patterns, at most one sentence on use,
  every damage the condition fields list named in the report, the report opening with the grade
  word, a sentence on stability, and names, years and model numbers only from the fields and the
  note. All four shipped examples pass.
- **The run's life (U3).** "Write texts", "Next to review" and "All checked" ask for a writer run.
  It is refused while research or Look again is busy with the session, and refused (with the
  reason) when there is no shop, no model key or no price for the writer's model. Automatic
  writing happens once per session; a failed or cancelled run may be retried. Repeat presses keep
  one queued run. A writer run goes ahead of waiting research runs, needs no photo source, and
  respects the day limit. Its start fixes the settings, the instruction and examples actually sent
  and the exact input.
- **The call, the rewrite and the writes (U4).** One call with no image. A text that fails the
  check is rewritten once, with each failure named; a text that fails twice arrives marked `low`
  with "Check: …" in its reason. When the run's cost limit stops the rewrite, the text is flagged
  "not checked again: run cost limit". Texts are written as `ai` suggestions through research's
  write table, unchanged: never over a typed or accepted text, never a text the owner dismissed
  again. The shop's text is kept as the prior. A title becomes a `product.title` suggestion;
  `items.title` is never touched. A killed run resumes without a second call.
- **Safeguard, cost, record, export (U5).** Questions and notes are never raised about a field the
  owner decided, and never again once answered or dismissed. At most `max_questions` asks (extra
  asks become notes; extra questions are dropped and listed in a step). Each run's findings
  replace the item's earlier open ones. Answering re-queues the writer with the answer in its
  input; dismissing writes nothing; a counts query per type. Writer calls are research calls (same
  costing, same day spend, no outbound photo rows). Export includes writer runs with `writes.json`
  (look runs: null). Deleting a session removes its writer findings.

## Tests → criterion rows
Unit tests (`crates/research/src/…`): `writer::config::tests` C1(a), C1(b), C2(a), C2(b) ·
`writer::input::tests` C3(a), C3(b) · `writer::postcheck::tests` C5(a)–C11(b) (36 rows, one test
each, named `w2_c<n><row>_…`) · `adapters::openai::schema::tests::w2_c14b_…` C14(b) ·
`rules::cost::tests::w2_c14c_…` C14(c) · `rules::run_rules::tests::w2_c13a_…` C13(a) ·
`writer::safeguard::tests` C21(a)–(f) · plus the core-mechanism tables `writer::run::tests::wc3_…`
(request_decision), `writer::writes::tests::wc4_3_…` (stored_state), `rules::run_rules::tests::w2_a_writer_run_waits_for_no_photo_source`, `writer::safeguard::tests::wc6_5_…`.

Engine tests (`crates/research/tests/engine/`): `w2_writer_run.rs` C4(a)–(f), C12(a)–(h),
C13(b), C13(c), C14(a), C15(a), C15(b) · `w2_writer_writes.rs` C16(a)–C20(b) ·
`w2_safeguard.rs` C22(a)–C24(c). App suite: `crates/testkit/tests/suite/a17_delete_session.rs`
`w2_c25a_…` and the p01 table list (C25(a)).

## Assumptions
1. Default `instructions` and `examples` are empty maps (no saved text for any language).
2. `validate` also refuses an unknown length or safeguard name, a non-text effort, a
   `visible_keys` that is not a list of keys, and a blank `condition_report_key`.
3. `answers` are ordered by when they were answered (`resolved_at`, then id).
4. `listings` come from the session's latest research run whatever its state.
5. Instructions sent = the instruction, the title formula, "Dimensions line: …" and each example
   as "Example <name>" with Fields, Title, Description, Condition report, Search title, Search
   description lines.
6. The rewrite call sends the same instructions plus one sentence naming every hit; its input is
   the first input with `texts` = the texts with hits, `previous` (their earlier versions) and
   `hits`.
7. Findings and questions are taken from the first answer only (the rewrite is a text fix).
8. A run whose rewrite the cost limit stopped ends `stopped_at_cap` (its texts are written,
   flagged). This counts as the session's automatic writing, as WC3.4 lists `stopped_at_cap`.
9. An automatic request while a writer run is already queued answers `queued: true` and creates
   nothing.
10. The origin waits in the queued run's settings snapshot (`writer_origin`) until the start
    transaction moves it into `writer.origin`.
11. A model `missing_in_text` finding is about the condition report; a model `text_vs_facts`
    finding is about the first requested text key among its keys (else none).
12. `postcheck::Hit` carries an optional `key`: the Condition field of a `Damage` hit. It becomes
    that finding's key.
13. Sentences split after ".", "!" or "?" followed by any whitespace, so a paragraph break ends a
    sentence. Use markers are matched with punctuation read as spaces. The stability words are
    matched as substrings, per language.
14. Questions beyond the cap are listed in a `not_determined` step "N not asked (at most M per
    write)".
15. `answer` refuses a blank answer and a finding that is no longer open; it returns the
    `Requested` of the re-queue. `writer_counts` is a `ResearchService` method (W4 surfaces it);
    it covers the five types.
16. A writer run reuses no unit of an earlier run.
17. Step wording: "matches a text you dismissed", "already holds this text", "<Label>: written
    (medium|low)", "<Label>: no text".
18. `PriceMissing` names `research.providers.model` (resolved) and `writer.model` (WC3.3).

## Deviations
1. **C18(f), row input against its own contract.** The row has the writer answer "old" to a
   dismissed "Old" ("same `N`") and expects no write. `product.description` is a built-in
   `rich_text` field. WC4.4 and 08B MC-A1 define rich-text sameness as canonical equality, not
   `N`, and WC4.4 says the table is unchanged. Under the contract "old" ≠ "Old", so the row as
   written would fail. I implemented the contract and 08B, and the test uses an answer "Old" that
   is canonically equal to the dismissed text. To make case-only variants of a dismissed
   description count as dismissed, the contract (and 08B's sameness table) would have to change.
   That is a planner or owner choice, recorded here for the reviewer.
2. The `writer_findings` table is created from U1, in `store::TABLES`, which `schema::ensure`
   runs. Row C4(c) of U1 reads it; U5 listed `schema.rs`.
3. The writer run is dispatched in `stages::execute`, beside the look run, not in `job.rs`, which
   is unchanged.
4. `writes::same_content` is now `pub(crate)` (the shared helper the plan allows).
5. `docs/guide/02_architecture.md` gained the W2 modules and the new table (flow §7A).

## Files changed
**Write perimeter.**
- New: `crates/research/src/writer/{config.rs, defaults.rs, input.rs, postcheck.rs, run.rs,
  schema.rs, writes.rs, safeguard.rs}`, `crates/research/tests/engine/{w2_writer_run.rs,
  w2_writer_writes.rs, w2_safeguard.rs}`.
- Edited in the research crate: `writer/{mod.rs, api.rs}`, `service.rs`, `store.rs`,
  `domain.rs`, `config.rs`, `runs.rs`, `stages.rs`, `rules/{cost.rs, run_rules.rs}`,
  `adapters/openai/schema.rs`, `api_impl.rs`, `export.rs`, `writes.rs`,
  `tests/engine/{harness.rs, main.rs}`.
- App files: `crates/integrations/src/lib.rs` (one line, `writer_settings: Arc::new(|| None)`),
  `crates/infra/src/sqlite/session_delete.rs`, `crates/testkit/tests/suite/{p01_storage_identity.rs,
  a17_delete_session.rs}`.
- Docs: `docs/guide/02_architecture.md`, this plan's Tracker line and Review log, this handoff.

No mutation-probe files (flow §4). No tool-recorded state.

## Checkpoint SHA
U1 `e7481f3` · U2 `66ac61b` · U3 `7a2724a` · U4 `c796d2b` · U5 `5c6067c` · closing (tracker,
review log, handoff): the `CHECKPOINT (not approved): W2_writer_engine closing` commit after these.

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace` (once, after the last unit) | exit 0; 0 failed (suite 991 passed, research engine 328, research lib 249, …) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |

Per unit: each unit's **Unit test commands** ran green before its checkpoint. No TypeScript
changed, so no npm checks were needed.

## Manual try-out
Nothing in the app runs the writer yet (W3), so there is nothing to click. What can be seen:
1. Read the shipped writing guide in `docs/implementation/master_plan_writer.md` §9; the app
   ships exactly that text (`crates/research/src/writer/defaults.rs`).
2. In the repository, `cargo test -p psm-research --lib writer::` runs the settings, input rules,
   check of every text (including all four shipped examples passing) and the safeguard filter.
3. `cargo test -p psm-research --test engine w2_` runs whole writer runs on fake providers: one
   call, the rewrite, the writes and their refusals, questions never asked twice, cost and
   export.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
