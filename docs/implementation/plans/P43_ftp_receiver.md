---
phase: P43_ftp_receiver
title: FTP receiver — the Rust passive-only FTP server the camera's Auto FTP uploads into (staging, failed-on-reset, resume, virtual namespace, refusals, private peers), the lab's 18 receiver tests ported to a testkit FTP client, the camera settings block applied in place
difficulty: high
projection: no
depends_on: [P42_camera_runtime]
---
# P43 — FTP receiver (MC-32)

## Goal
`psm_infra::ftp::FtpReceiver`: the in-process FTP server that receives the camera's Auto FTP uploads (10E MC-32 with inventory additions R-I24;
M37's "an aborted transfer yields none under a candidate name"): `STOR` writes `<capture root>/.incoming/.<name>.part`; **completed** means the
data connection closed with no socket error, anything else is a failure that keeps the `.part` for a resume; on completion the file is hashed
outside the transfer timing and handed to the `CaptureSink` as `{camera_filename, written_path = the .part, sha256, size, via: Wireless,
completed_at}`; a virtual namespace (`MKD`/`CWD` succeed and are ignored, listings and `SIZE`/`MDTM` answer from the names completed in the last
24 h); `DELE`, `RMD`, every rename and hidden/staging names refused; one upload per name at a time; `REST` resume onto an existing `.part`;
passive mode only on a fixed port range; private-network peers only; authentication with the Keychain password; at least two concurrent uploads.
Plus the `camera` settings block (receiver port, passive range) validated and applied **in place** by the runtime (R-I18: the listener is closed and
reopened; the sidecar untouched; never two listeners).
**Not in this phase:** the rename into the target folder, duplicates and `__dupN` (the ledger, P44 — here the sink is a stand-in that records
deliveries), FTPS (owner D5: later), the sidecar, the health items' UI, the FTP password generation and the camera setup text (P50/P51).

