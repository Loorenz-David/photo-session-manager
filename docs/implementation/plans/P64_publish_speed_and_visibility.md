---
phase: P64_publish_speed_and_visibility
title: Publishing fixes from the first real publish — files are found by name instead of listing the whole shop, images upload in batches and in parallel, the progress bar explains its current step and errors and can cancel the publish, and photos not yet reviewed are published after the approved main photos
difficulty: high
projection: no
depends_on: [P62_retention_after_publish]
---
# P64 — Publish speed and visibility; unreviewed photos are published (owner 2026-11-10, changelog 44)

Written by the orchestrator (Opus) on 2026-11-10 from the owner's first publish on the dev Mac with the current build. The owner: "pictures I haven't
approved were not uploaded; if I haven't approved them they should still count as desirable to upload"; "the progress line is stuck at 47% and I don't know
why, there is no status box or popover"; "why so long? doesn't it upload in batch optimally".

## What was observed
- A republish (main photos + condition photos) sat at 47% then 56% for minutes. The log shows the main uploads done
  (`staged_uploads_create` → upload → `product_create_media` → `MediaPoll`), then **615+ `FilesAll` calls in under 2 minutes** and counting:
  `ShopifyAdapter::list_files_by_name` (`crates/integrations/src/commerce/shopify/adapter.rs`) pages through **every file in the shop**
  (`files(first: 100)`, `graphql::FILES_ALL`) to find one file by name, for each condition photo (the idempotency check of 07A/09A). The shop holds tens
  of thousands of files.
- Uploads run strictly one at a time (reserve, upload, register, poll until READY, then the next).
- The progress bar shows only a percentage; no step, no error.
- Only `main` photos were uploaded; photos left `unreviewed` were skipped (07A `frozen_main` = the session's `main` photos).

## Goal
1. **Find files by name, not by listing the shop.** Every lookup of an already-uploaded file (the condition-photo idempotency check, and any other caller
   of `list_files_by_name`) uses Shopify's file search with a query on the file name / alt (e.g. `files(first: 10, query: "filename:<stem>*")` — verify the
   query syntax against the pinned Admin API version and the existing fixtures), returning in one or two requests. The match rules stay exactly as
   today (name starts with the stem, or alt equals `<stem>.jpg`).
