---
plan: LV2_current_shot_and_phone_server
role: implement
round: 1
state: DONE
date: 2026-11-26
actor: opus
---
# Handoff — LV2 the current shot and the phone server, implement r1

Actor note: the runner launched this session on Claude Opus after Codex (Sol, then Terra) was out of usage. A Codex
session had already left uncommitted Unit 1 work in the tree. I reviewed it, kept what was right and reworked the
rest: its lock order could deadlock, and its "taken" and "shift back" did not follow CS2–CS4/CS8.

## What was built
- **The current shot (U1).** In a templated session one container is current; the Mac outlines it "Next". Every
  shutter is bound to the shot current at that moment, inside the ledger's lock and the same database transaction
  as the exposure row. The current shot then moves to the next empty container in shooting order, else the first
  empty one, else none (Condition). A late Wi-Fi file goes to the shot of its shutter: the camera's file counter
  picks the right shutter even when the ledger's first-in-first-out match was wrong. A file with no shutter report
  takes the current shot when it arrives. Retake denies the photo in a container at once (undoable, released) and
  makes that container current. A Deny no longer refills; Shift the rest back rebinds later photos. A session switch
  or a relaunch forgets a chosen shot.
- **Phone operations (U2).** The phone's session view (item name, SKU, article number = barcode; the shots in
  shooting order with ratio, split, zoom, example flag and photo; extras; Condition; counts; current shot). Move
  swaps and never replaces. Delete = Deny. Every current-shot change emits one `session_changed`.
- **Pairing and Settings → Phone (U3).** Pair a phone → a QR code of the `.local` URL, plus the address URL and its QR.
  The token is shown once and stored only as its SHA-256. Rename, Remove (confirm), last seen, a connected dot, the
  server line, the port field and the on/off switch. `settings.phone` defaults to on, port 8686; it refuses ports
  below 1024, the FTP control port and the FTP passive range; saving it never rebuilds the App.
- **The HTTP server (U4).** One phone server per app process, on its own thread and runtime, started in the desktop
  setup. It is reachable only from this Mac and private networks, and rebinds when the port is saved.
  - Pages: the phone page and its assets (no token, no path traversal), and the web app manifest (token).
  - API routes: `hello`, `session`, `capture`, `focus`, `focus/clear`, `current`, `retake`, `move`, `delete`, plus
    preview and example images. Every POST checks that its `session_id` is the active session (else 409
    `session_changed`).
  - A placeholder phone page says "Connected to <item>" or the pairing text. `npm run build` writes
    `dist/phone.html` and `dist/phone/icon.png`.
- **The WebSocket (U5).** Each socket is a viewer of the live view hub.
  - Frames go out only while the phone shows Live and is visible, and only frames of the current demand generation,
    each with the 32-byte `PSMF` header.
  - A slow phone skips frames and never queues them.
  - The phone receives events: focus, focus frames, capture, camera state/exposure, and session changes with a
    rising `rev`.
  - Pings every 5 s. The server closes 4408 after 15 s idle or a write slower than 2 s, 4409 when the event queue
    is full, and 4401 when the pairing is removed.
  - The agent guide (`docs/guide/01–05`) is updated.

