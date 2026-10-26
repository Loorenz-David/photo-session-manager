---
plan: P14_publish_update
role: review
round: 1
verdict: CHANGES_REQUESTED
state: OWNER_DECISIONS_PENDING
date: 2026-10-26
actor: opus
---
# P14 review r1: CHANGES_REQUESTED

Most of the phase holds: 19 of 21 rows are right at their boundaries, and the suite is green. There are two
blocking behaviour defects, F1 (the tag merge) and F2 (the condition read-back on the real adapter), and one row
whose test cannot fail (F3). F1's correction needs a choice of merge base, so it waits for owner card 1. F2 and F3
can be fixed now.

## ⚠ OWNER DECISIONS REQUIRED (1)

**Card 1: what should the app compare against when it merges product tags?**
- **Question:** Should the app remember the tags *it* last published, and merge against that (option A)?
- **Story:** You type the tag "oak" in the app; the shop already has "vintage", added in the admin. The first publish
  correctly sends "vintage, oak". You publish again a week later without touching tags, and "vintage" vanishes from
  the listing. Separately, if you delete a tag in the app, the shop keeps it forever.
- **Branches:** **A**: the app remembers its own last-published tags. Admin tags are never touched, and removals
  made in the app reach the shop. · **B**: the app's tag field mirrors the shop. Each publish copies the shop's tags
  into the app, so admin tags appear in your form and deleting one there removes it from the shop.
- **Recommendation:** A. It is the smallest change, keeps your fields exactly as you typed them, and never removes a tag you did not remove.
- **On silence:** the gate holds. F1 is not fixed and P14 stays unapproved.
- **Trace:** 07A MC-13 (Product tags), P14 C2(c), M20.

## Verified correct
- **Gate:** tracker `IMPLEMENTED`; checkpoint `535dd2a` exists. Review tree `33d43e4` = checkpoint + review prompt,
  `git status` clean before and after.
- **Perimeter:** `git show --stat 535dd2a` lists exactly the 17 files the handoff declares.
- **Stamp (L4, tree `33d43e4`):** `cargo test --workspace` 369 passed / 0 failed; `cargo clippy --workspace
  --all-targets -- -D warnings` exit 0; `cargo fmt --all -- --check` exit 0; `npm run typecheck` exit 0; `npm test`
  79 passed.
- **C1(a,b,d,e):** `write_set` writes only writable, non-reference, non-media keys with a local value whose canonical
  form differs from the remote's. Keys with no local value are skipped, so nothing is cleared. The read-back compares
  canonically and fails with `AttributeVerification{keys}`. Each test asserts the exact outcome at the fake shop or
  the publish item.
- **C1(c) production:** `frozen_attributes` keeps only `valid` values. A reviewer probe with a canonical text value
  `"Sweden"` stored `valid = 0` confirmed it is not written. (The test itself is F3.)
- **Rich text:** a remote value that cannot be canonicalized makes the field read-only, per the MC-13 type map.
- **C2(a,b):** `merge_tags` implements the MC-13 formula exactly and returns `None` when the result equals
  `remote_now`. **C2(c):** the written set and `tags_base` match the row as written (but see F1).
- **C3(a):** title is compared with `item.title` (the P11 note) and written only when it differs. **C3(b):** HTML is
  built by the adapter's `description_html` (the fake calls the real function); with no description there is no
  call. **C3(c):** the SKU-only identity write never clears the barcode, and a read-back mismatch fails at `update`.
- **C4(a–d):** the selected set is condition order minus exclusions; files use role `attribute:<configured key>`; the
  old file stays in Files; with no selection and no stored value nothing is written, and a stored empty exclusion
  writes `[]`. A missing field skips the file uploads and the write, finishes `done`, and the notification ends with
  the required sentence.
