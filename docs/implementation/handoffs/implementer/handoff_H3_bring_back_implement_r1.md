---
plan: H3_bring_back
role: implement
round: 1
state: DONE
date: 2026-11-23
actor: claude-opus (pipeline-implementer, PSM_IMPLEMENTER=claude)
---
# H3 — Bring back: implementer handoff, round 1

Gate: intention RATIFIED; `H2_list_pictures_pages_search` APPROVED; H3 was NOT_STARTED. Passed.

## What was built
**For the owner, in product terms.** The app can now bring an archived product (files cleared after publishing) or a
shop-only product (linked, never photographed on this Mac) back from Shopify: it downloads every published picture at
full size into the product's session folder (`<SKU>-main-01.jpg` …, `<SKU>-condition-01.jpg` …; `-b2` on a second
bring back), three tries each, and only when **all** are on disk makes them the session's photos, main then condition
in the published order, exactly as published (no crop, no turn, no house style). If one picture cannot be fetched,
nothing changes and the downloaded files are removed. The earlier photo entries stay, hidden; the undo list starts
empty; a published product stays "Published" and is not cleared again before its next publish. Each brought-back
photo is linked to the Shopify media it came from, so the next Publish uploads only photos that were re-framed,
turned, edited in Capture One, or added since; untouched ones are left alone, a new order only reorders, a hidden
photo's media is removed, and a shop-only product's media now count as the app's own. JPEG photos work like RAWs in
every photo operation (preview, frame, Compose, review, Capture One render, export). There is no button yet: H4 adds it.

**Per unit.**
- **U1 (checkpoint `fb8d34f`).** `psm_infra::rawpreview::JpegSource` (an implementation of `RawPreviews`) and its
  choice in `OriginalPreviewJob` for `.jpg`/`.jpeg` targets (MC-H4.6); `SyntheticJpeg::with_orientation`;
  **VERIFY-JPEG probed: PASS** (recorded in `docs/intention/evidence/capture_one_16.8.6.md`); guide 02.
- **U2 (checkpoint `c9dad23`).** `mechanisms::remote::{bring_back_names, estimate_bytes, DOWNLOAD_RETRIES,
  UNKNOWN_PICTURE_BYTES}`; `domain::remote::{BringBackProgress, BringBackState, BringBackOutcome, BringBackInput,
  BroughtBack}`; `Error::{BringBackRunning, BringBackFailed {reason}}` → codes `bring_back_running`,
  `bring_back_failed`; `AppEvent::BringBack` → window event `bring_back_progress` (`onBringBack` in
  `services/events.ts`); `Store::bring_back` (one transaction, MC-H4.4 (a)–(e), templated marks L3, canonical paths L5,
  stored `retouch_gen` L16); `RemoteItems::bring_back` + `bring_back_progress` in `services/bring_back.rs`;
  `RemoteItems::new` in its registry form (`editing`, `capture_root`, `history` added; `App::build` passes the uncached
  `inspect_document` + canonicalize closure, L4); `FakeCommerce::{fetch_original, set_image, original_fetches}` and
  `fake_media_of_url` (stored and refreshed URLs map to their media, L15); guide 01–04.
- **U3 (checkpoint `747a2c4`).** 18 acceptance tests on the real `PublishService` and `StorageService`. **No production
  change was needed**: MC-H4.7 (publish needs no new code) and MC-H4.8 (retention waits for the next publish, built in
  H1) held as written. Guide 04 row.
- Tracker + Review log: checkpoint `93d843d`.

## Tests → criterion rows
One test per row, each at the row's boundary with the row's exact outcome (64 rows: 59 behaviour tests in the suite,
5 unit tests for the Core mechanisms).

