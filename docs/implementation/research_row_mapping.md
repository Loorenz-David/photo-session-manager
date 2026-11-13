# Research plan: where every criterion row went (2026-11-12, restructure into milestones)

The 24 phase plans `A01`–`A24` (commit `d116654`) were regrouped into 10 milestone plans `R1`–`R10` on the owner's decision of 2026-11-12 (`docs/process/flow.md` §10). **No row was dropped and no outcome was weakened.** A row keeps its old id as its permanent row id (`A07·C1(a)` = plan A07, criterion C1, row a); tests are named after it. Three rows are new (`F·…`, the freeze rows of R2·U2). Generated from the plan files; counts are computed, not typed.

Old rows: 393 · placed: 393 · new rows: 3 · total now: 396.

| What happened | Rows |
|---|---|
| boundary unchanged | 79 |
| pure rule, same boundary, moved from core to the research crate | 107 |
| boundary or test home moved to the research crate | 164 |
| test home moved to the tool crate | 26 |
| boundary changed inside the app | 6 |
| `[manual]` evidence rows, unchanged | 11 |
| row text edited (boundary or input cell; see the last column) | 8 |

## Old plan → units

| Old plan | Now in |
|---|---|
| A01 | R1·U1 |
| A02 | R1·U2 |
| A03 | R2·U1, R3·U1 |
| A04 | R3·U2 |
| A05 | R3·U3 |
| A06 | R3·U4 |
| A07 | R2·U2, R3·U5 |
| A08 | R5·U1 |
| A09 | R2·U2, R5·U2 |
| A10 | R2·U3, R4·U1 |
| A11 | R4·U1, R4·U2 |
| A12 | R4·U3 |
| A13 | R4·U4, R7·U1 |
| A14 | R4·U5 |
| A15 | R8·U1 |
| A16 | R8·U2 |
| A17 | R8·U3 |
| A18 | R4·U6, R6·U1 |
| A19 | R4·U6, R6·U2 |
| A20 | R6·U3 |
| A21 | R7·U2 |
| A22 | R9·U1 |
| A23 | R9·U2 |
| A24 | R10·U1 |

## New rows

| Row | Unit | Why |
|---|---|---|
| F·C1(a) | R2·U2 | the shapes frozen by R2 need a caller and an observable outcome in the unit that adds them |
| F·C2(a) | R2·U2 | the shapes frozen by R2 need a caller and an observable outcome in the unit that adds them |
| F·C3(a) | R2·U2 | the shapes frozen by R2 need a caller and an observable outcome in the unit that adds them |

## Every row

