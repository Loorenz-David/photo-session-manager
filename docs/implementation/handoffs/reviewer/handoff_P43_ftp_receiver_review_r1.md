---
plan: P43_ftp_receiver
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-06
actor: opus
---

# P43 FTP receiver — review r1

The receiver works as R-I24 says on 24 of the 27 rows. Two blocking findings remain. **F1 (B4):** the Rust suite is red at random,
because this phase's test client resets the connection in a different way from the lab. The receiver is right, and the test fixture
is wrong. **F2 (B1):** under the production `0.0.0.0` bind, the passive-mode reply gives the camera the address `0.0.0.0`. Both
fixes are small and sit in one file each.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Gate and evidence

- Tracker `P43_ftp_receiver | IMPLEMENTED`; checkpoint `e3569bed1c7daa6e486210c13f335a1eb159420e` is in `git log`. Gate passes.
- Tree: HEAD `c0147c9`, `git status --porcelain` empty. `git diff e3569be HEAD` touches only runner docs (STATUS, prompt, implementer handoff, state), so the code tree equals the checkpoint. Re-running was needed because the implementer's stamp predates the runner commits, and their green claim did not hold up (below).
- `git show --stat e3569be` lists **33 paths**, the same as the handoff's "Files changed" list (the handoff file itself is runner-owned and was not committed). The declared deviations are justified, so they are not findings: `Cargo.lock` (`ipnet`), 8 frontend test fixtures (`camera` is a required `SettingsDto` field), and `crates/infra/src/ftp/names.rs` (named by Ordered task 1 but missing from the file list).
- Commands (master plan §7.2), one run each: `cargo clippy --workspace --all-targets -- -D warnings` exit 0 · `cargo fmt --all -- --check` exit 0 · `npm run typecheck` exit 0 · `npm test` 369/369 · `cargo test --workspace` **exit 101**: `p43_ftp_receiver::c2a_reset_is_failure` panicked (`WouldBlock` after 15 s).
- Variation (authorised by the red result, to tell a flake from a deterministic failure): `cargo test -p psm-testkit --test p43_ftp_receiver` ×6 gave 4 red runs (C2(a), C2(b), C2(c); once also C7(b)). `… c2a` alone ×8 gave 4 red runs. C2 failed two ways: `226` where `426` was expected (C2(a) line 157, C2(c) line 196), and a 15 s control read timeout (C2(a) line 158, C2(b) line 174).

## Findings

### F1 — B4 — route: production — the reset fixture is not a reset (suite red ~50%)
- **What is wrong.** `crates/testkit/src/ftp_client.rs::stor_rst` writes the bytes, sets `SO_LINGER {1,0}`, then calls **`libc::shutdown(fd, SHUT_RDWR)`** before dropping the socket, and it never pauses between the data and the reset.
  (1) `shutdown(SHUT_RDWR)` sends a **FIN** before the RST. When the FIN arrives first, the server correctly sees "data connection closed with no socket error" and answers `226`. That is the C2(a)/C2(c) `226` failure.
  (2) Without the lab's pause, the RST can arrive while the server still has unread data. The reset is then lost at the TCP level, the server waits for its 600 s data idle timeout, and the client's 15 s read times out. That is the `WouldBlock` failure.
- **Evidence the receiver is not at fault.** I ran a scratch probe (a temporary test file, deleted, listed under Write perimeter) against the unchanged receiver:
  - shutdown-then-RST: 15/15 `426` in that batch. It races in the suite.
  - RST with no pause, 300 ms delay before writing: 11/15 got no reply within 3 s. With a 40 s wait, one upload was still open after 40 s, `uploads_in_progress() == 1`, and a retry got `450`.
  - **The lab's shape** (write, pause 300 ms, linger-close, no shutdown): **25/25 `426`**, 0 deliveries.
