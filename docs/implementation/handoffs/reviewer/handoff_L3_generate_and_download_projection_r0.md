---
plan: L3_generate_and_download
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-11-23
actor: claude-opus-5.5 (pipeline-projectionist)
---
# Projection of L3_generate_and_download (round 0)

The plan is buildable and its dangerous parts hold up against the real code and against Pricing's own package. Those parts are the one writing call, a single run key reused on every retry, a lost answer retried without marking anything twice, the download address rule, and the Labels folder. Seven small gaps would make an implementer guess, and three of them would leave a test that cannot be written with one exact answer. Each one closes with a sentence or two in the plan or the labels master plan. Nothing needs the owner. Once the coordinator folds the amendments and records the delegations below, the implementer prompt can be compiled.

## ⚠ OWNER DECISIONS REQUIRED (0)

Nothing needs the owner: every point below is a plan-level amendment or an explicit delegation.

## Gate check

- Intention `docs/intention/intention.md`: **RATIFIED** (2026-10-24); chapter `09C_pricing_labels.md`: **RATIFIED** (2026-11-23, A33).
- `depends_on` L2_labels_box: Tracker **APPROVED** (2026-11-23). L1 APPROVED. No upstream handoff for L3 in `OWNER_DECISIONS_PENDING`; labels card 1 answered (master plan §2A).
- Counts were derived from the plan file and match its own table: U1 4 criteria / 27 rows (C1 a–i, C2 a–e, C3 a–g, C4 a–f), U2 5 / 24, U3 2 / 24 (C10 a–g, C11 a–q), U4 6 / 32. Total 17 criteria, 107 rows; every unit has ≤ 8 criteria (flow §10).

## Decision ledger

