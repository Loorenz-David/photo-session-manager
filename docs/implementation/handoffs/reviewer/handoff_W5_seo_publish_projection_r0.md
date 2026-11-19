---
plan: W5_seo_publish
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-11-19
actor: opus (pipeline-projectionist)
---
# Projection of W5_seo_publish (round 0)

The search-texts phase can be built as planned once the orchestrator makes about a dozen
paragraph-sized corrections to the plan. Nothing needs the owner. Three corrections matter most.
First, one of the planned test recordings from Shopify cannot be replayed the way its test
describes. Second, one choice the plan hands to the implementer contradicts two of its own tests.
Third, the planned Shopify probe does not yet ask the questions that would catch Shopify quietly
changing or filling in the search texts. Several smaller fixes cover the "cannot remove" promise in
the field editor, a test command that does not run as written, and a planned unit test that cannot
be written in that crate. Once the orchestrator applies the amendments below, the implementer
prompt can be compiled. No new projection round is needed unless the orchestrator changes a
criterion's outcome rather than its wording.

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner. Every row below is a plan gap or a free choice the orchestrator can route
from the book and the master plan.

## Gate check
- Intention `docs/intention/intention.md`: **Status: RATIFIED** (2026-10-24); 08C RATIFIED 2026-11-19 (master_plan_writer header). ✔
- `depends_on: W4_writer_settings`: Tracker line `APPROVED (2026-11-19 …)`. ✔
- No upstream handoff for W5 in `OWNER_DECISIONS_PENDING`. ✔
- `blocked_on` precondition: `~/.psm_shopify.env` exists with `SHOPIFY_SHOP`, `SHOPIFY_CLIENT_ID`, `SHOPIFY_CLIENT_SECRET` (names checked, values not read). ✔ No `target/shopify_probe/` ledger exists in this worktree (no leftover ids).

## Decision ledger

