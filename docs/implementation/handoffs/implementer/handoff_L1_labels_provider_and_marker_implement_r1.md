---
plan: L1_labels_provider_and_marker
role: implement
round: 1
state: DONE
date: 2026-11-23
actor: claude-opus-5.5 (pipeline-implementer)
---
# L1 — The labels provider and the label marker · implement round 1

## What was built
**For the photographer.** Every row of the Sessions list (Open, Published, All) and the session page's title now
carry a small label marker read from Pricing: "Label printed · 4 Nov" (with the year when it is not this year),
"Missing price tag", "No price yet", or "Price changed / SKU changed / Barcode changed: new label needed". The rows
appear first; the markers arrive a moment later and never block the list. When Pricing prints or resets a label, the
regular 10-second Pricing check picks it up silently (no sound, no banner) and the markers follow. Settings → Pricing
shows a Labels line under the connection ("Labels: allowed", "…can't read labels…", "…couldn't check…"). A key
without label permission keeps its conversations working; it is asked once and remembered. The ⋯ menu shows a
greyed-out "Generate label" only where it explains why it is off ("Publish first so Pricing can see it", "Connect
Pricing in Settings → Pricing", the connection's own words, or the missing-permission words); the working entry
arrives with L3.

**In code.** U1: labels vocabulary (`psm_core::domain::label`) and port (`ports::labels::LabelsProvider`), the binary
transport (`PricingHttp::fetch`, `RawReply`), the Beyo Pricing adapter's `products`, `status`, `templates`,
`describe`, `product_id_of`, `commerce_id_of`, with one error function for label calls (never `ProviderError`, ACC.5).
U2: `plan`, `preview`, `run`, `download` on the adapter (`run` has no caller outside the adapter's tests). U3: the
display copy (`mechanisms::labels::LabelCopy`, `chunks`), migration `022_labels_access.sql` (the only stored fact:
`conversation_sync.labels_access`), `EventKind::Label`, the conversations service's `LabelsLink` hooks (connect →
`connected()` after the lane; `key_set` → `key_changed()` first; the pull hands label changes over on every way out),
`LabelsService` (`access`, `session_labels`), `AppEvent::LabelsChanged`, wiring (`App.labels`), `FakePricing` label
state. U4: `labels_status` command, the settings answer's `labels`, bindings, `useSessionLabels`, `LabelPill`,
`LabelMenuItem`, the Settings line, guide updates (02, 03, 04).

## Tests → criterion rows
One test per row, named after it; every test discharges a row (no orphan tests). Shape: the assertion is the row's
exact outcome unless noted.
- **U1** (`crates/integrations/src/conversations/beyo_pricing/labels.rs`, inline): L1·C1(a)–(g) `l1_c1a_…`–`l1_c1g_…`;
  C2(a)–(e) `l1_c2a_…`–`l1_c2e_…`; C3(a),(b); C4(a)–(i); C5(a)–(i); C6(a)–(c). 35 rows, 35 tests.
- **U2** (same file): C7(a)–(f), C8(a)–(e), C9(a)–(g), C10(a)–(e). 23 rows, 23 tests. C7(b) asserts the named fields
  of each item (not the whole struct, as the row lists fields).
- **U3**: C11(a)–(d) inline in `labels.rs` (adapter `events_after` over the recording transport);
  C12(a)–(i) `crates/testkit/tests/suite/l1_labels_access.rs`; C13(a)–(f), C14(a)–(i)
  `crates/testkit/tests/suite/l1_labels_markers.rs`. 28 rows, 28 tests. Core mechanism unit tests (plan's list):
  `mechanisms::labels` 8 tests (`chunks` ×3, answer-vs-event ×2, reset ×2, `fresh`). C12(i) runs both cases in one
  test. C13(c) moves the clock 9 s then 2 s (11 s total).
- **U4**: C15(a)–(e) desktop `l1_tests` (`apps/desktop/src-tauri/src/lib.rs`); C16(a)–(d)
  `features/settings/__tests__/l1_labels_line.test.tsx`; C17(a)–(d) `features/labels/__tests__/l1_marker_words.test.ts`
  (also the `markerText` core-mechanism tests); C18(a)–(f) `features/labels/__tests__/l1_markers.test.tsx`;
  C19(a)–(d) `features/labels/__tests__/l1_label_menu.test.tsx`. 23 rows, 23 tests. C15(c) asserts `code` and
  `message` (the row's two fields). C18(f) checks the pill is inside the header's title area, not its exact position
  (the position is a delegated choice).

Total: 109 rows → 109 tests, plus 8 core-mechanism unit tests.

## Assumptions
1. Recording transport shared through `#[cfg(test)] pub(super) mod testing` in `adapter.rs`; `label_error` lives in
   `labels.rs` (delegated choices).
2. A 409, 422 or other non-mapped status **without** an error envelope is `BadReply` (`Rejected` needs a code and a
   message).
3. The download path check (MC-L2.12) also refuses a backslash and percent-encoded dot segments (`%2e`), which URL
   parsers read as `/` and `..`. It only refuses more.
4. `RunItemOut.outcome` is read tolerantly (`rendered` iff the word is `rendered`), as D4 does for `lifecycleResult`;
   the plan named only `labelStatus`, the plan `outcome` and the run `status` as strict.
5. `LabelCopy::clear()` also ignores answers to requests started before it (Test connection may switch stores while
   a status read is in flight).
6. `LabelsFailure` is defined in `services/labels.rs` (beside the service, like `ConnectError`).
7. `key_changed()` emits no event, and a learned `allowed` (ACC.4) emits none (the contracts name none; L1·C12(b)
   needs exactly one `LabelsChanged` from `key_set`).
8. `useSessionLabels` guards its listener registration and its call, so the existing UI tests needed no
   `onLabelsChanged` in their events mocks and no labels mock; of D7's three allowed edits only `labels: 'off'` in two
   typed fixtures (`a28_test_feedback`, `c1_pricing_settings`) was needed (typecheck).
9. An answer is applied only while the page still shows the same sessions; the pills are cleared when another page
   is shown; a page with no rows asks nothing.
10. The ⋯ hint is a second line under "Generate label"; printed pills are neutral, the others use the `--warn`
    token; the header pill sits after the title's other pills; one hook serves the page and the header.
11. `FakePricing` logs label events in its first store; `label_printed` / `label_reset` create the label product when
    missing; `event_at_ms(id)` gives an event's time (D11).

## Deviations
- `crates/core/src/domain/label.rs` changed in U2's checkpoint (`PlanItem.product_id`, `RunItem.product_id` →
  `String`: the schema makes both required strings); the file is U1's.
- `crates/testkit/src/test_app.rs` and `apps/desktop/src/services/conversations.ts` (listed as edited) needed no change.
- `ResetReasonDto` carries `#[allow(clippy::enum_variant_names)]` (the wire words all end in `_changed`).
- `p01_storage_identity.rs` (not in the plan's Files) gained the new column in its full schema list: the closing
  suite showed it is the second tripwire of an `ADD COLUMN` migration besides p27 (guide row updated).
- The closing `cargo test --workspace` outlived the harness's 10-minute foreground limit and was moved to the
  background by the harness; no other cargo command ran until it finished.

## Files changed
Write perimeter (all in the four checkpoints, plus this handoff and the plan file's Tracker line and Review log):
- core: `crates/core/src/domain/{label.rs (new), mod.rs, conversation.rs}`, `crates/core/src/ports/{labels.rs (new),
  mod.rs}`, `crates/core/src/mechanisms/{labels.rs (new), mod.rs}`, `crates/core/src/services/{labels.rs (new), mod.rs,
  conversations.rs}`, `crates/core/src/events.rs`
- infra: `crates/infra/src/sqlite/{labels.rs (new), mod.rs, store.rs}`, `crates/infra/src/sqlite/migrations/022_labels_access.sql (new)`
- integrations: `crates/integrations/src/{lib.rs, labels.rs (new)}`, `crates/integrations/src/conversations/mod.rs`,
  `crates/integrations/src/conversations/beyo_pricing/{mod.rs, adapter.rs, transport.rs, labels.rs (new),
  label_types.rs (new)}`, fixtures `labels_products.json`, `labels_status.json`, `labels_templates.json`,
  `labels_plan.json`, `labels_run.json`, `events_labels.json` (all new)
- testkit: `crates/testkit/src/{fake_pricing.rs, fake_commerce.rs}`, `crates/testkit/tests/suite/{main.rs,
  l1_labels_access.rs (new), l1_labels_markers.rs (new), c1_conversations_connection.rs (c1_c9a's request list only),
  p27_retire_shape_on_arrival.rs (the DROP COLUMN line)}`
- desktop: `apps/desktop/src-tauri/src/{lib.rs, commands/mod.rs, commands/conversations.rs, commands/labels.rs (new)}`,
  `apps/desktop/src/services/{bindings.ts (generated), events.ts, labels.ts (new)}`,
  `apps/desktop/src/features/labels/{labelWords.ts, LabelPill.tsx, useSessionLabels.ts, LabelMenuItem.tsx}` and its
  `__tests__/{l1_marker_words.test.ts, l1_markers.test.tsx, l1_label_menu.test.tsx}` (all new),
  `apps/desktop/src/features/sessions/{SessionsPage.tsx, SessionRow.tsx, DeleteSession.tsx}`,
  `apps/desktop/src/features/shell/Header.tsx`, `apps/desktop/src/features/settings/PricingSection.tsx`,
  `apps/desktop/src/features/settings/__tests__/{l1_labels_line.test.tsx (new), a28_test_feedback.test.tsx,
  c1_pricing_settings.test.tsx}` (fixtures gain `labels: 'off'` only), `apps/desktop/src/styles/globals.css`
- guide: `docs/guide/{02_architecture.md, 03_rules_and_principles.md, 04_change_impact.md}`
- plan: `docs/implementation/plans/L1_labels_provider_and_marker.md` (Tracker line, Review log)
- Closing-gate fix outside the plan's file list: `crates/testkit/tests/suite/p01_storage_identity.rs` (`conversation_sync`'s
  column list gains `labels_access`; no assertion changed otherwise) and the guide row in `04_change_impact.md`.
- No mutation-probe files (no mutation regime, flow.md §4). No architecture graph in this repo's flow.

## Checkpoint SHA
U1 `28395bc` · U2 `793950f` · U3 `499b734` · U4 `8870b3f` · close `06318b5` (p01 schema list, guide row, tracker,
Review log). This handoff file is left for the runner to commit (flow.md §8).

## Stamp
Run once at the end, one command after the other, on the tree of `8870b3f` plus the p01 fix (in the close commit):
- `cargo test --workspace --no-run` (build first), then `cargo test --workspace`: 13 test binaries ran; all green
  except **1 failure** in the suite: `p01_storage_identity::c1a_open_creates_all_domain_columns` (its full column list
  lacked the new `conversation_sync.labels_access`). Suite: 1358 passed, 1 failed. Fixed (column added to that list,
  plus the guide's ADD COLUMN row), then re-ran only the failing file: `cargo test -p psm-testkit --test suite
  p01_storage_identity::` → 20 passed; the doc-tests skipped by the failure: `cargo test --workspace --doc` → green.
  Failure-ID delta against a clean baseline: none left (no pre-existing failures observed).
  Note: the harness moved the workspace run to the background after 10 minutes; no other cargo ran meanwhile.
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- `cargo fmt --all -- --check` → clean.
- `cd apps/desktop && npm run typecheck` → 0 errors.
- `npm test` → 129 files, 867 tests passed.
- Per-unit commands (during work): U1 adapter 68 tests; U2 adapter 91; U3 adapter 95, `mechanisms::labels` 8,
  `l1_labels_access::` 9, `l1_labels_markers::` 15, `c1_conversations_sync::` 44, `c1_conversations_connection::` 19,
  `p27_retire_shape_on_arrival::` 11 (also c2/c3/c4 conversations 23/31/6); U4 `cargo test -p psm-desktop` 97, the
  nine UI folders 104 files / 704 tests, typecheck, clippy — all green.

## Manual try-out
On the installed app, with a Pricing key that has `labels:read` (Settings → Pricing → Replace):
1. Settings → Pricing → Test connection: under the connection line it says "Labels: allowed". With a key without the
   label permission it says "Labels: this key can't read labels…", and conversations keep working.
2. Open Sessions. Rows of published products show a small marker: "Label printed · <date>", "Missing price tag" or
   "No price yet". The list appears first; the markers a moment later.
3. In Pricing, print a label for a product showing "Missing price tag": within about ten seconds its marker reads
   "Label printed · today", with no sound.
4. In Shopify, change the price of a product whose label is printed: within about ten seconds its marker reads
   "Price changed: new label needed".
5. Open a published session: the same marker stands beside its title.
6. Open the ⋯ menu of a session that was never published: "Generate label" is greyed out with "Publish first so
   Pricing can see it".

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
