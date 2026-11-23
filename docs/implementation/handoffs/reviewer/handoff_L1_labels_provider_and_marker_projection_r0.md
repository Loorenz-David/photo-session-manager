---
plan: L1_labels_provider_and_marker
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-11-23
actor: Claude Opus 5.5 (pipeline-projectionist, plan-projection doctrine; orchestrated)
---
# Projection of L1_labels_provider_and_marker (round 0)

The plan is mostly buildable as written: the Pricing data shapes match the package almost everywhere, the stated counts
are right, and almost all of the 109 test rows can be written today with one exact expected result. Eight things need a
short plan amendment before the implementer starts. One test row can never pass on the current database. Two existing
tests would go red because of this change, and the plan does not say they may be changed. In one place the Pricing
package's own example disagrees with its schema: if Pricing really sends the example's word, a successful label run
would show as an error. And the ⋯ menu's hint has no data to build its sentence from. None of this is a product
question, so nothing needs the owner. The orchestrator folds the amendments into the plan, then compiles the
implementer prompt.

## ⚠ OWNER DECISIONS REQUIRED (0)

Nothing needs the owner. Every finding is a plan amendment or a delegation the coordinator can record.

## Gate check

- Intention status header: **RATIFIED** (`docs/intention/intention.md`, 2026-10-24); chapter 09C: **RATIFIED** 2026-11-23
  (changelog A33). Pass.
- Upstream gate handoff `handoffs/reviewer/handoff_labels_plan_r1.md` is `OWNER_DECISIONS_PENDING`, but its one card (card 1,
  the Labels box number) is scoped to L2. Its on-silence line reads "L1 is built (it does not depend on this)", and L1 has
  `blocked_on: none`. Not an authority still moving under L1. Pass, recorded.
- `projection: yes` in the plan frontmatter (flow §2). Rule 17 rows present (externally derived fixtures). The gate is mandatory.

## Decision ledger