| Row | Unit | Boundary now (and before) | Text edit |
|---|---|---|---|
| A01·C1(a) | R1·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A01·C1(b) | R1·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A01·C1(c) | R1·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A01·C2(a) | R1·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A01·C2(b) | R1·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A01·C2(c) | R1·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A01·C3(a) | R1·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A01·C3(b) | R1·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A01·C4(a) | R1·U1 | recorded evidence, as before |  |
| A01·C5(a) | R1·U1 | recorded evidence, as before |  |
| A01·C6(a) | R1·U1 | recorded evidence, as before |  |
| A01·C7(a) | R1·U1 | recorded evidence, as before |  |
| A01·C8(a) | R1·U1 | recorded evidence, as before |  |
| A02·C1(a) | R1·U2 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A02·C1(b) | R1·U2 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A02·C1(c) | R1·U2 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A02·C1(d) | R1·U2 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A02·C1(e) | R1·U2 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A02·C2(a) | R1·U2 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A02·C3(a) | R1·U2 | recorded evidence, as before |  |
| A02·C4(a) | R1·U2 | recorded evidence, as before |  |
| A02·C5(a) | R1·U2 | recorded evidence, as before |  |
| A03·C1(a) | R2·U1 | pure function, now `psm_core::mechanisms::{name_key, sameness}` (was planned as `psm_core::mechanisms::research`) |  |
| A03·C1(b) | R2·U1 | pure function, now `psm_core::mechanisms::{name_key, sameness}` (was planned as `psm_core::mechanisms::research`) |  |
| A03·C1(c) | R2·U1 | pure function, now `psm_core::mechanisms::{name_key, sameness}` (was planned as `psm_core::mechanisms::research`) |  |
| A03·C1(d) | R2·U1 | pure function, now `psm_core::mechanisms::{name_key, sameness}` (was planned as `psm_core::mechanisms::research`) |  |
| A03·C1(e) | R2·U1 | pure function, now `psm_core::mechanisms::{name_key, sameness}` (was planned as `psm_core::mechanisms::research`) |  |
| A03·C1(f) | R2·U1 | pure function, now `psm_core::mechanisms::{name_key, sameness}` (was planned as `psm_core::mechanisms::research`) |  |
| A03·C1(g) | R2·U1 | pure function, now `psm_core::mechanisms::{name_key, sameness}` (was planned as `psm_core::mechanisms::research`) |  |
| A03·C2(a) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C2(b) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C2(c) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C2(d) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C3(a) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C3(b) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C4(a) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C4(b) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C4(c) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C4(d) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C4(e) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C5(a) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C5(b) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C5(c) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C5(d) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate | input reworded: unit names come from the definition (`validations.unit_names`), not a `UnitNames` argument; outcome unchanged |
| A03·C6(a) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C6(b) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C6(c) | R3·U1 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A03·C7(a) | R2·U1 | pure function, now `psm_core::mechanisms::{name_key, sameness}` (was planned as `psm_core::mechanisms::research`) |  |
| A03·C7(b) | R2·U1 | pure function, now `psm_core::mechanisms::{name_key, sameness}` (was planned as `psm_core::mechanisms::research`) |  |
| A03·C7(c) | R2·U1 | pure function, now `psm_core::mechanisms::{name_key, sameness}` (was planned as `psm_core::mechanisms::research`) |  |
| A03·C7(d) | R2·U1 | pure function, now `psm_core::mechanisms::{name_key, sameness}` (was planned as `psm_core::mechanisms::research`) |  |
| A03·C7(e) | R2·U1 | pure function, now `psm_core::mechanisms::{name_key, sameness}` (was planned as `psm_core::mechanisms::research`) |  |
| A04·C1(a) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C1(b) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C1(c) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C1(d) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C2(a) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C2(b) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C2(c) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C2(d) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C2(e) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C3(a) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C3(b) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C3(c) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C3(d) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C4(a) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C4(b) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C4(c) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C4(d) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C5(a) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C5(b) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C5(c) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C6(a) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C6(b) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C6(c) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C6(d) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C7(a) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C7(b) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C7(c) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A04·C7(d) | R3·U2 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C1(a) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C1(b) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C1(c) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C1(d) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C1(e) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C1(f) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C1(g) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C1(h) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C1(i) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C1(j) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C1(k) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C1(l) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C1(m) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C1(n) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C2(a) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C2(b) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C2(c) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C3(a) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C3(b) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C3(c) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C3(d) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C3(e) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C3(f) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C3(g) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C4(a) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C4(b) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C4(c) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C4(d) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C5(a) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C6(a) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C6(b) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A05·C6(c) | R3·U3 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C1(a) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C1(b) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C2(a) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C2(b) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C2(c) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C2(d) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C2(e) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C2(f) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C2(g) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C3(a) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C3(b) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C4(a) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C4(b) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C4(c) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C5(a) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C5(b) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C5(c) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C5(d) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C5(e) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C5(f) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C5(g) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C6(a) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C6(b) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C6(c) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C6(d) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C6(e) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C7(a) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C7(b) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A06·C7(c) | R3·U4 | pure function, same boundary; lives in `psm_research::rules` (was `psm_core::mechanisms::research`), inline test in the research crate |  |
| A07·C1(a) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C1(b) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C1(c) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C1(d) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C1(e) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C1(f) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C1(g) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C1(h) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C1(i) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C1(j) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C1(k) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C2(a) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C2(b) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C2(c) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C2(d) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C3(a) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C3(b) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C3(c) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C3(d) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C4(a) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C4(b) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C5(a) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C5(b) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C6(a) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C6(b) | R3·U5 | `psm_research::writes::{suggest, lower}` on the research harness (was `AttributesService::ai_suggest` in the testkit suite) |  |
| A07·C7(a) | R2·U2 | app database open: an idempotent schema step instead of a numbered migration | input and outcome reworded: an idempotent schema step replaces the numbered migration; outcome adds "the second open changes nothing; the migration list is not longer" |
| A08·C1(a) | R5·U1 | app suite / UI test, as before |  |
| A08·C1(b) | R5·U1 | app suite / UI test, as before |  |
| A08·C1(c) | R5·U1 | app suite / UI test, as before |  |
| A08·C1(d) | R5·U1 | app suite / UI test, as before |  |
| A08·C1(e) | R5·U1 | app suite / UI test, as before |  |
| A08·C1(f) | R5·U1 | app suite / UI test, as before |  |
| A08·C2(a) | R5·U1 | app suite / UI test, as before |  |
| A08·C3(a) | R5·U1 | app suite / UI test, as before |  |
| A08·C3(b) | R5·U1 | app suite / UI test, as before |  |
| A08·C3(c) | R5·U1 | app suite / UI test, as before |  |
| A08·C3(d) | R5·U1 | app suite / UI test, as before |  |
| A08·C4(a) | R5·U1 | app suite / UI test, as before |  |
| A08·C4(b) | R5·U1 | app suite / UI test, as before |  |
| A08·C4(c) | R5·U1 | app suite / UI test, as before |  |
| A08·C5(a) | R5·U1 | app suite / UI test, as before |  |
| A08·C5(b) | R5·U1 | app suite / UI test, as before |  |
| A09·C1(a) | R5·U2 | app suite / UI test, as before |  |
| A09·C1(b) | R5·U2 | app suite / UI test, as before |  |
| A09·C1(c) | R5·U2 | app suite / UI test, as before |  |
| A09·C1(d) | R5·U2 | app suite / UI test, as before |  |
| A09·C1(e) | R5·U2 | app suite / UI test, as before |  |
| A09·C2(a) | R5·U2 | app suite / UI test, as before |  |
| A09·C2(b) | R5·U2 | app suite / UI test, as before |  |
| A09·C3(a) | R5·U2 | app suite / UI test, as before |  |
| A09·C3(b) | R5·U2 | app suite / UI test, as before |  |
| A09·C4(a) | R5·U2 | app suite / UI test, as before |  |
| A09·C4(b) | R5·U2 | app suite / UI test, as before |  |
| A09·C4(c) | R5·U2 | app suite / UI test, as before |  |
| A09·C4(d) | R5·U2 | app suite / UI test, as before |  |
| A09·C5(a) | R2·U2 | app suite / UI test, as before |  |
| A09·C5(b) | R2·U2 | app suite / UI test, as before |  |
| A09·C6(a) | R5·U2 | app suite / UI test, as before |  |
| A09·C6(b) | R5·U2 | app suite / UI test, as before |  |
| A09·C6(c) | R5·U2 | app suite / UI test, as before |  |
| A09·C7(a) | R5·U2 | app suite / UI test, as before |  |
| A09·C8(a) | R5·U2 | app suite / UI test, as before |  |
| A10·C1(a) | R2·U3 | `Imaging::render_research` (infra) directly (was through `ResearchImages::render`); app suite |  |
| A10·C1(b) | R2·U3 | `Imaging::render_research` (infra) directly (was through `ResearchImages::render`); app suite |  |
| A10·C1(c) | R2·U3 | `Imaging::render_research` (infra) directly (was through `ResearchImages::render`); app suite |  |
| A10·C1(d) | R2·U3 | `Imaging::render_research` (infra) directly (was through `ResearchImages::render`); app suite |  |
| A10·C2(a) | R2·U3 | `Imaging::render_research` (infra) directly (was through `ResearchImages::render`); app suite |  |
| A10·C3(a) | R4·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A10·C3(b) | R4·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A10·C3(c) | R4·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A10·C3(d) | R4·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A10·C3(e) | R4·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A10·C4(a) | R4·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A10·C4(b) | R4·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A10·C5(a) | R4·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C1(a) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C1(b) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C2(a) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C2(b) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C2(c) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C3(a) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C3(b) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C4(a) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C4(b) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C5(a) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C5(b) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C5(c) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C5(d) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C5(e) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C6(a) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C6(b) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C7(a) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C7(b) | R4·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A11·C8(a) | R4·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C1(a) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C1(b) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C1(c) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C2(a) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C2(b) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C2(c) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C2(d) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C3(a) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C3(b) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C3(c) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C4(a) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C4(b) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C4(c) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C4(d) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C4(e) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C4(f) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C5(a) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C5(b) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C5(c) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C6(a) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C6(b) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C7(a) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C7(b) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A12·C8(a) | R4·U3 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A13·C1(a) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C1(b) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C1(c) | R7·U1 | the wired app (`TestApp::with_research()`), app suite, as before |  |
| A13·C1(d) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C2(a) | R7·U1 | the wired app (`TestApp::with_research()`), app suite, as before |  |
| A13·C2(b) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C2(c) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C3(a) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C3(b) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C3(c) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C4(a) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C4(b) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C5(a) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C5(b) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C5(c) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C6(a) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C6(b) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C7(a) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C7(b) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C7(c) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C8(a) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C8(b) | R4·U4 | research harness; photo changes followed by an explicit `session_changed` call (was the wired `TestApp`, where the app made that call) |  |
| A13·C8(c) | R7·U1 | the wired app (`TestApp::with_research()`), app suite, as before |  |
| A13·C8(d) | R7·U1 | the wired app (`TestApp::with_research()`), app suite, as before |  |
| A14·C1(a) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C1(b) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C1(c) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C1(d) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C1(e) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C1(f) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C2(a) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C3(a) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C3(b) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C3(c) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C3(d) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C3(e) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C4(a) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C5(a) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C5(b) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C6(a) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A14·C6(b) | R4·U5 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A15·C1(a) | R8·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A15·C1(b) | R8·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A15·C1(c) | R8·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A15·C2(a) | R8·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A15·C2(b) | R8·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A15·C2(c) | R8·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A15·C3(a) | R8·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A15·C3(b) | R8·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A15·C4(a) | R8·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A15·C4(b) | R8·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A15·C5(a) | R8·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A15·C5(b) | R8·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A15·C5(c) | R8·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A15·C6(a) | R8·U1 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A16·C1(a) | R8·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A16·C1(b) | R8·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A16·C2(a) | R8·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A16·C3(a) | R8·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A16·C4(a) | R8·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A16·C5(a) | R8·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A16·C6(a) | R8·U2 | same service or port boundary; test in `crates/research/tests/engine/` on the research harness (was the testkit suite) |  |
| A17·C1(a) | R8·U3 | same port boundary; research-crate test (was the testkit suite) |  |
| A17·C1(b) | R8·U3 | same port boundary; research-crate test (was the testkit suite) |  |
| A17·C2(a) | R8·U3 | same port boundary; research-crate test (was the testkit suite) |  |
| A17·C3(a) | R8·U3 | same port boundary; research-crate test (was the testkit suite) |  |
| A17·C4(a) | R8·U3 | app suite (`FakeTransport`), as before |  |
| A17·C4(b) | R8·U3 | app suite (`FakeTransport`), as before |  |
| A17·C5(a) | R8·U3 | app suite (`FakeTransport`), as before | input reworded: unit names arrive in the imported definition instead of a `unit_names()` port method; outcome unchanged |
| A17·C6(a) | R8·U3 | app suite (`FakeTransport`), as before |  |
| A18·C1(a) | R4·U6 | `ResearchApi` method on the research harness (was the service through a Tauri command, testkit suite) | boundary cell: `ResearchApi::status` instead of the service through a command; outcome unchanged |
| A18·C1(b) | R4·U6 | `ResearchApi` method on the research harness (was the service through a Tauri command, testkit suite) |  |
| A18·C1(c) | R4·U6 | `ResearchApi` method on the research harness (was the service through a Tauri command, testkit suite) |  |
| A18·C2(a) | R6·U1 | UI test, as before |  |
| A18·C2(b) | R6·U1 | UI test, as before |  |
| A18·C2(c) | R6·U1 | UI test, as before |  |
| A18·C2(d) | R6·U1 | UI test, as before |  |
| A18·C3(a) | R6·U1 | UI test, as before |  |
| A18·C3(b) | R6·U1 | UI test, as before |  |
| A18·C4(a) | R6·U1 | UI test, as before |  |
| A18·C5(a) | R6·U1 | UI test, as before |  |
| A18·C6(a) | R4·U6 | `ResearchApi` method on the research harness (was the service through a Tauri command, testkit suite) | boundary cell: `ResearchApi::run` instead of the Tauri command; outcome unchanged |
| A19·C1(a) | R4·U6 | `ResearchApi` method on the research harness (was a Tauri command, testkit suite) | boundary cell: `ResearchApi::secrets_set` / `secrets_state` instead of the Tauri commands; outcome unchanged |
| A19·C1(b) | R4·U6 | `ResearchApi` method on the research harness (was a Tauri command, testkit suite) |  |
| A19·C2(a) | R4·U6 | `ResearchApi` method on the research harness (was a Tauri command, testkit suite) | boundary cell: `ResearchApi::site_add` instead of the Tauri command; outcome unchanged |
| A19·C2(b) | R4·U6 | `ResearchApi` method on the research harness (was a Tauri command, testkit suite) |  |
| A19·C2(c) | R4·U6 | `ResearchApi` method on the research harness (was a Tauri command, testkit suite) |  |
| A19·C3(a) | R4·U6 | `ResearchApi` method on the research harness (was a Tauri command, testkit suite) | boundary cell: `ResearchApi::validate_settings`; outcome unchanged |
| A19·C4(a) | R4·U6 | `ResearchApi` method on the research harness (was a Tauri command, testkit suite) |  |
| A19·C5(a) | R6·U2 | UI test, as before |  |
| A19·C5(b) | R6·U2 | UI test, as before |  |
| A19·C5(c) | R6·U2 | UI test, as before |  |
| A19·C5(d) | R6·U2 | UI test, as before |  |
| A20·C1(a) | R6·U3 | app suite / UI test, as before |  |
| A20·C2(a) | R6·U3 | app suite / UI test, as before |  |
| A20·C2(b) | R6·U3 | app suite / UI test, as before |  |
| A20·C3(a) | R6·U3 | app suite / UI test, as before |  |
| A20·C3(b) | R6·U3 | app suite / UI test, as before |  |
| A20·C3(c) | R6·U3 | app suite / UI test, as before |  |
| A20·C4(a) | R6·U3 | app suite / UI test, as before |  |
| A20·C5(a) | R6·U3 | app suite / UI test, as before |  |
| A20·C5(b) | R6·U3 | app suite / UI test, as before |  |
| A21·C1(a) | R7·U2 | same service boundary; research-crate test (was the testkit suite) |  |
| A21·C1(b) | R7·U2 | same service boundary; research-crate test (was the testkit suite) |  |
| A21·C1(c) | R7·U2 | same service boundary; research-crate test (was the testkit suite) |  |
| A21·C1(d) | R7·U2 | the wired app / UI test, as before |  |
| A21·C2(a) | R7·U2 | same service boundary; research-crate test (was the testkit suite) |  |
| A21·C2(b) | R7·U2 | same service boundary; research-crate test (was the testkit suite) |  |
| A21·C3(a) | R7·U2 | same service boundary; research-crate test (was the testkit suite) |  |
| A21·C3(b) | R7·U2 | same service boundary; research-crate test (was the testkit suite) |  |
| A21·C3(c) | R7·U2 | same service boundary; research-crate test (was the testkit suite) |  |
| A21·C4(a) | R7·U2 | same service boundary; research-crate test (was the testkit suite) |  |
| A21·C5(a) | R7·U2 | same service boundary; research-crate test (was the testkit suite) |  |
| A21·C6(a) | R7·U2 | same service boundary; research-crate test (was the testkit suite) |  |
| A21·C7(a) | R7·U2 | the wired app / UI test, as before |  |
| A22·C1(a) | R9·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A22·C1(b) | R9·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A22·C2(a) | R9·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A22·C2(b) | R9·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A22·C2(c) | R9·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A22·C3(a) | R9·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A22·C4(a) | R9·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A22·C4(b) | R9·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A22·C5(a) | R9·U1 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A23·C1(a) | R9·U2 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A23·C1(b) | R9·U2 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A23·C2(a) | R9·U2 | same tool boundary; test in `crates/research-tool/tests/tool/` (was the testkit suite) |  |
| A23·C3(a) | R9·U2 | recorded evidence, as before |  |
| A23·C4(a) | R9·U2 | recorded evidence, as before |  |
| A23·C5(a) | R9·U2 | recorded evidence, as before |  |
| A24·C1(a) | R10·U1 | research harness with `ResearchSettings::default()` from `psm_research::config` (was `psm_infra::settings_file` and the wired `TestApp`) |  |
| A24·C1(b) | R10·U1 | research harness with `ResearchSettings::default()` from `psm_research::config` (was `psm_infra::settings_file` and the wired `TestApp`) |  |
| A24·C2(a) | R10·U1 | research harness with `ResearchSettings::default()` from `psm_research::config` (was `psm_infra::settings_file` and the wired `TestApp`) |  |
| A24·C2(b) | R10·U1 | research harness with `ResearchSettings::default()` from `psm_research::config` (was `psm_infra::settings_file` and the wired `TestApp`) |  |
| A15·C5(d) | R8 | new, from the R8 projection (orchestrator 2026-11-13) |  |
| A15·C7(a) | R8 | new, from the R8 projection (orchestrator 2026-11-13) |  |
| A17·C3(b) | R8 | new, from the R8 projection (orchestrator 2026-11-13) |  |
| A17·C3(c) | R8 | new, from the R8 projection (orchestrator 2026-11-13) |  |
| A17·C7(a) | R8 | new, from the R8 projection (orchestrator 2026-11-13) |  |
