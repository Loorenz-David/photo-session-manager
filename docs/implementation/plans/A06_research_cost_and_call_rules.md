---
phase: A06_research_cost_and_call_rules
title: Research core, part 4 — what a provider call costs in whole millionths of a dollar, when a run must stop reading or stop altogether, which day a call counts for, what to do after each kind of failed call, and how a key is kept out of every record
difficulty: high
projection: yes
depends_on: [A05_research_confidence]
---
# A06 — Cost, stop rule, the day (08B MC-A8) and call rules: error classes, attempts, redaction (MC-A9, pure part)

## Goal
The pure functions of MC-A8 and of the decision half of MC-A9. They take plain values and return plain values; the
gateway (A11) and the run rules (A13) call them. **Not in this phase:** recording calls, sending anything, the price
table's settings key (A13), adapters turning a provider's reply into a `CallFailure` (A15–A17).

## Read first
`master_plan_research.md` §3, §6.3, §6.5, §7.4 · book `14` (MA5, MA15, MA16), `08A` §9, §10, `08B` **MC-A8** (all),
**MC-A9** (all), MC-A6 (Offline), `05A` MC-4 (calendar dates, time zone) · code
`crates/core/src/mechanisms/values.rs` (`canonical_decimal`), `crates/core/src/mechanisms/time.rs`,
`crates/infra/src/jobs.rs` (`backoff`: 1 s, 2 s, …).

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `crates/core/src/mechanisms/research/{cost.rs, calls.rs}`; in `domain/research.rs`: `ErrorClass`,
`CallFailure {sent, status, kind, text, usage}`, `FailureKind {Connect, Timeout, Reset, Http, Schema, Refusal}` with a
`quota: bool` flag set by the adapter, `PriceTable`, `NextCall`, `CallAction`. Edited: `mechanisms/research/mod.rs`,
own tracker row.

## Core mechanisms (unit tests allowed; one inline test per row, `mc_a8_<slug>` / `mc_a9_<slug>`)
`cost::{parse_micro_usd, call_cost, next_call, day_spend}` · `calls::{classify, action, redact}`.

## Ordered tasks
1. `parse_micro_usd(&str) -> Option<i64>`: an MC-9 decimal string with at most 6 fractional digits, exact, no float.
2. `call_cost(usage: Option<&[(unit, count)]>, sent, provider, model, &PriceTable) -> i64`: `ceil(Σ usage × p / 10^6)`
   with `p = usd_per_million × 10^6` as an integer, in `i128`; no usage reported → `fallback_call_usd`; never sent →
   0; a reported unit missing from the table → the fallback for the whole call.
3. `next_call(run_cost, cap, reserve, stage) -> NextCall`: stages 1–6 → `Call` iff `run_cost + reserve < cap`, else
   `SkipToDecide`; stage 7 → `Call` iff `run_cost < cap`, else `EndAtCap`.
4. `day_spend(calls: &[(started_at_ms, cost)], now_ms, tz) -> i64`: the sum over calls whose **local calendar date**
   equals today's in `tz`.
5. `classify(&CallFailure) -> ErrorClass` and `action(class, attempt) -> CallAction {HoldOffline, Retry {wait_ms},
   FailRun, NotDetermined}` with `RESEARCH_CALL_ATTEMPTS = 3` and waits of 1 s then 2 s.
6. `redact(text, secrets) -> Redacted {text, hit}`: every occurrence of each secret, plain and URL-encoded, becomes
   `<redacted>`. An empty secret is ignored.

