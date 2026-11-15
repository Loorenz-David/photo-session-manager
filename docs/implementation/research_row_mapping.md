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
| A17·C3(d) | R8 | new, from the R8 projection attempt 2 (orchestrator 2026-11-13) |  |
| A17·C7(a) | R8 | new, from the R8 projection (orchestrator 2026-11-13) |  |

## Rows added by R11 and R12 (2026-11-15, master plan v10)

Two new plan numbers, the next free ones: **A25** = `R11_research_on_arranged`, **A26** = `R12_research_cost`. Criteria are numbered once across each plan's units. Tests are named after the row (`a25_c1a_<slug>`, `a26_c1a_<slug>`). Generated from the plan files; counts are computed, not typed.

New rows: A25 99 · A26 37 · together 136. Total now: 396 + the 6 rows added by the R8 projection + 136 = 538.

| Plan number | Now in | Rows per unit |
|---|---|---|
| A25 | R11·U1, R11·U2, R11·U3, R11·U4, R11·U5, R11·U6 | U1 22, U2 12, U3 26, U4 17, U5 9, U6 13 |
| A26 | R12·U1, R12·U2, R12·U3, R12·U4 | U1 17, U2 9, U3 6, U4 5 |

### Rows of earlier plans that R11 and R12 retire or amend

| Row | Was in | What happens | By |
|---|---|---|---|
| A13·C1(a), A13·C1(b) | R4·U4 | amended: seven prerequisites (`AutoOff` removed) | R11·U1 (08B changelog 13 d) |
| A13·C2(a) | R7·U1 | superseded: a photo change queues no run; replaced by A25·C21(a) | R11·U1, R11·U4 |
| A13·C2(b) | R4·U4 | superseded; replaced by A25·C1(b) | R11·U1 |
| A13·C4(a), A13·C4(b) | R4·U4 | superseded: there is no `auto` setting | R11·U1 |
| A13·C5(a), A13·C5(b), A13·C5(c) | R4·U4 | superseded: there is no quiet period; replaced by A25·C4(a) | R11·U1 |
| A13·C6(a), A13·C6(b), A15·C7(a) | R4·U4, R8 | superseded as written (an `auto` run); the same outcomes for an `arranged` run are A25·C4(b), A25·C4(c) | R11·U1 |
| A13·C7(a)–(c), A13·C8(a), A13·C8(b) | R4·U4 | kept; input amended: the run is started by `arranged` (C7(b): "before an earlier `arranged`") | R11·U1 |
| A14·C3(a)–(e), A14·C6(a) | R4·U5 | kept; input amended: the second run is started by a second `arranged`, trigger `arranged` | R11·U1 |
| A14·C5(a), A14·C5(b) | R4·U5 | amended: the trigger set decides whether the next `arranged` creates a run; a photo change alone queues nothing | R11·U1 |
| A19·C4(a) | R4·U6 | superseded: there is no `auto` setting | R11·U1 |
| A19·C5(d) | R6·U2 | superseded: attribution keys are a picker; replaced by A25·C28(d) | R11·U6 |
| A24·C1(a) | R10·U1 | amended: with everything in place the status names no reason (was `AutoOff` with an empty sentence) | R11·U1 |
| A12·C1(a)–(c), A12·C3(a), A12·C3(b) | R4·U3 | kept; input amended: one Observe unit and one `looked` step for the chosen photos | R12·U1 |
| A14·C1(a) | R4·U5 | superseded (a kill between two Observe calls cannot happen); replaced by A26·C5(a) | R12·U1 |
| A14·C3(a), A14·C3(d) | R4·U5 | amended: a new or recomposed chosen photo costs one Observe call over all chosen photos (A26·C4(b)) | R12·U1 |
| A17·C1–C3 (the page text cell) | R8·U3 | amended: `PageReply.text` is the product extract (A26·C9(a)); title, images, redirects and pacing unchanged | R12·U2 |

### Every new row