- **Authority.** flow.md §5 B4 (suite red). Plan Ordered task 5: `stor_rst` "sends `bytes` then sets `SO_LINGER {1, 0}` … and closes so the server sees `RST`". Lab `tests/python/test_ftp_server.py::_interrupted_upload`: `sendall`, `time.sleep(0.3)  # let the server accept the data connection and start reading`, linger-close `# RST, not FIN`.
- **Correction.** Drop the `shutdown` call. Close with linger 0 only, after waiting until the server has read the bytes: either the lab's 0.3 s pause, or a caller-side `settle` until `part(name)` reaches `bytes.len()`. The C2 rows then run green on every run.
- **Files the fix may touch.** `crates/testkit/src/ftp_client.rs`. Touch `crates/testkit/tests/p43_ftp_receiver.rs` only if the wait is done by predicate in the tests.

### F2 — B1 — route: production — PASV tells the camera to connect to 0.0.0.0
- **What is wrong.** `crates/infra/src/ftp/session.rs` (`"PASV" | "EPSV"` arm) builds the `227` address from `state.config.bind`'s octets. Production binds `0.0.0.0` (`App::open` → `CameraRuntime::new` → `Ipv4Addr::UNSPECIFIED`), so the camera receives `227 Entering Passive Mode (0,0,0,0,p1,p2)`. A client that honours the reply address cannot open the data connection, so no wireless upload ever starts. The tests bind `127.0.0.1`, which is why no row sees it. Nothing records whether the Sony client honours the PASV address or uses EPSV (the lab has no command log, and S7 is P47).
- **Authority.** 10E MC-32 §5 ("passive mode only on a fixed port range", which only works if the passive reply can be reached). The semantics being ported: the lab's pyftpdlib runs with `masquerade_address=None`, so its `227` carries the control connection's local address, the address the camera already reached, and that receiver worked with the real camera.
- **Correction.** Advertise the control connection's local IPv4 address (`control.get_ref().local_addr()?.ip()`) in the `227` reply instead of the bind address. Loopback tests keep answering `127,0,0,1`.
- **Test.** None required. No row covers the reply address, and loopback cannot discriminate. Binding `0.0.0.0` in a test triggers the macOS firewall prompt (plan Hazards). The re-review verifies by reading.
- **Files the fix may touch.** `crates/infra/src/ftp/session.rs`.

## Verified correct

