---
plan: P51_camera_setup_ui
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-08
actor: opus
---
# Review P51_camera_setup_ui, round 1: CHANGES_REQUESTED

The camera dot, its card, the Health camera rows and Settings → Camera do what the plan says at their boundaries, and the three
test files observe every row. One production defect blocks (F1). When Settings → Camera opens after the camera has already reported
an unconfirmed or changed fingerprint, the card puts the placeholder text `Check the camera fingerprint` where the fingerprint should
be. It still offers **Confirm**, which pins a fingerprint the photographer never saw (10E MC-31 §Fingerprint). Suite, lint and
typecheck: green (see below). Four notes went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
Tracker row `IMPLEMENTED` (2026-11-08, codex). Checkpoints `67975d9` (implementation) and `d8d6993` (continuation under the
orchestrator's perimeter ruling `d64eeec`) are in `git log`, both `CHECKPOINT (not approved):`. HEAD `6299500` differs from `d8d6993`
only by the runner's review prompt. Code tree reviewed = `d8d6993`, `git status --porcelain` clean at start and at end.

## Verified correct
- **Perimeter.** `git show --stat 67975d9` lists 18 paths and `d8d6993` lists 5. Together they equal the handoff's "Files changed"
  list exactly: the 17 code paths of `67975d9`, the plan Review log, P07 and P19 tests (ruling), the master plan tracker row, and the
  handoff. `Header.tsx` is unchanged although the plan listed it; the dot is self-contained, so nothing is missing. No Sony file, no
  `.env`, no credential and no fingerprint file appear in either diff.
- **Suite, lint, typecheck (reviewer run, code tree `d8d6993`):**

  | Command | Result |
  |---|---|
  | `npm test` (apps/desktop) | exit 0; 53 files / 400 tests |
  | `npm run typecheck` | exit 0 |
  | `cargo test --workspace` | exit 0; 816 passed, 0 failed, 0 ignored (first run was cut off by a 10-minute tool limit before reporting; re-run to completion) |
  | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
  | `cargo fmt --all -- --check` | exit 0 |

- **Wire values match the backend.** `link` is `cable`/`wireless` (`Link::label`); busy reasons `exposure`/`transfer`/`switching` and
  needs-action reasons `held_by_other_program`/`files_not_received`/`fingerprint_*` (`commands/camera.rs:203–220`). The error `data`
  keys the UI reads are `reason`, `step`, `cause`, `file` and `version` (`commands/items.rs:90–110`). `unwrap` throws `ServiceError`
  with `code` + `data`, so the mocked rejections in the tests have the production shape. The event name is `camera_fingerprint_pending`
  with `{fingerprint}` (`lib.rs:492–496`).
- **C1(a–f) dot.** `CameraDot` sets `data-state = kind`. Its labels follow the contract: `<name> · Cable|Wi-Fi`, `Connecting…`, the
  three busy texts, and the message cut to 48 characters plus `…`. The title reads `Camera connected by cable|Wi-Fi`. The dot reads
  the status on mount and follows `onCameraStatus`. CSS: connected green (existing), connecting amber with a pulse, busy amber,
  needs_action and disconnected red. The tests observe the accessible name and `data-state`. C1(d) opens the card and finds the full
  message, and C1(f) re-renders on `emit` without a re-mount.
- **C2(a–d) card.** Take the camera back appears only for `held_by_other_program` with holder `Capture One`. Otherwise the card offers
  Retry, or Reconnect (Wi-Fi only) plus Dismiss for `files_not_received`. While pending it shows `Taking the camera back… (about 20 s)`.
  On success it toasts `Camera back on the cable` or, when `camera_held` is false, `Capture One reopened; the camera is still held`.
  Blocked and failed take-backs toast the exact texts. C2(b) holds the promise open and observes the pending text, so it fails if the
  text is missing.
- **C3(a–c) Health.** Every §6.6 camera code is mapped. Dismiss calls `cameraFilesNotReceivedDismiss` and then reloads (`getHealth`
  is called twice). The `camera_held` row offers Take the camera back, with the same toasts as the card. The helper log is a closed
  `<details>` that loads `cameraHelperLog()` when opened. The backend `camera_helper_log` is a thin read of the runtime's ring of 50
  redacted lines (`runtime.rs:54–55, 551–556`), so H8 holds.
- **C4(a–c) support.** The explanatory text, `pickArchive` (`.zip`), `cameraSupportInstall(path)`, the success toast, the installed
  line and the appearance of *Transfer from the camera* all hold. Both error toasts name the file or the version. A cancelled picker
  makes no call and shows no toast.
- **C5(a–c) transfer.** The password is held in component state only, so a re-mount clears it. The card shows the shown-once sentence,
  Copy (clipboard, `Copied` for 1.5 s) and the steps. `cameraSteps.ftp` matches `camlab/experiments/camera_steps.py:20–38`, adapted to
  `sessionmanager` and the placeholder address. The ports are saved through the form's Save (`camera: settings.camera`), merged with
  the other fields. `invalid_value` shows the inline ports sentence (but see note N2).
- **C6(a–b) login and fingerprint from the event.** The fields are never pre-filled and are cleared after `cameraSecretsSet`, and the
  password field has `type="password"`. A fingerprint arriving by event renders in `<code>` with the menu path, Confirm and Not mine.
  Confirm closes the card.
- **C7(a), C8(a–b).** The three connection-check outcomes and the agreement line render as specified. *Transfer* and *Wi-Fi control*
  are hidden without support. A reviewer probe ran a temporary copy of C8(b) with `cameraSupportGet` resolving installed. It failed,
  so the row observes the loaded state and can fail. The copy was deleted.
- **C8(c), P07/P19 edits.** Only the three camera-dot tests in P07 and the camera mock in P19 changed, as the ruling allowed. Each test
  keeps its intent: the connected state, and a click opening the card with Retry.
- **Hard constraints H1–H8.** The UI writes no file and changes no camera setting; it only tells the photographer what to set. Secrets
  go only to `cameraSecretsSet` and are never shown back. The FTP password is shown once by design (R-I21). No platform write and no
  UI automation are added. The new Tauri command is thin.

## Findings

### F1 (blocking, B1). Route: production. Fingerprint Confirm is offered without the fingerprint
- **What is wrong.** `CameraSection.tsx:26–31`. When `cameraStatus` reports `fingerprint_unconfirmed` or `fingerprint_mismatch` and
  no `camera_fingerprint_pending` event has arrived since mount, the card sets the fingerprint to the literal
  `'Check the camera fingerprint'`. It renders that literal in the monospace value slot, under "Compare with the camera: MENU › …",
  and keeps **Confirm** enabled. `cameraFingerprintConfirm` succeeds whenever the adapter holds a pending fingerprint
  (`adapter.rs:278–289`), and it does in exactly this state (`adapter.rs:598–608`). This is the ordinary path. The event fires when
  the camera first connects over Wi-Fi. The photographer sees the dot's sentence and opens Settings afterwards, so the event fired
  before the section mounted. For `fingerprint_mismatch`, Confirm sends `replace=1`, which unpins the old key without the new one ever
  being shown.
- **Authority.** 10E MC-31 §Fingerprint: "the app shows the camera's fingerprint and asks the photographer to compare it with the
  camera's menu; it is pinned only on their confirmation". The handoff's fourth assumption makes the status-only card a confirmation
  prompt with no value. Under flow §6 an assumption that contradicts the intention is B1.
- **Correction.** Never offer Confirm (or Not mine) while no real fingerprint value is shown. Either read the pending value on mount
  through a read-only getter, or render the status-only card with its title and a line saying the fingerprint appears when the camera
  next reports it, with no Confirm until `onFingerprintPending` delivers the value. The getter returns the adapter's
  `pending_fingerprint` value, for example `camera_fingerprint_pending_get -> Option<String>`; it is the robust route because the
  value is already held. The second route is frontend-only, because the sidecar re-reports on its backoff retries. Extend C6(c): a
  status-only mount must not offer Confirm unless the real value is displayed.
- **Files the fix may touch.** `apps/desktop/src/features/settings/CameraSection.tsx`,
  `apps/desktop/src/features/settings/__tests__/p51_camera_section.test.tsx`. For the getter route also
  `apps/desktop/src/services/camera.ts`, `apps/desktop/src-tauri/src/commands/camera.rs`, `apps/desktop/src-tauri/src/lib.rs`,
  `apps/desktop/src/services/bindings.ts` (regenerated) and `crates/integrations/src/camera/direct/adapter.rs` (a read accessor only).

## Notes written to backlog
- N1. C3(a) checks the 12 plain `Camera` labels with `toHaveTextContent('Camera')`, a substring match, so a code mapped by mistake to
  `Camera support` or `Camera helper` still passes. It does catch the realistic defect, an unmapped code showing its raw name.
- N2. `SettingsPage` maps every `invalid_value` from Save to the ports sentence. The backend also raises `invalid_value` for
  `output_format`, `default_shape`, `master_recipe` and `document_id` (`services/settings.rs`). A bad JPEG quality (for example 50) now
  shows "Ports must be 1024–65535…". The plan's wording allows this; the backend message names the key (`invalid value: camera`), so
  the UI can tell them apart.
- N3. `<CameraSection>` is inserted between *Style* and *Default shape*, inside the Capture One block, so *Default shape* renders under
  the Camera section. The plan says "after Capture One".
- N4. R-I32 ("otherwise it is disabled and says why") is still unmet. Take the camera back is enabled and explains only after the
  click, by toast. This was already open from the P48 projection; it needs a read-only precondition query to close.

## Carry-forward dispositions
| Note | Destination |
|---|---|
| N1–N3 | P51 fix 1 may take N2/N3 if the orchestrator widens the perimeter; otherwise the next frontend polish phase |
| N4 | P52 (hardware acceptance) or a later backend phase adding the precondition query |

## Lessons for the plans
- C6's contract required "the value in monospace" on a card the plan also opens from the status, and the status DTO has no
  fingerprint value. The plan should have named the value's source for the status-only path, or said Confirm waits for the event.
  Projection was off for this phase; this is the class of gap projection catches.
- The C5(c) wording ("`invalid_value` → inline ports error") reads as unconditional. Plans that map a shared error code to a
  field-specific sentence should name the discriminator.

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P51_camera_setup_ui_review_r1.md` (this file)
- `docs/implementation/plans/P51_camera_setup_ui.md` (Review log entry appended)
- `docs/implementation/master_plan.md` (P51 tracker row → `CHANGES_REQUESTED`)
- `docs/implementation/backlog.md` (N1–N4 appended)
- Probe: `apps/desktop/src/features/settings/__tests__/zz_reviewer_probe.test.tsx` was created, run and deleted (`git status`
  clean afterwards). No production file was touched. No database or app state was used.
