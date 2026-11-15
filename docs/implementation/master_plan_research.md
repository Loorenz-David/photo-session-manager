# Session Manager — master plan, research agent part (v2, 2026-11-12)

Planner: Claude Fable 5.2 (`pipeline-planner`; `docs/process/flow.md` §10 names Fable for this track). Branch
`feature/agentic`. v1 (24 phase plans `A01`–`A24`, commit `d116654`) was restructured the same day on the owner's
decision recorded in flow §10: **an isolated crate, milestones as runner phases, two parallel tracks.**
**Companion to `docs/implementation/master_plan.md`, which stays the authority for everything not restated here.**
Every research session reads both files and flow §10. Where the two plans differ on a research name, this file wins.

## 0. Layout decision

| Thing | Where | Why |
|---|---|---|
| Milestone ids | `R1_research_benchmark` … `R13_research_chosen_fields` | cannot be confused with `main`'s `P…` or the superseded `A…`; the runner resolves every file from the id, so nothing collides. |
| Plan files | `docs/implementation/plans/R<n>_<slug>.md`, one per milestone, made of `## Unit` sections | one runner phase per milestone (flow §10). The runner is unchanged; three templates gained a few lines (§7.3). |
| Row ids | each criterion row keeps the id of the plan it was written in: `A07·C1(a)`; R11's rows are `A25·…`, R12's `A26·…` (v10), R13's `A27·…` (v12) | stable row identity across the regrouping; `docs/implementation/research_row_mapping.md` lists where every row went. Tests are named after the row (`a07_c1a_<slug>`). |
| Tracker | the **Tracker line at the top of each milestone plan file** | two tracks run at the same time in two worktrees; a shared tracker table would conflict at every merge. §4 below is only an index. `master_plan.md` §4 is not used. |
| Milestone list | `docs/implementation/phases_research.txt` | `phases.txt` is not touched. Driver loop in §5. |
| Reviewer notes | `docs/implementation/backlog_research/<milestone id>.md` | one file per milestone: no shared file between tracks, none shared with `main`'s `backlog.md`. |
| Engine code and its tests | `crates/research/` (package `psm-research`), tests **inside the crate**: inline unit tests for the rules, one integration test program `crates/research/tests/engine/` | flow §10 Isolation. A milestone that changes only this crate never runs the app suite. |
| Tool | `crates/research-tool/` (package `psm-research-tool`, bin `psm-research`), tests in `crates/research-tool/tests/tool/` | benchmark now, report and replay later. |
| App-seam tests | `crates/testkit/tests/suite/research/<stem>.rs`, one line `mod research;` in `tests/suite/main.rs` | only for behaviour that lives in app code (§5 Seams). |
| Storage | all research tables in the **app database file**; created by idempotent schema steps **outside the numbered migration list** | §6.4 (decision and reasons). `SqliteStore::migrations()` is never edited by this track, so `main`'s `014_…` and later cannot collide. |
| `master_plan.md`, `phases.txt`, `backlog.md` | not edited | merges with `main` touch no shared planning line. |

## 1. Goal
Build the research agent of `docs/intention/book/08A_research_agent.md` (RATIFIED 2026-11-12) under the exact rules of
`book/08B_research_contracts.md` (CONTRACTS RATIFIED 2026-11-12). This file never restates product semantics; where
it and the book differ, the book wins and this file is amended.

## 2. Sources of truth (additions to `master_plan.md` §2)
| Content | Artifact |
|---|---|
| Process for this track | `docs/process/flow.md` §10 |
| Meaning, HA1–HA6, stages, scope ladder | `book/08A_research_agent.md` |
| Exact rules MC-A1…MC-A12, R-IA1…R-IA26, probe ids RESEARCH-VERIFY-1…11 | `book/08B_research_contracts.md` |
| Measurements MA1–MA18 | `book/14_measurement_ledger.md` §Research agent |
| Provider facts, the nine trusted sites, probe and benchmark results, the owner's picks | `docs/intention/evidence/research_providers.md` |
| Test pieces and answer key | `docs/test-items/beyo_test_samples_1.json`, `docs/test-items/answer_key_corrections.json` |
| Sanitized provider captures | `crates/research/fixtures/<provider>/*.json`, written by R1, never by hand |
| Where each criterion row is | `docs/implementation/research_row_mapping.md` |
| Skeleton, sequencing, tracks | this file |
| Milestone goal / units / criteria / Tracker / Review log | `docs/implementation/plans/R<n>_<slug>.md` |

## 3. Standing rules (additions; `master_plan.md` §3 and `flow.md` apply, flow §10 first)
- **Trace.** Every row cites `MA1…MA18`, `MC-A1…MC-A12`, `HA1…HA6` or `H1…H8`. `[rule]` only for UI rendering and tool
  output rows. `[manual]` only in R1, R9, R12·U4 and R13·U4 (recorded in the evidence doc and the Review log, never asserted by a suite).
- **Rule 9.** No suite calls OpenAI, a search vendor, a listing site or the shop. Real calls happen only in R1, R9, R12·U4 and R13·U4 (task 3),
  from the tool, with keys from the environment.
- **Rule 17.** A fixture whose shape a provider owns is a sanitized capture written by R1, cited as
  `fixtures/<provider>/<file> (RESEARCH-VERIFY-n)`; such rows are only in R8. Every other row is written against our
  own ports and the research crate's fakes. RESEARCH-VERIFY-11 is grounded in §7.4.
- **Keys.** Benchmark keys only in `~/.psm_research.env` (mode 0600, outside the repo); the app's keys only in the
  Keychain through `ports::secrets`. No key, and no URL carrying one, in any log, fixture, evidence doc, handoff,
  recording or commit. Every recording is scanned before it is written (fail closed).
- **Spending ceilings.** R1: USD 50 (U1 15, U2 35). R9: USD 40 plus USD 2 for one replay. R12·U4: USD 20. R13·U4: USD 2. Enforced by the tool.
- **HA5 in code.** No model name, vendor name or reseller domain as a literal in `crates/core`. Literals live in
  `crates/research/src/{adapters, config.rs}` and the tool.
- **Isolation.** App crates use the research crate only through the surface of §6.6 (the `ResearchApi` trait and its
  shapes; four constructors called from `crates/integrations/src/lib.rs`). The testkit may also use
  `psm_research::{writes, fakes}` in tests, and (from R8, for the shop-facts and wiring rows) `ports::Providers`,
  `adapters::page_reader::HttpPageReader`, `rules::value_map::map_output` and `store` in tests. The research crate depends on `psm-core` and `psm-infra`, never on
  `psm-integrations` or the desktop crate; `psm-testkit` is a **dev**-dependency of it.
- **Scratch data only.** Until this branch is merged into `main`, the app built from it runs only with
  `PSM_DATA_DIR=$HOME/psm-research-dev` (or another scratch folder): it writes a publish step (`held`) and columns a
  `main` build does not know.
- **No `implementer: fable`.** **Projection** only for R5 and R8 (flow §10); R11, R12 and R13 carry `projection: no`.
- **Checkpoints.** One `CHECKPOINT (not approved):` commit after each unit, paths of that unit only.

