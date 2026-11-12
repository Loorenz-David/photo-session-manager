---
phase: A03_research_values_and_sameness
title: Research core, part 1 — turning what a model or a listing says into a valid form value, and deciding when two values are "the same" (names, choices, numbers and units), as pure tested functions
difficulty: high
projection: yes
depends_on: []
---
# A03 — Model output to value, name key, sameness (08B MC-A2, MC-A1 `S`)

## Goal
Add the pure functions of MC-A2 and the sameness relation of MC-A1 to `psm-core`: the name key `N`, the mapping of
a model's output to a stored value for each core type, what a listing "states" for a key, and `S(a, b)`. They are
the vocabulary every later research phase compares values with. **Not in this phase:** any table, service, port or
I/O; the write table (A07); site rules (A04); confidence (A05).

## Read first
`master_plan.md` §3, §6.1, §6.9, §7.2 · `master_plan_research.md` §3, §6.1–§6.3, §6.5 · book `01`, `14` (MA6, MA11),
`08A` §5 (Field rules), `08B` **MC-A2** (all), **MC-A1** (cleared set: the sameness table), `05A` MC-9 · code
`crates/core/src/mechanisms/values.rs` (`canonicalize`, `canonical_json`, `canonical_decimal`),
`crates/core/src/domain/attribute.rs` (`CoreType`, `Definition`, `Definition::units`).

## Dependencies and gate
Gate: 08A RATIFIED, 08B CONTRACTS RATIFIED. **Tracker row and gate: `docs/implementation/master_plan_research.md` §4.**
Reviewer notes: `docs/implementation/backlog_research.md`.

## Files expected to change
New: `crates/core/src/mechanisms/research/{mod.rs, normalise.rs, value_map.rs, sameness.rs}`.
Edited: `crates/core/src/mechanisms/mod.rs` (one line, `pub mod research;`), `crates/core/Cargo.toml` (the two
Unicode crates), `Cargo.lock`, `master_plan_research.md` §4 (own row) and §6.2 (the pins).

## Core mechanisms (unit tests allowed; every criterion row below is one inline test named `mc_a2_<slug>` or `mc_a1_<slug>`)
`normalise::name_key` · `value_map::{map_output, statement, page_statement}` · `sameness::same`.

## Ordered tasks
1. Pick and pin the crates for the full Unicode case fold and for a character's general category
   (`master_plan_research.md` §6.2); verify with `cargo generate-lockfile`; record the versions. NFKC comes from
   `unicode-normalization`, already a dependency. No hand-written Unicode tables.
2. `name_key`: NFKC → full case fold → every character of general category P\* or S\* becomes a space → runs of
   white space collapse to one space → trim. `RESEARCH_NORMALISE_VERSION = 1`.
3. `map_output(&Definition, &UnitNames, &Json) -> Mapped`: the MC-A2 table. The stored form is always produced by the
   existing `canonicalize`, so a mapped value is valid by construction. `UnitNames` maps the seven neutral unit
   names (`mm, cm, m, in` / `g, kg, lb`) to the definition's unit codes; it is data the caller supplies (H7: no
   platform code in core). Conversion uses exact integer arithmetic (1 in = 25.4 mm, 1 lb = 453.59237 g), rounding
   half-even to one decimal place; **no floating point**.
4. `statement` (one stated text → `Stated::Mapped(value)` or `Stated::Unmapped(name key)`) and `page_statement`
   (all texts one page states for a key → the single statement, or none when it states two different values).
5. `same(&Definition, &UnitNames, a, b)`: the MC-A1 table.