## Tests → criterion rows
| Row | Test | Shape |
|---|---|---|
| C1(a)–(e) | `lv2_current_shot::lv2_c1a…`, `lv2_c1b…`, `lv2_c1c…`, `lv2_c1d…`, `lv2_c1e…` | exact `current`, `next_shot`, DB binding columns, place |
| C2(a)–(d) | `lv2_c2a…` (incl. one `SessionChanged`, the `current_shot.bound` log line), `lv2_c2b…`, `lv2_c2c…`, `lv2_c2d…` | exact |
| C3(a)–(c) | `lv2_c3a…`, `lv2_c3b…`, `lv2_c3c…` | exact slots, A's presets |
| C4(a)–(e) | `lv2_c4a…` … `lv2_c4e…` | exact binding / error variant |
| C5(a)–(d) | `lv2_c5a…` (Deleted, denial, released, current, undo label, one `SessionChanged`), `lv2_c5b…`, `lv2_c5c…`, `lv2_c5d…` | exact |
| C6(a)–(b) | `lv2_c6a…` (`ActiveSessionChanged` with each id), `lv2_c6b…` | exact |
| C7(a)–(b) | `lv2_c7a…`, `lv2_c7b…` | exact DB columns |
| C8(a)–(f) | `lv2_c8a…` … `lv2_c8f…` | exact (C8(b) undo restores the whole snapshot) |
| core CS4 | `mechanisms::current_shot::tests` (4) | input → output |
| core CS2 | `mechanisms::shot_order::tests::slot_owners_*` (2) | input → output |
| C9(a)–(d) | `lv2_phone_ops::lv2_c9a…` … `lv2_c9d…` | exact view fields |
| C10(a)–(g) | `lv2_phone_ops::lv2_c10a…` … `lv2_c10g…` | exact order/slots, by hand, undo |
| C11(a) | `lv2_phone_ops::lv2_c11a…` | exact |
| C12(a) | `lv2_phone_ops::lv2_c12a…` | exact count after each change |
| C12(b) | `mechanisms::template::tests::lv2_c12b_shot_ratio_follows_the_ph6_table` (core mechanism) | ±1e-4 table |
| C13(a)–(d), C14(a)–(c) | `lv2_phone_pairing::lv2_c13a…` … `lv2_c14c…` (C13(a) scans every text column of the database) | exact |
| C15(a)–(b) | `psm-desktop` `lv2_tests::lv2_c15a…`, `lv2_c15b…` | exact URLs, `<svg`, bindings names |
| C16(a)–(e) | `apps/desktop/src/features/settings/__tests__/lv2_phone_settings.test.tsx` | calls once with exact args, texts |
| C17–C23 (27 rows) | `lv2_phone_http::lv2_c17a…` … `lv2_c23c…`, one test per row | exact status, code, body, stdin line |
| core U4 | `phone::auth::tests` (3), `phone::web_root::tests` (3), `phone::wire::tests::error_code…` | input → output |
| C24(a)–(b) | `lv2_phone_ws::lv2_c24a…`, `lv2_c24b…` | exact |
| C25(a); C25(b)(c)(d) | `lv2_c25a…`; `lv2_c25bcd…` (the rows chain with "then"; one test, asserts per row) | exact |
| C26(a) | `lv2_c26a…` (1 MB frames; the slow client has a fixed 64 KB receive buffer) | ≤ 2, last = 50th |
| C27(a)(b)(c)(d) | `lv2_c27abcd…` (asserts per row, both viewers) | exact JSON |
| C28(a)–(c) | `lv2_c28a…`, `lv2_c28b…`, `lv2_c28c…` | close code, viewer count |
| C29(a) | `lv2_c29a…` | 39 bytes, every header field |
| core U5 | `phone::wire::tests::frame_header_has_the_ph5_layout` | input → output |

Every test in the new test files is in this table. No tests cover the placeholder phone page; no row covers it.

**Rewritten t2/t3/t4 tests (the 10G rule each encoded → what it asserts now, MC-LV3):**
- `t2_c4b` late file without a report (10G §4 rules 1/5 by file counter) → the file takes the current shot (P2);
  placed photos stay. Renamed `…_without_report_takes_the_current_shot_and_emits`.
- `t2_c4c` expired hold → file 3 takes the second slot; file 4 (no report, current shot none) is Condition.
- `t2_c4d` a photo from before the template counted as shot 1 → it is unbound and in no container; the arrival
  takes the first container.
- `t2_c4e` tether file time reordered photos → each tether file takes the current shot when it arrives.
- `t2_c5b` a late file moved a framed photo into the preset container (rule 5) → the framed photo stays, the late
  file gets the preset container. Renamed `…_late_file_gets_the_current_containers_preset`.
- `t2_c5c` preset removed when a late file pushed a photo to Condition → removed when the container leaves the
  template (CS3).
- `t2_c7b`, `t2_c7d` late file 2 took container 1 by shot index → takes the current shot (container 3); file 3 stays
  in container 1.
