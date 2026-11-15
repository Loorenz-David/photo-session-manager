---
plan: R2_research_foundation
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-12
actor: opus
---

# R2 research foundation — review r1

**Verdict: APPROVED_WITH_NOTES.** All 23 criterion rows (9 criteria, 3 units) hold at their boundaries. The production code matches the plan, the book contracts it cites (08B MC-A1, MC-A2, MC-A5, MC-A10, MC-A12 shapes) and the frozen list in `master_plan_research.md` §5, name for name. There are no blocking findings (B1–B4). Three notes went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)

None. Nothing needs the owner.

## Verified correct

**Gate.** Tracker read `IMPLEMENTED`. Checkpoints `290fcf3` (U1), `879b2f0` (U2), `184e61c` (U3) and closing `9dccd32` exist in `git log`.

**Perimeter.** `git diff --name-only 290fcf3^ 9dccd32` equals the handoff's 52 declared files exactly: no extra paths, no missing ones. `commands/sessions.rs` and `p14_publish_update.rs` are outside the plan's Files list, but both fall under the plan's allowance ("every … place that … matches on `PublishStep` / `RowLabel`", "test fixtures that list the fields").

**Checks run (plan Review mode, one cargo command at a time, foreground):**
| Command | Result |
|---|---|
| `cargo test -p psm-core --lib -- mechanisms::name_key::tests mechanisms::sameness::tests mechanisms::status` | 12 passed |
| `cargo test -p psm-testkit --test suite -- research::r2_foundation:: p01_storage_identity:: p03_session_state_list:: p10_attributes:: p28_composition_foundation:: p29_master_and_output_render::` | 135 passed (10 R2 rows included) |
| `cargo test -p psm-research` | builds; 0 tests (api only, as planned) |
| `cargo test -p psm-desktop --lib -- r2_research_command_tests export_bindings` | 2 passed; regenerated `bindings.ts` is byte-identical (tree clean afterwards) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cd apps/desktop && npm run typecheck` | clean |
The implementer's full-suite stamp (822 suite tests, 466 UI tests, on `184e61c`) stands. The tree is unchanged since then except for docs.

**Row by row (production read first, then the test):**
- **A03·C1(a–g), `name_key`.** The order is NFKC → `case_fold` (full) → P\*/S\* and white space to a single space → trimmed (the leading space is suppressed, the trailing one never emitted). There is no alias table. `RESEARCH_NORMALISE_VERSION = 1`. Each test asserts the row's exact strings. C1(d) asserts that the two keys are equal, which fails under simple lowercasing (`straße` ≠ `strasse`).
- **A03·C7(a–e), `same`.** It follows the MC-A1 table: text/textarea/url by `N`; choice by canonical equality; multi_choice as sets of canonical strings; tags as sets of `N` keys (delegated arm); dimension/weight by exact `i128` cross-multiplication (in = 254/10 mm, lb = 45 359 237/100 000 g, overflow → not same). Neutral units are found only through `validations.unit_names` (H7), and the same unit code compares without them. It reads the MC-9 stored shape `{unit, value: "<decimal string>"}` (checked against `values.rs`). The tests assert both the "same" and the "not same" pair of each row.
- **A07·C7(a), schema step.** `ensure_research_columns` runs after `to_latest` on every open, outside `migrations()`. It adds a column only if it is missing, and uses `CREATE … IF NOT EXISTS`. The test builds a database at the last numbered migration holding a row, opens it twice, and checks after each open that the row is intact with `ai_prior` null, the feedback table is empty and the four columns are present. It also checks that `migrations().len()` is unchanged. A second open that was not idempotent would fail at `open().unwrap()`.
- **A09·C5(a)/(b), held status.** `Held` counts as publishing in `derive_status`. `row_label` returns `WaitingForYou(n)` before `Publishing n%`. The session list derives n as the pending keys without a resolution. The progress percentage for `held` is 0. Per 08B MC-A10 "Restart", `Held` is not in `active()`, so `run_item` returns at once, and it is not in the startup re-queue `step IN ('prepare','upload','update','describe')`. C5(b) goes through the real store and the session list (2 pending, 1 resolved → `WaitingForYou(1)`, status `Publishing`).
- **F·C1(a), unwired commands.** The eight commands are thin (H8): `engine()` → `App.research`, `None` → `research_unavailable`. `App::build` sets `research: None`. The test runs five commands through Tauri's IPC mock, plus `session_active` as the "other command" control. The attribute row count is unchanged.
- **F·C2(a), `research_transaction`.** It is one `rusqlite::Transaction`. `SqliteTx::conn()` returns that same transaction, so the raw SQL and the `StoreTx` writes share one commit. `?` on `Err` drops the transaction, which rolls it back. The test observes both writes after the commit, and neither after the rollback (scratch table and attribute value).
- **F·C3(a), `public_listing`.** The default body returns `PublicListing::default()`, and `ShopifyAdapter` does not override it (the test uses it as the non-overriding adapter). `FakeCommerce::set_public_listing` stores the values and the override returns them.
- **A10·C1(a–d), C2(a), `render_research`.** It reuses the MC-28 renderer (decode, MC-27 turn/flip/straighten/cut, Lanczos resize only when the cut's long edge exceeds the target, so it never upscales; one encode at q85, sRGB). It then strips the APP0–APP15 and COM segments without re-encoding and writes `dest` atomically. The scratch file is removed. The source must be JPEG, and nothing else is accepted (H3 as amended). C1(b) tells the turn direction and the turn's presence apart by which corner colours survive and by the 1024×768 output (an unturned right half would be 768×1024). C2(a) first proves that the source carries Model, BodySerialNumber, MakerNote and GPS. It then shows that `kamadak-exif` finds no EXIF in the copy and that the raw bytes contain none of the secrets.

**Frozen list (§5) against what was built, name for name:** `name_key::name_key`, `sameness::same` · `Value.ai_prior`, `AiPrior {value, valid, provenance}` · `domain::research::{Feedback, FeedbackAction}` + `feedback_by_key`, `insert_feedback`, `delete_feedback_by_step`, `reinsert_feedback` · `PublishItem.{frozen_attribute_origin, pending_conflicts, resolved_attributes, released_at}` · `PublishStep::Held`, `RowLabel::WaitingForYou(i64)` · `AppEvent::ResearchChanged {session_id}` → `research_changed` (listened to in `events.ts`) · `CommerceAdapter::public_listing`, `PublicListing {hosts, item_url, item_path_tail}` · `Settings.research: Option<Json>` · `SqliteStore::research_transaction` · `Imaging::render_research(preview, values, long_edge, dest)` · `psm_research::api::ResearchApi` with all 12 §6.6 methods and `ResearchStatus`, `StepView`, `RunView`, `Reason`, `SecretsInput`, `SecretsState`, `ExportSelection {Range{from,to} | Items(ids)}` · the eight commands in `collect_commands` and `bindings.ts`, with wrappers in `services/research.ts`. All present.

**Assumptions (handoff).** None contradicts the book: pinned crates; overflow counts as unequal; empty `{}` JSON maps; the n derivation; string display fields until the engine's enums exist; the settings slot omitted when unset; reuse of the renderer plus a segment strip; the quarter-turn reading under MC-27. They are reported to the owner in plain words in the narrative.

**Hard constraints H1–H8.** No violation. H1/H6: no integration call is added. H2: no RAW is touched. H3: the preview path is used only for the research copy, with frame/resize/single encode. H4: the schema step is idempotent and the transaction seam is atomic. H5: n/a. H7: unit names come from definition data and `PublicListing` is platform-neutral. H8: the commands are thin and no logic depends on Tauri.

## Findings

None blocking. No B1 (behaviour matches the plan and book at every row), no B2 (each row has a discriminating test at its boundary), no B3, no B4 (all checks green).

## Notes written to backlog

In `docs/implementation/backlog_research/R2_research_foundation.md`, the place the plan's Review mode names. A one-line pointer was appended to `docs/implementation/backlog.md`.
1. `StatusPill.tsx` has no `waiting_for_you` branch, so a held row renders an empty status cell. It is unreachable until R5·U2. → **R6**.
2. The held conflict count is derived twice: the session list uses pending keys without a resolution, and the publish activity text uses `pending.len() - resolved.len()`. → **R5·U2** should use one helper.
3. The research copy has no ICC profile, because the strip removes APP2 too. This meets MC-A5. Keep APP2 if a provider misreads colours. → no phase; recorded only.

**Carry-forward dispositions:** note 1 → R6 · note 2 → R5·U2 · note 3 → backlog only.

## Lessons for the plans
- The prompt's step 6 names `docs/implementation/backlog.md`, but the R-plan's Review mode names `backlog_research/<plan>.md`, which did not exist yet. The research prompt template should name the research backlog file.
- A07·C7(a)'s wording "the second open changes nothing" is observable only through the same fields as the first open. That was enough here; future schema-step rows could name the observable directly (for example, the column list is unchanged).

## Write perimeter (this review)
- `docs/implementation/handoffs/reviewer/handoff_R2_research_foundation_review_r1.md` (new)
- `docs/implementation/narratives/R2_research_foundation.md` (new)
- `docs/implementation/backlog_research/R2_research_foundation.md` (new)
- `docs/implementation/backlog.md` (one line appended)
- `docs/implementation/plans/R2_research_foundation.md` (Tracker line; Review log entry appended)

No production or test file was edited, not even temporarily. No database or state outside temporary test folders was touched. Running `export_bindings` rewrote `bindings.ts` byte-identically, and `git status` shows no change to it. Nothing was committed: the runner commits.