## Criteria (7 criteria, 29 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `parse_micro_usd` | `0.50`; `15`; `0.000001` | 500000; 15000000; 1 | MA15 |
| C1(b) | same | `0.0000001`; `1e3`; `1,5`; `-1`; the empty string | none, five times | MA15 |
| C2(a) | `call_cost` | 1000 `input_token` at 2.50 and 500 `output_token` at 10.00 USD per million | 7500 | MA15 |
| C2(b) | same | 1 `input_token` at 0.15 per million | 1 (rounded up) | MA15 |
| C2(c) | same | 3 `input_token` and 4 `output_token`, both at 0.15 per million | 2 (summed, then rounded up once) | MA15 |
| C2(d) | same | sent, no usage reported; fallback 0.02 | 20000 | MA15 (R-IA19) |
| C2(e) | same | never sent | 0 | MA15 |
| C2(f) | same | usage names a unit absent from the table; fallback 0.02 | 20000 | MC-A8 |
| C2(g) | same | 10^12 `output_token` at 1000.00 per million | 1000000000000000 (no overflow) | MA15 |
| C3(a) | `next_call`, cap 500000, reserve 20000, stage 3 | cost 479999; cost 480000 | `Call`; `SkipToDecide` | MA5 (card 5) |
| C3(b) | same, stage 7 | cost 499999; cost 500000 | `Call`; `EndAtCap` | MA5 |
| C4(a) | `day_spend`, zone Europe/Stockholm, now = 2026-11-12 10:00 local | calls at 2026-11-11 23:59:59.999 and 2026-11-12 00:00:00.000 local | only the second counts | MA15 |
| C4(b) | same, now on 2026-10-25 (the 25-hour day) at 23:30 local | calls at 00:30 and at the second 02:30 of that date, and one at 2026-10-26 00:10 | the first two count | MA15 |
| C4(c) | same | the same call list evaluated with zone UTC | the sum follows the evaluation zone's dates | MC-A8 |
| C5(a) | `classify` | not sent, connect failure | offline | MC-A9 |
| C5(b) | same | sent, timeout; sent, connection reset | transient; transient | MC-A9 |
| C5(c) | same | status 408; 429 without the quota flag; 500; 502; 503; 504 | transient, six times | MC-A9 |
| C5(d) | same | status 401; 403 | key; key | MC-A9 |
| C5(e) | same | status 429 with the quota flag; status 402 with the quota flag | quota; quota | MC-A9 |
| C5(f) | same | status 400; 404; 422 | request, three times | MC-A9 |
| C5(g) | same | reply not matching the schema; a refusal | schema; refusal | MC-A9 |
| C6(a) | `action` | offline, any attempt | `HoldOffline` | MC-A9 |
| C6(b) | same | transient at attempt 1; 2; 3 | `Retry 1000`; `Retry 2000`; `FailRun` | MC-A9 (R-IA20) |
| C6(c) | same | schema at attempt 1; 3 | `Retry 1000`; `FailRun` | MC-A9 |
| C6(d) | same | key; quota; request, each at attempt 1 | `FailRun`, three times | MC-A9 |
| C6(e) | same | refusal at attempt 1 | `NotDetermined` | MC-A9 |
| C7(a) | `redact`, secret `sk-ab+c/d=` | `Bearer sk-ab+c/d=` ; `?api_key=sk-ab%2Bc%2Fd%3D&q=1` | `Bearer <redacted>`; `?api_key=<redacted>&q=1`; `hit` true both | MA16 |
| C7(b) | same, two secrets | a text holding both, one of them twice | all three occurrences replaced | MA16 |
| C7(c) | same | a text holding no secret; a secret list containing the empty string | unchanged, `hit` false; unchanged, `hit` false | MA16 |

## Delegated choices
A status the contract does not name (for example 505) is classified `request` and recorded under Assumptions ·
the time-zone type used in `day_spend` (`chrono` with `chrono-tz` in tests, as `mechanisms/time.rs`) · whether the
URL-encoded form also matches lowercase hex escapes (it should; record it).

## Manual try-out
Nothing to see in the app yet. `cargo test -p psm-core mechanisms::research::cost` and `…::calls`.

## Notes
- Money is `i64` micro-USD everywhere; a float in this module is a B1 finding.
- C4(b) uses the date `master_plan.md` §7.4 names as the DST fixture day.
- Review mode: `cargo test -p psm-core mechanisms::research`, clippy, fmt.

## Review log
