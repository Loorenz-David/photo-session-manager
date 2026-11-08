---
plan: P50_camera_settings_and_support
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-08
actor: opus
---
# Review P50_camera_settings_and_support, round 1: CHANGES_REQUESTED

The support-package installer, the manifest check, the Keychain commands, the FTP password and the fingerprint flow all do what the
plan says at their boundaries. One production defect blocks (F1). The new startup path that keeps the Keychain from delaying the
camera start never passes credentials read during a USB session to the sidecar, not even at its next spawn. One test cell asserts its
own fixture (F2). Suite, lint and typecheck: green (Rust 814 passed / 0 failed, clippy, fmt, typecheck, UI 369/369).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
Tracker row `IMPLEMENTED` (2026-11-08, codex); checkpoint `06f5e61` (`CHECKPOINT (not approved): implement P50 camera settings and
support`) is in `git log`. HEAD `6efbba3` differs from it only by the runner's review prompt (`git diff 06f5e61 HEAD --name-only`), so
the code tree reviewed = `06f5e61`. `git status --porcelain` was clean at review start.

## Verified correct
- **Perimeter.** `git show --stat 06f5e61` = 33 paths = the handoff's "Files changed" list exactly. Deviations are disclosed (CMakeLists
  `LC_RPATH`, `ports/secrets.rs` + `fake_secrets.rs` `cached` seam, `items.rs` error mapping, Cargo manifests + lock; `CameraDot.tsx`
  unchanged because `link` is optional). No Sony byte in the repo: the manifest holds seven names + SHA-256 only; no `.env`, no
  credential, no fingerprint file in the diff.
- **Suite, lint, typecheck (reviewer run, code tree `06f5e61`):** 

  | Command | Result |
  |---|---|
  | `cargo test --workspace` | exit 0; 814 passed, 0 failed (p50 20/20 in 67.8 s; desktop `p50_tests` 2/2; unit `c2a_missing_and_c2b_hash_mismatch`, `c6a_password_shape_and_rejection`) |
  | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
  | `cargo fmt --all -- --check` | exit 0 |
  | `npm run typecheck` (Node 22) | exit 0 |
  | `npm test` | exit 0; 50 files / 369 tests |

  `git status` after the runs: the bindings export left `bindings.ts` unchanged.
