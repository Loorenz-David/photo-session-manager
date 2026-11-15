---
plan: R3_research_rules
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-12
actor: opus
---

# R3 research rules — review r1

**Verdict: APPROVED_WITH_NOTES.** All 132 criterion rows (31 criteria, 5 units) hold at their boundaries. The production code follows 08B MC-A1, MC-A2, MC-A3, MC-A4, MC-A7 (fingerprint), MC-A8 and MC-A9, and the names in `master_plan_research.md` §6.5. There are no blocking findings (B1–B4). Four notes went to `docs/implementation/backlog_research/R3_research_rules.md`, the folder the plan's Review mode names (not `backlog.md`).

## ⚠ OWNER DECISIONS REQUIRED (0)

None. Nothing needs the owner.

## Verified correct

**Gate.** Tracker read IMPLEMENTED. Checkpoints exist: U1 `11da4e1`, U2 `946d55f`, U3 `123a357`, U4 `667755f`, U5 `80c0c64`, closing `b2ad4a5`.

**Perimeter.** `git diff --stat 11da4e1^ 80c0c64` lists 16 files, and they equal the handoff's declared list: `Cargo.lock`, `crates/research/Cargo.toml`, `src/{lib.rs, domain.rs, writes.rs}`, `src/rules/{mod, value_map, sites, fingerprint, confidence, cost, calls, write_table}.rs` and `tests/engine/{main, harness, writes}.rs`.
- The `Cargo.lock` change is confined to `psm-research`'s dependency list.
- The closing commit holds only the plan's Tracker line and Review log, plus the implementer handoff.
- No app file changed.

**Checks.** Each was run once, in the plan's Review mode, on the clean checkpoint tree:

| Command | Result |
|---|---|
| `cargo test -p psm-research` | lib 110/110 and engine 25/25 passed |
| `cargo clippy -p psm-research --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cargo check --workspace` | clean |

The app suite was not run, as flow §10 directs.

**By reading, unit by unit.**
- **U1 (MC-A2).**
  - Choices: a byte-equal choice wins first, then the unique choice with the same `N`. An empty key never matches, and two choices sharing a key fail.
  - multi_choice: results follow the definition's choice order, dropped elements are reported, and an empty result fails.
  - Integer and decimal: exponents, thousands separators and decimal commas are refused.
  - Measurements: exact `i128` fractions (254/10 and 45 359 237/100 000), half-even to tenths. The target unit comes only from `validations.unit_names`, and a first unit with no name fails.
  - Every stored value goes through core `canonicalize`.
  - `page_statement` returns none when the page states two values that are not `S`-same. Unmapped statements compare by name key.
- **U2 (MC-A3, MC-A7 F).**
  - `host`: scheme check, IDNA, trailing dot, port.
  - `parse_entry`: strips one `www.`, needs a dot, refuses an IP.
  - `add`: refuses an entry that equals, is a parent of, or is a subdomain of an existing entry or a shop host, with the plan's exact text.
  - `matches` keeps the dot boundary.
  - Shop is checked before trusted. IP literals are never trusted.
  - `U`: default port dropped, `/` for an empty path, fragment dropped, tracking names dropped, the rest sorted by name then value.
  - Own listing: matched by `U`, or by the path tail on a shop host.
  - Reading order: shop, then trusted by priority, then untrusted; then rank; ties by `U`. Merging keeps the best rank. The own listing and invalid URLs are reported apart.
  - Canonical JSON: keys sorted by bytes, NFC on keys and strings, floats refused. The fingerprint is computed over `{stage, unit, v: 1}`.
- **U3 (MC-A4).**
  - Levels and support follow MC-A4. Medium counts the distinct shop and trusted identities that have a `match`.
  - When several values qualify at the top level, the one with the most support is written at `low`. A tie is not determined. An unmapped winner is not determined and is named.
  - Values that qualify only below the top level go into `named`.
  - Measured fields use `match` items only, and high is capped to medium.
  - Visible fields follow the six-row table, with `w` taken over listings only.
  - Condition fields are never written without a Condition photo. A sure read is medium, an unsure read is low.
  - Sources are the pages at the decided level, in the order given.
  - No confidence is taken from any input.
- **U4 (MC-A8, MC-A9).**
  - Money is exact micro-USD. No `f32` or `f64` appears in the crate.
  - `call_cost`: `i128` sum, one round-up, fallback on missing usage or an unknown unit, 0 when the call was never sent.
  - Stop rule: stages 1–6 call while `cost + reserve < cap`; Decide calls while `cost < cap`.
  - `day_spend` uses the local calendar date in the evaluation zone. C4(b) checks the 25-hour day, 2026-10-25. C4(c) is discriminating: in UTC the same lists give 0 and 110.
  - The class and action tables match MC-A9, with waits of 1000 and 2000 ms from `jobs::backoff` and 3 attempts.
  - `redact` covers the plain form and percent-encoding with upper- and lowercase hex, longest form first, and ignores an empty secret.
