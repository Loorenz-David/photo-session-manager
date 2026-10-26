---
plan: P15_publish_ui
role: implement
round: 1
state: DONE
date: 2026-10-26
actor: codex
---

# P15 publish UI handoff

## What was built

A photographer can review the exact publish summary, confirm one session or a selected batch, watch progress, see which photos reached Shopify, and retry a failed publish from the workspace, session row, or notification drawer. The bell counts sessions whose latest confirmed publish item failed.

The backend now supplies the latest confirmed publish item (including its retry target and reason) on list and workspace reads, per-photo upload state for the configured integration, and a failure notification linked to the publish item. The failure notification is written in the same transaction as the failed step. No migration was needed.

## Tests → criterion rows

| Row | Boundary test and exact outcome |
|---|---|
| C1(a) | UI `C1(a) renders ordered thumbnails, counts, notes and exact summary` checks the popover and verbatim summary. |
| C1(b) | UI `C1(b) sends the exact summary displayed in the confirmation surface` checks `publish_confirm([7], T)`. |
| C2(a) | UI `C2(a) refreshes the row after publish progress and session change to Publishing 42%` starts at 10%, delivers both events, then checks row and header text and progress values at 42. |
| C3(a) | UI `C3(a) shows the failed reason and sticky Retry for the workspace item` checks reason, safety text, infinite-duration toast, and `publish_retry(7)`. |
| C3(b) | UI `C3(b) retries the failed publish item from its session row` checks Description failed and `publish_retry(9)`. |
| C4(a) | UI parameter case `C4(a)` renders a workspace photo with `ready` as On Shopify. |
| C4(b) | UI parameter case `C4(b)` renders a workspace photo with `none` during failure as Not uploaded. |
| C4(c) | UI parameter case `C4(c)` renders a workspace photo with `uploaded` as Uploading. |
| C5(a) | UI `C5(a) publishes only the first two confirmed sessions with their fresh preview summary` checks `[A,B]` and T_AB, excluding C. |
| C5(b) | UI `C5(b) marks a confirmed queue item and shows its form fill count` checks ✓ and `1/3 filled`. |
| C6(a) | UI `C6(a) opens the failed session and retries its publish item from notification history` checks two rows, retry item 11, and `#/session/4`. |
| C7(a) | Rust `c7a_list_and_session_show_latest_confirmed_failure` checks the same failed item ID, step, reason and progress on list and workspace reads; never-published session is null. |
| C7(b) | Rust `c7b_session_photos_show_link_states_only_with_integration` checks ready/none link states and null with no integration. |
| C7(c) | Rust `c7c_failure_notification_links_to_retry_and_published_history` checks the failed notification text and item ID, then newest-first published and failed history after retry. |

The P15 test files contain 11 UI tests and 3 Rust tests, one for each row. No core-mechanism unit tests were added.

## Assumptions

- During a failed run, a photo without a link displays “Not uploaded”. This is the smallest visible state consistent with the upload badges in S-J.
- Each failed transition appends a history notification. Repeated failures remain in publish history; the bell counts latest failed items, so retry success clears it.
- A successful Retry dismisses that item's sticky toast. The drawer's Open action lands on the workspace hash.
- Unreviewed photos remain excluded from publish, as the amended plan confirms.

## Deviations

The generated DTO fields made three existing frontend test fixtures incomplete. I added null values to `p07_shell_sessions_ui.test.tsx`, `p08_workspace_ui.test.tsx`, and `p11_form_ui.test.tsx`; these fixture edits are outside the listed P15 file perimeter and required for the repository typecheck. No behavior from those phases changed.

## Files changed

- `crates/core/src/domain/session.rs`
- `crates/core/src/domain/publish.rs`
- `crates/core/src/ports/store.rs`
- `crates/core/src/services/sessions.rs`
- `crates/infra/src/sqlite/store.rs`
- `crates/testkit/tests/p15_publish_backend.rs`
- `apps/desktop/src-tauri/src/commands/sessions.rs`
- `apps/desktop/src-tauri/src/commands/publish.rs`
- `apps/desktop/src/services/bindings.ts`
- `apps/desktop/src/features/publish/FailurePopover.tsx`
- `apps/desktop/src/features/publish/NotificationsDrawer.tsx`
- `apps/desktop/src/features/publish/usePublish.ts`
- `apps/desktop/src/features/publish/__tests__/p15_publish_ui.test.tsx`
- `apps/desktop/src/features/sessions/SessionRow.tsx`
- `apps/desktop/src/features/sessions/SessionsPage.tsx`
- `apps/desktop/src/features/shell/Header.tsx`
- `apps/desktop/src/features/workspace/PhotoTile.tsx`
- `apps/desktop/src/features/workspace/WorkspacePage.tsx`
- `apps/desktop/src/features/form/__tests__/p11_form_ui.test.tsx` (fixture only)
- `apps/desktop/src/features/sessions/__tests__/p07_shell_sessions_ui.test.tsx` (fixture only)
- `apps/desktop/src/features/workspace/__tests__/p08_workspace_ui.test.tsx` (fixture only)
- `docs/implementation/master_plan.md` (P15 tracker only)
- `docs/implementation/plans/P15_publish_ui.md` (Review log only)
- `docs/implementation/handoffs/implementer/handoff_P15_publish_ui_implement_r1.md` (this handoff)

## Checkpoint SHA

This handoff is included in the single checkpoint commit. Resolve its SHA with `git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P15_publish_ui_implement_r1.md`; the final session line reports the resolved SHA.

## Stamp

| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test p15_publish_backend` | 3 passed |
| `npm test -- --run src/features/publish/__tests__/p15_publish_ui.test.tsx` | 11 passed |
| `cargo test --workspace --quiet` | passed, including all 3 P15 Rust tests |
| `npm test` | 6 files, 90 tests passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed |
| `cargo fmt --all -- --check` | passed |
| `npm run typecheck` | passed |
| `cargo test -p psm-desktop export_bindings` | passed; bindings regenerated |
| `git diff --check` | passed |

No pre-existing failures remain. The suite uses fake commerce and temporary data; no live Shopify or camera was used.

## Manual try-out

1. In a Ready session with Shopify configured, click Publish. Check the main and condition thumbnails, details count and summary, then click “Publish N images”. Watch the row and header progress finish at Published.
2. For a separate Ready session, disconnect Wi-Fi before confirming. The workspace should show Publish failed with a sticky Retry toast, a reason popover, and photo upload badges. Reconnect Wi-Fi and click Retry.
3. On the Sessions page, select three publishable rows and choose “Review & publish 3”. Confirm two in the form queue, then choose “Publish 2 confirmed”; the third remains out of the run.
4. Open the bell to see publish history. A failed entry offers Retry and Open; Open goes to that session's workspace.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