- **Installer (`support/sony.rs`, `manifest.rs`), read against R-I36 and the plan contract:** non-zip → `SupportArchiveInvalid{archive
  name}`; every entry name is checked for `..` and a leading `/` (after `\`→`/`) before anything is written; per manifest file the first
  archive entry whose path ends with `/<path>` wins (lab glob rule); missing / hash mismatch → `SupportArchiveInvalid{manifest path}`,
  verified on the buffered bytes before any write; files written by the Rust process (no quarantine xattr can reach them); `CrAdapter/*`
  also written under `Contents/Frameworks/CrAdapter/`; bundled helper copied, mode 0o755; signing by Mach-O magic (the four magics), the
  helper with `--options runtime --entitlements` (`com.apple.security.cs.disable-library-validation`, P47 S2 →
  `SIDECAR_NEEDS_LIBRARY_VALIDATION_OFF = true`); `version` with a 3 s timeout, `sdk_compiled_in:false` → `"unavailable"`, unsupported →
  `SupportVersionUnsupported{version}`; every refusal drops the `TempDir` (no `.install-*` leftover); success renames an existing version
  folder aside, renames temp into place, removes the old one. `installed()` requires every manifest file plus an executable
  `camera-sidecar` in the folder named for the manifest's version; `damaged()` = a non-dot version folder exists and `installed()` is
  `None` → health `camera_support_not_installed` with the §6.6 detail, cleared otherwise; the Capture One adapter stays in use.
- **Secrets.** `CameraService::set_secrets` writes `access-auth-user` / `access-auth-password` only to the `SecretStore`; the command
  registers both with the redactor and `secrets_changed` does too; `secrets_changed` puts the new line into the running supervisor's
  spec and restarts only off USB (`quit_and_wait` + spawn). `generate_ftp_password` = `ftp_password(getrandom)` (62-symbol alphabet,
  bytes ≥ 248 rejected), stored under `ftp-password`, registered, receiver stopped and restarted on the same port (port 0 keeps the old
  port), returned once. No command reads `ftp-password` back (grep: only `bind` and `ftp_password_changed` read it).
- **Fingerprint.** C++: `ssh.fingerprint` carries `fingerprint` (lowercase hex with `:`), the stderr print of the value is gone,
  `accept-fingerprint [replace=1]` arms the next connect (replace unlinks the pinned file first); an untrusted fingerprint is now
  non-fatal so the sidecar's backoff retries (the fingerprint check runs before any credential is sent, so this is not the "auth failure
  never retried" case, which stays fatal). Rust: `unknown` → `FingerprintUnconfirmed` + `FingerprintPending{fp}`, `MISMATCH` →
  `FingerprintMismatch` + `FingerprintPending{fp}` with `replace = true`; confirm sends `AcceptFingerprint{replace}`; reject sends
  nothing; `match`/`pinned` clear both. `TauriEvents` maps the event to `camera_fingerprint_pending {fingerprint}`.
- **Commands.** The six commands are in `collect_commands!`; wire shapes as master plan §6.7; error codes `support_archive_invalid{file}`,
  `support_version_unsupported{version}`, `support_not_installed`, `fingerprint_not_pending` mapped in `items.rs`.
  `needs_action.link` is the current session's link.
- **P47 answers.** (a) entitlement signed; (b) no quarantine removal anywhere; (e) `BUILD_WITH_INSTALL_RPATH TRUE` leaves only
  `@executable_path` (the implementer's `otool` check); (f) the staging folder is removed on clean shutdown only when empty
  (`fs::remove_dir`), so no camera file is touched (H2). (d) is F1.
- **Criterion rows.** Every row's test was read against the production code that makes it true:
  C1(a) bytes, Frameworks copies, no quarantine on any copy (xattr set on the archive by the test), helper SHA, exec bit,
  `codesign --verify`, `installed()`; C1(b) one pid, `argv[0]` = the installed helper, receiver port ≠ 0, direct adapter;
  C2(a)–(e) exact error + named file/version and no `camera/sony`, no `.install-*`; C2(a)/(b) also at the `verify` unit;
  C3(a) refreshed SHA + verify, then mtime unchanged on an untouched restart; C4(a) health detail, Capture One adapter, no spawn;
  C4(b); C5(b) `quit` to the first pid, second pid's first line `credentials user=u3 password=p3`; C5(c) no quit, no second pid on
  USB, then the backoff spawn carries `u4/p4`; C6(a) the fixed stream `[248,0,61,247]` gives `A99…A9` (a non-rejecting mapping would
  give `AA9…`), 1 000 seeded runs all 32 × `[A-Za-z0-9]`; C6(b) Keychain value, generation +1, same port, `530` for a wrong password and
  `230` for the new one, no sidecar restart, not in `settings.json`; C6(c) old `530`, new `230`; C7(a)–(e) events, statuses, details,
  `accept-fingerprint cmd_id=` / `replace=1`, nothing on reject, no `usb-retry`/`reconnect`/`accept-fingerprint` after an auth refusal;
  C8(a) the four wire shapes through the mock webview, incl. the error body `{"code":"support_archive_invalid",…,"data":{"file":
  "CrAdapter/libssh2.dylib"}}`; C8(b) one `{"fingerprint":"ab:cd:ef"}` payload. Each would fail on a wrong outcome, except the C5(a)
  redaction cell (F2).
- **Hard constraints.** H1 (no platform write), H2 (no camera file moved or deleted; install removes only its temp and replaced folders;
  staging only when empty), H3, H4 (install is all-or-nothing by rename), H5, H6 (nothing installed = normal Capture One mode, C4(b)), H7
  (core gains only neutral names: `FingerprintPending`, `Support*` errors, `ftp_password`), H8 (see the backlog smell note; not a
  violation: the logic is in integrations and core, the commands compose it). The suite needs no camera, Capture One or network.

## Findings

### F1 (blocking, B1), route: production. Credentials read at startup during a USB session never reach the sidecar
**What is wrong.** P50 (d) moved the Keychain read off the start path: in production `KeychainSecrets::cached` is always `None`, so
`bind` spawns the sidecar with `credentials user= password=` and a thread reads the Keychain afterwards
(`crates/integrations/src/camera/runtime.rs`, the `if self.credentials…is_none()` block after `*guard = Some(SidecarClient::new(…))`).
That thread stores the values in `self.credentials` and calls `restart_if_not_usb()`, which returns at once when a USB session is open.
Unlike `secrets_changed()`, it never writes the new line into the running supervisor's `spec.first_line`. The supervisor's backoff
respawn reuses its `spec` (`supervisor.rs` `spawn`), so **every later spawn still sends the empty line**, and the running process keeps
the empty credentials too. With an empty user the sidecar connects over Wi-Fi without SSH (`camera_connection.cpp`
`ssh = opt.ssh_user.empty() ? 0 : 1`), so after the cable is pulled the photographer cannot connect over Wi-Fi until the app is
relaunched. This is the exact case (d) was written for: a Keychain prompt (after every update, S6) lets the USB session open first.
Implementer assumption 4 says "a USB session uses them on its next spawn". The code does not do that.
**Authority.** 10E R-I18 ("camera credentials → used at the sidecar's next spawn; the sidecar is restarted for them only while it holds
no USB session"); plan Review log, P47 answer (d) ("Spawn on USB without waiting, and pass the credentials when read"); plan Review log
carried L17 ("the next spawn, immediate (off USB) or later (on USB), sends the new line").
**Correction.** When the startup read returns, put the credentials into the running supervisor's spawn spec, as `secrets_changed`
does. If a USB session is open, also restart the sidecar the first time it holds no USB session (session lost) while the running
process still has the empty line, so the read credentials reach the sidecar at the first moment R-I18 allows. Add one boundary test
with a secret store whose `cached` returns `None`, as the Keychain does: USB script, stored credentials, then `session.lost usb` →
a second process whose first line carries them.
**Files the fix may touch.** `crates/integrations/src/camera/runtime.rs`; `crates/testkit/src/fake_secrets.rs` (a Keychain-like mode
with no cache); `crates/testkit/src/test_app.rs` (a constructor for it, if needed); `crates/testkit/tests/p50_camera_settings_and_support.rs`.

### F2 (blocking, B2), route: production (test). The C5(a) redaction cell asserts its own fixture
**What is wrong.** C5(a)'s outcome includes `logging::redact_registered("x p2 y") == "x [REDACTED] y"`. The test helper
`setting_secrets` calls `psm_infra::logging::add_secret(user)` and `add_secret(password)` itself before `secrets_changed()`, so the cell
stays green even if neither the command nor the runtime registered the secret. Production does register it today (command and
`secrets_changed`, verified by reading), but the cell cannot fail on a wrong outcome, and that wrong outcome (a camera password in the
log) is what M39 measures.
**Authority.** flow.md §5 B2 ("asserts the fixture"); plan row C5(a) (trace R-I21, M39).
**Correction.** Remove the two `add_secret` calls from `setting_secrets`'s closure. The cell then observes the registration that
production's `secrets_changed` performs.
**Files the fix may touch.** `crates/testkit/tests/p50_camera_settings_and_support.rs`.

## Assumptions (flow §6)
1. Bundled helper beside the app executable, dev fallback `binaries/`: matches Tauri's `externalBin` layout. Fine.
2. ZIP buffered; absolute and `..` names refused: within Delegated choices; the absolute-name refusal is stricter than the hazard text
   (backlog note).
3. "Same name and signing options give the same SHA": **false for the packaged build.** Tauri signs the bundled helper without the
   entitlement, so the copy never matches and is re-copied and re-signed at every spawn (backlog note; not blocking: the copy is always
   correct, about 20 ms per start).
4. USB starts with empty credentials, and the values are used at the next spawn: contradicted by the code → F1.
5. Staging folder removed only when empty: consistent with H2.

## Notes written to backlog
Eight P50 reviewer r1 entries in `docs/implementation/backlog.md`:
- the SHA comparison can never match in a packaged build (refresh-always; plan lesson);
- repeated `camera_fingerprint_pending` events (→ P51);
- `accept-fingerprint` pins whatever key comes next (hardening);
- C6(b)'s "no command returns it" cell holds by structure;
- test seams in production code;
- absolute-name refusal and whole-archive buffering;
- H8 smell (orchestration in commands, duplicated in the testkit);
- three p50 tests take about 65 s each (suite time).

## Lessons for the plans
- A row cannot require both "the copy has the same SHA-256 as the bundled helper" (C1(a), C3(a)) and "signed after copying" (R-I36).
  In a packaged build the two signatures differ. The refresh check should compare against a stored source hash.
- When a phase moves a read off a start path (P47 (d)), the plan needs a row for the slow-read case. The suite's `FakeSecrets` answers
  `cached` synchronously, so the production path had no test at all. F1 adds the first one.

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P50_camera_settings_and_support_review_r1.md` (this file, new)
- `docs/implementation/master_plan.md` (P50 tracker row only → `CHANGES_REQUESTED`)
- `docs/implementation/plans/P50_camera_settings_and_support.md` (Review log entry appended)
- `docs/implementation/backlog.md` (eight entries appended)
- No production or test file edited. Scratch only: `/tmp/p50rev/` (suite logs; `sim/` held a copy of the packaged app's
  `camera-sidecar`, signed there to compare hashes, outside the repo). Build artefacts from the suite run under `target/` (untracked).
  No database, Keychain or settings state touched.
