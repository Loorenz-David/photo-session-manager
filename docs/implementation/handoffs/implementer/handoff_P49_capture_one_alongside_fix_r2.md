---
plan: P49_capture_one_alongside
role: fix
round: 2
state: DONE
date: 2026-11-07
actor: codex
---

# P49 fix r2 handoff

Resolved F1: style application is limited to newly arrived RAWs that have not completed a first preview. A style changed after that preview no longer gets applied to a retouched variant. No owner decisions are required.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Finding → what changed → files

**F1 (B1, production).** `photo_arrived` now persists style eligibility. Before render, styling requires that eligibility and `rendered_gen == 0`; after a first completed preview, the RAW is ineligible even if settings rebuild the app. The existing style-applied timestamp interface masks the internal pending sentinel, and applying a style replaces the sentinel with the timestamp.

- C3(b) now changes the style after arrival but before variant discovery, then verifies the style is applied before render.
- New boundary test `f1_retouch_after_first_render_does_not_receive_late_style` renders with an empty style, changes settings, retouches the photo, and verifies the configured style is not applied.

**Files:** `crates/integrations/src/editing/capture_one/follow.rs`, `crates/core/src/ports/store.rs`, `crates/infra/src/sqlite/store.rs`, `crates/testkit/tests/p49_capture_one_alongside.rs`.

## Divergences

The review suggested a pending column in migration 007 as one option. I used a reserved `i64::MIN` value in the existing nullable `style_applied_at` column instead. Adding a column made P01's exact raw_state schema behavior test fail, and F1 did not authorize changing that test. The Store API translates the sentinel back to `None`, so existing callers continue to see only actual application timestamps. Eligibility persists through settings rebuilds and ends after `rendered_gen` becomes nonzero.

## Files changed

- `crates/core/src/ports/store.rs`
- `crates/infra/src/sqlite/store.rs`
- `crates/integrations/src/editing/capture_one/follow.rs`
- `crates/testkit/tests/p49_capture_one_alongside.rs`
- `docs/implementation/master_plan.md` (tracker row → `IMPLEMENTED`, note: fix 1)
- `docs/implementation/plans/P49_capture_one_alongside.md` (Review log)

## Checkpoint SHA

`084f1ee` — `CHECKPOINT (not approved): fix P49 style eligibility`.

## Stamp

Final checks on the committed content (`084f1ee`):

- `cargo test --workspace` — PASS, including P49 20/20 and the P01 exact schema check.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo fmt --all -- --check` — PASS.
- `cd apps/desktop && npm run typecheck` — PASS.
- `cd apps/desktop && npm test` — PASS, 369/369.
- `git diff --check` — PASS before checkpoint.

The checkpoint tree was clean immediately after commit. The handoff is written after the checkpoint so it can cite its SHA; the phase runner commits handoffs.
