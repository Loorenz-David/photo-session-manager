---
phase: W5_seo_publish
title: Search title and search description — two new built-in Product fields that every item type shows; read from the shop for linked items; written by the writer and checked like the other texts; publishing writes them to Shopify's search fields, only when they differ, never clearing one you left empty, and safely again after an interruption
difficulty: high
projection: done (r0 2026-11-19 AMENDMENTS_REQUIRED, amendments applied by the orchestrator; no new round per the handoff)
depends_on: [W4_writer_settings]
blocked_on: U2 task 1 captures Shopify replies with the probe credentials of P12 (`~/.psm_shopify.env` on the dev Mac); without them the implementer reports BLOCKED at U2 task 1 and U1 stands committed
---
# W5 — Search title and search description

**Tracker:** IMPLEMENTED (2026-11-19, opus, implement r1: 4 units checkpointed 3e0a878, 5586490, 2150e14, bbee5e1 + gate fix; 27 rows, each a boundary test (w5_seo_publish 16, p12 w5_ 4, w5_carried 5 + vitest 2) plus 7 core-mechanism unit tests; probe `seo` captured null/null, byte-identical read-back, 75-char title stored whole; gate green: suite 1033, clippy, fmt, typecheck, npm test 661)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `08C` §9 by WC8 of `master_plan_writer.md` §5 (items 1–6). (U1) The two built-in fields, always present in the Product step, read from the shop, requested by the writer. (U2) The Shopify adapter: read and write `seo`, grounded in captured replies. (U3) The publish step. **Not here:** Restoration (W6, which reuses U1's always-present mechanism); Google Merchant Center (deferred, 08C §12).

## How to work this plan (flow.md §10, master_plan_writer.md §3)
One runner phase of 4 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Projection runs before the implementer (`projection: yes`): it checks U2's capture task and every row whose fixture is a Shopify shape (rule 17). Row ids `W5·C<n>(<row>)`; tests named after their row. Open details: the smallest option consistent with 07A MC-11, MC-13 and WC8, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_writer/W5_seo_publish.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The two fields, always present, read from the shop, written by the writer | 3 | 8 | **yes** |
| U2 | Shopify adapter: read and write `seo` | 2 | 4 | **yes** |
| U3 | The publish step | 4 | 8 | **yes** |
| U4 | Carried fixes from W1–W4 reviews (orchestrator, 2026-11-19) | 5 | 7 | **yes** |
| **Total** | | **14** | **27** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §10) · `master_plan_writer.md` §4, **§5 WC4.8, WC8** · book `08C` §9 · `07A` MC-10, MC-11, MC-13 · `08B` MC-A10 · `09A` · `docs/guide/02_architecture.md` §Publish · `docs/guide/05_working.md` §Debugging recipes (publish) · `crates/integrations/examples/shopify_probe.rs` (how P12 captured fixtures).

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **Every item type shows the two search fields at the end of the Product step,** even where you arranged that step before; you cannot remove them (as title, description and tags).
- **A linked product's existing search texts are read into the form;** a writer text that replaces one is a conflict at publish, as for the description.
- **Publishing writes the search fields only when they differ from the shop's,** and never empties one you left blank.
- **A search title over Shopify's limit is sent as you wrote it;** what Shopify does with it is recorded in the evidence (U2).

## Unit 1 — The two fields, always present, read from the shop, written by the writer

**Unit id:** `W5·U1` · **Rows:** W5·C1–C3 · **Changes app files:** yes

### Goal
WC8.1 (built-in definitions; always present via `layout_keys` and `form_get`), WC8.2 (the remote read), and the writer requesting `seo_title` and `seo_description` (WC4.1 now finds them).

### Read first
`crates/core/src/services/attributes.rs` (`builtin_definitions`, `PRODUCT_FIELDS`, `layout_keys`, `form_get`, `presets_get`, `presets_set`, the remote read that stores `remote.description`) · `crates/core/src/domain/publish.rs` (`RemoteValues`) · `crates/integrations/src/commerce/shopify/{adapter.rs (product_values), graphql.rs (PRODUCT_VALUES)}` · `crates/testkit/src/fake_commerce.rs` · `crates/testkit/tests/suite/{p10_attributes.rs, p67_form_values_defaults_and_batch_step.rs}`.

