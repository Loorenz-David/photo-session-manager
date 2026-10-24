# Session Manager — Intention (book map)

**Status: READY_FOR_RATIFICATION** (since 2026-10-24; awaiting the owner's explicit act)
Owner: David Loorenz. Shaper: Claude (intention-shaper doctrine). Book created 2026-10-24,
replacing the single-file draft of 2026-10-23; backend revised to Rust/Tauri the same day.

## How to read this book

The intention is split into chapters so a downstream session reads only what its task touches.
**This map is the authority on what exists; each chapter is the authority on its subject.** Nothing
is stated in two chapters. When a chapter cites another, it cites it by file name. Chapters are
never renumbered; amendments add lettered chapters (`07A_…`).

Every session reads this map, `book/01_objective_and_constraints.md`, and
`book/14_measurement_ledger.md`. Then only the rows below that match its work.

| Chapter | Read when your task touches… |
|---|---|
| [01 Objective and constraints](book/01_objective_and_constraints.md) | anything (always read) |
| [02 Product direction](book/02_product_direction.md) | architecture boundaries, "is this MVP?", commercial future |
| [03 Grounding](book/03_grounding.md) | the stack, the two Macs, where evidence lives |
| [04 Workflow and scenarios](book/04_workflow_and_scenarios.md) | user-facing behaviour, acceptance stories |
| [05 Domain model](book/05_domain_model.md) | tables, fields, ownership, facts vs derived |
| [06 Photo review and ordering](book/06_photo_review_and_ordering.md) | capture arrival, approve/condition/delete, undo, order, Original/Adjusted, unassigned, move/copy |
| [07 Session states and publishing](book/07_session_states_and_publishing.md) | status order, ready rule, dirty state, publish steps, failures, local export |
| [08 Attributes and AI](book/08_attributes_and_ai.md) | attribute schema, field kinds, customize fields, AI suggestions and provenance |
| [09 Integrations](book/09_integrations.md) | commerce integration interface, capabilities, the Shopify adapter |
| [10 Camera and editing providers](book/10_imaging_providers.md) | camera adapter, Capture One events and commands, live previews, rendering |
| [11 Architecture and packaging](book/11_architecture_and_packaging.md) | Tauri + Rust, layers, commands/events, repo layout, packaging spike, phone remote |
| [12 Operations](book/12_operations.md) | settings, logs, backup, health |
| [13 Scope ladder](book/13_scope_ladder.md) | must ship / only if cheap / deferred |
| [14 Measurement ledger](book/14_measurement_ledger.md) | any criterion or test (always read) |
| [15 Decisions](book/15_decisions.md) | resolved decisions and the open owner cards |
| [16 Changelog](book/16_changelog.md) | shaping history |

Evidence (source-contract facts, beside the book):
- `evidence/capture_one_16.8.6.md` — AppleScript dictionary findings and runtime VERIFY items.
- `evidence/design_handoff.md` — rules extracted from the Claude Design handoff in `claude_design/`.
- `evidence/shopify_admin.md` — written when the dev store is provided (not yet).

Stages of the brief are referred to as **S0–S8** (`M` is reserved for measurements).
