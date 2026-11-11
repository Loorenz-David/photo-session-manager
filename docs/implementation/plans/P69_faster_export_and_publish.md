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
