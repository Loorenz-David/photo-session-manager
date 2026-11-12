---
plan: R3_research_rules
role: implement
round: 1
state: DONE
date: 2026-11-12
actor: opus
---

# R3 research rules — implement r1

## What was built

The research agent now has its rulebook and its one door for writing into the form. Nothing calls these yet (that is R4), and there is nothing new to see in the app.

- **From a model's answer to a form value (U1).** A model's answer for a field becomes a valid stored value or is set aside as "not determined". Choices are matched exactly first, then by name with punctuation, case and spacing ignored. Lists keep the field's own choice order and report what was dropped. Numbers refuse exponents, thousands separators and decimal commas. Sizes and weights are converted exactly into the field's unit (inches, pounds and so on) and rounded half-even to one decimal, with no floating point. The unit also decides what a listing page "states" for a field: a page that names two different makers states nothing.
- **Which website a page belongs to (U2).** Web addresses are normalised: lowercase, international names, port and trailing dot. The trusted-sites list refuses overlapping entries ("already covered by etsy.com"). A page is classified as the shop, a trusted site or an untrusted site, so `notetsy.com` is never Etsy. Duplicates are recognised even when tracking parameters differ, and the item's own shop listing is kept out of the evidence. The reading order is shop, then trusted by priority, then the rest. Fingerprints for reusing finished work are exact SHA-256 hashes of a canonical form.
- **How sure a suggestion is (U3).** This is the MC-A4 rule for designer/maker, measured, visible and condition fields, and every enumerated row of the contract has its outcome. A designer reaches medium only from two different trusted sites, and nothing gets above low without a label, a shop match or two such sites.
- **What a call costs and what to do when one fails (U4).** Money is integer micro-USD and prices are parsed exactly. A call's cost is rounded up once. A call with no reported usage pays the fallback, and a call never sent costs nothing. The per-run cap keeps a reserve for the final step. Today's spend follows the local calendar day, including the 25-hour DST day. Each error has a class and an action: offline holds; transient retries after 1 s, then 2 s, then fails; a bad key fails at once; a refusal gives "not determined". Secrets are scrubbed from any text, both plain and URL-encoded.
- **What research may write over (U5).** `writes::suggest` and `writes::lower` work on the app's real store in one transaction:
  - they never overwrite a photographer's value, an accepted suggestion or a human clear;
  - they save the replaced default or Shopify value as the prior;
  - they never re-suggest a value the photographer cleared, even through earlier clears;
  - they leave alone a field that already holds the value;
  - they skip fields research never fills;
  - writing the same suggestion twice is a no-op, and they add no undo step.

## Tests → criterion rows

Every row has exactly one test named after it (`<row id>_<slug>`, e.g. A05·C1(d) → `a05_c1d_two_trusted_sites_medium`). Each asserts the row's exact outcome at the row's boundary.