| Row | Unit | Boundary / test home | Text edit |
|---|---|---|---|
| A25·C1(a) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C1(b) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C1(c) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C1(d) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C1(e) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C1(f) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C1(g) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C1(h) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C1(i) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C1(j) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C2(a) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C2(b) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C2(c) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C3(a) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C3(b) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C4(a) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C4(b) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C4(c) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C4(d) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C5(a) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C6(a) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C7(a) | R11·U1 | research harness (`crates/research/tests/engine/arranged.rs`) | new (v10) |
| A25·C8(a) | R11·U2 | rule unit tests and research harness (`crates/research/tests/engine/settings_r11.rs`) | new (v10) |
| A25·C8(b) | R11·U2 | rule unit tests and research harness (`crates/research/tests/engine/settings_r11.rs`) | new (v10) |
| A25·C8(c) | R11·U2 | rule unit tests and research harness (`crates/research/tests/engine/settings_r11.rs`) | new (v10) |
| A25·C8(d) | R11·U2 | rule unit tests and research harness (`crates/research/tests/engine/settings_r11.rs`) | new (v10) |
| A25·C9(a) | R11·U2 | rule unit tests and research harness (`crates/research/tests/engine/settings_r11.rs`) | new (v10) |
| A25·C9(b) | R11·U2 | rule unit tests and research harness (`crates/research/tests/engine/settings_r11.rs`) | new (v10) |
| A25·C9(c) | R11·U2 | rule unit tests and research harness (`crates/research/tests/engine/settings_r11.rs`) | new (v10) |
| A25·C9(d) | R11·U2 | rule unit tests and research harness (`crates/research/tests/engine/settings_r11.rs`) | new (v10) |
| A25·C9(e) | R11·U2 | rule unit tests and research harness (`crates/research/tests/engine/settings_r11.rs`) | new (v10) |
| A25·C10(a) | R11·U2 | rule unit tests and research harness (`crates/research/tests/engine/settings_r11.rs`) | new (v10) |
| A25·C10(b) | R11·U2 | rule unit tests and research harness (`crates/research/tests/engine/settings_r11.rs`) | new (v10) |
| A25·C11(a) | R11·U2 | rule unit tests and research harness (`crates/research/tests/engine/settings_r11.rs`) | new (v10) |
| A25·C12(a) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C12(b) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C12(c) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C12(d) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C12(e) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C12(f) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C12(g) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C12(h) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C12(i) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C12(j) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C12(k) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C13(a) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C13(b) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C13(c) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C14(a) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C14(b) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C14(c) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C14(d) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C15(a) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C15(b) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C15(c) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C15(d) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C15(e) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C15(f) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C15(g) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C16(a) | R11·U3 | rule unit tests and research harness (`crates/research/tests/engine/derived.rs`) | new (v10) |
| A25·C17(a) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C17(b) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C17(c) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C17(d) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C18(a) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C18(b) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C18(c) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C18(d) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C19(a) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C20(a) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C20(b) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C20(c) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C20(d) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C20(e) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C21(a) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C22(a) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C23(a) | R11·U4 | app suite on the wired app (`crates/testkit/tests/suite/research/r11_arranged.rs`); C23 the desktop crate | new (v10) |
| A25·C24(a) | R11·U5 | UI tests (`apps/desktop/src/features/{shell,sessions,publish,form}/__tests__/r11_*.test.tsx`) | new (v10) |
| A25·C24(b) | R11·U5 | UI tests (`apps/desktop/src/features/{shell,sessions,publish,form}/__tests__/r11_*.test.tsx`) | new (v10) |
| A25·C24(c) | R11·U5 | UI tests (`apps/desktop/src/features/{shell,sessions,publish,form}/__tests__/r11_*.test.tsx`) | new (v10) |
| A25·C24(d) | R11·U5 | UI tests (`apps/desktop/src/features/{shell,sessions,publish,form}/__tests__/r11_*.test.tsx`) | new (v10) |
| A25·C24(e) | R11·U5 | UI tests (`apps/desktop/src/features/{shell,sessions,publish,form}/__tests__/r11_*.test.tsx`) | new (v10) |
| A25·C25(a) | R11·U5 | UI tests (`apps/desktop/src/features/{shell,sessions,publish,form}/__tests__/r11_*.test.tsx`) | new (v10) |
| A25·C26(a) | R11·U5 | UI tests (`apps/desktop/src/features/{shell,sessions,publish,form}/__tests__/r11_*.test.tsx`) | new (v10) |
| A25·C26(b) | R11·U5 | UI tests (`apps/desktop/src/features/{shell,sessions,publish,form}/__tests__/r11_*.test.tsx`) | new (v10) |
| A25·C27(a) | R11·U5 | UI tests (`apps/desktop/src/features/{shell,sessions,publish,form}/__tests__/r11_*.test.tsx`) | new (v10) |
| A25·C28(a) | R11·U6 | UI tests (`apps/desktop/src/features/settings/__tests__/r11_research_pickers.test.tsx`) | new (v10) |
| A25·C28(b) | R11·U6 | UI tests (`apps/desktop/src/features/settings/__tests__/r11_research_pickers.test.tsx`) | new (v10) |
| A25·C28(c) | R11·U6 | UI tests (`apps/desktop/src/features/settings/__tests__/r11_research_pickers.test.tsx`) | new (v10) |
| A25·C28(d) | R11·U6 | UI tests (`apps/desktop/src/features/settings/__tests__/r11_research_pickers.test.tsx`) | new (v10) |
| A25·C28(e) | R11·U6 | UI tests (`apps/desktop/src/features/settings/__tests__/r11_research_pickers.test.tsx`) | new (v10) |
| A25·C29(a) | R11·U6 | UI tests (`apps/desktop/src/features/settings/__tests__/r11_research_pickers.test.tsx`) | new (v10) |
| A25·C30(a) | R11·U6 | UI tests (`apps/desktop/src/features/settings/__tests__/r11_research_pickers.test.tsx`) | new (v10) |
| A25·C30(b) | R11·U6 | UI tests (`apps/desktop/src/features/settings/__tests__/r11_research_pickers.test.tsx`) | new (v10) |
| A25·C31(a) | R11·U6 | UI tests (`apps/desktop/src/features/settings/__tests__/r11_research_pickers.test.tsx`) | new (v10) |
| A25·C31(b) | R11·U6 | UI tests (`apps/desktop/src/features/settings/__tests__/r11_research_pickers.test.tsx`) | new (v10) |
| A25·C32(a) | R11·U6 | UI tests (`apps/desktop/src/features/settings/__tests__/r11_research_pickers.test.tsx`) | new (v10) |
| A25·C32(b) | R11·U6 | UI tests (`apps/desktop/src/features/settings/__tests__/r11_research_pickers.test.tsx`) | new (v10) |
| A25·C33(a) | R11·U6 | UI tests (`apps/desktop/src/features/settings/__tests__/r11_research_pickers.test.tsx`) | new (v10) |
| A26·C1(a) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C1(b) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C1(c) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C1(d) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C2(a) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C2(b) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C3(a) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C3(b) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C3(c) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C4(a) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C4(b) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C4(c) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C5(a) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C6(a) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C6(b) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C7(a) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C7(b) | R12·U1 | rule unit tests and research harness (`crates/research/tests/engine/photo_look.rs`); C7 `adapter_openai.rs` | new (v10) |
| A26·C8(a) | R12·U2 | adapter unit tests and `crates/research/tests/engine/adapter_pages.rs` | new (v10) |
| A26·C8(b) | R12·U2 | adapter unit tests and `crates/research/tests/engine/adapter_pages.rs` | new (v10) |
| A26·C8(c) | R12·U2 | adapter unit tests and `crates/research/tests/engine/adapter_pages.rs` | new (v10) |
| A26·C8(d) | R12·U2 | adapter unit tests and `crates/research/tests/engine/adapter_pages.rs` | new (v10) |
| A26·C8(e) | R12·U2 | adapter unit tests and `crates/research/tests/engine/adapter_pages.rs` | new (v10) |
| A26·C8(f) | R12·U2 | adapter unit tests and `crates/research/tests/engine/adapter_pages.rs` | new (v10) |
| A26·C8(g) | R12·U2 | adapter unit tests and `crates/research/tests/engine/adapter_pages.rs` | new (v10) |
| A26·C8(h) | R12·U2 | adapter unit tests and `crates/research/tests/engine/adapter_pages.rs` | new (v10) |
| A26·C9(a) | R12·U2 | adapter unit tests and `crates/research/tests/engine/adapter_pages.rs` | new (v10) |
| A26·C10(a) | R12·U3 | research harness (`crates/research/tests/engine/same_kind.rs`); C12 `adapter_openai.rs` | new (v10) |
| A26·C10(b) | R12·U3 | research harness (`crates/research/tests/engine/same_kind.rs`); C12 `adapter_openai.rs` | new (v10) |
| A26·C10(c) | R12·U3 | research harness (`crates/research/tests/engine/same_kind.rs`); C12 `adapter_openai.rs` | new (v10) |
| A26·C10(d) | R12·U3 | research harness (`crates/research/tests/engine/same_kind.rs`); C12 `adapter_openai.rs` | new (v10) |
| A26·C11(a) | R12·U3 | research harness (`crates/research/tests/engine/same_kind.rs`); C12 `adapter_openai.rs` | new (v10) |
| A26·C12(a) | R12·U3 | research harness (`crates/research/tests/engine/same_kind.rs`); C12 `adapter_openai.rs` | new (v10) |
| A26·C13(a) | R12·U4 | tool crate (`crates/research-tool/tests/tool/u4_accept.rs`); `[manual]` rows recorded evidence | new (v10) |
| A26·C13(b) | R12·U4 | tool crate (`crates/research-tool/tests/tool/u4_accept.rs`); `[manual]` rows recorded evidence | new (v10) |
| A26·C14(a) | R12·U4 | recorded evidence (`[manual]`) | new (v10) |
| A26·C15(a) | R12·U4 | recorded evidence (`[manual]`) | new (v10) |
| A26·C16(a) | R12·U4 | recorded evidence (`[manual]`) | new (v10) |