## Read first
Book: `10E_direct_camera_adapter.md` MC-32 (guarantees 1–5 and the inventory additions **R-I24**: namespace, upload lifecycle, network, capture
root without Capture One), MC-31 inventory R-I18 (a camera setting changed is applied in place), §Invariants M37, M39 · `05A_mechanism_contracts.md`
MC-2 (capture root `canon(<document folder>/Capture)`; `.incoming` ignored by the candidate filter) · `14_measurement_ledger.md` M37, M39 ·
evidence `sony_camera_lab.md` C5, C6 (F-1), H-11 · process `flow.md` §4, §6, §9 · master plan §3, §6.1, §6.2 (`ipnet`; no FTP crate), §6.3
(`.incoming/.<name>.part`), §6.8 (`camera` block), §6.9, **§6.11** (Constants, Ports `CaptureSink`/`ArrivalSource`, Receiver, Runtime `bind` (d)), §7.5.
Code at `41bef14` (+ P42's additions by their §6.11 names):
- `crates/infra/src/nudge.rs:1–80` — the pattern for a tokio server owned by a std thread with a oneshot stop (`NudgeServer::start`); the receiver
  follows it (its own multi-thread runtime, `TcpListener::bind`, graceful stop).
- `crates/core/src/mechanisms/hash.rs:5` (`sha256_file`), `crates/core/src/mechanisms/canon.rs`, `crates/core/src/mechanisms/arrival.rs:297–321`
  (`is_candidate`: dot components rejected — the `.part` and `.incoming/` are invisible to arrival).
- `crates/infra/src/settings_file.rs:11–28` (`Settings`, `#[serde(default)]`), `:48–66` (`Default`); `crates/core/src/services/settings.rs:36–120`
  (`SettingsService::set` key allowlist `:41–53` and validations); `apps/desktop/src-tauri/src/commands/settings.rs:28–73` (`SettingsDto`,
  `SettingsPatchDto`); `apps/desktop/src/features/settings/SettingsPage.tsx` (unchanged in this phase: the `camera` block is not shown until P51;
  `SettingsDto.camera` is additive and the page ignores it).
- P42: `CameraRuntime::bind` (§6.11 Runtime (c), (d)), `TestApp::with_camera`, `FakeSecrets` (`ftp-password = f`).
- Lab (read-only): `camlab/ftp/server.py` (the semantics being ported: `AbortAwareDTPMixin` `:118–151` — a reset or pending `SO_ERROR` aborts (426),
  `ftp_STOR` `:216–248` (hidden names 553, staging path), `on_file_received` `:250–274`, `on_incomplete_file_received` `:323–342`, `ftp_RNTO` `:345–352`),
  `camlab/ftp/config.py:10–21` (`DEFAULT_ALLOWED_NETWORKS` — **ours omits loopback and `::1` in production**, 10E R-I24), `tests/python/test_ftp_server.py`
  (the 18 tests; mapping in §Criteria), `tests/python/conftest.py` (`upload`, `fake_arw`), `camlab/experiments/camera_steps.py:20–38` (the camera's
  FTP settings: Standard hierarchy, Does Not Overwrite, passive only).

## Files expected to change
Infra (new): `crates/infra/src/ftp/{mod.rs, server.rs, session.rs, peers.rs}`. Infra (changed): `crates/infra/src/{lib.rs, settings_file.rs}`,
`crates/infra/Cargo.toml` (`ipnet`). Core (changed): `crates/core/src/ports/camera.rs` (`CaptureSink`, `ArrivalSource`, `DeliveryOutcome`),
`crates/core/src/domain/camera.rs` (`Delivery`, `CameraSettings { receiver_port, passive_port_start, passive_port_end }`),
`crates/core/src/services/settings.rs` (the `camera` key and its validation), `crates/core/src/error.rs` (none new; `InvalidValue("camera.…")`).
Integrations (changed): `crates/integrations/src/camera/runtime.rs` (the receiver slot, `bind` (c)/(d)), `crates/integrations/src/lib.rs`
(`CameraSettings` into the binding), `crates/integrations/src/camera/mod.rs` (`NoopSink` until P44: logs and answers `DeliveryOutcome::Photo`).
Desktop (changed): `apps/desktop/src-tauri/src/commands/settings.rs` (`camera: CameraSettingsDto` in `SettingsDto`/`SettingsPatchDto`),
`apps/desktop/src/services/bindings.ts` (regenerated).
Testkit (new): `crates/testkit/src/ftp_client.rs`, `crates/testkit/src/collecting_sink.rs`. Testkit (changed): `crates/testkit/src/{lib.rs,
test_app.rs}` (`with_camera` wires the receiver on `127.0.0.1:0` with loopback allowed; `receiver_port()`, `ftp()`, `deliveries()`),
`crates/testkit/Cargo.toml` (`libc` for `SO_LINGER`). Tests (new): `crates/testkit/tests/p43_ftp_receiver.rs`.
Estimated: 6 new source paths, 12 changed, 1 new test — 19 paths.

## Core mechanisms (unit tests allowed)
`crates/infra/src/ftp/peers.rs::is_allowed(ip, &[IpNet]) -> bool` and `ReceiverConfig::default_allowed_peers()` (rows C5(a)); `psm_infra::ftp::
names::{is_hidden_or_staging(name), base_name(path) -> Result<String>}` (553 rules). Everything else is a behaviour row at the socket boundary.

## The receiver contract (master plan §6.11 Receiver)
- One tokio runtime on its own thread; the control listener on `config.bind:config.port` (`0` → ephemeral; `ReceiverHandle::port()`); passive data
  listeners bound per transfer from `config.passive` (`0..=0` → ephemeral; otherwise the lowest free port of the range, `425` if none is free).
  Replies with the codes master plan §6.11 fixes. Commands implemented: `USER`, `PASS`, `QUIT`, `NOOP`, `SYST`, `FEAT`, `OPTS UTF8 ON`, `TYPE`, `PWD`,
  `CWD`, `CDUP`, `MKD`, `PASV`, `EPSV`, `REST`, `STOR`, `APPE`, `ABOR`, `LIST`, `NLST`, `SIZE`, `MDTM`, `DELE`, `RMD`, `RNFR`, `RNTO`; `PORT`/`EPRT` and
  anything else → `502`. Before `PASS` every other command but `USER`/`QUIT`/`FEAT`/`SYST`/`NOOP` → `530`. Username is `FTP_USERNAME`; the password is
  compared in constant time; a failed login is logged **without** the password (`add_secret` is registered for it before the receiver starts).
- `STOR <name>` / `APPE <name>`: `name` is reduced to its base name (the camera's *Standard* hierarchy does `CWD` first); a base name starting with `.`,
  ending with `.part`, or a path containing a `..` component → `553`; the capture root (the closure) `None` or not a directory → `451`; the key
  `name.to_uppercase()` already in flight → `450`; `STOR` without `REST` truncates `.incoming/.<name>.part` to 0; `REST n` + `STOR` or `APPE`:
  the `.part` must exist with length ≥ n → truncated to n and appended, else `554` (and the pending `REST` is cleared); `150` is sent, then bytes are
  read until EOF. **Completion** = the data socket read returned 0 bytes with no error **and** `SO_ERROR == 0` → `226`, then (on a blocking thread)
  `sha256_file(.part)`, `size`, `completed_at = clock.now_ms()` taken at the close, and `sink.delivery(Delivery { camera_filename: name (as sent),
  written_path: the .part, sha256, size, via: Wireless, completed_at })`. A reset, abort, timeout or pending `SO_ERROR` → `426`, the `.part` is kept,
  nothing is handed over; the control connection closing with an upload pending counts as failed too.
- `MKD`/`CWD`/`CDUP` always succeed (`257`/`250`); `PWD` echoes the virtual path. `LIST`/`NLST` (any directory) list `names.completed_names_since(now −
  COMPLETED_NAMES_WINDOW_MS)` — `LIST` lines `-rw-r--r-- 1 camera camera <size> <Mon dd HH:MM> <name>`; `SIZE <name>` → `213 <size>` / `550`; `MDTM
  <name>` → `213 <yyyyMMddHHmmss>` (UTC) / `550`. `DELE`, `RMD`, `RNFR` → `550`; `RNTO` → `553`.
- `allowed_peers` is checked before the `220` banner: a refused peer gets `421 Connections from this network are not allowed.` and the socket is
  closed; data connections are accepted only from the control connection's peer address.
- `ReceiverHandle::uploads_in_progress() -> usize` (the watchdog's "no upload in progress", P44/P46) and `stop()` (closes the listener, aborts open
  transfers as failures — their `.part`s stay).
- **Runtime:** `CameraRuntime::bind` starts the receiver on the first bind with the sidecar installed **and** a non-empty `ftp-password` secret
  (`FakeSecrets` in tests: `ftp-pw-unique-1`), with `ReceiverConfig { bind: 0.0.0.0 (tests: 127.0.0.1), port: settings.camera.receiver_port, passive: start..=end,
  allowed_peers: default (tests: loopback), username: FTP_USERNAME, password }` and `capture_root = canon(<effective document folder>/Capture)` read
  **from settings, never from Capture One** (R-I24); on a later bind whose `CameraSettings` differ in any of the three keys it calls `stop()` then
  `start()` again (one listener at any time); a bind with the same keys leaves the receiver untouched.
- **Settings:** `camera` joins the `SettingsService::set` allowlist; validation (master plan §6.8): ports 1024–65535, `passive_port_start ≤
  passive_port_end`, `passive_port_end − passive_port_start + 1 ≥ 2`, `receiver_port ∉ passive range`; anything else → `InvalidValue("camera")`.
  A `settings.json` without `camera` reads the defaults (`#[serde(default)]`).

## Ordered tasks
0. **Registry first** (§6.11): `Delivery`, `CameraSettings`, `CaptureSink`, `ArrivalSource`, `DeliveryOutcome`, `ReceiverConfig`, `FTP_USERNAME`, the
   constants; `Settings.camera: CameraSettings` with `Default`.
1. `ftp/peers.rs`, `ftp/names.rs` (+ their unit tests, C5(a), C3(d)).
2. `ftp/server.rs` + `ftp/session.rs`: the runtime thread, the listener, one task per control connection, the command table above, passive data
   sockets, the upload state machine, the completion/failure split (`SO_ERROR` through `socket2`-free `libc::getsockopt` on the raw fd or tokio's
   `TcpStream::take_error()`), the blocking hash + `sink.delivery`.
3. Settings key, DTOs, bindings; `SettingsService::set` validation.
4. Runtime slot + `bind` rules (start, in-place reopen); `NoopSink`.
5. Testkit: `ftp_client.rs` (`connect`, `login`, `cmd`, `pasv`, `stor`, `stor_rst(name, bytes)` — sends `bytes` then sets `SO_LINGER {1, 0}` via
   `libc::setsockopt` and closes so the server sees `RST`, `stor_resume(name, offset, bytes)`, `appe`, `list`, `nlst`, `size`, `mdtm`, `dele`,
   `rmd`, `rnfr_rnto`, `mkd`, `cwd`, `noop`), `collecting_sink.rs`, `TestApp` wiring and accessors.
6. Tests `p43_ftp_receiver.rs`; `cargo test --workspace`, clippy, fmt, `npm run typecheck`, `npm test`.

## Criteria (8 criteria, 27 rows; 0 `[rule]` rows)
Fixture: `a = TestApp::with_camera()` (receiver on `127.0.0.1:0`, loopback allowed, `CollectingSink` as the sink until P44); `c = a.ftp()` (connected
and logged in); `root` = the test's capture root; `part(name)` = `root/.incoming/.<name>.part`; `arw(n, seed)` = `n` pseudo-random bytes starting
`II*\0`; `D` = `a.deliveries()`. Standalone receivers (`FtpReceiver::start` with an explicit `ReceiverConfig`) are used where the row says so. Lab
test names in brackets are the ported source.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | completed upload [test_upload_completion_event_and_integrity] | `data = arw(3 MiB, 1)`; `c.stor("DSC00001.ARW", data)` | reply `226`; `fs::read(part("DSC00001.ARW")) == data`; `D == [Delivery { camera_filename: "DSC00001.ARW", written_path: part(..), sha256: sha256(data), size: 3 MiB, via: Wireless, completed_at: a.clock.now_ms() }]`; `root/DSC00001.ARW` does not exist (the rename is P44's) and `root` lists only `.incoming` | MC-32 §2, §4; R-I24 §Upload lifecycle; M37 |
| C1(b) | upload after `MKD`/`CWD` [test_upload_into_subdirectory] | `c.mkd("2026-11-05")` → `257`; `c.cwd("2026-11-05")` → `250`; `c.stor("DSC00003.ARW", arw(64 KiB, 3))` | `226`; the file is `part("DSC00003.ARW")` (the directory is virtual); `D.last().camera_filename == "DSC00003.ARW"` | R-I24 §Namespace ("`MKD` and `CWD` of any path succeed and are ignored; a file is identified by its base name") |
| C1(c) | two concurrent uploads [H-11] | `c1`, `c2` = two logged-in clients; `c1` opens `STOR A.ARW` and sends half of `arw(1 MiB, 4)`; `c2` opens `STOR B.ARW` and sends all of `arw(1 MiB, 5)` and closes; `c1` sends the rest and closes | both `226`; `D` holds `A.ARW` and `B.ARW` with the right hashes | MC-32 §5 ("at least 2 concurrent uploads") |
| C2(a) | reset mid-transfer is a failure [test_interrupted_upload_is_not_completed] | `data = arw(4 MiB, 6)`; `c.stor_rst("DSC00020.ARW", &data[..1 MiB])` | reply `426`; `D` is empty; `part("DSC00020.ARW")` exists with 0 < len ≤ 1 MiB; `root/DSC00020.ARW` does not exist | MC-32 §1 (F-1); M37 ("an aborted transfer yields none under a candidate name") |
| C2(b) | resume with `REST` [test_resume_with_rest_completes_original] | after C2(a)'s state: `off = len(part)`; `c.stor_resume("DSC00020.ARW", off, &data[off..])` | `350` to `REST`, `226` to `STOR`; `fs::read(part) == data`; `D == [Delivery { sha256: sha256(data), size: 4 MiB, .. }]` | R-I24 §Upload lifecycle ("`REST` + `STOR`/`APPE` resumes only onto an existing `.part` at least as long as the offset") |
| C2(c) | `REST` past the staged length | `c.stor_rst("X.ARW", &arw(1 MiB, 7)[..64 KiB])`; `c.stor_resume("X.ARW", 2 MiB, b"tail")` | `554` to the `STOR`; `part("X.ARW")` unchanged (len ≤ 64 KiB); `D` empty | R-I24 ("else 554") |
| C2(d) | new `STOR` truncates | after C2(c): `c.stor("X.ARW", arw(10 KiB, 8))` | `226`; `fs::read(part("X.ARW")) == arw(10 KiB, 8)`; one delivery of size 10 KiB | R-I24 ("a new `STOR` of the same name truncates its `.part`") |
| C2(e) | second `STOR` of a name in flight | `c1` opens `STOR Y.ARW` and sends 1 byte (no close); `c2.stor("Y.ARW", b"zz")` | `c2` gets `450`; after `c1` sends the rest and closes → `226` and one delivery for `Y.ARW` | R-I24 ("One upload per name at a time: … 450") |
| C2(f) | control connection closed with an upload pending | `c` opens `STOR Z.ARW`, sends 1 KiB on the data socket, then drops the **control** socket (no FIN on data first) | no delivery; `part("Z.ARW")` kept; `settle(a, || a.receiver().uploads_in_progress() == 0)` (P42 L7 fixture) | R-I24 ("a reset, an abort, a timeout … is a failure") |
| C2(g) | `APPE` resumes at the staged length | after a `stor_rst("W.ARW", first half)`: `c.appe("W.ARW", second half)` | `226`; `fs::read(part("W.ARW")) == data` | R-I24 (`APPE`) |
| C3(a) | `DELE`, `RMD`, `RNFR` refused [test_delete_and_overwriting_rename_refused] | after `c.stor("A.ARW", …)` and `c.stor("B.ARW", …)`: `c.dele("A.ARW")`, `c.rmd("x")`, `c.rnfr_rnto("A.ARW", "C.ARW")` | `550`, `550`, `550` on `RNFR` (the `RNTO` never runs); both `.part`s unchanged | MC-32 §3; R-I24 ("`DELE`, `RMD` and `RNFR`/`RNTO` are refused (550/553): every rename") |
| C3(b) | `RNTO` alone | `c.cmd("RNTO C.ARW")` | `553` | R-I24 |
| C3(c) | hidden and staging names [test_hidden_names_rejected] | `c.stor(".DSC00030.ARW.part", b"x")`; `c.stor(".hidden", b"x")`; `c.stor("Q.ARW.part", b"x")` | each `553`; no `.part` created, `D` empty | R-I24 ("a name that is hidden or looks like a staging file is refused (553)") |
| C3(d) | path traversal contained [test_path_traversal_is_contained] (unit + boundary) | `names::base_name("../escape.ARW")` → `Err`; `base_name("sub/DSC1.ARW")` → `Ok("DSC1.ARW")`; `c.stor("../escape.ARW", b"II*\0")` and `c.stor("/../../escape.ARW", b"II*\0")` | the two uploads get `553`; no file named `escape.ARW` exists anywhere under the test's temp dir | R-I24; H2 (nothing outside the capture root is written) |
| C4(a) | `LIST`/`NLST` from completed names | the `CollectingSink`'s `CompletedNames` returns `[("DSC00001.ARW", 3 MiB, t1), ("DSC00003.ARW", 64 KiB, t2)]`; `c.cwd("anything")`; `c.nlst()`; `c.list()` | `NLST` → exactly the two names; `LIST` → two lines each ending with the name and containing the size | R-I24 ("In every directory, `LIST`, `NLST`, `SIZE` and `MDTM` answer for the names completed in the last 24 h") |
| C4(b) | `SIZE`/`MDTM` | `c.size("DSC00001.ARW")`, `c.mdtm("DSC00001.ARW")`, `c.size("NOPE.ARW")` | `213 3145728`; `213 <t1 as yyyyMMddHHmmss UTC>`; `550` | R-I24 |
| C4(c) | names older than 24 h are not listed | names provider answers only for `since ≥ now − 86 400 000`; a name completed 25 h ago | absent from `NLST` and `SIZE` → `550` | R-I24 ("last 24 h"); §6.11 `COMPLETED_NAMES_WINDOW_MS` |
| C5(a) | default peer set (unit) | `ReceiverConfig::default_allowed_peers()` with `is_allowed` over `10.0.0.5`, `172.16.0.1`, `172.31.255.254`, `192.168.1.20`, `169.254.1.1`, `fd00::1`, `fe80::1` (allowed) and `8.8.8.8`, `127.0.0.1`, `172.32.0.1`, `11.0.0.1`, `::1`, `2001:db8::1` (refused) | exactly as listed | R-I24 §Network (the peer list); MC-32 §5 |
| C5(b) | refused peer at connect [test_peer_outside_allowed_networks_is_refused] | standalone receiver with `allowed_peers: [10.0.0.0/8]` on `127.0.0.1:0`; a raw TCP connect | first line `421 …`, then the server closes; nothing else accepted | R-I24 ("Any other peer gets 421 at connect") |
| C5(c) | authentication [test_startup_and_authentication_events] | `FtpClient::connect(port)`; `login("sessionmanager", "wrong")`; then `login("sessionmanager", "ftp-pw-unique-1")`; `login("other", "ftp-pw-unique-1")` on a fresh connection | `530`; `230`; `530`; `a.camera_runtime().helper_log()` and the tracing output contain neither `wrong` nor `ftp-pw-unique-1` (the receiver logs `login failed user=sessionmanager` only) | MC-32 §5; R-I21/M39 (secret never in a log) |
| C5(d) | passive only | `c.cmd("PORT 127,0,0,1,4,1")`, `c.cmd("EPRT |1|127.0.0.1|1025|")` | `502`, `502`; `c.pasv()` answers `227` with a port inside the configured range when the standalone receiver is started with `passive: 50000..=50009` | MC-32 §5 ("passive mode only on a fixed port range") |
| C6(a) | capture root missing [R-I24 §Capture root] | standalone receiver whose `capture_root` closure returns a non-existent folder; `c.stor("DSC1.ARW", b"II*\0")` | `451`; no `.part` anywhere; `D` empty | R-I24 ("If that folder does not exist, `STOR` gets 451, and the camera keeps the file queued and retries") |
| C6(b) | capture root from settings, not Capture One | `a = TestApp::with_camera()` whose fake Capture One is **not running** (`fake.state.running = false`) and `document_info` fails | `c.stor("DSC2.ARW", …)` → `226` and `part("DSC2.ARW")` is under `canon(<settings document folder>/Capture)/.incoming/`; `fake.state.ops` contains no `document_info` call caused by the upload | R-I24 ("never ask Capture One, which is quit during Take the camera back") |
| C7(a) | settings validation | `a.settings_set({"camera": {"receiver_port": 2121, "passive_port_start": 50000, "passive_port_end": 50000}})`; `…end: 49999`; `…receiver_port: 50005, start 50000, end 50009`; `…receiver_port: 80` | each `Err(InvalidValue(_))` with code `invalid_value`, settings unchanged; `{"receiver_port": 2222, "passive_port_start": 51000, "passive_port_end": 51009}` → `Ok` and `settings_get()["camera"]` echoes it | master plan §6.8 (`camera` validation); MC-32 §5 |
| C7(b) | port change reopens the listener in place | `p0 = a.receiver_port()`; `a.settings_set({"camera": {"receiver_port": <p1 = a free port>, …}})` | a TCP connect to `127.0.0.1:p0` is refused; `FtpClient::connect(p1)` gets `220`; `a.camera_runtime().receiver_generation()` increased by exactly 1; `rec().pids` unchanged (the sidecar was not restarted) | R-I18 ("receiver port, passive range or password → the listener is closed, then reopened"; "Never two sidecar processes, never two listeners"); M39 |
| C7(c) | an unrelated save keeps the listener and its connections | `c` logged in; `a.settings_set({"sku_prefix": "PX"})` (rebuild) | `c.noop()` → `200` on the same connection; `a.receiver_port()` unchanged; `receiver_generation()` unchanged | R-I18 ("A settings save hands the camera runtime to the new runtime unchanged") |
| C8(a) | stop aborts and keeps staging | `c` opens `STOR V.ARW` and sends 1 KiB; `a.camera_runtime().shutdown()` | no delivery; `part("V.ARW")` exists; the control socket is closed | R-I24 (failure keeps the `.part`); MC-31 §Process |

## Delegated choices
Whether the hash runs on `spawn_blocking` or a std thread; the exact `LIST` date format beyond "month day HH:MM"; `FEAT` contents; idle timeout
(recommended 600 s, as the lab); `max` control connections (recommended 16); the `receiver_generation` counter's type; whether `NoopSink` logs at
`info` or `debug`; how `TestApp` finds a free port for C7(b) (bind `127.0.0.1:0`, read, drop).

## Manual try-out
(Hardware-free; the camera is not needed.)
1. Start the app (Capture One can stay closed). On this Mac, open Terminal and run `ftp 127.0.0.1 2121`: no answer — the receiver refuses loopback
   on purpose, and starts only when camera support is installed (nothing is installed yet), so the port is closed. This is expected.
2. Settings → nothing new to see yet (the Camera section is P51). Saving another setting still works and the app does not hang.
3. Health shows no new rows.

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, the lab's `sdk_shim` headers, a `.env`, a credential or an SSH fingerprint; no Python (the
  receiver is Rust; `camlab/ftp` is read, not run); never change the camera's network/FTP settings, delete a camera file or kill any process; live view
  out of scope; the suite never needs the camera. No `implementer: fable`.
- **Why hand-written and not `libunftp` (planner reading):** R-I24 fixes exact reply codes (450/451/553/554/421), the "reset = failure" rule and
  `REST`-onto-`.part` semantics; a library's error-to-code mapping and resume behaviour would each need source verification (charter rule 17) and
  still not match. The camera uses a small command subset (passive, `CWD`, `MKD`, `STOR`, listings); unknown commands get `502` and the real command
  sequence is recorded at P47 (spike S7) before P52.
- **Planner readings (reversible):** (1) the FTP username is the constant `sessionmanager` (the lab used `sony`; the owner re-enters user and
  password on the camera once, P50/P51). (2) The receiver starts only when camera support is installed **and** an FTP password exists (until P50
  generates one, nothing listens). (3) Loopback is not an allowed peer in production (10E's list); tests allow it explicitly. (4) `APPE` resumes at
  the `.part` length. (5) A name with a path separator is reduced to its base name; `..` is refused.
- **Hazards:** binding `0.0.0.0` in a test binary triggers the macOS firewall prompt — tests always bind `127.0.0.1`. The `226` must be sent **after**
  `SO_ERROR` is checked, never on EOF alone (the lab's F-1). Hashing a 51 MB file takes ~0.1 s — off the transfer path. `uploads_in_progress` must
  count a data connection from `STOR` accepted until `226`/`426`, including C2(f)'s orphaned data socket (a control-close must abort it). The runtime
  must read the capture root from **settings** (`Settings::effective_document_id(paths)/Capture`), not `ArrivalService::capture_root` (which asks
  Capture One, `arrival.rs:57–68`). `Settings` gains a field: every struct literal of `Settings` in tests still compiles through `..Default::default()`
  or the serde default.
- Registry additions made by the planner in master plan §6.8, §6.11. No intention-book change.
- Estimated size: ~19 paths, ~1 900 lines (≈ 900 receiver + settings, ≈ 1 000 client + tests).

## Review log

- **2026-11-06, carried from P42 projection r0** (planner): the FTP password fixture is `ftp-pw-unique-1` (L10; the receiver registers it with the redactor
  when it starts — P42 registers only the two credentials it sends); every timed read of the fake or the receiver uses P42's `settle(a, pred)` fixture
  (L7; C2(f) reworded); the runtime's `bind` is the last statement of `App::build` and `tick` is serialised under the supervisor `Mutex` (L3, L18) — the
  receiver's in-place reopen (C7(b)) runs inside that serialised `bind`, never from two Apps at once.
- **2026-11-06, implement r1 (codex):** Built the passive-only receiver, virtual namespace, `.incoming` staging, reset/failure split, REST/APPE resume, private-peer and password gates, and settings-driven in-place reopen. The sink is a routing stand-in until P44; a separate `ftp::CompletedNames` source keeps FTP listing state out of the core `CaptureSink`. Assumptions: hash on `spawn_blocking`; LIST dates in UTC; FEAT advertises EPSV/REST/UTF8; 600 s data idle timeout; no explicit control-connection cap; NoopSink logs at debug; receiver generation is `u64`; the test client finds a free port by temporary loopback bind. The test seam binds loopback with ephemeral ports; production binds `0.0.0.0` with the contracted private-peer filter. Deviations: `Cargo.lock` for `ipnet`, and eight existing frontend test fixtures gained the required `camera` settings block so typecheck stays green. Focused tests: 26 socket rows and 2 mechanism units pass; final `cargo test --workspace`, Clippy, fmt, TypeScript and 369 UI tests pass. One unchanged P33 UI test failed in an intermediate parallel run, passed alone and in the final full UI run. No live camera or Capture One was used; socket tests used only loopback.
- **2026-11-06, review r1 (opus): CHANGES_REQUESTED.** Gate: tracker `IMPLEMENTED`, checkpoint `e3569be` present; `git show --stat` = the handoff's 33 paths (declared deviations: `Cargo.lock`, 8 frontend fixtures for the additive `camera` DTO field, `ftp/names.rs`). Reviewer runs on clean `c0147c9` (code tree = `e3569be`): clippy, fmt, typecheck, UI 369/369 green; `cargo test --workspace` **red** (`c2a_reset_is_failure`); the `p43` binary re-run 6× → 4 red (C2(a)/(b)/(c)), `c2a` alone 8× → 4 red. **F1 (B4, route: production):** testkit `FtpClient::stor_rst` calls `shutdown(SHUT_RDWR)` (sends a FIN) before the `SO_LINGER {1,0}` close, and has no pause between the data and the reset (lab `_interrupted_upload`: `sendall`, `sleep(0.3)`, linger-close). FIN-first → the server correctly sees a clean close → `226` (observed: C2(a) and C2(c) got `226`); RST while the server still has unread data → the reset is lost at TCP level and the server waits its 600 s idle timeout → the client's 15 s read times out (observed `WouldBlock`). Reviewer probe (scratch test file, deleted): the lab's shape (write, 300 ms pause, linger-close, no shutdown) → `426` 25/25 on the unchanged receiver. Fix: make `stor_rst` the lab's shape (files: `crates/testkit/src/ftp_client.rs`; `crates/testkit/tests/p43_ftp_receiver.rs` only if the pause needs a predicate). **F2 (B1, route: production):** `session.rs` PASV reply uses `state.config.bind`'s octets; production binds `0.0.0.0` → `227 Entering Passive Mode (0,0,0,0,…)`, an unconnectable address for a camera that honours it (MC-32 §5 passive mode; the lab's pyftpdlib with `masquerade_address=None` advertises the control connection's local address). Fix: advertise the control socket's local IPv4 address (`control.get_ref().local_addr()`); file `crates/infra/src/ftp/session.rs`; no new test required (loopback cannot discriminate; a `0.0.0.0` bind in tests triggers the firewall prompt, plan Hazards). Verified correct: 24 other rows (C1(a)–(c), C2(d)–(g), C3(a)–(d), C4(a)–(c), C5(a)–(d), C6(a)–(b), C7(a)–(c), C8(a)) observe their outcome at the named boundary and fail on a wrong one; completion = EOF + `take_error()` none, `.part` kept on failure, 553/451/450/554 order, constant-time password compare, failed login logs no secret, `add_secret` before start, capture root from settings (`canon(<document id>/Capture)`, same as `adapter.rs:101`), in-place reopen only on a changed `CameraSettings`, stop aborts uploads. Assumptions within Delegated choices; none contradicts the book. H1–H8 hold (H7 placement of FTP ports in core is plan-mandated → lesson). 11 notes in `backlog.md`.

- **2026-11-06, fix 1 (codex):** Resolved F1 by making `stor_rst` wait 300 ms after sending data and then close with `SO_LINGER {1,0}` only; removed the `shutdown(SHUT_RDWR)` FIN. Resolved F2 by deriving the PASV reply's IPv4 octets from the control socket's local address. No test file edit was needed. Validation: `cargo test --workspace` passed (P43 socket suite 26/26), Clippy and fmt passed, desktop typecheck passed, and UI suite passed (50 files / 369 tests). No divergence or additional judgment call.
- **2026-11-06, re-review r2 (opus): CHANGES_REQUESTED (1 MUST).** Gate: tracker `IMPLEMENTED`, fix checkpoint `5677669` present. Perimeter: `git show --stat 5677669` = `crates/testkit/src/ftp_client.rs`, `crates/infra/src/ftp/session.rs`, tracker row, this log, which are exactly the files F1/F2 allowed. `5677669..HEAD` changes only runner docs. Runs on clean `6956b7c` (code tree = `5677669`): clippy, fmt, typecheck exit 0; UI 50 files / 369 tests; `cargo test --workspace` run 1 red (`c7b_port_reopens_in_place` panicked at the `settle` helper, line 85; output lost to my filter, so re-run), run 2 exit 0. Variation: `p43` binary ×10 → 2 red, both `c7b` at line 85; `c7b` alone ×15 → 0 red. **F1 resolved:** `stor_rst` = write, `sleep(300 ms)`, `SO_LINGER {1,0}`, drop; no `shutdown`. This is the lab's `_interrupted_upload` shape. C2(a)–(g) green in all 12 runs (r1: ~50 % red). **F2 resolved:** the `227` octets now come from `control.get_ref().local_addr()`, the interface the camera reached. The passive listener still binds `config.bind`, so it is reachable on that interface. EPSV is unchanged. A non-IPv4 control connection now ends the session (unreachable under the IPv4 bind; backlog). **F3 (B4, route: production (test fixture; no tester role), MUST):** the suite is red in 3 of 12 runs. `c7b` waits 3 s (`settle`, 300×10 ms) for the fake sidecar pid. Reviewer probe (scratch test file, deleted): 24 parallel `TestApp::with_camera()` → pid latencies grow linearly by ~125 ms per app (first exec of each fresh per-app sidecar copy is serialised), the last at 3.0–3.1 s; every sidecar started. So `c7b` fails whenever it is created behind ~24 queued first execs (14 test threads + C2/C1 load). r1 backlog row predicted "lengthen the bound if it recurs after the C2 fix"; it recurs. Fix: in `c7b`, wait for `pids` non-empty with a bound that covers the queue (e.g. 15 s), leaving the other settles at 3 s; file `crates/testkit/tests/p43_ftp_receiver.rs` only. Triage: F3 MUST (red suite, flow.md §5). No DROP items. Backlog +2 rows (testkit per-app sidecar copy queue, P42 exposure; PASV non-IPv4 behaviour).
- **2026-11-06, orchestrator ruling on re-review r2.** FIX2_AUTHORISED: F3 is a red suite (flow.md §5 MUST). It is not a product defect, but at ~25 % red it would hand every later phase false B4s. The fix is limited to `crates/testkit/tests/p43_ftp_receiver.rs`: in `c7b_port_reopens_in_place`, wait up to 15 s for the fake sidecar pid; the other 3 s settles and all assertions stay unchanged. The re-reviewer should repeat the `p43` binary at least 10× (a single run cannot detect a flake).