## 4. Milestones (index; the state of each is the Tracker line in its plan file)
| Milestone | Units | Criteria / rows | Changes app files | Projection | depends_on | Blocked on |
|---|---|---|---|---|---|---|
| R1_research_benchmark | U1 probes and search benchmark (A01) · U2 first model comparison (A02) | 13 / 22 | no (workspace manifest only) | no | — | the owner's keys (§7.6) |
| R2_research_foundation | U1 name key and sameness in core · U2 storage seam and frozen shapes · U3 research-copy renderer | 9 / 23 | **yes** | no | — | — |
| R3_research_rules | U1 value mapping · U2 sites and fingerprints · U3 confidence · U4 cost and call rules · U5 write table | 31 / 132 | no | no | R2 | — |
| R4_research_engine | U1 ports, image guard, tables, fakes · U2 gateway · U3 stages and Decide · U4 run lifecycle · U5 resume, reuse, follow-ups · U6 research API | 39 / 99 | no | no | R3 | — |
| R5_research_app_seams | U1 feedback and undo · U2 publish hold | 12 / 34 | **yes** | **yes** | R3 | — |
| R6_research_ui | U1 panel · U2 Settings → Research · U3 publish review | 10 / 21 | **yes** (TypeScript only) | no | R5 | — |
| R7_research_wiring_and_export | U1 wiring · U2 retention and export | 10 / 17 | **yes** | no | R4, R6 | — |
| R8_research_real_adapters | U1 OpenAI · U2 visual search · U3 page reader and shop facts | 18 / 29 | **yes** (U3: Shopify adapter) | **yes** | R4, R1 | R1's evidence; the owner's two picks |
| R9_research_tool_and_acceptance | U1 report and replay · U2 acceptance measurement | 10 / 15 | no | no | R7, R8, R1 | owner-attended measurement; keys |
| R10_research_defaults | U1 defaults and reachability | 2 / 4 | no | no | R9 | the owner's model pick |
| R11_research_on_arranged | U1 the arranged trigger · U2 fields left alone, extra shop addresses, reset · U3 detailed condition derived, no attribution from another shop product · U4 session mark, Arranged — next, publish marks first · U5 button, tag, publish line, panel hint · U6 Settings pickers and reset | 33 / 99 | **yes** | no | R10 | — (one owner card on the detailed-condition table; not blocking) |
| R12_research_cost | U1 photo look in one call · U2 product part of a page · U3 same kind or no comparison · U4 second acceptance measurement | 16 / 37 | no | no | R11 | U4: owner-attended measurement; keys |
| R13_research_chosen_fields | U1 the list of fields research fills (engine and settings seam) · U2 detailed condition from the levels that have a value · U3 Settings: what the list means, warnings · U4 the tool's owner list and a check on a few pieces | 15 / 40 | **yes** | no | R12 | U4 task 3: the orchestrator's check on four pieces (USD 2); keys |

Totals, computed from the plan files: 154 criteria, 396 rows (393 from the superseded plans, 3 new freeze rows; a
criterion whose rows sit in two units is counted in both). Every unit has at most 8 criteria. R11 and R12 (v10) add
49 criteria and 136 rows, computed from their plan files; their rows are new (`A25·…`, `A26·…`). R13 (v12) adds
15 criteria and 40 rows (`A27·…`), computed from its plan file.

## 5. Sequencing, seams and the two tracks

### Dependency graph
```
R2 ──► R3 ──► R4 ──────────────► R7 ──► R9 ──► R10 ──► R11 ──► R12 ──► R13
        │      └──────► R8 ──────────────┘
        └──► R5 ──► R6 ──► R7
R1 (keys) ──────────► R8, R9
```
A milestone starts only when the Tracker line of every plan in its `depends_on` reads APPROVED.

### Frozen by R2 (after R2 is approved these change only through the orchestrator)
`psm_core::mechanisms::{name_key::name_key, sameness::same}` · `attribute::Value.ai_prior` and `AiPrior` ·
`psm_core::domain::research::{Feedback, FeedbackAction}` and the feedback store methods · the four `publish_items`
fields · `PublishStep::Held`, `RowLabel::WaitingForYou(n)` · `AppEvent::ResearchChanged` / Tauri event
`research_changed` · `CommerceAdapter::public_listing` and `PublicListing` · `Settings.research` (a JSON slot) ·
`SqliteStore::research_transaction` · `Imaging::render_research` · `psm_research::api` (the `ResearchApi` trait and
its shapes, §6.6) · the eight research commands and their generated bindings (`apps/desktop/src/services/bindings.ts`).
R5 adds two attribute commands and widens the publish preview and confirm; R6 (same track, after R5) builds on them.
**No other milestone regenerates `bindings.ts`,** except R7·U1 (two fields, v3) and R11·U4 (v10: two commands and the
shapes of §6.8).

### Seams: every place app code changes, and where
| Seam | App files | Milestone·unit |
|---|---|---|
| Values vocabulary | `crates/core/src/mechanisms/{name_key.rs, sameness.rs}` | R2·U1 |
| Storage | `crates/infra/src/sqlite/{ensure.rs, store.rs}` (schema step, `research_transaction`, `ai_prior`, feedback methods, publish columns); `crates/core/src/{domain/attribute.rs, domain/research.rs, ports/store.rs}` | R2·U2 |
| Research copy of a photo | `crates/core/src/ports/images.rs`, `crates/infra/src/imaging/research.rs` | R2·U3 |
| Shapes for publish, events, settings, shop | `crates/core/src/{domain/session.rs, domain/publish.rs, mechanisms/status.rs, events.rs, ports/commerce.rs}`, `crates/infra/src/settings_file.rs`, `crates/testkit/src/fake_commerce.rs` | R2·U2 |
| Commands (research) | `apps/desktop/src-tauri/src/{commands/research.rs, lib.rs, state.rs}`, `apps/desktop/src/services/{bindings.ts, research.ts, events.ts}` | R2·U2 |
| Attribute actions and undo (MC-A12) | `crates/core/src/services/{attributes.rs, history.rs}`, `commands/attributes.rs`, `FormDialog.tsx` (Looks right) | R5·U1 |
| Publish: withheld, conflicts, held (MC-A10) | `crates/core/src/{services/publish.rs, mechanisms/publish_partition.rs}`, `commands/publish.rs`, `PublishPopover.tsx` (pass-through) | R5·U2 |
| UI | `apps/desktop/src/features/{form, settings, publish, sessions}` | R6; the Export button R7·U2 |
| Wiring (job runner, service, call sites, settings save, purge schedule) | `crates/integrations/src/lib.rs`, the `session_changed` call sites, `crates/testkit/src/test_app.rs` | R7·U1, R7·U2 |
| Shop facts for Shopify | `crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs, type_map.rs}` | R8·U3 |
| The session mark and "Arranged — next" | `crates/core/src/{domain/session.rs, ports/store.rs, services/sessions.rs}`, `crates/infra/src/sqlite/{ensure.rs, store.rs}`, `crates/integrations/src/{arranged.rs, lib.rs}`, `apps/desktop/src-tauri/src/commands/{sessions.rs, publish.rs, research.rs, settings.rs}`, `bindings.ts` | R11·U4 |
| UI of R11 | `apps/desktop/src/features/{shell/Header.tsx, sessions/SessionRow.tsx, publish/PublishPopover.tsx, form/ResearchPanel.tsx, settings/{ResearchSection.tsx, ShopFieldPicker.tsx}}` | R11·U5, R11·U6 |
Everything else is in `crates/research` or `crates/research-tool`.

