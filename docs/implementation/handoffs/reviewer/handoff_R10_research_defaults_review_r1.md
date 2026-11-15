---
plan: R10_research_defaults
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-15
actor: claude-opus-5.5 (pipeline-reviewer)
---
# Handoff: R10_research_defaults, review r1

Review history: first review of this milestone (1 unit, 4 rows). Nothing was settled earlier.

## Gate
- Tracker said `IMPLEMENTED` (2026-11-15). Checkpoint `4d87ea8` (U1) is in `git log`. The gate passes.
- Perimeter: `git show --stat 4d87ea8` lists 6 paths: `crates/research/src/config.rs`,
  `crates/research/tests/engine/{defaults.rs, main.rs, lifecycle.rs}`, `docs/implementation/master_plan_research.md` and
  the plan file. These equal the handoff's declared files. The handoff itself was committed separately by the runner (`1c49f73`).
  `lifecycle.rs` is outside the plan's Files list but declared as a forced deviation (see Notes). No finding.

## Verified correct
- **Checks (Review mode, run once by the reviewer, one cargo command at a time):**
  `cargo test -p psm-research --test engine defaults::` → 4 passed · `cargo clippy -p psm-research --all-targets -- -D warnings` → clean ·
  `cargo fmt --all -- --check` → exit 0 · `cargo check --workspace` → clean. No app suite (flow §10: no app file changed).
  The implementer's crate stamp (120 unit + 174 engine + 1 doc) stands.
- **Values against the authority:**
  - Stage models and efforts match `research_providers.md` "Owner pick of models (2026-11-15)" for every stage. Find has no model; it is the visual search.
  - Token prices match the evidence doc's "Candidate models" table (Sol 2.00/0.10/2.50/10.00, Terra 1.00/0.05/1.25/5.00, Luna 0.10/0.01/0.125/0.50).
  - Web search is `10000` per 1M (USD 10 / 1,000, as in the evidence doc). Lens is `25000` per 1M (USD 25 / 1,000, the cheapest paid plan in the evidence doc).
  - Every OpenAI pair has a `cache_write_token` row, which resolves the carried R8 note.
- **Seams:**
  - Unit names match what the adapters emit: `usage_of` gives `input_token`, `cached_input_token`, `cache_write_token`, `output_token` and `web_search_call`; `serpapi_lens` gives `image_search_call`; `page_reader/http` gives `page_fetch`.
  - The gateway keys search and page calls as `(provider id, "")`, which is the model the shipped rows use, so `call_cost` finds them.
  - `missing_price` checks exactly the six model stages plus search plus reader, so the shipped table clears MC-A6's price prerequisite.
  - The stop-rule reserve (Decide fallback, 50 000 µUSD) is below the 0.50 run cap.
  - `§6.6`'s rewritten long-edge text matches `ResearchConfig::default` (2048 for labels and condition, 1024 for the rest).
- **Rows (by reading the production path, then the test):**
  - **A24·C1(a)** `a24_c1a_shipped_defaults_start_a_requested_run`: the fresh app has the slot removed, keys in FakeSecrets and fakes under the default ids.
    After the quiet period, `tick` gives `None` and there are no runs. Then `run_again` + `tick` makes one Manual, Running run with `started_at`.
    The run's snapshot holds the default stages and providers, and the reason is exactly (`AutoOff`, "").
    It would fail on either wrong default: with the old empty prices or models, `run_again` is refused with `PriceMissing`; with `auto` true, the first `tick` creates an auto run.
  - **A24·C1(b)**: the refusal sentence, no run, the exact `NoModelKey` reason, and the export ends `(1,1,Done)` with the file on disk.
  - **A24·C2(a)**: the units come from the adapter's own `usage_of`, so a missing `cache_write_token` row (or any unit row) fails the test.
    It also checks a positive fallback per pair, that the price texts parse, the caps in micro-USD, `record_days ≥ 2` and `validate`.
  - **A24·C2(b)**: `attribution_keys` is empty, as the plan's 2026-11-15 scope note narrows the row.
  - No test checks internal call counts or private helpers. All four pass the over-engineering test.
- **Assumptions (none contradicts the book):**
  - (1) `AutoOff` counts as "no prerequisite". MC-A6 orders `auto` last and gives it no sentence, and the owner set `auto` off.
  - (2) Find has no model.
  - (3) Lens costs 0.025 per search.
  - (4) A page download has a 1 µUSD fallback.
  - (5) A model call has a 0.05 fallback.
  - (6) The fresh app is built inside the test file instead of a `Harness` method.
  - (7) `max_pages` stays 10.
  - All are in the narrative in plain words.
- **H1–H8:** hold. The diff changes defaults in the research crate only, plus one test file and docs. HA5 holds: the model literals are in `crates/research/src/config.rs`. No key, secret or live call.

## Findings
None blocking (no B1–B4).

## Notes written to backlog
`docs/implementation/backlog_research/R10_research_defaults.md` (5 notes):
1. Plan lesson: a plan that changes a shipped default should list the tests that pin the old one. The deleted `a13_c1d_…` was forced but outside the Files list.
2. → R11: testkit's `research_test_settings()` has no `attribution_keys`, so app tests now run with no attribution field.
3. The tracker line named actor `codex`, but the session was Claude Opus 5.5. This review corrected it.
4. The shipped Lens price (0.025 per search) differs from the measurement's 0. Expected.
5. → R11 / orchestrator: a `research` slot saved before R10 keeps its old explicit values (`prices: []`, empty models,
   `auto: true`, `beyo.designer`). Such installs stay blocked with "price missing". Use a fresh folder, or a "reset to defaults" action.

## Carry-forward dispositions
| Note | Destination |
|---|---|
| 1 (plan lesson) | orchestrator / planner, for future default-changing plans |
| 2 (testkit attribution keys) | R11 |
| 3 (tracker actor) | resolved in this review |
| 4 (Lens price vs measurement) | none needed (owner informed in narrative) |
| 5 (pre-R10 saved slots) | R11 (reset action), or the owner uses a fresh folder (narrative "Not yet") |

## Write perimeter (this session)
- `docs/implementation/plans/R10_research_defaults.md`: Tracker line and Review log entry
- `docs/implementation/backlog_research/R10_research_defaults.md` (new)
- `docs/implementation/narratives/R10_research_defaults.md` (new)
- this handoff
No production or test code was touched; there were no probes (flow §4: no mutation regime). No archgraph delta. Nothing committed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
