---
plan: R10_research_defaults
role: implement
round: 1
state: DONE
date: 2026-11-15
actor: claude-opus-5.5 (pipeline-implementer)
---
# Handoff — R10_research_defaults, implement r1

Gate: intention RATIFIED; `depends_on` R9_research_tool_and_acceptance Tracker = APPROVED (2026-11-13); R10 was
NOT_STARTED. Task 0: the plan's scope note sends Task 0 to the evidence doc's "Owner pick of models (2026-11-15)".
That section names every stage's model, so the phase is not blocked.

## What was built
- **Research works with no further setup once the keys are entered.** A fresh app with no research settings saved
  now has a model and effort for every stage that calls the model: photo look `gpt-6.1-sol` low, reading listings
  `gpt-6-luna` low, comparing `gpt-6.1-terra` low, labels `gpt-6.1-sol` low, condition `gpt-6.1-terra` high, decide
  `gpt-6.1-terra` low. Find is the visual search and needs no model. It also has a price for every unit the adapters
  report, with a fallback price per call. Pressing "Run research" starts a run straight away.
- **Automatic research is off by default** (owner 2026-11-15). Runs start from an explicit request: today the
  manual "Run research", later the Arranged — next button (R11).
- **No shop field counts as designer or maker by default.** The dev store's `beyo.designer` is no longer the
  default. Attribution fields come only from the owner's choice; the R11 picker will set them.
- Price table: for `openai` × {Sol, Terra, Luna} the units `input_token`, `cached_input_token`, `cache_write_token`,
  `output_token` (from the evidence doc's price table, read 2026-11-12) and `web_search_call` (USD 10 per 1,000),
  with a fallback of USD 0.05 per call. `serpapi_lens` `image_search_call` is USD 25 per 1,000 with a fallback of
  0.025. `page_http` `page_fetch` is 0 with a fallback of 0.000001.
- `master_plan_research.md` §6.6 records the defaults.

## Tests → criterion rows
| Row | Test (`crates/research/tests/engine/defaults.rs`) | Boundary / outcome asserted |
|---|---|---|
| A24·C1(a) | `a24_c1a_shipped_defaults_start_a_requested_run` | Fresh app (settings slot absent, keys in `FakeSecrets`, fake shop, fakes under ids `openai`/`serpapi_lens`/`page_http`). After the first Main photo with a preview and the quiet period, `tick` starts nothing and there are no runs. Then `run_again` makes one run: `manual`, `running`, with `started_at` set. Its settings snapshot carries the default stages and provider ids. Status reason is exactly `AutoOff` with an empty sentence. |
| A24·C1(b) | `a24_c1b_without_the_keys_no_run_and_export_works` | Same app without the keys. `run_again` is refused with the NoModelKey sentence and no run exists. Status reason is (`NoModelKey`, "Add the model key in Settings → Research"). The session export ends with `ExportProgress (1,1,Done)` and `R4-1_01_main.jpg` on disk. |
| A24·C2(a) | `a24_c2a_defaults_are_complete_and_priced` | `ResearchSettings::default()`: each model-calling stage has a non-empty model and effort. For each (provider, model) pair in use there is a price row for every unit that adapter reports, and a fallback above 0. The OpenAI units come from the adapter's own `usage_of` applied to a reply that has every usage field. All price texts parse. The caps parse as micro-USD above 0, `record_days` ≥ 2, and `validate` accepts the value. |
| A24·C2(b) | `a24_c2b_attribution_keys_are_empty_by_default` | `attribution_keys` is empty (the row as narrowed by the 2026-11-15 scope note). |

Core mechanisms: none (the plan lists none), so no unit tests were added.

## Assumptions
1. **"status names no prerequisite"** with `auto` off by default: the status reason is `AutoOff` with an empty
   sentence. MC-A6 puts `auto` off last and the panel shows nothing for it. The test asserts that exact value rather
   than accepting several outcomes.
2. **C2(a) "every stage"** means every stage that calls the model. Find is the visual search and keeps an empty
   model. Giving it one would put a meaningless model in Settings.
3. **Lens price:** the cheapest paid SerpApi plan (USD 25 per 1,000 searches, evidence doc), not the measurement's
   0, so the caps count searches. It is one search per run, so USD 0.025.
4. **Page download fallback:** 0.000001 USD (1 µUSD), the smallest positive amount. The row asks for a positive
   fallback on every pair, and a download is free.
5. **Model fallback:** USD 0.05 per call, the acceptance measurement's worst-case figure.
6. **The fresh app** is built in the test file: the research slot is removed and the fakes are wrapped under the
   default ids. The plan's `Harness::with_default_settings()` was not added, so `harness.rs` stays outside the edit.
7. `max_pages` stays 10 because the measurement did not change it.

## Deviations
- `crates/research/tests/engine/lifecycle.rs`: removed `a13_c1d_default_attribution_keys_are_the_shops` (R4·U4, row
  A13·C1(d)). It asserted a non-empty `beyo.designer` default, which the owner's 2026-11-15 decision removes, and
  A24·C2(b) now covers the default. The imports it alone used went with it. This file is outside the plan's listed
  perimeter. The edit is forced: otherwise the crate suite is red.
- Not run: the app suite (flow §10: no app file changed). One residual note for the reviewer: `research_test_settings()` in
  testkit leaves out `attribution_keys`, so app tests now read an empty list instead of `["beyo.designer"]`. I
  checked: no app-suite test asserts attribution.

## Files changed (declared write perimeter)
- `crates/research/src/config.rs` (defaults)
- `crates/research/tests/engine/defaults.rs` (new)
- `crates/research/tests/engine/main.rs` (`mod defaults;`)
- `crates/research/tests/engine/lifecycle.rs` (superseded test removed, see Deviations)
- `docs/implementation/master_plan_research.md` (§6.6 defaults)
- `docs/implementation/plans/R10_research_defaults.md` (Tracker line, Review log)
- this handoff
No mutation-probe files (no mutation regime). No archgraph delta.

## Checkpoint SHA
- U1: `4d87ea8`

## Stamp
- `cargo test -p psm-research --test engine defaults::` → 4 passed
- `cargo test -p psm-research` → 120 unit + 174 engine + 1 doc passed, 0 failed
- `cargo clippy -p psm-research --all-targets -- -D warnings` → clean
- `cargo fmt --all -- --check` → clean
- `cargo check --workspace` → clean
- App suite: not run (flow §10, no app file changed)

## Manual try-out
1. Use a fresh scratch folder: `PSM_DATA_DIR=$HOME/psm-research-dev-2 npm run tauri -- dev`. Connect the test shop,
   then enter the model key and the visual-search key in Settings → Research (add trusted sites if you like).
2. Settings → Research already shows a model and effort for each stage (find has none). Automatic research is off.
3. Create a session and shoot one photo, then classify it Main. Within a few seconds of pressing "Run research" on
   the form's research panel, steps appear. You do not need to touch any other research setting.
4. Settings → Research shows no designer or maker field chosen. Until you choose them (R11), no field counts as
   designer or maker.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