- `t2_c7c` a moved photo counted as shot 1 → unbound; the first arrival takes the first container in shooting order.
- `t3_c1d`, `t3_c1e` a moved/copied photo differed by shot index → unbound, does not differ. Renamed
  `…_is_unbound…`.
- `t3_c3f` Back to template filed photos from before the template → they stay; no undo step. Renamed
  `…_stay_where_they_are`.
- `t4_c1a` the dropped photo was placed by hand → placed by the template, place row bound to K3 (CS8).
- `t4_c1d` the retake carried the old shot key → the photo's place row is bound to the container. Renamed
  `…_binds_the_container`.
- `t4_c2b`, `t4_c2b2` refill after a Deny (10G §6 rule 4 refill) → no refill; the next shot is Condition, the ones
  after fill the empty containers in shooting order. `t4_c2b` renamed `…_does_not_refill_the_next_one_does`.
- `t4_c2b3` the late file kept its own place → takes the current shot (container 3). Renamed
  `…_takes_the_current_shot`.
- `next_shot_follows…` "the gap is filled" → the emptied container stays next.
- `t4_c4a_the_owners_story` A7/A8: refill → Condition and Shift offered; A8 drops that photo.
- Test counts per file are unchanged (t2 21, t3 19, t4 15).

## Assumptions
1. **Hand wins (CS3)** applies to a photo *coming into* a container. A template-placed photo already in its container
   stays even when a hand-placed photo also has that slot, so a Main reorder followed by an arrival moves nothing
   (t2 B1).
2. `ActiveSessionChanged` is emitted only when the active session really changes; re-activating the same session
   keeps the chosen shot. The Capture One activation path now goes through `store_target`, which adds its
   `arrival.target` line on a change.
3. `current()` for a session that is not active returns its default and leaves the held value alone. CS1's literal
   "hold" would displace the active session's tap, and CS8 only reads other sessions' default.
4. A held `Free` is not re-read (CS1 advances only a held shot). After every container was filled, a Mac Deny is
   followed by one Condition shot; the next shot fills the emptied container. This is consistent with C8(e) and
   "refill gone".
5. `phone_view` takes a `now` argument (the registry signature has none; the hold window needs the time).
6. An image's `ratio` is the shot ratio of the photo's own composition (split 2.0, else its shape); `revision` is its
   composition revision.
7. `move_to`:
   - In an untemplated session, `Container` answers `no_template`; the other targets work on the Main list.
   - A position out of range answers `invalid_value`.
   - A photo moved to Condition keeps its last slot in its row, as Deny does.
8. The phone's preview is the developed preview, else the camera's original preview, else the thumbnail.
9. POST `current`, `retake`, `move` and `delete` answer `{"ok": true}`; `current` with `container_id: null` means Free.
   `move` and `delete` of a photo outside the named session answer 404.
10. One `camera` message (a snapshot) follows `hello`; later ones follow changes. The phone learns the camera state
    at connect.
11. The phone server's listening socket has a fixed 256 KB send buffer (HB4). Measured: macOS otherwise buffered
    more than one 1 MB frame for a client that did not read, so a phone would see stale frames.
12. The server observes each App's event hub when a request or socket first resolves it (`ensure_fed`). Session
    events between an App rebuild (a settings save that rebuilds) and the phone's next request or socket are not
    forwarded.
13. Web root order: `PSM_PHONE_WEB`, then `<Resources>/phone-web` when it exists, then `apps/desktop/dist` (debug).
    Host fallbacks: no name → the address URL is used for both; no address → the name; neither → `localhost`;
    `hello.mac_name` = the name, else "Mac".
14. Pairing names are trimmed; empty → "Phone <n>"; renaming to empty is refused (`invalid_value`); the list hides
    removed pairings. Unassigned (no item) shows as "Unassigned" in the phone view.
15. Settings → Phone has its own "Save phone server" button (port + switch) and a "Done" button under the QR code.
    The confirm dialog's button reads "Remove phone".
16. `SettingsDto.phone` is optional in the DTO (always present on read), so the older UI test fixtures typecheck
    unchanged.
17. The desktop keeps the server in one process-wide slot (`commands/phone.rs`); `settings_set` applies a saved
    `phone`. Command signatures stay unchanged, so the mock-runtime command tests still build.

