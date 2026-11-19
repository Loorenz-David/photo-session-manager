# Backlog — W6_restoration_choices (reviewer notes, never blocking)

Written by the review r1 (2026-11-19, Opus). Each line: what, why it matters, where it should land.

- **"soiled" counts as stating "Oiled".** Rule 11 matches a phrase as a substring (WC5's phrase rule), so a
  condition report saying "slightly soiled seat" passes the Oiled check without saying the piece was oiled. Other
  phrases are safe in practice (e.g. "limmad" inside "omlimmad" still means re-glued). A wrong pass lets a text skip
  a restoration step the owner ticked. → the WC8-R phrase table or a word-boundary match for rule 11 (master plan
  writer §5), with the next edit of `postcheck.rs`.
- **The default instruction does not ask for each step in the condition report.** The condition-report paragraph says
  "Then the restoration in one sentence"; "naming each restoration step exactly as the fields give it" is only in the
  description paragraph. Rule 11 checks the report only (owner card 1), so a model that follows the instruction
  literally can earn a Restoration hit and a paid retry, or a flagged text. → master plan writer §5 instruction
  text (`writer::defaults` instruction), orchestrator; a one-line wording change.
- **The 14 restoration choices are furniture-workshop constants in a core module** (`RESTORATION_CHOICES` in
  `crates/core/src/services/attributes.rs`). Not an H7 breach (no device, app or platform concept), and the plan puts
  them there, but a second business with other trades needs a code change. → the later build that makes the choices
  editable ("more can be added in a later build").
- **The plan's unit test commands are not valid cargo syntax.** `cargo test … writer::postcheck rules::write_table`
  passes two positional filters; cargo needs them after `--` (`cargo test -p psm-research --lib -- writer::postcheck
  rules::write_table`). The implementer ran each filter separately. → planner template for writer-track plans.
- **The W6 tracker line said actor `codex` while the implementer was Claude Opus** (handoff frontmatter). Record
  only; the reviewer's APPROVED line replaces it. → none (recorded).
