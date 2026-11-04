---
plan: P28_composition_foundation
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-04
actor: opus
---
# P28 review r1: APPROVED_WITH_NOTES

Nothing blocks under flow §5 (B1–B4). Production matches the plan's geometry contract and book 10D MC-27 (Values, Default, Validation, Pixel rule, Revisions from Capture One) plus MC-28 §Thumbnail. Every row C1(a)–C7(b) has a test at the row's boundary that would fail on a wrong outcome. The suite, lint and typecheck are green on the reviewer's run. The geometry property reads were checked live, read-only. Four notes go to the backlog. Two of them were confirmed by a reviewer probe and should be fixed in P29, before P31 depends on them.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
- **Gate.** Tracker `IMPLEMENTED`. Checkpoint `9b24cbec2ab3a36a7cda2a896917b671bd7b88fc` is in `git log`. Intention header RATIFIED.
- **Perimeter.** `git show --name-only --format= 9b24cbe` sorted equals the handoff's 56 declared paths (diff empty). The paths beyond the plan's list are all compile- or fixture-forced:
  - `commands/items.rs`: the two new `CommandError` arms (precedent P12/P27).
  - `infra/previews_store.rs`: `thumbnail_path`.
  - `core/ports/editing.rs`: `RenderCompletion.geometry`.
  - `core/services/settings.rs`: P27's name check moved onto `Shape::parse`, as task 0 asks.
  - P01, P17 and P27 test fixtures, which encoded pre-P28 shapes.
  - Shell and pending UI `Photo` fixtures.
  - `Cargo.lock`.
- **Suite** (reviewer, clean tree `345ff13`, production identical to `9b24cbe`; the tree was dirty only during probes, see the declaration below):
  - `cargo test --workspace`: 481 passed, 0 failed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: clean.
  - `cargo fmt --all -- --check`: no diff.
  - `npm run typecheck`: clean.
  - `npm test`: 20 files, 131 tests passed.
- **C1(a)–(d), core geometry.** `mechanisms/composition.rs` was read against the plan's contract:
  - `default_rect` has both branches: `h = (W/H)/r` or `w = r/(W/H)`, centred.
  - `free`/`original` return the full frame.
  - `shape_ratio`: split is 2; `turned` gives `1/r`; `original` gives `W/H`.
  - `validate` runs in the contract's order, with the 1e-9 overrun tolerance, the 5 % minimum and the one-pixel ratio tolerance.
  - `inside_after_straighten` computes `s = c + R(−θ)(p − c)` (`sx = cx + cos·dx + sin·dy`, `sy = cy − sin·dx + cos·dy`) with `ε = 1e-6·max(W,H)`.
  - `pixel_rect` uses `f64::round` (half away from zero).
  - `split_halves`: integer `width/2` is the half width and the height; the right half starts at `left + half`.
  - `fingerprint_compatible`/`review_reason` follow the contract, including preview→master and master→preview.
  - `composition_cases.json` has 43 cases: 11 default, 4 pixel, 1 split, 13 validate, 12 fingerprint, 2 inside. Every expected value equals the plan's C1 table. Each of the four tests filters a non-empty op set, so none can pass vacuously.
- **C2(a).** `arrival_insert`/`photo_insert_or_get` insert the default row in the same transaction, using the store's `default_shape`. No Capture One call is made. The test asserts every field, `thumbnail_path` NULL, and no crop op.
- **C2(b)/(c).** `PreviewService` takes `dimensions` from the copied preview, makes the thumbnail, then calls `complete_preview(…, thumb, Fingerprint{frame, Preview, entry.geometry})`. `adopt_frame_tx` stores the fingerprint and materialises `default_rect`. The test checks the frame, source, geometry, rect, revision/origin/state, the exact `thumb-1.jpg` path, `image_dimensions == (640, 427)`, and exactly one `PhotoChanged`. `original` gives `(0,0,1,1)`.
- **C2(d).** `copy_photo` runs `INSERT INTO compositions SELECT ?1, <every other column>`. Copies of both a default row and a user row equal their source except `photo_id`, and keep `1:1` under a `4:5` setting.
- **C2(e).** For stub bytes, `image_dimensions` gives None and `image::open` fails, so the result is `Ok(false)`. The preview path is recorded, the thumbnail stays NULL, the composition stays frameless, and one event is emitted.
- **C3(a)–(f), set.**
  - `CompositionService::set` runs `composition` → `frame()` or `FrameUnknown` → `validate` or `CompositionInvalid{reason}` → `save_composition(User)`.
  - `save_composition` is a no-op when values, `ok` and origin are all equal. Otherwise it applies `revision+1`, `state ok`, `review_reason NULL`, and `change_seq+1` only if the photo is not `deleted`, all in one transaction.
  - Events are emitted only when a row was written.
  - The tests assert exact values after a restart, the four exact reasons with the row unchanged and no events, the no-op (same `updated_at`/`change_seq`/events), dirty on a published session's main photo (`published_seq + 1`, `SessionChanged`), not dirty for a deleted photo, and `FrameUnknown` on a frameless row.
  - C3(a) uses `orientation turned` where the plan says `named`. For `1:1` the ratio is identical, so this is immaterial.
