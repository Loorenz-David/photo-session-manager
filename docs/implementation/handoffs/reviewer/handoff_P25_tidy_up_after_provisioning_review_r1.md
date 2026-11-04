---
plan: P25_tidy_up_after_provisioning
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-04
actor: opus
---

# P25 review r1: APPROVED_WITH_NOTES

All 12 criterion rows are made true by production code and each has a boundary test that fails on the wrong
outcome. Perimeter matches the declaration, the suite is green, H1–H8 hold. There are no blocking findings,
and 4 notes went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Gate
- Tracker row `IMPLEMENTED` (2026-11-04, codex). Checkpoint `b2f2ce8ac738a64eda43224c8735e30281428f04` present in `git log`.

## Verified correct
- **Perimeter.** `git show --stat b2f2ce8` lists 30 paths, equal to the handoff's "Files changed". The declared deviation
  (`apps/desktop/src-tauri/Cargo.toml`, `Cargo.lock`: `tracing-appender` named in `open_logged`'s return type) is justified.
  The `master_plan.md` diff is the implementer's own tracker row only. The plan file diff is its Review log entry only.
- **Suite (L4, reviewer tree `5c1f62e`, clean `git status`; code identical to `b2f2ce8`).** `cargo test --workspace`: 457 passed,
  0 failed. `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. `cargo fmt --all -- --check`: exit 0.
  `npm run typecheck`: exit 0. `npm test`: 18 files / 129 tests passed. The tree is still clean after the run.
- **C1(a)** `open_document.js:6–10`: after `open`, a front document whose name starts with `PSM Open Window` is closed and the
  script throws with `argv[0]`. A `null` or other front document → success. The catch re-checks the front, which is gone, so
  no second close. The harness asserts the exact ordered call lists for `replaced`/`null`/`refused`; an always-close,
  never-close or wrong-target close fails the test.
- **C2(a)** `editing_startup.rs:61–63` clears `document_not_front` when no session is active. The test raises the item through a
  refused activation, so it fails without the clear. `_Unassigned` routing, `Connected("Sony ZV-E10")` and `capture` asserted.
- **C2(b)** `sessions.rs:79` clears it on deactivate. Asserted immediately after `deactivate()` (not only after a check), plus a
  later check with no `front:` op.
- **C3(a)** `Settings::default_document_id` = `paths.home/Pictures/Session Manager`; `effective_document_id` falls back to it;
  `settings_get` adds `default_document_id`; `SettingsDto` and regenerated bindings carry it; the six TS fixtures were updated.
- **C4(a)/(b)** `Settings::default().style_name = ""`. Fresh file + DTO read `""`, no style write, `next_capture_styles` empty.
  Explicit `PSM House` survives `kill_and_restart` byte-identical and is applied once. An absent key reads `""`.
- **C5(a)–(c)** `integrity.rs:16–19`: NULL hash → `set_raw_hash` (fills NULL rows of the path only), persists `Ok`. A stored hash
  behaves as before (`RawChanged`, never overwritten). `NotFound` → `RawMissing` with the hash left NULL. Publish (no failed
  step, `staged_uploads_create` reached) and export (`K1_01_main.jpg` written) of an unhashed photo pass, and the hash
  is stored. Each test fails under the old code (first verify returned `RawChanged`).
- **C6(a)** `open_logged`: `paths.create` → `logging::init` → `open`. `App::build` logs `app open` with `data_dir` as its first
  statement after `create`. The test reads the single `psm.*` file for `app open` and the temp path, which fails if init ran
  after open. `run()` uses `open_logged(paths, App::open)` and keeps the guard alive.
- **C7(a)** `lib.rs` `health_check`: `refresh_shape_health` → `EditingStartup` (sets/clears `crop_ratio_unknown` from the live
  list) → `requeue_failed_jobs(kind, has("crop_ratio_unknown").then_some(..), now)`. The SQL exclusion mirrors `count_jobs`.
  The real-start re-queue passes `None`. Test: two checks leave the job `failed`/1 with no `set_crop:`; listing the ratio
  re-queues it and the next tick applies `Squircle`. P23 C3(c)/(d) still green.
- **C8(a)** `adapter.rs:833–889`: candidate = `newest`; poll to `READY` (`FAILED` → `Provider(mediaErrors[0])`, exhaustion →
  `Timeout`); filename compared with the staged stem through the shared `upload_name::matches`; else first gallery match;
  else `Provider("created media <stem>.jpg not found in the gallery")`. Fixtures are the captured SV-2 replies with ids and URLs
  from the real gallery. Case 2 fails a position-based implementation. Crash path: a timeout leaves the link `uploaded`
  without an id; `publish.rs` next attempt returns early while gallery media is processing, then adopts by filename (M18/H4).
- **Shared matcher.** `PublishService::name_matches` moved to `psm_core::mechanisms::upload_name::matches`, with one function and two
  callers (publish adoption + ownership, adapter).
- **Backfills (task 9)** present in p24 `c1a`, `c1b`, `c1e`, `c3a`, `c1f` (explicit style), p17 `c1b`, p00 `:78`, p12 `c6c`.
- **Assumptions.** (1) `Settings::default_document_id` and `upload_name::matches` are delegated choices. (2) The matcher requires a
  v4 UUID suffix, consistent with `shopify_admin.md:203`. Publish adoption narrows from any UUID to v4, which is consistent with
  the evidence. (3) Real-start re-queue unchanged, as plan task 7 states. (4) One tracing subscriber per desktop test binary.
  None contradicts the intention (no B1).
- **H1–H8.** No new external write (H1). Hashing is read-only (`sha256_file`) and no RAW is moved or written (H2). No pixel work
  (H3). Create-media resolution is crash-safe through adoption (H4). Capture One only via its dictionary (H5). No commerce
  dependency on the local path (H6). Core's new `upload_name` names no platform (H7). Tauri side is wiring only (H8).
  §6A: the suite contacted neither Capture One nor Shopify (fakes, node harness, captured fixtures).

## Findings
None blocking (B1–B4).

## Notes written to backlog
1. P25 (not verified live): silent-`open` detection rests on the P24 r2 observation; no live probe this phase.
2. P25: `product_create_media` gallery fallback errors when our own media is still `PROCESSING`. One attempt fails; the retry adopts.
3. P25 → process: `p25_tests::c6a` must stay the only `logging::init` caller in the desktop test binary.
4. P25: `document_not_front` survives a check that stops before the session step (Capture One down / document not open).
Also marked 10 backlog rows resolved by P25 (11, 15, 69, 106, 133, 164, 167, 168, 173; first clause of 107, its staging-sweep
clause stays open).

## Lessons for the plans
- None. The plan's code citations and fixture instructions matched the tree. Each row was decidable at its boundary as written.

## Write perimeter (this session)
- `docs/implementation/master_plan.md` (P25 tracker row only)
- `docs/implementation/plans/P25_tidy_up_after_provisioning.md` (Review log entry appended)
- `docs/implementation/backlog.md` (4 rows appended; resolution marks on rows 11, 15, 69, 106, 107, 133, 164, 167, 168, 173)
- `docs/implementation/narratives/P25_tidy_up_after_provisioning.md` (new)
- this handoff (new)
- No production or test file touched; no probe applied; no archgraph delta; no commit.