- **C5(a–d):** the replacement pass runs in `prepare` on `ready` links whose export generation is newer. It bumps
  `revision`, pushes the old id to `superseded_ids`, resets the state to `none`, and uploads under `-r<n>`.
  `name_matches` matches the exact revision stem (plus Shopify's uuid uniquifier) only. MC-12's `own` includes
  `superseded_ids`, so the old media is deleted and the new one takes the same index. The C5(d) test would fail on
  adopting the old media (`external_id != old`) and on a duplicate upload (`media.len() == 1`).
- **C6(a):** summary strings asserted exactly. **C7(a):** `Update → Done`, with `Prepare` emitted. No `describe` step.
- **Assumptions:** (1) empty base when `tags_base` is null: it does not contradict the book by itself, but it feeds
  F1(ii). (2) Missing-field note only when condition photos were confirmed: consistent with 09A MC-24. (3) The adapter
  builds the HTML: consistent with the port doc and MC-13. (4) The retouch generation is captured at the render
  request: a sound reading of MC-21 (the uploaded file reflects that generation). (5) Batches of 25 values, per SV-5.
  None of them is B1.
- **H1–H8:** writes run only inside a confirmed run (H1). No RAW access (H2). No pixel work (H3). `prepare` is
  re-entrant, since a replaced link is no longer `ready` and so is not replaced twice (H4). No UI automation (H5).
  The publish service is absent without commerce (H6). Core adds no platform types; the new `"shopify"` literals are
  a note (H7). No Tauri dependency (H8).

## Findings

### F1 (B1, blocking): `route: production` (after owner card 1)
- **What:** `tags_base` is set to the written tag set, and only when a write happens. The app's `product.tags` holds
  only tags the photographer typed; the book never seeds it from the shop. Two observable failures follow.
  (i) Reviewer probe: local `["x"]`, shop `["y"]`. Publish 1 writes `["y","x"]` and sets base `["y","x"]`. Publish 2,
  with nothing changed, computes `base − local = {y}` and writes `["x"]`, deleting the admin tag.
  (ii) Reviewer probe: local `["a"]`, shop `["a"]`. Publish 1 writes nothing, so the base stays null (treated as `[]`).
  The photographer then clears the tags and publishes, and the shop still has `["a"]`.
- **Authority:** 07A MC-13 Product tags ("so tags added in the Shopify admin are not lost"; base = the tags at the
  last read, refreshed even when nothing is written); M20.
- **Correction:** under card 1 option A (recommended), set `tags_base` to the local tag list after every successful
  `update` where a local tag value exists, whether or not a write happened, and amend C2(c)'s stated base to `["x"]`.
  Under option B, refresh the local `product.tags` and `tags_base` from the shop at each read. Add one boundary test
  covering the two-publish scenario (i) and the removal scenario (ii).
- **Files:** `crates/core/src/services/publish.rs`, `crates/testkit/tests/p14_publish_update.rs`, the plan's C2(c)
  cell (orchestrator).

### F2 (B1, blocking): `route: production`
- **What:** the condition-attribute read-back reads `commerce.attributes_read`. The Shopify adapter skips
  `condition_photos_key` in that read (adapter.rs `attributes_read`, P12 assumption A11; backlog "P12 → P14"). On the
  real shop, `readback.get("custom.damage_report")` is `None`, so the key is counted as failed. Every publish that
  writes the damage report, meaning any item with selected condition photos or a stored exclusion, ends `failed` at
  `update` with `attribute_verification{custom.damage_report}`. The fake returns every key, so C4(a–c) and C5(c) pass
  in the suite while the real adapter fails.
- **Authority:** 07A MC-13 (read back and compare), MC-14/M21; plan C4(a), C5(c); flow §5 B1 (integration-observable).
- **Correction:** give the read-back a real source for the condition file ids, either a dedicated port read of that
  metafield or a flag on `attributes_read`, implemented in the Shopify adapter, and make `FakeCommerce` omit the key
  from `attributes_read` exactly as the adapter does, so the C4(a) test would catch this. Keep the adapter's skip for
  the other callers.
- **Files:** `crates/core/src/ports/commerce.rs`, `crates/core/src/services/publish.rs`,
  `crates/integrations/src/commerce/shopify/{adapter.rs,graphql.rs}` (+ its unit test), `crates/testkit/src/fake_commerce.rs`,
  `crates/testkit/tests/p14_publish_update.rs`.

### F3 (B2, blocking): `route: production` (test only)
- **What:** C1(c)'s stored value is the raw text `bad` (not JSON) on an integer definition. It is dropped by
  `value.json()` and by `canonicalize` before the `valid` flag matters, so the test stays green if the `valid` filter
  is removed.
- **Authority:** plan C1(c); flow §5 B2.
- **Correction:** store a value that would otherwise be written (for example JSON `"Sweden"` on a text definition,
  `valid = false`, shop without the key) and assert no write. The production code already passes this; it was
  confirmed by a reviewer probe.
- **Files:** `crates/testkit/tests/p14_publish_update.rs`.

## Notes written to backlog
1. The manual description is re-sent on every publish (shop HTML compared with plain text).
2. C2(a)/C2(b) testkit tests duplicate the `merge_tags` unit test.
3. The export retouch generation lives only in the `render_export` job payload and falls back to 0, which risks a
   needless `-r1` re-upload.
4. Three more `"shopify"` literals in core `update` (H7 family).
5. The MC-24 notification is now done; the health message for a missing condition field is still absent.

## Lessons for the plans
- MC-13's tag base ("remote tags when the session last read them") presumes the local tag field is seeded from the
  shop, and the book never seeds it. The book should state which model it uses (card 1) and amend C2(c).
- When a backlog note routes an adapter gap to a phase (P12 → P14 condition read-back), the plan should carry a row
  that crosses the real adapter's behaviour, or require the fake to mirror the adapter's omission. Otherwise the fake
  hides the gap.

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P14_publish_update_review_r1.md` (new)
- `docs/implementation/plans/P14_publish_update.md` (Review log entry appended)
- `docs/implementation/master_plan.md` (P14 tracker row only → `CHANGES_REQUESTED`)
- `docs/implementation/backlog.md` (5 rows appended)
- Probe: `crates/testkit/tests/zz_review_p14_probe.rs` was created, run (4 probes) and deleted. It was untracked, and
  no production or tracked test file was touched; `git status` was clean afterwards. The only side effects were
  temporary directories made by the test harness. No live services were used.
