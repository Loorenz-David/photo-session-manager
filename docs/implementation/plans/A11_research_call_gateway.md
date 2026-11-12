---
phase: A11_research_call_gateway
title: Every call research makes goes through one gate — it writes down what is about to be sent (and which photos) before sending, never sends a photo you deleted, retries sensibly, counts the cost, and keeps keys out of the record
difficulty: high
projection: yes
depends_on: [A09_research_publish_hold, A10_research_ports_images_and_fakes]
---
# A11 — The call gateway: record then send, outbound rows, attempts, cost (08B MC-A5 order, MC-A9, MC-A8; MA3, MA5, MA12, MA15, MA16)

## Goal
One function, `CallGateway::call`, through which every provider call of a run passes. In one transaction it
re-reads photo eligibility, applies the stop rule, inserts the call record (`sent`) and one outbound row per photo;
only then it sends; then it records the reply, the usage and the cost, or the failure and the next action. Add the
research run tables. **Not in this phase:** who creates and starts runs (A13), the stages (A12), real adapters.
(`depends_on` names A09 only for the migration order, `master_plan_research.md` §6.1.)

## Read first
`master_plan_research.md` §3, §6.3–§6.6, §7.4, §7.5 · book `14` (MA3, MA5, MA12, MA15, MA16), `08A` §2 (HA3, HA6),
§10, §10A, `08B` **MC-A5** (Order per call), **MC-A9** (all), **MC-A8** (Call cost, Stop rule, One call in flight),
MC-A6 (Offline) · code `mechanisms/research/{cost.rs, calls.rs}` (A06), `ports/research.rs` and
`crates/testkit/src/fake_research.rs` (A10), `crates/core/src/ports/store.rs` (`transaction`),
`crates/infra/src/sqlite/store.rs`, `crates/testkit/src/{fake_secrets.rs, logs.rs, isolation.rs}`.

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `crates/infra/src/sqlite/migrations/a03_research_runs.sql`, `crates/core/src/services/research/{mod.rs,
gateway.rs}`, `crates/testkit/tests/suite/research/a11_research_call_gateway.rs` (+ `mod` line).
Edited: `crates/core/src/services/mod.rs`, `crates/core/src/domain/research.rs` (`ResearchRun`, `ResearchCall`,
`Outbound`, `CallState`), `crates/core/src/ports/store.rs`, `crates/infra/src/sqlite/store.rs` (run, call, outbound
methods; the migration in the research block after `a02`), `crates/testkit/src/test_app.rs`, own tracker row.

