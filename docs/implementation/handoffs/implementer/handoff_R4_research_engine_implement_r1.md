---
plan: R4_research_engine
role: implement
round: 1
state: DONE
date: 2026-11-12
actor: opus
---

# R4 research engine — implement r1

## What was built

A complete research run now works end to end inside the research crate, against stand-in providers. The app is not wired to it yet (R7), real providers come in R8, and nothing is visible in the app.

- **Photos that leave the Mac (U1).** A photo can reach a provider only as a *research copy*: a reduced, metadata-free JPEG cut from Capture One's adjusted preview with the photo's current framing. Only the research code can make one, and only for a photo still in the run's session and classified Main, Condition or Unreviewed. A deleted or moved photo, or one with only the camera preview, gets no copy. Each copy is stored once, under its SHA-256. The engine's five tables live in the app database, so its rows commit together with the app's.
- **Every provider call goes through one door (U2).** In one transaction before anything is sent, the door:
  - checks each photo again and drops any deleted since ("photo removed since");
  - applies the cost cap;
  - records the call and one outbound row per photo.

  Only then is the request sent. The reply, its usage and its cost (integer micro-USD from the run's price snapshot) are recorded, and the run's cost grows in the same commit. On a failure the door acts by class:
  - offline: held, no attempt counted;
  - transient: retried after 1 s, then 2 s;
  - rejected key: the run fails with "key rejected by …";
  - refusal: the unit's answer is "not determined".

  Any secret echoed back is replaced by `<redacted>` in the record and in logs.
- **The seven stages and Decide (U3).** Each stage does its own work:
  - Observe looks at each Main/Unreviewed photo.
  - Find searches with the cover.
  - Read reads pages in shop → trusted → other order, up to the page limit, skips the item's own listing and records "could not be read" for dead pages.
  - Compare compares the cover with each page's images.
  - Labels reads markings.
  - Condition reads the Condition photos.

  Each unit's output and its trace steps are committed together. Decide computes value and confidence with R3's rules, never from the model, and asks the model once for the reasons. It then writes through the MC-A1 door, with its `concluded` and `not_determined` steps, in one transaction: checked or cleared values are never overwritten, and an unsupported earlier suggestion is lowered to low. When only the Decide reserve is left, reading stops and the run ends `stopped_at_cap` with what it found.
- **When runs exist and start (U4).** A session's first Main photo queues an automatic run unless one of the first five prerequisites fails. The panel's status names the first failing one: no shop, no model key, no search key, a missing price, nothing to research, day limit, offline, or Automatic off. Runs start after the quiet period, one at a time app-wide, with Run again first. A run waits up to two minutes for Capture One's previews, stays queued at the day limit until local midnight, and waits out being offline with retries at most 60 s apart. The start freezes the settings, the form and the photo inputs. The work is a `research` job.
- **Crash, repeat, follow-ups (U5).** Killing the app at any point and relaunching ends with the same outputs, steps and values as an uninterrupted run. A call cut off mid-flight is closed as `unknown`, charged its fallback, and asked again. A changed photo set (added, deleted, restored, re-framed, re-classified, new cover) queues one follow-up run. That run reuses every unchanged unit and asks the providers only about what changed. Retouches, settings changes and swaps of non-cover photos queue nothing. Run again reuses nothing.
- **The research API (U6).** `ResearchApi` is implemented: status with "n fields filled · n to review", Run again, keys into the Keychain port (never read back), trusted-site add/remove/reorder ("already covered by …"), and settings validation (money as decimal strings with at most 6 decimals, record days ≥ 2).

## Tests → criterion rows

Every row has one test named after it (`<row>_<slug>`), at the row's boundary, asserting the row's exact outcome. Test program `crates/research/tests/engine/` unless marked.

| Unit | Rows | Tests | Count |
|---|---|---|---|
| U1 | A10·C3(a)–(d), C4(a)–(b), C5(a), A11·C8(a) | `images_and_storage::a10_c3a_deleted_photo_is_refused`, `a10_c3b_photo_moved_to_another_session_is_refused`, `a10_c3c_main_condition_unreviewed_are_rendered`, `a10_c3d_original_preview_without_adjusted_is_no_source`, `a10_c4a_same_photo_twice_is_one_copy`, `a10_c4b_recomposed_photo_is_a_new_copy`, `a10_c5a_fake_model_fault_then_reply_and_log`, `a11_c8a_one_running_and_one_queued_per_session` | 8 |
| U1 | A10·C3(e) | doc test `images::ResearchImage` (`compile_fail`: a struct literal from a path does not compile) | 1 |
| U2 | A11·C1(a)–C7(b) | `gateway::a11_c1a_…` … `a11_c7b_…` (one per row: c1a, c1b, c2a, c2b, c2c, c3a, c3b, c4a, c4b, c5a–c5e, c6a, c6b, c7a, c7b) | 18 |
| U3 | A12·C1(a)–C8(a) | `stages::a12_c1a_…` … `a12_c8a_…` (c1a–c1c, c2a–c2d, c3a–c3c, c4a–c4f, c5a–c5c, c6a, c6b, c7a, c7b, c8a) | 24 |
| U4 | A13·C1(a), C1(b), C1(d), C2(b), C2(c), C3(a)–(c), C4(a)–(b), C5(a)–(c), C6(a)–(b), C7(a)–(c), C8(a)–(b) | `lifecycle::a13_…` (one per row) | 20 |
| U4 core | `first_failing`, `may_start`, `start_order` | inline `rules::run_rules::tests` (8) | — |
| U5 | A14·C1(a)–(f), C2(a), C3(a)–(e), C4(a), C5(a)–(b), C6(a) | `resume::a14_…` (one per row) | 16 |
| U5 | A14·C6(b) | inline `rules::run_rules::tests::a14_c6b_order_of_identities_does_not_count` (the core mechanism `follow_up_needed`; one more inline test covers each identity) | 1 |
| U6 | A18·C1(a)–(c), C6(a), A19·C1(a)–(b), C2(a)–(c), C3(a), C4(a) | `api::a18_…`, `api::a19_…` (one per row) | 11 |
| **Total** | | | **99 rows** |

**Shape notes, rows where the assertion is narrower than it may look:**
- A12·C1(c) recomputes the fingerprint for observe, find, compare and decide units from their MC-A7 identities. For read and condition units it asserts one row per unit with the expected key, but does not recompute their fingerprints.
- A12·C7(b) and A14·C1(a) hold by construction:
  - A12·C7(b): `execute` reads only the run's snapshot. The test changes the live list from the search hook and asserts the reading order and the designer's medium.
  - A14·C1(a): a call reaches a fake only after its record is committed (MC-A5). "Before its record is committed" is read as "before its unit is committed". That call is closed `unknown` and re-asked on resume.
- A14·C3(c) and the cover half of A14·C3(d) give the new cover a search that finds other pages (see Assumption 22).
- A12·C5(c) uses C5(a)'s run (shared helper `reserve_run`) and checks that every call started below the cap and that the final cost is ≤ cap + the crossing call.
- Red baseline: not captured. Tests and code were written together unit by unit, and each unit's tests ran green before its checkpoint. No mutation regime applies (flow §4).

Counts: research crate lib 120 (R3's 110 + 10 `run_rules`), engine 122 (R3's 25 + 97 new), doc 1.

## Assumptions

1. **Default `attribution_keys` = `["beyo.designer"]`.** Row A13·C1(d) requires each default key to be a definition of `sv1_definitions_p1.json`. That fixture (the real shop) has `beyo.designer` and no manufacturer field. 08A §6 says "the shop's designer and manufacturer keys". §6.6's `custom.designer` / `custom.manufacturer` are not in the shop.
2. **Price rows** in `research.prices` are `{provider, model, unit?, usd_per_million?, fallback_call_usd?}`: a row may carry a unit price, a per-call fallback, or both. §6.6 leaves the shape open.
3. **A research source** is the photo's adjusted preview only when it belongs to the current `adjusted_gen` (`rendered_gen ≥ adjusted_gen`) and the file exists. A composition with no rect yet frames the whole preview.
4. **Waits between attempts** use an injected `Waiter`. The harness moves the manual clock; the app (R7) gives a sleep.
5. **A call that needs photos and has none left** sends nothing and records nothing. Its outcome is `NotDetermined("photo removed since")` with the dropped ids.
6. **A refusal** is recorded as `failed`, class `refusal`, charged its usage or the fallback.
7. **Run error texts:** key class → `key rejected by <provider>`; otherwise `<stage> failed: <text>`.
8. **An offline (`not_sent`) call's outbound rows are deleted:** nothing left the Mac.
9. **One error log line per gateway call** reports a redaction hit, over all its attempts.
10. **Attempt numbers count per (run, stage, unit, provider):** a stage 3 unit has a reader call and a model call.
11. **An unobtainable page is a reply** (`PageReply.obtained = false`), not a failure, so it is "could not be read" (MC-A3) and never fails the run. R8's reader must answer that way.
12. **Own-listing steps** ("this item's own listing") are committed with the find unit, from its candidates.
13. **Fields asked per stage:**
    - observe: visible fields;
    - read: attribution, measured and visible fields;
    - labels: attribution and visible fields;
    - condition: condition fields.

    A visible field's photo read is the first observe read, else a mapped label statement (sure).
14. **Decide's reasons call** happens only when ≥ 1 field is decided; with none, the cap is checked directly. Written and unchanged writes get a `concluded` step, and a lowering gets "no longer supported". One `not_determined` step is always written.
15. **Decide is never reused**, because its writes go to the rows of now. Every other unit is reused under MC-A7.
16. **`settings_changed` creates no run.** It only cancels queued auto/follow-up runs when `auto` goes off. Entering keys does not start research on every old session; the next photo change or Run again does. The status reflects the keys at once (A19·C1(b)).
17. **Run again with a creation-blocking prerequisite** returns an error carrying the sentence, and no row is created.
18. **Readiness timing:**
    - The 120 s source wait is measured from the session's last change, the same base as the quiet period.
    - "No eligible photo has a source" keeps any run queued, a manual run too.
19. **Offline does not block a start:** the run is held at its first call.
20. **The Unassigned session's status** has no reason and `can_run` false.
21. **`fields_filled`** counts the latest run's stage 7 writes with outcome `written` (not `unchanged` or `lowered`).
22. **Stage 3 units are identified without the cover** (MC-A7), so a new cover that finds the same pages reuses their reads. The tests for a new cover script a search that finds other pages.
23. **`secrets_set` with an empty string removes the key.** `site_remove` of an unlisted site returns the value unchanged.
24. **`export` and `purge_expired` wait for R7·U2:** export refuses ("not available in this version") and purge does nothing.
25. **The day** is `chrono::Local`'s calendar date (MC-A8).
26. **`session_changed` call sites (delegated; R7·U1 adds them):**
    - arrival, when a photo row is created;
    - `ReviewService::{classify, reorder, delete, move_photos, copy_photos}`;
    - `CompositionService::{set, apply, reset}`;
    - preview completion, when a research source becomes available.

    Each is called after its transaction. The research runner's wake-up interval is also R7's.

## Deviations

- `crates/research/src/writes.rs` (not in U3's file list) gained `suggest_in` / `lower_in`. With them, stage 7's writes commit with its steps (MC-A7 commit rule); `suggest` / `lower` behave as before (R3's 25 tests pass).
- `config.rs` was created in U2 (plan: U4) for the snapshot the gateway reads. `domain.rs` gained the run, step, call and prerequisite enums in U1.
- `CallGateway::call` returns `Called {outcome, dropped}` in place of a bare `GatewayOutcome`, so the caller gets the dropped ids as the plan asks.
- `CallFailure` keeps R3's shape, without §6.6's `exchange`: a failed call records its redacted error text.
- Default attribution keys differ from the §6.6 registry literal (Assumption 1).
- Dependencies: `rusqlite` (the workspace's 0.40, bundled), `tracing`, dev `tracing-subscriber`.
- Tracker actor `opus` (this session ran on Claude Opus), not `codex`.

## Files changed

All inside `crates/research` plus `Cargo.lock`. No app file changed.

- New: `src/{adapters/mod.rs, api_impl.rs, config.rs, decide.rs, fakes.rs, files.rs, gateway.rs, images.rs, job.rs, ports.rs, rules/run_rules.rs, runs.rs, schema.rs, service.rs, shop.rs, stages.rs, store.rs}`, `tests/engine/{api.rs, gateway.rs, images_and_storage.rs, lifecycle.rs, resume.rs, stages.rs}`.
- Edited: `Cargo.toml`, `src/{domain.rs, lib.rs, rules/mod.rs, writes.rs}`, `tests/engine/{harness.rs, main.rs}`; `Cargo.lock`.
- Closing commit: `docs/implementation/plans/R4_research_engine.md` (Tracker line and Review log only), this handoff.
- No mutation probes; no file was changed and reverted.

## Checkpoint SHA

U1 `d536832` · U2 `28411b2` · U3 `cb16665` · U4 `d891bf8` · U5 `e4f24c9` · U6 `620b2ac` · closing commit `CHECKPOINT (not approved): R4 closing tracker, review log and handoff` (the commit that adds this file).

## Stamp

| Command | Result |
|---|---|
| `cargo test -p psm-research` | lib 120 passed · engine 122 passed (33.6 s) · doc 1 passed; 0 failed |
| `cargo clippy -p psm-research --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cargo check --workspace` | clean |

No app suite (flow §10: no app file changed). Pre-existing failures: none seen.

## Manual try-out

Nothing is visible in the app yet; the engine is wired in R7. A run can be read as a story:
1. `cargo test -p psm-research --test engine stages::` runs whole research runs against scripted stand-ins: a full run fills designer (medium, two trusted sites), wood and scratches; the cap stops reading; refusals and failures end as described.
2. `cargo test -p psm-research --test engine resume::` kills runs at six points and compares each with an uninterrupted run; follow-ups ask only about what changed.
3. `cargo test -p psm-research --test engine lifecycle::` covers when runs are created and started (quiet period, one at a time, day limit, offline).

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
