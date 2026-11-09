---
plan: P58_fast_compose_on_camera_preview
role: fix
round: 2
state: DONE
date: 2026-11-09
actor: claude (pipeline-implementer, Sonnet)
---

# P58 fix 1 (round 2): F1 and F2 resolved

## Finding → what changed → files
- **F1 (B1): a non-matching camera preview became composable through an export's master frame.**
  - `CompositionDto` now carries `frame_source` (`camera` / `preview` / `master`, null until a frame exists), read from the stored fingerprint's source.
  - `canCompose` needs a rect and a frame, plus either an adjusted preview, or the camera original **and** `frame_source === 'camera'`. `composePreview` calls `canCompose`, so the editor follows.
  - Bindings regenerated with `cargo test -p psm-desktop export_bindings`.
  - C1(b) UI test: new case, frame from `master`, no adjusted preview → stage not opened, Compose `aria-disabled`, menu line "Waiting for Capture One's full-frame preview". The input was flipped to `camera` once and the case went red; then restored.
  - The camera fixtures of C1(a)/(c)/(d) now name `frame_source: 'camera'`.
  - Files: `apps/desktop/src-tauri/src/commands/compositions.rs`, `apps/desktop/src/services/bindings.ts`, `apps/desktop/src/services/compositions.ts`, `apps/desktop/src/features/compose/__tests__/harness.tsx` (default `preview`), `apps/desktop/src/features/compose/__tests__/p58_compose_on_camera_preview.test.tsx`.
  - Fixtures that must name the field now (type requires it): `machine.test.ts` (default `preview`) and inline compositions (`frame_source: null`) in `p11_form_ui`, `p19_navigation`, `p08_workspace_ui`, `p19_drop`, `p20_drop_continuity`, `p22_workspace_tidy`, `p26_double_drag`.
- **F2 (B2): the C3(b) test never delivered a capture.**
  - The test queues 10 backlog requests, then delivers `new-1` and `new-2` through the capture hook (new helper `captured`, same path as `camera_photo`: hook → Capture One variant present → tick). It asserts the next renders are new-2, new-1, then old-2, old-3 (old-1 was already with Capture One).
  - File: `crates/testkit/tests/p58_fast_compose_on_camera_preview.rs`.

## Divergences
None. No backend test was added for F1: the finding allowed one only "if a backend case is added". The DTO field is a plain read of the stored source, and the UI case carries the behaviour.

## Files changed (this cycle)
- `apps/desktop/src-tauri/src/commands/compositions.rs`
- `apps/desktop/src/services/bindings.ts` (generated)
- `apps/desktop/src/services/compositions.ts`
- `apps/desktop/src/features/compose/__tests__/harness.tsx`
- `apps/desktop/src/features/compose/__tests__/machine.test.ts`
- `apps/desktop/src/features/compose/__tests__/p58_compose_on_camera_preview.test.tsx`
- `apps/desktop/src/features/form/__tests__/p11_form_ui.test.tsx`
- `apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx`
- `apps/desktop/src/features/workspace/__tests__/{p08_workspace_ui,p19_drop,p20_drop_continuity,p22_workspace_tidy,p26_double_drag}.test.tsx`
- `crates/testkit/tests/p58_fast_compose_on_camera_preview.rs`
- `docs/implementation/plans/P58_fast_compose_on_camera_preview.md` (Review log entry)
- `docs/implementation/master_plan.md` (tracker row P58 → `IMPLEMENTED`)
- No mutation probes touched any file. No production file besides the two above.

## Checkpoint SHA
`7549fc8` (CHECKPOINT (not approved): P58 fix 1). The handoff itself is part of that commit.

## Stamp
Tree: working tree at the checkpoint.
- `cargo test --workspace --no-fail-fast`: all targets ok, 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: ok.
- `cargo fmt --all -- --check`: ok.
- `npm run typecheck`: ok.
- `npm test`: 58 files, 419/419 (was 418; +1 C1(b) case).
- Flakes seen under load, not caused by this change, each green on rerun:
  - `p42_camera_runtime` (a 3 s `settle` timeout; different tests failed on two runs; green with `--test-threads=1` and in the full run).
  - `p33_split_grid` C4(d) (1 s UI timeout, once).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