| # | Decision point (artifact:line) | Class | Proposed routing |
|---|---|---|---|
| L1 | **W5·C4(b) cannot be replayed as written.** It feeds "`fixtures/seo_update.json`'s read-back product" to `product_seo`. That file is the `productUpdate` request/response pair (one fixture = one call, `fixtures/shopify.rs:1-4`). `FakeTransport::replay` keys on the fixture's **operation name** (`fake_transport.rs:98-103`). `product_seo` sends the read operation (`PRODUCT_SEO_READ`), so the replay panics `unmatched request`. Making it pass needs a hand-built synthetic reply, which is not the capture the row claims. | plan gap (high) | U2 task 1: after the write, capture a 4th call `PRODUCT_SEO_READ` → `fixtures/seo_read_after_update.json`. Point C4(b) at it and add it to U2 Files. The same read is the read-back step of WC8.4, so this grounds U3's read-back too. |
| L2 | **U3 delegated choice contradicts its own rows.** "Whether the SEO read reuses `product_fields` (one query) or `product_seo`" (plan:154). C6(a) asserts "one `product_seo` read before and one after", and C6(c) asserts "neither `product_seo` nor `update_seo` is called". If the implementer takes the `product_fields` option, C6(a) cannot pass and C6(c) passes for the wrong reason (`product_fields` is called anyway for the description, publish.rs:1963). | plan gap (high) | Delete the delegation. The SEO read is `product_seo` (U2 builds it for exactly this). The before and after reads stay as C6(a) states them. |
| L3 | **Rule 17: the probe cannot detect a Shopify fallback value.** Task 1 asks only "whether an unset field reads `null` or `""`" (plan:98). A third possibility is not checked: the admin API returning the product's title, or an excerpt of the body, for an unset SEO field. The test product is created with no description, so a body fallback could never show. If a fallback exists, C4(a)'s "maps to none" is false. WC8.2 would then import a value the owner never set as an `integration` row, and publish would write it back explicitly. Every test stays green and the behaviour is fictional. | plan gap (high, rule 17) | Task 1: create the probe product with a non-empty `descriptionHtml`. The evidence file records one of `null` / `""` / fallback (equal to the product title or a body excerpt) for each SEO field. **If the capture shows a fallback, stop at U2 task 1 with BLOCKED**, because WC8.2's "blank → none" then cannot tell unset from set. |
| L4 | **Rule 17: special characters are never checked for verbatim storage.** The capture specifies its values by length only (40 / 140 characters). Shopify's `descriptionHtml` capture already shows normalisation (`&quot;` stored as `"`, `description_update.json`). The writer's texts contain `|` ("Teak chair \| Beyo"), `&`, `ø`/`å`/`ö` (`writer/defaults.rs:171,232`) and quotes. If Shopify escapes or strips any of these in `seo`, WC8.4's read-back fails `AttributeVerification` on every such publish. No row or capture would have warned. | plan gap (medium, rule 17) | Task 1: the 40- and 140-character values contain `&`, `"`, `<`, `\|` and `å ö ø`. The evidence file records whether the read-back is byte-identical to what was sent (after trimming). If it is not, report BLOCKED with the observed mapping. |
| L5 | **The real adapter's `seo` parse in `PRODUCT_VALUES` has no row and no grounded shape.** U1 task 2 changes `PRODUCT_VALUES` and `product_values` (adapter.rs:884-930) before U2 captures anything. C2 runs on the fake adapter only, and C4 covers `product_seo` only. Also, p67 `c1a_shopify_adapter_reads_the_form_values_in_one_request` (p67…rs:155-171) uses a hand-built `ProductValues` reply with no `seo` object. | plan gap (medium, rule 17) | One seo mapping function (null / blank → none, missing `seo` object → none) shared by `product_values` and `product_seo`. Move the `PRODUCT_VALUES` / adapter edit from U1 to U2, or keep it in U1 under the shared function. Extend C4(a) to also replay the captured `seo` object inside a `ProductValues` reply, with outcome `RemoteValues.seo_title/seo_description = None` (a declared synthetic edit of the capture, `source` line). |
| L6 | **The planned core-mechanism unit test cannot be written in psm-core.** "`attributes::layout_keys`" (plan:52) takes `&dyn StoreTx`. Only `psm-infra` implements `StoreTx` (`crates/infra/src/sqlite/store.rs`), psm-core has no dev-dependencies, and `attributes.rs` has no `#[cfg(test)]` module, so `cargo test -p psm-core services::attributes` runs 0 tests today. A hand-written `StoreTx` would be a mock of an internal collaborator, which flow §4 forbids. | plan gap (medium) | Name a pure helper as the core mechanism, for example "append the always-present keys of `group` missing from a step's key list, in list order", called by `layout_keys`, `form_get` and `presets_get`. Unit-test that helper. Alternatively drop the unit-test entry and the command, since C1(a–c) cover the behaviour. |
| L7 | **"You cannot remove them" is not true in the UI.** `FIXED_KEYS` (`apps/desktop/src/services/attributes.ts:30`) decides whether Customize fields shows Remove (`CustomizeMode.tsx:156`). Without the two keys, Search title gets a Remove button, the save succeeds (task 1: `presets_set` accepts it), and the field comes back at the end on the next read. That contradicts the plan's point "you cannot remove them (as title, description and tags)" (plan:33). U1 Files lists no frontend file. | plan gap (medium) | U1: add `product.seo_title` and `product.seo_description` to `FIXED_KEYS` (W6 will add `local.restoration` the same way). Add `services/attributes.ts` to Files and `npx vitest run src/features/form` to U1's commands. No new row (B2 does not apply to a non-row edit). If the orchestrator wants it proven, use one `[rule]` row. |
| L8 | **Test commands do not run as written.** Several filters are given before `--`. Verified: `cargo test -p psm-testkit --test suite aaa:: bbb:: --no-run` fails with `error: unexpected argument 'bbb::' found`. This affects U1, U3 and U4's commands and the Review mode command (plan:79,160,202,208). The same template form is in W2/W3 plans. | plan gap (low) | `cargo test -p psm-testkit --test suite -- research::w5_seo_publish:: p10_attributes:: …` (libtest accepts several filters after `--`). Fold into the plan template. |
| L9 | **U4's commands miss the tests U4's changes reach.** The filter `research::w2_` in the testkit suite matches no stem (the testkit `research/` folder holds `w1_note_seam`, `w3_*`; the W2 tests are engine tests in `crates/research/tests/engine/`). `cargo test -p psm-research writer::` does not reach engine stems named `w2_writer_run::…`. The WC1.7 amendment changes `stale_condition`, which `tests/engine/w1_note.rs:765,772` assert true. A break there would first show at the closing gate. | plan gap (low) | U4 commands: add `cargo test -p psm-research --test engine -- w1_note:: w2_safeguard:: w2_writer_run::`. Drop `research::w2_` from the testkit command. |
| L10 | **C8(a)'s fixture claims a Shopify behaviour that is not grounded.** "the fake stores the title cut to 70 characters" (plan:148). U2's capture may show Shopify storing the long title whole or refusing it (`userErrors`, C5(b)). The row still exercises a real verification path. | plan gap (low, rule 17) | Reword C8(a)'s input: "the fake's read-back returns a title different from the one sent (cut to 70 characters; a synthetic mismatch, independent of what U2's capture shows)". Same outcome. |
| L11 | Fake shapes. The SEO store in `FakeShop`, a setter for C2/C6 setup, and the knob that changes the read-back (C8). | free choice | Delegate explicitly. Fixed points the rows depend on: op names `product_seo` / `update_seo` (C6 and C9 count them). The fake's `update_seo` stores the SEO **then** calls `effect(run, "update_seo")`, as `update_description` does (`fake_commerce.rs:812-817`), so `fail_after_effect("update_seo")` leaves the write applied (C9(a)). `product_seo` goes through `read("product_seo")`. |
| L12 | The probe's create document and safety. The plan says `productCreate`, but the probe has no such document. The existing grounded create is `PRODUCT_SET` (sv7, `shopify_probe.rs:634-655`). | free choice (with fences) | Delegate (`productSet` as sv7, or a new `productCreate`). Record which in the evidence file. Fences to state: `ledger_add("products", id)` right after the create. The `seo` step is **not** added to `all` and never calls the `cleanup` step: `cleanup` deletes every ledger product, file and definition on whatever shop the env names, which is now the real store. The delete uses the existing `PRODUCT_DELETE` (synchronous) on that one id. |
| L13 | Ordering inside U2. Task 1 captures with documents that task 2 defines. The `graphql.rs` header requires every fixture to be a pair of the exact document the adapter sends (graphql.rs:1-5). | plan gap (low) | "Write `PRODUCT_SEO_READ` / `PRODUCT_UPDATE_SEO` in `graphql.rs` first; the probe step imports them." |
| L14 | C3(b) outcome "its reason's 'Check:' names the length" (plan:69) is not one exact assertion. | plan gap (low, decidability) | The exact detail is `61 characters (at most 60)` (`postcheck.rs:299-303`). Outcome: `ai_reason` contains `Check: ` followed by that string. |
| L15 | C5(a) "the mutation is logged with the run as other product updates" (plan:108): which event and which fields are not named. | plan gap (low, decidability) | Outcome: a `shopify mutation` event with `run_id` = the run's id and `op = "update_seo"` (adapter.rs:485), read through the log capture other suite tests use (e.g. p16). |
| L16 | C14(a) boundary "the research panel status" (plan:195) has two candidates: `WriterApi::note` and `WriterApi::status`. | plan gap (low) | Name `WriterApi::status(session).stale_condition` (api.rs:194-213). |
| L17 | C10(a)'s fixture "A 1 cm" meets **both** new exclusions (single letter A, and followed by a unit), so the row stays green if either exclusion breaks (charter rule 2 companion). | plan gap (low) | Keep the row (it is the owner's sentence) and require the `check` core-mechanism unit tests to cover each exclusion alone: "A 1" with no unit; "I 2"; "PK 22 cm" (unit only). |
| L18 | C15(a) is a `grep` (review read), not a test at a boundary (charter rule 1, flow §4 "one behavioural test per row"). Its trace "W2/W3 backlog" is not a declared measurement. C12(a) and C13(a) trace "W3 backlog", which master_plan_writer §3 does not list. | plan gap (low, trace) | Move C15 to task 6 plus a Review-mode reading check (already "review reads"). Retrace C12 and C13 to `[rule]` (UI rows, §3). Counts become U4 5 criteria / 7 rows, total 14 / 27. |
| L19 | Task 6 "use core's `LINK_INTEGRATION`": that constant is private (`attributes.rs:257`, `const`, no `pub`). The literals are at `writer/note.rs:36`, `writer/writes.rs:23`, `writer/run.rs:1145`. | free choice | Delegate: make it `pub`, or re-export it once. |
| L20 | Port default for `product_seo` / `update_seo` "not supported": core `Error` has no generic unsupported variant (error.rs). The only other adapter, `research-tool/src/accept.rs` `StandInShop`, relies on the default. | free choice | Delegate the error value (e.g. `Error::Provider("search texts are not supported by this platform")`). It is unreachable on the publish path. |
| L21 | Task 4 "the same function wherever the writer shows money (panel and Settings → Writer)": Settings → Writer shows no money today. `money` exists only at `WriterPanel.tsx:4`. | reality note | No change. The implementer should not invent a cost display in Settings. |
| L22 | `master_plan_writer.md` §2 row W5 says 3 units / 9 criteria / 20 rows, and the totals line says 73 / 231. The plan now has 4 units / 15 / 28 (14 / 27 after L18). | upstream (count drift) | Orchestrator updates §2 row and totals (derived from the plan files). |

