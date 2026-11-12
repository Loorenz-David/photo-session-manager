---
phase: P69_faster_export_and_publish
title: Faster export and publish — 8-bit masters, masters prepared in the background once a session is Ready, a batch overlaps Capture One renders with uploads, and every publish logs how long each step took
difficulty: high
projection: no
depends_on: [P68_setup_polish_address_password_shop]
---
# P69 — Faster export and publish (owner 2026-11-11, changelog 47)

Written by the orchestrator (Opus) on 2026-11-11. Owner: "is there some optimizations we can make to the export and publishing to make it faster also?"
after a 9-item batch on the work Mac (P64 build) where two items failed `provider: export failed: timeout` (P65 already turns Capture One timeouts into a wait).

## Evidence
- Master recipe (`crates/integrations/src/editing/capture_one/recipes.rs`): TIFF, **16-bit**, 100 %, compression `MASTER_TIFF_COMPRESSION`. On the a7R V
  (≈ 61 MP) that is ≈ 360 MB per photo written by Capture One before the app crops/scales and compresses once (10D).
- Dev-Mac publish 2026-11-11 07:12: masters/outputs then uploads; uploads and Shopify calls took ≈ 13 s for a small item (P64). The slow part on batches is
  Capture One rendering masters item after item.

## Goal
1. **8-bit masters (10D MC-28, changelog 47 (1)).** The master recipe becomes 8-bit TIFF (keep the compression choice unless a faster lossless one is clearly
   better — measure with the fake, record in the handoff). Existing 16-bit masters stay valid until replaced; the recipe self-repair (P60) updates the
   recipe in Capture One. Output JPEGs are still made once from the master (no extra compression step).
2. **Prepare masters when Ready (changelog 47 (2)).** When a session becomes Ready (07) and its masters are missing or stale, a low-priority background job
   asks Capture One for them: one photo at a time, only while Capture One is idle (no capture expected, no take-back, no preview render queued, no publish
   or export running for another session), paused otherwise. A publish then reuses ready masters (existing `(adjusted_gen, composition revision,
   output_format)` cache key) and only renders what is missing. Masters made this way follow 10F/MC-28 deletion after publish; the P61 disk budget may clear
   them (level 3) — they are re-made on demand. A setting `export.prepare_masters` (default on) in Settings → Storage.
3. **Overlap in a batch.** In a batch publish, while item N uploads, Capture One renders item N+1's missing masters (at most one item ahead), so the editor
   and the network work at the same time. Order of completion and P64 semantics (progress, cancel, waiting) unchanged.
4. **Per-step times.** Each publish item logs one INFO line `publish.timing item=<sku> masters_ms=… outputs_ms=… upload_ms=… shopify_ms=… total_ms=…
   masters_reused=n/m`, and the popover shows `Masters ready in advance: n of m` while publishing.

## Read first
Changelog 47, 45, 44, 40–41; book 10D (MC-28 masters), 07, 07A, 10F. Code: `crates/integrations/src/editing/capture_one/{recipes.rs, adapter.rs, renders.rs}`,
`crates/integrations/src/render_output.rs`, `crates/core/src/services/{export.rs, publish.rs, storage.rs, sessions.rs}`, job runner, settings, P61 storage job
(pause rules). Tests: p09, p44, p61, p62, p64, p65 testkit files; publish UI tests.

## Files expected to change (write perimeter)
The files above as needed; `crates/testkit/tests/p69_faster_export_and_publish.rs` (new); settings/publish UI as needed. ≈ 16 paths, ≈ 600 lines.

