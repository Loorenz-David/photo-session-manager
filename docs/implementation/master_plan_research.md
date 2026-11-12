# Session Manager — master plan, research agent part (v1, 2026-11-12)

Planner: Claude Fable 5.2 (`pipeline-planner`, implementation-planner doctrine under `docs/process/flow.md`; the owner
asked for this planning on Fable, 2026-11-12: "you can commit and start the master planning"). Branch `feature/agentic`,
written at `d4ce98c`. **Companion to `docs/implementation/master_plan.md`, which stays the authority for everything
not restated here** (roles, standing rules, repository layout, pinned versions, environment, commands, test harness).
Every research phase reads both files. Where the two differ on a research name, this file wins.

## 0. Layout decision (why this is a separate file)

`main` keeps planning and running phases `P71…` while this branch plans and runs the research agent; `main` is merged
into this branch again, and this branch into `main` later. The layout is chosen so that those merges touch as few
shared lines as possible and **`tools/pipeline/run_phase.sh` runs unchanged**.

| Thing | Where | Why |
|---|---|---|
| Phase ids | `A01_…` … `A24_…` | `main` owns `P…`; the runner resolves every file from the id (`plans/<id>.md`, `state/<id>.json`, `prompts/<role>/<id>_*.md`, `handoffs/<role>/handoff_<id>_*.md`, `narratives/<id>.md`), so `A…` files can never collide with a `P…` file. Checked in `run_phase.sh` `run_phase()`, `set_state`, `archive`. |
| Plan files | `docs/implementation/plans/A<nn>_<slug>.md` | same folder as every other plan: the runner needs no option and `archive` works. |
| Phase list | `docs/implementation/phases_research.txt` (new) | `phases.txt` is not touched. `run_phase.sh all` reads only `phases.txt`, and `main` itself stopped appending to it at P52 (later phases are started one by one with `run_phase.sh phase <id>`). Research phases are started the same way, in the order of the list file. Driver loop, no runner change: `while read -r ph; do [[ -z "$ph" \|\| "$ph" == \#* ]] && continue; grep -q '"state":"APPROVED"' "docs/implementation/state/$ph.json" 2>/dev/null && continue; tools/pipeline/run_phase.sh phase "$ph" \|\| break; done < docs/implementation/phases_research.txt`. |
| Tracker, registry, sequencing | this file | `master_plan.md` is 1,067 lines and `main` appends to its tracker (§4), registry (§6) and amendment log (§9) with every phase. **This plan set edits no line of `master_plan.md`.** The templates name `master_plan.md` as the tracker's home; every `A…` plan says, in its Dependencies section, that its tracker row and gate are in **this** file (the plan file wins over the prompt, `templates/implement.md` item 4). |
| Reviewer notes | `docs/implementation/backlog_research.md` (created by the first review that has a note) | `backlog.md` is appended by every `main` review. Each `A…` plan's Review mode says where notes go. |
| Behaviour tests | `crates/testkit/tests/suite/research/<phase_lower>_<slug>.rs`, listed in `crates/testkit/tests/suite/research/mod.rs`; **one** line `mod research;` in `tests/suite/main.rs` (added, with `research/mod.rs`, by whichever of A01 and A07 runs first; every plan with a suite file says "create if absent") | `main` adds one `mod pNN_…;` line per phase to `main.rs`; with a sub-module the research work adds one line there in total. Per-phase command: `cargo test -p psm-testkit --test suite research::<stem>::`. |
| Migrations | files `crates/infra/src/sqlite/migrations/a01_….sql`, `a02_….sql`, `a03_….sql`, appended to `SqliteStore::migrations()` in one block **after the last `0NN` entry**, under the comment `// research (feature/agentic): keep this block last` | `main` adds `014_…` and later. `rusqlite_migration` applies by list position, so at a merge `main`'s new entries go **above** the research block. Consequence, a standing rule (§3): until this branch is merged into `main`, a build of this branch opens only scratch data directories. |
| Code | new modules and folders (§6.1); existing files are edited only where a contract changes existing behaviour (`services/attributes.rs`, `services/history.rs`, `services/publish.rs`, `mechanisms/status.rs`, `domain/attribute.rs`, `domain/session.rs`, `ports/commerce.rs`, `ports/images.rs`, `integrations/src/lib.rs`, the form and publish UI) | smallest shared surface; each such edit is named in the plan that makes it. |

Not done, and why: `PSM_IMPL_DIR=docs/implementation/research` (a second implementation folder) would also run the
runner unchanged, but it splits narratives and state away from where `CLAUDE.md` and the orchestrator's instructions
look, and it is exercised only by the dry run. A one-line `PSM_PHASES_FILE` option in the runner would allow `all`,
at the price of editing a file `main` also edits; the loop above gives the same without it.