2. **Batch and parallel uploads.** For one publish item, reserve all staged uploads in **one** `stagedUploadsCreate` call (chunked if Shopify limits
   the count), upload the files **in parallel (up to 4 at a time)**, register them in as few mutations as the API allows (`productCreateMedia` with
   several media; `fileCreate` with several files for condition photos), then poll readiness for all of them together. The order of the product's media
   stays the frozen order (07A MC-21's reorder step is unchanged). Idempotency (H4) and the "no duplicate uploads" invariant hold: a crash mid-batch
   resumes without uploading a file twice.
3. **Progress explains itself.** The publish progress bar (session list row and the session header) opens a popover on click/hover with: the current
   step in words (`Preparing outputs`, `Uploading main photos 3 of 8`, `Uploading condition photos 1 of 3`, `Checking Shopify`, `Updating the product`,
   `Writing the description`), the elapsed time, and the last error with its retry state if any. The percentage still shows on the bar.
4. **Unreviewed photos are published (changelog 44).** At publish, `frozen_main` = the session's `main` photos in position order, **followed by its
   `unreviewed` photos in position order**. Deleted and condition photos are unchanged. The session's `ready` status (07) no longer requires every photo to
   be reviewed: it needs at least one `main` or `unreviewed` photo with its adjusted preview, and every such photo's adjusted preview.

5. **Cancel publish (owner 2026-11-10: "there should be also a cancel publish option").** The popover has **Cancel publish**. It asks to confirm,
   then stops the publish item at the next safe point (after the request in flight; never between a mutation and recording its result, H4). The item's
   publish ends as `cancelled`: the session shows `Publish cancelled · Publish` (07 status: like `failed`, but not an error), and it is not "published"
   (retention never starts from a cancelled publish). Nothing already written to Shopify is deleted by the cancel; the next publish reuses files and media
   already uploaded (idempotency) and continues to the full frozen set. A product created by a cancelled first publish stays as a draft and is linked.

**Not in this phase:** retention/clearing (P62, unchanged except that it verifies the same published images), camera work, style.

## Read first
Book `07_session_states_and_publishing.md` (status, ready), `07A_publish_contracts.md` (frozen sets, upload, MC-21 reorder, idempotency), `09A` (condition
photos → metafield), `16_changelog.md` entry 44, H1/H4. Code: `crates/core/src/services/publish.rs`, `crates/core/src/services/sessions.rs` (status),
`crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs, transport.rs}`, the publish progress UI (find by the session row's progress bar and
`Publishing`). Tests: P13–P18 publish tests, P30, P62; Shopify fixtures (made-up data only).

## Files expected to change (write perimeter)
The files above as needed; new/updated Shopify fixtures (made-up); `crates/testkit/tests/p64_publish_speed_and_visibility.rs` (new); a UI test for the
progress popover. Existing assertions change only where the frozen set or `ready` changes (list each in the handoff). ≈ 16 paths, ≈ 700 lines.

## Core mechanisms (unit tests allowed)
Frozen-set computation; batching/chunking of a publish item's files.

## Criteria (7 criteria, 9 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | file lookup by name | fake Shopify with 50 000 files, one named `SKU-c01.jpg` | the lookup finds it with ≤ 2 requests; no `files` request without a query | `[rule]` 07A idempotency; owner 2026-11-10 |
| C1(b) | lookup keeps the match rules | files `SKU-c01_a1b2.jpg` (name starts with stem) and one with alt `SKU-c02.jpg` | both found, as today | `[rule]` 07A |
| C2(a) | batched reservations | 8 main + 3 condition photos | staged uploads reserved in ≤ 2 `stagedUploadsCreate` calls (not 11); at most 4 uploads in flight at once (fake transport counts concurrency) | `[rule]` owner 2026-11-10 |
| C2(b) | crash mid-batch | crash after 5 of 11 uploads | on resume no file is uploaded twice and the product's media end in the frozen order | `[rule]` H4; 07A MC-21 |
| C3(a) | progress popover | publish running at the condition step (2 of 3); then a failed upload with retry | popover shows `Uploading condition photos 2 of 3` and elapsed time; then the error and its retry state | `[rule]` owner 2026-11-10 |
| C3(b) | cancel publish | publish at `Uploading main photos 3 of 8`; Cancel → confirm | no new upload or mutation starts after the one in flight; status `Publish cancelled`; next Publish uploads only the 5 missing photos and ends with the full frozen order; nothing deleted on Shopify by the cancel | `[rule]` owner 2026-11-10; H4 |
| C4(a) | unreviewed published | session: main A, B; unreviewed C, D (positions 1, 2, 3, 4 interleaved as B, C, A, D); condition E; deleted F | frozen_main = B, A, C, D (main in position order, then unreviewed in position order); E as condition; F not published | `[rule]` changelog 44 |
| C4(b) | ready without full review | session with main A (preview ready) and unreviewed C (preview ready) | status `ready` (Publish offered); with C's preview missing → `in_progress` | `[rule]` changelog 44; 07 |
| C5(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (known intermittent `p33_split_grid` aside), clippy, fmt, typecheck green | `[rule]` master plan §7.2 |

## Delegated choices
Chunk size of `stagedUploadsCreate`; how readiness polling is combined; the popover's placement; step wording beyond the list above.

## Manual try-out
On the test store: publish an item with 8 photos (some not reviewed) and 3 condition photos. The bar's popover names each step; the whole publish takes
well under a minute; in Shopify admin the product shows the approved photos first, then the unreviewed ones, and the condition photos are in the damage
report field.

## Notes
- **Hard rules:** H1 (only the photographer's confirmed publish writes to Shopify), H4 (idempotent, no duplicate uploads), H6; tests never touch the live
  store. Default implementer (Codex; Claude when Codex is exhausted).

## Review mode (owner 2026-11-10: "I need this within an hour")
- **Implementer:** high effort; run the full closing gate once (`cargo test --workspace`, clippy, fmt, typecheck, `npm test`) and put the stamp in the handoff.
- **Reviewer (time-boxed, still correct):** do not re-run `cargo test --workspace`. Run this phase's new test file(s), the test files of the crates and features the diff touches, clippy, fmt, typecheck and the touched UI test files; trust the implementer's full-suite stamp unless it is missing or contradicts what you see. Read every changed production path against the rows; only B1–B4 findings block. Aim to finish in about 15 minutes.

## Review log