- **U5 (MC-A1).**
  - `suggest` reads the item, the definitions, the layout, the stored row and the cleared set inside one `Store::transaction`. The layout comes from the stored preset, then the `""` preset, then every definition of the step, which mirrors `resolve_preset` and `presets_get`.
  - Never-written fields are checked first.
  - The write table fails closed: `ai` with no status, and `default`/`integration` with `suggested` or `accepted`, are refused.
  - The prior is saved over a live default or integration row, keeping validity and the raw value. It is kept when research replaces its own suggestion.
  - Cleared set: the cleared row's value plus the values of `cleared` and `kept_previous` feedback, membership by `same`. Core's `clear` keeps the row's value and provenance, so the stored cleared value is the real one.
  - A live default or integration row that already holds an `S`-same value gives "already holds this value".
  - Idempotency uses core's `same_content` rule: no write and no activity bump. No undo step is recorded.
  - `lower` acts only on `ai` + `suggested` rows.
- **Production code hygiene.** There is no `unwrap` or `expect` outside tests.

**Tests.**
- Each row has exactly one test named for it.
- Each test asserts that row's exact outcome at the row's boundary. For U5 the boundary is the stored row read back from real SQLite plus the returned outcome.
- Each test would fail on the wrong outcome. The fixtures isolate their rule:
  - C1(e): one entry, two pages.
  - C1(f): one trusted site and one untrusted site.
  - C3(c): a disagreeing `w` at low, so treating low like high would fail.
  - C6(a)–C6(c): the given order is not sorted, and the input mixes label, shop and trusted evidence.
  - A06·C4(c): two zones.
  - A07·C3(b): kept_previous over a live default.
  - A07·C5(b): eight never-written kinds, each of which would otherwise write or refuse differently.
- The three `write_table` unit tests trace to A07·C1, C2 and C5. `write_table` is a listed core mechanism, so they are not orphans.
- No test checks internal calls or query text.

**Assumptions (28).** None contradicts the book.
- 23 (the cleared set is checked even when there is no stored row) applies MA10's invariant.
- 24 (a Product-step field is one in the resolved Product layout or of group Product) fails closed.
- 1 (text trimmed at both ends) is what row A03·C4(a) requires.
- 15 (the shop's identity counts toward medium) is MC-A4's own wording ("among `shop` and the trusted entries").
- The owner-relevant assumptions are in the narrative in plain words.

**Hard constraints H1–H8.** None is violated.
- H1: no external write.
- H2, H3: nothing touches a RAW file or pixels.
- H4: research writes are idempotent.
- H5: no UI automation.
- H7: core is untouched, and the research crate depends only on `psm-core` and `psm-infra`, with `psm-testkit` as a dev-dependency (§3 isolation).
- H8: no Tauri in the crate.

**Secrets and live services.** The only secrets are synthetic test strings, and no live service is called.

**Review probes.** None. No file was touched and reverted.

## Findings

None blocking (B1–B4). Nothing is routed to a fix.

## Notes written to backlog
All four are in `docs/implementation/backlog_research/R3_research_rules.md`.
1. **→ R4·U4.** `call_cost` charges 0 when a (provider, model) has no fallback price. `PriceMissing` must therefore cover every pair a run can call, including the `""`-model providers, or a timed-out call goes uncounted.
2. **→ R6 / R4·U3.** A key listed in both the Details and the Product layout is researched, because the first step found wins and `presets_set` does not prevent it. The fail-closed reading of MC-A1 is "never written when any step listing it is Product".
3. **→ R8.** A 401/403 reply that also carries the quota flag is classified `key`, so the run says "key rejected" when the account is out of credit.
4. **→ book 08B.** MC-A4's invariant ("nothing above low without a label, a shop match, or two trusted sites") contradicts the visible table (a sure read with no listing is medium) and the condition paragraph (a sure read is medium). The code correctly follows the enumerated rows. The invariant should be scoped to attribution and measured fields.

## Lessons for the plans
- U4's file list named neither `Cargo.toml` nor `Cargo.lock`, although its tasks need `chrono`, `chrono-tz` and `psm-infra`. Future unit file lists should include the manifest whenever a task names a new crate.
- The prompt template still says actor `codex` for research implementers. The runner used Claude Opus (`PSM_IMPLEMENTER=claude`).
- The review prompt says notes go to `backlog.md`, while the plan's Review mode says `backlog_research/<plan>.md`. The template should take the path from the plan.

## Write perimeter of this review
- `docs/implementation/plans/R3_research_rules.md`: the Tracker line, and an appended Review log entry.
- `docs/implementation/backlog_research/R3_research_rules.md` (new).
- `docs/implementation/narratives/R3_research_rules.md` (new).
- This handoff.

No production or test file was touched. No commit was made.