| Row | Test | Shape |
|---|---|---|
| H3·C1(a)–(e) | `h3_jpeg_photos::h3_c1a_…`, `h3_c1b_…`, `h3_c1c_…`, `h3_c1d_…`, `h3_c1e_…` | as the row (C1(b) also reads the fixture's tag back with `kamadak-exif`; C1(d) job status by SQL) |
| H3·C2(a)–(e) | `h3_jpeg_photos::h3_c2a_…` … `h3_c2e_…` | as the row (C2(c) registers the file after the first `tick`, see Assumptions) |
| H3·C3(a)–(e) | `mechanisms::remote::tests::h3_c3a_…` … `h3_c3e_…` (unit, Core mechanisms) | exact lists / numbers |
| H3·C4(a)–(j) | `h3_bring_back::h3_c4a_…` … `h3_c4j_…` | as the row |
| H3·C5(a)–(c) | `h3_bring_back::h3_c5a_…`, `h3_c5b_…`, `h3_c5c_…` | as the row |
| H3·C6(a)–(f) | `h3_bring_back::h3_c6a_…` … `h3_c6f_…` | as the row (C6(e): `catch_unwind` + `kill_and_restart`) |
| H3·C7(a)–(e) | `h3_bring_back::h3_c7a_…` … `h3_c7e_…` | as the row (C7(c): test waits for the first `original_fetches` entry) |
| H3·C8(a)–(d) | `h3_bring_back::h3_c8a_…` … `h3_c8d_…` | C8(a): no mutation **during** each of the four bring backs (Deviation d); the rest as the row |
| H3·C9(a)–(c) | `h3_bring_back::h3_c9a_…`, `h3_c9b_…`, `h3_c9c_…` | as the row |
| H3·C10(a)–(c) | `h3_publish_after_bring_back::h3_c10a_…`, `h3_c10b_…`, `h3_c10c_…` | as the row (C10(b): gallery checked as `[m1, new, m3]`) |
| H3·C11(a)–(d) | `h3_publish_after_bring_back::h3_c11a_…` … `h3_c11d_…` | as the row |
| H3·C12(a)–(d) | `h3_publish_after_bring_back::h3_c12a_…` … `h3_c12d_…` | as the row (C12(c) also counts one `file_create`; C12(d) also checks the arrival is Main 4) |
| H3·C13(a)–(c) | `h3_publish_after_bring_back::h3_c13a_…`, `h3_c13b_…`, `h3_c13c_…` | as the row (C13(a): "no foreign_media_added" = the publish reaches `done`) |
| H3·C14(a)–(d) | `h3_publish_after_bring_back::h3_c14a_…` … `h3_c14d_…` | as the row (C14(a): no `media_readback` during the 30-day scan) |

Every test in the three new test files maps to exactly one row; no orphan tests.

## Assumptions
1. **A failed bring back is announced for every failure after it started**: any error once the running mark is taken
   (also "wait until the publish has finished", "the capture folder is not known yet", "… no longer in the shop" and
   store errors) emits `BringBack {Failed, error}` with the progress so far. "No integration", "not remote" and
   "already running" emit nothing.
2. A picture found gone at the bring back's address refresh also goes on Health `remote_picture_gone`, as H1's
   "stored as gone" does.
3. Several gone addresses: one `media_urls` (chunks of 25) for all of them after the first pass; the refreshed
   addresses then get the full three tries; a second "gone" counts as a failed try.
4. A leftover file of an interrupted attempt is replaced by the rename; a failed attempt removes it along with its own
   files (by MC-H4.2 no photo row names it).
5. The JPEG-marker check (`FF D8 FF`) is a private constant of `services/bring_back.rs` (delegated choice).
6. A store error inside the names' `taken` check is returned as it is (not wrapped in `BringBackFailed`).
7. In a templated session only brought-back **main** photos get a placement row (the plan names main *k* only).
8. The `Done` event carries `done = total = N`; `Running` events are emitted under a lock, so `done` is 1…N in order
   even when downloads finish together; no `done: 0` event.
9. `RemoteItems::bring_back_progress(session_id)` is public: it is the running mark's progress the plan says H4's view
   reports.
10. `JpegSource` decodes the whole JPEG to validate it (a file that does not decode gets no preview); the scaled copy
    is Lanczos3 at quality 80 (delegated choice); EXIF via `kamadak-exif` (already an infra dependency).
11. Test setup, H3·C2(c): the fake Capture One lists a registered image only in the collection that was current when
    it was registered; the import's first `tick` switches the collection to the item's folder, so the test registers
    the file after that tick (the `p58` pattern registers after the hook for the same reason).

## Deviations
- **(a)** `crates/testkit/tests/suite/h2_pages_and_search.rs` (an H2 file) gained the three new `RemoteItems::new`
  arguments at its one direct call. The registry's final constructor forces it; that row's behaviour is unchanged.
- **(b)** The U3 checkpoint also touches `crates/testkit/tests/suite/h3_bring_back.rs` (fixtures `Remote`,
  `archived_of`, `adopted`, `register_pictures` made `pub(crate)` so U3 reuses them).
- **(c)** H3·C4(d) and the other "archived item" rows use a fixture without `p62`'s two hidden photos, so the archived
  item has exactly the five earlier rows the row names.
- **(d)** H3·C8(a) runs the four scenarios in one test and checks that no mutation was recorded **during** each bring
  back: the archived fixtures were really published first, so `mutation_ops()` holds that publish's mutations (the
  shop-only case asserts it empty).
- **(e)** The prompt template asks for actor `codex` on the tracker; this session is Claude Opus (master plan archive
  §3: implementer Claude), so the tracker says `claude opus`.