- **Completion/failure split (MC-32 §1, F-1).** `upload()` reads until `Ok(0)`, polls `take_error()` every 100 ms while waiting, and checks `take_error()` again after EOF before `226`. Reset, timeout (600 s), stop and control-close all go to `426`. The `.part` is never removed (no `remove_*` anywhere in `ftp/`). `UploadGuard` frees the name and decrements `uploads` on every path.
- **Staging (MC-32 §2, R-I24).** `.incoming/.<name>.part` under `capture_root()`. `root/<name>` is never written. The `Delivery` fields match the row exactly (C1(a) asserts the whole struct). `completed_at` is taken at the close, before the hash. The hash runs on `spawn_blocking`.
- **Order of refusals.** 553 (base name / hidden / `.part` / `..`, case-insensitive `.part`) → 451 (root `None` or not a dir) → 450 (upper-case key in flight) → 554 (`REST`/`APPE` offset > `.part` length; `.part` untouched, pending `REST` cleared by `rest.take()`). A new `STOR` truncates. `REST n` truncates to n and appends.
- **Namespace.** `MKD`/`CWD`/`CDUP` succeed and are virtual. `LIST`/`NLST`/`SIZE`/`MDTM` read `completed_names_since(now − COMPLETED_NAMES_WINDOW_MS)`. `MDTM` is UTC `%Y%m%d%H%M%S` (C4(b)'s `20231114221320` for `1_700_000_000_000` checked). `DELE`/`RMD`/`RNFR` give 550, `RNTO` gives 553.
- **Network/auth.** Peers are checked before the banner (`421`, then the socket closes). Data connections are accepted only from the control peer's IP. `PORT`/`EPRT`/unknown give 502. Everything before login except `USER/PASS/QUIT/FEAT/SYST/NOOP` gives 530. The password compare runs in constant time. A failed login logs `user=sessionmanager|<other>` only. `logging::add_secret(password)` runs before `FtpReceiver::start`. The default peer list equals R-I24 exactly (loopback excluded).
- **Runtime (R-I18, R-I24).** The receiver starts only when a sidecar is installed **and** a non-empty `ftp-password` exists. The capture root is `canon(<effective document id>/Capture)` from settings, the same formula as `adapter.rs:101` and MC-2, and is never taken from Capture One. A changed `CameraSettings` causes `stop()` then `start()` (one listener). Equal settings leave it untouched, and a failed start retries on the next bind. `shutdown()` stops the receiver before the sidecar.
- **Settings.** `camera` is in the allowlist. Validation: ports 1024–65535, start ≤ end, at least 2 ports, receiver port outside the range, else `InvalidValue("camera")`. `#[serde(default)]` fills it from an older `settings.json`. The DTO and bindings changes are additive.
- **Tests.** The other 24 rows each observe the row's outcome at the named boundary and would fail on a wrong one: C1(a)–(c), C2(d)–(g), C3(a)–(d), C4(a)–(c), C5(a)–(d), C6(a)–(b), C7(a)–(c), C8(a). C2(d) and C2(g) also open with `stor_rst` and assert its `426`. None of my runs saw them fail (64 KiB payloads fit in the socket buffers), but they share F1's fixture, so F1's fix covers them too. No orphan tests: every test in `p43_ftp_receiver.rs` and the two unit tests map to a row.
- **Assumptions** (handoff) are all inside Delegated choices, and none contradicts the book: hash on the blocking pool, UTC `LIST` dates, `FEAT` contents, 600 s data timeout and no control cap, `NoopSink` at debug, `u64` generation, the free-port method, infra-side `CompletedNames`.
- **Hard constraints.** H1 no external writes. H2 writes only `.part` files under the capture root's `.incoming/`, never deletes, traversal refused. H3 no pixel work. H4 resume semantics are idempotent. H5 none. H6 not applicable. H7 is a plan-mandated placement, see the lesson. H8: Tauri change is DTO-only. Secrets stay out of logs, argv and settings, and tests use `FakeSecrets` and loopback only. No live services.

## Notes written to backlog (11)

`docs/implementation/backlog.md`, P43 rows:
- any control line during an upload aborts it;
- an accept error ends the listener silently;
- no control idle timeout or cap, so a lost RST holds the name and the upload counter up to 600 s (probe-observed);
- `226` comes after the hash and delivery;
- `APPE` with no `.part` answers 554;
- the §6.11 registry signature differs from the tree;
- IPv6 peers cannot reach the IPv4 bind;
- a password change does not reopen the listener (P50);
- C2(b) does not assert `size`, and C7(b) does not connect on the new port;
- `c7b` flaked once on its local 3 s settle;
- H7 lesson: the FTP ports sit in core.

## Lessons for the plans

- When a row ports a lab test that depends on timing, the plan should quote the lab's exact sequence, including its pauses. Task 5 paraphrased `_interrupted_upload` without the 0.3 s pause, and the implementer added a `shutdown` the lab does not have.
- A server that binds an unspecified address needs its advertised addresses (PASV) specified in the plan. Loopback tests cannot catch this class of defect.
- H7: the planner placed FTP-specific settings (`receiver_port`, passive range) in core `domain::camera`. Decide before P51 whether they belong in an adapter-owned block.

## Write perimeter

- `docs/implementation/master_plan.md`: P43 tracker row only.
- `docs/implementation/plans/P43_ftp_receiver.md`: Review log entry appended.
- `docs/implementation/backlog.md`: 11 P43 rows inserted after the P42 rows.
- `docs/implementation/handoffs/reviewer/handoff_P43_ftp_receiver_review_r1.md`: this file.
- **Probe files (created, run, deleted; never committed):** `crates/testkit/tests/zz_review_probe_p43.rs`, written three times and removed after each run. `git status --porcelain` showed only the doc files above afterwards. Production code was not touched. The probes used only `127.0.0.1` and temp dirs, with no camera and no Capture One.
- Build artefacts in `target/` from the probe compiles (not tracked).
- No narrative: it is written at approval.