| # | Decision point (where the artifacts stop determining the next step) | Class | Proposed routing |
|---|---|---|---|
| D1 | **L3·C4(c) "no temporary PDF is left"** is undecidable when the delegated choice picks "the system's temporary folder". The test cannot know the file's name in a shared `$TMPDIR`, and setting `TMPDIR` in a parallel test is racy. | **plan gap** (decidability) | Amend U1: remove that delegated choice. `raster` writes its temporary PDF inside `<previews>/labels/` (e.g. `<name>.src.pdf`) and always removes it. L3·C4(c)'s outcome becomes "after both calls `<previews>/labels/` holds exactly `x.png`". |
| D2 | **How a `LabelFiles::save` error becomes `SaveFailed(reason)`.** L3·C10(d) and L3·C11(i) pin `"disk full"` and `Couldn't save the PDF: disk full.` exactly. The port answers `psm_core::Error`, and no variant displays a bare text: `Io` displays `I/O: …` and `Provider` displays `provider: …` (`crates/core/src/error.rs`). An implementer who writes `error.to_string()` gets `Couldn't save the PDF: I/O: disk full.` | **plan gap** | Amend U1 task 3 / U3 task 1: `FakeLabelFiles::fail_save_next(reason)` answers `Error::Io(io::Error::other(reason))`. The service's reason is the inner `io::Error`'s text for `Error::Io` and the error's `Display` otherwise. State the same rule for `PreviewFailed`, which no row pins. |
| D3 | **Where a failure's `sku` / `title` come from** (MC-L2.9 is silent). L3·C9(a) asserts `sku: P2's, title: P2's`, and L3·C15(b) shows `TEST-3 · Test lamp`. Rule 17: `RunItemOut.title` is *required but nullable* (`openapi-integration.json`: `anyOf [string, null]`), and the `API.md` runs example sends no title at all. Reading it from the run item passes against the fake and shows an empty name against the real Pricing. | **plan gap** (rule 17, home = master plan §5 MC-L2.9) | Amend MC-L2.9: a failure's `sku` and `title` are the **check item's** (always kept with the check), falling back to the run item's or `Unresolved`'s own. Add to U2 task 3: the fake's run items carry `sku` and `title` (schema fields). |
| D4 | **`merge_plans` position renumbering** when an earlier chunk had `unresolved` entries or answered `NoProducts`. Is it "chunk start + Pricing's position" or "index among the merged items"? `API.md` does not say whether Pricing's `position` skips unresolved entries. The unit test of U1 task 2 needs one exact answer. | **plan gap** (home = MC-L2.1) | Amend MC-L2.1 and U1 task 2: `position` := the item's index among the merged items (0, 1, 2, … in merged order), independent of Pricing's numbering. Add one unit-test case where chunk 1 has an unresolved entry. |
| D5 | **The copy after a `Reprinted` item**: "Printed with the time it had (the run's when unknown)". Is "it had" the copy's entry or the check item's `printed_at`? The check does not write to the copy, so for a product never shown in a list the two differ. L3·C5(c) asserts only `Printed`, so either reading passes. | **plan gap** (home = MC-L2.8) | Amend MC-L2.8: `printed_at` = the check item's `printed_at`, else the run's `created_at`. L3·C5(c) adds "with `printed_at` T0". |
| D6 | **The ⋯ entry with access `unknown`.** ACC.1 allows calls when A ≠ denied, so the entry should be enabled. The replaced row L1·C19(c) covered *both* `allowed` and `unknown`, but its replacement L3·C17(a) states only `allowed`. The new expectation for `unknown` is left open, and `LabelMenuItem` today hides the item for both. | **plan gap** | Amend L3·C17(a): input "access `allowed`; access `unknown`", both with an enabled `Generate label`. Say in Files that L1·C19(c) is replaced for both states. |
| D7 | **L3·C12(e) "Printed 4 Nov."**: the date rule is unstated. `labelWords.ts`'s `day()` adds the year when it is not `now`'s year (MC-L3.5). A fixture hard-coded to 2026-11-04 turns red on 1 Jan 2027 if that helper is reused. | **plan gap** (decidability, time bomb) | Amend U4 task 1 and the row: the date is the marker's day rule (local, `d MMM`, the year when not this year). The fixture is 4 Nov of the current year, or the test fixes `now`. |
| D8 | **The fake's template resolution and sizes.** U1 task 5 says "its template by product type else the default". L3·C2(d) ("P2 on template B (no product types)", "none assigned") and L3·C2(b) need an *assignment* (`LabelSpec.template` already exists). `sizes` must count only ready items, as L3·C11(a)'s `count: 2` and `API.md` ("one PDF per size among the ready items") both require, and `page_count` = the rendered items of that size. | **plan gap** (our fake; package-grounded) | Amend U1 task 5: the template is `LabelSpec.template` (an assigned id), else the template whose `product_types` holds the product's type, else the default, else none. `sizes` covers items whose outcome is not `fail`. Amend U2 task 3: an artifact's `page_count` = the rendered items of its size; the run answers `201`. |
| D9 | The fake's read accessors the rows assume but no task lists: the label state of a product (L3·C1(g) "as before", L3·C5(b), L3·C7(d)), the label-event log or its count (L3·C5(c), L3·C7(a)), and `runs()`'s record shape, which must include `created_at` for L3·C5(b) "at the run's time". | free choice | Delegate in writing: the implementer adds these to `FakePricing` (each has a row caller, charter rule 4), and `runs()` returns key, item ids and `created_at`. |
| D10 | A `check` that fails while an earlier check is kept: is the slot emptied or does the old check stay? MC-L2.1 says "nothing kept", but no row covers this case. | free choice | Recommend recording: a failed check empties the slot (and clears the previews), the literal reading of "nothing kept". |
| D11 | "One product" vs "batch" wording in both dialogs (L3·C12(e), L3·C15(c)): decided by the product ids the dialog was opened with, or by the check's item count? A batch of 2 with one unresolved has 1 item. | free choice | Recommend: decided by the number of product ids the dialog was opened with (as `LABELS.md` §3.3 frames it by entry point). |
| D12 | `FakeLabelFiles::clear_previews` is not specified, but L3·C3(f) needs the first check's PNG to be gone. | free choice (forced by the row) | Record: it delegates to its inner `DiskLabelFiles::clear_previews`. |
| D13 | `label_runs.rs` is a sibling module (`services/label_runs.rs`), so `impl LabelsService` there cannot read `labels.rs`'s private fields (`copy`, `provider`, `events`, …). | free choice | Record: fields become `pub(super)`, or small `pub(super)` accessors. No behaviour change. |
| D14 | In `check`, the order of the templates read against the plan reads, and whether the templates read is made when a plan read fails. Only counts are asserted (L3·C1(a)). | free choice | Record as delegated. |
| D15 | Step order of ‹ › (L3·C13(b)): plan order or group display order. They differ when templates interleave. | free choice | Recommend the group display order (what the photographer sees). |
| D16 | **Close** after a `labels_denied` Generate (L3·C14(e)): does it call `labelsPartial` like **Cancel**? When chunk 1 already finished, skipping it loses the done view and the selection rule. | free choice | Recommend: Close behaves as Cancel (calls `labelsPartial`; `not_found` → closes). |
| D17 | `labelsCheck` rejecting with `labels_off` (empty message by design) leaves the error state with no words. | free choice | Recommend: 09B's connection words / `Open Settings → Pricing`, as the ⋯ hint does. |

## Reality checks