## Criteria decidability (both questions per row)

| Row | Test writable now, one exact outcome? | Fixture producible by the dependency? |
|---|---|---|
| C1(a) | yes (store `[title, description, tags]` via `presets_set`, which still accepts it, or via the store) | internal |
| C1(b) | yes | internal |
| C1(c) | yes | internal |
| C2(a) | yes (fake adapter needs an SEO setter, L11) | fake; real-adapter parse unserved → L5 |
| C2(b) | yes (`apply_remote` skips Human/Ai rows, attributes.rs:1207-1210) | internal |
| C3(a) | yes (the fake model's texts must pass post-check: seo_description 120–155 characters, no banned words, years or model tokens) | research fakes |
| C3(b) | **after L14** | research fakes |
| C3(c) | yes (`decide_write` saves `ai_prior` over a live `integration` row) | research fakes |
| C4(a) | yes, outcome fixed as None | **conditional: L3** (fallback case not considered) |
| C4(b) | **no → L1** (operation mismatch) | after L1, yes (capture) |
| C5(a) | **after L15** (the request body is assertable via `FakeTransport` records) | request shape is ours; `SEOInput{title,description}` is in WC8.6's schema read |
| C5(b) | yes (`errors::provider("update_seo", msg)` → `shopify update_seo: <msg>`) | grounded or declared-synthetic per plan; only `message` is read, so the `field` path does not matter |
| C6(a) | **after L2** | fake |
| C6(b) | yes (no `update_seo`; one `product_seo`) | fake |
| C6(c) | **after L2** | fake |
| C7(a) | yes (desired title = remote None → `null`) | fake |
| C8(a) | yes, fake knob (L11); wording L10 | synthetic, declare it (L10) |
| C9(a) | yes (`fail_after_effect("update_seo")`, `kill_and_restart`, test_app.rs:663; requires L11's apply-then-effect) | fake |
| C9(b) | yes (`partition` withholds low with no prior, publish_partition.rs:85-88; requires U1 layout append for the partition's layout) | internal |
| C9(c) | yes (conflict → held; r5_publish_hold helpers `conflict`, `attrs().accept`) | internal |
| C10(a) | yes (detail format `"<token> is not in the fields or the note"`, postcheck.rs:591); weak per L17 | internal |
| C10(b) | yes ("PK 22 is not in the fields or the note") | internal |
| C11(a) | yes (human description with no feedback is not `reviewed` by WC2, so the new rule 2 is the only cause) | research fakes |
| C11(b) | yes | research fakes |
| C12(a) | yes (`RequestedDto.reason.sentence` exists, bindings.ts:1344) | mocked command |
| C13(a) | yes (21 000 → `$0.02` with `toFixed(2)`) | internal |
| C14(a) | **after L16** | internal |
| C15(a) | **not a test → L18** | — |

## Trace verification
- WC8.1 → C1 · WC8.2 → C2 · WC8.3 → C9(b,c) · WC8.4 → C6, C7, C8 · WC8.5 → C9(a) · WC8.6 → C4, C5. Every item of WC8 is served; M18 (C9a) and M20 (C6b, C7a) are served.
- WC1.7 → C14 · WC5.10(iii) → C10 · WC6 rule 2 → C11: served, and the amended text is in master_plan_writer (lines 172-173, 341, 362-364).
- Untraced to a declared measurement: C12, C13, C15 (L18).

## Reality checks (paths and citations)
All paths in Files exist or are marked new. Checked: `attributes.rs` symbols (`builtin_definitions` :42, `PRODUCT_FIELDS` :39, `layout_keys` :157, `presets_get` :1064, `presets_set` :1090, `apply_remote` :1176, `form_get` :1413), `RemoteValues` (domain/publish.rs:91), `PRODUCT_VALUES` (graphql.rs), `fake_commerce.rs`, the p10, p12, p14 and p67 stems, `research/r5_publish_hold.rs`, `research/mod.rs`, `shopify_probe.rs` (`Redactor`, `gql`, ledger), `errors.rs`, `fixtures/description_update.json`, `docs/intention/evidence/shopify_admin.md` (exists: append), guide 02 §Publish (:306), guide 05 §Debugging recipes (:113), and guide 04's two rows named by U4 (:27, :28). Also the writer's `candidates` and `store_findings` (run.rs:1043, :1124), `safeguard::filter` (:177), `stale_condition` (note.rs:657), the four `backlog_writer/W1…W4` files, `WriterBlock.tsx`, `WriterPanel.tsx`, `WriterSection.tsx`. Prior-phase outputs verified in code: WC4.1 already requests `seo_title` and `seo_description` when the form carries them (`writer/input.rs:27-61`), so U1's built-ins are enough. `partition_in` already passes `builtin_definitions()` (publish.rs:125-131). Research's `never_written` refuses Product-group keys (`rules/write_table.rs:15-29`). Imported Shopify definitions are never group Product (adapter.rs:520-527), so with no stored preset the two fields already sit at the end of the Product step.

## Write perimeter
This file only. No plan, intention, code or tracker edits; no commit.

## Appendix: non-authoritative skeleton notes (discard; not guidance for the implementer)
- U3 block placement: after the description `if` (publish.rs:1963-1973). `F = item.run_attributes()`. Only `product.seo_*` with a non-blank string are considered. Read `product_seo` → desired → compare trimmed → `update_seo(run, …)` → `product_seo` → compare changed keys trimmed → `AttributeVerification{keys}`. Log `publish.seo item= written=`.
- Errors surface as `failed` with `failed_step = Update` through `run_item`'s error arm (publish.rs:1062-1073). C8(a)'s exact outcome is then `step Failed`, `failed_step Some(Update)`, error text `attribute_verification: product.seo_title`.
