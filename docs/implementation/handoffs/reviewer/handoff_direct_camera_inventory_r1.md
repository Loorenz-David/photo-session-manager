---
plan: intention-10E
role: inventory
round: 1
state: DONE
owner_cards: 1
date: 2026-11-06
actor: Claude Opus 5.5 (pipeline-inventory, mechanism-inventory doctrine)
---
# Mechanism inventory of 10E (direct camera adapter), round 1

**Summary.** The gate passed: the intention is RATIFIED, and 10E was ratified at `53d6ea4`. I inventoried 21 load-bearing
mechanisms behind MC-31…MC-34 and checked each against the lab code (Camera-Connection-Tests `221aa49`: `sdk/src/*.cpp`,
`camlab/ftp/server.py`, `camlab/ledger.py`, `camlab/experiments/camera_steps.py`) and against this repo's code (`crates/core`,
`crates/integrations`, `crates/infra`, `apps/desktop/src-tauri`). Most were not contract-grade, and several are contradicted by
the current code. The fix is 20 contract items, R-I18…R-I38 (R-I34 is unused), written into 10E as
"Inventory additions" under each contract, with pointers in 05A, 05B, 10A, 10C, 11 and 12. I reworded M37–M39 so a test can decide
them and added M40. One product question needs the owner (card CI1, non-blocking). Nothing changes what the photographer gets, so the
header stays RATIFIED.

The frontmatter says `state: DONE`, as the task asked. CI1 has an on-silence rule (the ratified text stands), so planning is not
gated by it. My chat line reports `OWNER_DECISIONS_PENDING`, because a card is present.

Exit gate: every silent-failure mechanism now has a contract-grade definition, except the support-package loading questions, which are
an explicit spike (S1–S6) that 10E already owed before planning. **The planner can start**. It should schedule the support-package
spike before the package phase.

## ⚠ OWNER DECISIONS REQUIRED (1)

**CI1: Capture One's browser when a photo arrives**
- **Question:** When a new photo arrives while you are working in another folder in Capture One, should Capture One jump to
  the session's folder at once, or only once you are back in Session Manager?
- **Story:** You are shooting the oak sideboard. Between shots you are healing a dust spot on yesterday's chair in Capture One.
  Each time the shutter fires, Capture One's browser jumps to today's folder and the chair disappears mid-edit. That is twenty shots
  and twenty jumps.
- **Branches:** *Always jump* (as written now): Capture One shows each photo at once and its adjusted preview follows within
  seconds, but your work in Capture One is interrupted on every shot. *Only when Capture One is not in front*: no interruptions; new
  photos and their adjusted previews appear a few seconds after you switch back to Session Manager, and the plain preview shows
  meanwhile.
- **Recommendation:** Only when Capture One is not in front. Being in Capture One means you chose that folder.
- **On silence:** Always jump (the ratified rule) stays; nothing else waits on this.
- **Trace:** 10E §Capture One alongside (inventory additions); 10A MC-18 step 6; R-I35.

## Inventory, ranked by silent-failure risk