## Deviations
- **Files outside the units' lists**, each needed for a row or for H8:
  - `apps/desktop/src/features/settings/__tests__/t1_settings_drawer.test.tsx` (the section list now has Phone).
  - `crates/core/src/services/phone_view.rs` helpers `preview_file`, `session_photo`, `example_file`, and
    `template_placement::container_final_no` (U4, so the handlers decide nothing).
  - `crates/core/src/services/live_view.rs::viewer_count` (U5 rows assert the hub's viewer count).
  - `apps/desktop/src-tauri/src/commands/{sessions.rs, templates.rs}` (U1: events into `SessionsService`, the Mac's
    "Next", `HandAction.shots`); `commands/settings.rs` in U4 (rebind on save).
  - `crates/testkit/src/test_app.rs` (U1: `sessions()` passes the events).
  - Extra dependencies, already in `Cargo.lock`: `futures-util` and `ipnet` in integrations, `futures-util` as a
    testkit dev-dependency.
- **The U1 checkpoint (`7a685d76`) dropped five unchanged t4 tests** (`t4_c2c`, `c2d`, `c2e`, `c2e2`, `c2f`) through an
  editing mistake. They were restored unchanged in the U4 checkpoint (`d3db9091`) and pass.
- `crates/testkit/tests/suite/p01_storage_identity.rs`: the inherited schema tripwire now lists `phone_pairings`
  and the two binding columns (satisfied, not neutered).
- **Closing gate:** `cargo test --workspace` ran through the tool's background mode, because the tool caps a
  foreground command at 10 minutes. No other command ran while it ran (one cargo process at a time).
- `npm run typecheck`/`npm test`/`npm run build` ran from `apps/desktop` as the gate lists.

**Observations for the orchestrator (not owner decisions):**
- `psm-desktop` does not build without `apps/desktop/dist/` (verified: "resource path `../dist` doesn't exist",
  from the plan's `bundle.resources` mapping). On a fresh checkout, `npm run build` must run before cargo builds the
  desktop crate. The closing gate lists `npm run build` last.
- LV2·C3(b) needs an earlier file of the same counter run: with the ledger's FIFO match alone, the first late file
  sits in its FIFO shutter's shot until the second file arrives. The test starts with one Condition shot
  (DSC00100) as the anchor.

## Files changed
- U1: `crates/core/src/{domain/camera.rs, domain/template.rs, error.rs, events.rs, mechanisms/mod.rs,
  mechanisms/shot_order.rs, mechanisms/current_shot.rs (new), ports/store.rs, services/arrival.rs,
  services/camera_ledger.rs, services/current_shot.rs, services/sessions.rs, services/template_placement.rs}`,
  `crates/infra/src/sqlite/{ensure.rs, store.rs}`, `crates/integrations/src/{lib.rs, camera/runtime.rs}`,
  `apps/desktop/src-tauri/src/{lib.rs, commands/items.rs, commands/sessions.rs, commands/templates.rs}`,
  `crates/testkit/src/test_app.rs`, `crates/testkit/tests/suite/{main.rs, lv2_current_shot.rs (new),
  t2_template_placement.rs, t3_template_hand.rs, t4_template_retake_shift.rs}`.
- U2: `crates/core/src/services/{phone_view.rs (new), mod.rs, template_placement.rs}`,
  `crates/core/src/mechanisms/template.rs`, `crates/testkit/tests/suite/{lv2_phone_ops.rs (new), main.rs}`.
- U3: `Cargo.lock`, `crates/integrations/{Cargo.toml, src/lib.rs, src/phone/{mod.rs, pairing.rs, host.rs} (new)}`,
  `crates/core/src/{domain/live.rs, ports/store.rs, services/settings.rs}`,
  `crates/infra/src/{settings_file.rs, sqlite/store.rs}`,
  `apps/desktop/src-tauri/src/{lib.rs, commands/mod.rs, commands/phone.rs (new), commands/settings.rs}`,
  `apps/desktop/src/{services/bindings.ts, services/phone.ts (new), features/settings/{PhoneSection.tsx (new),
  SettingsNav.tsx, SettingsPage.tsx, __tests__/lv2_phone_settings.test.tsx (new),
  __tests__/t1_settings_drawer.test.tsx}}`, `crates/testkit/tests/suite/{lv2_phone_pairing.rs (new), main.rs}`.
- U4: `Cargo.lock`, `crates/infra/Cargo.toml`, `crates/integrations/{Cargo.toml, src/phone/{mod.rs, auth.rs, routes.rs,
  server.rs, web_root.rs, wire.rs} (new except mod.rs)}`, `crates/core/src/services/{phone_view.rs,
  template_placement.rs}`, `crates/testkit/{Cargo.toml, src/lib.rs, src/phone.rs (new),
  tests/suite/{lv2_phone_http.rs (new), main.rs, t4_template_retake_shift.rs}}`,
  `apps/desktop/src-tauri/{tauri.conf.json, src/lib.rs, src/commands/phone.rs, src/commands/settings.rs}`,
  `apps/desktop/{vite.config.ts, phone.html (new), public/phone/icon.png (new), src/phone/{main.tsx, PhoneApp.tsx,
  api.ts} (new)}`.
- U5: `Cargo.lock`, `crates/core/src/services/live_view.rs`, `crates/integrations/src/phone/{mod.rs, routes.rs,
  server.rs, ws.rs (new)}`, `crates/testkit/{Cargo.toml, tests/suite/{lv2_phone_ws.rs (new), main.rs}}`,
  `docs/guide/{01_product.md, 02_architecture.md, 03_rules_and_principles.md, 04_change_impact.md, 05_working.md}`.
- Close: `docs/implementation/plans/LV2_current_shot_and_phone_server.md` (Tracker line, Review log),
  `crates/testkit/tests/suite/p01_storage_identity.rs`, this handoff (committed by the runner).
- No mutation probes (flow §4: the regime is off); no file was changed and reverted.

## Checkpoint SHA
U1 `7a685d76` · U2 `18cdd4f2` · U3 `84d94253` · U4 `d3db9091` · U5 `1463728a` · close: `b5cc3336`

## Stamp
Tree: U5 checkpoint `1463728a` + the p01 schema tripwire update (in the close commit). One command at a time:
| Command | Result |
|---|---|
| `cargo test --workspace` (4 min 30 s) | psm-testkit suite 1597 passed, 1 failed: `p01_storage_identity::c1a_open_creates_all_domain_columns` (schema tripwire: the new `phone_pairings` table and the `shot_bound`/`shot_container_id` columns of `camera_exposures` and `photo_places`). Expected schema updated in `p01_storage_identity.rs`; `cargo test -p psm-testkit --test suite p01_storage_identity::` → 20 passed. Every other test binary green (desktop 126, core 181, integrations 151, infra 270, research 340, …). No pre-existing failure. |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `tools/sidecar/build.sh --stub --test` | 100% tests passed out of 2 |
| `cd apps/desktop && npm run typecheck` | clean |
| `npm test` | 139 files, 953 tests passed |
| `npm run build` | built; `dist/phone.html`, `dist/phone/icon.png` |
Stability: `lv2_phone_ws::` and `lv2_phone_http::` re-run 4× green after the slow-client fix (C26).

## Manual try-out
On this Mac, with the app built from this branch: `cd apps/desktop && npm run build` once (so the phone page exists),
then `npm run tauri dev`.
1. Open a session whose item type has a shooting template. The container outlined "Next" is the first in shooting
   order.
2. Shoot with the camera's own button: the photo lands in that container and "Next" moves to the next empty one.
3. Shoot two more quickly over Wi-Fi: each lands in the container that was "Next" when you pressed, even if the files
   arrive out of order.
4. Settings → Phone shows "Phone server on port 8686". **Pair a phone**, **Create**: a QR code appears. Scan it with
   the phone's camera (same Wi-Fi): the page says "Connected to <item>". If the phone cannot open the `.local` address,
   use **Show the address instead**.
5. Settings → Phone → **Remove** that phone, confirm, and reload the page on the phone: it says it was removed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