| # | Decision point (where the artifacts do not determine the next step) | Class | Proposed routing |
|---|---|---|---|
| D1 | **L1·C13(f) can never pass.** "The only column whose name contains `label` is `conversation_sync.labels_access`". But `attribute_definitions.label TEXT NOT NULL` exists (`crates/infra/src/sqlite/migrations/002_domain.sql:80`, read and written at `crates/infra/src/sqlite/store.rs:1203`, `:1270`) and no later migration drops it. | plan gap | Amend the row's outcome: "no table whose name contains `label`; the columns whose name contains `label` are exactly `attribute_definitions.label` (pre-existing: an attribute field's display name) and `conversation_sync.labels_access`, whose value is one of …". Alternative: compare the column list against a database at migration 021 and require the difference to be exactly that one column. |
| D2 | **Does ACC.4 also apply to `connected()`'s own templates read?** U3 task 5 says "the access changes of ACC.4 on every label call it makes". `connected()` makes one, so a `NotAllowed` there would emit `LabelsChanged {[], access: true}` twice (ACC.4, then ACC.2), and L1·C12(b) says exactly one. The row pins the result, but the code path is a choice the implementer would make silently. | plan gap | Amend ACC.2 (master plan §5) or U3 task 5: "`connected()` applies only ACC.2 (its own three-way assignment and exactly one `LabelsChanged`); ACC.4 governs every other label call". |
| D3 | **An existing conversations test breaks, and the plan does not authorise changing it.** `c1_c9a_test_connection_saves_the_matching_store` asserts `request_lines() == ["GET /stores"]` after a `test_connection` that ends `Ready` (`crates/testkit/tests/suite/c1_conversations_connection.rs:302`). ACC.2 adds `GET /stores/<id>/labels/templates`. Review mode says "the existing conversations tests changed only where `EventKind` gained an arm", and `c1_conversations_connection::` is not in U3's unit test commands, so the break first shows at the closing gate. | plan gap | Amend U3: in Files, add `crates/testkit/tests/suite/c1_conversations_connection.rs` (the C1·C9(a) expectation becomes `["GET /stores", "GET /stores/<id>/labels/templates"]`, the ACC.2 contract); in Unit test commands, add `cargo test -p psm-testkit --test suite c1_conversations_connection::`; in Review mode, widen "changed only where …" to name this test. I checked the other C-suite request assertions (`c1_conversations_sync.rs` `c1_c14a`/`c1_c14b`, `c3_conversations_answer.rs:375`): they clear requests after `key_set` or count only `/events`, so they are unaffected. |
| D4 | **Rule 17: `RunItemOut.lifecycleResult` for a failed item.** The package contradicts itself. The schema enum is `marked_printed \| reprinted \| no_price \| null` (`openapi-integration.json` `RunItemOut`). But `API.md:263`'s run example sends `"lifecycleResult": "none"`, and `examples/client.ts:64` types the field as a plain `string`. The master plan (§3 Rule 17) picked the schema. If the wire type is a strict enum and live Pricing sends `"none"`, a **201 run that already marked products Printed** parses as `BadReply`. In L3 that chunk is never retried, and Try again (same key) returns the same run and the same `BadReply`. The photographer is told "Pricing answered in a way this version does not understand" while the labels exist and are marked. All tests stay green. | plan gap | Amend U2: (i) parse `lifecycleResult` tolerantly: the three words map to `RunResult`, and anything else, `"none"` included, maps to `None`; (ii) add an input to L1·C9(c): "an item `outcome: "failed"`, `lifecycleResult: "none"`" → `result None`; (iii) wire types do not strictly type enum fields the domain never reads (`LabelStatusOut.source`, `IntegrationPlanTemplate.source`, `RunItemOut.templateSource`, `requestedByKind`): leave them out of the struct, or keep them as `String`. The strictness the domain does want stays: `labelStatus` (L1·C1(g)), `outcome` (L1·C7(e)), run `status`. |
| D5 | **The ⋯ hint for `off` states other than `no_key` has no data.** U4 task 6 says the hint is "`connectionLine`'s sentence". `connectionLine(connection: ConversationsConnection)` reads `shop`, `store_name` and `visible_stores` (`apps/desktop/src/features/conversations/connectionLine.ts:7-18`). The menu item receives only `access = {state, reason}`. No row covers these states (L1·C19(b) tests only `no_key`), so the implementer would improvise: a second fetch, or sentences with an empty shop name ("This key is no longer tied to "). | plan gap | Amend U4 task 6 with one source. Smallest consistent option: when the menu opens with `access.state = "off"` and `reason ≠ "no_key"`, call the existing `conversationsSettingsGet()` and use `connectionLine(settings.connection)`, falling back to `CONNECT_PRICING` while it loads or if it fails. The existing menu already fetches on open (`sessionFilesMisplaced`, `DeleteSession.tsx:41`). Optionally add a row: access `{off, store_gone}` → the hint `This key is no longer tied to <shop>`. |
| D6 | **The ⋯ entry's precedence and its state before an answer arrives.** Task 6 lists the conditions (no product id / off / denied / unknown / enabled) without saying whether the order is a first-match precedence. Example: an unpublished session with access `unknown`: "Publish first" or nothing? Also unspecified: what renders before `labelsStatus` has answered (label undefined). | plan gap | Amend task 6: "first match in this order: no answer yet → not rendered; `product_id` null → disabled, `Publish first so Pricing can see it` (in every access state); `off` → …; `denied` → …; `unknown` → not rendered; otherwise not rendered until L3". |
| D7 | **The U4 perimeter for existing UI tests is undercounted.** (a) 47 test files `vi.mock('@/services/events', () => ({…}))` with factories that do not export `onLabelsChanged` (e.g. `features/sessions/__tests__/h2_list_pages_and_pictures.test.tsx:20`, `features/shell/__tests__/h4_capture_target.test.tsx:43`, `features/conversations/__tests__/c2_header_icon.test.tsx:17`). Vitest throws on access to a missing export of a factory mock, so `useSessionLabels`'s listener crashes every one of those tests that renders `SessionsPage` or a session `Header`. The plan adds only a `@/services/labels` mock. (b) `typecheck` includes tests (`tsconfig.json` `include: ["src", …]`). The typed fixture `settings()` in `features/settings/__tests__/c1_pricing_settings.test.tsx:21` (and the same pattern in `a28_test_feedback`, `t1_settings_drawer`) builds a `ConversationsSettings` without the new `labels` field, so typecheck goes red. (c) Tests that render `Header` also live in `features/workspace`, `features/publish`, `features/conversations` and `features/pending`, outside U4's vitest command. | plan gap | Amend U4 Files: the existing tests "gain `onLabelsChanged` in their `@/services/events` mock, a `@/services/labels` mock answering access off, and `labels: 'off'` in typed settings fixtures; no assertion changes". Amend U4 Unit test commands to `npx vitest run` over every folder whose tests render `SessionsPage`, `Header` or `PricingSection` (or plain `npm test`). Alternative that shrinks the perimeter: export the listener from `@/services/labels` (already mocked) instead of `events.ts`. That amends master plan §4 Event. |
| D8 | **L1·C12(i) cannot tell which `key_set` branch resets the access.** After step 1 (`key_set(None)`), ACC.3 has already stored `unknown`, so step 2's assertion holds whether or not the `Some` branch calls `key_changed()`. Step 1's `Off(NoKey)` comes from ACC.7 whatever is stored. The row only catches "never called" (flow §5 B2 family). | plan gap | Amend the row: assert `LabelStore::labels_access() = unknown` after step 1 as well, and run step 2 from a freshly `Allowed` app (no `key_set(None)` before it). |
| D9 | **Which clock feeds `LabelCopy`'s freshness?** The registry signature takes `now: Instant`. Only `Clock::mono()` is advanced by the test clock (`crates/testkit/src/clock.rs:26`); `Instant::now()` is not. L1·C13(c) ("the clock moved 11 s") can only be written against `clock.mono()`. | plan gap (minor) | Amend U3 task 1 or 5: "`now` = `clock.mono()`". |
| D10 | **Freshness after a `None` status answer.** MC-L3.2 says a `None` answer removes the entry, but freshness is tracked per entry. A product Pricing does not know is therefore asked again at every refreshing call, against planning's own point "never more than once per 10 seconds for the same products". No row observes it. | free choice | Delegate explicitly, with a recommendation: keep an answered-at time for a `None` answer (state none, fresh for `STATUS_FRESH_MS`). |
| D11 | **What `FakePricing::label_printed` / `label_reset` return.** L1·C14(a) asserts `S.cursor` = "that event's id", and L1·C13(a)/C14(a) assert `printed_at = T`. The test must learn both, and the plan does not give these fake methods a return value. | free choice | Delegate: return the event id (as `raw_event` does, `fake_pricing.rs:451`). The event's time is then read from the fake, or returned with it. |
| D12 | **Trace cells outside the declared grammar.** Master plan §3 admits `M-L<n>`, `MC-L<n>.<item>`/`ACC.<item>`, 09C §/scenario, CR/MC-C1, 10H, H1–H8, `[rule]`. Rows L1·C1(d), C1(i), C2(b), C2(e), C4(g), C4(h), C4(i), C5(h), C7(c), C7(e), C9(c), C9(g), C10(e), C15(d) cite only registry names ("Adapter paths", "Rule 17", "Ports", "Words", "Event"). Each one does point at a contract-grade registry line, so this is a notation defect, not an orphan. | plan gap (note) | Either admit "master plan §4 <registry entry>" into the §3 trace grammar, or retrace (e.g. status mapping → 09C §10; `%PDF-`/address → MC-L2.12; nullable fields → MC-L3.1). |
| D13 | **Month words in `markerText`.** "4 Nov" is pinned by L1·C17. `toLocaleDateString` in jsdom/en-US gives "Nov 4". | free choice | Delegate: fixed English month abbreviations, local calendar day. |