| # | Mechanism | Silent failure if wrong | Status before | Now |
|---|---|---|---|---|
| 1 | Camera runtime lifetime across settings saves | `AppState::settings_set` → `SettingsSetContext::prepare` → `App::build` builds a **new** runtime (startup thread, watcher, nudge server) **before** `old.shutdown()`, on every non-theme save. A camera runtime built there means two sidecars (the second fights the first for the USB interface; the first's `quit` hands the camera to `ptpcamerad`) and two FTP listeners (the second fails to bind, so the receiver is silently dead) | absent | R-I18, M39 |
| 2 | Ledger association (FIFO) | `ledger.py` FIFO picks the oldest capture still expecting a stem, with no window. One exposure that never yields a file shifts every later file by one, and with persistence that lasts forever: the first photo after each session switch lands in the previous session | "by file-stem order", no bound | R-I26 |
| 3 | Duplicate identity and the final rename | `_refresh_duplicates(cap)` compares only inside one capture, and unassociated files are never compared, so a camera-initiated shot delivered by both paths becomes two photos. `std::fs::rename` replaces an existing file on macOS, which would silently overwrite a RAW | key stated, scope and rename not | R-I27, M37 |
| 4 | Session attribution of ledger files | `mechanisms/arrival.rs::attribute` rule 3 uses the session active **when processed**. `items.folder_key` follows the SKU until the first photo row (`ItemsService` update: `item_has_photos`), so a SKU edit between exposure and arrival puts the file in a folder that no session owns, and it falls to rule 3 | "MC-2 applies unchanged" | R-I28 (rule 0 + freeze) |
| 5 | `captured_at` of received files | MC-4: mtime = arrival time (FTP ≈ 4 s, cable ≈ 2 s, re-sends minutes late), so unreviewed order (`captured_at`, then id) follows arrival, not exposure | unstated | R-I28 (MC-4 amended) |
| 6 | Store destination and Save-to-PC folder | lab `disconnect()` calls `restore_store_destination()` after `configure_save` (up to 3 s against a dead session after a cable pull); `SetSaveInfo` is set only on request, so a download over Wi-Fi with a stale *Host PC + card* goes to an unset folder | "set per link, verified" | R-I19 |
| 7 | Capture One launch paths vs the USB claim | launches through `EditingProvider::ensure`: `EditingStartup::run` at startup, in `AppHandles::health_check` (the Health page and Settings call `getHealth` on load) and in the startup thread of every settings rebuild; `SessionsService::activate`; unguarded scripts (`front_document`, `make_document`, `open_document`, `ensure_recipe`, `ensure_capture_style`). Any one of them during "Take the camera back", or before the USB session at start, loses the camera within 28 ms (V6) | sequence named, gate absent | R-I31, R-I32, M40 |
| 8 | Camera and session activation coupled to Capture One | `CameraService::status` returns `Disconnected` and `capture()` refuses on `capture_folder_not_set` / `document_not_front`; `SessionsService::activate` stores `active_session_id` only after Capture One accepts the folder. With Capture One quit (take-back) or failing, the photographer cannot switch sessions and the camera reads disconnected | contradicts "Wi-Fi continues" | R-I33, 10A MC-18 step 5 |
| 9 | Local Network privacy of the packaged sidecar and receiver | the lab ran from Terminal (which held the permission); a packaged app with no `NSLocalNetworkUsageDescription` (none in `Info.plist` today) may never connect over Wi-Fi | unknown | spike S4, R-I36 |
| 10 | File-key normalisation | 10E said core strips `_N`; lab `file_key` strips only `__dupN`. Stripping `_N` in core collapses `IMG_1234` (H7) | contradiction | R-I26 |
| 11 | Paths that close a USB session | Mac sleep, camera auto power-off, a USB watchdog reconnect, IOKit presence counted over any vendor-0x054C device (see point 5) | "never let go" without a list | R-I23 |
| 12 | FTP namespace and staging | the camera uses *Directory Hierarchy: Standard* and *Same File Name: Does Not Overwrite* (lab setup): MKD/CWD must succeed, and the camera may LIST/SIZE after upload (the lab renamed synchronously for that). Two parallel STORs of one name would share a `.part`. REST behaviour | partial | R-I24 |
| 13 | Waiting for the Capture One variant | `find_image` searches only the current collection; V4 saw renders fail with `variant_not_found` and not recover | stated, unbounded | R-I35 |
| 14 | Secret hand-over | env vars of a child are readable by same-user processes (`ps -E`); redaction must be registered before the sidecar's stderr is logged | "env at spawn" | R-I21 |
| 15 | Save-to-PC partial and leftover files | the SDK writes under the final name; a crashed run leaves a file that the next download of that name meets | staging named | R-I20 |
| 16 | Camera status precedence | `reason` "a neutral enum", order not total | absent | R-I37 |
| 17 | Watchdog | "one notice", "no transfer active": not decidable; reconnect on USB gives the camera away | partial | R-I29 |
| 18 | Holder naming | lab `competing_processes` is `pgrep` of fixed names; which app asked `ptpcamerad` is not readable | stated as if readable | R-I30 |
| 19 | Support package verify, location, signing | "checks it is 2.02", "copies into support folder, where the sidecar loads them": the SDK finds `CrAdapter/` next to the executable, and a signed bundle cannot receive files | adjectives | R-I36, spike S1–S6 |
| 20 | Busy vs Auto FTP | deferring a switch on FTP uploads would block USB forever while shooting fast | ambiguous | R-I22 |
| 21 | Capture root source | `ArrivalService::capture_root` asks Capture One (`inspect_document`); the receiver must work while Capture One is quit | absent | R-I24 |

## The ten expected points

1. **Target folder vs MC-2.** The premise needs one correction: `items.folder_key` is never null (`TEXT NOT NULL`, computed at item
   creation, `services/items.rs`), but it *follows the SKU* until the item's first photo row. Risk: the file lands in `<k_old>/`
   and the SKU edit moves the key to `k_new`, so MC-2 rule 2 finds no owner of `k_old` and rule 3 attributes it to whichever session
   is active at processing. Settled by R-I28: the destination key is read at rename time; a committed delivery freezes the key (05B
   MC-22 amended); and MC-2 rule 0 attributes a file whose canonical path is a delivery's `final_path` to that delivery's session, so
   the folder name no longer carries the decision alone. One session per item holds today (`SessionsService::create` returns the
   existing one), so rule 2 is unambiguous apart from this race.
2. **Ledger persistence.** R-I25: SQLite tables `camera_exposures` and `camera_deliveries` (columns in 10E), one migration, kept
   forever, crash-ordered (`pending` row before the rename). Exposure → session: the active session read when the adapter receives
   `exposure observed` is stored on the exposure row; a file's session comes from R-I26's association, with the whole FIFO window
   deciding when it is unanimous.
3. **Store-destination races.** R-I19 moves the write into the sidecar's connect sequence, before `connection.ready`, so no app
   command interleaves. It forbids *Host PC* alone, so every race (an exposure during the ~210 ms apply; a cable pull leaving
   *Host PC + card* until the Wi-Fi session rewrites it) still leaves the card copy for Auto FTP. It sets `SetSaveInfo` to staging on
   **every** session, so Save-to-PC over Wi-Fi with a stale value lands in staging as an ordinary delivery (`via: Wireless`). It drops
   the lab's restore-on-disconnect.
4. **Save-to-PC completion.** The signal is `OnCompleteDownload` → `sdk.download.completed {filename (path), size_bytes,
   provisional_capture_id}`; the lab then hashes in a detached thread (`sdk.download.hashed`). The SDK writes under the final name, so
   partial-file safety rests on the dot folder (`is_candidate` rejects any path component starting with `.`; verified in
   `mechanisms/arrival.rs`) and on the ledger acting only on completion events (R-I20). It never scans `.incoming`. A staging folder
   per sidecar run avoids leftover collisions. "First complete copy wins": one ledger worker serialises decide → `pending` row →
   no-replace rename → `done`, and on `EEXIST` a content compare decides between duplicate and `__dupN` (R-I27).
5. **"Never let go" vs the lab sidecar.** Every path in the lab code that closes a USB session:
   - `TransportManager::step()` with `stop_` → `intentional_disconnect("stop requested")`, on `quit` **and on stdin EOF**
     (`StdinQueue` pushes `quit` at EOF, `main.cpp`).
   - `step()` in `ConnectedUsb` when `!usb_present_` → `session_lost("usb device removed")`. Presence is a count of **any** vendor-0x054C
     IOKit device (`usb_monitor_macos.cpp::matching`, `cmd_auto`).
   - an unintentional `OnDisconnected` → `on_sdk_disconnected` → `session_lost` → `adapter_.disconnect()`. This covers camera power-off,
     auto power-off and Mac sleep.
   - `session_lost` with `usb_from_discovery_` assumes the cable is gone.
   - `CameraSession::disconnect()` first runs `restore_store_destination()` after a `configure_save` change (`camera_capture.cpp`).
   - `~CameraSession()` at process exit, and the process dying.

   There is **no** idle disconnect, and in `auto` no disconnect on property or command errors (`session` mode exits on loss with code 3,
   which is unused). A Wi-Fi → USB switch closes only the Wi-Fi session. 10E's proposed watchdog reconnect would be a new closing path,
   so R-I29 forbids it on USB. Settled by R-I23 (closed list, idle-sleep assertion, camera-specific IOKit match, `usb.initial` and
   `usb-retry`).
6. **"Take the camera back".** The code that fights it is item 7 of the table, plus `CameraService`'s Capture One gates and activation
   gated on Capture One (item 8), plus the settings rebuild (item 1). V6's "relaunches Capture One itself" fits `ensure` being reached
   from startup and health checks. R-I32 gives one editor-launch gate and the step-by-step sequence with preconditions, bounds and
   aborts. It needs `usb-retry`, because after three failures the lab sidecar waits for a re-plug (`usb_retry_at_ = INT64_MAX`). M40
   makes it testable with the fakes.
7. **Current collection.** The effect on a photographer browsing elsewhere is card CI1. The render path: `PreviewService::request`
   catches the provider error and reschedules through `retry_after` with `render_backoff`, but V4 observed no recovery. The planner
   should reproduce V4 against the fake (`find_image` → none ×N, then found) before changing code. R-I35 fixes the observable: a 15 s
   bounded wait, then backoff for as long as the photo exists; `variant_not_found` never ends retries nor marks a generation failed; the
   style is applied at most once per RAW.
8. **Support package.** R-I36 sets out what is verified (a per-file SHA-256 manifest per SDK version compiled in, hashes only, plus the
   sidecar's reported SDK version), where it lives (`~/Library/Application Support/com.loorenz.sessionmanager/camera/sony/<version>/`
   with a **copy of the sidecar**, because the SDK resolves `CrAdapter/` beside the executable and the signed bundle cannot hold Sony
   files), how quarantine is avoided (the app extracts the archive itself), the ad-hoc local signature, and re-copying on app update.
   **Unverified, spike:** S1 loading from the support folder on a second account; S2 hardened-runtime library validation against the
   separately signed Sony dylibs (fallback: `com.apple.security.cs.disable-library-validation` on the sidecar); S3 Gatekeeper; S4 Local
   Network privacy (+ `NSLocalNetworkUsageDescription`); S5 ImageCaptureCore detection; S6 firewall and Keychain prompts after updates of
   an ad-hoc-signed build (`tauri.conf.json` `signingIdentity: "-"`).
9. **Secrets.** R-I21 defines the Keychain service `com.loorenz.sessionmanager.camera` with accounts `access-auth-user`,
   `access-auth-password` and `ftp-password`. Credentials go over a first stdin line, not env. Each secret is registered with
   `logging::add_secret` (the existing redactor) before spawn, and the sidecar's stderr reaches the log only through it. Nothing goes in
   argv or settings.json. The FTP password is generated by the app. The lab already logs only `ssh: bool` and the FTP username, never
   passwords.
10. **M37–M39.** As written they were not decidable. M37 ("exactly one visible file under a candidate name") contradicted the
   `.duplicates` rule and "never overwritten" covered staging truncation. M38's session clause had no rule to test against. M39 in 10E
   lacked the "quit only on click" clause that chapter 14 has, and "busy" was undefined. All three are reworded in 10E and chapter 14
   (R-I38). M40 (launch gate, start order) is new.

## Contradictions resolved by contract (for ratification)

- 10E "camera filename without server or retry suffixes (`__dupN`, `_N`)" vs the lab's `file_key` (only `__dupN`) and H7: the adapter
  strips camera suffixes, and core strips only `__dupN` (R-I26).
- "Never switches during … a file transfer" vs the evidence that Auto FTP runs whatever link the SDK session uses: "file transfer"
  means a Save-to-PC download (R-I22).
- 10A MC-18 step 5 ("camera status red, capture disabled") vs 10E "Wi-Fi shooting continues": with the direct adapter, step 5 leaves
  the camera alone (R-I33).
- "From there MC-2 applies unchanged" vs the attribution race and arrival-time mtimes: MC-2 gains rule 0 and MC-4 a `captured_at`
  exception for delivered files (R-I28).
- "Passed to the sidecar only through its environment" vs "kept in the macOS Keychain": stdin hand-over (R-I21).
- M39 differs between 10E and chapter 14: aligned to chapter 14 plus the inventory wording (R-I38).

## Ratify items added (all in 10E unless noted)

R-I18 one camera runtime per process · R-I19 connect sequence and store destination · R-I20 Save-to-PC staging and completion ·
R-I21 secrets · R-I22 busy reasons · R-I23 USB close paths, sleep assertion, protocol additions · R-I24 receiver namespace, lifecycle,
peers, capture root · R-I25 ledger tables · R-I26 file key and association order · R-I27 duplicates, worker, no-replace rename ·
R-I28 target folder, freeze, MC-2 rule 0, `captured_at` (05A, 05B) · R-I29 watchdog · R-I30 holder naming · R-I31 start order (10C) ·
R-I32 launch gate and take-back sequence (10C) · R-I33 independence from Capture One's health (10A) · R-I35 variant wait and style
once · R-I36 support package and spike S1–S6 (11) · R-I37 status values and precedence (12) · R-I38 M37–M40 wording (14).

## Notes for the planner

- The camera runtime must be designed around R-I18 first: it changes how `App`/`AppState` own background services, and every later
  camera phase depends on it.
- The ledger (R-I25…R-I28) is a core mechanism: unit tests are allowed under flow.md §4, ported from the lab's 15 ledger tests and
  extended with the window, global duplicate and rule-0 cases. The receiver's 18 lab tests become boundary tests (R-I24).
- Hardware acceptance gained three rows (no FTP re-send; settings save on USB; take-back including opening Health meanwhile).
- The existing `jxa/import.js` failure (evidence side finding) is unrelated to 10E and is not addressed here.

## Write perimeter

Documents only. No code, no commits, no archgraph delta, and nothing run against the camera, Capture One or Shopify. No secret was read.
- `docs/intention/book/10E_direct_camera_adapter.md` (inventory additions, invariants, acceptance rows, amended list)
- `docs/intention/book/05A_mechanism_contracts.md` (MC-2 rule 0)
- `docs/intention/book/05B_sku_generation_and_change.md` (MC-22 freeze at a delivery)
- `docs/intention/book/10A_capture_one_contracts.md` (MC-18 steps 5 and 6)
- `docs/intention/book/10C_capture_one_setup_by_the_app.md` (MC-26 step 1)
- `docs/intention/book/11_architecture_and_packaging.md`, `docs/intention/book/12_operations.md`
- `docs/intention/book/14_measurement_ledger.md` (M37–M39 reworded, M40)
- `docs/intention/book/15_decisions.md` (open card CI1), `docs/intention/book/16_changelog.md` (entry 32)
- this handoff
