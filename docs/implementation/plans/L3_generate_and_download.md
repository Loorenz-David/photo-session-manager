---
phase: L3_generate_and_download
title: Generate and download price labels — "Generate label" (or "Reprint label") in every session's ⋯ menu and "Generate labels for N" in the Labels box open a check that shows, per template, what will happen to each product and a picture of the real label; Generate PDF makes the labels in Pricing with one run key per chunk of 300 that every retry reuses, the result names every failure, and Download PDF saves the file in Documents › Session Manager Labels and opens it in Preview
difficulty: high
projection: done (r0 amendments applied 2026-11-23)
depends_on: [L2_labels_box]
blocked_on: none
---
# L3 — Generate and download price labels

**Tracker:** IMPLEMENTED (2026-11-23, opus, 4 units checkpointed 00dd4e2f / 51815be6 / 9acd03f1 / 4ec02089; closing gate green: workspace suite, clippy, fmt, typecheck, npm test 935)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `09C` §6 and §10 by contract **MC-L2** (`docs/implementation/master_plan_labels.md` §5), serving **M-L2, M-L3, M-L4, M-L5**. (U1) The check (chunked, merged, grouped by template), the preview picture, the label files on disk. (U2) Generating: one key per chunk, retries with the same key, Try again, the double press, the result. (U3) Download into the Labels folder and Preview; the commands. (U4) The check and done dialogs; the ⋯ entry and the batch bar wired; the errors of 09C §10 on screen. **Not here:** choosing a template (09C L8), printing to a printer, batch labels from Open or Published, a history of runs (09C §11); **the first real Generate, which is the owner's** (09C §13).