| Unit | Rows | Tests (module) | Count |
|---|---|---|---|
| U1 | A03·C2(a)–C6(c) | `rules::value_map::tests::a03_*` (inline, on `map_output` / `statement` / `page_statement`) | 18 |
| U2 | A04·C1(a)–C7(d) | `rules::sites::tests::a04_c1…c6*` (24), `rules::fingerprint::tests::a04_c7*` (4) | 28 |
| U3 | A05·C1(a)–C6(c) | `rules::confidence::tests::a05_*` | 32 |
| U4 | A06·C1(a)–C7(c) | `rules::cost::tests::a06_c1…c4*` (14), `rules::calls::tests::a06_c5…c7*` (15) | 29 |
| U5 | A07·C1(a)–C6(b) | `engine` program `writes::a07_*`: `writes::suggest` / `lower` on a `TestApp` with real SQLite, with the stored row read back | 25 |
| U5 core | A07·C1, C2, C5 (supporting) | `rules::write_table::tests::a07_c1_decide_write_table`, `a07_c2_conflict_needs_a_differing_prior`, `a07_c5_never_written_fields` (plan's Core mechanisms) | 3 |

That is 132 row tests plus 3 core-mechanism unit tests. Every test maps to a row, so there are no orphans. Fixture notes:
- C1(e) uses two pages on one entry and nothing else.
- C1(f) uses one trusted site and one untrusted site.
- A06·C4(c) evaluates both the C4(a) list and the C4(b) list in UTC (0 and 110), so it fails if the zone is ignored.

## Assumptions

U1
1. Text and textarea trim leading as well as trailing white space before `canonicalize`. Row A03·C4(a) expects `Oak table`, while MC-9 alone trims only the end.
2. In a multi_choice answer, a non-text element fails the whole answer. Only text elements that match no choice are dropped.
3. An integer given as a string may carry a leading `-`. A decimal given as a JSON number is read through its JSON text, so an exponent fails.
4. Measurements:
   - The input unit must be one of the seven neutral names for that type, and it needs no `unit_names` entry itself.
   - The target is `validations.units[0]`, and it must have a `unit_names` entry.
   - A field with no `units` list fails.
5. `statement(def, stated)` maps the stated text. R4 may add the model's own value as a parameter (§6.5 allows signatures to gain parameters).
6. A url field accepts exactly what `sites::host` accepts (absolute http/https with a host), after trimming.

U2
7. Parsing uses the `url` crate, including its IDNA.
8. `page_identity` drops any user-info. It keeps query parameters as written (no re-encoding), sorted by raw name and then raw value.
9. `TrustedSites::add`:
   - It returns the stored entry and checks existing entries before shop hosts.
   - Parse refusals have their own texts: "not a web address", "an IP address cannot be a trusted site", "a site needs a dot in its name".
   - `remove` takes typed text.
   - `reorder` requires the same set of entries.
10. An IP-literal host is untrusted, and its identity is the address itself.
11. The own-listing path tail may be preceded by any segments (a locale or more). A trailing slash is ignored.
12. Fingerprint canonical JSON normalises object keys to NFC as well as values. The stage is a `u8`.

U3
13. `Decision {written: Option<(value, confidence)>, sources, named}`:
   - `named` is ordered by level, then support, then first appearance (labels before listings).
   - `sources` lists each page once.
   - For a visible field, `sources` holds w's pages only when v and w are the same value. When v disagrees with w, w comes first in `named`.
14. `field_class(def, never_written: bool, attribution_keys)` follows the plan's note rather than its `in_layout` wording.
15. The shop's identity `shop` counts as one of the "distinct identities among shop and trusted" for medium.
16. A measured value capped from high to medium keeps the high level's sources (the shop pages).

U4
17. `CallFailure`, `FailureKind`, `ErrorClass` and `Stage` are in `psm_research::domain`, and `CallFailure` has no `exchange` field yet. R4·U1's ports can re-export them or add the field.
18. `classify` order:
   - not sent → offline;
   - 401/403 → key, checked before the quota flag;
   - the quota flag on any other status → quota;
   - 408, 429 and 500–504 → transient;
   - every other status, including 505 → request;
   - a connect-kind failure after sending → transient.
19. `call_cost` charges 0 when the table has no fallback for (provider, model), because MC-A6's `PriceMissing` prerequisite keeps such a run from starting. A sum beyond `i64` saturates.
20. `redact` matches upper- and lowercase hex escapes and replaces the longest form first.
21. `action` uses `psm_infra::jobs::backoff`, so the research crate now depends on `psm-infra` (§3 allows this).
22. `parse_micro_usd` accepts leading zeros and refuses `1.`.

U5
23. The cleared-set check runs in every writable state, including when there is no stored row. This applies MA10 ("no value in C is ever written"); the MC-A1 table names the check only for cleared rows.
24. A product-step field is a key in the resolved Product layout or a definition whose group is Product. A step with no stored preset, and no `""` preset, lists every definition of that step, as `presets_get` does. A key with no definition is "not researched".
25. Refusal texts:
   - "the photographer accepted this value";
   - "the photographer wrote this value";
   - "the photographer cleared this field";
   - "the stored value cannot be changed by research" (fail closed);
   - "the value is not valid for this field" (when `canonicalize` rejects the proposed value).
26. `lower` returns `Unchanged` for any row that is not `ai` + `suggested`, or when there is no row. It is idempotent.
27. `suggest` takes `reason: Option<&str>` and `sources: &[String]`.
28. The harness uses its own small definition set under integration `shop`, and a Sideboard Details preset that leaves out `test.outside`.

## Deviations

- `crates/research/src/domain.rs` was created in U2 for `SiteClass`, not in U3.
- U1 needed no `Cargo.toml` edit.
- U4 added `psm-infra`, `chrono` and dev `chrono-tz` to `crates/research/Cargo.toml`; U4's file list does not name `Cargo.toml` or `Cargo.lock`.
- U5 added dev `psm-testkit`, `autotests = false` and `[[test]] engine`, as planned.
- Three extra unit tests in `rules::write_table` (a listed core mechanism) support rows A07·C1, C2 and C5.
- No `Value` field was added, and no app test file was touched (the P67 hazard did not apply). Core's `ai_suggest` and its P10 test are unchanged.
- The Tracker actor is `opus` because this session ran on Claude Opus; the prompt template says `codex`.

## Files changed

- `Cargo.lock`: the research crate's dependency list only
- `crates/research/Cargo.toml`
- `crates/research/src/lib.rs`
- `crates/research/src/domain.rs` (new)
- `crates/research/src/writes.rs` (new)
- `crates/research/src/rules/{mod.rs, value_map.rs, sites.rs, fingerprint.rs, confidence.rs, cost.rs, calls.rs, write_table.rs}` (new)
- `crates/research/tests/engine/{main.rs, harness.rs, writes.rs}` (new)
- `docs/implementation/plans/R3_research_rules.md`: the Tracker line and the Review log entry
- this handoff

No mutation probes were run (flow §4), so no file was touched and reverted.

## Checkpoint SHA

| Unit | SHA |
|---|---|
| U1 | `11da4e1` |
| U2 | `946d55f` |
| U3 | `123a357` |
| U4 | `667755f` |
| U5 | `80c0c64` |
| Closing (tracker, Review log, handoff) | the final `CHECKPOINT (not approved): R3 closing …` commit |

## Stamp

Tree at U5 `80c0c642c28f1a91226f5ec3f03fa8e053df7e49`, clean. Each command was run once, one after the other:

| Command | Result |
|---|---|
| `cargo test -p psm-research` | lib 110 passed, engine 25 passed, 0 failed |
| `cargo clippy -p psm-research --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cargo check --workspace` | clean |

There are no pre-existing failures. The app suite was not run, because no app file changed (flow §10).

## Manual try-out

1. In the repository root, run `cargo test -p psm-research --lib rules`. The rules print by name, row by row (`a05_c1d_two_trusted_sites_medium`, `a04_c3c_suffix_without_dot_is_untrusted`, `a06_c4b_twenty_five_hour_day`, …), all `ok`.
2. Run `cargo test -p psm-research --test engine writes::`. This shows the 25 write-table rows working on a real app database: refusals, the cleared set, priors, lowering and idempotency.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
