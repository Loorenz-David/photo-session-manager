---
plan: W5_seo_publish
role: implement
round: 1
state: DONE
date: 2026-11-19
actor: claude-opus (pipeline-implementer)
---
# W5_seo_publish — implement r1

## What was built
- **Search title and Search description** are two new built-in Product fields. Every item type shows them at the end
  of the Product step, including types whose Product step was arranged before. Customize fields offers no Remove for
  them; a layout that places them elsewhere keeps that order.
- **The shop's search texts** are read into the form for a linked product as "from Shopify" values, just like the
  description: a blank one is left empty, and a value the owner typed is never replaced.
- **The writer now writes both search texts** whenever it writes texts. They are checked like the other texts: a
  search title over 60 characters is flagged "low" with its Check line, and a shop value becomes the suggestion's
  prior.
- **Publishing writes them to Shopify's search fields** after the description, and only when one differs (trimmed).
  A field the owner has no value for keeps the shop's value; nothing is ever emptied. Shopify's value is read back,
  and a mismatch fails the item at "update", naming the field. After an interruption the step reads again first, so
  it never writes twice. A low suggestion with no prior is withheld. A suggestion that replaces a shop value holds the
  item until the owner decides.
- **Grounded in Shopify's real replies.** A probe step created its own DRAFT test product on the store in
  `~/.psm_shopify.env`, read, wrote and read back its search texts, sent a 75-character title, and deleted the
  product. Results: unset fields are `null`. The read-back is byte-identical, `& " < | å ö ø` included. The
  75-character title was stored whole.
