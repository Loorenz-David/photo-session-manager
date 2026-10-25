---
phase: P15_publish_ui
title: Publish UI — confirmation popover, progress, failure and retry, batch queue, notifications (frontend)
difficulty: normal
projection: no
depends_on: [P14_publish_update, P11_form_ui]
---
# P15 — Publish UI: confirmation popover, progress, failure and retry, batch queue, notifications (frontend)

## Goal
Surface publishing in the UI from the design: the Publish popover as the confirmation surface (thumbnails, counts,
notes, "Publish N images" → `publish_confirm` with the exact summary shown), live progress on rows and in the header,
failure (sticky toast with Retry, popover with the reason, per-photo badges), batch "Review & publish N" → form queue
("Confirm & next", "Publish N confirmed"), the notifications drawer with Retry/Open, and the header action switching
from Export to Publish when an integration is configured. **Not in this phase:** settings/health screens (P16).
Thin frontend phase — **refine at prompt time**.

## Read first
master plan §6.7 (P13 commands, `publish_progress`), §6.9 · book `01` (H1), `14`, `07` §Publish, §Failure and retry,
`04` S-G, S-H, S-J, S-K · contracts `07A` MC-10 (the popover is the confirmation surface; summary text) · evidence
`design_handoff.md` §Publish, §Sessions list (status pills) · `claude_design/NOTES.md` §Flows (Publish one, Publish many,
Publish steps, Failure), §Screens 8; `COMPONENTS.md` §Sessions page (Failure popover, Batch bar), §Workspace (Publish
popover, Failure notice), §Product details form (Batch queue strip, Footer); screens `desktop/06–08`, `17–18`, `30`, `35`.

## Files expected to change
`apps/desktop/src/features/publish/{PublishPopover.tsx, PublishProgress.tsx, FailurePopover.tsx, PhotoUploadBadge.tsx,
BatchQueue.tsx, NotificationsDrawer.tsx, usePublish.ts, __tests__/p15_publish_ui.test.tsx}`, `apps/desktop/src/services/publish.ts`,
`apps/desktop/src/features/{shell/Header.tsx (Publish action, bell count), sessions/{BatchBar.tsx, StatusPill.tsx (Retry
click), SessionsPage.tsx}, workspace/{PhotoTile.tsx (badges), WorkspacePage.tsx}, form/FormDialog.tsx (queue strip, footer buttons)}`.

## Core mechanisms (unit tests allowed)
None.

## Ordered tasks
1. `usePublish`: `publish_preview(ids)` → popover data; `publish_confirm(ids, summary)`; subscribe to
   `publish_progress`; `publish_retry(item)`; `notifications_list`.
2. Popover per `COMPONENTS.md`; progress labels per MC-7 (`Publishing n%` from `session_changed`/`publish_progress`);
   failure toast (sticky, Retry) and popover; badges from `session_get` photo `upload_state` for the integration.
3. Batch: selection bar → form queue mode (strip, "Confirm & next" marks the item confirmed, "Publish N confirmed"
   → `publish_confirm` for confirmed sessions only).
4. Notifications drawer (bell count = failed items).
5. Tests.

## Criteria (6 criteria, 11 rows; 10 `[rule]` rows)
| Row | Boundary | Input (services mocked) | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `PublishPopover` render | `publish_preview` → 3 main, 1 condition, 2 unreviewed, 1 removal, `create_item`, details 3/8, summary text T | main thumbnails in order, condition thumbnails, "Product details 3/8", the text T rendered verbatim, button "Publish 4 images" | `[rule]` ch07 §Publish (confirmation surface) |
| C1(b) | click "Publish 4 images" | — | `publish_confirm([S], T)` called with the **same** T the popover showed | M17 (summary recorded = text shown) |
| C2(a) | `publish_progress{step: upload, uploaded: 2, total: 4}` then `session_changed` with label `Publishing(42)` | — | row and header read "Publishing 42%"; progress bar value 42 | `[rule]` ch07 (live progress) |
| C3(a) | item failed (`session_get` → `label PublishFailed`, error "Upload timed out") | — | sticky toast with "Retry"; clicking calls `publish_retry(item)`; failure popover shows the reason and "Nothing was overwritten" | `[rule]` ch04 S-J |
| C3(b) | label `DescriptionFailed` | — | row reads "Description failed"; Retry present | `[rule]` ch07 (list rows) |
| C4(a–c) | `PhotoUploadBadge` | `upload_state` `ready` / `none` after a failure / `uploaded` | "On Shopify" / "Not uploaded" / "Uploading" (3 rows) | `[rule]` ch04 S-J (badges) |
| C5(a) | tick rows A, B, C → "Review & publish 3" → form queue; "Confirm & next" on A and B; "Publish 2 confirmed" | — | `publish_confirm([A, B], T_AB)` — C excluded | `[rule]` ch04 S-H |
| C5(b) | queue strip | after confirming A | A shows ✓ and "n/N filled" | `[rule]` evidence §Form (batch queue strip) |
| C6(a) | `NotificationsDrawer` | `notifications_list` → one `published`, one `failed` with run/item ids | two rows; the failed one has Retry → `publish_retry(item)` and Open → navigates to the session | `[rule]` ch12 §Notifications drawer |

## Delegated choices
Popover vs. sheet on narrow widths, throttling of progress re-renders, exact iconography, where the "Open" of a
notification lands (workspace).

## Manual try-out
1. Open a Ready session with Shopify configured; click Publish: the popover lists the photos, counts and notes; click
   "Publish N images". The row shows "Publishing n%", then "Published"; the bell stays quiet.
2. Switch the Mac's Wi-Fi off and publish another: "Publish failed" with a sticky toast; Wi-Fi on → Retry → finishes.
3. Tick three rows → "Review & publish 3" → confirm two → "Publish 2 confirmed".

## Notes
- Estimated size: ~12 files, ~1 500 lines.

## Review log
(empty)
