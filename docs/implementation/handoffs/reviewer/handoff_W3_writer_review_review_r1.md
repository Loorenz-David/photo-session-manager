---
plan: W3_writer_review
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-19
actor: opus (pipeline-reviewer)
---
# Handoff: W3_writer_review, review r1

**Gate:** passed. The plan's Tracker line read `IMPLEMENTED`. Checkpoints `79eb78f` (U1), `e4f7e3e` (U2), `f2e18ac` (U3) and closing `c5a833e` are all in `git log`.

## Verified correct
- **Perimeter.** `git diff --stat 79eb78f^ f2e18ac` lists 32 paths. That equals the handoff's per-unit Files changed lists, and includes the three declared deviations (`crates/core/src/services/settings.rs`, `crates/research/src/writer/run.rs`, `crates/research/src/api_impl.rs`). `App.tsx` was listed in the plan and left unchanged, as declared. The closing commit touches only the plan and the implementer handoff.
- **Checks** (once, the plan's Review mode, tree `43c17cc`): all green.
  - `cargo test -p psm-testkit --test suite research::`: 99 passed.
  - `cargo test -p psm-core mechanisms::publish_partition`: 4 passed.
  - `cargo test -p psm-desktop`: 52 passed.
  - `npx vitest run src/features/form src/features/sessions src/features/shell`: 181 passed.
  - `cargo clippy` on core, infra, integrations, research, desktop and testkit (`--all-targets -D warnings`), `cargo fmt --all -- --check`, `npm run typecheck`.
  - The implementer's full-suite stamp (1818 Rust / 648 UI on `f2e18ac`) stands.
- **Criterion rows.** For all 30 rows I read the production code that makes the row true, then its test. Each test observes the row's outcome at the row's boundary and would fail on a wrong outcome:
  - **C1–C3:** the form, `items.title`, rows, feedback, undo/redo, the publish preview and the fake shop's calls.
  - **C4–C6:** the prepared settings kind, the file bytes, run rows, status fields against the run row, and a real Tauri mock invoke.
  - **C7–C10:** service calls with their order, toast text, DOM order, rendered text.
  - **Discriminating points I checked:**
    - C2(b) proves the cleared set with a real writer run. Without the cleared set, the linked title's synthetic `integration` stored state would let the identical title through.
    - C3(a) fails without the built-in definitions, because the description would be left out instead of becoming a conflict.
    - C7(a) proves "Next to review" still moves on when the request rejects.
- **Review-mode reads.**
  - (1) `items.title` is written only by `title_follows` (accept / Use suggestion / accept_all), `write_title` (owner typing), `clear_title` (owner clear), `read_integration_values` (the shop's own title) and history undo/redo. The writer writes only the `product.title` row.
  - (2) `partition` filters every `product.title` row before any rule, so a title row is never frozen, withheld or pending.
  - (3) `requestTexts` is synchronous fire-and-forget with a sync `try` and a `.catch`. `onNext` is called unconditionally after it, in both `AiReviewTab` and `Header`.
- **Undo coupling (plan hazard).** Accept records the row change and the `FieldState::Title` change in one `Act`. `history::apply` checks the title beside its row, so undo/redo restore both or refuse both.
- **Assumptions 1–10.** I read them against 08C §5–§6 and WC3, WC4.7 and WC4.8. None contradicts the intention. Assumption 8 changes no backend action: Use suggestion and Accept both call `AttributesService::accept`.
- **Hard constraints.**
  - **H1:** a title suggestion never reaches the shop unaccepted (C3(b)).
  - **H4:** each action is one transaction.
  - **H7:** only the existing built-in `product.title` key appears in core.
  - **H8:** the four commands are thin delegations.
  - **H2, H3, H5, H6:** untouched.
  - **Tests:** no live service and no secret in any test.
  - **HW1/HW7:** the writer writes no `items.title`. Without a key it says why, and publish still works (C6(b)).
- **Guide (flow §7A).** `docs/guide/01–05` are updated for the writer, the title coupling, the partition, the commands and the `writer` settings field.

## Findings
None blocking (B1–B4). No test is implementation-coupled under the over-engineering test.

## Notes written to backlog
Seven notes, in `docs/implementation/backlog_writer/W3_writer_review.md`, the backlog this plan's Review mode names:

| Note | Destination |
|---|---|
| Answer refused `ResearchBusy` shows no reason (carried from W2) | W4 |
| C5(a) observes two of the row's three texts (condition report not in the fixture) | none (recorded) |
| Sub-cent costs show "$0.00" | W4 |
| Guide row on the research runs list still stale (carried from W1/W2) | W4 |
| "shopify" literal in three writer places (carried from W2) | W5 / tidy |
| `answer`-origin refusal untested at the command layer | none (recorded) |
| A batch's "Confirm & next" also fires `all_checked` (harmless, WC3.4 lets one through) | none (recorded) |

**Lessons for the plans:**
- W2's carry-forward notes addressed to W3 (the answer's refusal reason, the guide row, the literal) were not folded into W3's tasks, so the implementer never saw them. The coordinator should copy notes routed to a phase into that phase's plan or prompt.
- Row C5(a) names "the three text suggestions" while the plan's fixture guidance does not require a condition report in the layout. Future rows should name the fixture that makes every listed outcome reachable.

## Write perimeter (this review)
- `docs/implementation/plans/W3_writer_review.md`: the Tracker line → APPROVED, and the Review log entry "review r1".
- `docs/implementation/backlog_writer/W3_writer_review.md` (new).
- `docs/implementation/narratives/W3_writer_review.md` (new).
- This handoff (new).
- Mutation probes: none (flow §4). No production or test file was edited. No database or state side effects.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