- **C4(a).** `reset` uses the service's `default_shape`, which is rebuilt by `settings_set` → `apply_settings` (the desktop `AppState` path is the same). The result is `4:5`, rect = C1 case 3, revision 2, one event. A second reset changes nothing.
- **C5(a)–(e).** An incompatible observation sets `needs_review`, records the reason and replaces the fingerprint; the rect and revision stay. The tests cover rotation 3 (geometry) and 1707×2560 (frame), keep 1920×1280 and 2560×1700 `ok`, flag both rows of a shared RAW, and clear the flag on save.
- **C6(a)/(b).** `backfill_compositions` runs at every `App::build` and never touches `change_seq`. `mark_previews_stale_once` runs only under `start_backup`. It sets the flag in the same transaction as the `adjusted_gen+1` bump, and bumps only RAWs with an adjusted preview. The test fires a real start twice, checks that `retouch_gen` and `change_seq` are unchanged, that the flag reads `done`, that one more `process:…:PSM Preview` is issued, and that the second start changes nothing.
- **C7(a)/(b).** The view filters compositions to visible photos. Their order is the same as `photos_in_session` (same ORDER BY), so the positional zip in `SessionViewResponse` aligns. The IPC test checks the DTO fields, that there is no `geometry` key, `composition_invalid` + `off_shape`, `frame_unknown`, reset `origin "default"`, and that `session_get` carries the same composition and `thumb-1.jpg`.
- **Capture One geometry read (live, read-only).** Run on Capture One 16.8.6, document "chairs 4 november", variant DSC04451, via `osascript -l JavaScript`. The four `adjustments` expressions of the committed `process.js` (`rotation()`, `String(flip())`, `keystoneVertical()`, `keystoneHorizontal()`, `keystoneAmount()`) returned `{0, "none", 0, 0, 100}`, which parses as `EditorGeometry::default()`. Nothing was written. `process()` was not called.
- **Hard constraints H1–H8.**
  - H1: no external write; the geometry is read only.
  - H2: no RAW touch. Thumbnails decode only the copied JPEG preview.
  - H3: pixel work is limited to the 10D thumbnail and sits in `psm-infra` behind the core `Imaging` port.
  - H4: the backfill is idempotent. The stale mark is guarded by a flag in the same transaction. The thumbnail is written to a temp file and renamed.
  - H5: AppleScript dictionary only.
  - H6: commerce untouched.
  - H7: core sees `EditorGeometry` as opaque values under the names the plan's §6.10 registry fixed. No Capture One identifiers appear in core tables.
  - H8: `commands/compositions.rs` is thin (`spawn_blocking` → service), and the service has no Tauri dependency.
  - Secrets and live services: no test touches either.
- **Assumptions** (handoff). All three are consistent with the intention and the plan's delegated choices: JPEG quality 80 with Lanczos3; a compatible preview keeps the stored fingerprint; `restart_with_startup` is the real start (the plan's Read first was stale). The first two are in the narrative. The third is test-only.

## Findings
None blocking (B1–B4). Nothing else rises above a note under flow §5.

## Notes written to backlog
1. **Invalid hand-edited `default_shape` is stored raw (P28 → P29; this is P27's carry-forward, which was not folded into this plan).**
   - The store and the backfill write `settings.default_shape` verbatim. A value like `"Square"` makes `composition_row` fail to parse.
   - Reviewer probe: `composition(id)` and `sessions().view(s)` return `InvalidData`. `adopt_frame_tx` would also fail the preview commit.
   - `CompositionService` already maps unknown → `1:1`; the store does not.
   - Not B1: the Settings UI and the legacy mapping only ever write valid names, the book has no rule for an invalid stored value, and the precedent (P00 note on hand-edited `preview_delay_s`) is a note.
   - Fix: normalise once in `App::build`.
2. **Second photo row on an already-previewed RAW stays frameless (P28 → P29, before P31).**
   - `set_missing_preview_paths` copies the preview path, but not `thumbnail_path` or the frame.
   - Reviewer probe: the row reads `thumbnail None`, `rect None`, `frame None`, and `reset` returns `FrameUnknown` until the RAW re-renders.
   - Not observable in P28, since no UI reads these fields yet. It breaks P31's assumption "has an adjusted preview ⇒ has a rect".
3. **Exact `f64` equality (test).** C2(b) and C4(a) compare default rects with exact equality, where the plan says 1e-9. This fails the flow §4 over-engineering test against an algebraically equal rewrite.
4. **Not verified live end to end.** No real render has yet produced a `needs_review` flag. The owner's try-out step 4 is the first such run.

## Lessons for the plans
- A backlog note that a reviewer routes to a named next phase ("P27 → P28: P28 must map an unknown stored value to `1:1`") must reach that phase's plan as a row or a task line. Here it did not, and the defect it predicted shipped.
- Read first called `TestApp::kill_and_restart` "a real start". It passes `run_startup = false`; the real start is `restart_with_startup`. Plans that cite testkit helpers by behaviour should check the helper body.
- Plan Notes asserted that "P31's editor opens only on a photo with an adjusted preview, so it always has a rect". `set_missing_preview_paths` is a path where that is false. An invariant stated in Notes should name every writer of the field it relies on.

## Write perimeter and probe declaration
- **Documents written:**
  - this handoff;
  - `docs/implementation/narratives/P28_composition_foundation.md` (new);
  - `docs/implementation/master_plan.md` (P28 tracker row only);
  - `docs/implementation/plans/P28_composition_foundation.md` (Review log entry appended);
  - `docs/implementation/backlog.md` (four rows appended).
- **Code:** none. No production file was edited, not even temporarily.
- **Probes:**
  - Two scratch test files were created, run once each and deleted: `crates/testkit/tests/zz_review_probe_p28.rs` (second row on a previewed RAW) and `crates/testkit/tests/zz_review_probe_p28b.rs` (hand-edited `default_shape`). `git status --porcelain` was empty after each.
  - Their temp app directories are tempdirs owned by the testkit.
  - Capture One: one read-only `osascript` read of one variant's `adjustments`; no state changed.
- **Tool-recorded state** (archgraph or similar): none.
- **Commits:** none (the runner commits these files).
