---
plan: R12_research_cost
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-15
actor: opus (claude, pipeline-reviewer)
---
# Review: R12_research_cost, round 1

**Gate:** passed. The plan's Tracker line read IMPLEMENTED. The checkpoints `af0e9ce` (U1), `64ef178` (U2), `6e7c9d0` (U3), `5a2d855` (U4 tasks 1–2) and `c2a5364` (closing) are all in `git log`, and the working tree was clean at the start.

**Scope.** U1–U3 and the two automated rows of U4, as the plan's Review mode says. The `[manual]` rows A26·C14(a), C15(a) and C16(a) (U4 tasks 3–6: the owner-attended 42-piece measurement, USD 20 ceiling) belong to the orchestrator. They are **open**, not judged here.

## Verified correct
- **Perimeter.** I ran `git show --stat` on each of the five checkpoints. Their union equals the implementer handoff's "Files changed": production, tests, `accept_settings_owner.json`, `master_plan_research.md` §6.3, the plan and the handoff. The extras outside the units' file lists are all disclosed: `rules/run_rules.rs` (doc comments only), engine `api.rs`/`export.rs`/`settings_r11.rs`/`adapter_openai.rs`, and `fakes.rs` in the closing commit. No undeclared path, and no app file changed.
- **Checks.** I ran these once, one at a time, in the foreground, on code tree `c2a5364`, with the Review mode's commands:

  | Command | Result |
  |---|---|
  | `cargo test -p psm-research` | ok: 154 unit, 222 engine, 1 doc |
  | `cargo test -p psm-research-tool` | ok: 28 |
  | `cargo test -p psm-testkit --test suite research::` | ok: 77 |
  | `cargo clippy -p psm-research --all-targets -- -D warnings` | ok |
  | `cargo clippy -p psm-research-tool --all-targets -- -D warnings` | ok |
  | `cargo fmt --all -- --check` | ok (exit 0) |
  | `cargo check --workspace` | ok |

  No full app suite (flow §10: no app file changed).
- **U1 (A26·C1–C7), the production code read.**
  - `rules::photo_plan::look_at`: Main photos by position (ties by id, no position last), then unreviewed by id, Condition never, cut at the limit. Only the first chosen photo can be `wide`, and only if it is a Main photo.
  - `stages::observe`: one unit `photos`. Its identity is the ordered `[id, rev, role]` list plus long edge, model, effort, instructions version and language.
    - Photos that cannot be rendered are left out and named in the detail. If none renders, the unit commits "Could not read the photos" with no read.
    - The input `photos` is indexed over the rendered list.
    - Marking indexes are mapped through the **sent** list, i.e. after the gateway's drops (the plan Note's MC-A5 hazard is handled). An index out of range is ignored and named in the detail.
  - Labels run once per id in `marking_photo_ids`.
  - `decide::Evidence::photo_read` reads the single Observe output, with no "first wins" loop.
  - `runs::input_now` takes the chosen photos under the run's limit, so a change beyond the limit is invisible to `arranged`.
  - `RESEARCH_FINGERPRINT_VERSION` = `INSTRUCTIONS_VERSION` = 2.
  - `observe_max_photos`: default 8; `validate` refuses values below 1 and names the field; `ResearchConfig` clamps it to at least 1; snapshots from before R12 get 8 through `serde(default)`.
  - OpenAI `schema_for`: `reads` + `marking_photos` only when the input carries `photos`, otherwise `reads` + `shows_marking`.
  - The carried owner row: `ResearchSettings::default()` holds table A.
- **U2 (A26·C8–C9).** `product_extract` matches the plan's contract. The part order is title, descriptions, stated, page text. Empty parts are dropped, and white-space-collapsed duplicates of an earlier part are dropped. Title follows the `og:title` → Product name → `<title>` rule. Stated lists only `creator`/`manufacturer`/`material`/`productionDate`, as a string or an object's `name`, and never `brand`. The page text starts at the first `<h1>`'s text, or at the start of the visible text, and is cut by character to 10,000. A Product block that does not parse is ignored. The whole extract is cut at 30,000. `read_html` returns the extract as `text`; the title and image paths are unchanged.
- **U3 (A26·C10–C12).**
  - The Read input carries `piece` when the item type is not empty, and the Read identity always carries `item_type`.
  - Only `same_kind == Some(false)` removes the page from Compare: null, a missing answer and a refused call all compare.
  - The read step's detail is exactly "lists another kind of piece (<kind>): not compared". The page still counts toward `max_pages`.
  - The Read schema gains `same_kind` (nullable bool) and `page_kind` (nullable string) only when `piece` is present.