### Files
Edited: `crates/core/src/services/attributes.rs`, `crates/core/src/domain/publish.rs`, `crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs}` (the read only), `crates/testkit/src/fake_commerce.rs`, `apps/desktop/src/services/attributes.ts` (`FIXED_KEYS`). New: `crates/testkit/tests/suite/research/w5_seo_publish.rs` (its `mod` line in `research/mod.rs`). Tests that enumerate the built-in definitions or a stored Product layout follow (grep `builtin_definitions`, `PRODUCT_FIELDS`).

### Core mechanisms (the only functions with unit tests)
A pure helper in `attributes.rs`: "append the always-present keys of a group that a step's key list lacks, in list order", called by `layout_keys`, `form_get` and `presets_get`; unit-tested in a `#[cfg(test)]` module of `attributes.rs` (no `StoreTx`).

### Ordered tasks
1. `builtin_definitions()` gains `product.seo_title` (Text, "Search title") and `product.seo_description` (Textarea, "Search description"), group Product. An "always present" list (the two now; W6 adds one) is appended to the end of its step by `layout_keys`, `form_get` and `presets_get` when the stored preset lacks it; `presets_set` accepts them like any built-in.
2. `RemoteValues.seo_title`, `.seo_description` (`Option<String>`, blank → none); `PRODUCT_VALUES` reads `seo { title description }` through **one seo mapping function** in the Shopify adapter (null / blank / missing `seo` object → none), the same function U2's `product_seo` uses; the remote read stores them as `integration` rows exactly as `remote.description` (same conditions). Existing hand-built `ProductValues` replies without `seo` (p67 `c1a_…`) keep passing (missing object → none).
3. `FIXED_KEYS` (`apps/desktop/src/services/attributes.ts`) gains `product.seo_title` and `product.seo_description`, so Customize fields shows no Remove for them (the plan's "you cannot remove them").
4. Tests for the rows.

### Criteria (3 criteria, 8 rows)
Boundary: `AttributesService` through `TestApp` with the fake commerce adapter; a `write` run on the wired app with the research fakes.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W5·C1(a) | `form_get` | an item type with a stored Product preset [title, description, tags] (saved before this change) | the Product step lists title, description, tags, Search title, Search description, in that order | WC8.1 |
| W5·C1(b) | `layout_keys` | the same | includes `product.seo_title` and `product.seo_description` | WC8.1 |
| W5·C1(c) | `presets_set` | a Product layout [title, Search description, description, tags, Search title] | stored in that order; `form_get` shows that order, nothing appended twice | WC8.1 |
| W5·C2(a) | the remote read (P67) | the fake shop's product has seo title "Teak chair | Beyo" and description "" | a row `product.seo_title` `integration` "Teak chair | Beyo"; no `product.seo_description` row | WC8.2 |
| W5·C2(b) | same | the item already holds a `human` `product.seo_title` | the human row is unchanged (as the description rule) | WC8.2, M20 |
| W5·C3(a) | a `write` run (wired app, fakes) | default writer settings; no seo rows | the request's `texts` include `seo_title` and `seo_description`; both are written `ai` + `suggested` at `medium` with clean texts | WC4.1, 08C §9 |
| W5·C3(b) | same | the answer's search title has 61 characters, twice | written at `low`; its `ai_reason` contains `Check: ` followed by `61 characters (at most 60)` | WC5.1, WC4.5 |
| W5·C3(c) | same | `product.seo_title` `integration` "Teak chair | Beyo" | the suggestion's `ai_prior` = {"Teak chair | Beyo", valid, integration} | WC4.4 |

### Delegated choices
Where the always-present list lives (a constant beside `PRODUCT_FIELDS`) · how `presets_get` reports an appended key.

### Notes
- Hazard: a test that pins the Product step's field count or the built-in list changes; each is updated to the new list in this unit, never weakened.

### Unit test commands
`cargo test -p psm-core services::attributes` · `cargo test -p psm-testkit --test suite -- research::w5_seo_publish:: p10_attributes:: p67_form_values_defaults_and_batch_step::` · `cd apps/desktop && npx vitest run src/features/form` · `cargo clippy -p psm-core -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Unit 2 — Shopify adapter: read and write `seo`

**Unit id:** `W5·U2` · **Rows:** W5·C4–C5 · **Changes app files:** yes

### Goal
WC8.6: the shapes Shopify actually returns, captured; `CommerceAdapter::product_seo` and `update_seo` in the Shopify adapter and the fake.

### Read first
`crates/integrations/examples/shopify_probe.rs` (steps, `Redactor`, the `gql <file>` step) · `crates/integrations/src/commerce/shopify/{adapter.rs (update_product, update_description, product, product_fields), graphql.rs, errors.rs, fixtures/description_update.json}` · `crates/core/src/ports/commerce.rs` · `crates/testkit/tests/suite/p12_shopify_adapter.rs`.

### Files
New: `crates/integrations/src/commerce/shopify/fixtures/{seo_read_unset.json, seo_update.json, seo_read_after_update.json, seo_update_long_title.json}` (captured, sanitized). Edited: `crates/integrations/examples/shopify_probe.rs` (an `seo` step), `crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs}`, `crates/core/src/ports/commerce.rs`, `crates/core/src/domain/publish.rs` (`Seo {title, description}`), `crates/testkit/src/fake_commerce.rs`, `crates/testkit/tests/suite/p12_shopify_adapter.rs`; `docs/intention/evidence/shopify_admin.md` (new or appended: what was captured, the API version, the 70-character question).

### Core mechanisms
None.

### Ordered tasks
1. **Documents first (L13).** Write `PRODUCT_SEO_READ` and `PRODUCT_UPDATE_SEO` (task 3) in `graphql.rs` before the capture; the probe step imports them, so every fixture is a pair of the exact document the adapter sends.
2. **Capture (rule 17; owner-attended only in that it uses the stored probe credentials).** Add a probe step `seo` that **creates its own test product** (owner 2026-11-19: `~/.psm_shopify.env` now points to the **real store**; the owner allowed a test product), titled `PSM-PROBE (test, do not publish)`, `status: DRAFT`, with a **non-empty `descriptionHtml`** (a sentence of plain text), no sales-channel publication, no media. The create document is delegated (`PRODUCT_SET` as the probe's sv7, or a new `productCreate`); record which in the evidence file. **Fences:** `ledger_add("products", id)` right after the create; the `seo` step is **not** added to `all` and never calls the `cleanup` step (cleanup deletes every ledger item on whatever shop the env names, now the real store); the step touches no other product. Calls, one fixture each:
   - `PRODUCT_SEO_READ` before any write → `seo_read_unset.json`;
   - `PRODUCT_UPDATE_SEO` with a 40-character title and a 140-character description that both contain `&`, `"`, `<`, `|` and `å ö ø` → `seo_update.json`;
   - `PRODUCT_SEO_READ` again → `seo_read_after_update.json`;
   - `PRODUCT_UPDATE_SEO` with a 75-character title → `seo_update_long_title.json`;
   - then delete the test product with the existing `PRODUCT_DELETE` on that one id (if the delete fails, leave it as DRAFT and say so in the evidence file).
   Run it once with `~/.psm_shopify.env`, sanitize with the probe's `Redactor`, commit the four files, and record in `docs/intention/evidence/shopify_admin.md`: the API version and the create document used; for **each** SEO field when unset, one of `null` / `""` / **fallback** (equal to the product title or an excerpt of the body); whether the read-back after the update is byte-identical (after trimming) to what was sent; whether a 75-character title is stored whole, cut or refused (and the `userErrors` text).
   **Stop and report BLOCKED** (U1 stands committed) if: the credentials are absent; the unset read shows a fallback (WC8.2's "blank → none" could not tell unset from set); or the read-back is not byte-identical (report the observed mapping).
3. Documents: `PRODUCT_SEO_READ` (`product(id) { id seo { title description } }`) and `PRODUCT_UPDATE_SEO` (`productUpdate(product: $product) { product { id seo { title description } } userErrors { field message } }`). `CommerceAdapter::product_seo(external_id) -> Result<Seo>` (through U1 task 2's shared seo mapping function: null and blank → none) and `update_seo(run, external_id, &Seo)` (both fields sent; a none field sends `null`); `userErrors` → the existing provider error. Defaults on the port: not supported (an adapter without SEO never receives a call: U3 asks only when frozen values exist, and the fake implements both).
4. Tests for the rows, replaying the captures through the existing transport stub as `p12` does.

### Criteria (2 criteria, 4 rows)
Boundary: the Shopify adapter over the transport stub replaying the captures (outcome = the parsed value, the request sent, the error).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W5·C4(a) | `product_seo` | `fixtures/seo_read_unset.json` (captured, WC8.6) | `Seo {title: None, description: None}` (whatever the capture shows for an unset field, null or "", maps to none); and the same captured `seo` object placed inside a `ProductValues` reply (a declared synthetic edit, `source` line) gives `RemoteValues.seo_title` and `.seo_description` = None | WC8.6, rule 17 |
| W5·C4(b) | same | `fixtures/seo_read_after_update.json` (captured) | `Seo` with the 40- and 140-character values sent, byte-identical | WC8.6 |
| W5·C5(a) | `update_seo` | a confirmed run; `Seo {title: "T", description: None}` | the request body's `product` is `{"id": <id>, "seo": {"title": "T", "description": null}}` and the document is `PRODUCT_UPDATE_SEO`; a `shopify mutation` log event with `run_id` = the run's id and `op = "update_seo"`, read through the log capture other suite tests use (e.g. p16) | WC8.4, H1 |
| W5·C5(b) | same | a reply with `userErrors` [{field: ["seo","title"], message: …}] (taken from `seo_update_long_title.json` if Shopify refused, else a reply in the captured envelope with that one array changed) | the provider error naming `update_seo` and the message | WC8.6, rule 17 |

### Delegated choices
The probe step's code shape · the create document (with the fences of task 2) · the evidence file's layout · the port default's error value (e.g. `Error::Provider("search texts are not supported by this platform")`, unreachable on the publish path).

### Notes
- Rule 17: W5·C4(a)'s outcome depends on what Shopify returns for an unset field; the capture decides it, and the row's fixture cell names the file. Projection checks the capture task before implementation.

### Unit test commands
`cargo test -p psm-testkit --test suite p12_shopify_adapter::` · `cargo test -p psm-integrations` · `cargo clippy -p psm-integrations -p psm-core -p psm-testkit --all-targets -- -D warnings`

## Unit 3 — The publish step

**Unit id:** `W5·U3` · **Rows:** W5·C6–C9 · **Changes app files:** yes

### Goal
WC8.3–5 in `PublishService::update`, after the description.

### Read first
Master plan §5 WC8 · `crates/core/src/services/publish.rs` (`update`, `run_attributes`, `confirm`, `partition_in`, the description block, `AttributeVerification`) · `crates/testkit/tests/suite/{p14_publish_update.rs, research/r5_publish_hold.rs}` and the kill-and-resume helpers they use.

### Files
Edited: `crates/core/src/services/publish.rs`, `crates/testkit/tests/suite/research/w5_seo_publish.rs`; `docs/guide/{02_architecture.md (§Publish), 04_change_impact.md, 05_working.md (the log line)}`.

### Core mechanisms
None.

### Ordered tasks
1. The SEO block of WC8.4 with the log line `publish.seo item= written=`.
2. Tests for the rows; the guide.

### Criteria (4 criteria, 8 rows)
Boundary: `PublishService` through `TestApp` with the fake commerce adapter (outcome = the adapter calls received, publish item states, errors).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W5·C6(a) | a publish of a linked item | frozen `product.seo_title` "Teak chair, 1960s" (human), remote seo title none and seo description "Shop meta" | one `update_seo` with {"Teak chair, 1960s", "Shop meta"}; one `product_seo` read before and one after; the item reaches `done` | WC8.4 |
| W5·C6(b) | same | frozen values equal (after trimming) to the remote ones | no `update_seo` call; `done` | WC8.4, M20 |
| W5·C6(c) | same | no frozen SEO value | neither `product_seo` nor `update_seo` is called | WC8.4 |
| W5·C7(a) | same | frozen `product.seo_description` only; remote title none | `update_seo` with title `None` (sent as null) and the frozen description | WC8.4, M20 |
| W5·C8(a) | same | the fake's read-back returns a title different from the one sent (cut to 70 characters; a synthetic mismatch, independent of what U2's capture shows); frozen title of 75 | the item `failed` at `update` with `AttributeVerification` naming `product.seo_title` | WC8.4 |
| W5·C9(a) | kill and resume | the process killed after the fake accepted `update_seo` and before the step committed; resume | after resume no second `update_seo` (the read finds them equal); `done`; the fake's final seo equals the frozen values | WC8.5, M18, H4 |
| W5·C9(b) | `publish_preview`, `publish_confirm` | `product.seo_title` `ai` + `suggested` `low` with no prior | counted withheld; no `update_seo` for it | WC8.3, MA8 |
| W5·C9(c) | same | `product.seo_description` `ai` + `suggested` `medium` with prior {"Shop meta", integration} | a conflict; the item `held`; after "Use suggestion" the item runs and `update_seo` carries the suggestion | WC8.3, MA17 |

### Delegated choices
The exact place of the block after the description. (Not delegated: the SEO reads are `product_seo`, one before and one after, as C6(a) states.)

### Notes
- Hazard (H1, M17): `update_seo` takes the confirmed run like every mutation; never call it from preview.
- Fake (delegated shape, fixed points): op names `product_seo` / `update_seo` (C6, C9 count them); the fake's `update_seo` stores the SEO **then** calls `effect(run, "update_seo")`, as `update_description` does, so `fail_after_effect("update_seo")` leaves the write applied (C9(a)); `product_seo` goes through `read("product_seo")`; a knob changes the read-back for C8(a).

### Unit test commands
`cargo test -p psm-testkit --test suite -- research::w5_seo_publish:: p14_publish_update:: research::r5_publish_hold::` · `cargo clippy -p psm-core --all-targets -- -D warnings`

## Unit 4 — Carried fixes from the W1–W4 reviews

**Unit id:** `W5·U4` · **Rows:** W5·C10–C14 · **Changes app files:** yes · Added by the orchestrator 2026-11-19 from `docs/implementation/backlog_writer/W1…W4`; each fix is small and has a stated rule.

### Goal
Six rough edges the owner would meet on the first real runs: a normal condition sentence read as a model number; questions about a draft the owner never saw; an answer that silently queues nothing; "$0.00" costs; a "Photos changed" banner Look again cannot clear; stale guide rows and a triplicated constant.

### Read first
Master plan §5 **WC1.7, WC5.10(iii), WC6 rule 2** (amended 2026-11-19) · the four backlog files above · `crates/research/src/writer/{postcheck.rs, run.rs (candidates, store_findings), safeguard.rs, writes.rs, note.rs}` · `crates/research/src/writer/note.rs` or wherever `stale_condition` is computed · `apps/desktop/src/features/form/{WriterBlock.tsx, WriterPanel.tsx}`, `apps/desktop/src/features/settings/WriterSection.tsx` · `crates/core/src/services/attributes.rs` (`LINK_INTEGRATION`).

### Files
Edited: the files above; `docs/guide/04_change_impact.md` (row "The research runs list (`research::store::runs`)…": the views leave out `browse`, `look` and `write`; row "A writer command or `WriterStatus` field": add `WriterSection.tsx`, `WriterInstruction.tsx` and `npx vitest run src/features/settings`). New: `crates/testkit/tests/suite/research/w5_carried.rs` (its `mod` line), `apps/desktop/src/features/form/__tests__/w5_carried.test.tsx`.

### Core mechanisms
`writer::postcheck::check` (Models rule) · `writer::safeguard::filter` (rule 2) · the `stale_condition` function · `money`.

### Ordered tasks
1. WC5.10(iii) as amended: single-letter A and I are not model tokens; a token followed by a unit is not one.
2. WC6 rule 2 as amended: drop `missing_in_text` / `text_vs_facts` for a text whose write outcome in the run is not `written` or `unchanged`.
3. `WriterBlock` Answer: when the returned `Requested.queued` is false, show `reason.sentence` as a toast (Sonner, as the Write texts button does).
4. `money`: below one cent (10 000 µUSD) and above zero show "< $0.01"; zero shows "$0.00"; otherwise unchanged. Only the writer panel shows money today; add no cost display elsewhere.
5. WC1.7 as amended.
6. One constant: `writer/writes.rs`, `writer/note.rs` and `run.rs` use core's `LINK_INTEGRATION` (private today in `attributes.rs`: make it `pub` or re-export it once, delegated); after this, `crates/research/src/writer` holds no string literal `"shopify"` (checked by the reviewer by reading, not a row); the two guide rows.

The `check` unit tests cover each new Models exclusion alone (C10(a) meets both): "A 1" with no unit → no hit; "I 2" → no hit; "PK 22 cm" (unit only) → no hit.

### Criteria (5 criteria, 7 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W5·C10(a) | `check` | English; condition report "Very Good condition. A 1 cm veneer chip on the front edge (photo 3). Stable." with the damage field holding that chip | no `Models` hit | WC5.10(iii) |
| W5·C10(b) | same | description "Model PK 22 lounge chair." with no "PK22" in the fields or note | one `Models` hit naming "PK 22" | WC5.10(iii) |
| W5·C11(a) | a `write` run (wired app, fakes) | `product.description` `human` (no feedback); the fake model's description says "made in 1958", no 1958 in the fields | no `writer_findings` row of type `text_vs_facts` for `product.description`; the human row unchanged | WC6 rule 2 |
| W5·C11(b) | same | `product.description` empty; the same answer twice | the description written `low` (flagged) and one open `text_vs_facts` finding for it | WC6 rule 2, WC5 |
| W5·C12(a) | `WriterBlock` (vitest) | an open question; `writer_finding_answer` resolves `{queued: false, reason: {sentence: "Research is running on this session."}}` | a toast with that sentence | [rule] |
| W5·C13(a) | `money` (vitest) | 4 000; 0; 21 000 | "< $0.01"; "$0.00"; "$0.02" | [rule] |
| W5·C14(a) | `WriterApi::status(session).stale_condition` | an N6 exists; no Condition field in `fill_keys`; a condition photo changed | `stale_condition` false (no banner) | WC1.7 |

### Delegated choices
The toast's wording beyond the returned sentence · whether `money` moves to a shared util.

### Unit test commands
`cargo test -p psm-testkit --test suite -- research::w5_carried:: research::w1_` · `cargo test -p psm-research writer::` · `cargo test -p psm-research --test engine -- w1_note:: w2_safeguard:: w2_writer_run::` · `cd apps/desktop && npx vitest run src/features/form src/features/settings`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-testkit --test suite -- research::w5_seo_publish:: research::w5_carried:: p12_shopify_adapter:: p14_publish_update:: p10_attributes::` · `cargo test -p psm-integrations` · clippy on the touched crates, fmt. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: the fixtures are captures (their `source` line, sanitized), and the evidence file records what Shopify did with the long title; no SEO mutation can happen without a confirmed run; a blank frozen value never reaches `update_seo` as a change; `crates/research/src/writer` holds no `"shopify"` literal (U4 task 6); the probe's `seo` step is not in `all` and never calls `cleanup`. Notes go to `docs/implementation/backlog_writer/W5_seo_publish.md`.

## Manual try-out (the narrative is built from this)
On scratch data with the dev store connected:
1. Open a session's Product step: "Search title" and "Search description" sit at the end.
2. Press **Write texts**: AI review now also lists the two search texts.
3. Accept them, publish. In Shopify's admin, the product's "Search engine listing" shows them.
4. Publish again without changes: nothing is written to the search fields (the publish log has `publish.seo … written=false`).

## Review log

- **2026-11-19, orchestrator: probe target changed.** Owner: the store in `~/.psm_shopify.env` is now the real shop; create a test product for the probe. U2 task 1 amended: create a DRAFT product, probe it, delete it; never touch an existing product.
- **2026-11-19, orchestrator: U4 added** (carried fixes from the W1–W4 reviewer notes; master plan WC1.7, WC5.10(iii) and WC6 rule 2 amended the same day).
- **2026-11-19, orchestrator: projection r0 amendments applied** (`handoffs/reviewer/handoff_W5_seo_publish_projection_r0.md` L1–L22): a 4th capture `seo_read_after_update.json`; the probe's product has a body and its values carry special characters, with BLOCKED on a fallback or a non-identical read-back; one shared seo mapping function; `FIXED_KEYS`; a pure core helper as the unit-tested mechanism; `--` before test filters; C3(b), C5(a), C8(a), C14(a) worded exactly; the U3 read delegation removed; the fake's fixed points; probe fences (ledger, not in `all`, no `cleanup`); C15 moved to a reading check, C12/C13 retraced `[rule]`; U4 commands reach the engine tests. Outcomes unchanged, so no new projection round.
- **2026-11-19, implementer (Claude Opus, r1): built.** U1 `product.seo_title` / `product.seo_description` built-in
  (Text / Textarea, Product), `ALWAYS_PRESENT` + the pure `append_always_present` applied in `layout_keys`, `form_get`,
  `presets_get`; `RemoteValues.seo_*` read through one `adapter::seo_of`; `FIXED_KEYS`. U2 `PRODUCT_SEO_READ`,
  `PRODUCT_UPDATE_SEO`, `CommerceAdapter::product_seo` / `update_seo` (port default refuses with
  `SEO_UNSUPPORTED`), the fake's ops; probe step `seo` (own DRAFT product via `PRODUCT_SET`, ledgered, deleted; not in
  `all`, no `cleanup`) run once on the store in `~/.psm_shopify.env`: unset `seo` fields are `null`, the read-back is
  byte-identical, a 75-character title is stored whole (no `userErrors`); four fixtures, evidence `shopify_admin.md`
  §SEO search texts. U3 `PublishService::update_seo` after the description, log `publish.seo item= written=`. U4 the
  six carried fixes; `LINK_INTEGRATION` made `pub` in core and used by `writer/{writes,note,run}.rs` (no `"shopify"`
  literal left there).
  **Assumptions:** `Seo` lives in `domain/publish.rs` from U1 (U1's shared mapping returns it); `presets_get` reports an
  appended key at the end with no default; the fake stores a cut title through a `seo_title_cut` knob (C8(a)) and
  records `seo_writes`; `publish.seo item=` is the publish item id (as the other publish log lines), with
  `verified=false` added on a failed read-back, and no line when nothing is frozen; frozen values are sent as frozen
  (compared trimmed); WC6 rule 2 drops a text finding only when its `text_field` is non-empty and not written or
  unchanged in the run; the Answer toast is `toast.error(reason.sentence)` only; `money` stays in `WriterPanel.tsx`.
  **Deviations:** tests outside the U-file lists updated to the new built-in list (plan hazard): `p10_attributes::c6b`
  (U1), engine `w2_writer_run::w2_c4e` and `w2_writer_writes::w2_c19a` / `w2_c20b` (the search texts are now
  requested; C19a keeps the report as its not-requested text), `p13_publish_runs::c9b` (definition count from
  `builtin_definitions().len()`). `import_definitions` now refuses a shop key equal to any built-in key (was: the three
  fixed ones). A red baseline per row was not captured before production edits; C11(a) and C14(a) were checked red by
  temporarily reverting their fixes (restored).
