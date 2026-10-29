---
plan: P21_backend_robustness
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-29
actor: opus
---
# Review P21_backend_robustness, round 1: APPROVED_WITH_NOTES

The gate held: the tracker showed `IMPLEMENTED` and checkpoint `e33ddfb` is in `git log`. All 14 rows hold at their boundaries. No finding meets B1–B4. There are four notes, written to the backlog. The tracker row is now `APPROVED`, the Review log entry is appended, and the owner narrative is written.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
- **Perimeter.** `git show --stat e33ddfb` lists 17 paths. They equal the handoff's declared files. That is the plan's list minus the optional `commands/settings.rs` (no DTO defaults were needed, because `settings_get` normalises the block), plus `crates/infra/src/paths.rs`, which the handoff disclosed under Deviations. `bindings.ts` gained only the doc comment. No signature changed.
- **Stamp, re-run by the reviewer** on the clean tree at `6c5e3c4` (code = `e33ddfb`, `git status --porcelain` empty):

  | Check | Result |
  |---|---|
  | `cargo test --workspace` | 421 passed, 0 failed |
  | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
  | `cargo fmt --all -- --check` | exit 0 |
  | `npm run typecheck` (Node 22) | exit 0 |
  | `npm test` | 12 files, 114 passed |

- **C1(a).** `Settings::open_lenient` creates the file only when it is absent. On a parse error it returns `Settings::default()` plus the parser message, and it never writes. `App::build` sets `settings_invalid` with detail `settings.json: <msg>`. The test checks the file's bytes after the restart and after the health check. Its `preview_delay_s` save is a runtime save: `prepare` writes a complete file, and the rebuilt app's fresh `HealthService` has no item. The test fails on pre-phase code, where `kill_and_restart` returned `Err`.
- **C1(b).** `integration_settings()` returning `Err` sets commerce and publish to `None`, and a test override is also refused when the block is invalid. The detail reads `integration: integration settings: missing field \`client_id\``. `settings_get` writes `integration: null`. The test asserts the `integration: ` prefix and the `client_id` substring, both Export-mode `None`s, and that `document_id` was applied.
- **C1(c).** A start-backup error is captured, `requeue_running_jobs` and the publish re-enqueue still run, and `backup_failed` is set. `PublishJob` now sets or clears `backup_failed` instead of returning `Err`. The test's start half fails on pre-phase code. Its publish half cannot fail on its own (note 2). Reviewer probe 1 covered that half against the real production code: a backup that worked at start and broke before publishing gave job `done` at `attempts 0` and `backup_failed` set.
- **C2(a).** `AppState { app: RwLock<App>, save: Mutex<()> }`. The save guard is held from the context read through `prepare`, `apply_settings` and `old.shutdown()`. Each save's context is built from `self.settings` after the previous save applied, so no save starts from a stale snapshot. The test is judged by reading, as the plan directs. It asserts all four keys in both `settings_get` and the file.
- **C2(b).** The stored secret is kept only when `secret == "••••" + the last four chars of the stored secret`, using the same char-based tail as `SettingsService::get`, so it is safe for multibyte characters. The three later saves are asserted exactly.
- **C2(c).** `settings_get` serialises the block through `IntegrationSettings`, which fills the serde defaults, and only then masks the secret. All four values and `commerce.is_some()` are asserted.
- **C3(a).** A runner with no handler returns `Ok(0)` before `job_due`, so nothing is claimed. Pre-phase code would claim the job and return `Err`, and a variant without the guard would return `Ok(1)`. The test asserts `queued` with attempts 0.
- **C3(b).** An unknown kind goes through `job_retry(id, "unknown_job_kind", now + 60_000, now)`, is counted, and the loop continues. `Ok(2)`, the exact row state and the `hash` row's `done` are asserted.
- **C4(a).** `adapter.item_find(query)?`. `Error::Provider` maps to code `provider` (`commands/items.rs:67`). `reads == ["item_find"]` proves that `search_local` never touches the adapter. The inherited `p13 c9c` was updated to the new semantics. No command calls the merged `search` (note 4).
- **C5(a).** The text is exactly `Published`, the condition suffix is unchanged, and `created_at` is bounded by the confirmation time and the current time. `p13:589` follows the new text.
- **C5(b).** The compare is now against `commerce.render_description(local)`. The Shopify adapter returns `description_html`, and the fake returns the same function. Exactly one `update_description` happens across two runs. The pre-phase compare of plain text against HTML would send it twice.
- **C6(a).** `barcode.map(trim).or(old).filter(non-empty)`: `Some("")` stores `NULL`, `None` keeps the old value, `Some("123")` stores it, and the SKU is unchanged.
- **C7(a)/(b).** After the token check, `check_integration` refreshes the schema, then reads the stored definitions. "Refresh failed and nothing stored" gives the exact `unknown: shop fields not read yet` detail. Stored definitions still count after a failed refresh. `p16 c3a` keeps `not found` (suite green). The refresh duration is logged.
- **Hard constraints H1–H8.**
  - H1: the health refresh is a read, and the description compare removes writes.
  - H2/H3/H5: not touched.
  - H4: unknown kinds retry, and publish completes idempotently after a backup failure.
  - H6: strengthened, because an incomplete block now gives Export mode.
  - H7: one new `"shopify"` literal in core `health.rs`. It belongs to a tracked family, and the plan put it out of scope (note 3). The description rendering stays in the adapter.
  - H8: the save lock sits in the desktop state container, and the logic stays in services.
  - No secret appears in the diff, the health details (serde "missing field" messages carry no values) or the tests (fixture strings only).