- **U4 tasks 1–2 (A26·C13).** `scratch_settings` serialises `ResearchSettings::default()` (R10's stages, prices and limits). It then sets the two attribution keys, the trusted sites, the `serpapi_lens` prices at 0 and a day cap of `"20"`, and applies the `--settings` patch last. The tool no longer names any model of its own. `accept_settings_owner.json` holds the six left-alone keys, the three hosts and table A.
- **Rows → tests.** Every automated row A26·C1(a)–C13(b), and the carried owner row `a26_owner_detailed_condition_default`, has a test that observes the row's exact outcome at the row's boundary and would fail if that outcome were wrong:
  - exact ids and roles;
  - exact image lists in position order;
  - exact outbound photo set;
  - exact step texts;
  - exact Compare unit lists;
  - exact schema member sets;
  - snapshot fields per stage, with C13(b) checked against its own control.

  **Changed-input probe on A26·C6(b):** I rewrote the test's version-1 fingerprints as version 2, and the test failed on `reused_from_run_id`. So the Find identity it seeds equals the current one, and the version bump is exactly what the test proves. File touched: `crates/research/tests/engine/photo_look.rs`, restored with `git checkout`, sha256 checked byte-identical. No production file was edited, and no database or state outside temp dirs was touched.
- **A26·C7(b) deviation.** I confirmed it as a plan defect: the capture's `output_text` is `{"fields":…, "piece_type":…, "shows_marking":…}`. The test pins what the row protects: the old schema for requests without `photos`, the unchanged refusal of the capture, and acceptance of an answer in that schema. Not blocking.
- **"Tests that pin what this plan changes".** Each was handled as the table says:
  - `a12_c1a–c1c`, `c3a`, `c3b` keep their ids, with one Observe unit;
  - `a14_c1a` deleted (C5(a) replaces it);
  - `a14_c3a` and `a14_c3d` amended to one Observe call over all chosen photos;
  - the Observe and Read rows of the adapter tests unchanged;
  - the tool helpers follow;
  - `adapter_pages` A17·C1's text assertion is now the extract.

  The six other amended tests (`a04_c7a`, `a14_c3c`, `a15_c6a`, `A25·C11(a)`, `a12_c6b`, the `export.rs` helper) each follow from the plan's own rules. I read each one.
- **Assumptions 1–9.** None contradicts the intention: 08B changelog 12 (d), (e), 13, 14 and 15, and MC-A4/A5/A7 as the plan supersedes them. The narrative reports them in plain words.
- **H1–H8 and the service rules.**
  - H1: research writes only the local form, with no external write.
  - H2: no RAW access.
  - H3: no new pixel work; renders are the existing research copies.
  - H4: kill and resume proven by A26·C5(a).
  - H5: no UI automation.
  - H6: nothing needs commerce.
  - H7: no shop concept in `psm_core`. The shop key in the research default is a note.
  - H8: no Tauri in business logic.
  - No live service in any test (fakes and `FakeResearchHttp`), and no secret committed.

## Findings
None blocking (no B1–B4).

## Notes written to backlog
Six notes in `docs/implementation/backlog_research/R12_research_cost.md`, the path the plan's Review mode names for this research track (not `backlog.md`):
1. Plan lesson: A26·C7(b)'s "capture as captured accepted" could not hold (version-0 answer shape).
2. `FakeModel` maps a retired `photo:<id>` Observe script onto the one unit, kept only for two app tests (`r11_arranged::a25_c20b`/`c20e`). Re-script them and remove the fallback at the next phase allowed to touch app tests.
3. Judgment → book: the follow-up trigger compares the chosen photos as a set while the Observe identity is ordered. 08B MC-A7 could state "set plus cover".
4. A page with no visible text but a title or description is now obtained and read. 08A §4 could state it.
5. Multi-shop debt: the default detailed-condition table names the owner's shop field and strings (owner-directed).
6. Plan lesson: the pin table missed tests pinned through bumped constants. Grep the constants too.

**Carry-forward dispositions.**

| Note | Destination |
|---|---|
| 1, 6 | Planner (lessons for research plans) |
| 2 | Next phase allowed to edit `crates/testkit/tests/suite/research/` |
| 3, 4 | Orchestrator's 08A/08B rewrite (changelog 14) |
| 5 | Multi-shop phase, whenever planned |

## Lessons for the plans
- A row that says "answered by the capture as captured" must be checked against the capture's actual `output_text` shape.
- A pin table built by grepping one key misses tests pinned through counts and bumped constants (`"v":1`, `instructions_version`). Grep those too.
- A supersession sentence that names an identity as a trigger ("the stage 1 unit identity") should say whether order counts.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Write perimeter of this session
- `docs/implementation/handoffs/reviewer/handoff_R12_research_cost_review_r1.md` (this file, new)
- `docs/implementation/plans/R12_research_cost.md`: the Tracker line (→ APPROVED) and a new Review log entry
- `docs/implementation/backlog_research/R12_research_cost.md` (new)
- `docs/implementation/narratives/R12_research_cost.md` (new)
- Probe: `crates/research/tests/engine/photo_look.rs`, edited, then restored byte-identical (sha256 checked)
- No commits made; build artefacts under `target/` only.
