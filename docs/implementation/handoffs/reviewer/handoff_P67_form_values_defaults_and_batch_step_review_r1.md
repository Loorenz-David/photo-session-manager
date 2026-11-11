---
plan: P67_form_values_defaults_and_batch_step
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-11
actor: opus
---
# Review — P67 form values, defaults and saved values, no auto type, batch step memory (round 1)

Gate: tracker row `IMPLEMENTED`; checkpoint `a4ee3aec5a79a9b25cd41a19b7da5aad885de566` exists → passed. First review of
the phase (no earlier rounds). Process authority: `docs/process/flow.md` (no mutation probes; blocking only B1–B4).

## Verified correct
- **Perimeter.** `git show --stat a4ee3ae` lists exactly the 27 declared paths (the handoff, the 28th, is left for the
  runner). Production/test code unchanged since the checkpoint (`git diff a4ee3ae HEAD -- crates apps` empty).
- **Checks, reviewer run on code tree `a4ee3ae`:** `cargo test --no-fail-fast -p psm-testkit` with p67 7/7, p01 20/20, p11 1/1,
  p59 5/5, p64 8/8, p65 8/8; `cargo fmt --all -- --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings`
  exit 0; `npm run typecheck` exit 0; `vitest run src/features/form src/features/publish src/features/history` 7 files,
  77/77. Implementer full-suite stamp consumed: it covers every binary (p61 c1a load flake re-run alone green; p62–p67
  and `psm-desktop` run after cargo stopped). It does not contradict anything seen here.
- **C1(a)** `AttributesService::read_integration_values` → exactly one adapter read (`product_values`), no mutation op.
  `apply_remote` stores the title through `items.title` + `item_remote_titles`, description (HTML → paragraphs,
  `description_paragraphs`, the inverse of `description_html`), tags and every metafield incl. a condition field with
  provenance `integration`. The type follows the existing changelog-42 base rule. The real adapter issues one `ProductValues`
  GraphQL request (pages on only past 250 metafields) and drops the condition-photos key. The Rust test asserts
  each value and its provenance plus the single read op. The UI test proves no field renders while the read is pending,
  then each field and the type picker carry "from Shopify". Both fail on a wrong outcome.
- **C1(b)** Human, AI and cleared (non-live) rows are skipped by `apply_remote`, so a re-read leaves the human `Blue`
  while the untouched field follows the shop. A failed read returns `RemoteRead::Failed` before any write. The UI shows
  the exact sentence and saves the next edit.
- **C2(a)** `default_set` writes the default into the type's step layout (the Customize-fields data).
  `prefill_defaults` at creation fills Sofa and not Chair. `apply_remote` gives an empty shop value the type's default
  (`default`) and keeps a set one (`integration`). The test asserts all four cases with provenance.
- **C2(b)** `attribute_saved_values` ordered `saved_at DESC, rowid DESC`; the upsert moves a re-saved value to the top.
  Pick = `setAttributeValue` (human), remove deletes the canonical text, `presets_get` returns the form's default.
  The UI test asserts chip order, pick, remove, Save value and Save as default with the item type.
- **C2(c)** `fill_defaults` skips live rows and fills only fields with a default: 3 of 5 empty, the human value
  untouched, one undo step (`record(…"Fill defaults"…)`).
- **C3(a)** `onOpenAutoFocus` is prevented, `onChange` only sets local state, and a type is committed on blur, Return or
  Save & close (no-op when unchanged). The UI test (`change` to a listed type saves nothing; blur saves it) fails on the
  pre-P67 `onChange → commitType`. The Rust test shows that opening an untyped item reads nothing and stores nothing.
- **C4(a)** `position?.step` seeds the step state and `BatchQueue` keeps the last position in a ref. The UI test drives
  form 1 → Product → Confirm & next → form 2 on Product, then a single form on Details. The batch read is called once
  with all ids.
- **Assumptions A1–A8.** None contradicts the book. A1 (`integration`, not `shopify`) follows H7. A2's "a shop value
  replaces a default" is what changelog 46 (4) "defaults fill … linked items whose Shopify value is empty" implies, and
  C2(a) states it. A3–A8 are smallest-option choices and are reported in the narrative.
- **Hard constraints.** H1: reads only, no mutation op on open (asserted). H2: no file touched. H3/H5: not involved.
  H4: the read and fill are re-runnable to the same state (upserts; `same_content` skips identical rows). H6: an unlinked
  item returns `not_linked`, and a linked item without an adapter shows the failure sentence with the form usable. H7:
  provenance `integration`; the new tables carry no platform field (see note on the `"shopify"` link-key literal, a known
  family). H8: commands only call `AttributesService`.

## Findings
None blocking (no B1–B4).

## Notes written to backlog
1. Batch scroll memory can be lost when `BatchQueue` re-renders before the restore effect. The new form's mount-time
   `{scroll: 0}` replaces the remembered scroll. Step memory is unaffected.
2. Save as default on an untyped item writes the shared `""` layout, the fallback for every type without its own layout,
   while the toast says "for items without a type".
3. The first Save as default for a type without its own layout copies the shared layout into that type (P59 semantics),
   so later shared-layout edits stop reaching it. Book 08 should say so.
4. H7 family (P13/P14/P21): `LINK_INTEGRATION = "shopify"` literal in core `AttributesService`.
5. A5's cause of the automatic type was deduced, not reproduced in the real WebKit window. The owner's try-out is the
   first real check.

## Lessons for the plans
- "Scroll section" in a goal without a criterion row leaves it unverified. If it matters, give it a row, or drop it from
  the goal.
- "Per item type" defaults meet the P59 shared-layout fallback (`""`). Future plans touching presets should state which
  of the two a default lands in.

## Write perimeter (this session)
- `docs/implementation/master_plan.md` (P67 tracker row only → APPROVED)
- `docs/implementation/plans/P67_form_values_defaults_and_batch_step.md` (Review log entry appended)
- `docs/implementation/backlog.md` (5 notes appended)
- `docs/implementation/narratives/P67_form_values_defaults_and_batch_step.md` (new)
- `docs/implementation/handoffs/reviewer/handoff_P67_form_values_defaults_and_batch_step_review_r1.md` (this file)

No production or test file was edited, not even temporarily. No database or other state was changed outside the test
runs' own temporary stores. Nothing committed (the runner commits).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
