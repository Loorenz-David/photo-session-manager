---
plan: P15_publish_ui
role: implement
round: 1
state: BLOCKED
date: 2026-10-26
actor: codex
---

# P15 publish UI, implement round 1

## What was built
The Publish button opens a review surface with ordered Main and Condition thumbnails, product details count, unreviewed and removal notes, and the exact backend summary. Confirm sends that same summary. Progress appears in the header and session rows. A batch selection opens the product form as a queue; confirmed sessions alone are previewed and published. The bell opens published notification history. Badge and failure detail components are ready for the missing backend data.

**Blocked:** failure reason, failed publish item ID, photo upload states, and failed notification records are unavailable from current commands. Consequently sticky failure Retry, per-photo badge integration, list row Retry, and notification Retry cannot work at their named boundaries. P15 cannot honestly be marked IMPLEMENTED. The backend must expose these values; that work is outside P15's file perimeter.

## Tests → criterion rows
| Row | Test | Result |
|---|---|---|
| C1(a) | `C1(a) renders ordered thumbnails, counts, notes and exact summary` | pass |
| C1(b) | `C1(b) sends the exact summary displayed in the confirmation surface` | pass |
| C2(a) | `C2(a) displays the backend-derived publishing percentage in the header progress` | partial: progress component 42%; no event-to-row-and-header test |
| C3(a) | none | blocked by missing failed item ID and reason |
| C3(b) | `C3(b) describes a failed description and offers Retry` | partial: StatusPill with supplied callback; no list retry target |
| C4(a–c) | three `C4` state-to-label tests | component passes; `session_get` lacks upload state |
| C5(a) | `C5(a) publishes only the first two confirmed sessions with their fresh preview summary` | pass at queue boundary; list selection is wired |
| C5(b) | `C5(b) marks a confirmed queue item and shows its form fill count` | pass |
| C6(a) | none | blocked by missing failed notification and item ID |

## Assumptions
- Unreviewed photos are excluded from publish, as chapter 07 and the backend say. Chapter 04 S-G and the design notes contain earlier wording that they publish in Main.
- The batch strip's fill counts come from each item's `form_get`; confirmation order determines the batch ID order.
- Opening a notification navigates to its workspace.

## Deviations
- `FailurePopover` and `PhotoUploadBadge` exist but are not wired to live failure/photo data because the current DTOs do not contain it.
- The list's Ready action remains disabled because `SessionRow.tsx` is outside this phase's stated file perimeter. The header action works.
- The tracker remains IMPLEMENTING and this handoff is BLOCKED; the required IMPLEMENTED closeout would misstate the criterion coverage.

## Files changed
- `apps/desktop/src/features/publish/PublishPopover.tsx`, `PublishProgress.tsx`, `FailurePopover.tsx`, `PhotoUploadBadge.tsx`, `BatchQueue.tsx`, `NotificationsDrawer.tsx`, `usePublish.ts`, `__tests__/p15_publish_ui.test.tsx`
- `apps/desktop/src/services/publish.ts`
- `apps/desktop/src/features/shell/Header.tsx`
- `apps/desktop/src/features/sessions/BatchBar.tsx`, `SessionsPage.tsx`, `StatusPill.tsx`
- `apps/desktop/src/features/form/FormDialog.tsx`
- `docs/implementation/master_plan.md`, `docs/implementation/plans/P15_publish_ui.md`, this handoff

## Checkpoint SHA
The checkpoint containing this handoff is the commit returned by `git log -1 --format=%H` after this session; its subject starts `CHECKPOINT (not approved): P15 publish UI partial`.

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `cd apps/desktop && npm test` | pass, 88 UI tests |
| `cd apps/desktop && npm run typecheck` | pass |

## Manual try-out
1. With a configured Shopify integration, open a Ready session and click Publish in its header. Check ordered thumbnails, counts, and summary; confirm to start publishing.
2. Watch the header's percentage and the session row's progress. Open the bell to see published notifications, then use Open to return to a session.
3. On the Sessions page, tick three ready rows, click Review & publish 3, confirm two forms, and click Publish 2 confirmed. The third session is excluded.

## ⚠ OWNER DECISIONS REQUIRED (0)
none