## Reality checks

- **Paths.** Every existing path in the four Files lists and the Read-first lists resolves at `47887c5`. Every "New" path is
  absent. Migration number `022` is free (last is `021_publish_details_only.sql`). The four `PricingHttp` implementations
  that must gain `fetch` are exactly the ones the plan names: `ReqwestPricing` `transport.rs:129`, `NoTransport`
  `conversations/mod.rs:142`, `Recording` `adapter.rs:438`, `FakePricing` `fake_pricing.rs:772`.
- **Cited code says what the plan claims.** `send` → JSON with `from_utf8_lossy` fallback (`transport.rs:152-158`).
  `read_error`/`write_error` map 401/403 → `Refused` (`adapter.rs:95-118`). `encode` keeps only the unreserved set
  (`adapter.rs:71-82`), so L1·C1(c)'s `q=teak%20bord%20%26%20co`, `cursor=a%2Bb%2Fc%3D` follow from it. `connect` holds the
  lane `:536-560`. `pull` early returns `:786-866`. `first_connection` uses only `for_us` events (`:663-674`), and label
  events have `conversation_id` none, so `for_us` is false (`adapter.rs:243-245`). `check_failed` emits
  `ConversationsChanged` (the "as today" of L1·C14(f)). `EventKind` derives `Eq` (`conversation.rs:163`), so
  `LabelChange` must too (no float in it, fine). `adopt` writes the link with integration `"shopify"` and the shop's
  external id (`services/items.rs:211`). `FakeCommerce::add_product` ids are `product-<n>` (`fake_commerce.rs:281`), so
  `add_product_with_id` is needed, as the plan says. `ManualClock` advances `mono()` (`testkit/src/clock.rs:18-29`).
  `refuse_connection_next` gives `Error::Provider("http: connection refused")` (`fake_pricing.rs:513`).
  `request_lines()` lines are `METHOD /path?query` after `/integration/v1` (`fake_pricing.rs:43-49`), the form the U3
  rows use. The rewind test's drop list (`p27_retire_shape_on_arrival.rs:353-380`) works as the plan describes:
  `019` is `CREATE TABLE IF NOT EXISTS`, so without the new `DROP COLUMN` line, `022` would hit "duplicate column".