## How to work this plan (flow.md §10, master_plan_labels.md §3)
One runner phase of 4 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `L3·C<n>(<row>)`; tests named after their row (`l3_c1a_<slug>`). Open details: the smallest option consistent with 09C §6 and MC-L2, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_labels/L3_generate_and_download.md`. **Never call Pricing, Shopify or a CDN, never launch the built app, never open Preview and never write into the real Documents folder from a test** (master plan §3: every `TestApp` uses `FakeLabelFiles` and its own temporary home).

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The check, the preview picture, the label files | 4 | 27 | **yes** |
| U2 | Generating: keys, retries, the result | 5 | 24 | **yes** |
| U3 | Download and the commands | 2 | 24 | **yes** |
| U4 | The dialogs, the ⋯ entry and the batch bar wired | 6 | 32 | **yes** |
| **Total** | | **17** | **107** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §10) · `docs/implementation/master_plan_labels.md` §3 (Rule 9, Rule 17, the preview probe), §4 (Vocabulary L3, Errors, Pure rules, Ports `LabelFiles`, Files, Commands, Words, Tests), **§5 MC-L2 (every item), ACC.1, ACC.4, MC-L3.2**, §6 · book `09C` §6, §9 (M-L2…M-L5), §10, §12 scenarios 3–7, 10, 12 · the Pricing package (read-only): `LABELS.md` §3.3–§3.5, §5, §6, `API.md` › `POST …/labels/plan`, `…/preview`, `…/runs`, `GET …/artifacts/…` · the L1 and L2 plans (what the adapter, `LabelsService`, `FakePricing`, the pill, the ⋯ item and the Labels box already offer) · `docs/guide/03_rules_and_principles.md` (H1, H4, H7, CR2).

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **The done dialog counts the products the check refused as failed** ("2 of 3 labels generated"), although only the products that passed the check are sent to Pricing.
- **A dropped connection during Generate is retried twice at once with the same run key;** after that the dialog offers Try again, which continues where it stopped. Labels already generated are never generated again by a retry.
- **Leaving a batch after a failed part** shows what was generated so far; the rest is listed as not generated and stays selected.
- **Run keys are not kept across a quit:** generating again after a quit makes a new run, and products already marked come back as reprints.
- **The preview is a picture of the PDF's page** made on this Mac, shown on white; if it cannot be made the dialog says so and Generate still works.
- **A PDF whose name is taken gets " (2)", " (3)" …** and always lands in Documents › Session Manager Labels.
- **If Preview cannot be opened** the PDF is still saved and the app says where.
- **macOS asks once** to let Session Manager use the Documents folder, the first time a PDF is saved.
- **Products are always sent to Pricing by their product id;** Pricing's template choice is never overridden.
- **Outcome words in the check list:** "Will be marked Printed", "Reprint · stays Printed", "No price · stays Not printed", "Can't generate".

## Unit 1 — The check, the preview picture, the label files

**Unit id:** `L3·U1` · **Rows:** L3·C1–C4 · **Changes app files:** yes

### Goal
`LabelsService::{check, preview}`, `merge_plans`, `group_check`, the `LabelFiles` port with `DiskLabelFiles` and `FakeLabelFiles`, `label_file_name` / `numbered_name`, and the fake Pricing's plan and preview.

### Read first
Master plan §5 **MC-L2.1–.5, .13**, §4 Files, §3 "The preview picture is verified" · `crates/core/src/services/labels.rs` (L1, L2) · `crates/core/src/mechanisms/labels.rs` (`chunks`, `LabelCopy`) · `crates/core/src/ports/labels.rs` · `crates/integrations/src/labels.rs` (L1: `build_service`) · `crates/integrations/src/conversations/mod.rs` (`with_test_conversations` :156: the test-seam pattern) · `crates/infra/src/paths.rs` (`AppPaths.home` :10, `create` :31) · `apps/desktop/src-tauri/src/lib.rs` (`asset_protocol_scope` around :3127: only `previews` and `templates` are readable by the webview) · `crates/testkit/src/test_app.rs` (`create_inner` :490-502: the temporary home) · `crates/testkit/src/fake_pricing.rs` (`fetch` :1147, the scripted failures :787-850, `hold` :853) · the package's `API.md` plan and preview sections.

### Files
New: `crates/core/src/services/label_runs.rs`, `crates/infra/src/label_files.rs`, `crates/testkit/src/fake_label_files.rs`, `crates/testkit/tests/suite/l3_labels_check.rs`. Edited: `crates/core/src/{domain/label.rs, mechanisms/labels.rs, ports/labels.rs, services/mod.rs, services/labels.rs}`, `crates/infra/src/lib.rs`, `crates/integrations/src/{labels.rs, lib.rs}`, `crates/testkit/src/{lib.rs, fake_pricing.rs, test_app.rs}`, `crates/testkit/tests/suite/main.rs`.

### Core mechanisms (the only functions with unit tests)
`mechanisms::labels::{merge_plans, group_check, label_file_name, numbered_name}`.

### Ordered tasks
1. Vocabulary of L3 (master plan §4: `CheckGroup`, `LabelCheck`, `LabelFailure`, `LabelRunResult`, `SavedLabel`, the `LabelsFailure` variants `Busy`, `NothingToGenerate`, `NotFound`, `PreviewFailed`, `SaveFailed`) and the constants `RUN_CHUNK`, `RUN_RETRIES`, `PREVIEW_EDGE`.
2. `merge_plans` (MC-L2.1), `group_check` (MC-L2.2, .4), `label_file_name` and `numbered_name` (MC-L2.13), with unit tests: two plans merged (positions renumbered, sums, sizes merged in first-appearance order, unresolved concatenated, a `NoProducts` chunk turned into `Unresolved {message: "Not found"}`; **`position` := the item's index among the merged items, 0, 1, 2, … in merged order, independent of Pricing's own numbering** (MC-L2.1 as amended; projection r0, D4), with one case where chunk 1 has an unresolved entry and chunk 2's first item still gets the next index after chunk 1's last item); groups in first-appearance order with the no-template group last and product types joined by template id; `label_file_name` of `"a/b/c.pdf"` → `c.pdf`, `"..\\x.PDF"` → `x.PDF`, `"report"` → `report.pdf`, `""` and `".pdf"` and `"../"` → `labels.pdf`, `"a:b\u{7}.pdf"` → `ab.pdf`, a 200-character stem → its first 120 characters + `.pdf`; `numbered_name("a.pdf", 2)` → `a (2).pdf`.
3. `ports::labels::LabelFiles`; `psm_infra::label_files::DiskLabelFiles` (master plan §4 Files, MC-L2.13; **`raster` writes its temporary PDF inside `<previews>/labels/` as `<name>.src.pdf` and always removes it**, on success and on failure alike, never in the system's temporary folder: projection r0, D1); `psm_testkit::fake_label_files::FakeLabelFiles` (saves through a `DiskLabelFiles` on the test's home; `raster` writes a fixed 1 × 1 PNG to `<previews>/labels/<name>.png` and records the bytes it was given; `open` records the path; `clear_previews` delegates to its inner `DiskLabelFiles::clear_previews` (D12); `fail_raster_next()`, `fail_save_next(reason)` (**answers `Error::Io(io::Error::other(reason))`**, D2), `fail_open_next()`; `rasters()`, `opened()`). **The reason text of `SaveFailed` and `PreviewFailed`** (projection r0, D2): for `Error::Io(e)` it is `e.to_string()` (the inner `io::Error`'s text, so `fail_save_next("disk full")` gives `SaveFailed("disk full")`), for any other error its `Display`; never `error.to_string()` on an `Io` (that gives `I/O: disk full`); `psm_integrations::labels::with_test_label_files`; `App::build` hands the service `DiskLabelFiles::new(paths.root.join("previews"), paths.home.join("Documents/Session Manager Labels"))` unless a test seam is installed; **`TestApp` installs `FakeLabelFiles` in every constructor** and offers `label_files()`.
4. `label_runs.rs`: `check` (MC-L2.1–.4) and `preview` (MC-L2.5). The kept check lives in the service's memory (one slot).
5. `FakePricing` (master plan §4 Tests, the **L3** list, plan and preview part): `POST /labels/plan` (per product: **its template is `LabelSpec.template` (an assigned template id), else the template whose `product_types` holds the product's type, else the default, else none** (projection r0, D8; L3·C2(b), C2(d)), `outcome` by the package's rules — failing when `fail_label` names it or no template applies, `reprint` when printed, `noPrice` when unpriced, else `markPrinted`; an id it does not hold → `unresolved` with `product_not_found`; none resolved → 422 `no_products` with `details.unresolved`; `summary` computed; **`sizes` covers only the items whose outcome is not `fail`**, `count` = those items of that size (`API.md`: "one PDF per size among the ready items"; L3·C11(a) `count: 2`, D8)), `POST /labels/preview` through `fetch` (`%PDF-1.4 fake preview <productId>`; 404 `product_not_found` for an unknown id; 422 `no_template` when none applies). The scripted failures, `timeout_next`, `drop_answer_next` and `hold` act on `fetch` as on `send`.
6. Tests for the rows.

### Criteria (4 criteria, 27 rows)
Boundary for C1–C3: `App.labels` on `TestApp::with_pricing()`, connected and `Allowed`; outcome = the returned value, `FakePricing::request_lines()` and request bodies, `FakeLabelFiles`' records, files on disk. For C4: `DiskLabelFiles` on a temporary folder (inline tests in `label_files.rs`); outcome = the files. "Pn" as in the L2 plan; the fake has a default template `Label 70×100` (70 × 100 mm) unless the row says otherwise.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| L3·C1(a) | `check(["1", "2", "3"])` | P1 priced and not printed; P2 without a price; P3 with `fail_label("3", "missing_code_value", "Barcode: no barcode.")` | exactly one `POST /stores/<id>/labels/plan` whose body's items are product ids 1, 2, 3 in that order, and one `GET …/labels/templates`; the check's `summary` = `{items 3, ready 2, failing 1, mark_printed 1, reprint 0, no_price 1}`; its items' outcomes are `MarkPrinted`, `NoPrice`, `Fail` with `issues ["Barcode: no barcode."]`; `id` is not empty | MC-L2.1; 09C §6.1; §12.4 |
| L3·C1(b) | `check(["1", "1", "2"])` | P1, P2 | the plan request's items are 1, 2 | MC-L2.1 |
| L3·C1(c) | `check([])` | — | `Err(Storage(InvalidValue(_)))`; no label request | MC-L2.1 |
| L3·C1(d) | `check(["1", "99"])` | Pricing does not hold 99 | one item (P1); `unresolved` = one entry for 99 with the provider's message; the summary counts one item | MC-L2.1; 09C §6.1 ("Not found in Pricing, left out") |
| L3·C1(e) | `check(["98", "99"])` | Pricing holds neither | `Err(Provider(NoProducts))` | 09C §10 |
| L3·C1(f) | `check(["4"])` | P4 printed at T | its item: `outcome Reprint`, `status Printed`, `printed_at Some(T)` | MC-L2.1; 09C §6.1 |
| L3·C1(g) | after (a) | — | no request line contains `/labels/runs`; the fake's label state of P1, P2, P3 is as before; the fake shop's `mutation_ops()` is empty | M-L2; M-L5 |
| L3·C1(h) | `check(["1"])` | no key saved; access `Denied` | `Err(Off(NoKey))`; `Err(Denied)`; no label request | ACC.1; M-L6 |
| L3·C1(i) | `check(["1"])` | the plan answers 500 | `Err(Provider(Unreachable(_)))`; a following `generate(<any id>)` answers `Err(NotFound)` | MC-L2.1 ("nothing kept") |
| L3·C2(a) | `check(<301 product ids>)` | 301 priced, unprinted products | exactly two plan requests, of 300 and 1 items, in order; one check with 301 items whose positions are 0…300 in request order; `summary.items = 301`, `summary.mark_printed = 301` | MC-L2.1; 09C §6.1 ("shown as one") |
| L3·C2(b) | `check(<301 ids>)` | products 1–200 and product 301 use the 70 × 100 template, products 201–300 a 50 × 30 template | `sizes` = `[{70, 100, 201}, {50, 30, 100}]` | MC-L2.1 (sizes merged) |
| L3·C2(c) | `check(<301 ids>)` | Pricing does not hold the 301st | 300 items; `unresolved` = one entry for that id with message `Not found` | MC-L2.1 (a `NoProducts` chunk) |
| L3·C2(d) | `check(["1", "2", "3"])` | P1 on template A (`Label 70×100`, product types `Tables`, `Chairs`), P2 on template B (no product types), P3 without any template (the fake has no default and none assigned) | `groups` = `[{template A, product_types ["Tables", "Chairs"], [P1]}, {template B, [], [P2]}, {template None, [], [P3]}]` | MC-L2.2, .4; 09C §6.1 |
| L3·C2(e) | `check(["1"])` | the templates read answers 500 | `Ok`: one group whose `product_types` is empty | MC-L2.2 |
| L3·C3(a) | `preview(check.id, "1")` after `check(["1", "2"])` | — | exactly one `POST …/labels/preview` with body `{"productId": "1"}`; `FakeLabelFiles` recorded one raster whose bytes are `%PDF-1.4 fake preview 1`; the answer is `<previews>/labels/<check id>-1.png` and that file exists | MC-L2.5; 09C §6.1 (preview) |
| L3·C3(b) | after (a): `preview(check.id, "1")` again | — | the same path; no second preview request and no second raster | MC-L2.5 |
| L3·C3(c) | `preview(check.id, "9")`; `preview("another-id", "1")` | 9 is not in the check | `Err(NotFound)` for both; no label request | MC-L2.5; M-L2 |
| L3·C3(d) | after `check(["1", "3"])`: `preview(check.id, "3")`; `preview(check.id, "1")` | P3 has no template (the fake has no default and none assigned), so the fake answers 422 `no_template` with message `No template applies.`; for P1, set just before its call, `fail_matching_with("/labels/preview", 1, 503, "renderer_unavailable", …)` (projection r0, nit) | `Err(Provider(Rejected {message: "No template applies.", ..}))`; `Err(Provider(RendererUnavailable))` | 09C §10 |
| L3·C3(e) | `preview(check.id, "1")` | `fail_raster_next()` | `Err(PreviewFailed(_))`; a following `generate(check.id)` answers `Ok` | MC-L2.5 ("never blocks Generate") |
| L3·C3(f) | after (a): a new `check(["2"])`; then `preview(<the first id>, "1")` | — | the first check's PNG file is gone; `Err(NotFound)` | MC-L2.3 |
| L3·C3(g) | after (a)–(f) | — | no request line contains `/labels/runs` | M-L2 |
| L3·C4(a) | `DiskLabelFiles::save("labels-2026-11-20.pdf", bytes)` | the labels folder does not exist | the folder exists; `<folder>/labels-2026-11-20.pdf` holds the bytes; no `.part` file is in the folder | MC-L2.13; 09C §6.4 |
| L3·C4(b) | the same call twice more | — | `labels-2026-11-20 (2).pdf` and `labels-2026-11-20 (3).pdf` hold the bytes; the first file is unchanged | MC-L2.13; 09C §6.4 |
| L3·C4(c) | `raster(pdf, "x")` with a one-page PDF the test writes itself (MediaBox 198.43 × 283.46 pt, one line of text); `raster(b"not a pdf", "y")` | — | `<previews>/labels/x.png` exists and starts with the PNG signature `89 50 4E 47 0D 0A 1A 0A`; the second answers an error and `<previews>/labels/y.png` does not exist; after both calls `<previews>/labels/` holds exactly `x.png` (no `.src.pdf` is left; projection r0, D1) | master plan §3 (the probe); MC-L2.5 |
| L3·C4(d) | `save("../../evil.pdf", bytes)`; `save("", bytes)` | — | the files are `<folder>/evil.pdf` and `<folder>/labels.pdf`; the folder's parent and grandparent hold no `evil.pdf` | MC-L2.13 ("nothing outside the folder") |
| L3·C4(e) | `clear_previews()` | `<previews>/labels/` holds two PNGs and `<previews>/` holds another file | the two are gone; the other file is there | MC-L2.3 |
| L3·C4(f) | `save("a.pdf", bytes)` | the labels folder's path is an existing regular file | an error; no `.part` file was left beside it | MC-L2.13; 09C §10 (saving fails) |

### Delegated choices
Where the kept check lives (one mutex slot in the service) · how the test of L3·C4(c) writes its PDF (a short literal byte string is enough). (The temporary PDF's place is no longer delegated: task 3, projection r0 D1.)

**Decided at projection r0** (`handoffs/reviewer/handoff_L3_generate_and_download_projection_r0.md`; the projectionist's recommendation is the rule): (D10) a `check` that fails while an earlier check is kept **empties the slot and clears the previews** (`LabelFiles::clear_previews`), the literal reading of MC-L2.1's "nothing kept": a later `preview` / `generate` of the earlier id answers `NotFound`; (D12) `FakeLabelFiles::clear_previews` delegates to its inner `DiskLabelFiles::clear_previews` (L3·C3(f) needs the first check's PNG gone); (D13) `label_runs.rs` is a sibling module of `labels.rs`, so the `LabelsService` fields it reads (`copy`, `provider`, `events`, …) become `pub(super)`, or small `pub(super)` accessors are added; no behaviour change; (D14) in `check`, the order of the templates read relative to the plan reads, and whether the templates read is made when a plan read fails, are the implementer's (rows assert counts only, L3·C1(a)).

### Notes
- Hazard: `sips` renders a PDF at 72 dpi unless `-Z` is given; `-Z 1200` is what gives a sharp picture (master plan §3). The PNG has an alpha channel; the dialog must show it on white (U4).
- Hazard: no test may run `open` (it would start Preview on the dev Mac). `DiskLabelFiles::open` has no test; it is one `std::process::Command` and is checked by reading.
- Hazard: `AppPaths::for_dir` reads `$HOME` (`paths.rs:17`); `TestApp` overrides `home` with its temporary folder (`test_app.rs:502`). The labels folder must be derived from `paths.home` at the one place `App::build` builds `DiskLabelFiles`, and nowhere else.
- Hazard: the webview may read only `<data>/previews` and `<data>/templates`; a preview written anywhere else shows a broken image.
- L3·C4(c) is the one place the suite runs `sips` (the seam: our command line yields a PNG file). It needs macOS, no network and no app.
- Hazard (projection r0): today's fake `fetch` (`fake_pricing.rs:1147`) records the request and answers 404 without honouring `hold`, `timeout_next`, `drop_answer_next` or the scripted failures. Task 5's last sentence ("act on `fetch` as on `send`") is what L3·C7(a)–(e), L3·C8(a) and L3·C11(p) depend on; do not skip it.

### Unit test commands
`cargo test -p psm-core --lib mechanisms::labels` · `cargo test -p psm-infra --lib label_files` · `cargo test -p psm-testkit --test suite l3_labels_check::` · `cargo check --workspace` · `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Unit 2 — Generating: keys, retries, the result

