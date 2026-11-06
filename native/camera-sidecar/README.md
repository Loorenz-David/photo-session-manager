# Camera sidecar

The C++ helper process Session Manager uses to control a Sony camera over USB-C and Wi-Fi through
Sony's Camera Remote SDK. It is the Sony camera lab's `crlab` (lab repo `Camera-Connection-Tests`,
`sdk/`, commit `221aa49`), imported without any Sony file and extended for the app. The app starts
it as `camera-sidecar auto --save-dir <staging> --usb-product-id 0x0E0C` and talks to it over
stdio: one command per line on stdin, one JSON event per line on stdout, diagnostics on stderr.
Protocol and lifecycle: `docs/intention/book/10E_direct_camera_adapter.md` (MC-31); the lab's
evidence: `docs/intention/evidence/sony_camera_lab.md`.

## Build

| What | Command (repo root) | Output |
|---|---|---|
| Stub (any Mac, CI): refuses every camera operation with `sdk.unavailable` | `tools/sidecar/build.sh --stub` | `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin` |
| Stub + the mock-camera tests (`ctest`) | `tools/sidecar/build.sh --stub --test` | the same, plus `20 tests, 0 failed` |
| With the Sony SDK (build machine only) | `tools/sidecar/build.sh --sdk <extracted CrSDK_vX_Mac dir>` | the same file, built against the SDK |
| Support-package manifest (hashes only) | `tools/sidecar/manifest.sh <dir holding libCr_Core.dylib and CrAdapter/> <version>` | `native/camera-sidecar/manifest/sony-<version>.json` |

- Builds go to `target/camera-sidecar/{stub,sdk}`, never into this directory.
- `apps/desktop/src-tauri/build.rs` builds the stub when the `binaries/` file is missing, so
  `cargo test` and `tauri build` work on a clean machine (CMake and Apple clang are required).
  It never rebuilds an existing file: **after any C++ edit run `tools/sidecar/build.sh`** (or delete
  the binary).
- The SDK build runs CMake symbol probes (`src/sdk_symbols.def`) against the installed headers and
  copies Sony's runtime libraries next to the binary **in the build directory only**, so it can be
  run there on the build machine. The app never bundles them: the user installs Sony's runtime as a
  camera support package from Settings → Camera.

## Never commit

The repository is public. Never commit a Sony SDK file (headers, `*.dylib`, `CrAdapter/`, the
`CrSDK_*` archive or folder), the lab's hand-written SDK stand-in headers (`sdk_shim`), a `.env`
file, a credential, or an SSH fingerprint (`*.fingerprint`). `.gitignore` covers `vendor/`, `CrSDK*`,
`*.fingerprint`, the build directories and `binaries/`. No Python is used anywhere.

## What changed from the lab

- Credentials arrive only as `auto`'s first stdin line, `credentials user=<pct> password=<pct>`
  (percent-encoded; its `cmd.received` carries no other field). No credential is read from the
  environment; `CRLAB_FINGERPRINT_DIR` (not secret) still is.
- Every new session (USB and Wi-Fi): `SetSaveInfo(--save-dir)`, then the store destination
  (USB *Host PC + Memory Card*, Wi-Fi *Memory Card*) written and read back within 3 s;
  `connection.ready` carries `model`, `store_destination` (`verified`/`failed`) and `store_value`.
  The write is persistent (nothing is restored on disconnect) and *Host PC* alone is never written.
  A quit during a USB session writes *Memory Card* once (≤ 1 s) before closing.
- A USB exposure holds busy `file_transfer` until its download completes (≤ 10 s); a quit while
  busy waits for the safe point, at most 10 s.
- `usb.initial {present}` once after the USB monitor starts; the IOKit match is limited to the
  camera's product ids (`--usb-product-id`); the `usb-retry` command clears the three-failure USB
  fallback (`usb.retry_requested`).
- The executable and CMake project are renamed (`camera-sidecar`, `camera_sidecar`); the internal
  `crlab` namespace and the event names are the lab's.
