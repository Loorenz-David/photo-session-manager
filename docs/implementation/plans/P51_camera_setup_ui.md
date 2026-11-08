---
phase: P51_camera_setup_ui
title: Camera setup UI — the header camera dot's five states and plain sentences with "Take the camera back", the Health camera items with their actions, and Settings → Camera (add Sony support, FTP password shown once with the camera steps, receiver ports, Access Authentication, fingerprint confirmation, Test camera connection)
difficulty: normal
projection: no
depends_on: [P50_camera_settings_and_support]
---
# P51 — Camera setup UI (10E §What the photographer gets, §Camera support packages, MC-34 §Guide/§Setup check, chapter 12)

## Goal
Everything the photographer sees of the direct adapter, over the P42–P50 commands and events: the header **camera dot** (`connected` green with
`<name> · Cable|Wi-Fi`, `connecting`, `busy`, `needs_action` red with the one plain sentence as its card and, when Capture One holds the camera, the
**Take the camera back** button; `disconnected`), the **Health** page's camera rows (every §6.6 code named in plain words, with **Dismiss** on "photos
not received", **Take the camera back** on the holder item, the helper log's last lines behind a disclosure), and **Settings → Camera**: *Camera support*
(Add camera support → Sony: where to download, accept Sony's terms there, pick the archive, the installed version; errors name the file), *Transfer from
the camera* (the FTP user `sessionmanager`, **Generate password** shown once with Copy and the camera menu steps, receiver port and passive range), *Wi-Fi
control* (Access Authentication user and password → Keychain, never shown back; the fingerprint card with Confirm/Reject when one is pending), *Check*
(Test camera connection → who holds or could take the camera), and the one-line agreement that the app sets where the camera saves photos (card +
computer on the cable, card on Wi-Fi).
**Not in this phase:** backend behaviour (P42–P50), hardware (P52), the automatic take-back.

## Read first
Book: `10E_direct_camera_adapter.md` §What the photographer gets (the dot and its sentences), §Camera support packages (the Settings flow text), MC-31
§Store destination ("The photographer agrees to this once in camera setup"), §Fingerprint, MC-34 §Guide and §Setup check, §Port changes (R-I37: the dot
shows the first reason; Health lists all) · `12_operations.md` §Health screen / header · `04_workflow_and_scenarios.md` S-A · evidence `design_handoff.md`
(components, timings, toasts) · lab `camlab/experiments/camera_steps.py:20–38` and `:41–55` (the camera menu paths shown to the photographer: FTP server,
user, password, Access Authentication, Still Img. Save Dest.) · master plan §6.6 (health codes and details), §6.7 (commands, events), §6.8, **§6.11**
(Frontend), §6.9 (frontend tests).
Code at `41bef14` (+ P42–P50): `apps/desktop/src/features/shell/CameraDot.tsx` (P42's minimal version), `apps/desktop/src/features/shell/Header.tsx`,
`apps/desktop/src/features/health/HealthPage.tsx` (`names` map `:9–17`), `apps/desktop/src/features/settings/SettingsPage.tsx` (sections at `:87`, `:112`,
`:130`; `pickFolder`/`pickSessionFile` in `services/settings.ts`), `apps/desktop/src/services/{camera.ts, events.ts, health.ts, settings.ts, bindings.ts}`,
`apps/desktop/src/lib/pending.ts` (`useAction`, `usePageData`), `apps/desktop/src/components/ui/*` (shadcn), `apps/desktop/src/features/history/HistoryButtons.tsx`
(the header icon-button + `Hint` card pattern), `apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx` (header test harness and mocks).

## Files expected to change
Frontend (new): `apps/desktop/src/features/settings/CameraSection.tsx`, `apps/desktop/src/features/settings/cameraSteps.ts` (the camera menu texts, one
export per list), `apps/desktop/src/features/shell/CameraCard.tsx` (the dot's popover: sentence, Take the camera back), `apps/desktop/src/features/health/
HelperLog.tsx`, tests `apps/desktop/src/features/shell/__tests__/p51_camera_dot.test.tsx`, `apps/desktop/src/features/health/__tests__/p51_health_camera.test.tsx`,
`apps/desktop/src/features/settings/__tests__/p51_camera_section.test.tsx`. Frontend (changed): `apps/desktop/src/features/shell/{CameraDot.tsx, Header.tsx}`,
`apps/desktop/src/features/health/HealthPage.tsx`, `apps/desktop/src/features/settings/SettingsPage.tsx`, `apps/desktop/src/services/{camera.ts, events.ts,
settings.ts}` (`pickArchive`), `apps/desktop/src/styles/*.css` (dot states). Backend (changed, small): `apps/desktop/src-tauri/src/commands/camera.rs`
(`camera_helper_log -> string[]`), `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/services/bindings.ts` (regenerated).
Estimated: 7 new paths, 10 changed — 17 paths. **No `implementer: fable`** (owner): the default implementer.

## Core mechanisms (unit tests allowed)
None (frontend rendering rows; the status → text mapping is tested at the component boundary).

## The contract (texts are exact; master plan §6.6 details are the health strings)
- **Dot** (`CameraDot`, `data-state` = `kind`): `connected` → light green, text `<name> · Cable` / `<name> · Wi-Fi`, title `Camera connected by cable|Wi-Fi`;
  `connecting` → amber pulsing, text `Connecting…`; `busy` → amber, text `Taking a photo` | `Receiving a photo` | `Switching connection` (by `reason`);
  `needs_action` → red, text = `message` truncated to 48 chars with `…`, full `message` in the card; `disconnected` → red, `Camera disconnected`. Click: opens
  `CameraCard` (Radix Popover, `modal={false}`): the full sentence; **Take the camera back** button when `reason === 'held_by_other_program' && holder ===
  'Capture One'`; **Retry** otherwise (today's `cameraRetry`); **Reconnect** when `reason === 'files_not_received'` and the link is Wi-Fi (the status
  payload carries `link` for needs_action too — P50 adds it to the DTO: `link: 'cable' | 'wireless' | null`); **Dismiss** for `files_not_received`. The button
  calls `cameraTakeBack()`; while running the card shows `Taking the camera back… (about 20 s)`; success → toast `Camera back on the cable` (or `Capture One
  reopened; the camera is still held` when `camera_held` is false); `take_back_blocked` → toast `Can't take the camera back yet: <reason>`; `take_back_failed`
  → toast `Taking the camera back stopped at step <n>: <cause>`. The dot follows `camera_status_changed` (already) and re-reads on mount.
- **Health**: `names` gains every §6.6 camera code → `Camera` (all `camera_*` codes), `Capture One follows` (`editor_not_following`), `Camera support`
  (`camera_support_not_installed`), `Camera helper` (`camera_sidecar_unavailable`); the row of `camera_files_not_received` has a **Dismiss** button
  (`cameraFilesNotReceivedDismiss` then reload), `camera_held` has **Take the camera back** (same handler as the card); a `Camera helper log` disclosure shows
  `cameraHelperLog()` (last 50 lines, monospace) under System status.
- **Settings → Camera** (`CameraSection`, after "Capture One"): (1) *Camera support*: when `cameraSupportGet()` is null → text `Session Manager can control a
  Sony camera directly. Download Sony's Camera Remote SDK for Mac from Sony's developer site (you accept Sony's terms there), then add it here.` + button
  **Add camera support → Sony** → `pickArchive()` (`.zip`) → `cameraSupportInstall(path)` → toast `Camera support for Sony <version> installed` and the
  section shows `Sony Camera Remote SDK <version> installed`; errors: `support_archive_invalid` → toast `The archive is missing or has a changed file:
  <file>`; `support_version_unsupported` → `This SDK version (<version>) is not supported; use 2.02.00`. (2) *Transfer from the camera* (shown only when
  support is installed): `User: sessionmanager`, **Generate password** → `cameraFtpPasswordGenerate()` → a card `Enter this password on the camera now; it is
  shown only once.` with the value, **Copy**, and the camera steps list (`cameraSteps.ftp(host, port, user)` — the texts of `camera_steps.py:20–38` adapted to
  the user `sessionmanager` and this Mac's address shown as `<this Mac's address on your network>`); fields *Receiver port*, *Passive ports from / to* bound to
  `settings.camera` (saved with the form's Save, `invalid_value` → inline error `Ports must be 1024–65535, the passive range at least 2 ports and not
  containing the receiver port`). (3) *Wi-Fi control*: fields *Access Authentication user* and *password* (password type), **Save camera login** →
  `cameraSecretsSet({…})` → toast `Camera login saved` and both fields cleared (never pre-filled); the fingerprint card when `onFingerprintPending` fired
  since mount or the status is `fingerprint_unconfirmed`/`fingerprint_mismatch`: `Compare with the camera: MENU › Network › Network Option › Access Authen.
  Info` + the value in monospace + **Confirm** / **Not mine** (`cameraFingerprintConfirm/Reject`). (4) *Check*: **Test camera connection** →
  `cameraTestConnection()` → result list: `holder` → `<holder> is holding the camera` + the sentence; else `programs` non-empty → `These programs could take
  the camera: a, b`; else `Nothing is holding the camera`. (5) The agreement line under the section title: `Session Manager sets where the camera saves
  photos: card and computer on the cable, card only on Wi-Fi. The card always keeps a copy.`

## Ordered tasks
0. Services: `camera.ts` functions for every P46–P50 command, `events.ts::onFingerprintPending`, `settings.ts::pickArchive`; `camera_helper_log` command.
1. `CameraDot` + `CameraCard`; CSS states. 2. Health names, actions, `HelperLog`. 3. `CameraSection` + `cameraSteps.ts`; wire into `SettingsPage`.
4. Tests (three files); `npm run typecheck`, `npm test`, Rust suite, clippy, fmt (bindings).

## Criteria (8 criteria, 26 rows; 26 `[rule]` rows)
Harness: vitest + jsdom + Testing Library; `services/*` mocked with `vi.mock` (the P19 pattern); `status(x)` = the mocked `cameraStatus` resolving `x`;
`emit(x)` = the mocked `onCameraStatus` handler called with `x`.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | dot: connected | `status({kind:'connected', name:'ILCE-7RM5', link:'cable'})` | `[data-state="connected"]`, text `ILCE-7RM5 · Cable`; with `link:'wireless'` → `ILCE-7RM5 · Wi-Fi` | `[rule]` 10E §What the photographer gets ("shows the camera, the link (cable or wireless)") |
| C1(b) | dot: connecting | `{kind:'connecting', link:'wireless'}` | `[data-state="connecting"]`, text `Connecting…` | `[rule]` 10E §Port changes |
| C1(c) | dot: busy | `{kind:'busy', reason:'exposure'}` / `'transfer'` / `'switching'` | `Taking a photo` / `Receiving a photo` / `Switching connection` | `[rule]` R-I22 |
| C1(d) | dot: needs action | `{kind:'needs_action', reason:'held_by_other_program', holder:'Capture One', message:"Capture One is holding the camera's USB connection — Take the camera back", link:'wireless'}` | `[data-state="needs_action"]`; the button text is the message truncated to 48 chars + `…`; the card (open on click) shows the full message | `[rule]` 10E §What the photographer gets ("one plain sentence") |
| C1(e) | dot: disconnected | `{kind:'disconnected'}` | `[data-state="disconnected"]`, `Camera disconnected` | `[rule]` S-A |
| C1(f) | dot follows events | `status(connected)`; `emit({kind:'disconnected'})` | the dot re-renders `disconnected` without a re-mount | `[rule]` master plan §6.7 (`camera_status_changed`) |
| C2(a) | take-back button only for Capture One | C1(d)'s status → the card has `Take the camera back`; with `holder:'Photos'` → no such button, `Retry` instead | as stated | `[rule]` MC-34 §Take it back ("the app offers Take the camera back"); §Guide (other programs) |
| C2(b) | click → command → success toast | click `Take the camera back` with `cameraTakeBack` resolving `{completed_steps:5, camera_held:true}` | `cameraTakeBack` called once; while pending the card shows `Taking the camera back… (about 20 s)`; toast `Camera back on the cable` | `[rule]` MC-34 ("It runs only on the photographer's click"); R-I32 |
| C2(c) | blocked and failed toasts | `cameraTakeBack` rejecting `{code:'take_back_blocked', data:{reason:'a file is still arriving'}}` / `{code:'take_back_failed', data:{step:'3', cause:'Capture One did not quit (a dialog may be open)'}}` (every `CommandError.data` value is a string, master plan §6.7) | toasts `Can't take the camera back yet: a file is still arriving` / `Taking the camera back stopped at step 3: Capture One did not quit (a dialog may be open)` | `[rule]` R-I32 ("otherwise it is disabled and says why"; "says which step failed") |
| C2(d) | reconnect and dismiss on the card | `{kind:'needs_action', reason:'files_not_received', link:'wireless', message:'3 photos taken but not received for 60 s'}` → buttons `Reconnect` and `Dismiss`; with `link:'cable'` → `Dismiss` only | clicks call `cameraReconnect` / `cameraFilesNotReceivedDismiss` | `[rule]` R-I29 ("offered as a button, never done automatically"; Wi-Fi only) |
| C3(a) | Health names | `getHealth` resolving items for every §6.6 camera code | each row's label is `Camera` / `Capture One follows` / `Camera support` / `Camera helper` as mapped; details shown verbatim | `[rule]` 12 §Health screen |
| C3(b) | Health actions | items `camera_files_not_received` and `camera_held` | the first row has `Dismiss` → `cameraFilesNotReceivedDismiss` then `getHealth` again; the second has `Take the camera back` → `cameraTakeBack` | `[rule]` R-I29; MC-34 |
| C3(c) | helper log | `cameraHelperLog` resolving 3 lines | a `Camera helper log` disclosure, closed by default; open → the 3 lines in order | `[rule]` 12 §Logs |
| C4(a) | add support | `cameraSupportGet` → null; click `Add camera support → Sony`; `pickArchive` → `/x/CrSDK.zip`; `cameraSupportInstall` → `{make:'sony', version:'2.02.00', …}` | the explanatory text is shown before; `cameraSupportInstall('/x/CrSDK.zip')` called; toast `Camera support for Sony 2.02.00 installed`; the section now reads `Sony Camera Remote SDK 2.02.00 installed` and shows *Transfer from the camera* | `[rule]` 10E §Camera support packages ("explains where to download … takes the downloaded archive … Settings → Camera shows the installed version") |
| C4(b) | install errors | reject `{code:'support_archive_invalid', data:{file:'CrAdapter/libssh2.dylib'}}` / `{code:'support_version_unsupported', data:{version:'2.01.00'}}` | toasts `The archive is missing or has a changed file: CrAdapter/libssh2.dylib` / `This SDK version (2.01.00) is not supported; use 2.02.00` | `[rule]` R-I36 ("refuses the install and names the file") |
| C4(c) | picker cancelled | `pickArchive` → null | no install call, no toast | `[rule]` design handoff (cancel is silent) |
| C5(a) | generate password shown once | click `Generate password`; `cameraFtpPasswordGenerate` → `{password:'abc…32'}` | the card shows the value, `Enter this password on the camera now; it is shown only once.`, `Copy` (writes the clipboard), and the camera steps (first step text contains `[FTP Transfer Func.]` and the user `sessionmanager`); after a re-mount the value is gone | `[rule]` R-I21 ("shows it once in camera setup, for the photographer to enter on the camera"); lab `camera_steps.py` |
| C5(b) | ports save | edit *Receiver port* to `2222`, *Passive ports* `51000`–`51009`; Save | `saveSettings({camera:{receiver_port:2222, passive_port_start:51000, passive_port_end:51009}})` (merged with the form's other fields) | `[rule]` master plan §6.8 |
| C5(c) | invalid ports | `saveSettings` rejecting `{code:'invalid_value'}` | inline error `Ports must be 1024–65535, the passive range at least 2 ports and not containing the receiver port` | `[rule]` master plan §6.8 |
| C6(a) | camera login saved and never shown | type user `u`, password `p`; click `Save camera login`; `cameraSecretsSet` → ok | called with `{access_auth_user:'u', access_auth_password:'p'}`; toast `Camera login saved`; both fields empty afterwards; the password field has `type="password"` | `[rule]` R-I21 ("never in settings.json"); 12 |
| C6(b) | fingerprint card from the event | `onFingerprintPending` handler called with `{fingerprint:'ab:cd:ef'}` | the card appears with `ab:cd:ef` in monospace, the menu path text and `Confirm` / `Not mine`; `Confirm` → `cameraFingerprintConfirm` and the card closes; `Not mine` → `cameraFingerprintReject` | `[rule]` MC-31 §Fingerprint ("shows the camera's fingerprint and asks the photographer to compare it with the camera's menu") |
| C6(c) | fingerprint card from status | mount with `cameraStatus` → `{kind:'needs_action', reason:'fingerprint_unconfirmed'|'fingerprint_mismatch', message:…}` and no event | the status-specific card is shown without Confirm or Not mine and says the fingerprint will appear when the camera reports it; when a real fingerprint arrives by event, its value is shown and both actions appear | `[rule]` R-I37; 10E MC-31 §Fingerprint |
| C7(a) | test connection | click `Test camera connection`; `cameraTestConnection` → `{holder:'Capture One', programs:['Capture One'], sentence:"Capture One is holding …"}` | result `Capture One is holding the camera` + the sentence; with `{holder:null, programs:['Photos'], sentence:null}` → `These programs could take the camera: Photos`; with all empty → `Nothing is holding the camera` | `[rule]` MC-34 §Setup check |
| C8(a) | the agreement line | render the section | text `Session Manager sets where the camera saves photos: card and computer on the cable, card only on Wi-Fi. The card always keeps a copy.` present | `[rule]` MC-31 §Store destination ("The photographer agrees to this once in camera setup") |
| C8(b) | section hidden parts | `cameraSupportGet` → null | *Transfer from the camera* and *Wi-Fi control* are not rendered; *Camera support* and *Check* are | `[rule]` 10E §Camera support packages |
| C8(c) | existing tests | the P19/P07/P08/P22/P20/P26/compose harness mocks (`link:'cable'`) | all existing frontend tests still pass (`npm test` green) | `[rule]` master plan §7.2 (B4) |

## Delegated choices
Icons (lucide) for the dot states and buttons; the card's width; whether the helper log polls; the Copy button's feedback (`Copied` for 1.5 s recommended);
how `cameraSteps.ts` formats the Mac's address line (the backend does not supply an address: show the placeholder text).

## Manual try-out
(Hardware-free except step 4, which the owner may do at P52 instead.) 1. Settings → Camera shows the Sony support card; add the archive the owner has: the
version appears. 2. Generate a password: it is shown once with the camera steps; relaunch: it is gone. 3. Enter the camera login; `settings.json` has no
trace of it. 4. (Camera, optional) Switch the camera on with the cable in: the dot shows the camera and "Cable"; pull the cable: "Wi-Fi"; the first Wi-Fi
connect shows the fingerprint card — compare with the camera menu and confirm.

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, the lab's `sdk_shim` headers, a `.env`, a credential or an SSH fingerprint (the UI never writes files);
  no Python; never change the camera's network/FTP settings (the UI **tells** the photographer what to set on the camera; the app writes only the store
  destination, which this screen discloses), delete a camera file or kill any process; live view out of scope; the suite never needs the camera. No
  `implementer: fable` (owner's rule stands for this frontend phase too).
- **Planner readings (reversible):** (1) the dot's text truncates long sentences at 48 characters; the card has the whole sentence. (2) "Not mine" is the
  reject label. (3) The camera steps are shown beside the generated password, not on a separate page.
- **Hazards:** the `CameraStatusResponse` for `needs_action` must carry `link` (P50 DTO addition) or C2(d) cannot decide; `vi.mock` of `services/camera` must
  include every new export or the module import fails in unrelated suites; the `Hint` card component (P37) is for icon buttons — the dot keeps its own
  popover.
- Registry: §6.11 Frontend. No intention-book change.
- Estimated size: ~17 paths, ~1 600 lines (≈ 800 components, ≈ 800 tests).

## Review log
- 2026-11-08, implement r1 continuation (codex), resumed from `67975d9` under the orchestrator's perimeter ruling below. Updated only the superseded camera-dot assertions in P07 (connected title, disconnected card, click opens card) and P19's camera-service mock so Settings mounts. Targeted P07/P19: 25/25; closing UI suite: 400/400; Rust workspace suite, clippy, fmt and typecheck green. Earlier implementation assumptions and deviations remain as recorded below; no new product choice or backend change. The intermittent P33 case passed in this full run. P51 tracker set to IMPLEMENTED.
- 2026-11-08, implement r1 (codex), blocked at the closing suite gate. Built the five-state camera dot and action card, Health camera labels/actions/helper log, Settings → Camera support, transfer, login, fingerprint and connection check, and 31 passing targeted UI cases. Assumptions: helper log loads on disclosure open; Copy says `Copied` for 1.5 s; the Mac address remains the specified placeholder; fingerprint status without a pending event shows a comparison prompt because the status DTO has no fingerprint value. Camera menu steps were adapted from `Camera-Connection-Tests/camlab/experiments/camera_steps.py`. No backend mechanism deviation. Closing contradiction: C8(c) requires the existing suite green, while P07's inherited camera-dot tests require the former exact title, hover help, and click-to-retry behavior, conflicting with this phase's exact title and click-to-open-card contract. P19's mocked camera service omits the newly required `cameraSupportGet` export, causing two unhandled render errors; its test path is outside this plan's file perimeter. A P33 test failed in both full runs but passed in isolation. Tracker remains NOT_STARTED until the plan expands the test perimeter or resolves the inherited assertions; no IMPLEMENTED claim is made.
- 2026-11-07, planner (Fable), consistency edit from the P48 projection r0 fold (L18): C2(c)'s `take_back_failed` mock now carries `step:'3'` (a string) —
  `CommandError.data` is `BTreeMap<String, String>` (`apps/desktop/src-tauri/src/commands/items.rs:28–32`), so the backend never sends a number; the
  toast text is unchanged. The `camera_held` field this plan already reads from `cameraTakeBack` is now registered in master plan §6.7. Open for P51's
  projection (from the P48 projection's notes): R-I32 says the button "is disabled and says why", while this plan shows the preconditions only as a toast
  after the click; P48 offers no read-only precondition query.

- **2026-11-08, orchestrator ruling on implement r1 (BLOCKED at C8(c)).** P51 deliberately changes the camera dot. The title becomes `Camera connected by cable|Wi-Fi`, and a click opens the card instead of retrying (10E, this plan C1/C2). The older assertions in `apps/desktop/src/features/sessions/__tests__/p07_shell_sessions_ui.test.tsx` that pin the P07 title, hover help and click-to-retry therefore describe superseded behaviour. **The perimeter is extended to that file and to `apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx`.**
  - In p07, update only the camera-dot assertions to P51's behaviour. Keep each test's intent: the dot shows the connected state and the click does what P51 says. Delete nothing else.
  - In p19, add `cameraSupportGet` (and any other new camera service call Settings mounts) to the `@/services/camera` mock so Settings mounts without unhandled errors.
  - No other old test may change.
  - The known intermittent `p33_split_grid` case is not P51's (backlog); if it is red in one full run and green on a rerun, record both.

  Resume from checkpoint `67975d9`: finish C8(c), then close normally.
- **2026-11-08, fix r2 (codex):** F1 resolved via the frontend-only event route. A status-only fingerprint card shows its status title and explains that the value will appear when the camera reports it; Confirm and Not mine are available only after the real fingerprint arrives. C6(c) now covers both status reasons, the no-action status-only state, and the transition to the event value. No getter or backend change.
- **2026-11-08, review r1 (opus): CHANGES_REQUESTED.** Code tree `d8d6993` (checkpoints `67975d9` + `d8d6993`; perimeter = the handoff's list; `Header.tsx` unchanged, not needed). UI 400/400 and typecheck green Rust 816/0, clippy, fmt green. Verified: wire values (`link`, busy and needs-action reasons, error `data` keys, `ServiceError` shape, event name) match the backend; C1(a–f), C2(a–d), C3(a–c), C4(a–c), C5(a–c), C6(a–b), C7(a), C8(a–c) hold at their boundaries, and their tests observe the row's outcome (C8(b) probed with support installed → fails, so it observes the loaded state); the P07/P19 edits stay inside the ruling; H1–H8 hold. **F1 (B1, route production):** `CameraSection.tsx:26–31`. With status `fingerprint_unconfirmed`/`fingerprint_mismatch` and no event since mount, the card shows the literal `Check the camera fingerprint` in the value slot and keeps Confirm enabled. Confirm then pins (mismatch: replaces) a fingerprint that was never shown, against 10E MC-31 §Fingerprint. Fix: no Confirm or Not mine without the real value. Either read the adapter's pending fingerprint through a read-only getter on mount, or wait for `onFingerprintPending`. Extend C6(c) accordingly. Fix may touch `CameraSection.tsx` and its test, and for the getter route `services/camera.ts`, `commands/camera.rs`, `lib.rs`, `bindings.ts` and a read accessor in `direct/adapter.rs`. Notes (backlog): C3(a) substring match; every `invalid_value` shows the ports sentence; the Camera section sits inside the Capture One block; R-I32 "disabled and says why" still open.
- **2026-11-08, re-review r2 (opus): APPROVED_WITH_NOTES.** Fix checkpoint `6145db7`. Perimeter: 5 paths = the fix handoff's list (`CameraSection.tsx`, its p51 test, this plan's C6(c) cell + log, the tracker row, the handoff); no getter-route file changed. The C6(c) cell edit carries out F1's "Extend C6(c)" correction and is accepted. **F1 resolved:** a status-only `fingerprint_unconfirmed`/`fingerprint_mismatch` card shows its title and `The fingerprint will appear when the camera reports it.`; the value, Confirm and Not mine render only once `onFingerprintPending` sets the fingerprint, which is the only writer. The premise of the event route was verified in code: an untrusted fingerprint is non-fatal (`camera_connection.cpp:466–473`), every retry re-emits `ssh.fingerprint` with the value, the runtime emits `FingerprintPending` for each one, and the backoff cap is 30 s (`transport.hpp:92`). The new C6(c) test fails on the round-1 code (Confirm present, literal present). Evidence on code tree `6145db7`: UI: 4 full runs; 2 were 401/401 and 2 were 400/401. The only red in both was the known intermittent `p33_split_grid` C4(d), recorded per this plan's Notes. p51 files 32/32; typecheck, `cargo test --workspace` (exit 0, 0 failed), clippy and fmt green. No blocking item, so the triage is empty. New backlog notes: N5 (a mismatch arriving while Settings is open is titled "Confirm camera fingerprint"; Confirm still replaces correctly) and N6 (Not mine tells the sidecar nothing, so the card returns at each retry).