**Unit id:** `L3·U2` · **Rows:** L3·C5–C9 · **Changes app files:** yes

### Goal
`LabelsService::{generate, partial}`, `merge_runs`, and the fake Pricing's runs.

### Read first
Master plan §5 **MC-L2.6–.10, .14**, ACC.4, MC-L3.2 · U1's `label_runs.rs` · `crates/core/src/services/conversations.rs` (the outbox flush :1099-1210, as the model of "same id on every attempt") · `crates/testkit/src/fake_pricing.rs` (`drop_answer_next` :838, `timeout_next` :828, `fail_matching` :787, `fail_matching_with` :810, `hold` / `release` / `wait_until_held` :853-873, `mutations` as the model of an idempotency map) · the package's `API.md` runs section and `LABELS.md` §3.4.

### Files
New: `crates/testkit/tests/suite/l3_labels_generate.rs`. Edited: `crates/core/src/{mechanisms/labels.rs, services/label_runs.rs}`, `crates/testkit/src/fake_pricing.rs`, `crates/testkit/tests/suite/main.rs`, `docs/guide/{02_architecture.md, 03_rules_and_principles.md}`.

### Core mechanisms (the only functions with unit tests)
`mechanisms::labels::merge_runs`.

### Ordered tasks
1. `merge_runs` (MC-L2.9) with unit tests: totals over two chunks; the order of `failures` (check failures, then per chunk run failures, run unresolved, a `NoProducts` chunk); `not_run` for a chunk without an outcome; `failed_product_ids`. **A failure's `sku` and `title` are the check item's** (always kept with the check), falling back to the run item's or the `Unresolved`'s own (MC-L2.9 as amended; projection r0, D3: Pricing's `RunItemOut.title` is nullable and its `API.md` runs example sends none, so a title read from the run item passes against the fake and shows an empty name against the real Pricing); one unit-test case where the run item has no title and the failure still carries the check item's.
2. `generate` (MC-L2.6–.8, .10): the generation is created once per check with its chunks and keys (`sm-` + `uuid::Uuid::new_v4()`); the `running` flag is set and cleared around the work so that every way out clears it; chunks without an outcome are sent in order; the copy and `LabelsChanged` after each finished chunk (MC-L2.8 as amended: a `Reprinted` item → `Printed` with the **check item's `printed_at`**, else the run's `created_at`; projection r0, D5). `partial` (MC-L2.10).
3. `FakePricing` runs: `POST /labels/runs` through `fetch`, answering **`201`** — a key seen before answers the stored run **without marking again**; otherwise each resolved item is rendered unless `fail_label` / `fail_in_run` names it (then `outcome: "failed"` with that error, `lifecycleResult: null`), priced unprinted products become printed (one `label.printed` event each), printed ones are `reprinted`, unpriced ones `no_price`; unknown ids go to `unresolved`; none resolved → 422 `no_products`; `status` `rendered` / `partial` / `failed`; one artifact per label size among the rendered items (`filename` `labels-<run n>-<w>x<h>.pdf`, `downloadUrl` the documented path, **`page_count` = the rendered items of that size**, projection r0 D8); **every run item carries `sku` and `title`** (schema fields of `RunItemOut`; projection r0, D3); `runs()` → the stored runs with their keys, item ids and `created_at` (D9); `remove_label_product(id)`.
4. Guide: `02_architecture.md` §Labels (check → generate → result; where the keys live), `03_rules_and_principles.md` (M-L2, M-L3, M-L4 with where each bites).
5. Tests for the rows.

### Criteria (5 criteria, 24 rows)
Boundary: `App.labels` on `TestApp::with_pricing()`, connected and `Allowed`; outcome = the returned value, the recorded `POST …/labels/runs` requests and their bodies, `FakePricing::runs()` and its event log, the display copy (through `session_labels(.., false)`), collected events. "The run requests" = the recorded requests whose path ends `/labels/runs`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| L3·C5(a) | `generate(check.id)` after the check of L3·C1(a) | — | exactly one run request; its items are product ids 1, 2 (not 3); its `idempotencyKey` starts with `sm-` and is 39 characters long; the result is `{total 3, rendered 2, marked_printed 1, reprinted 0, no_price 1, failures: [{product_id "3", message "Barcode: no barcode."}], not_run: [], artifacts: one of 70 × 100 with `page_count` 2, failed_product_ids: ["3"]}` | MC-L2.6, .7, .9; §12.4 |
| L3·C5(b) | after (a) | `s1` a linked session for P1 | the fake holds P1 printed and P2 not printed; `session_labels([s1], false)` = `Printed` at the run's time with no status request; one `LabelsChanged {["1", "2"], access: false}` | MC-L2.8; §12.3 |
| L3·C5(c) | `check(["4"])`, `generate` | P4 printed at T0 | the result has `reprinted 1`, `marked_printed 0`, `rendered 1`; the fake logged no `label.printed` for P4; the copy holds P4 `Printed` with `printed_at` T0 (the check item's, projection r0 D5) | MC-L2.8; 09C §6.3 |
| L3·C5(d) | `check(["3"])` (the failing product), `generate` | — | `Err(NothingToGenerate)`; no run request | MC-L2.6; 09C §6.1 ("disabled when every product fails") |
| L3·C5(e) | `generate("no-such-check")`; `check(["1"])` (id A), `check(["2"])` (id B), `generate(A)` | — | `Err(NotFound)` for both; no run request | MC-L2.6; M-L2 ("products that check just showed") |
| L3·C6(a) | `check(<301 ids>)`, `generate` | 301 priced, unprinted products | exactly two run requests, of 300 and 1 items in check order; their keys differ; the fake stores two runs | MC-L2.6; M-L3; 09C §6.2 |
| L3·C6(b) | same | — | the result has `total 301`, `rendered 301`, `marked_printed 301`, and the two runs' artifacts in chunk order | MC-L2.9 |
| L3·C6(c) | `check(["1", "2"])`, `generate` | P1 on a 70 × 100 template, P2 on a 50 × 30 template | one run request; the result has two artifacts, 70 × 100 and 50 × 30, each with `page_count` 1 | MC-L2.9; §12.5 |
| L3·C7(a) | `check(["1"])`, `generate` | `drop_answer_next(1)` on the run (the fake makes the run; its answer is lost) | `Ok` with `marked_printed 1`; exactly two run requests with **identical bodies** (the recorded `FakeRequest.body` values are equal as JSON: the same key, the same items; projection r0, nit); the fake stores one run and logged exactly one `label.printed` for P1 | MC-L2.7; M-L3; §12.7 |
| L3·C7(b) | same | the run answers 500 twice, then succeeds | `Ok`; exactly three run requests with the same key | MC-L2.7 (`RUN_RETRIES`) |
| L3·C7(c) | same | the run answers 500 three times; then `generate(check.id)` again with the fake healthy | first `Err(Provider(Unreachable(_)))` after exactly three run requests; then `Ok` after one more; all four carry the same key | MC-L2.7; M-L3 (Try again) |
| L3·C7(d) | same | the run answers 503 `renderer_unavailable` three times | `Err(Provider(RendererUnavailable))`; three run requests with the same key; the fake holds P1 not printed | MC-L2.7; 09C §10 ("nothing was marked") |
| L3·C7(e) | `check(<301 ids>)`, `generate`; then `partial(check.id)`; then `generate(check.id)` again | the second chunk's run answers 500 three times, then the fake is healthy | first `Err(Provider(Unreachable(_)))`; `partial` = `{total 301, rendered 300, not_run: [the 301st id], failed_product_ids: [that id]}` with chunk 1's artifacts; the second `generate` makes exactly one more run request, carrying chunk 2's key and its one item (chunk 1's key is not sent again), and answers `rendered 301` | MC-L2.7, .10; M-L3 |
| L3·C7(f) | `check(["1"])`, `generate` | the run answers 403 `insufficient_scope` | `Err(Provider(NotAllowed))` after exactly one run request; `access()` = `Denied` | MC-L2.7; ACC.4 |
| L3·C7(g) | `check(["1"])`, then `remove_label_product("1")`, `generate` | the run answers 422 `no_products` | `Ok` with `rendered 0` and `failures: [{product_id "1", message "None of these products were found in Pricing."}]`; exactly one run request | MC-L2.7, .9 |
| L3·C7(h) | `check(["1"])`, `generate` | the run answers 422 `validation_error` with message `M`; on another check, 201 with a body that is not a run | `Err(Provider(Rejected {message: M, ..}))`; `Err(Provider(BadReply(_)))`; one run request each | MC-L2.7 (not retried) |
| L3·C8(a) | `check(["1"])`; `fake.hold()`; thread A `generate(check.id)`; once held, thread B `generate(check.id)` and `check(["2"])`; `release` | — | B's two calls answer `Err(Busy)` at once; A answers `Ok`; exactly one run request was recorded | MC-L2.6; M-L3 ("a double press makes no second run") |
| L3·C8(b) | after a finished `generate`: `generate(check.id)` again | — | the same result; no further run request | MC-L2.10 |
| L3·C8(c) | after a finished `generate`: `kill_and_restart`, then `generate(<that id>)` | — | `Err(NotFound)`; the database holds no table or column about a check, a run or a key: the L1·C13(f) assertion still holds, and L3 adds no migration and no column (the L1·C13(f) instrument only searches names containing `label`, so a stored `check_key` column would pass it; the reviewer checks by reading that `SqliteStore::migrations()` is unchanged by L3; projection r0, nit) | MC-L3.4; M-L1 |
| L3·C9(a) | `check(["1", "2"])`, `generate` | `fail_in_run("2", "missing_code_value", "Barcode: no barcode.")` (P2 passes the check and fails in the run) | `{total 2, rendered 1, failures: [{product_id "2", sku: P2's, title: P2's, message "Barcode: no barcode."}], failed_product_ids: ["2"]}` | MC-L2.9; M-L4 |
| L3·C9(b) | `check(["1", "2"])`, then `remove_label_product("2")`, `generate` | the run answers P2 in `unresolved` | `failures` holds P2 with the provider's message; `rendered 1`, `total 2` | MC-L2.9; M-L4 |
| L3·C9(c) | `check(["1", "2", "3"])`, `generate` | P3 fails the check; P2 fails in the run | `failures` is `[P3, P2]` in that order; `rendered 1`, `total 3` | MC-L2.9 (order) |
| L3·C9(d) | `check(["1", "2"])`, `generate` | both priced, nothing fails | `failures` and `not_run` are empty; `rendered = total = 2` | M-L4 |
| L3·C9(e) | the result of L3·C7(e)'s `partial` | — | `rendered (300) < total (301)` and the product not run is in `not_run` and in `failed_product_ids` | M-L4 ("never reported as fully generated") |

### Delegated choices
How the generation is stored beside the check · how `Busy` is detected (a flag under the check's mutex) · whether a retry logs each attempt (`labels.run … attempt=`).

**Decided at projection r0** (`handoffs/reviewer/handoff_L3_generate_and_download_projection_r0.md`; the projectionist's recommendation is the rule): (D9) the implementer adds to `FakePricing` the read accessors the rows assume (each has a row caller, charter rule 4): the label state of a product (L3·C1(g) "as before", L3·C5(b), L3·C7(d); added in U1, whose L3·C1(g) first calls it), the `label.printed` event log or its count per product (L3·C5(c), L3·C7(a)), and `runs()` returning each stored run's key, item ids and `created_at` (L3·C5(b) "at the run's time"); names and shapes are the implementer's.

### Notes
- Hazard: the retry must send **the stored key and the stored ids** (L3·C7(a) compares the request bodies byte for byte); never rebuild the item list from a new read.
- Hazard: `running` must be cleared on every way out of `generate`, including an error from the store or a panic-free early return; otherwise the dialog stays "busy" for ever.
- Hazard: a finished chunk is never sent again, not even by Try again (L3·C7(e)); its outcome is kept in the generation.
- Hazard: `generate` holds no lock while the request is in flight that `session_labels`, `list` or the pull's `changed` need (the copy's own mutex is taken only to apply an answer).
- Hazard: the run also produces `label.printed` events that the pull delivers within 10 s; applying them again is harmless (MC-L3.2: same state), and the answer of an older status request can never undo them (the sequence rule).
- No pause between the automatic retries (master plan §6); the request's own timeout spaces them.

### Unit test commands
`cargo test -p psm-core --lib mechanisms::labels` · `cargo test -p psm-testkit --test suite l3_labels_generate::` · `cargo test -p psm-testkit --test suite l3_labels_check::` · `cargo clippy -p psm-core -p psm-testkit --all-targets -- -D warnings`

## Unit 3 — Download and the commands

**Unit id:** `L3·U3` · **Rows:** L3·C10–C11 · **Changes app files:** yes

### Goal
`LabelsService::download`, the fake Pricing's artifact download, and the five commands with their error codes.

### Read first
Master plan §5 **MC-L2.11–.13**, §4 Commands (wire forms, error codes), Words · `apps/desktop/src-tauri/src/commands/labels.rs` (L1, L2: the `LabelsFailure` → `CommandError` mapping) · `apps/desktop/src-tauri/src/lib.rs` (`l1_tests`, `l2_tests`) · U1's `FakeLabelFiles`.

### Files
New: `crates/testkit/tests/suite/l3_labels_download.rs`. Edited: `crates/core/src/services/label_runs.rs`, `crates/testkit/src/fake_pricing.rs`, `crates/testkit/tests/suite/main.rs`, `apps/desktop/src-tauri/src/{lib.rs, commands/labels.rs}`, `apps/desktop/src/services/{bindings.ts (generated), labels.ts}`, `docs/guide/{02_architecture.md, 04_change_impact.md}`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. `download` (MC-L2.11); a `LabelFiles::save` error becomes `SaveFailed(reason)` with the reason rule of U1 task 3 (`Error::Io(e)` → `e.to_string()`, any other error → its `Display`; so `fail_save_next("disk full")` → `SaveFailed("disk full")` → `Couldn't save the PDF: disk full.`, L3·C10(d), L3·C11(i); projection r0, D2). `FakePricing`: `GET /labels/runs/<run>/artifacts/<artifact>` through `fetch` (`%PDF-1.4 fake labels <run> <artifact>`; 404 for an unknown one; `expire_artifacts()` → 410 `artifact_expired`).
2. Commands `labels_check`, `labels_preview`, `labels_generate`, `labels_partial`, `labels_download` (thin; DTOs in master plan §4 wire forms: outcomes `mark_printed | reprint | no_price | fail`, paths as text), the remaining `LabelsFailure` codes; registered; bindings regenerated; wrappers `labelsCheck`, `labelsPreview`, `labelsGenerate`, `labelsPartial`, `labelsDownload`.
3. Guide: `02_architecture.md` §Labels (download, the Labels folder, the preview cache), `04_change_impact.md` (rows: a label command or DTO → bindings, `services/labels.ts`, the dialogs; the Labels folder rule → `label_file_name`, `DiskLabelFiles::save`).
4. Tests for the rows.

### Criteria (2 criteria, 24 rows)
Boundary for C10: `App.labels` as in U2, with `FakeLabelFiles` (outcome = the returned value, the recorded requests, the files under the test's home, the fake's `opened()`). For C11: the command bodies in the desktop crate's `l3_tests`. "The labels folder" = `<test home>/Documents/Session Manager Labels`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| L3·C10(a) | `download(check.id, artifact.id)` after L3·C5(a) | — | exactly one `GET /stores/<id>/labels/runs/<run>/artifacts/<artifact>`; the file `<labels folder>/<artifact.filename>` holds the fake's bytes; the fake recorded one open of that path; the answer is `{path: that file, opened: true}` | MC-L2.11; 09C §6.4; §12.3 |
| L3·C10(b) | the same call again | — | a second file `<stem> (2).pdf` with the same bytes; the first is unchanged | MC-L2.13; 09C §6.4 |
| L3·C10(c) | `download(…)` | `expire_artifacts()` | `Err(Provider(Expired))`; the labels folder holds no new file | 09C §10 |
| L3·C10(d) | `download(…)` twice | `fail_save_next("disk full")` | first `Err(SaveFailed("disk full"))` and no file; the second call saves the file | 09C §10 ("with Try again") |
| L3·C10(e) | `download(…)` | `fail_open_next()` | `Ok({path, opened: false})`; the file is saved | MC-L2.11 |
| L3·C10(f) | `download(check.id, "no-such-artifact")`; `download("no-such-check", artifact.id)`; `download(check.id, …)` before any `generate` | — | `Err(NotFound)` for each; no label request | MC-L2.11 |
| L3·C10(g) | after a whole flow (check, preview, generate, download) | — | the fake shop saw no call at all (`mutation_ops()` empty and no read); deleting the session of P1 afterwards makes no label request | M-L5 |
| L3·C11(a) | `labels_check(["1", "2", "3"])` | the products of L3·C1(a) | `{id, groups: [{template: {id, name: "Label 70×100", width_mm: 70.0, height_mm: 100.0}, product_types: […], items: [{product_id: "1", sku, title, price_minor, status: "not_printed", printed_at: null, outcome: "mark_printed", issues: [], notes: []}, {…, outcome: "no_price"}, {…, outcome: "fail", issues: ["Barcode: no barcode."]}]}], unresolved: [], summary: {items: 3, ready: 2, failing: 1, mark_printed: 1, reprint: 0, no_price: 1}, sizes: [{width_mm: 70.0, height_mm: 100.0, count: 2}]}` (the millimetres are `f64` and serialise as `70.0`; `serde_json` holds `70.0 ≠ 70`; projection r0, nit) | MC-L2.1, .4; Commands |
| L3·C11(b) | `labels_preview(id, "1")` | — | `{path}` ending `/previews/labels/<id>-1.png` | MC-L2.5 |
| L3·C11(c) | `labels_generate(id)`; then `labels_partial(id)` | — | both answer `{total: 3, rendered: 2, marked_printed: 1, reprinted: 0, no_price: 1, failures: [{product_id: "3", sku, title, message}], not_run: [], artifacts: [{id, filename, width_mm: 70.0, height_mm: 100.0, page_count: 2}], failed_product_ids: ["3"]}` (`70.0`: see L3·C11(a)) | MC-L2.9, .10; Commands |
| L3·C11(d) | `labels_download(id, artifact)` | — | `{path, opened: true}` with `path` in the labels folder | MC-L2.11 |
| L3·C11(e) | `labels_check(["98", "99"])` | Pricing holds neither | `CommandError {code: "labels_no_products", message: "None of these products were found in Pricing."}` | 09C §10 |
| L3·C11(f) | `labels_generate(id)` | the run answers 503 `renderer_unavailable` three times | `{code: "labels_renderer_unavailable", message: "Labels can't be generated right now. Try again in a moment."}` | 09C §10 |
| L3·C11(g) | `labels_generate(id)` | the run answers 500 three times | `{code: "labels_unreachable", message: "Pricing can't be reached right now."}` | 09C §10 |
| L3·C11(h) | `labels_download(id, artifact)` | `expire_artifacts()` | `{code: "labels_expired", message: "This file has expired. Generate the labels again."}` | 09C §10 |
| L3·C11(i) | `labels_download(id, artifact)` | `fail_save_next("disk full")` | `{code: "labels_save_failed", message: "Couldn't save the PDF: disk full."}` | 09C §10 |
| L3·C11(j) | `labels_preview(id, "3")` | the fake answers 422 `no_template`, message `No template applies.`; on another check `fail_raster_next()` | `{code: "labels_rejected", message: "No template applies."}`; `{code: "labels_preview_failed", message: "The label preview can't be shown."}` | 09C §10 ("the message as given") |
| L3·C11(k) | `labels_generate(id)` | every item of the check fails | `{code: "labels_nothing", message: "None of these products can be generated."}` | MC-L2.6 |
| L3·C11(l) | `labels_generate("no-such-check")` | — | `{code: "not_found"}` | MC-L2.6 |
| L3·C11(m) | `labels_generate(id)` | the run answers 403 `insufficient_scope` | `{code: "labels_denied", message: "Session Manager isn't allowed to print labels. Ask a Pricing admin."}` | 09C §10; ACC.4 |
| L3·C11(n) | `labels_check(["1"])` | the store was removed from the fake Pricing (the plan answers 404) | `{code: "labels_store_gone", message: "This key is no longer tied to shop-a.myshopify.com"}` | 09C §10 |
| L3·C11(o) | `labels_check(["1"])` | the plan answers 200 with a body that is not a plan | `{code: "labels_bad_reply", message: "Pricing answered in a way this version does not understand"}` | Commands |
| L3·C11(p) | two threads call the body of `labels_generate(id)` while the fake holds the run | — | the second answers `{code: "labels_busy", message: "Labels are being generated."}` | MC-L2.6; M-L3 |
| L3·C11(q) | `labels_check(["1"])` | no key saved | `{code: "labels_off", data: {reason: "no_key"}}` | ACC.1; M-L6 |

### Delegated choices
How the result DTO names the artifact's label count (`page_count`) · whether `labels_partial` answers `not_found` as an error or as `null` (an error, as `labels_generate`).

### Notes
- Hazard: `download_url` is whatever the run answer carried; the adapter (not the service) refuses anything that is not the store's own artifact path (L1·C10(b)). Do not "fix" an address in the service.
- Hazard: `labels_download` returns a path outside the data folder (the Documents folder); the screen shows it as text and never loads it through `previewUrl`.
- Hazard: every label command runs on a blocking worker (`blocking` in `commands/conversations.rs:106` is the pattern); `labels_generate` can take minutes for a large batch with retries.

### Unit test commands
`cargo test -p psm-testkit --test suite l3_labels_download::` · `cargo test -p psm-desktop` · `cd apps/desktop && npm run typecheck` · `cargo clippy -p psm-core -p psm-testkit -p psm-desktop --all-targets -- -D warnings`

## Unit 4 — The dialogs, the ⋯ entry and the batch bar wired

**Unit id:** `L3·U4` · **Rows:** L3·C12–C17 · **Changes app files:** yes

### Goal
`LabelDialog` (check view, preview, busy state, error states, done view, downloads), the enabled ⋯ entry on session rows, Labels rows and the session page, the working `Generate labels for N`, and what happens to the selection afterwards.

### Read first
Master plan §4 Words, **§5 MC-L2** (what each call answers), MC-L1.9 (selection, refresh) · book `09C` §6 (every sentence, verbatim), §10 · the package's `LABELS.md` §3.3–§3.5 (the flow and wording these dialogs copy; the styling is Session Manager's) · `apps/desktop/src/features/labels/` (L1, L2) · `apps/desktop/src/features/sessions/DeleteSession.tsx` (a menu item that owns a dialog; `Dialog` usage :105-143) · `apps/desktop/src/components/ui/{dialog.tsx, action-button.tsx}` · `apps/desktop/src/features/workspace/remote/BringBackDialog.tsx` (a dialog with a busy state that cannot be closed and a Try again) · `apps/desktop/src/services/sessions.ts` (`previewUrl` :106) · `docs/intention/evidence/design_handoff.md`.

### Files
New: `apps/desktop/src/features/labels/{LabelDialog.tsx, LabelCheckView.tsx, LabelDoneView.tsx, useLabelDialog.ts}`, `apps/desktop/src/features/labels/__tests__/{l3_label_check_dialog.test.tsx, l3_label_done_dialog.test.tsx, l3_label_wiring.test.tsx}`. Edited: `apps/desktop/src/features/labels/{labelWords.ts, LabelMenuItem.tsx, LabelRow.tsx, LabelsBatchBar.tsx, LabelsBox.tsx}`, `apps/desktop/src/features/sessions/{DeleteSession.tsx, SessionRow.tsx}`, `apps/desktop/src/features/shell/Header.tsx`, `apps/desktop/src/styles/globals.css`, and two earlier tests whose expectation this unit replaces: `l1_label_menu.test.tsx` row L1·C19(c) (the item is now shown enabled, see L3·C17(a); replaced for **both** its states, access `allowed` and access `unknown`, projection r0 D6) and `l2_labels_selection.test.tsx` row L2·C13(a) (the button is now enabled, see L3·C17(d)); no other existing assertion changes.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. `labelWords.ts`: the sentences of 09C §6 as functions of the check's summary and the result (the exact texts are the outcome cells below), the outcome words of master plan §6, `sizeText(w, h)` = `<w> × <h> mm` (whole numbers without decimals). **The date in `Printed <date>.`** (L3·C12(e)) is the marker's day rule of MC-L3.5 (the existing `day()` helper: local calendar day, `d MMM` with the fixed English month abbreviation, the year added when it is not `now`'s year); the test's fixture is 4 Nov **of the current year**, or the test fixes `now` (a fixture hard-coded to 2026-11-04 turns red on 1 Jan 2027; projection r0, D7).
2. `LabelDialog(productIds, onClose(result | null))` with `useLabelDialog`: the states `checking` → `check` → `generating` → `done`, plus `error` on each call; it calls `labelsCheck` when opened, `labelsPreview` for the product shown, `labelsGenerate` on the button (a second press while one is in flight does nothing), `labelsPartial` when the photographer leaves after a failed Generate, `labelsDownload` per file. While `generating` the dialog cannot be closed (the close button, Escape and a click outside do nothing).
3. `LabelCheckView`: one section per group — heading `<template name> · <product types joined by ", "> · <size>` (the product types part is left out when there are none; `No template` for the last group) — with one line per product (SKU, name, `formatPrice`, the outcome words, and under a `Can't generate` line its issue messages); the unresolved entries as `<SKU or product id> · Not found in Pricing`; the footer and notes; the preview pane (a placeholder with the template's proportions while loading, then the picture on a white tile, or the error's message in its place; `‹` `›` for more than one product); `Generate PDF` / `Reprint PDF` and `Cancel`.
4. `LabelDoneView`: the title, the result lines (a line whose number is zero is not shown), the failures (`<SKU> · <name>` and the message), the not-generated count, the file rows and buttons, `Print at 100 % (not Fit to page).`, and for a batch with failures `Failed products stay selected so you can fix and retry.`
5. Wiring: `LabelMenuItem` now also renders when it is enabled — access `allowed` **or `unknown`** (ACC.1: calls are allowed while the access is not `denied`; projection r0, D6) with a product id — `Reprint label` when the marker is printed, else `Generate label` — and opens the dialog for that one product; `LabelRow` without a session gets a ⋯ holding only this item; `LabelsBatchBar`'s `Generate labels for <N>` opens the dialog with the selected product ids in selection order; when the dialog closes after a run, `LabelsBox` sets the selection to the result's `failed_product_ids` that were selected and refreshes its page (MC-L1.9).
6. Tests for the rows.

### Criteria (6 criteria, 32 rows)
Boundary: `LabelDialog` (C12–C16) and the menu / the Labels box (C17) rendered in UI tests with `@/services/labels` and `@/services/events` mocked (outcome = what is rendered and the exact lists of `labelsCheck`, `labelsPreview`, `labelsGenerate`, `labelsPartial`, `labelsDownload`, `labelsList` calls). "A check answer" has the shape of L3·C11(a). Texts are compared exactly; `<thin>` in a price is U+2009.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| L3·C12(a) | `LabelDialog(["1"])` | `labelsCheck` stays pending | the dialog is open with a busy state; exactly one `labelsCheck({product_ids: ["1"]})` | 09C §6.1 ("opens at once with a busy state") |
| L3·C12(b) | `LabelDialog(["1", "2", "3", "4"])` | groups: template `Label 70×100` (product types `Tables`, `Chairs`, 70 × 100) with a `mark_printed` and a `reprint` item; template `Small tag` (no product types, 50 × 30) with a `no_price` item; a group without a template with a `fail` item (issue `Barcode: no barcode.`) | the headings `Label 70×100 · Tables, Chairs · 70 × 100 mm`, `Small tag · 50 × 30 mm`, `No template`; the four lines show their SKU, name and price and the words `Will be marked Printed`, `Reprint · stays Printed`, `No price · stays Not printed`, `Can't generate` with `Barcode: no barcode.` under it | MC-L2.4; 09C §6.1 |
| L3·C12(c) | same | `unresolved: [{product_id: "99", sku: null, message: "…"}, {product_id: null, sku: "X-1", message: "…"}]` | the lines `99 · Not found in Pricing` and `X-1 · Not found in Pricing`, outside every group | 09C §6.1 |
| L3·C12(d) | a batch | `summary {mark_printed: 5, failing: 0, reprint: 0}`; then `{mark_printed: 1, failing: 1, reprint: 2}` | `5 products will be marked Printed` and neither other sentence; then `1 products will be marked Printed`, `1 can't be generated with the current template. They're marked below.` and `2 already printed. They're reprinted and stay Printed.` | 09C §6.1 (Pricing's wording); §12.4 |
| L3·C12(e) | one product | outcome `mark_printed`; outcome `reprint` printed on 4 Nov of the current year (or `now` fixed; the date by MC-L3.5's day rule, projection r0 D7); outcome `no_price` | `The product will be marked Printed.` with the button `Generate PDF`; `Label status stays Printed.` and `Printed 4 Nov. Reprinting keeps it marked Printed.` with the button `Reprint PDF`; `Label status stays Not printed.` and `This product has no selling price. You can generate its label, but it stays marked Not printed.` with `Generate PDF` | 09C §6.1 |
| L3·C12(f) | a check whose every item is `fail` | then `Cancel` is pressed | `Generate PDF` is disabled; the dialog closes; `labelsGenerate` was never called | 09C §6.1; M-L2 |
| L3·C12(g) | `labelsCheck` rejects | `labels_no_products` (message `None of these products were found in Pricing.`); `labels_denied` (its message) | the message in the dialog and only `Close` | 09C §10 |
| L3·C13(a) | after a check answer for products 1 and 2 on a 70 × 100 template | `labelsPreview` pending, then `{path: "/d/previews/labels/c-1.png"}` | exactly one `labelsPreview({check_id, product_id: "1"})`; while pending a placeholder whose aspect ratio is 70 / 100; then an `<img alt="Label preview">` of `previewUrl("/d/previews/labels/c-1.png")` inside an element with the class `label-preview` (white background in both themes) | MC-L2.5; 09C §6.1 |
| L3·C13(b) | a batch of three | press `›` twice | `‹` is disabled on the first product and `›` on the last; each step asks `labelsPreview` for the product then shown | 09C §6.1 ("‹ › to step through a batch") |
| L3·C13(c) | `labelsPreview` rejects | `labels_rejected` with message `No template applies.`; `labels_preview_failed` with message `The label preview can't be shown.` | that message where the picture would be; `Generate PDF` stays enabled | 09C §10; MC-L2.5 |
| L3·C13(d) | one product | — | no `‹` and no `›` | 09C §6.1 |
| L3·C14(a) | press `Generate PDF` | `labelsGenerate` stays pending; the button is pressed again; Escape; a click outside | exactly one `labelsGenerate({check_id})`; a busy state; `Generate PDF` and `Cancel` are disabled; the dialog is still open | 09C §6.2; M-L3 |
| L3·C14(b) | `labelsGenerate` rejects with `labels_unreachable` (message `Pricing can't be reached right now.`); then `Try again` | — | the message with `Try again` and `Cancel`; the press makes a second `labelsGenerate` with the **same** `check_id` | 09C §6.2, §10; M-L3 |
| L3·C14(c) | `labelsGenerate` rejects with `labels_renderer_unavailable` | — | `Labels can't be generated right now. Try again in a moment.` with `Try again` | 09C §10 |
| L3·C14(d) | after (b): `Cancel` | `labelsPartial` answers a result with one artifact and `not_run` of 47; on another render it rejects with `not_found` | one `labelsPartial({check_id})`; the done view of that result; with `not_found` the dialog closes | MC-L2.10 |
| L3·C14(e) | `labelsGenerate` rejects with `labels_denied` | — | `Session Manager isn't allowed to print labels. Ask a Pricing admin.` and only `Close` (no `Try again`) | 09C §10 |
| L3·C15(a) | the done view of a batch | `{total: 3, rendered: 3, marked_printed: 2, no_price: 1, reprinted: 0, failures: []}` | the title `3 labels generated`; the lines `✓ 2 marked Printed` and `○ 1 stay Not printed · No selling price`; no line starting `↻` or `✕` | 09C §6.3 |
| L3·C15(b) | same | `{total: 3, rendered: 2, marked_printed: 1, no_price: 1, failures: [{sku: "TEST-3", title: "Test lamp", message: "Barcode: no barcode."}]}` | the title `2 of 3 labels generated`; `✕ 1 failed`; `TEST-3 · Test lamp` with `Barcode: no barcode.`; `Failed products stay selected so you can fix and retry.` | 09C §6.3, §6.5; M-L4; §12.4 |
| L3·C15(c) | one product | `{total: 1, rendered: 1, marked_printed: 1}`; `{total: 1, rendered: 0, failures: [one]}` | `Label generated`; `Label not generated` with the failure's message | 09C §6.3 |
| L3·C15(d) | a batch | `{total: 2, rendered: 2, reprinted: 2}` | `↻ 2 reprinted · stay Printed` | 09C §6.3 |
| L3·C15(e) | a batch | `{total: 347, rendered: 300, marked_printed: 300, not_run: <47 ids>}` | the title `300 of 347 labels generated` and the line `✕ 47 not generated` | MC-L2.9, .10; M-L4 |
| L3·C16(a) | a result with one artifact | press `Download PDF`; `labelsDownload` answers `{path, opened: true}` | a single `Download PDF`; the text `Print at 100 % (not Fit to page).`; one `labelsDownload({check_id, artifact_id})`; the dialog closes | 09C §6.4; §12.3 |
| L3·C16(b) | a result with two artifacts (70 × 100 with `page_count` 2, 50 × 30 with 1) | press the first row's button; then `Done` | two rows: the filename, `2 labels` / `1 label`, `70 × 100 mm` / `50 × 30 mm`, each with its own `Download PDF`; plus `Done`; the download leaves the dialog open; `Done` closes it | 09C §6.4; §12.5 |
| L3·C16(c) | a result without an artifact | — | no `Download PDF`; a `Close` button | 09C §6.4 |
| L3·C16(d) | `labelsDownload` rejects | `labels_expired`; `labels_save_failed` with message `Couldn't save the PDF: disk full.`, then `Try again` | `This file has expired. Generate the labels again.`; the save message with `Try again`, whose press makes a second `labelsDownload` | 09C §10 |
| L3·C16(e) | `labelsDownload` answers `{path: "/Users/x/Documents/Session Manager Labels/a.pdf", opened: false}` | — | the dialog says the file was saved and shows that path (Preview did not open); it stays open until closed | MC-L2.11 |
| L3·C17(a) | `SessionActionsMenu` of a row | label `{product_id: "1"}`, access `allowed`, marker not printed; access `unknown`, marker not printed; access `allowed`, marker printed | an enabled item `Generate label` (for `allowed` and for `unknown`, projection r0 D6); `Reprint label`; a press opens the dialog and calls `labelsCheck({product_ids: ["1"]})` | 09C §6 "⋯ menu entry"; §12.3 |
| L3·C17(b) | the session page's ⋯ (`Header`) | the same label | the same enabled item opening the same dialog | 09C §1.2 |
| L3·C17(c) | a Labels row without a session | — | a ⋯ whose only item is `Generate label` / `Reprint label`; a press opens the dialog with that row's product id | 09C §6 ("⋯ on one row, any box") |
| L3·C17(d) | the Labels box with rows "3" then "1" ticked | press `Generate labels for 2` | the button is enabled; the dialog opens with `labelsCheck({product_ids: ["3", "1"]})` | 09C §5 Selection, §6 |
| L3·C17(e) | after (d): the run's result has `failed_product_ids: ["3"]`; the dialog is closed | — | the selection is exactly `{"3"}` (`1 selected`); exactly one `labelsList` refresh for the page shown | 09C §6.5; MC-L1.9 |
| L3·C17(f) | `SessionActionsMenu` | label `{product_id: null}`; access `{off, no_key}` | still disabled with `Publish first so Pricing can see it`; still disabled with `Connect Pricing in Settings → Pricing` | 09C §6; §12.12 |

### Delegated choices
The dialog's layout (the check list left, the preview right is the package's own arrangement; design handoff tokens) · whether each `LabelMenuItem` owns its dialog or one host in `App.tsx` does · the busy state's look · the words of the saved-but-not-opened line.

**Decided at projection r0** (`handoffs/reviewer/handoff_L3_generate_and_download_projection_r0.md`; the projectionist's recommendation is the rule): (D11) "one product" against "batch" wording in both dialogs (L3·C12(e), L3·C15(c)) is decided by **the number of product ids the dialog was opened with** (`LABELS.md` §3.3 frames it by entry point), not by the check's item count: a batch of 2 with one unresolved product keeps the batch wording; (D15) `‹` `›` (L3·C13(b)) step through the products in **group display order** (what the photographer sees), not plan order; (D16) **Close** after a `labels_denied` Generate (L3·C14(e)) behaves as **Cancel**: it calls `labelsPartial({check_id})` and shows the done view of its answer, and closes on `not_found` (so a chunk finished before the refusal keeps its done view and the selection rule); (D17) `labelsCheck` rejecting with `labels_off` (whose message is empty by design) shows 09B's connection words as the ⋯ hint does: `no_key` → `Connect Pricing in Settings → Pricing` (`CONNECT_PRICING`), any other reason → `connectionLine(settings.connection)` from `conversationsSettingsGet()` (`CONNECT_PRICING` while it loads, on failure or when it answers null), with `Open Settings → Pricing` and `Close`.

### Notes
- Hazard: the preview PNG has an alpha channel; without a white tile it shows the dark theme's background behind the label (L3·C13(a)).
- Hazard: `1 products will be marked Printed` is Pricing's own wording (09C §6.1, §12.4); do not "correct" the plural.
- Hazard: the dialog must not be closable while `generating` (09C §6.2), but must be leavable after a failed Generate (`Cancel` → `labelsPartial`).
- Hazard: a Radix `Dialog` opened from a `DropdownMenu` item closes with the menu unless the item's default is prevented or the dialog is rendered outside the menu content (`DeleteSession.tsx:75-81` renders it beside the menu).
- Hazard: the downloaded file's path is plain text on screen; never pass it to `previewUrl` (it is outside the data folder).
- Two earlier tests change by design (named under Files); the reviewer checks no other one did.

### Unit test commands
`cd apps/desktop && npx vitest run src/features/labels src/features/sessions src/features/shell` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-core --lib mechanisms::labels` · `cargo test -p psm-infra --lib label_files` · `cargo test -p psm-testkit --test suite l3_labels_check::` · `cargo test -p psm-testkit --test suite l3_labels_generate::` · `cargo test -p psm-testkit --test suite l3_labels_download::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/labels` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: `provider.run` is called at exactly one place, inside `generate`, with the stored key and the stored ids; nothing else can start a run (no command takes product ids for a run); a key is made once per chunk and never logged; `running` is cleared on every way out; the labels folder comes from `paths.home` at one place; no test calls `DiskLabelFiles::open` or runs `open`; the service never changes a `download_url`; no label state, check, run or key is written to the database; core names no platform; the two replaced test expectations are the only changed ones; L3 adds no migration (`SqliteStore::migrations()` unchanged, L3·C8(c)); `DiskLabelFiles::raster` keeps its temporary PDF in `<previews>/labels/` and removes it on every way out (D1); `SaveFailed` / `PreviewFailed` carry the inner `io::Error` text for `Error::Io` (D2); a failure's `sku` / `title` come from the check item first (D3). MC-L2.12 (the download address and the `%PDF-` check) has no L3 row by design: L1's adapter rows (`download_path_ok`, L1·C8(b), L1·C10(b)) serve it and U3's hazard keeps the service from rewriting an address; do not raise it as unserved (projection r0). Notes go to `docs/implementation/backlog_labels/L3_generate_and_download.md`.

## Manual try-out (the narrative is built from this; done by the owner on the installed app, never by a session)
Needs a Pricing key with `labels:read` and `labels:write`. **Generating marks products Printed in Pricing:** do the first try on products you mean to label.
1. Sessions → Published → ⋯ of a product with a price → **Generate label**. The check shows the template and its size, "The product will be marked Printed." and a picture of the label. Press **Cancel**: nothing changed.
2. Open it again and press **Generate PDF** → "Label generated" → **Download PDF**: the file appears in Documents › Session Manager Labels and opens in Preview (macOS asks once to allow the Documents folder). Print at 100 %.
3. Back in the list the row reads "Label printed · today"; the Labels box's number dropped by one. Its ⋯ now says **Reprint label**.
4. Labels box → tick three products, one of them without a price → **Generate labels for 3**: the check lists them by template and says how many will be marked Printed. Generate: the result says what was marked and what stays Not printed.
5. A product that cannot be generated (for example no barcode on a barcode template) is shown as "Can't generate" with the reason; after Generate the title reads "N of M labels generated" and that product is still ticked.
6. A batch over two label sizes ends with two Download PDF rows and **Done**.
7. Turn Wi-Fi off, press Generate PDF, wait for "Pricing can't be reached right now.", turn Wi-Fi on, press **Try again**: the labels are generated once.
8. A session that was never published: its ⋯ shows Generate label greyed out with "Publish first so Pricing can see it".

## Review log

- **2026-11-23, orchestrator: projection r0 amendments applied** (`handoffs/reviewer/handoff_L3_generate_and_download_projection_r0.md`, AMENDMENTS_REQUIRED, 0 owner cards; master plan v1.2). Where each ledger row landed:
  - **D1** (C4(c)'s temporary file undecidable): U1 task 3 (`raster`'s temporary PDF is `<previews>/labels/<name>.src.pdf`, always removed); the delegated choice "the temporary PDF's place" removed from U1 Delegated choices; row L3·C4(c) outcome "after both calls `<previews>/labels/` holds exactly `x.png`"; master plan §4 Files says the same.
  - **D2** (`SaveFailed` reason text): U1 task 3 (`FakeLabelFiles::fail_save_next(reason)` answers `Error::Io(io::Error::other(reason))`; reason = the inner `io::Error`'s text for `Error::Io`, the `Display` otherwise; the same rule for `PreviewFailed`); U3 task 1 applies it to `download`; Review mode check.
  - **D3** (a failure's `sku` / `title`, rule 17): master plan MC-L2.9 (the check item's, falling back to the run item's or `Unresolved`'s own); U2 task 1 (`merge_runs`, with a unit-test case for a run item without a title); U2 task 3 (the fake's run items carry `sku` and `title`); Review mode check.
  - **D4** (`merge_plans` positions): master plan MC-L2.1 (`position` = the index among the merged items); U1 task 2 (the rule and a unit-test case with an unresolved entry in chunk 1).
  - **D5** (the copy after `Reprinted`): master plan MC-L2.8 (`printed_at` = the check item's, else the run's `created_at`); U2 task 2; row L3·C5(c) adds "with `printed_at` T0".
  - **D6** (the ⋯ entry with access `unknown`): row L3·C17(a) gains the input access `unknown` with an enabled `Generate label`; U4 task 5 (enabled for `allowed` or `unknown`, ACC.1); U4 Files say L1·C19(c) is replaced for both states.
  - **D7** (the date in C12(e)): U4 task 1 (MC-L3.5's day rule, the existing `day()`; fixture of the current year or a fixed `now`); row L3·C12(e) input.
  - **D8** (the fake's templates, sizes, page counts): U1 task 5 (template = `LabelSpec.template`, else by product type, else the default, else none; `sizes` over items not `fail`); U2 task 3 (`page_count` = the rendered items of the size; the run answers `201`).
  - **D9** (the fake's read accessors, delegated): U2 "Decided at projection r0" (label state, `label.printed` log or count, `runs()` with key, item ids and `created_at`; the label-state accessor arrives in U1 with L3·C1(g)); U2 task 3 names `created_at` in `runs()`.
  - **D10** (a failed check while one is kept, delegated): U1 "Decided at projection r0": the slot is emptied and the previews cleared.
  - **D11** (one product vs batch wording, delegated): U4 "Decided at projection r0": by the number of product ids the dialog was opened with.
  - **D12** (`FakeLabelFiles::clear_previews`, delegated): U1 "Decided at projection r0" and U1 task 3: delegates to the inner `DiskLabelFiles::clear_previews`.
  - **D13** (`label_runs.rs` reaching `labels.rs`'s fields, delegated): U1 "Decided at projection r0": `pub(super)` fields or accessors, no behaviour change.
  - **D14** (order of the templates read in `check`, delegated): U1 "Decided at projection r0": the implementer's (rows assert counts only).
  - **D15** (`‹` `›` step order, delegated): U4 "Decided at projection r0": group display order.
  - **D16** (Close after `labels_denied`, delegated): U4 "Decided at projection r0": Close behaves as Cancel (`labelsPartial`; `not_found` → closes).
  - **D17** (`labels_off` from `labelsCheck`, delegated): U4 "Decided at projection r0": 09B's connection words as the ⋯ hint (`CONNECT_PRICING` / `connectionLine`), with `Open Settings → Pricing`.
  - **Nits:** L3·C7(a) "identical bodies" as JSON equality of `FakeRequest.body`; L3·C3(d) names its check (`["1", "3"]`), the product without a template and the scripted 503 for P1; L3·C8(c) adds "L3 adds no migration" (the L1·C13(f) instrument only sees names containing `label`), checked by reading in Review mode; L3·C11(a) and L3·C11(c) write the millimetres as `70.0` / `100.0` (`f64`); line drift corrected (`fake_pricing.rs` :787–873 and `fetch` :1147, `asset_protocol_scope` desktop `lib.rs:3127`, `blocking` `commands/conversations.rs:106`, `with_test_conversations` :156); U1 Notes hazard that today's fake `fetch` honours no scripted failure, hold, timeout or drop; Review mode records that MC-L2.12 is served by L1's rows (L1·C8(b), L1·C10(b)).
  Counts recounted by script from the three plan files: L3 U1 4 / 27, U2 5 / 24, U3 2 / 24, U4 6 / 32 = 17 / 107 (unchanged); totals 11 units, 50 criteria, 294 rows (master plan §2 unchanged). Outcomes are only sharpened (no row added or removed), so no new projection round (`projection: done`).
- **2026-11-23, implementer (Claude Opus, `pipeline-implementer`): implement r1** (handoff `handoffs/implementer/handoff_L3_generate_and_download_implement_r1.md`).
  - **Built.** U1 `00dd4e2f`: the L3 vocabulary and constants; `merge_plans`, `group_check`, `label_file_name`, `numbered_name` (unit tests); the `LabelFiles` port, `psm_infra::label_files::DiskLabelFiles` (`sips -Z 1200`, `.src.pdf` in `<previews>/labels/` always removed, `.part` then rename, numbered names), `FakeLabelFiles` in every `TestApp`, `psm_integrations::labels::{files_for, with_test_label_files, LABELS_FOLDER}`; `LabelsService::{check, preview}` in `services/label_runs.rs` with the one kept check; `FakePricing` plan and preview, and `fetch` now honours hold, time-outs, refusals, scripted failures and dropped answers. U2 `51815be6`: `merge_runs` (unit tests incl. the D3 case); `generate` (keys `sm-<uuid v4>` made once per chunk and stored, `RUN_RETRIES` at once, Try again continues at the failed chunk, `Busy` before any request, a drop guard clears `running`, the copy follows each finished chunk with one `LabelsChanged`) and `partial`; `FakePricing` runs (one per key), `runs()`, `printed_events()`, `fail_in_run`, `remove_label_product`; guide 02 / 03. U3 `9acd03f1`: `download`; the fake's artifact route and `expire_artifacts`; the five commands with DTOs and codes, registered; bindings regenerated; wrappers; guide 02 / 04. U4 `4ec02089`: `LabelDialog` / `useLabelDialog` / `LabelCheckView` / `LabelDoneView`, the enabled ⋯ entry (allowed / unknown; Reprint label when printed), a ⋯ with only that entry on Labels rows without a session, `Generate labels for N` wired, the selection after a run; CSS (preview on a white tile); L1·C19(c) and L2·C13(a) replaced by design; guide 02 frontend line.
  - **Tests.** 107 rows: `l3_labels_check::` 21 (C1–C3), `label_files` inline 6 (C4), `l3_labels_generate::` 24 (C5–C9), `l3_labels_download::` 7 (C10), desktop `l3_tests` 17 (C11), vitest `l3_label_check_dialog` 16 (C12–C14), `l3_label_done_dialog` 10 (C15–C16), `l3_label_wiring` 6 (C17); core mechanisms 9 new unit tests.
  - **Assumptions** (handoff § Assumptions, 24 items; the ones the owner may notice): Try again after a failed Generate only for "can't be reached" and "renderer busy", every other refusal offers Close (which shows what was generated so far, as Cancel does, D16); the batch footer always shows "{n} products will be marked Printed" (also 0); a failed check drops the kept check only on a provider error (D10), not on Off / Denied / Busy / an empty list; a finished generation is answered again even after the connection went off; downloads are offered for the files of every finished chunk (so from the partial result too).
  - **Deviations.** `Header.tsx` and `SessionRow.tsx` (listed under U4 Files) needed no change: `SessionActionsMenu` hosts the dialog. `docs/guide/02_architecture.md` also edited in U4 (flow §7A, the Frontend line was stale). L3·C1(i) and L3·C3(e) were committed in U1 without their `generate` half (no `generate` yet) and completed in U2 as the rows read. No migration, no column (`SqliteStore::migrations()` untouched).