- **Assumptions.** None contradicts the book.
  - Defaults are used until a save; this is the plan's own design.
  - Stored definitions survive a failed refresh; this is C7(b).
  - `backup_failed` survives a runtime rebuild until a successful backup; this matches master plan §6.6 "cleared by the next successful backup".
  - A theme-only save clears a repaired-file warning: the theme path's `prepare` rewrote the complete file.

  All four are in the narrative in plain words.

## Findings
None blocking (B1–B4).

## Notes written to backlog
1. **→ P22 · P21.** While the stored block is incomplete, every save that lacks a full `integration` block fails in `prepare` with "missing field `client_id`". This includes the theme toggle. Because `settings_get` shows `null`, the photographer must re-enter all three shop fields (reviewer probe 2). The plan's try-out step 2 says "enter the secret"; the narrative says "re-enter the shop details".
2. **P21 (test).** C1(c)'s publish half cannot fail if `PublishJob` returned the backup error again. The 1 s retry re-runs the job with `was_done == true` and it ends `done`, while the start failure already holds the item. Probe 1 confirmed the production code is right. Lesson for planners: start from a working state and break it between start and the step.
3. **P21 → H7.** Another core `"shopify"` literal in `health.rs`.
4. **P21.** The test-only merged `ItemsService::search` now drops its local hits on a shop outage. No production caller.

## Lessons for the plans
- A row about a failure that happens *later* (C1(c)'s post-publish backup) must break the resource after start. Otherwise the start failure satisfies the row's outcome.
- When a plan nulls an invalid block on read, it should say what a save does with that block. Here it is left in memory and blocks every save that omits it.

## Write perimeter and probes
- Documents written:
  - `docs/implementation/master_plan.md` (P21 tracker row only)
  - `docs/implementation/plans/P21_backend_robustness.md` (Review log entry)
  - `docs/implementation/backlog.md` (4 rows appended)
  - `docs/implementation/narratives/P21_backend_robustness.md` (new)
  - this handoff
- Production code: none touched.
- Probe: a temporary test file `crates/testkit/tests/zz_review_p21_probe.rs`, which included the P21 test file's helpers via `/tmp`. It was created, run (2 passed) and deleted. Afterwards `git status --porcelain` was empty before any document write. It had no database side effects outside its temp dirs.
- Commits: none (the runner commits).