### Two-track schedule (at most two milestones at once, each in its own worktree with its own build folder)
| Step | Track A (`…/Photo-Session-Manager-agentic`, branch `feature/agentic`) | Track B (`…/Photo-Session-Manager-agentic-b`, branch `research/track-b`) |
|---|---|---|
| 0 | **R2** foundation | — (R1 here if the keys exist) |
| 1 | **R3** rules | **R1** benchmark, if the keys exist; otherwise idle |
| 2 | **R4** engine, then **R5** app seams and **R6** UI (v3: moved here, track B busy with R1) | **R1** benchmark |
| 3 | **R7** wiring and export | **R8** real adapters (needs R1 approved and the owner's two picks; otherwise R1, or idle) |
| 4 | **R9** tool and acceptance, then **R10** defaults | — |
| 5 | **R11** research on arranged, then **R12** research cost (v10: one track; each depends on the one before), then **R13** research fills chosen fields (v12) | — |

- **Create track B once**, after R2 is approved and committed on `feature/agentic`:
  `git worktree add ../Photo-Session-Manager-agentic-b -b research/track-b feature/agentic`, then
  `cd ../Photo-Session-Manager-agentic-b/apps/desktop && npm ci` (R5 and R6 need `node_modules`). A worktree has its
  own `target/` and its own `.pipeline/runs/`, so the two builds never wait on each other's lock. A second build
  folder costs tens of gigabytes; delete `../Photo-Session-Manager-agentic-b/target` when the track is idle.
- **Run a milestone in a worktree** from that worktree's root: `tools/pipeline/run_phase.sh phase <id>`.
- **Merge order.** Each milestone is merged the moment it is APPROVED, never two unmerged milestones on one branch:
  on track A's worktree `git merge --no-ff research/track-b` (after a B milestone), on track B's worktree
  `git merge --no-ff feature/agentic` (after an A milestone, and always before B starts its next milestone). So
  the order is: R2 · R3 and R1 (either order) · R5 · R6 and R4 (either order) · R7 and R8 (either order) · R9 · R10 · R11 · R12 · R13.
- **Expected conflicts, all one-line and mechanical:** workspace `Cargo.toml` `members` (R1 adds the tool crate, R2 the
  research crate) · `Cargo.lock` (take either side, then `cargo check --workspace`) · `crates/research/Cargo.toml`
  dependencies (R7 `zip`, R8 `reqwest` and a parser) · `crates/research/tests/engine/main.rs` and
  `crates/testkit/tests/suite/research/mod.rs` (`mod` lines added by both) · `crates/research/src/lib.rs` (`pub mod`
  lines). No other file is written by both tracks in the same step: track A's steps 1–2 write only `crates/research`,
  track B's steps 1–2 write only the tool crate, the fixtures folder, and app files.
- **Why R5 can run beside R4:** R5 needs from the research crate only `psm_research::writes` (R3·U5) to make
  suggestions in its tests; everything else it touches is app code frozen in shape by R2. **Why R6 can run beside R4:**
  it is TypeScript against generated bindings with the services mocked. **What cannot be parallel:** R7 needs both the
  engine (R4) and the screens (R6).

### Driver loop (one track; the runner is unchanged)
```
while read -r ph; do [[ -z "$ph" || "$ph" == \#* ]] && continue
  grep -q '"state":"APPROVED"' "docs/implementation/state/$ph.json" 2>/dev/null && continue
  tools/pipeline/run_phase.sh phase "$ph" || break
done < docs/implementation/phases_research.txt
```
`phases_research.txt` lists the milestones in single-track order; with two tracks the orchestrator starts each
milestone by id per the table above.

### Ledger coverage (every research measurement is served)
MA1 R3·U5, R4·U3 · MA2 R3·U3, R3·U5, R4·U3, R4·U4, R9·U2, R10 · MA3 R4·U1, R4·U2, R8·U2 · MA4 R4·U5, R7·U1 ·
MA5 R1, R3·U4, R4·U2, R4·U3, R10 · MA6 R3·U1, R4·U3 · MA7 R2·U2, R4·U4, R4·U6, R6·U1, R7·U1, R10 · MA8 R5·U2, R6·U3 ·
MA9 R7·U2, R9·U1 · MA10 R3·U5, R4·U3 · MA11 R2·U1, R3·U1–U3, R4·U3, R8·U3 · MA12 R2·U3, R4·U1, R4·U2, R8·U2 ·
MA13 R4·U1, R4·U4, R4·U5 · MA14 R2·U2, R3·U2, R3·U5, R4·U3, R4·U5 · MA15 R3·U4, R4·U2, R8 · MA16 R1, R3·U4, R4·U2, R4·U6,
R6·U2, R7·U2, R8, R9·U1 · MA17 R5·U2, R6·U3 · MA18 R5·U1. The operational measurement of 08A §11 is R9·U2.
v10 additions: MA2 R11·U3, R12·U3, R12·U4 · MA3, MA12 R12·U1 · MA4 R12·U1 · MA5 R12·U4 · MA6 R11·U2, R11·U3 · MA7 R11·U1,
R11·U2, R11·U4 · MA8, MA17 R11·U4 · MA9 R12·U1–U3 · MA10 R11·U2, R11·U3, R12·U4 · MA11 R11·U2, R11·U3, R12·U1, R12·U2 ·
MA13 R11·U1 · MA14 R12·U1 · MA16 R12·U1, R12·U3. The second operational measurement is R12·U4.
v12 additions: MA5 R13·U1 · MA6 R13·U1, R13·U2, R13·U4 · MA7 R13·U1 · MA10 R13·U1, R13·U2, R13·U4 · MA11 R13·U2, R13·U4.

## 6. Research skeleton and naming registry (new unless marked existing)

### 6.1 Layout
| Path | Role |
|---|---|
| `crates/research/src/api.rs` | `ResearchApi` and its shapes: the only thing app code calls (R2·U2) |
| `crates/research/src/rules/{mod.rs, value_map.rs, sites.rs, fingerprint.rs, confidence.rs, cost.rs, calls.rs, write_table.rs, run_rules.rs}` | pure rules (§6.5); inline unit tests |
| `crates/research/src/{domain.rs, config.rs, ports.rs, fakes.rs, images.rs, files.rs, schema.rs, store.rs, writes.rs, gateway.rs, service.rs, stages.rs, decide.rs, shop.rs, runs.rs, job.rs, api_impl.rs, retention.rs, export.rs}` | the engine (R3·U5, R4, R7·U2) |
| `crates/research/src/adapters/{mod.rs (build), http.rs, openai/, visual_search/, page_reader/}` | real adapters and their HTTP seam (R8) |
| `crates/research/fixtures/<provider>/` | sanitized captures (R1) |
| `crates/research/tests/engine/{main.rs, harness.rs, <stem>.rs}` | the crate's one integration test program; `autotests = false`; every file has its `mod` line in `main.rs` |
| `crates/research-tool/{src/, tests/tool/}` | the tool and its one test program |
| `crates/core/src/mechanisms/{name_key.rs, sameness.rs, publish_partition.rs}`, `crates/core/src/domain/research.rs` | the core pieces the app's own services need (R2·U1, R5·U2, R2·U2) |
| `crates/infra/src/sqlite/ensure.rs`, `crates/infra/src/imaging/research.rs` | schema step for app-owned columns; the renderer |
| `apps/desktop/src-tauri/src/commands/research.rs`; `apps/desktop/src/services/research.ts`; `features/form/ResearchPanel.tsx` (existing, rewritten); `features/settings/ResearchSection.tsx`; `features/publish/ConflictList.tsx` | commands and UI |
| `crates/testkit/tests/suite/research/{mod.rs, r2_foundation.rs, r5_feedback_and_undo.rs, r5_publish_hold.rs, r7_wiring.rs, r7_export_schedule.rs, r8_shop_facts.rs}` | app-seam tests |

### 6.2 Dependencies
Already in the lock file through `reqwest 0.13.5`: `url 2.5.8`, `idna 1.1.0`. `unicode-normalization 0.1` is a
dependency of `psm-core`. R2·U1 pins `unicode-casefold = 0.2.0` for full Unicode case folding and
`unicode-general-category = 1.1.0` for a character's general category. Not verified by the planner: an HTML parser (R8·U3, HTTP route only). Each is a task
with the pin recorded here. `psm-research` uses `rusqlite` and `reqwest` at the workspace's existing versions.
R8 (implementer, 2026-11-13): `psm-research` uses `reqwest = 0.13` (`blocking`, `rustls`; 0.13.5 in the lock), `base64 =
0.23` (0.23.1, already in the lock) for image data URLs, and `html5ever = 0.39` (0.39.0, already in the lock; its tokenizer
only, no tree) as the HTML parser.

### 6.3 Enums (`psm_research::domain` unless noted; serialized snake_case)
`RunState {Queued, Running, Finished, Failed, StoppedAtCap, Cancelled}` · `RunTrigger {Auto, Manual, FollowUp}` (R11: plus
`Arranged`; `Auto` and `FollowUp` are read from old rows and never created) ·
`Stage {Observe, Find, Read, Compare, Labels, Condition, Decide}` (1–7; settings names `observe, find, read, compare,
labels, condition, decide`) · `StepKind {Looked, Searched, Read, Compared, Concluded, NotDetermined}` · `CallState
{Sent, Answered, Failed, Unknown, NotSent}` · `SiteClass {Shop, Trusted, Untrusted}` · `Verdict {Match, Similar,
Different}` · `ErrorClass {Offline, Transient, Key, Quota, Request, Schema, Refusal}` · `FieldClass {NeverWritten,
Condition, Attribution, Measured, Visible}` (R11: plus `Derived`) · `Prerequisite {NoIntegration, NoModelKey, NoSearchKey, PriceMissing,
NothingToResearch, DayLimit, Offline, AutoOff}` (MC-A6 order; R11 removes `AutoOff`) plus `WaitingForEditor`. In core:
`psm_core::domain::research::FeedbackAction {Accepted, Cleared, Edited, KeptPrevious}`; `PublishStep::Held`;
`RowLabel::WaitingForYou(n)`. Job kind string `research`. Constants: `RESEARCH_NORMALISE_VERSION = 1` (core),
`RESEARCH_FINGERPRINT_VERSION = 2` (1 until R12·U1), `RESEARCH_CALL_ATTEMPTS = 3`. `INSTRUCTIONS_VERSION = 2` (`psm_research::config`; 1 until R12·U1).

### 6.4 Storage decision: one database file, schema steps outside the numbered list
**Decision.** Every research table lives in the app's SQLite file and is written inside the app store's
transactions. Nothing is added to `SqliteStore::migrations()`. Two idempotent schema steps create what is needed:
`psm_infra::sqlite::ensure` (run by the store at every open, after `to_latest`) for what app code reads and writes —
`attribute_values.ai_prior`, the four `publish_items` columns, `research_feedback`; and `psm_research::schema::ensure`
(run by whoever opens the store for research: the app from R7·U1, the test harness before that) for the engine's own
five tables. The engine's SQL lives in the research crate and runs through `SqliteStore::research_transaction`, which
hands a closure the `StoreTx` **and** the raw connection of the same transaction.

**Why not a separate SQLite file for the engine's tables.** It was evaluated against each contract:
- MC-A7 Commit rule: "Stage 7's attribute writes and its `concluded` and `not_determined` steps commit together."
  `attribute_values` is app data; steps and stage outputs would be in the other file. The app database runs in WAL
  mode (`store.rs` line 52), and SQLite does not make a commit across two attached files atomic in WAL mode: a crash
  between the two leaves either values without steps (MA6 broken until resume) or a committed stage 7 unit without
  its values (lost for good, since a committed unit never runs again: MA4, MA14, H4 broken).
- MC-A5 step 1: eligibility is read from `photos` and the call and outbound rows are inserted "in one transaction";
  with two connections a deletion can commit between the read and the insert and the photo is still sent (MA12).
- MC-A6: the start transaction sets the run `running` and the work is a row of the app's `jobs` table.
- MC-A1 and MC-A12 need the feedback rows and `ai_prior` in the photographer action's own transaction, and MC-A10 the
  publish columns: these are app tables in any case.
So a separate file breaks MC-A7 outright and MC-A5 unless it is attached to the same connection, which gives the
isolation but not the crash atomicity. **It is not an owner trade-off:** the contracts decide it.

**What this does to the migration-order problem with `main`.** It removes it. The numbered list is `main`'s alone;
the research schema is not versioned by position. A later numbered migration on `main` that recreates
`attribute_values` or `publish_items` must carry the research columns (the step would re-add a missing column, empty);
that is one sentence for the merge checklist, not a renumbering.

| Created by | Table / column |
|---|---|
| `psm_infra::sqlite::ensure` (R2·U2) | `attribute_values.ai_prior TEXT NULL` (JSON `{value, valid, provenance}`) · `publish_items.frozen_attribute_origin TEXT`, `.pending_conflicts TEXT`, `.resolved_attributes TEXT`, `.released_at INTEGER NULL` · `research_feedback(id, item_id, key, suggested_value, confidence, final_value NULL, action, at, step_token NULL)`, index `(item_id, key)` |
| `psm_research::schema::ensure` (R4·U1) | `research_runs(id, item_id, session_id, trigger, state, stage_reached, queued_at, started_at NULL, finished_at NULL, cost_micro_usd DEFAULT 0, settings_snapshot, form_snapshot, input_snapshot, error NULL)` with the partial unique indexes `(session_id) WHERE state='running'` and `(session_id) WHERE state='queued'` · `research_steps(id, run_id, seq, stage, kind, summary, detail, source_url NULL, source_title NULL, photo_ids, field_keys, created_at, UNIQUE(run_id, seq))` · `research_calls(id, run_id, stage, unit_key, attempt, provider, model, effort, state, request, response NULL, usage NULL, cost_micro_usd DEFAULT 0, started_at, finished_at NULL, duration_ms NULL, error_class NULL, error_text NULL, redaction_hit DEFAULT 0)` · `research_outbound(id, run_id, call_id, photo_id, provider, sent_at, sha256)` · `research_stage_outputs(run_id, stage, unit_key, fingerprint, output, reused_from_run_id NULL, committed_at, UNIQUE(run_id, stage, unit_key))` |
All `*_at` are UTC ms; money is `INTEGER` micro-USD; JSON is TEXT.

### 6.5 Rules (names fixed; signatures may gain parameters)
| Module | Functions | Contract | Unit |
|---|---|---|---|
| `psm_core::mechanisms::name_key` | `name_key(&str) -> String` | MC-A2 `N` | R2·U1 |
| `psm_core::mechanisms::sameness` | `same(&Definition, &Json, &Json) -> bool` (unit names from `definition.validations.unit_names`) | MC-A1 `S` | R2·U1 |
| `psm_core::mechanisms::publish_partition` | `partition`, `is_conflict` | MC-A10, MC-A1 | R5·U2 |
| `rules::value_map` | `map_output(&Definition, &Json) -> Mapped`; `statement`; `page_statement` | MC-A2 | R3·U1 |
| `rules::sites` | `host`; `TrustedSites::{parse_entry, add, remove, reorder, matches}`; `site_of`; `page_identity`; `is_own_listing`; `reading_order` | MC-A3 | R3·U2 |
| `rules::fingerprint` | `canonical`; `fingerprint(stage, unit)` | MC-A7 `F` | R3·U2 |
| `rules::confidence` | `field_class`, `decide_attribution`, `decide_measured`, `decide_visible`, `decide_condition` | MC-A4 | R3·U3 |
| `rules::cost` | `parse_micro_usd`; `call_cost`; `next_call`; `day_spend` | MC-A8 | R3·U4 |
| `rules::calls` | `classify`; `action`; `redact` | MC-A9 | R3·U4 |
| `rules::write_table` | `never_written`; `decide_write`; `is_conflict` (delegates to core from R5·U2) | MC-A1 | R3·U5 |
| `rules::run_rules` | `first_failing`; `may_start`; `start_order`; `follow_up_needed` | MC-A6, MC-A7 | R4·U4, R4·U5 |
| `rules::run_rules` | `arranged_needed` (whether an "Arranged" press creates a run) | MC-A6, MC-A7, 08B changelog 13 | R11·U1 |
| `rules::derived_condition` | `derive` | MC-A4, 08B changelog 12 (b); the rule is in the R11 plan, Unit 3 | R11·U3 |
| `rules::photo_plan` | `look_at` (which photos stage 1 looks at, in which order and role) | MC-A7, 08B changelog 12 (d) | R12·U1 |
| `adapters::page_reader::extract` (an adapter module, not a rule) | `product_extract` | 08A §4; the rule is in the R12 plan, Unit 2 | R12·U2 |

### 6.6 The surface app code uses, ports, secrets, commands, settings
- **`psm_research::api` (frozen by R2·U2).** `trait ResearchApi: Send + Sync { status(session_id) -> Result<ResearchStatus>;
  run(session_id) -> Result<()>; session_changed(session_id); settings_changed(); validate_settings(&Json) -> Result<()>;
  secrets_set(SecretsInput) -> Result<()>; secrets_state() -> Result<SecretsState>; site_add(&str) -> Result<Json>;
  site_remove(&str) -> Result<Json>; sites_reorder(&[String]) -> Result<Json>; export(ExportSelection, dest) -> Result<PathBuf>;
  purge_expired(now_ms) -> Result<()> }` (the three site methods return the new `research` settings value to save).
  Shapes: `ResearchStatus {reason: Option<Reason {code, sentence}>, run: Option<RunView {id, state, trigger,
  stage_reached, error}>, steps: Vec<StepView {seq, stage, kind, summary, detail, source_url, source_title, photo_ids,
  field_keys}>, fields_filled, to_review, can_run}` · `SecretsInput {model_key, search_key, reader_key}` (each optional) ·
  `SecretsState {model, search, reader}` (booleans) · `ExportSelection {Range {from, to} | Items(ids)}`.
- **Constructors used only by `crates/integrations/src/lib.rs` (R7·U1):** `psm_research::schema::ensure`,
  `ResearchService::new`, `psm_research::shop::CommerceShopFacts::new`, `psm_research::adapters::build` (from R8·U1:
  `build(settings, secrets: Arc<dyn SecretStore>, files: ResearchFiles)`, keys read per call).
- **Ports (`psm_research::ports`, R4·U1).** `ModelProvider::call(&ModelRequest) -> Result<ModelReply, CallFailure>` ·
  `VisualSearchProvider::search(&ResearchImage) -> Result<SearchReply, CallFailure>` · `PageReader::read(&self, url:
  &str) -> Result<PageReply, CallFailure>` · `Exchange {request, response}` (carried by the three replies only) ·
  `CallFailure {sent, status, kind, quota, text, usage}` with `FailureKind {Connect, Timeout, Reset, Status, Schema,
  Refusal}` (no `exchange` field; a refusal is a `CallFailure` of kind `Refusal`; `ModelReply.output` is plain JSON). `ResearchImage` has no public constructor; `ResearchImages::render` is the only maker.
- **Core seams.** `Imaging::render_research(preview, values, long_edge, dest)` · `CommerceAdapter::public_listing(
  external_id) -> PublicListing {hosts, item_url, item_path_tail}` (default: empty) · unit names in
  `Definition.validations.unit_names` (filled by the Shopify adapter in R8·U3).
- **Keychain accounts** (service `secrets::SERVICE`): `research-model-key`, `research-search-key`, `research-reader-key`.
  The redaction scan reads these and the integration's secret and token through one function the wiring supplies.
- **Commands.** Frozen in R2·U2: `research_status`, `research_run`, `research_secrets_set`, `research_secrets_state`,
  `research_site_add`, `research_site_remove`, `research_sites_reorder`, `research_export`. R5 adds
  `attribute_ai_accept(item_id, key)`, `attribute_conflict_resolve(item_id, key, action, value?)` and widens
  `publish_preview` (`n_withheld`, `conflicts`) and `publish_confirm` (the shown set).
- **Settings** (`settings.json` key `research`, a JSON slot in `psm_infra`; the shape is `psm_research::config::
  ResearchSettings`, R4·U4). **R11 removes `auto` and `quiet_seconds` and adds three settings, R12 one: §6.8.** `auto` (false; owner 2026-11-15, runs start from an explicit request, 08B changelog 13) ·
  `quiet_seconds` (1) · `source_wait_seconds` (120) · `attribution_keys` ([]: no field is attribution until the owner
  points at the shop's designer and maker fields; owner 2026-11-15, R10) · `trusted_sites` ([]) · `max_pages` (10) ·
  `stages.<stage>.{model, effort, image_long_edge}` (R10, evidence "Owner pick of models" 2026-11-15: observe
  `gpt-6.1-sol` low · find no model (the visual search) · read `gpt-6-luna` low · compare `gpt-6.1-terra` low · labels
  `gpt-6.1-sol` low · condition `gpt-6.1-terra` high · decide `gpt-6.1-terra` low; long edge 1024 for observe, find,
  read, compare, decide and 2048 for labels, condition) · `max_cost_per_run` ("0.50") · `max_cost_per_day` ("15") ·
  `prices` (R10: for `openai` × each of the three models the rows `input_token`, `cached_input_token`,
  `cache_write_token`, `output_token` (Sol 2.00/0.10/2.50/10.00, Terra 1.00/0.05/1.25/5.00, Luna 0.10/0.01/0.125/0.50
  USD per 1M, pricing page read 2026-11-12) and `web_search_call` (10000 per 1M = USD 10 per 1,000), fallback "0.05"
  per call; `serpapi_lens` `image_search_call` 25000 per 1M (USD 25 per 1,000 searches), fallback "0.025";
  `page_http` `page_fetch` 0, fallback "0.000001") · `language` ("English") · `record_days` (30, minimum 2) · `call_timeout_seconds` ({model: 180,
  other: 60}) · `providers.{model, visual_search, page_reader}` (adapter ids; from R8 the defaults are `openai`, `serpapi_lens`,
  `page_http`, and a saved `""` means the default).
- **Panel sentences (MC-A6).** `NoIntegration` "Connect a shop to use research" · `NoModelKey` "Add the model key in
  Settings → Research" · `NoSearchKey` "Add the visual search key in Settings → Research" · `PriceMissing` "No price is
  set for <provider> <model>" · `NothingToResearch` "This item type has no fields research can fill" · `DayLimit` "Day
  limit reached" · `Offline` "Offline" · `AutoOff` (no sentence; the **Research** button) · `WaitingForEditor` "Waiting
  for Capture One".

### 6.7 Export research files (R7·U2; read by R9·U1's report and replay)
`ExportSelection::Range {from, to}` takes local dates as `YYYYMMDD` numbers (inclusive). `export` returns the zip's
path; the folder `research-export-<local YYYYMMDD-HHMMSS>/` stays beside it, and the zip's entries are prefixed
with that folder's name. Per run, `<item id>-<folder key>/run-<id>/`:
- `run.json`: `{run: {id, item_id, session_id, trigger, state, stage_reached, queued_at, started_at, finished_at,
  cost_micro_usd, error}, settings_snapshot, form_snapshot, input_snapshot, call_record}` (the three snapshots
  parsed JSON or `null`; `call_record` is `"kept"` or `"call record expired"`).
- `steps.json`: `[{seq, stage, kind, summary, detail, source_url, source_title, photo_ids, field_keys, created_at}]`.
- `writes.json`: the stage 7 output as committed (`{writes, not_determined, ending}`), or `null`.
- `feedback.json`: `[{id, key, action, suggested_value, confidence, final_value, at}]` (values parsed JSON).
- `calls/<id>.json`: `{id, run_id, stage, unit_key, attempt, provider, model, effort, state, request, response,
  usage, cost_micro_usd, started_at, finished_at, duration_ms, error_class, error_text, redaction_hit}`; `request`
  and `response` are the recorded texts as JSON strings, byte for byte except that an image of a photo deleted at
  export time reads `{"withheld":"photo deleted"}` in place of `{"image_sha256": …}`.
- `images/<sha256>.jpg`: every stored image the run's exported call records reference.

### 6.8 Added by R11 and R12 (v10; names fixed, signatures may gain parameters)
- **Session mark.** `sessions.arranged_at INTEGER NULL` (UTC ms of the first press), created by
  `psm_infra::sqlite::ensure` like the other app-owned research columns (§6.4); `Session.arranged_at`; `SessionRow.arranged`.
  `SessionsService::{arrange(session_id) -> bool, next_unarranged(after) -> Option<session id>}`. Not an undo step; no
  `change_seq` bump.
- **`ResearchApi` gains** `arranged(session_id) -> Result<bool>` (a run was queued) and `settings_reset() -> Result<Json>`
  (the `research` value to save). `session_changed` creates no run from R11 on.
- **Integration layer.** `crates/integrations/src/arranged.rs`: arrange and next; preview for publish (marks the
  unmarked sessions of the selection first, then `PublishService::preview`, then which items have a research run queued
  or running). Constructs and connects only (H8).
- **Commands.** `session_arrange_next(session_id) -> {next_session_id, research_queued}` · `research_settings_reset() ->`
  the saved `research` value · `publish_preview` items gain `researching`. R11·U4 regenerates `bindings.ts`.
- **Settings (R11).** Removed: `auto`, `quiet_seconds` (a saved slot that carries them loads; they are ignored and not
  written back). Added: `excluded_keys` ([]: fields research leaves alone, treated as never-written) · `extra_shop_hosts`
  ([]: stored as typed, used as `H` with one leading `www.` removed) · `detailed_condition` (`{key: "", levels: []}`; a
  level is `{level, choice}` with `choice` null for "counts for nothing"; best level first).
- **Settings (R12).** `observe_max_photos` (8, at least 1).
- **Page text (R12·U2).** `PageReply.text` is the product extract; `PRODUCT_TEXT_CHARS = 10_000` beside
  `MAX_TEXT_CHARS` in `adapters::page_reader`.
- **Stage shapes (R12).** Stage 1 is one unit; its output holds `reads` and the ids of the photos that show a marking.
  The Read output gains the same-kind answer and the page's kind. The OpenAI strict schemas gain the new members only
  for requests that carry the new input (`photos`, `piece`), so R1's captures and replays of older exports stay valid.
- **Panel.** With every prerequisite passing and no run, the status carries no reason; the panel shows "Research starts
  when you press Arranged — next." and the Research button.

### 6.9 Added and retired by R13 (v12; names fixed, signatures may gain parameters)
- **Settings.** Added: `fill_keys` ([]: "Fields research fills"; a key not in it is treated as never-written, MC-A1).
  **Retired: `excluded_keys`** (a saved slot that carries it loads; it is ignored and not written back). The settings
  DTO and `bindings.ts` follow (R13·U1 regenerates the bindings). `attribution_keys` and `detailed_condition.key` only
  say how a chosen field is decided.
- **Field class.** `rules::confidence::field_class` takes the chosen list where it took the left-alone list. Order:
  never-written (MC-A1) · not chosen · derived · condition · attribution · measured · visible.
- **Prerequisite.** `NoFieldChosen` ("Choose the fields research fills in Settings → Research"), after
  `NothingToResearch`, before `DayLimit`; it blocks the creation of a run row.
- **Stage 6** makes no call when no condition field is chosen. Stage 7's identity names `fill_keys`;
  `RESEARCH_FINGERPRINT_VERSION` stays 2.
- **`rules::derived_condition::derive`** leaves out level fields with no value; the rule is in the R13 plan, Unit 2.
- **Tool.** `scratch_settings` chooses every field of `accept::shop_definitions()`;
  `docs/test-items/accept_settings_owner.json` carries the owner's 22 keys in `fill_keys`.
- **Test helpers.** The engine harness (`test_settings()`, `Harness::config()`) and the testkit's
  `research_test_settings()` choose every field their layouts use.

## 7. Environment (additions to `master_plan.md` §7)

### 7.2 Commands (flow §10: which tests a unit runs)
| Purpose | Command |
|---|---|
| Research rules (inline unit tests) | `cargo test -p psm-research --lib rules::<module>` |
| One engine test file | `cargo test -p psm-research --test engine <stem>::` |
| Whole research crate | `cargo test -p psm-research` |
| Research-only lint and compile check | `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace` |
| Tool crate | `cargo test -p psm-research-tool [<stem>::]`; lint as above with `-p psm-research-tool` |
| App-seam tests | `cargo test -p psm-testkit --test suite research::<stem>::` and the named `p…::` stems |
| Full gate (only at the end of a milestone that changed app files) | `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test` |
| The tool | `cargo run -p psm-research-tool -- <probe \| bench-search \| bench-models \| report \| replay \| accept>` |
| Dev app on scratch data | `cd apps/desktop && PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev` |

Per milestone (the exact commands are in each plan's unit and Closing gate sections):

| Milestone | Per-unit tests | Closing gate |
|---|---|---|
| R1 | tool crate tests | tool crate tests + lint + `cargo check --workspace`; no app suite |
| R2 | core unit tests, `research::r2_foundation::`, `p01`, `p03`, `p28`, `p29`, bindings, typecheck | **full gate** |
| R3, R4, R10 | research crate tests of the unit + research lint + `cargo check --workspace` | `cargo test -p psm-research` + research lint; no app suite |
| R5 | `research::r5_…::`, `p39`, `p40`, `p10`, `p13`, `p14`, `p64`, the form and publish UI tests | **full gate** |
| R6 | `npx vitest run` of the touched features, typecheck | `npm run typecheck` · `npm test`; no cargo test (no Rust file changed) |
| R7 | `research::r7_…::`, `p16`, `p21`, `p09`, `p01`, `p61`, research `export::`, settings UI tests | **full gate** + `cargo test -p psm-research` |
| R8 | research `adapter_…::`; U1 and U3 also `research::r7_wiring::`; U3 also `research::r8_shop_facts::`, `p12` | **full gate** + `cargo test -p psm-research` |
| R9 | tool crate tests | tool crate tests + lint; no app suite |
| R11 | U1–U3: research crate tests of the unit, research lint, `cargo check --workspace`; U1 also the tool crate and `research::` (it edits three testkit test files); U4: `research::r11_arranged::`, `research::r7_wiring::`, `p01`, `p03`, `p66`, `p13`, `p16`, `cargo test -p psm-desktop`, typecheck; U5, U6: `npx vitest run` of the touched features, typecheck | **full gate** + `cargo test -p psm-research` |
| R12 | research crate tests of the unit, research lint, `cargo check --workspace`; U1 also the tool crate; U4 the tool crate | `cargo test -p psm-research` + `cargo test -p psm-research-tool` + `cargo test -p psm-testkit --test suite research::` + lint of both crates; no full app suite (the `research::` stems drive the wired engine with the crate's fakes, so they are run although no app file changed) |
| R13 | U1: research crate, tool crate, `research::`, `cargo test -p psm-desktop`, `npx vitest run src/features/settings`, typecheck, research lint, `cargo check --workspace`; U2: research crate tests and lint; U3: `npx vitest run src/features/settings`, typecheck; U4: the tool crate's `u4_accept::` and lint | **full gate** (`cargo test --workspace` includes the research and tool crates) |

### 7.3 Runner and templates
`run_phase.sh` is unchanged: a milestone is a phase to it. Three templates gained text that applies only when a plan
has `## Unit` sections or a **Closing gate** section, so `main`'s `P…` plans behave as before:
`templates/implement.md` (the gate may be a tracker the plan names; units in order with a checkpoint after each; the
plan's Closing gate replaces the template's), `templates/review.md` (several checkpoints: check the range; the
plan's Closing gate names the commands), `templates/fix.md` (the plan's Closing gate names the commands).
`rereview.md`, `projection.md` and the wrappers are unchanged. The implementer wrapper says "make one commit"; the
plan and the implement template say one per unit, and the plan wins.

### 7.4 Verified by the planner (2026-11-12)
- **RESEARCH-VERIFY-11 (installed source, `~/.cargo/registry/src/*/reqwest-0.13.5/src/error.rs`).** `Error::is_connect()`
  (line 158) is true when the connection was never established, so no request byte left (DNS failures arrive this
  way too). `Error::is_timeout()` (line 119) is also true for a connect timeout, so `is_connect()` is tested first.
  Mapping used by the research HTTP seam (R8·U1): `is_connect()` → `CallFailure {sent: false}`; anything else →
  `sent: true`. **Errors carry the URL** (`with_url`, `src/async_impl/client.rs:3100`), and a vendor key can sit in a
  query parameter: the seam calls `without_url()` before any error text is formed.
- The store is one `Mutex<Connection>` and a transaction wraps a `rusqlite` transaction in `SqliteTx`
  (`crates/infra/src/sqlite/store.rs` lines 38–41, 1547–1556), so one closure can be given both views of it. WAL mode
  is set at line 52.
- Nothing in production calls `AttributesService::ai_suggest` (`services/attributes.rs:476`); it refuses only
  `Provenance::Human`. `History` restores attribute rows unconditionally (`services/history.rs:734–745`).
  `PublishService::confirm` freezes every live valid row (`services/publish.rs:640–648`). "Looks right" writes a
  human edit today (`features/form/FormDialog.tsx`, `rawValue`).
- A dev-dependency of `psm-research` on `psm-testkit`, while the testkit (through `psm-integrations`, from R7) depends
  on `psm-research`, is a dev-dependency cycle, which Cargo allows; integration tests then link one copy of the
  crate. Inline unit tests (`--lib`) must not use testkit types. Not run by the planner (no cargo in this session);
  R3·U5 is the first unit to exercise it and reports `BLOCKED` with the error if Cargo refuses.

### 7.4A Verified by the planner (2026-11-15, for R11 and R12)
- **No shoot template exists** in the code or the book (grep "template": 08A §3 "Not existing: shot templates", §12
  deferred; the only code hits are HTML templates and Capture One hook templates).
- **No field picker exists.** "Condition photos key" is a text box (`apps/desktop/src/features/settings/SettingsPage.tsx`);
  the attribution keys are text boxes checked against `listDefinitions()` (`ResearchSection.tsx`).
- **`www.` of a shop host is covered:** `rules::sites` matches a shop host or any subdomain of it (`covers`).
- **Shop pages and attribution today:** `rules::confidence::decide_attribution` gives `high` for a `match` on site class
  `shop` and counts `shop` toward `medium`; the item's own listing is set apart in `rules::sites::reading_order` and
  never read (R-IA9). `decide_measured` and `decide_visible` call the same function.
- **Stage 1 today:** one unit and one model call per Main or unreviewed photo (`stages.rs`, `observe`); stage 7 takes a
  visible field's photo read from the first Observe output that has one, which is the cover's (`decide.rs`,
  `Evidence::photo_read`).
- **Cost of one photo call (R1 capture `fixtures/openai/stage_observe_gpt-6-luna.json`):** 1,558 input tokens for one
  1024 px image with its instructions and fields, 77 output tokens. **Several images in one request work**
  (`stage_condition_gpt-6-luna.json`: six images, 30,385 input tokens at 2048 px).
- **Detailed condition against the condition levels, 42 test pieces** (`docs/test-items/beyo_test_samples_1.json`): of
  the 28 pieces that have both, worst level None → Very Good 2; Minor → Very Good 17, Good 5; Moderate → Very Good 2,
  Good 2; "Excellent" and a worst level of Major do not occur. The data does not determine a table (R11 owner card).
- **Recorded listing pages** (`~/psm-research-bench/a01/pages/calls`, R1, 375 plain fetches, read on the dev Mac, not
  committed). A schema.org Product block: the owner's shop 37 of 42 pages, bukowskis 3 of 35, maleforsimports 1 of 1,
  auctionet 0 of 84, pamono (.se, .com, .eu) 0 of 38. Its `brand` names the seller on the shop ("Beyo Vintage") and on
  two dealers; only bukowskis names a designer, as `creator`, on those 3 pages. No trusted site states a maker in a
  structured field. Exactly one `<h1>` whose text occurs in the visible text: auctionet 83 of 84, pamono 38 of 38, the
  shop 42 of 42; bukowskis has none (35 of 35). Median visible text: auctionet about 3,900 characters, bukowskis 3,200,
  the shop 5,500, pamono 17,700 to 21,500 with the `<h1>` about 7,800 characters in and the "Creator / Manufacturer"
  block about 4,300 characters after it (90th percentile of its end 7,200 on .se, 9,700 on .com and .eu); the shop's
  "Designer:" block about 2,500 after its `<h1>`.
- **App tests that depend on the research crate's behaviour:** `crates/testkit/tests/suite/research/{r7_wiring.rs,
  r7_export_schedule.rs}` start runs through the wired engine with `"auto": true`; R11·U1 changes them, and R12's
  closing gate runs the `research::` stems.

### 7.5 Test harnesses
- **Research harness** (`crates/research/tests/engine/harness.rs`, grown by R3·U5 → R4): a `TestApp` with a fake shop,
  `schema::ensure` run, the crate's fakes, `FakeSecrets` with distinctive key values, a manual clock, a `JobRunner`
  for kind `research`; `Harness::{new, add_main_photo_with_preview, tick, restart, with_default_settings}`.
- **Fakes** (`psm_research::fakes`): `FakeModel`, `FakeVisualSearch`, `FakePageReader` (scripted replies per stage
  and unit, request log, fault queue, `before_call` hook), `FakeResearchHttp` (R8·U1: replays capture files).
- **Testkit** (app): `FakeCommerce::set_public_listing` (R2·U2); `TestApp::with_research()` (R7·U1: the wired app
  with the crate's fakes as providers).

### 7.6 What the owner provides before R1 can run
One file, `~/.psm_research.env`, mode 0600, never copied into the repo:
```
OPENAI_API_KEY=…          # exists already; best a key of its own project with a USD 50 monthly budget limit set in the OpenAI dashboard
SERPAPI_API_KEY=…         # new: account at serpapi.com, free plan (250 searches a month is enough), key from the dashboard
GOOGLE_VISION_API_KEY=…   # new: Google Cloud console → new project → enable "Cloud Vision API" → attach a billing account (first 1,000 units a month are free) → Credentials → Create API key → restrict it to Cloud Vision API
```
`~/.psm_shopify.env` (existing) is read for one read-only shop query. The orchestrator checks the three names are
present (never printing a value) before launching R1.

## 8. Points recorded by planning (no card)
- **R6's closing gate** runs `npm test` and typecheck but no `cargo test --workspace`, although R6 changes app files:
  it changes TypeScript only, and flow §10's own reason ("no app file changed, so no app test can change") applies
  to the Rust suite. If the orchestrator reads §10 literally, add the full gate to R6's Closing gate.
- **Core's `ai_suggest` is left in place**, unused; research writes through `psm_research::writes`. Removing it and
  moving its P10 test is a later tidy-up.
- **Attribution keys are shop-specific.** The defaults carry the two keys from R4·U4 on; for a shop with other key
  names nothing stops a run (a refusal would be a ninth prerequisite in MC-A6).
- **Address guard of the HTTP page reader** (R8·U3) is a task without a criterion: no contract states it.
- **"Looks right" changes meaning by contract** (MC-A12: an accept); R5·U1 makes the change.
- **The implementer wrapper's "one commit"** is overridden by the plan (one per unit); if the orchestrator prefers,
  the wrapper can say so in one line.
- The intention map still says 08A/08B are "not planned yet".
- **R11 and R12 supersede book text** (each plan has a table "What this plan supersedes in the book"). The
  orchestrator folds those tables into 08A and 08B as changelog entries; until then the plan tables are the rule.
- **R12's closing gate runs the `research::` app stems** although no app file changes (§7.2); flow §10's reason for
  skipping the app suite does not hold for tests that drive the wired engine.

## 9. Amendment log
- 2026-11-15 v12 (planner) — plan written for **R13_research_chosen_fields** (4 units, 15 criteria, 40 rows `A27·…`,
  1 `[manual]`; changes app files; depends on R12; `projection: no`). §0 (ids), §3 (`[manual]`, real calls and a
  USD 2 ceiling also in R13·U4 task 3; projection), §4 (one row, totals sentence), §5 (graph, schedule, merge order,
  ledger coverage), new §6.9 (names added and retired), §7.2 (one row), this line. `research_row_mapping.md` gained
  plan number A27 and the rows of A05, A13, A24, A25 and A26 that R13 retires or amends. Decided by planning, no
  owner card: the list is empty on a fresh install and research then does nothing (prerequisite `NoFieldChosen`;
  A24·C1(a) amended); a designer, maker or detailed-condition field must also be in the list; an old `excluded_keys`
  is ignored, not converted; stage 6 is not called when no condition field is chosen; the fingerprint version stays 2.
- 2026-11-15 v11 (orchestrator) — second measurement done (evidence doc "Acceptance measurement 2": USD 0.11 per
  piece, was 0.30; no sure designer/maker really wrong). Owner decision 08B changelog 17: research fills only the
  chosen fields (an allow-list replacing `excluded_keys`), and the detailed condition is derived from the levels that
  have a value. **R13 research fills chosen fields** named; needs its plan before it runs.
- 2026-11-15 v10 (planner) — plans written for **R11_research_on_arranged** (6 units, 33 criteria, 99 rows `A25·…`;
  changes app files; depends on R10) and **R12_research_cost** (4 units, 16 criteria, 37 rows `A26·…`, 3 `[manual]`;
  research and tool crates only; depends on R11). §0, §3 (`[manual]` and real calls also in R12·U4; ceiling USD 20), §4,
  §5 (graph, seams, schedule, merge order, ledger coverage), §6.3, §6.5, new §6.8 (names), §7.2, new §7.4A (facts
  verified by the planner), §8. Decisions recorded in the plans: the mark is `sessions.arranged_at` through the ensure
  step; `auto` and `quiet_seconds` are removed, not kept inert; "next" is shooting order with wrap; publishing marks
  at the popover's preview and Confirm is never disabled; reset keeps the person's lists; for designer and maker no
  page of the shop is evidence; stage 1 becomes one call over at most 8 photos (supersedes owner card 2 for stage 1,
  on the owner's cost idea); no template and no skip-on-structured-data rule are built, with the reasons. One owner
  card (the detailed-condition table), not blocking. `phases_research.txt` and `research_row_mapping.md` updated.
- 2026-11-15 v9 (orchestrator) — owner picked the models (evidence doc) and moved the research trigger to an
  "Arranged — next" button (08B changelog 13). R10 stays crate-only: the picked models and prices, `auto` off by
  default (the quiet-period trigger is retired), `attribution_keys` empty by default; the pickers move to R11.
  **R11 research starts when the session is arranged** (the button and session mark, the trigger, the pickers for
  designer/maker, fields research leaves alone and extra shop addresses, detailed condition derived, no attribution
  from look-alike own listings). **R12 research cost** (photos chosen by the shoot template, a light local parser,
  then the 42-piece measurement again). Both need their plans written before they run.
- 2026-11-13 v8 (orchestrator) — after the acceptance measurement the owner set a follow-up phase **R11 research flow
  by shoot template** (08B changelog 12): physical fields never researched (a settings picker of excluded fields: dimensions, weight, damage details for the owner), detailed condition derived, condition
  step deferred to classification + leaving/publishing, photo look by template position, a light local parser,
  no attribution from look-alike own listings. Not yet planned; it comes after R10 and needs its own plan.
- 2026-11-13 v7 (orchestrator) — §3 Isolation names the testkit's R8 test surface (R8 review lesson); R8 page-reader
  fixes (charset, no proxy, address ranges) added to R9·U1 task 0 before the measurement.
- 2026-11-13 v6 (orchestrator) — R8 projection r0 attempt 2 folded (ledger L1–L13, 0 owner cards; 08B MC-A3 body
  aligned with changelog 11(d)); `adapters::build(settings, Arc<dyn SecretStore>, ResearchFiles)`, keys read per call;
  testkit gains `TestApp::with_research_using`. R8 is 20 criteria / 35 rows. **Projection is off for R8 from attempt 3**
  (two rounds, 0 owner cards after the first; the review is the gate; frontmatter `projection: no`).
- 2026-11-13 v5 (orchestrator) — R8 projection r0 folded (card 1 = No for now; ledger L1–L31 applied to the R8 plan;
  08B changelog 11). §6.6 corrected to the code: `PageReader::read(&self, url)`, `CallFailure` without `exchange`,
  the `FailureKind` names, `adapters::build` gaining `ResearchFiles`, default provider ids from R8. §7.2: R8·U1 now
  changes an app file and runs `research::r7_wiring::`. R8 is 20 criteria / 34 rows.
- 2026-11-12 (R7·U2 implementer) — §6.7 Export research files: the field names the plan delegated.
- 2026-11-12 v4 (orchestrator) — R5 projection turned off after seven rounds (0 owner cards, each round smaller;
  every ledger applied to the plan and 08B changelogs 6, 8, 9, 10); the review is R5's gate. R8 keeps `projection: yes`.
- 2026-11-12 v3 (orchestrator) — R2–R4 review notes folded into R5–R10 as "Carried from earlier reviews" sections
  (implementers do not read `backlog_research/`); §6.6 default `attribution_keys`; schedule: R5 and R6 run on
  track A after R4 (track A was idle until R6, track B busy with R1), then R7 on A; track B does R1, then R8.
- 2026-11-12 v2 — restructured into 10 milestones on `crates/research` (flow §10); storage decision §6.4; tracks §5;
  tracker moved into the plan files; superseded plans `A01`–`A24` deleted (they are in `d116654`).
- 2026-11-12 v1 — created by the planner (24 phase plans).
