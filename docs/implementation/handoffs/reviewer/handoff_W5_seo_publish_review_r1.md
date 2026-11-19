---
plan: W5_seo_publish
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-19
actor: opus (pipeline-reviewer)
---
# W5_seo_publish — review r1

**Gate.** Tracker line read `IMPLEMENTED`; checkpoints `3e0a878` (U1), `5586490` (U2), `2150e14` (U3), `bbee5e1` (U4),
`3e7ef83` (closing) all in `git log`. Passed.

## Verified correct
**Perimeter.** `git show --stat` of each checkpoint equals the handoff's Files changed: U1 9 paths, U2 11, U3 5, U4 14,
closing 2. No extra or missing path. The out-of-unit test edits (`p10::c6b`, engine `w2_c4e`/`w2_c19a`/`w2_c20b`,
`p13::c9b`) are disclosed and follow the plan's U1 hazard (updated to the new built-in list, not weakened).

**Checks (Review mode, run once, one cargo command at a time, tree `b0c18da` = code `3e7ef83`).**
| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test suite -- research::w5_seo_publish:: research::w5_carried:: p12_shopify_adapter:: p14_publish_update:: p10_attributes::` | 147 passed, 0 failed (10.5 s) |
| `cargo test -p psm-integrations` | 40 passed, 0 failed |
| `cargo clippy -p psm-core -p psm-integrations -p psm-testkit -p psm-research --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `npx vitest run src/features/form/__tests__/w5_carried.test.tsx` | 2/2 |
The implementer's full-suite stamp (workspace 1033; first run's one red `p13::c9b` pinned the old built-in count,
fixed and re-run, as flow §4A allows) stands.

**Rows (production read, then test; each test observes the row's outcome at its boundary and fails on a wrong one).**
- **C1(a–c)** `append_always_present` (pure, unit-tested) applied in `layout_keys`, `form_get`, `presets_get`; tests:
  exact ordered Product step from a pre-W5 stored preset; `layout_keys` contains both; a placed layout round-trips
  with nothing appended twice.
- **C2(a–b)** the remote read inserts `seo_title`/`seo_description` into `incoming` exactly as the description;
  `seo_of` maps null/blank/missing → none. Tests: exact `integration` row, no description row for `""`; the human
  row equal before/after.
- **C3(a–c)** wired write run: request `texts` contain both; rows `ai`+`suggested`+`medium` with the exact texts; a
  61-char title twice → `low` with `Check: … 61 characters (at most 60)`; the integration value becomes `ai_prior`.
- **C4(a–b)** `product_seo` over the captured `seo_read_unset` → `Seo{None,None}`, and the declared synthetic
  `ProductValues` edit → `seo_*` None; `seo_read_after_update` → the exact 40/140-char strings of `seo_update`'s
  request.
- **C5(a–b)** `update_seo` through `mutate(run, "update_seo", PRODUCT_UPDATE_SEO, …)`; test pins the document, the
  exact `product` variables (`"description": null`) and the `shopify mutation` log line with `run_id` and `op`;
  `userErrors` → `Error::Provider("shopify update_seo: Title is too long")` (synthetic edit declared; Shopify did not
  refuse).
- **C6–C9** `PublishService::update_seo` (after the description): returns before any read when neither frozen key is
  non-blank; reads `product_seo`; desired = frozen non-blank per key else the remote value; compares trimmed; one
  `update_seo` with both fields carrying the confirmed run; reads back and fails `AttributeVerification` naming only
  the changed keys that differ. Tests: C6(a) one write {frozen title, remote description}, reads at write-count
  [0, 1], `done`; C6(b) no write when equal after trimming; C6(c) zero reads; C7(a) title sent as `None`; C8(a)
  `failed` at `update`, error names `product.seo_title` and not the description; C9(a) killed after the shop applied
  the write → after restart still one write, `done`, shop holds the frozen values; C9(b) `n_withheld` 1, origin
  `withheld`, no write; C9(c) conflict on the description key, `held`, no write until accept, then `done` with the
  suggestion sent.
- **C10–C14** `model_tokens` excludes single A/I and tokens followed by a unit (`unit_follows`); `filter` rule 2
  drops `missing_in_text`/`text_vs_facts` for a text not in this run's written/unchanged keys; `stale()` requires
  `condition_chosen`; `WriterBlock` toasts `reason.sentence` when not queued; `money` "< $0.01" for 0 < µUSD < 10 000.
  Tests assert each exact outcome (C11(a)/C14(a) also shown red by the implementer with the fix reverted).

**Reading checks the plan asked for.** Fixtures carry `"source": "captured: W5·U2 probe step …, sanitized"`; no
token, secret or shop domain in them. Evidence `shopify_admin.md` §SEO search texts records API 2026-10, the create
document (`PRODUCT_SET`), null/null when unset, the byte-identical read-back and the 75-character title stored whole
(no `userErrors`). No SEO mutation without a confirmed run (`update_seo(&ConfirmedRun, …)`, called only from
`update`). A blank frozen value never reaches `update_seo` as a change (filtered to none, then the remote value is
used). `crates/research/src/writer` holds no `"shopify"` literal. The probe's `seo` step is not in `all` and never
calls `cleanup`.

**Assumptions.** None contradicts 08C §9 or WC8/WC4/WC5/WC6/WC1; each is in the narrative's "Choices made for you".

**Hard constraints.** H1 confirmed run on the only SEO mutation; H2/H3/H5 untouched; H4 read-before-write resume
(C9(a)); H6 no publish path change for an unconfigured shop (step runs only inside publish); H7 `Seo`/`product_seo`
are platform-neutral names in core, Shopify shapes stay in the adapter (the pre-existing core `"shopify"` link
constant is a note, not introduced here); H8 no Tauri dependency added. Guide updated (`02_architecture`,
`04_change_impact`, `05_working`).

## Findings
None blocking (no B1–B4).

## Notes written to backlog
`docs/implementation/backlog_writer/W5_seo_publish.md` (the plan's named notes file):
1. A search title over Shopify's suggested length is published as written (Shopify neither cuts nor refuses; only the
   writer's own 60-char check flags it) → owner, told in the narrative.
2. `LINK_INTEGRATION = "shopify"` is now `pub` in core and used by research (pre-existing H7 slip, wider now) → tidy.
3. A stale doc comment stacked over `DIMENSION_LETTERS` in `postcheck.rs`.
4. The Answer toast shows only when the reply carries a reason (guard only; today a reason always comes).
Plus a line listing the W3/W4 notes this phase resolved, for the orchestrator to mark.

## Files written by this review
- `docs/implementation/plans/W5_seo_publish.md` (Tracker line → APPROVED; Review log entry)
- `docs/implementation/backlog_writer/W5_seo_publish.md` (new)
- `docs/implementation/narratives/W5_seo_publish.md` (new)
- `docs/implementation/handoffs/reviewer/handoff_W5_seo_publish_review_r1.md` (this file)
No production or test file edited; no probe; no commit.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
