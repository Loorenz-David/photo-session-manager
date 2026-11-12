---
phase: R2_research_foundation
title: Research foundation — the small set of shared pieces is put in place and frozen so the engine and the app-side work can then proceed side by side — when two values count as the same, the place for a replaced value, the waiting step of a publish, the research commands' shapes, and the reduced research copy of a photo
difficulty: high
projection: no
depends_on: []
---
# R2 — Foundation: the shapes both tracks build on, frozen first

**Tracker:** NOT_STARTED · 2026-11-12 · planner · —

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. The gate for `depends_on` is the Tracker line of each plan named there. `master_plan.md` §4 is not used for research milestones.)

## Goal
Everything the two tracks share, and nothing else: (U1) name key and sameness in core, because core's publish and form code must tell a conflict; (U2) the storage seam and the frozen shapes — `Value.ai_prior`, the feedback table and publish columns created by an idempotent schema step outside the numbered migration list, `PublishStep::Held` with its status rule, the `ResearchApi` trait and its data shapes in a new crate `crates/research`, the thin research commands and regenerated bindings, the `research_changed` event, `CommerceAdapter::public_listing`, the `research` slot in settings; (U3) the research-copy renderer in infra (the only pixel code, H3 as amended). **After this milestone is approved, the items in `master_plan_research.md` §5 "Frozen by R2" change only through the orchestrator.** **Not here:** any research behaviour (no run, no write, no hold).