## Criteria (7 criteria, 30 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `name_key` | `Hans J. Wegner` | `hans j wegner` | MC-A2 |
| C1(b) | `name_key` | `  Børge   Mogensen ` and `Borge Mogensen` | `børge mogensen`; the two keys differ | MC-A2 |
| C1(c) | `name_key` | `Ｗｅｇｎｅｒ` (full-width letters), `ﬁnn` (ligature) | `wegner`, `finn` | MC-A2 |
| C1(d) | `name_key` | `STRASSE`, `Straße` | both `strasse` | MC-A2 |
| C1(e) | `name_key` | `Fritz-Hansen & Co.`, `A/S` | `fritz hansen co`, `a s` | MC-A2 |
| C1(f) | `name_key` | `Hans J. Wegner`, `Hans Wegner` | different keys (no alias table) | MA11 |
| C1(g) | `name_key` | `...`, the empty string | the empty string | MC-A2 |
| C2(a) | `map_output`, choice `[Oak, Teak]` | `"Teak"` | `Teak` | MA6 |
| C2(b) | choice `[Oak, Teak]` | `"teak "` | `Teak` (unique choice with the same name key) | MC-A2 |
| C2(c) | choice `[Oak, oak]` | `"oak"`; `"OAK"` | `oak` (byte-equal wins); fail (two choices share the key) | MC-A2 |
| C2(d) | choice `[Oak, Teak]` | `"Pine"`; `12` | fail; fail | MA6 |
| C3(a) | multi_choice `[Oak, Teak, Rosewood]` | `["teak", "Oak", "Plastic"]` | `["Oak","Teak"]` (definition order), dropped `["Plastic"]` | MC-A2 |
| C3(b) | multi_choice | `["Plastic"]`; `[]`; `"Teak"` | fail; fail; fail | MA6 |
| C4(a) | text | `"  Oak table "`; `"   "` | canonical `Oak table`; fail | MC-A2 |
| C4(b) | url | `"https://a.example/x"`; `"ftp://a.example"`; `"a.example/x"` | stored; fail; fail | MC-A2 |
| C4(c) | integer | `12`; `"12"`; `12.5`; `"1e3"`; `"1,200"` | `12`; `12`; fail; fail; fail | MC-A2 |
| C4(d) | decimal | `"12.5"`; `12.5`; `"12,5"` | canonical `12.5`; canonical `12.5`; fail | MC-A2 |
| C4(e) | boolean, date, color | `true`; `"2026-02-30"`; `"2026-02-28"`; `"#A1B2C3"`; `"red"` | `true`; fail; stored; `#a1b2c3`; fail | MC-A2 |
| C5(a) | dimension, definition's first unit = the code for `cm` | `{value: 10, unit: "in"}` | value `25.4` in that unit code | MC-A2 |
| C5(b) | dimension, first unit `cm` | `{value: "2.5", unit: "mm"}`; `{value: "3.5", unit: "mm"}` | `0.2`; `0.4` (half-even at one decimal) | MC-A2 |
| C5(c) | weight, first unit `kg` | `{value: 1, unit: "lb"}` | `0.5` | MC-A2 |
| C5(d) | dimension | unit `"ft"`; a `UnitNames` with no entry for the definition's first unit; `{value: 10}` | fail; fail; fail | MA6 |
| C6(a) | `statement`, choice `[Hans J. Wegner]` | stated `Hans J Wegner` | `Mapped("Hans J. Wegner")` | MC-A2 |
| C6(b) | `statement`, same definition | stated `Unknown Maker` | `Unmapped("unknown maker")` | MC-A2 (R-IA24) |
| C6(c) | `page_statement`, single-valued key | a page stating `Wegner` and `Mogensen`; a page stating `Wegner` and `WEGNER` | none; one statement | MC-A2 |
| C7(a) | `same`, text | `Hans J. Wegner` / `hans j wegner`; `Hans J. Wegner` / `Hans Wegner` | same; not same | MC-A1 |
| C7(b) | `same`, choice | `Teak` / `Teak`; `Teak` / `teak` | same; not same (canonical equality) | MC-A1 |
| C7(c) | `same`, multi_choice | `["Oak","Teak"]` / `["Teak","Oak"]`; `["Oak"]` / `["Oak","Teak"]` | same; not same | MC-A1 |
| C7(d) | `same`, dimension | 25.4 mm / 2.54 cm; 25 mm / 2.54 cm | same; not same | MC-A1 |
| C7(e) | `same`, boolean and integer | `true` / `true`; `12` / `13` | same; not same | MC-A1 |

## Delegated choices
The two crates (recorded) · internal number representation for the unit conversion (integers only) · error text
inside `Mapped::Fail` (it is shown in the trace; keep it one short sentence) · whether `tags` gets a `same` arm now
(it is never written by research; the arm exists for the cleared set and compares name keys as sets).

## Manual try-out
Nothing to see in the app yet. To see the rules work:
`cargo test -p psm-core mechanisms::research` — the test names read as the rules ("mc_a2_choice_by_name_key", …).

## Notes
- Hazard: an empty name key (C1(g)) must never match a choice whose key is also empty; `map_output` fails on it.
- A mapped choice is byte-equal to one of the definition's current choices (MA11); the test for C2(b) asserts the
  returned string, not a normalised form.
- Review mode: the reviewer runs `cargo test -p psm-core mechanisms::research`, clippy, fmt; no full suite.

## Review log