## 1. Goal
Build the research agent of `docs/intention/book/08A_research_agent.md` (RATIFIED 2026-11-12) under the exact rules of
`book/08B_research_contracts.md` (CONTRACTS RATIFIED 2026-11-12): an in-app, service-ready engine on the existing job
queue that looks at an item's photos, finds and reads matching listings, and writes suggested form values with a
computed confidence, a reason, sources and a visible trail of steps; with the publish rules for unchecked
suggestions, cost caps, the call record, Export research and the dev-Mac report and replay tool. This file never
restates product semantics; where it and the book differ, the book wins and this file is amended.

## 2. Sources of truth (additions to `master_plan.md` §2)
| Content | Artifact |
|---|---|
| Meaning, hard constraints HA1–HA6, stages, scope ladder, pre-implementation protocol | `docs/intention/book/08A_research_agent.md` |
| Exact rules MC-A1…MC-A12, resolutions R-IA1…R-IA26, probe ids RESEARCH-VERIFY-1…11 | `docs/intention/book/08B_research_contracts.md` |
| Measurements MA1–MA18 (root of every research criterion's trace) | `docs/intention/book/14_measurement_ledger.md` §Research agent |
| H3 as amended for research copies | `docs/intention/book/01_objective_and_constraints.md` |
| Provider facts, the nine trusted sites, probe and benchmark results (written by A01, A02, A23), the owner's picks | `docs/intention/evidence/research_providers.md` |
| Test pieces and answer key | `docs/test-items/beyo_test_samples_1.json` (42 pieces, 12 of them unpublished, 32 with condition images; counted from the file 2026-11-12), `docs/test-items/answer_key_corrections.json` (corrections scored in their own bucket; hedged-attribution rule) |
| Sanitized provider captures (fixtures) | `crates/integrations/src/research/fixtures/<provider>/*.json`, written by A01 (and A02 for model replies), never by hand |
| Research skeleton, tracker, sequencing | this file |
| Phase goal / tasks / criteria / Review log | `docs/implementation/plans/A<nn>_<slug>.md` |
| Execution order | `docs/implementation/phases_research.txt` |

Fold-back rule as in `master_plan.md` §2. A probe result that contradicts a contract goes to the orchestrator as a
finding against 08B (never patched into a plan).

## 3. Standing rules (additions; `master_plan.md` §3 and `flow.md` apply unchanged)
- **Trace.** Every research criterion row cites `MA1…MA18`, `MC-A1…MC-A12`, `HA1…HA6` or `H1…H8`. `[rule]` rows are
  allowed only for UI rendering and tool output rows, citing the chapter section. `[manual]` rows exist only in the
  owner-keyed phases A01, A02 and A23 (flow §4 rule 1 exemption: they are recorded in the evidence doc and the plan's
  Review log, never asserted by the suite).
- **Rule 9 holds without exception.** The suite never calls OpenAI, a search vendor, a listing site or the shop.
  Real calls happen only in A01, A02 and A23, from the tool `psm-research`, with keys from the environment.
- **Rule 17 (external shapes).** A fixture whose shape a provider owns is a **sanitized capture written by A01/A02**,
  cited in the row as `fixtures/<provider>/<file> (RESEARCH-VERIFY-n)`. A row that needs such a capture is in a phase
  that depends on A01. Every other row is written against our own ports and their fakes (§7.5 additions), whose
  shapes are ours. One external shape is grounded by this plan from the installed source: RESEARCH-VERIFY-11 (§7.4).
- **Keys.** Benchmark keys live only in `~/.psm_research.env` (mode 0600, outside the repo), loaded by the session with
  `set -a; . ~/.psm_research.env; set +a`; the app's keys live only in the Keychain through `ports::secrets`
  (accounts in §6.6). No key, and no URL carrying one, is ever printed, logged, written to a fixture, an evidence
  doc, a handoff, a recording or a commit. Every recording is scanned for the key values before it is written
  (fail closed).
- **Spending ceiling.** USD 50 for the two benchmark phases together (A01 ≤ 15, A02 ≤ 35), enforced by the tool
  (`--ceiling-usd` is required; hard call caps per provider). A23 states its own ceiling (USD 40). The owner changes
  a ceiling by telling the orchestrator; a session never raises one.
- **HA5 in code.** No model name, vendor name or reseller domain as a literal in `crates/core`. In core they are
  opaque strings read from the settings snapshot. Literals live in `crates/integrations/src/research/`, in
  `crates/infra/src/settings_file.rs` defaults (A24) and in the tool.
- **Scratch data only.** Until this branch is merged into `main`, the app built from it is run only with
  `PSM_DATA_DIR=$HOME/psm-research-dev` (or another scratch folder), never against the real data folder: the research
  migrations sit after `main`'s in the list, and `main` will add migrations before the merge. After each merge of
  `main` into this branch the scratch folder is deleted. Every Manual try-out says so.
- **No `implementer: fable`** (flow log v1.3). No plan sets it; UI phases run on the default implementer.
- **Projection** is on (`projection: yes`) for every phase that touches a silent-failure mechanism (charter rule 6)
  or carries a provider-owned fixture (rule 17); the list is in §5.
- **`Value` gains a field in A07** (`ai_prior`). Every later phase that builds an `attribute::Value` literal sets it.

## 4. Progress tracker (one row per phase; agents update only their own row)
| Phase | State | Date | Actor | Note |
|---|---|---|---|---|
| A01_research_probes_and_search_benchmark | NOT_STARTED | 2026-11-12 | planner | needs the owner's keys (§7.6) |
| A02_research_stage_model_benchmark | NOT_STARTED | 2026-11-12 | planner | |
| A03_research_values_and_sameness | NOT_STARTED | 2026-11-12 | planner | |
| A04_research_sites_and_fingerprints | NOT_STARTED | 2026-11-12 | planner | |
| A05_research_confidence | NOT_STARTED | 2026-11-12 | planner | |
| A06_research_cost_and_call_rules | NOT_STARTED | 2026-11-12 | planner | |
| A07_research_storage_and_write_permission | NOT_STARTED | 2026-11-12 | planner | |
| A08_research_feedback_and_undo | NOT_STARTED | 2026-11-12 | planner | |
| A09_research_publish_hold | NOT_STARTED | 2026-11-12 | planner | |
| A10_research_ports_images_and_fakes | NOT_STARTED | 2026-11-12 | planner | |
| A11_research_call_gateway | NOT_STARTED | 2026-11-12 | planner | |
| A12_research_stages_and_decide | NOT_STARTED | 2026-11-12 | planner | |
| A13_research_run_lifecycle | NOT_STARTED | 2026-11-12 | planner | |
| A14_research_resume_reuse_follow_ups | NOT_STARTED | 2026-11-12 | planner | |
| A15_research_model_adapter | NOT_STARTED | 2026-11-12 | planner | fixtures from A01 |
| A16_research_visual_search_adapter | NOT_STARTED | 2026-11-12 | planner | blocked on the owner's adapter pick |
| A17_research_page_reader_and_shop_hosts | NOT_STARTED | 2026-11-12 | planner | blocked on the page-reader route pick |
| A18_research_panel_ui | NOT_STARTED | 2026-11-12 | planner | |
| A19_research_settings_ui | NOT_STARTED | 2026-11-12 | planner | |
| A20_research_publish_ui | NOT_STARTED | 2026-11-12 | planner | |
| A21_research_retention_and_export | NOT_STARTED | 2026-11-12 | planner | |
| A22_research_report_and_replay_tool | NOT_STARTED | 2026-11-12 | planner | |
| A23_research_acceptance_benchmark | NOT_STARTED | 2026-11-12 | planner | owner-attended |
| A24_research_defaults | NOT_STARTED | 2026-11-12 | planner | blocked on the owner's model pick |

## 5. Sequencing and gates
A phase starts only when every phase in its `depends_on` is APPROVED in §4. One phase runs at a time in this
worktree (one build lock, one checkpoint stream), in the order of `phases_research.txt`. A01 and A02 are first in that list (the owner's order) and need the owner's
keys; while the keys are missing the orchestrator starts at A03 and takes A01 and A02 as soon as they arrive (they
touch only the tool crate, the fixtures folder, the evidence doc and the testkit's research test list).

| Phase | Builds (contracts) | depends_on | Difficulty | Projection | Blocked on |
|---|---|---|---|---|---|
| A01 | probes RESEARCH-VERIFY-1…10, visual-search benchmark on the 42 pieces, page reading on the nine sites, the tool crate | — | high | no | owner's keys (§7.6) |
| A02 | first per-stage model comparison (observe, read, condition) on recorded inputs | A01 | normal | no | owner's keys |
| A03 | name key, model output → value, sameness (MC-A2, MC-A1 `S`) | — | high | yes | — |
| A04 | hosts, trusted entries, site identity, page identity, own listing, reading order (MC-A3); fingerprints (MC-A7 `F`) | A03 | high | yes | — |
| A05 | confidence derivation (MC-A4) | A04 | high | yes | — |
| A06 | money, call cost, stop rule, the day (MC-A8); error classes and actions, redaction (MC-A9 pure part) | A05 | high | yes | — |
| A07 | migration a01, `ai_prior`, the write table in `ai_suggest`, cleared set, lowering (MC-A1) | A03 | high | yes | — |
| A08 | feedback rows, conflict actions, guarded undo/redo (MC-A12) | A07 | high | yes | — |
| A09 | withheld and pending values, step `held`, resolution and release (MC-A10); migration a02 | A08 | high | yes | — |
| A10 | provider ports, `ResearchImage` and its renderer, research files, fakes (MC-A5 source and type guard) | A06 | high | yes | — |
| A11 | migration a03, the call gateway: record-then-send, outbound rows, attempts, cost recording (MC-A5 order, MC-A9, MC-A8) | A09, A10 | high | yes | — |
| A12 | stages 1–7 over the ports, unit commit, steps, Decide and its writes, cap and failure endings (MC-A7 commit rule, MC-A3, MC-A4, MC-A1, MC-A8) | A11 | high | yes | — |
| A13 | run rows, prerequisites, triggers, quiet period, readiness, start order, day limit, offline, the `research` job and runner (MC-A6); `research` settings | A12 | high | yes | — |
| A14 | kill-and-resume, reuse, follow-up trigger set (MC-A7) | A13 | high | yes | — |
| A15 | OpenAI model adapter: stage instructions and schemas, usage, errors, refusal | A11, A01 | high | yes | — (A01's captures) |
| A16 | the chosen visual-search adapter | A11, A01 | normal | yes | owner's pick after A01 |
| A17 | page reader by the chosen route; shop hosts, the item's own listing and unit names from the shop (RESEARCH-VERIFY-8, -9) | A12, A01 | high | yes | route pick after A01 |
| A18 | commands, events, the live research panel, "n to review", Research / Run again, Looks right as accept | A14 | normal | no | — |
| A19 | Settings → Research: keys, trusted sites with priorities, models and effort per stage, caps, language, auto, quiet period, prices | A13 | normal | no | — |
| A20 | publish popover: withheld line, conflicts warning and resolution, "Waiting for you" label; conflict marks in the form | A09, A18 | normal | no | — |
| A21 | retention of call records, Export research (MC-A11) | A14 | high | yes | — |
| A22 | `psm-research report` and `replay` (MC-A11 replay identity, 08A §10A) | A21, A01 | normal | yes | — |
| A23 | the 08A §11 operational measurement on the real engine and adapters; second model comparison | A15–A22 | high | no | owner-attended, keys |
| A24 | shipped defaults (models, efforts, prices, attribution keys) and the reachability check | A23 | normal | no | owner's model pick |

**Can start now, with no owner input:** A03 → A14 in order, then A18–A21 (16 phases). A22 needs A01 only for the
tool crate. **Wait for the keys:** A01, A02. **Wait for A01's evidence:** A15. **Wait for the owner's pick
from A01's report:** A16, A17. **Wait for the owner's model pick:** A24 (after A02 and A23).

**Ledger coverage** (every research measurement is served; checked row by row against the plan files):
MA1 A07, A12 · MA2 A05, A07, A12 · MA3 A10, A11, A16 · MA4 A13, A14 · MA5 A06, A11, A12, A24 · MA6 A03, A12 ·
MA7 A13, A18, A24 · MA8 A09, A20 · MA9 A21, A22 · MA10 A07, A12 · MA11 A03, A04, A05, A12, A17 · MA12 A10, A11, A16 ·
MA13 A11, A13, A14 · MA14 A04, A12, A14 · MA15 A06, A11, A15, A16, A17 · MA16 A06, A11, A15, A16, A19, A21, A22 ·
MA17 A09, A20 · MA18 A08. The operational measurement of 08A §11 is A23.

## 6. Research skeleton and naming registry (new unless marked existing)

### 6.1 Layout
| Path | Role |
|---|---|
| `crates/core/src/domain/research.rs` | enums and rows of §6.3 and §6.4, `AiPrior`, `FormSnapshot`, `ResearchConfig` (the settings snapshot, opaque strings for models and providers) |
| `crates/core/src/mechanisms/research/{mod.rs, normalise.rs, value_map.rs, sameness.rs, sites.rs, fingerprint.rs, confidence.rs, cost.rs, calls.rs, write_table.rs, publish_partition.rs, run_rules.rs}` | the pure functions of MC-A1…MC-A10 (§6.5); the only research code with unit tests |
| `crates/core/src/ports/research.rs` | `ModelProvider`, `VisualSearchProvider`, `PageReader`, `ResearchImages`, the `ResearchImage` type (§6.6) |
| `crates/core/src/services/research/{mod.rs, gateway.rs, runs.rs, stages.rs, decide.rs, export.rs, retention.rs}` | `ResearchService` and its parts (H8: no Tauri, no HTTP) |
| `crates/infra/src/imaging/research.rs`, `crates/infra/src/research_files.rs` | the research-copy renderer (H3 as amended) and the file store `research/images/<sha[0..2]>/<sha>.jpg` under the data dir |
| `crates/infra/src/sqlite/migrations/{a01_research_values.sql, a02_research_publish.sql, a03_research_runs.sql}` | A07, A09, A11 (list order = this order; A11 depends on A09 for that reason) |
| `crates/integrations/src/research/{mod.rs, http.rs, job.rs, openai/, visual_search/, page_reader/, fixtures/}` | adapters, their HTTP seam (`ResearchHttp`, impl `ReqwestResearchHttp` modelled on `commerce/shopify/transport.rs`), the `research` job handler, sanitized captures |
| `crates/research-tool/` (package `psm-research-tool`, lib `psm_research_tool`, bin `psm-research`; added to the workspace `members`) | A01/A02 benchmark and probes, later `report` and `replay` (A22), `accept` (A23) |
| `crates/testkit/src/fake_research.rs`; `crates/testkit/tests/suite/research/` | `FakeModel`, `FakeVisualSearch`, `FakePageReader`; the behaviour tests |
| `apps/desktop/src-tauri/src/commands/research.rs`; `apps/desktop/src/services/research.ts`; `apps/desktop/src/features/form/ResearchPanel.tsx` (existing, rewritten in A18); `apps/desktop/src/features/settings/ResearchSection.tsx`; `apps/desktop/src/features/publish/ConflictList.tsx` | commands and UI |

### 6.2 Dependencies
Already in the local registry as dependencies of `reqwest 0.13.5`: `url 2.5.8`, `idna 1.1.0`. `unicode-normalization 0.1`
is a direct dependency of `psm-core` today (NFKC and NFC come from it). Needed and **not verified by the planner**
(this session made no network call): a full Unicode case fold and the general category of a character (MC-A2 `N`).
A03 picks the crates (candidates: `caseless`, `unicode-general-category` or `unicode-properties`), verifies the pins
with `cargo generate-lockfile`, and records versions here; hand-written tables are not allowed. `psm-core` gains
`url`/`idna` only if A04's implementer shows they add no I/O type to core (they do not: both are pure).

### 6.3 Enums (Rust names in `psm_core::domain::research`, serialized snake_case)
`RunState {Queued, Running, Finished, Failed, StoppedAtCap, Cancelled}` · `RunTrigger {Auto, Manual, FollowUp}` ·
`Stage {Observe, Find, Read, Compare, Labels, Condition, Decide}` (numbers 1–7 in that order; settings names
`observe, find, read, compare, labels, condition, decide`) · `StepKind {Looked, Searched, Read, Compared, Concluded,
NotDetermined}` · `CallState {Sent, Answered, Failed, Unknown, NotSent}` · `FeedbackAction {Accepted, Cleared, Edited,
KeptPrevious}` · `SiteClass {Shop, Trusted, Untrusted}` · `Verdict {Match, Similar, Different}` · `ErrorClass {Offline,
Transient, Key, Quota, Request, Schema, Refusal}` · `FieldClass {NeverWritten, Condition, Attribution, Measured,
Visible}` · `Prerequisite {NoIntegration, NoModelKey, NoSearchKey, PriceMissing, NothingToResearch, DayLimit, Offline,
AutoOff}` (MC-A6 order) plus the waiting reason `WaitingForEditor`. Existing enums that grow: `PublishStep` gains
`Held` (A09); `RowLabel` gains `WaitingForYou(n)` (A09); job kind string `research` (A13; `Job.kind` is a `String`).
Constants: `RESEARCH_NORMALISE_VERSION = 1`, `RESEARCH_FINGERPRINT_VERSION = 1`, `RESEARCH_CALL_ATTEMPTS = 3`.

### 6.4 Tables (all `*_at` UTC ms; money `INTEGER` micro-USD; JSON as TEXT)
| Migration | Table / column |
|---|---|
| `a01` (A07) | `attribute_values.ai_prior TEXT NULL` (JSON `{value, valid, provenance}`) · `research_feedback(id, item_id, key, suggested_value, confidence, final_value NULL, action, at, step_token NULL)`, index `(item_id, key)` |
| `a02` (A09) | `publish_items.frozen_attribute_origin TEXT`, `.pending_conflicts TEXT`, `.resolved_attributes TEXT`, `.released_at INTEGER NULL` |
| `a03` (A11) | `research_runs(id, item_id, session_id, trigger, state, stage_reached, queued_at, started_at NULL, finished_at NULL, cost_micro_usd DEFAULT 0, settings_snapshot, form_snapshot, input_snapshot, error NULL)` with the two partial unique indexes of MC-A6 (`(session_id) WHERE state='running'`, `(session_id) WHERE state='queued'`) · `research_steps(id, run_id, seq, stage, kind, summary, detail, source_url NULL, source_title NULL, photo_ids, field_keys, created_at, UNIQUE(run_id, seq))` · `research_calls(id, run_id, stage, unit_key, attempt, provider, model, effort, state, request, response NULL, usage NULL, cost_micro_usd DEFAULT 0, started_at, finished_at NULL, duration_ms NULL, error_class NULL, error_text NULL, redaction_hit DEFAULT 0)` · `research_outbound(id, run_id, call_id, photo_id, provider, sent_at, sha256)` · `research_stage_outputs(run_id, stage, unit_key, fingerprint, output, reused_from_run_id NULL, committed_at, UNIQUE(run_id, stage, unit_key))` |

### 6.5 Core mechanisms (functions in `psm_core::mechanisms::research`; signatures may gain parameters, names are fixed)
| Module | Functions | Contract | Phase |
|---|---|---|---|
| `normalise` | `name_key(&str) -> String` | MC-A2 `N` | A03 |
| `value_map` | `map_output(&Definition, &Json) -> Mapped {Value{canonical, dropped}, Fail{reason}}`; `statement(&Definition, stated: &str) -> Stated {Mapped(Json), Unmapped(String)}` | MC-A2 | A03 |
| `sameness` | `same(&Definition, &Json, &Json) -> bool` | MC-A1 `S` | A03 |
| `sites` | `host(&str) -> Option<String>`; `TrustedSites::{parse_entry, add, remove, reorder, matches}`; `site_of(url, shop_hosts, &TrustedSites) -> Option<Site {class, identity}>`; `page_identity(&str) -> Option<String>`; `is_own_listing(url, &OwnListing {url, path_tail}, shop_hosts) -> bool`; `reading_order(candidates, …) -> Vec<…>` | MC-A3 | A04 |
| `fingerprint` | `canonical(&Json) -> String`; `fingerprint(stage: u8, unit: &Json) -> String` | MC-A7 `F` | A04 |
| `confidence` | `field_class`, `decide_attribution`, `decide_measured`, `decide_visible`, `decide_condition` → `Decision {value, confidence, sources, named}` | MC-A4 | A05 |
| `cost` | `parse_micro_usd(&str) -> Option<i64>`; `call_cost(usage, provider, model, &PriceTable) -> i64`; `next_call(run_cost, cap, reserve, stage) -> NextCall {Call, SkipToDecide, EndAtCap}`; `day_spend(calls, now_ms, tz) -> i64` | MC-A8 | A06 |
| `calls` | `classify(&CallFailure) -> ErrorClass`; `action(ErrorClass, attempt) -> CallAction`; `redact(&str, secrets: &[String]) -> Redacted {text, hit}` | MC-A9 | A06 |
| `write_table` | `never_written(&Definition, in_layout) -> bool`; `decide_write(stored: Option<&Value>, cleared: &[Json], proposed, &Definition) -> WriteDecision`; `is_conflict(&Value, &Definition) -> bool` | MC-A1 | A07 |
| `publish_partition` | `partition(values, definitions) -> Partition {frozen, withheld, pending, origin}` | MC-A10 | A09 |
| `run_rules` | `first_failing(&PrerequisiteInput) -> Option<Prerequisite>`; `may_start(&StartInput) -> bool`; `start_order(&mut Vec<QueuedRun>)`; `follow_up_needed(latest: &InputSnapshot, now: &InputSnapshot) -> bool` | MC-A6, MC-A7 trigger set | A13, A14 |

### 6.6 Ports, secrets, services, commands, settings
- **Ports (`psm_core::ports::research`, A10).** `ModelProvider::call(&ModelRequest) -> Result<ModelReply, CallFailure>`
  (`ModelRequest {stage, model, effort, instructions, schema, images: Vec<ResearchImage>, text_inputs}`;
  `ModelReply {output: ModelOutput {Json, Refusal(text)}, usage: Vec<(unit, u64)>, exchange: Exchange}`) ·
  `VisualSearchProvider::search(&ResearchImage) -> Result<SearchReply {candidates: Vec<Candidate {url, title, rank}>,
  usage, exchange}, CallFailure>` · `PageReader::read(url, keys…) -> Result<PageReply {final_url: Option<String>, title,
  text, images: Vec<PageImage>, usage, exchange}, CallFailure>` · `Exchange {request, response}` is what the gateway
  records (MC-A9), with the adapter's declared secret-bearing parts already replaced · `CallFailure {sent: bool,
  status: Option<u16>, kind, text, usage: Option<…>, exchange: Option<Exchange>}`. `ResearchImage` has **no public
  constructor**; only `ResearchImages::render(photo, long_edge) -> Result<ResearchImage>` (impl in infra) makes one
  (MC-A5 type guard). `Imaging` (existing) gains `render_research(preview, values, long_edge, dest) -> Result<OutputInfo>`.
- **`CommerceAdapter` (existing) gains** `public_listing(&self, external_id: Option<&str>) -> Result<PublicListing {hosts:
  Vec<String>, item_url: Option<String>, item_path_tail: Vec<String>}>` with a default body returning empty values
  and `unit_names(&self) -> Vec<(String, String)>` (the seven neutral unit names of MC-A2 → this adapter's unit
  codes; default empty, so measured fields are not determined). A12 adds both methods and the `FakeCommerce`
  setters; A17 implements them for Shopify (RESEARCH-VERIFY-9; the unit codes are the adapter's own constants).
- **Keychain accounts** (service `secrets::SERVICE`, existing): `research-model-key`, `research-search-key`,
  `research-reader-key` (only if the chosen page-reader route needs one). The redaction scan (MC-A9 layer 2) reads
  these and the integration's `client_secret` and in-memory access token through one function,
  `ResearchService::secret_values()`.
- **Service.** `ResearchService` (`App.research`, `AppHandles.research`; built in `crates/integrations/src/lib.rs`
  beside `publish`), runner `App.research_jobs = JobRunner::new(store).only_kind("research")` on its own thread, the
  main runner's `excluding_kinds` gains `"research"` (A13).
- **Events.** `AppEvent::ResearchChanged { session_id }` → Tauri event `research_changed` (emitted on every run
  state change and every committed step).
- **Commands (A18–A21; snake_case, `#[tauri::command] #[specta::specta]`).** `research_status(session_id)`,
  `research_run(session_id)`, `attribute_ai_accept(item_id, key)`, `attribute_conflict_resolve(item_id, key, action,
  value?)` (A08 adds the last two), `research_secrets_set`, `research_secrets_state`, `research_site_add`,
  `research_site_remove`, `research_sites_reorder` (A19), `research_export` (A21). `publish_preview` and
  `publish_confirm` grow (A09: `n_withheld`, `conflicts`, the shown set).
- **Settings (`settings.json` key `research`, struct `ResearchSettings` in `psm_infra::settings_file`, A13).**
  `auto` (true) · `quiet_seconds` (1) · `source_wait_seconds` (120) · `attribution_keys` (`["custom.designer", "custom.manufacturer"]`, the shop's Designer and Manufacturer field keys,
  08A §6; set in A13 so no run ever treats them as visible fields; A24 confirms them) ·
  `trusted_sites` ([]) · `max_pages` (10) · `stages.<stage>.{model, effort, image_long_edge}` (models and efforts
  empty until A24; long edge 1024 for observe, find, compare and 2048 for labels, condition) · `max_cost_per_run`
  ("0.50") · `max_cost_per_day` ("15") · `prices` ([] until A24) · `language` ("English") · `record_days` (30,
  minimum 2) · `call_timeout_seconds` ({model: 180, other: 60}) · `providers.{model, visual_search, page_reader}`
  (adapter ids, empty until A15–A17).
- **Panel sentences (MC-A6; fixed here so service, UI and tests agree).** `NoIntegration` "Connect a shop to use
  research" · `NoModelKey` "Add the model key in Settings → Research" · `NoSearchKey` "Add the visual search key in
  Settings → Research" · `PriceMissing` "No price is set for <provider> <model>" · `NothingToResearch` "This item type
  has no fields research can fill" · `DayLimit` "Day limit reached" · `Offline` "Offline" · `AutoOff` (no sentence;
  the **Research** button) · `WaitingForEditor` "Waiting for Capture One".

## 7. Environment (additions to `master_plan.md` §7)

### 7.2 Commands
| Purpose | Command |
|---|---|
| One research phase's behaviour tests | `cargo test -p psm-testkit --test suite research::<phase_lower>_<slug>::` |
| One research core mechanism | `cargo test -p psm-core mechanisms::research::<module>::tests` |
| The tool | `cargo run -p psm-research-tool -- <subcommand>` (`probe`, `bench-search`, `bench-models`, later `report`, `replay`, `accept`) |
| Dev app on scratch data | `cd apps/desktop && PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev` |
The closing gate is `master_plan.md` §7.2's, unchanged.

### 7.4 Verified by the planner (2026-11-12)
- **RESEARCH-VERIFY-11 (installed source, `~/.cargo/registry/src/*/reqwest-0.13.5/src/error.rs`).** `Error::is_connect()`
  (line 158) is true when a source in the chain is a `hyper_util::client::legacy::Error` with `is_connect()`: the
  connection was never established, so no request byte left (DNS failures arrive this way too; `is_dns()` is at
  line 176). `Error::is_timeout()` (line 119) is true for a `TimedOut` source anywhere in the chain, **including a
  connect timeout**, so `is_connect()` is tested first. Kinds `Request`, `Body`, `Decode` (lines 329–340) without
  `is_connect()` mean the request was started. Mapping used by `ReqwestResearchHttp` (A15): `is_connect()` →
  `CallFailure {sent: false}`; anything else → `sent: true`. **Errors carry the URL** (`with_url`, e.g.
  `src/async_impl/client.rs:3100`), and a vendor key can sit in a query parameter: the research HTTP seam calls
  `without_url()` before any error text is formed. A15 confirms the mapping with one loopback reproduction (a
  refused `127.0.0.1` port), which is not an external service.
- The runner needs no change for `A…` ids (§0). `crates/testkit/tests/suite/main.rs` lists modules by hand; a file
  missing from a `mod` list never runs and nothing warns (P70 narrative): every plan names the line it adds.
- `AttributesService::ai_suggest` (`crates/core/src/services/attributes.rs:476`) refuses only `Provenance::Human`;
  `same_content` (line 81) is the existing idempotency check; `History` restores attribute rows unconditionally
  (`services/history.rs:734–745`); `PublishService::confirm` freezes every live valid row (`services/publish.rs:
  640–648`) and `derive_status` lists the publishing steps by name (`mechanisms/status.rs:16–27`). "Looks right" in
  the form today writes the value as a human edit (`features/form/FormDialog.tsx`, `rawValue`); MC-A12 makes it an
  accept (A08, A18).

### 7.5 Test harness additions (each added by the phase named, with a caller in that phase)
- A10: `FakeModel`, `FakeVisualSearch`, `FakePageReader` (`psm_testkit::fake_research`): scripted replies keyed by
  stage and unit, a record of every request received (images by SHA-256), faults per call (`CallFailure` values), a
  `pause_before(n)` hook used for kill points; `TestApp::with_research()` (fake shop, fake providers, `FakeSecrets`
  holding distinctive key values that appear in no expected text, research settings filled, a price table);
  `TestApp::add_main_photo_with_preview(session)` (a photo with an adjusted preview and a composition).
- A12: `FakeCommerce::{set_public_listing, set_unit_names}`.
- A13: `TestApp::research()`, `research_tick(ms)`; the manual clock drives the quiet period and the day.
- A15: `FakeResearchHttp` (`psm_testkit::fake_research_http`): replays capture files, records the requests received.

### 7.6 What the owner provides before A01 can run
One file, `~/.psm_research.env`, mode 0600, never copied into the repo:
```
OPENAI_API_KEY=…          # exists already; best a key of its own project with a USD 50 monthly budget limit set in the OpenAI dashboard
SERPAPI_API_KEY=…         # new: account at serpapi.com, free plan (250 searches a month is enough), key from the dashboard
GOOGLE_VISION_API_KEY=…   # new: Google Cloud console → new project → enable "Cloud Vision API" → attach a billing account (first 1,000 units a month are free) → Credentials → Create API key → restrict it to Cloud Vision API
```
`~/.psm_shopify.env` (existing, `master_plan.md` §3) is read for the one read-only shop query of RESEARCH-VERIFY-9.
Nothing else. The orchestrator checks the three names are present (never printing a value) before launching A01.

## 8. Gaps surfaced by planning (no card; recorded for the orchestrator)
- `research.attribution_keys` is shop-specific (08A §6: "initially the shop's designer and manufacturer keys"). With a
  wrong or empty list a designer field would be treated as a visible field and could be written at `medium` from a
  photo read. The default carries the two keys from the first phase that has settings (A13), A24 confirms them
  against the shop's definitions, and A19 shows a warning when a listed key is not a definition. For a shop with
  other key names nothing stops a run: a refusal is not in MC-A6 and is not added. If the owner wants one, MC-A6
  needs a ninth prerequisite.
- The HTTP page reader (A17, if that route is picked) fetches URLs that come from search results. The plan orders
  it never to request an IP-literal, `localhost`, loopback, private or link-local address. No contract states this,
  so it has no criterion row; if the orchestrator wants it tested, MC-A3 needs one sentence first.
- `Looks right` changes meaning by contract (MC-A12: accept, the value stays an AI value) from what the form does
  today (a human edit). A08 makes the change; the owner sees it in A08's narrative.
- The intention map (`docs/intention/intention.md`, rows 08A and 08B) still says "not planned yet"; the
  orchestrator updates it.

## 9. Amendment log
- 2026-11-12 v1 — created by the planner (research part, round 1).