- **Counts.** Computed from the plan file by script: U1 6 criteria / 35 rows, U2 4/23, U3 4/28, U4 5/23; total 19 / 109.
  These match the plan's table and the master plan.
- **Prior-phase outputs.** The 09B connection, pull, fake and desktop `c1_tests` harness (`apps/desktop/src-tauri/src/lib.rs:546`)
  exist as the plan assumes.

## Criteria decidability (both questions: writable now with one exact outcome? fixture producible by the dependency?)

| Criterion | Writable now | Producible (rule 17) | Notes |
|---|---|---|---|
| L1·C1 (a–g) | yes | yes | Wire field is `items`, not `products` (`IntegrationLabelProductPage`); the domain name differs on purpose. `labelResetReason`, `thumbUrl`, `createdAt` not required (absent allowed); `hasPrice` on a list row is a non-null bool; `workFrom` required. U1 task 6's "every field set" must still carry `labelPrintedAt: null` (the row expects `printed_at None`), as `API.md`'s example does. `1_795_165_920_000` checked = 2026-11-20T09:12:00Z. |
| L1·C2 (a–e) | yes | yes | `GET …/status` answers a bare array, one entry per id, in order (`API.md` › status; OpenAPI response schema). `LabelStatusOut.hasPrice` nullable confirmed. |
| L1·C3 (a–b) | yes | yes | Bare array (`IntegrationLabelTemplate[]`). Integer `widthMm` deserialises into `f64`. |
| L1·C4 (a–i) | yes | yes | Codes match `API.md` › Errors (401 `invalid_api_key`, 403 `insufficient_scope`, 404 `not_found`, 503 `product_list_unavailable`, 422 `validation_error`). The two transport shapes are `map_error`'s (`transport.rs:121-127`). |
| L1·C5 (a–i) | yes | n/a | Sentences match 09C §10 and master plan Words. |
| L1·C6 (a–c) | yes | n/a | |
| L1·C7 (a–f) | yes | yes | `IntegrationPlanItem.template`/`title` nullable confirmed; `IntegrationRunItemIn` has `additionalProperties: false`, so absent fields must not be serialised (row (a) asserts exactly that). |
| L1·C8 (a–e) | yes | yes, except where the master plan already marks it ungrounded | Preview 404 can also be `product_not_found` (`API.md` › preview); still `Rejected`, consistent. `%PDF-` is listed as not grounded in master plan §3. |
| L1·C9 (a–g) | yes | **incomplete** | See D4 (`lifecycleResult: "none"`). Otherwise `IntegrationLabelRun` required fields confirmed (`expiresAt`, `requestedByKind`, `requestedByName` nullable). `unresolved` is optional, as row (c) assumes. |
| L1·C10 (a–e) | yes | yes | `downloadUrl` is a path (`API.md` run example). 410 `artifact_expired` confirmed. |
| L1·C11 (a–d) | yes | yes | `label.printed` data `{productId, sku, runId, priceMinor}`; `label.reset` data `{productId, sku, reason}`; `caseId` null (`EVENTS_AND_WEBHOOKS.md` › Price-label events). |
| L1·C12 (a–i) | yes (b: see D2; i: see D8) | n/a (fake) | (e): `fail_matching` applies to any path containing the part, before auth (`fake_pricing.rs:805-816`). |
| L1·C13 (a–e) | yes (c: see D9) | n/a | |
| L1·C13 (f) | **no** | n/a | D1: fails on the current schema. |
| L1·C14 (a–i) | yes (a: see D11) | n/a | (d): the fake redelivers at the front of the next page (`fake_pricing.rs:690-705`); `pull` skips `id ≤ cursor`, so the page's label list is empty and nothing is emitted, as the row expects. |
| L1·C15 (a–e) | yes | n/a | The harness must set up the linked session on `TestApp` before it is moved into `AppState` (`lib.rs` `c1_tests::wired`). |
| L1·C16 (a–d) | yes | n/a | D7(b) applies to the existing settings tests, not to these rows. |
| L1·C17 (a–d) | yes (see D13) | n/a | |
| L1·C18 (a–f) | yes | n/a | D7(a) applies to the existing tests rendering the same components. |
| L1·C19 (a–d) | yes for the rows as written | n/a | D5/D6: the states the rows leave out are undetermined. |