## Core mechanisms
None new (A06's functions are called, not re-tested).

## Ordered tasks
1. Migration `a03_research_runs.sql`: the five tables of §6.4 with the two partial unique indexes.
2. `CallGateway::call(run_id, stage, unit_key, request) -> GatewayOutcome {Answered(reply), NotDetermined(text),
   SkipToDecide, EndAtCap, HoldOffline, RunFailed {stage, class, text}}` for the three ports.
   Step 1, one transaction: for each `ResearchImage` re-read the photo and drop it when no longer eligible (the
   caller gets the dropped ids for the step detail `photo removed since`); `next_call` (A06) with the run's cost,
   cap and reserve — if not `Call`, return without a record; insert `research_calls` (state `sent`, attempt, the
   request with each image as `{"image_sha256": …}`) and one `research_outbound` row per remaining photo (provider,
   `sent_at`, SHA-256); commit. A call left with no photo although it needs one is not sent.
   Step 2: send. Step 3, one transaction: state `answered` with response, usage, `cost_micro_usd`, times; add the
   cost to `research_runs.cost_micro_usd`. On failure: `classify` and `action` (A06) — `HoldOffline` marks the call
   `not_sent`, cost 0, no attempt counted; `Retry` records `failed`, charges per MC-A8 and re-issues after the wait
   as the next attempt (the wait is driven by the clock the service is given); `FailRun` sets the run `failed` with
   stage and error (`key rejected by <provider>` for class key); `NotDetermined` returns the refusal text.
3. Redaction layer 2: before every insert or update of a call record and before every research log line, `redact`
   with `ResearchService::secret_values()` (the research keys from the secrets port and the integration's secret
   and token); a hit sets `redaction_hit` and logs an error without the value.
4. Third-party images a page reply carries are stored in the research files and referenced by the call record;
   they are never outbound rows.

## Criteria (8 criteria, 19 rows)
Boundary: `CallGateway::call` on `TestApp::with_research()` with a run row inserted in state `running`; outcome =
the rows of `research_calls`, `research_outbound`, `research_runs`, and the fakes' request logs.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | order | a model call with two photos; the fake's `before_call` hook reads the database | inside the hook: one call row in state `sent` and two outbound rows already committed | MA12 |
| C1(b) | outbound = sent | the same call | the SHA-256 values of the outbound rows equal the images the fake received, exactly | MA3 |
| C2(a) | delete before the gate | two photos; one is classified `deleted` before the call | the fake receives one image; one outbound row; the outcome names the dropped photo | MA12 |
| C2(b) | delete after the gate | the hook deletes a photo while the call is in flight; a second call follows with the same two images | the first call's two outbound rows stand; the second call sends one image and has one outbound row | MA12 |
| C2(c) | nothing left | the only photo is deleted before the call | no request reaches the fake; no call row; no outbound row | MA3 |
| C3(a) | record | an answered call with usage | state `answered`; the stored request has `{"image_sha256"}` in place of each image; response, usage, cost, `started_at`, `finished_at`, duration present | MA9 |
| C3(b) | listing images | a page reply carrying two images | both stored as files and referenced by the call; no outbound row | MC-A5 |
| C4(a) | redaction | the fake's reply body and a fault's error text both contain the model key value (plain, and URL-encoded) | the stored response and error text read `<redacted>`; `redaction_hit` is true; no stored column contains the value | MA16 |
| C4(b) | log | the same call, with the log captured (`in_own_process`) | no log line contains the key value; one error line reports the hit | MA16 |
| C5(a) | attempts | faults: transient, transient, then a reply | three call rows, attempts 1, 2, 3; the second is issued 1 s and the third 2 s later on the manual clock; outcome `Answered` | MC-A9 (R-IA20) |
| C5(b) | attempts | three transient faults | outcome `RunFailed`; the run is `failed`, its error names the stage and the failure | MC-A9 |
| C5(c) | key | a fault with status 401 | one call row; run `failed` with `key rejected by <provider>`; no second request | MC-A9 |
| C5(d) | refusal | a refusal reply | outcome `NotDetermined` with the refusal text; one request; the run stays `running` | MC-A9 |
| C5(e) | offline | a fault with `sent: false` | call state `not_sent`, cost 0; outcome `HoldOffline`; the next call for the same unit has attempt 1 | MA15, MC-A6 |
| C6(a) | cost | a reply reporting usage, with the run's price snapshot | the call's `cost_micro_usd` equals `call_cost` for that usage; the run's cost grew by the same amount in the same commit | MA15 |
| C6(b) | cost | a timeout after send (no usage) | the failed call is charged `fallback_call_usd` | MA15 (R-IA19) |
| C7(a) | stop rule | run cost + reserve = cap; a stage 3 call | outcome `SkipToDecide`; no call row; no request | MA5 |
| C7(b) | stop rule | run cost = cap; a stage 7 call | outcome `EndAtCap`; no call row; no request | MA5 |
| C8(a) | migration | insert a second `running` run for one session; a second `queued` run for one session | each insert is refused by the database | MA13 |

## Delegated choices
Internal split of `gateway.rs` · whether waits between attempts are a sleep on an injected clock or a returned
"retry at" the caller honours (the tests need the manual clock to drive them; no real-time wait in a test, flow §4A)
· column encodings of request and response. Not delegated: the order commit → send, and that eligibility and the
stop rule are read in that first transaction.

## Manual try-out
Nothing visible yet. `cargo test -p psm-testkit --test suite research::a11_research_call_gateway::` reads as the
list of promises: recorded before sent, deleted photos stay home, keys never stored.

## Notes
- C4 uses `FakeSecrets`' distinctive key values, which appear in no expected text (`master_plan.md` §7.5 practice).
- Hazard: a retry inside the gateway must not hold a database transaction across the send.
- Review mode: this phase's module, `p01_storage_identity::` (table list), clippy, fmt.

## Review log
