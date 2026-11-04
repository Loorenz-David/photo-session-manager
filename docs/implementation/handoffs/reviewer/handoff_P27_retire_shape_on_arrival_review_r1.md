---
plan: P27_retire_shape_on_arrival
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-04
actor: opus
---
# P27 review r1: APPROVED_WITH_NOTES

Nothing blocks under flow §5 (B1–B4). Production matches the plan and book 10D MC-27 §Default / MC-28 §Recipes. Every automated row's test observes its outcome at the row's boundary and would fail on a wrong outcome. C6(a) was run live by the reviewer on scratch recipe names. There are four notes for the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
- **Gate.** Tracker `IMPLEMENTED`; checkpoint `463cb2e4c42d430ff92fed1836bcb89173fab46e` is in `git log`. Intention header RATIFIED.
- **Perimeter.** `git show --name-status 463cb2e` sorted equals the handoff's 48 declared `M/A/D` lines exactly (diff empty). The paths outside the plan's list are compile- or fixture-forced: `commands/items.rs` drops the removed error's match arm; `p04_capture_one_adapter.rs` and `p24_capture_one_provisioning.rs` add Master to their fixtures and expectations.
- **Suite** (reviewer, tree `7fbe1d3` clean; production identical to `463cb2e`). Results: `cargo test --workspace` 456 passed / 0 failed; `cargo clippy --workspace --all-targets -- -D warnings` 0; `cargo fmt --all -- --check` 0; `npm run typecheck` 0; `npm test` 20 files / 131 tests. `git status --porcelain` was empty afterwards, so the bindings are in sync.
- **C1(a)/(b).** `SqliteStore::arrival_insert` loops over `["original_preview","hash","render_preview"]` only. `CaptureOneScripting`/`EditingProvider` have no crop method left, so no crop write is reachable. The tests assert the exact sorted job list (it fails if any kind is added back), and C1(a) asserts the `PSM Preview` issue.
- **C2(a)/(b).** `Settings::from_legacy` is used by both `open` and `open_lenient`, and maps only when `default_shape` is absent. `legacy_default_shape` follows the plan table. `Settings::set` serialises the struct, so the old key is dropped. The tests rewrite the file, restart, and assert `settings_get` and the stored file per value, across all six legacy values.
- **C2(c).** Allow-list swaps `crop_on_arrival` → `default_shape` and adds `master_recipe`; the seven-name and non-empty rules apply; exact `InvalidValue(key)` per patch; the stored value is unchanged after a reject.
- **C3(a)/(b)/(d).** `provision.rs` loops preview → master → web with strict read-back equality (`root_folder_type` excluded). The tests compare the fake's recipe to `RecipeSpec::*.props()` field by field. C3(d) asserts exactly one `set_recipe_props:PSM Preview:exportCropMethod`. C3(b) asserts `make_recipe:PSM Master` and `MASTER_TIFF_COMPRESSION` by name.
- **C3(c).** `PSM Web` is still ensured. A renamed master is created, and the old `PSM Master` is still present.
- **C3(e).** The node harness runs the committed `ensure_recipe.js`. Writes are exactly `exportCropMethod`/`tiffCompression`, with no `bits` write; `changed` matches; the props carry the four keys.
- **C4(a).** Migration 003 is registered after 002. The test resets `user_version` to 2, so the migration really runs on restart. Health is in-memory (`HealthService.items: Mutex<Vec<_>>`) and nothing sets the retired codes any more. No `unknown_job_kind` rows.
- **C4(b)/C5(a).** The HealthPage shows the raw label. The Settings select has the right values, texts and order with value `1:1`; there is no "Shape on arrival"; the Master field and the patch keys are correct.
- **Retired names.** A grep of `crates/` and `apps/` finds `crop_on_arrival|crop_not_applied|crop_ratio_unknown|CropOnArrival|shape_on_arrival|listCropRatios|CROP_GIVE_UP` only in the legacy reader, migration 003 and the tests.
- **C6(a) live** (reviewer, 2026-11-04, Capture One 16.8.6, document `/Users/davidloorenz/Pictures/session test 1`).
  - Read-only pre-check: the real `PSM Preview` reads JPEG/8/Uncompressed/respect/1600/q80, and `PSM Web` reads JPEG/8/Uncompressed/respect/2048/q90. The strict read-back therefore holds on JPEG recipes.
  - Then the committed `ensure_recipe.js` ran via `osascript -l JavaScript` with the MASTER and PREVIEW specs on `PSM Review Probe Master`/`PSM Review Probe Preview`, twice each.
  - Run 1 created both. Every read-back field equals the spec: Master TIFF, 16, Uncompressed, ignore, Fixed/percent/100, sRGB, disabled, overwrite, `_session_manager`. Preview JPEG, 8, Uncompressed, ignore, Long_Edge/pixels/2560, q85.
  - Run 2 returned `changed []` for both.
  - Variant from the row: scratch names rather than the app's own recipes through `JxaScripting`, so the owner's recipes stayed as found.
- **Hard constraints H1–H8.** H1: no external write. H2: no RAW access. H3: no pixel code. H4: the migration is an idempotent DELETE. H5: AppleScript dictionary only. H6: commerce untouched. H7: no Capture One types in core; the `master_recipe` key follows the existing `preview_recipe`/`web_recipe` pattern. H8: the commands stay thin. Secrets and live services: no tests touch either.
- **Assumptions.** Four, and none contradicts the intention: `off → original`, the other unmapped names → `1:1`, the master is ensured before web, and the clippy allowance on the constructor. All four are in the narrative.

## Findings
None blocking. None should-fix under flow §5.

## Notes written to backlog
1. Fake fidelity: the fake's fresh recipe and pre-ensure `PSM Preview` use `bits 16`. Live, a fresh recipe needed a `bits` write, and the real Preview reads 8. The plan's "VERIFY-15: fresh = 16 bit" is not in the evidence.
2. `RecipeSpec::props()` uses `MASTER_TIFF_COMPRESSION` for every recipe. Changing the master to ZIP would also rewrite the JPEG recipes. Split the field in P29.
3. `default_shape` is not validated when read from `settings.json`. P28 must map an unknown stored value to `1:1`.
4. C6(a) ran on scratch names. The owner's own recipes are first repaired at their first launch with this build (try-out step 4).

## Lessons for the plans
- A delegated choice that cites evidence ("VERIFY-15 recorded a fresh recipe as 16-bit TIFF") should quote the evidence line. Here the evidence records format and scaling for a fresh recipe, not `bits`.

## Write perimeter and probe declaration
- Files written: `docs/implementation/master_plan.md` (P27 tracker row only), `docs/implementation/plans/P27_retire_shape_on_arrival.md` (Review log entry), `docs/implementation/backlog.md` (4 rows appended), `docs/implementation/narratives/P27_retire_shape_on_arrival.md` (new), this handoff (new). Scratch logs in `/tmp/p27rev/` (outside the repo).
- No production or test file was edited, not even temporarily.
- External state: in Capture One, two scratch recipes (`PSM Review Probe Master`, `PSM Review Probe Preview`) were created and then deleted. `recipes.name()` after the probe is byte-identical to before (`diff` empty). The owner's `PSM Preview`/`PSM Web` were read and never written.
- Commits: none (the runner commits).