## Criteria (5 criteria, 7 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | 8-bit master | master render (fake Capture One records the recipe) | recipe `bits = 8`, TIFF; output JPEG made once from it | `[rule]` changelog 47 (1); 10D |
| C2(a) | prepared when Ready | session becomes Ready, Capture One idle | masters requested one at a time; publish afterwards renders none (`masters_reused=m/m`) | `[rule]` changelog 47 (2) |
| C2(b) | stays out of the way | a capture expected / a preview render queued / another session publishing | no master request until it ends; the setting off → nothing prepared | `[rule]` changelog 47 (2); 10F §6 |
| C2(c) | stale and cleared | composition changed after preparing; budget clears masters | stale master re-rendered at publish; cleared ones re-made; never a wrong frame | `[rule]` MC-28; 10F §4 |
| C3(a) | overlap | batch of 3 items, no prepared masters, fake upload delay | item 2's master render starts before item 1's upload ends; at most one item ahead; final order and results as without overlap | `[rule]` changelog 47 |
| C4(a) | timing line | any publish item | one `publish.timing` INFO line with all fields; popover shows `Masters ready in advance: n of m` | `[rule]` changelog 47 |
| C5(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (known intermittent `p33_split_grid` aside), clippy, fmt, typecheck green | `[rule]` master plan §7.2 |

## Notes
- **Hard rules:** no pixel quality change beyond 16 → 8-bit master; still one compression; never delete RAWs; tests never need Capture One or a shop.
  Default implementer (Codex; Claude when out).

## Review mode (owner 2026-11-10: fast; kept)
- **Implementer:** high effort; run the full closing gate once (`cargo test --workspace`, clippy, fmt, typecheck, `npm test`) and put the stamp in the handoff.
- **Reviewer (time-boxed, still correct):** do not re-run `cargo test --workspace`. Run this phase's new test file(s), the test files of the crates and features the diff touches, clippy, fmt, typecheck and the touched UI test files; trust the implementer's full-suite stamp unless it is missing or contradicts what you see. Read every changed production path against the rows; only B1–B4 findings block. Aim to finish in about 15 minutes.

## Review log

### 2026-11-12 — implement r1 (Claude, pipeline-implementer)
**Built.** (1) `RecipeSpec::MASTER` is an 8-bit TIFF (compression kept `Uncompressed`, VERIFY-15); the per-batch recipe self-repair
updates an existing 16-bit `PSM Master`. (2) `services/prepare_masters.rs` (`PrepareMastersService`, own thread, 1 s tick): the first photo
of the newest Ready session without a master for its current `adjusted_gen` is asked for through `ExportService::prepare_master`, one at
a time, only while no capture is on its way, no take-back, no render out, no preview waiting, no export or publish running and free space ≥
the storage minimum; prepared masters are committed by `master_ready` like an export's (frame adopted), held in `State::masters` so a run
that starts meanwhile waits for them, and are released/cleared by the existing rules. Setting `export.prepare_masters` (default on;
Settings → Storage checkbox; applied without reopen). (3) Batch gate in `PublishService::prepare` (an item waits while an earlier item of
its run is in prepare) and `start_next_in_batch` (the next item's export starts when an item leaves prepare); the export block of
`prepare` became `ensure_export`. (4) `ExportService` records per-run timing and masters ready/total (`ExportTiming`); `PublishService`
logs `publish.timing item=… masters_ms=… outputs_ms=… upload_ms=… shopify_ms=… total_ms=… masters_reused=n/m` at done, and
`PublishActivity.masters_ready` carries `Masters ready in advance: n of m` (popover line).
**Assumptions.** A1–A10 in the handoff (idle definition, which photos, one at a time, batch = one confirmation, timing field
definitions, popover only while running, setting name as the plan writes it).
**Deviations.** p27 `c3b` expects `bits = 8` and p56 `c3c` counts preview renders only (the restarted app now prepares the Ready
session's masters, changelog 47 (2)); the p69 tests run serialised (thread-scoped log capture); book 10D MC-28 still says 16 bits (book fold).
**Observations.** Before P69 every item of a batch asked Capture One for all its masters at once (all prepare jobs are due together),
which is the likely cause of the 9-item `export failed: timeout`. The closing `cargo test --workspace` stopped at p56 `c3c`; after the fix
p56–p69 were re-run (93/93) — files p00–p55 and every crate's unit tests had passed in the gate (852).


### 2026-11-12 — review r1 (opus): APPROVED_WITH_NOTES
**Perimeter.** `git show --stat b1129e1` = the 25 declared paths (handoff left for the runner); tree clean; HEAD `541a014` adds only pipeline docs.
**Checks (reviewer run, code tree = `b1129e1`).** `cargo test -p psm-testkit` p69 7/7, p09 8/8, p27 11/11, p44 25/25, p56 4/4, p61 7/7, p62 12/12, p64 8/8, p65 8/8; `cargo test -p psm-core -p psm-infra -p psm-integrations -p psm-desktop --lib` 133/133; `cargo fmt --all -- --check` 0; `cargo clippy --workspace --all-targets -- -D warnings` 0; `npm run typecheck` 0; vitest `features/publish` + `features/settings` 15 files 63/63. Full-suite stamp consumed (945/0 Rust in two legs, 466 UI).
**Verified correct.** C1(a): `RecipeSpec::MASTER.bits = 8`, compression kept `Uncompressed`; the test seeds a 16-bit `PSM Master` in the fake and observes the repair before the master is issued, then the output from an 8-bit master (one `render_output` job, one JPEG, master colour); `imaging/render.rs` decodes both `ImageRgb8` and `ImageRgb16`. C2(a): `PrepareMastersService::tick` asks at most one master (`ExportService::prepare_master` refuses while `state.masters`/`waits` holds the RAW; `busy` → `preparing` while one is out); the prepared master is committed by `master_ready` → `commit_prepared` when no run waits, else it feeds the waiting run; test observes order a, b, c with `paused("preparing")` between, the publish issuing no master and `masters_reused=3/3`, masters gone after publish. C2(b): `busy` order pause-check (capture/take-back/disk) → preparing → export → any active publish item → renders in flight → stale preview the preview service will still ask for (mirrors `schedule_due`: failed-for-good and missing RAWs skipped, so it cannot wait forever); test observes `render`, `publish`, `off` (persisted `false`, applied without reopen via `set_enabled` in the settings apply path) and `capture` (real FTP upload), each followed by the request once it ends. C2(c): master cache key is `(raw, adjusted_gen)` (crop ignored), outputs keep `(adjusted_gen, composition revision, output_format)`; test observes only the edited photo's master rendered, manifest at current gen/revision, recomposed frame 600×400; cleared masters re-rendered by the publish and not re-prepared below the minimum (`disk`). C3(a): `prepare` gate (`earlier_in_prepare` on the same `run_id`, unless started ahead) + `start_next_in_batch` after an item leaves prepare; a waiting item returns `Ok(false)` → job retry (no attempt cap, back-off ≤ 15 s), the same path as P65 waiting; test fails both with no overlap (next item's masters not asked during the upload) and with the pre-P69 all-at-once behaviour (2 ahead). C4(a): `log_timing` at done with all fields; `activity().masters_ready` only while running; test parses the INFO line and the activity during uploads/after; UI renders the line and none without it. C5(a): stamp + reviewer checks.
**Assumptions.** A1–A10 within changelog 47 and 10D/10F; none contradicts the intention (A2 is stricter than the plan: any session's publish pauses the job). 10D MC-28 text (16 bits, "only when an output is needed") superseded by changelog 47: book fold.
**Hard constraints.** H1 the overlap only starts local exports of an already confirmed batch; H2 masters only; H3 no new pixel code (recipe bit depth only); H4 a prepared master in flight at quit is an ordinary `render_master` job (abandoned at restart, re-asked); H5 AppleScript channel unchanged; H6 the job needs no commerce; H7 `shopify_ms` in core (plan-dictated log key, backlog); H8 Tauri DTO additions only. No live services in tests.
**Findings.** None blocking. Notes (backlog): 10D book fold; a take-back waits for one in-flight background master with "an export is running"; `shopify_ms` in core.