- **Carried fixes (U4).**
  - "A 1 cm chip" and "PK 22 cm" no longer count as model numbers.
  - The writer no longer raises text findings about a text it did not write (e.g. the owner's own description).
  - An Answer that queues nothing now shows why in a toast.
  - Costs under one cent show "< $0.01".
  - The "Photos changed" condition banner no longer appears when no Condition field is chosen.
  - One link constant replaces three `"shopify"` literals in the writer.

## Tests → criterion rows
| Row | Test | Shape |
|---|---|---|
| W5·C1(a) | `research::w5_seo_publish::w5_c1a_a_stored_product_preset_shows_both_search_fields_at_the_end` | exact ordered key list of the Product step |
| W5·C1(b) | `…::w5_c1b_layout_keys_include_both_search_fields` | `layout_keys` contains both keys |
| W5·C1(c) | `…::w5_c1c_a_layout_with_the_search_fields_placed_keeps_that_order` | `presets_get` and `form_get` equal the given order (nothing appended twice) |
| W5·C2(a) | `…::w5_c2a_the_shops_search_title_is_read_and_a_blank_description_is_not` | `integration` row with the exact value; no description row |
| W5·C2(b) | `…::w5_c2b_a_human_search_title_is_never_replaced_by_the_shops` | the human row is equal before and after the read |
| W5·C3(a) | `…::w5_c3a_the_writer_requests_and_writes_both_search_texts` | request `texts` include both; rows `ai` + `suggested` + `medium` with the exact texts |
| W5·C3(b) | `…::w5_c3b_a_search_title_over_the_limit_twice_is_written_low_with_its_check` | `low`; `ai_reason` after "Check: " contains "61 characters (at most 60)" |
| W5·C3(c) | `…::w5_c3c_a_shop_search_title_becomes_the_suggestions_prior` | `ai_prior` = {"Teak chair \| Beyo", valid, integration} |
| W5·C4(a) | `p12_shopify_adapter::w5_c4a_unset_search_texts_are_none_in_both_reads` | `Seo{None, None}` from the capture; `RemoteValues.seo_*` None from the declared synthetic `ProductValues` edit |
| W5·C4(b) | `p12_shopify_adapter::w5_c4b_search_texts_read_back_byte_identical_to_what_was_sent` | `Seo` equals the 40/140-char strings of the `seo_update` request |
| W5·C5(a) | `p12_shopify_adapter::w5_c5a_update_seo_sends_both_fields_with_null_and_logs_the_run` | document == `PRODUCT_UPDATE_SEO`; `product` == `{"id", "seo": {"title": "T", "description": null}}`; log line `shopify mutation` with `run_id=<id>` and `op="update_seo"` |
| W5·C5(b) | `p12_shopify_adapter::w5_c5b_a_refused_search_title_is_the_provider_error_naming_update_seo` | `Error::Provider("shopify update_seo: Title is too long")` (synthetic `userErrors` edit of the captured envelope, since Shopify did not refuse) |
| W5·C6(a) | `research::w5_seo_publish::w5_c6a_a_frozen_search_title_is_written_with_the_remote_description_kept` | `seo_writes` == [{"Teak chair, 1960s", "Shop meta"}]; writes seen at each `product_seo` read == [0, 1]; `done` |
| W5·C6(b) | `…::w5_c6b_search_texts_equal_after_trimming_are_not_written` | no `update_seo`; `done` |
| W5·C6(c) | `…::w5_c6c_no_frozen_search_text_reads_and_writes_nothing` | 0 `product_seo` reads, no writes |
| W5·C7(a) | `…::w5_c7a_a_frozen_description_alone_sends_the_unset_title_as_null` | `seo_writes` == [{None, description}] |
| W5·C8(a) | `…::w5_c8a_a_search_title_read_back_different_fails_the_update_naming_it` | `failed` at `update`; error names `product.seo_title` and not the description |
| W5·C9(a) | `…::w5_c9a_a_kill_after_the_search_write_resumes_without_writing_again` | one write before and after resume; `done`; the shop's seo equals the frozen values |
| W5·C9(b) | `…::w5_c9b_a_low_search_title_without_prior_is_withheld_and_not_sent` | `n_withheld` 1; origin class `withheld`; no write |
| W5·C9(c) | `…::w5_c9c_a_search_description_with_a_prior_holds_until_use_suggestion` | conflict keys == [seo_description]; `held`; after accept `done` with `update_seo` carrying the suggestion |
| W5·C10(a) | `research::w5_carried::w5_c10a_a_condition_sentence_with_a_and_a_unit_is_no_model_number` | no `Models` hit |
| W5·C10(b) | `research::w5_carried::w5_c10b_a_model_number_not_in_the_fields_is_one_hit` | exactly one `Models` hit naming "PK 22" |
| W5·C11(a) | `research::w5_carried::w5_c11a_no_text_finding_about_the_owners_own_description` | no `text_vs_facts` row for the description; the human row unchanged (checked red with the fix reverted) |
| W5·C11(b) | `research::w5_carried::w5_c11b_a_written_description_with_an_unknown_year_is_flagged_and_asked` | description `ai` `low`; exactly one open `text_vs_facts` finding for it |
| W5·C12(a) | `features/form/__tests__/w5_carried.test.tsx` `w5_c12a_…` | `toast.error` called with the returned sentence |
| W5·C13(a) | same file `w5_c13a_…` | "< $0.01", "$0.00", "$0.02" |
| W5·C14(a) | `research::w5_carried::w5_c14a_changed_condition_photos_show_no_banner_without_a_condition_field` | `status(session).stale_condition` false (checked red with the fix reverted) |

Core-mechanism unit tests (plan lists only): `services::attributes::tests::w5_append_always_present_*` (3);
`writer::postcheck::tests::w5_models_*` (3: "A 1", "I 2", "PK 22 cm"); `writer::safeguard::tests::w5_rule2_*` (1);
`writer::note::tests::w1_c10_stale_rules` extended with the amended case. Every test in the new files maps to a row.

## Assumptions
- `Seo {title, description}` is in `domain/publish.rs` from U1 on, because U1's shared seo mapping returns it.
- `presets_get` reports an appended always-present key at the end of the step, with no default.
- Fake (delegated shape): `product_seo` goes through `read("product_seo")`. `update_seo` records the call in
  `seo_writes`, stores the values (the title cut when `seo_title_cut` is set, the C8(a) knob) and then calls
  `effect(run, "update_seo")`.
- `publish.seo item=` is the **publish item id**, as in the other publish log lines. `verified=false` is added on a
  failed read-back. No line is written when nothing is frozen, because the step does nothing then.
- Frozen values are sent as frozen and compared trimmed. The probe showed Shopify stores them byte for byte.
- WC6 rule 2 drops a `missing_in_text` / `text_vs_facts` finding only when its `text_field` is non-empty and that
  text was not `written` or `unchanged` in the run. A model finding naming no text key is kept as before.
- The Answer toast is `toast.error(reason.sentence)` and nothing else. `money` stays in `WriterPanel.tsx`.
- Probe create document: `PRODUCT_SET` (sv7 shape, no SKU/barcode). This is recorded in the evidence file.

## Deviations
- Tests outside the units' file lists were updated to the new built-in list, as the plan's U1 hazard says (never
  weakened):
  - `p10_attributes::c6b` (U1).
  - Engine `w2_writer_run::w2_c4e`, `w2_writer_writes::w2_c19a` and `w2_c20b`, committed in U4, where the engine
    commands first ran. The search texts are now requested. C19a keeps the condition report as its "not requested"
    text. C20b now expects two extra "no text" outcomes.
  - `p13_publish_runs::c9b`, found by the closing gate: the count is now `builtin_definitions().len()` + shop
    definitions.
- `import_definitions` now refuses a shop key equal to any built-in key. Before, it refused only title, description
  and tags.
- No per-row red baseline was captured before the production edits. C11(a) and C14(a) were shown red by temporarily
  reverting their fixes (see Files changed). The other rows' tests were new behaviour that could not pass before:
  unknown keys, missing methods.
- The tracker actor is `opus`, not `codex`: this session ran on Claude Opus (`PSM_IMPLEMENTER=claude`).

## Files changed
U1 (3e0a878):
- `apps/desktop/src/services/attributes.ts`
- `crates/core/src/domain/publish.rs`
- `crates/core/src/services/attributes.rs`
- `crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs}`
- `crates/testkit/src/fake_commerce.rs`
- `crates/testkit/tests/suite/p10_attributes.rs`
- `crates/testkit/tests/suite/research/{mod.rs, w5_seo_publish.rs}`

U2 (5586490):
- `crates/core/src/ports/commerce.rs`
- `crates/integrations/examples/shopify_probe.rs`
- `crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs}`
- `crates/integrations/src/commerce/shopify/fixtures/{seo_read_unset, seo_update, seo_read_after_update, seo_update_long_title}.json`
- `crates/testkit/src/fake_commerce.rs`
- `crates/testkit/tests/suite/p12_shopify_adapter.rs`
- `docs/intention/evidence/shopify_admin.md`

U3 (2150e14):
- `crates/core/src/services/publish.rs`
- `crates/testkit/tests/suite/research/w5_seo_publish.rs`
- `docs/guide/{02_architecture.md, 04_change_impact.md, 05_working.md}`

U4 (bbee5e1):
- `apps/desktop/src/features/form/{WriterBlock.tsx, WriterPanel.tsx}`
- `apps/desktop/src/features/form/__tests__/w5_carried.test.tsx`
- `crates/core/src/services/attributes.rs`
- `crates/research/src/writer/{note.rs, postcheck.rs, run.rs, safeguard.rs, writes.rs}`
- `crates/research/tests/engine/{w2_writer_run.rs, w2_writer_writes.rs}`
- `crates/testkit/tests/suite/research/{mod.rs, w5_carried.rs}`
- `docs/guide/04_change_impact.md`

Closing (3e7ef83):
- `crates/testkit/tests/suite/p13_publish_runs.rs`
- `docs/implementation/plans/W5_seo_publish.md` (Tracker line, Review log)

Temporarily edited and restored byte for byte (red check of C11(a) and C14(a)): `crates/research/src/writer/note.rs`
and `crates/research/src/writer/safeguard.rs`.

Outside the repository: the probe wrote to `target/shopify_probe/` (captures, `seo_summary.json`, ledger; not
committed). On the store, it created and deleted one DRAFT product, `gid://shopify/Product/15530896752793`.

## Checkpoint SHA
3e0a878 (U1) → 5586490 (U2) → 2150e14 (U3) → bbee5e1 (U4) → 3e7ef83 (closing gate fix, tracker, review log).

## Stamp
Tree 3e7ef83, run once at the end, one command at a time:

| Command | Result |
|---|---|
| `cargo test --workspace` | First run: 1032 passed, 1 failed (`p13_publish_runs::c9b`, it pinned the old count of 3 built-ins). Fixed, then re-ran only that test: 1/1 ok. The skipped doc-tests were run alone: ok. Other binaries: core 102, desktop 61, infra 11, integrations 40, research lib 255, engine 328, tool 30, all green. |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `npm test` | 661 passed |

No pre-existing failures. `bindings.ts` is unchanged: no command or DTO changed.

## Manual try-out
On scratch data (`PSM_DATA_DIR=$HOME/psm-research-dev`) with the dev store connected:
1. Open a session's Product step. "Search title" and "Search description" are at the end. Customize fields shows no
   Remove for them.
2. For a session linked to a product that has search texts in Shopify, those texts appear marked as from Shopify.
3. Press **Write texts**. AI review now also lists the two search texts. A search title over 60 characters shows as
   flagged.
4. Accept them and publish. In Shopify's admin, the product's "Search engine listing" shows them.
5. Publish again without changes. Nothing is written to the search fields, and the log has
   `publish.seo item=<n> written=false`.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