## Trace verification (both directions)

- Rows → contracts: every row cites at least one ledger or contract item, or a registry line that states the asserted
  behaviour (D12 for notation). I found no row whose trace says something other than what the row asserts. Checked
  samples: C12(g) ↔ ACC.4/.5; C13(c) ↔ MC-L3.3 `STATUS_FRESH_MS`; C14(f) ↔ MC-L4.3 "every way out"; C10(b) ↔ MC-L2.12.
- Contracts → rows: the ledger entries the planner's handoff assigns to L1 are each served. M-L1 by C13(f) (row defective,
  D1). M-L2 by C7(a). M-L3 by C9(a). M-L6 by C12(a), (c), (h). ACC.1–.7 and MC-L4.1–.4 each have at least one row.
  MC-L3.5–.7 are served by C17/C18. MC-L4.5 → C18(d).

## Points for the coordinator (not findings)

- The pull hazard the planner flagged holds up on paper. `Stop::Changed`, `Stop::Provider` and `Stop::Storage` all leave `pull`
  through `?` inside `in_lane`, and the empty-page path calls `complete_check`. A single wrapper around the current body
  that delivers the collected list on return covers all of them. L1·C14(f) exercises the `Stop::Provider` exit only.
  Optionally add a `Stop::Changed` input (a `key_set` between two pages) if the reviewer wants that exit pinned. I don't
  require it.
- The only places that match on `AppEvent` variant names are `apps/desktop/src-tauri/src/lib.rs` (bridge) and tests. No other
  match needs a `LabelsChanged` arm.

## Write perimeter of this session

- Created: `docs/implementation/handoffs/reviewer/handoff_L1_labels_provider_and_marker_projection_r0.md` (this file).
- Nothing else: no plan, intention, prompt, tracker or code edited; no commit; no archgraph delta (repo has none in use for
  this track); no test or build run.

## Appendix (non-authoritative, discard): none

No skeleton is attached, on purpose. The ledger above is the product.