- **Paths.** Every "Edited" path in U1–U4 exists. Every "New" path is absent. `docs/implementation/backlog_labels/L3_generate_and_download.md` does not exist yet; the reviewer creates it, as for L1 and L2.
- **Symbols the plan builds on.** All verified: `LabelsService` and `LabelsFailure` (all nine variants already present, with the error-code mapping already complete in `commands/labels.rs::labels_error`, so U3 task 2's "the remaining codes" is registration only); the `*_body` command pattern; `PricingHttp::fetch`; `RawReply`; the adapter's `plan` / `preview` / `run` / `download` with `RUN_TIMEOUT` / `FILE_TIMEOUT`, the `%PDF-` check and `download_path_ok`; `IntegrationLabelRun` / `RunItemOut` / `RunArtifactOut`; `IntegrationPreviewRequest` skipping `None` fields (so L3·C3(a)'s body `{"productId": "1"}` holds); `FakePricing::{fail_matching, fail_matching_after, fail_matching_with, timeout_next, drop_answer_next, hold, release, wait_until_held, remove_store}`; `FakeCommerce::mutation_ops` and its `state.reads`; `TestApp::{with_pricing, kill_and_restart}` with `paths.home` = the temporary folder (`test_app.rs:502`; `restart` reuses `paths`, so a seam keyed by the data folder survives); `with_test_conversations` (`conversations/mod.rs:155`); `App::build` → `labels::build_service` (`crates/integrations/src/lib.rs:1250`); `AppPaths` (`paths.rs:10`, `:17`, `:31`); `useThrottled` (the box's initial load does not go through it, so L3·C17(e)'s "exactly one refresh" fires at once); `SessionActionsMenu` in `DeleteSession.tsx`; `previewUrl` (`sessions.ts:106`); `l1_c19c` and `l2_c13a`.
- **Line drift (nit, the symbols beside them resolve):** `fake_pricing.rs` scripted failures, `timeout_next`, `drop_answer_next` and `hold` are now at :787–873, not :480–552. `asset_protocol_scope` is at desktop `lib.rs:3127`, not ~:2639. `blocking` is at `commands/conversations.rs:106`, not :102. The outbox `flush` is at `conversations.rs:1149` (inside the cited range).
- **Today's fake `fetch`** records the request and answers 404 without honouring `hold` / `timeouts` / `drops` / `scripted`. U1 task 5's sentence ("act on `fetch` as on `send`") is what L3·C7(a)–(e), L3·C8(a) and L3·C11(p) depend on. It is correct as written; flagged so it is not skipped.

## Criteria decidability (per row; both questions)

**Can the test be written now with one exact outcome?** Yes for every row except:
- L3·C4(c), the temporary-file clause (D1).
- L3·C10(d) and L3·C11(i), the exact reason text (D2).
- L3·C12(e), the date (D7).
- L3·C17(a), the `unknown` state (D6).
- The `merge_plans` unit test of U1 task 2 (D4).
- L3·C5(c), which is decidable but weaker than its contract (D5).

**Nits that do not block:**
- L3·C7(a) "byte-identical bodies": `FakeRequest.body` is `Option<Json>`, so this is decided as JSON equality (or `to_string()` equality; serde_json's map order is deterministic). Reword.
- L3·C3(d) does not name its check or which product gets the 503. Name them (e.g. `check(["1", "3"])`, P3 without a template, `fail_matching_with("/labels/preview", …, 503, "renderer_unavailable", …)` for P1).
- L3·C8(c) reuses L1·C13(f)'s instrument, which only searches table and column names containing `label`. A stored `check_key` column would pass it. Have the row assert the schema's table/column list is unchanged by L3.
- L3·C11(a) and L3·C11(c): `width_mm` / `height_mm` are `f64` and serialise as `70.0`, and `serde_json`'s `70.0 ≠ 70`. The test writes `70.0`.

**Is the fixture producible (rule 17)?** Every externally shaped row was traced to the package (`API.md` › plan / preview / runs / artifacts, `LABELS.md` §3.3–§3.5, `openapi-integration.json`) and to the adapter's mapping (`beyo_pricing/labels.rs::label_error`, `pdf`, `run`):
- A lost answer becomes `Error::Timeout` → `Unreachable` → retried with the same key. L3·C7(a) is producible.
- 500 → `Unreachable`; 503 `renderer_unavailable` → `RendererUnavailable`; 403 `insufficient_scope` → `NotAllowed`; 422 `no_products` → `NoProducts`; other 422 → `Rejected` with its message; 2xx of the wrong shape → `BadReply` (a scripted 200/201 envelope body does this); 404 on plan → `StoreGone`; 410 → `Expired`.
- Runs answer 201. Unresolved entries are outside `counts`. `downloadUrl` is a path. Preview 422 `no_template` → `Rejected`. `sizes` count ready items only.
- The one row whose fixture the real dependency may not produce is **L3·C9(a)'s failure `title`** (D3).
- The `sips` row (L3·C4(c)) is grounded by the planner's probe (master plan §3). Its open point is the temporary file (D1), not `sips`.

## Trace verification (both directions)

- **Rows → ledger.** Every row's trace cell resolves and supports the row: MC-L2.1–.11, .13, .14, ACC.1, ACC.4, MC-L3.4, MC-L1.9, M-L1–M-L6, 09C §6, §10, §12.
- **Ledger → rows.** M-L2 (C1(g), C3(g), C5(d–e), C12(f)); M-L3 (C6(a), C7(a, c, e), C8(a), C11(p), C14(a–b)); M-L4 (C9(a–e), C15(b, e)); M-L5 (C1(g), C10(g)); M-L6 (C1(h), C11(q)).
- **MC-L2.12** (the download address and the `%PDF-` check) has no L3 row. It is served by L1's adapter rows (`download_path_ok`, `l1_c8b`), and U3's hazard note keeps the service from rewriting an address. Acceptable; recorded so the reviewer does not raise it as unserved.

## Write perimeter

This file only. No plan, intention, code, prompt, tracker or archgraph change. No commit.

## Appendix (non-authoritative, discard)

The skeleton used to walk the plan was kept only in this session. No sketch is offered to the implementer.
