---
plan: P03_session_state_list
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-10-25
actor: opus
---
# P03 review, round 1: CHANGES_REQUESTED

Production code is correct on every row. One test (C1(l)) cannot fail when the latest publish item is chosen wrongly, which is B2 under flow §5. The fix is one added assertion in the P03 test file. No production change is needed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
- **Gate.** Tracker `IMPLEMENTED`; checkpoint `203a3de` exists. `git show --stat 203a3de` lists 16 paths, equal to the handoff's declared list. The 4 paths beyond the plan's list (`Cargo.lock`, `crates/testkit/Cargo.toml`, `apps/desktop/src-tauri/src/lib.rs`, `crates/core/src/mechanisms/mod.rs`) are declared deviations, all needed for the planned tests or registration.
- **Stamp** (tree `a622771`, `git status --porcelain` empty): `cargo test --workspace` exit 0 (P03 28 + desktop `c5a` 1, P02 25, P01 20, P00 16, unit tests); `cargo clippy --workspace --all-targets -- -D warnings` 0; `cargo fmt --all -- --check` 0; `npm run typecheck` 0; `npm test` 0 (no frontend test files).
- **MC-7 status** (`mechanisms/status.rs::derive_status`): rules 1–5 in order, first match wins. Publishing covers prepare/upload/update/describe, failed only when `published_at` is null, published = `published_at` set and not dirty, ready = main > 0 and unreviewed 0 and no non-deleted photo without `adjusted_preview_path` (SQL `classification<>'deleted' AND adjusted_preview_path IS NULL`), else in progress. `raw_state` staleness is not consulted, so a stale preview counts as present (MC-17).
- **MC-7 label and progress**: precedence Publishing n% → Publish failed (failed at any non-describe step, published before or not, R-I7) → Description failed → Published → Ready → InProgress(unreviewed). Upload progress = 10 + ⌊65·uploaded/total⌋ (2 of 4 → 42), prepare 0, update 75, describe 85; `progress` is `Some` only for publishing rows.
- **Latest publish item** (`sqlite/store.rs::session_list_records`): confirmed runs only, `ORDER BY pr.confirmed_at DESC, pr.id DESC`, matching MC-7's tie rule.
- **MC-4 grouping** (`mechanisms/time.rs::group_by_activity`): calendar dates in the passed zone (no 24 h subtraction); `IsoWeek` comparison (Monday start, year-qualified); Today and Yesterday checked before the week tests; month groups `%B %Y`, newest first because rows are sorted before grouping; within a group `last_activity_at` desc then id desc. The service passes `chrono::Local`.
- **List** (`services/sessions.rs::list`): Unassigned (id 1) excluded in SQL, so it never forms a row (this settles the P01 backlog note about a "January 1970" group). Open = every status except published; Published = published only; search is a case-insensitive substring on title or SKU; counts are taken before search (declared assumption).
- **Tests.** C1(a)–C1(k), C2(a)–C2(f), C3(a)–C3(f), C4(a)–C4(d) in `crates/testkit/tests/p03_session_state_list.rs` and C5(a) in `commands/sessions.rs::list_tests` each assert the row's exact outcome at the row's boundary and would fail on a wrong one. Examples: C1(a) sets `published_at` so a wrong rule order fails; C1(c) separates rule 2 from rule 3; C3(c)/C3(d) separate Monday-start weeks from Sunday-start; C3 runs both 00:30 and 23:30 on the 25-hour DST day; C4(c) includes a SKU-only match; C5(a) uses distinct values per field, so a swapped mapping fails. No test fails the over-engineering test: they assert outcomes, not internals.
- **Assumptions.** None contradicts the intention. Excluding unconfirmed runs follows MC-7 ("greatest `confirmed_at`") and H1. Counts before search, progress only while publishing, and the cover choice are within the plan's delegated choices or ch07/design.
- **H1–H8.** Read-only phase: no external writes (H1); no file I/O or pixel work (H2, H3); no provider-specific names in core (H7); `sessions_list` is lock → `SessionsService::list` → DTO mapping (H8). H4–H6 not touched. No secrets, no live services.

## Findings

### F1: blocking (B2), route: production
- **What is wrong.** `c1l_latest_confirmed_wins` cannot fail on a wrong latest-item choice. With `published_at` set and not dirty, status is `published` whether the latest item is the newer `done` or the older `failed` one (MC-7 rule 2 needs `published_at` null, so rule 3 applies either way). No other test discriminates the ordering. Changed-input probe: a scratch test with the confirmation times reversed (failed newer than done) gave `status=Published label=PublishFailed`, so the existing assertion holds under both orderings. The scratch file was deleted and the tree is clean.
- **Authority.** Plan row C1(l), "`published` (latest by `confirmed_at`)", trace MC-7 (latest item); flow §5 B2.
- **Correction.** In `c1l_latest_confirmed_wins`, also assert the row's `label == RowLabel::Published`. A wrong latest-item choice gives `PublishFailed`. Keep the row's input as written.
- **Files the fix may touch.** `crates/testkit/tests/p03_session_state_list.rs` only.

## Notes written to backlog
- P03 → P07: `unassigned_count` counts deleted photos too, so the Unassigned card could show for a bucket holding only hidden photos.
- P03 → P07: the row DTO has flat `main_count`/`condition_count`/`unreviewed_count`, not a nested `counts{…}`. P07 builds against the binding as generated.
- P03: about four queries per session in `session_list_records`. Fine at MVP scale.
- P03: a `last_activity_at` later than now (clock moved back) never groups as Today.

## Lessons for the plans
- C1(l)'s input (`published_at` set, not dirty) makes the status the same for both candidate items, so the row cannot prove the selection rule it traces to. A row that proves a selection rule needs an input where the two candidates give different outcomes at the asserted boundary, for example an unpublished session or an assertion on the label.

## Write perimeter
- `docs/implementation/handoffs/reviewer/handoff_P03_session_state_list_review_r1.md` (this file, new).
- `docs/implementation/plans/P03_session_state_list.md`: Review log entry appended.
- `docs/implementation/master_plan.md`: P03 tracker row → `CHANGES_REQUESTED`.
- `docs/implementation/backlog.md`: 4 rows appended.
- Probe: `crates/testkit/tests/zz_review_probe_p03.rs` was created, run and deleted (an untracked scratch test, no production file touched). Build artefacts under `target/` only. No database or state outside temp dirs.
- No commits.