- No production change in U3; the publish service was not touched.

## Files changed
Production: `crates/infra/src/rawpreview.rs`, `crates/integrations/src/lib.rs`, `crates/core/src/{domain/remote.rs,
error.rs, events.rs, mechanisms/remote.rs, ports/store.rs, services/mod.rs, services/remote.rs, services/bring_back.rs
(new)}`, `crates/infra/src/sqlite/store.rs`, `apps/desktop/src-tauri/src/{lib.rs, commands/items.rs}`,
`apps/desktop/src/services/events.ts`.
Test kit and tests: `crates/testkit/src/{fixtures/synthetic_jpeg.rs, fake_commerce.rs}`,
`crates/testkit/tests/suite/{main.rs, h3_jpeg_photos.rs (new), h3_bring_back.rs (new),
h3_publish_after_bring_back.rs (new), h2_pages_and_search.rs}`.
Docs: `docs/intention/evidence/capture_one_16.8.6.md` (VERIFY-JPEG), `docs/guide/{01_product.md, 02_architecture.md,
03_rules_and_principles.md, 04_change_impact.md}`, `docs/implementation/plans/H3_bring_back.md` (tracker line, Review
log), this handoff.
`apps/desktop/src/services/bindings.ts`: regenerated by `cargo test -p psm-desktop`, unchanged (no specta type changed).

**Files a temporary probe touched (applied and reverted, not in any commit):** `crates/infra/src/sqlite/store.rs`
(`Store::bring_back`: the retired rows' `released` insert disabled → H3·C9(a), C9(c) red; the link state written as
`'none'` → H3·C10(b) red, C10(a) green because Upload polls a non-ready link that has a media id as "attached");
`crates/testkit/tests/suite/h3_publish_after_bring_back.rs` (C10(b) with a real settings edit after the first `.cos` →
2 uploads instead of 1). Both files were restored from copies before the checkpoints; `git diff` was clean afterwards.

**Outside the repository (VERIFY-JPEG probe, flow §6A; never deleted, H2):** in the owner's open Capture One session
`~/Pictures/Beyo Studio`: `Capture/_Unassigned/psm-probe-verify-jpeg-2026-11-23.jpg` (also now an entry in that
document's catalogue), its exports `Output/_session_manager/psm-probe-verify-jpeg-2026-11-23.{jpg,tif}`, and Capture
One's `Capture/_Unassigned/CaptureOne/Settings1690/psm-probe-verify-jpeg-2026-11-23.jpg.cos`. The shutter was never
fired.

## Checkpoint SHA
In order: U1 `fb8d34f` · U2 `c9dad23` · U3 `747a2c4` · tracker + Review log `93d843d`.

## Stamp
Closing gate, once, one command after the other, on tree `93d843d`'s parent content (`747a2c4` + plan file only):
- `cargo test --workspace` → exit 0 (psm-testkit suite 1299 passed; psm-core 151; psm-desktop 88; research 340 + 270
  + 77 + 30 + 14; 0 failed).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all -- --check` → exit 0.
- `cd apps/desktop && npm run typecheck` → exit 0.
- `npm test` → 123 files, 821 tests passed.
No pre-existing failures seen.
Per-unit commands (all green): U1 `cargo test -p psm-infra --lib rawpreview`, `h3_jpeg_photos::`, `p58_…::`,
`p06_live_previews::`, clippy (infra, integrations, testkit); U2 `cargo test -p psm-core --lib mechanisms::remote`,
`h3_bring_back::`, `p62_…::`, `a17_delete_session::`, `t2_template_placement::`, `cargo test -p psm-desktop`, clippy
(core, infra, integrations, desktop, testkit), plus `h1_remote_record::`, `h2_pages_and_search::`; U3
`h3_publish_after_bring_back::`, `p14_publish_update::`, `p30_…::`, `p62_…::`.

## Manual try-out
Nothing to try on screen yet: H4 adds the workspace and the **Bring back** button (no session launched the app, rule 9).
What the owner can look at today:
1. Open `docs/intention/evidence/capture_one_16.8.6.md`, last section **VERIFY-JPEG**: Capture One lists, finds and
   renders a JPEG in a capture folder like a RAW.
2. In Capture One's `Beyo Studio` session, the image `psm-probe-verify-jpeg-2026-11-23.jpg` in `_Unassigned` is that
   probe file (an app icon at 3000 × 2000); it can be removed by hand if unwanted (the app never deletes it).
For the orchestrator's live check after H4 (from the plan): bring back one archived test product, then publish with one
re-framed photo — Shopify must show one replaced picture and the others untouched; record the first bytes and content
type of one full-size download made through `ReqwestTransport`.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