## How to work this plan (flow.md §10)
This milestone is one runner phase made of 3 units. Work the units in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. One review, one fix budget and one narrative cover the whole milestone. Row ids keep the number of the superseded plan they were written in (`A07·C1(a)`); `docs/implementation/research_row_mapping.md` says where every row went. Name each test after its row (`a07_c1a_<slug>`). Reviewer notes go to `docs/implementation/backlog_research/R2_research_foundation.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Name key and sameness in core | 2 | 12 | yes |
| U2 | Storage seam and frozen shapes | 5 | 6 | yes |
| U3 | The research copy of a photo (renderer) | 2 | 5 | yes |
| **Total** | | **9** | **23** | |

## Read first (whole milestone)
`docs/process/flow.md` (**§10 is the authority for this track**) · `docs/implementation/master_plan.md` §3, §6, §7 · `docs/implementation/master_plan_research.md` (all: layout §0, standing rules §3, sequencing and tracks §5, registry §6, commands §7.2, the storage decision §6.4) · book `01`, `14` §Research agent, `08A`, `08B` · then each unit's own Read first. Name key and sameness are `psm_core::mechanisms::{name_key, sameness}` (R2·U1); every other research rule is `psm_research::rules`.

## Unit 1 — Name key and sameness in core

**Unit id:** `R2·U1` · **Replaces:** A03 criteria C1 and C7 · **Changes app files:** yes

### Goal
The name key `N` (MC-A2) and the sameness relation `S` (MC-A1) as pure functions in `psm-core`, because core services use them to tell a conflict (MC-A10, MC-A12) and the research crate uses them for everything else.

### Read first
book `08B` MC-A2 (Name key), MC-A1 (the sameness table), `05A` MC-9 · code `crates/core/src/mechanisms/values.rs` (`canonicalize`, `canonical_decimal`), `crates/core/src/domain/attribute.rs` (`Definition`, `Definition::units`).

### Files
New: `crates/core/src/mechanisms/{name_key.rs, sameness.rs}`. Edited: `crates/core/src/mechanisms/mod.rs`, `crates/core/Cargo.toml` (two Unicode crates), `Cargo.lock`, `master_plan_research.md` §6.2 (the pins).

### Core mechanisms (the only functions with unit tests)
`name_key::name_key` · `sameness::same` (one inline test per row, named `a03_c1a_<slug>` …).

### Ordered tasks
1. Pick and pin the crates for the full Unicode case fold and for a character's general category (`master_plan_research.md` §6.2); verify with `cargo generate-lockfile`; record the versions. NFKC comes from `unicode-normalization`. No hand-written Unicode tables.
2. `name_key(&str) -> String`: NFKC → full case fold → every character of general category P\* or S\* becomes a space → runs of white space collapse to one space → trim. `RESEARCH_NORMALISE_VERSION = 1`.
3. `same(&Definition, a, b) -> bool`: the MC-A1 table. For `dimension` and `weight` the two values are converted to one unit with exact integer arithmetic (1 in = 25.4 mm, 1 lb = 453.59237 g; **no floating point**). The seven neutral unit names (`mm, cm, m, in` / `g, kg, lb`) are found through `definition.validations.unit_names`, an object `{neutral name: the definition's unit code}` that the commerce adapter fills at schema import (R8·U3) and tests build by hand (H7: no platform unit code in core). Two values in the same unit code compare without it; values in different codes with no `unit_names` entry are not the same.

### Criteria (2 criteria, 12 rows)
Boundary: the pure functions. Unit names for C7(d) come from the definition's `validations.unit_names`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A03·C1(a) | `name_key` | `Hans J. Wegner` | `hans j wegner` | MC-A2 |
| A03·C1(b) | `name_key` | `  Børge   Mogensen ` and `Borge Mogensen` | `børge mogensen`; the two keys differ | MC-A2 |
| A03·C1(c) | `name_key` | `Ｗｅｇｎｅｒ` (full-width letters), `ﬁnn` (ligature) | `wegner`, `finn` | MC-A2 |
| A03·C1(d) | `name_key` | `STRASSE`, `Straße` | both `strasse` | MC-A2 |
| A03·C1(e) | `name_key` | `Fritz-Hansen & Co.`, `A/S` | `fritz hansen co`, `a s` | MC-A2 |
| A03·C1(f) | `name_key` | `Hans J. Wegner`, `Hans Wegner` | different keys (no alias table) | MA11 |
| A03·C1(g) | `name_key` | `...`, the empty string | the empty string | MC-A2 |
| A03·C7(a) | `same`, text | `Hans J. Wegner` / `hans j wegner`; `Hans J. Wegner` / `Hans Wegner` | same; not same | MC-A1 |
| A03·C7(b) | `same`, choice | `Teak` / `Teak`; `Teak` / `teak` | same; not same (canonical equality) | MC-A1 |
| A03·C7(c) | `same`, multi_choice | `["Oak","Teak"]` / `["Teak","Oak"]`; `["Oak"]` / `["Oak","Teak"]` | same; not same | MC-A1 |
| A03·C7(d) | `same`, dimension | 25.4 mm / 2.54 cm; 25 mm / 2.54 cm | same; not same | MC-A1 |
| A03·C7(e) | `same`, boolean and integer | `true` / `true`; `12` / `13` | same; not same | MC-A1 |

### Delegated choices
The two crates (recorded) · the internal number representation (integers only) · whether `tags` gets a `same` arm now (it compares name keys as sets).

### Notes
- Hazard: an empty name key (C1(g)) must never be treated as equal to another empty key by a caller choosing a choice; R3·U1's mapping fails on it.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-core mechanisms::name_key::tests` · `cargo test -p psm-core mechanisms::sameness::tests` · `cargo clippy -p psm-core --all-targets -- -D warnings` · `cargo fmt --all -- --check`

## Unit 2 — Storage seam and frozen shapes

**Unit id:** `R2·U2` · **Replaces:** A07 criterion C7; A09 criterion C5; new rows F·C1–F·C3 · **Changes app files:** yes

### Goal
Put in place, without any behaviour, the shared shapes and seams listed in the milestone goal, so that R3/R4 (engine track) and R5/R6 (app track) need no further change to a shared file's public shape.

### Read first
`master_plan_research.md` §5 (Frozen by R2), §6.3, §6.4 (storage decision), §6.6 · book `08B` MC-A1 (`ai_prior`), MC-A10 (`held`, status and label), MC-A12 (feedback fields), MC-A6 (prerequisites, for the status shape), `05A` MC-7 and its research pointer · code `crates/infra/src/sqlite/store.rs` (`open_with_default_shape` line 44–58, `transaction` line 1547, `SqliteTx`), `crates/core/src/domain/{attribute.rs, session.rs, publish.rs}`, `crates/core/src/mechanisms/status.rs`, `crates/core/src/{events.rs, ports/commerce.rs}`, `crates/infra/src/settings_file.rs`, `crates/integrations/src/lib.rs` (`App`, `AppHandles`), `apps/desktop/src-tauri/src/{commands/mod.rs, lib.rs (collect_commands), state.rs}`, `apps/desktop/src/services/{bindings.ts, events.ts}`, `crates/testkit/src/fake_commerce.rs`.

### Files
New: `crates/research/{Cargo.toml, src/lib.rs, src/api.rs}` (package `psm-research`; workspace `members` gains it), `crates/infra/src/sqlite/ensure.rs`, `apps/desktop/src-tauri/src/commands/research.rs`, `apps/desktop/src/services/research.ts`, `crates/testkit/tests/suite/research/{mod.rs, r2_foundation.rs}` and the one line `mod research;` in `crates/testkit/tests/suite/main.rs`.
Edited: `crates/core/src/domain/{attribute.rs (Value.ai_prior, AiPrior), research.rs (new: Feedback, FeedbackAction), mod.rs, session.rs (PublishStep::Held, RowLabel::WaitingForYou), publish.rs (PublishItem's four fields)}`, `crates/core/src/mechanisms/status.rs`, `crates/core/src/events.rs`, `crates/core/src/ports/{commerce.rs, store.rs}`, `crates/infra/src/sqlite/{mod.rs, store.rs}`, `crates/infra/src/settings_file.rs` (`research: Option<serde_json::Value>`), `crates/integrations/{Cargo.toml, src/lib.rs}` (`research: Option<Arc<dyn ResearchApi>>`, absent), `apps/desktop/src-tauri/{Cargo.toml, src/commands/{mod.rs, attributes.rs}, src/lib.rs, src/state.rs}`, `apps/desktop/src/services/{bindings.ts (regenerated), events.ts}`, `crates/testkit/src/fake_commerce.rs`, every Rust and TypeScript place that builds a `Value` literal or matches on `PublishStep` / `RowLabel`, test fixtures that list the fields or tables.

### Core mechanisms (the only functions with unit tests)
`mechanisms::status::{derive_status, row_label}` (existing, extended with `held`).

### Ordered tasks
1. **Schema step, outside the numbered list** (`master_plan_research.md` §6.4). `sqlite/ensure.rs`: `ensure_research_columns(conn)`, run by `SqliteStore::open…` right after `to_latest`, on every open, idempotent: add `attribute_values.ai_prior` if missing; add `publish_items.{frozen_attribute_origin, pending_conflicts, resolved_attributes, released_at}` if missing; `CREATE TABLE IF NOT EXISTS research_feedback(…)` with its index. `SqliteStore::migrations()` is **not** touched.
2. **Storage seam.** `SqliteStore::research_transaction(&self, op)`: one transaction in which the closure gets both the `StoreTx` and the raw connection of that same transaction (for example a `&mut SqliteTx` with a public `conn()`); commit on `Ok`, roll back on `Err`. The research crate's own tables are read and written through it (R4·U1).
3. **Shapes.** `Value.ai_prior: Option<AiPrior {value, valid, provenance}>` end to end (store, DTO); existing writers write null. `PublishItem` gains the four fields (empty). `PublishStep::Held` (ordered before `Prepare`; never an active step for the startup re-queue) and `RowLabel::WaitingForYou(n)`; `derive_status` counts `held` as publishing; the label is placed before `Publishing n%`, n = pending keys without a resolution, passed in `StatusInput`. `Feedback`, `FeedbackAction` in `psm_core::domain::research`, with store methods to insert, list by `(item, key)`, delete and re-insert by `step_token`. `AppEvent::ResearchChanged {session_id}` forwarded as the Tauri event `research_changed`. `CommerceAdapter::public_listing(external_id) -> PublicListing {hosts, item_url, item_path_tail}` with a default body returning empty values, and `FakeCommerce::set_public_listing`.
4. **`crates/research`** with `api.rs` only: the trait `ResearchApi` and its plain data shapes exactly as `master_plan_research.md` §6.6 lists them (`ResearchStatus`, `StepView`, `RunView`, `Reason`, `SecretsInput`, `SecretsState`, `ExportSelection`), every method of the trait included. It depends on `psm-core`; nothing implements the trait yet.
5. **Commands** (`commands/research.rs`, thin): `research_status`, `research_run`, `research_secrets_set`, `research_secrets_state`, `research_site_add`, `research_site_remove`, `research_sites_reorder`, `research_export`. Each calls `App.research`; when it is absent the command returns the error code `research_unavailable`. Regenerate `bindings.ts`; add `services/research.ts` wrappers.

### Criteria (5 criteria, 6 rows)
Boundary: `SqliteStore::open` on a temp folder, `mechanisms::status`, the session list, the command handlers. Rows `F·…` are new in this plan (the freeze); the others moved here from the superseded A07 and A09.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A07·C7(a) | migration | a database at the last numbered migration holding one `attribute_values` row; open the store (the schema step runs); close and open it again | the row is intact with `ai_prior` null; `research_feedback` exists and is empty; `publish_items` has the four columns; the second open changes nothing and `SqliteStore::migrations()` has the same length as before this unit | MC-A1 |
| A09·C5(a) | `derive_status`, `row_label` | latest step `held`, two pending keys, one resolved | `Publishing`; `WaitingForYou(1)` | M14 |
| A09·C5(b) | session list | a session whose latest item is `held` | not offered Publish again (status `publishing`) | M14 |
| F·C1(a) | research commands with no engine wired | `research_status`, `research_run`, `research_secrets_state`, `research_site_add`, `research_export`, each on an app whose `research` service is absent | each returns the error code `research_unavailable`; no table changes; every other command of the app behaves as before | MA7 (HA4) |
| F·C2(a) | `SqliteStore::research_transaction` | one closure that writes a row through the raw connection (a scratch table made in the test) and an attribute value through `StoreTx`; a second closure that does both and then returns an error | first: both writes are visible after the call; second: neither is | MA14 (H4) |
| F·C3(a) | `CommerceAdapter::public_listing` | the default body on an adapter that does not override it; `FakeCommerce` after `set_public_listing(hosts, item_url, path_tail)` | empty hosts, no URL, empty tail; exactly the values set | MC-A3 |

### Delegated choices
How the raw connection is handed to the closure · JSON layouts of the three new `publish_items` columns (key → object; R5·U2 fills them) · the progress percentage of `held` (0) · how `StatusInput` carries the unresolved count. Not delegated: the names and fields of `master_plan_research.md` §6.6, and that nothing is added to the numbered migration list.

### Notes
- Hazard (earned in P67): a new field on `Value` changes many struct literals and one generated DTO; no test's expectation changes except to name the new field. List the touched test files under Deviations.
- Hazard: `Held` reaches every exhaustive `match` on `PublishStep` (`services/publish.rs` lines 114, 264–281, 767–813; `status.rs`; the TypeScript unions). Until R5·U2 nothing produces it.
- A later numbered migration that recreates `attribute_values` or `publish_items` must carry these columns; the schema step would re-add a missing column empty.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-testkit --test suite research::r2_foundation::` · `cargo test -p psm-testkit --test suite p01_storage_identity::` · `cargo test -p psm-testkit --test suite p03_session_state_list::` · `cargo test -p psm-core mechanisms::status` · `cargo test -p psm-desktop export_bindings` · `cd apps/desktop && npm run typecheck` · `cargo check --workspace`

## Unit 3 — The research copy of a photo (renderer)

**Unit id:** `R2·U3` · **Replaces:** A10 criteria C1 and C2 · **Changes app files:** yes

### Goal
`Imaging::render_research`: the composition rule applied to the adjusted preview, resized, encoded once, with all metadata removed (H3 as amended, 08B MC-A5 Source). It is the only new pixel code of the research work and lives in infra, where all pixel code lives.

### Read first
book `01` (**H3 as amended**), `08B` MC-A5 (Source), `10D` MC-27 (pixel rule), MC-28 (render contract) · `master_plan.md` §6.4 (`Imaging`), §6.10 · code `crates/core/src/ports/images.rs`, `crates/infra/src/imaging/{render.rs, linear.rs, metadata.rs}`, `crates/testkit/src/fixtures/` (`SyntheticJpeg`), `TestApp::decode`.

### Files
New: `crates/infra/src/imaging/research.rs`. Edited: `crates/core/src/ports/images.rs` (`Imaging::render_research(preview, values, long_edge, dest) -> Result<OutputInfo>`; `NoImaging`), `crates/infra/src/imaging/mod.rs`, `crates/testkit/tests/suite/research/{mod.rs, r2_foundation.rs}`.

### Core mechanisms (the only functions with unit tests)
None new. The pixel rule is `psm_core::mechanisms::composition` (existing).

### Ordered tasks
1. `render_research`: decode the adjusted preview JPEG; apply the MC-27 pixel rule for the given composition values (turn, flip, straighten, cut); resize so the long edge is `min(long_edge, the cut's long edge)` (never upscale); encode once as JPEG quality 85, sRGB, with no EXIF, GPS, maker notes or other metadata; write only `dest`.
2. It never decodes a RAW or a master, and nothing but a research copy is produced by it.

### Criteria (2 criteria, 5 rows)
Boundary: `Imaging::render_research` of the real infra implementation on synthetic previews in a temp folder; outcome = the file, decoded with the testkit's `decode`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A10·C1(a) | render | a 3000 × 2000 adjusted preview with corner markers, default composition; long edge 1024 | a JPEG 1024 × 683 (±1 px on the short edge) showing the whole frame (the four markers in their corners) | MC-A5 |
| A10·C1(b) | render | the same preview, a composition cutting the right half, one quarter turn | the copy shows only the right half's markers, turned | MC-A5 (H3) |
| A10·C1(c) | render | an 800 × 600 preview; long edge 1024; long edge 2048 | both 800 × 600 (never upscaled) | MC-A5 (R-IA13) |
| A10·C1(d) | render | a 6000 × 4000 preview; long edge 2048 | 2048 on the long edge | MC-A5 |
| A10·C2(a) | metadata | a preview carrying EXIF with camera model, serial number, GPS and a maker note | the copy has no EXIF segment (the testkit's EXIF reader finds none) and no GPS data | MA12 (R-IA13) |

### Delegated choices
The JPEG encoder call (`jpeg-encoder` as P29) · chroma subsampling.

### Notes
- Rule 17: "no EXIF" is observed with `kamadak-exif` (a dev-dependency of the testkit); check what that reader returns for a JPEG without an APP1 segment before writing C2(a).

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-testkit --test suite research::r2_foundation::` · `cargo test -p psm-testkit --test suite p28_composition_foundation::` · `cargo test -p psm-testkit --test suite p29_master_and_output_render::`

## Closing gate (once, after the last unit; replaces the template's closing gate)
This milestone changed app files, so the full gate runs once: `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer runs: `cargo test -p psm-core mechanisms::name_key::tests`, `cargo test -p psm-core mechanisms::sameness::tests`, `cargo test -p psm-testkit --test suite research::r2_foundation::`, `p01_storage_identity::`, `p03_session_state_list::`, `p10_attributes::`, `p28_composition_foundation::`, `p29_master_and_output_render::`, `cargo test -p psm-research`, clippy, fmt, typecheck; the implementer's full-suite stamp stands. Checks by reading that the frozen list of `master_plan_research.md` §5 matches what was built, name for name. Notes: `docs/implementation/backlog_research/R2_research_foundation.md`.

## Manual try-out (the narrative is built from this)
Nothing new is visible. On scratch data (`cd apps/desktop && PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev`) open a form and a publish review: existing values, defaults and "from Shopify" values show as before, and publishing an item works as before.

## Review log
