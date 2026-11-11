---
plan: P68_setup_polish_address_password_shop
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-11
actor: opus
---
# Review — P68 setup polish (round 1)

Gate: tracker row `IMPLEMENTED`; checkpoint `e39c105` in `git log` → passed. Code tree reviewed = `e39c105` (HEAD `1211576` adds only the runner's handoff/prompt commits).

## Verified correct
**Perimeter.** `git show --stat e39c105` = exactly the 30 paths in the handoff's "Files changed" (no extra, none missing). The extra files outside the plan's named list are disclosed, and each one is needed by a row.

**Checks (reviewer run, one cargo command at a time, foreground).**
| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test p68_… --test p12_shopify_adapter --test p50_camera_settings_and_support` | 3/3, 69/69, 22/22 |
| `cargo test -p psm-core --lib secrets` | 1/1 (`c6a_password_shape_and_rejection`) |
| `cargo test -p psm-desktop --lib p50_tests` | 2/2 |
| `cargo fmt --all -- --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `npm run typecheck` | 0 |
| `npx vitest run src/features/{settings,shell,health}` + p07 + p18 | 102/102 |
I consumed the implementer's full-suite stamp (938/0 Rust, 463 UI) without re-running it, as the Review mode directs. Nothing I saw contradicts it.

**Rows.**
- **C1(a).** The receiver binds IPv4 `UNSPECIFIED`, so `stream.local_addr()` is the plain interface address that `getifaddrs` also reports (no IPv4-mapped mismatch). `FtpAddress::of`: the last upload address while the Mac still has it, else private IPv4 → IPv4 → IPv6. `changed` only after an upload, and only when the Mac has an address. `check_address` runs on the tick at most once a second and sets or clears `camera_address_changed`. `HealthService::set` does not log, and no `info!/warn!` carries the detail. The address file is `<app support>/camera/last_upload_address` (same root as fingerprints), and its write-failure `warn!` carries no address. The Rust test fails if the accepted connection is not recorded: on B there would be no sentence. The UI test asserts step 3 (the third list item) shows B, Copy address copies B, and the header label, tooltip and card carry the backend's sentence.
- **C2(a).** Alphabet: 31 symbols with no `0 o 1 l i`. The rejection threshold 248 = 8×31, so there is no modulo bias. Length is 16. `camera_ftp_password_get` → `CameraService::ftp_password` → Keychain, and the value is registered with the redactor. The receiver still reads the Keychain, so an old password logs in (the test logs in before regenerating). The Rust test checks the shape, Show = generated, and that the receiver accepts the new password. The UI test checks `^k7mx 4rtp 9qad 2wen$`, hidden after remount, and Show shows the same value. Copy copies without spaces.
- **C3(a).** Normalisation sits in `SettingsSetContext::prepare` (integrations, not Tauri) and applies only when the patch carries `integration`. Both inputs give `x.myshopify.com`, and the keys are trimmed. Every Shopify request, the token grant included (`auth.with_token` inside `call_inner`), passes the domain check first, so a bad domain sends no request and cannot produce a builder error. The test asserts `fake.requests()` is empty and the exact sentence. `ShopDomainInvalid` maps to `shop_domain_invalid` with `message = to_string()`. The saved (normalised) settings come back into the form.
- **C4(a).** The login state is lifted to `SettingsPage`, and `dirty` includes a typed login. The leave dialog's Save and the Save settings button share `save`, and Discard clears the login. The test checks that leaving asks first, `cameraSecretsSet` receives the typed values, the `saveSettings` payload contains neither value, and the fields clear. The P51 C6(a) test was updated because the separate button is gone.
- **C4(b).** `CameraCard` shows the action for `fingerprint_unconfirmed`/`fingerprint_mismatch`. The Radix dialog is a React child of the popover content, so interacting with it does not dismiss the popover. `FingerprintBox` (one component, also used in Settings) reads `camera_fingerprint_pending` for a fingerprint reported before it opened. The test checks the box opens from the header with the pending value, Confirm sends `ab:cd:ef`, and the dialog closes.
- **C5(a).** The stamp plus the reviewer checks above.

No orphan tests: every new or changed test traces to a row or to a behaviour change the handoff discloses. No test is implementation-coupled.

**Assumptions A1–A8.** All stay within the plan's words, and none contradicts the intention. The difference from book 10E R-I21 (32 characters from `[A-Za-z0-9]`, shown once) and 09A ("Shopify credentials rejected") comes from the plan's owner rule of 2026-11-11, not from an implementer choice. It is a book fold for the orchestrator (P66 precedent), not B1.

**Hard constraints.** H1: nothing is written to Shopify, and a bad domain is refused before any request. H2: no capture file is touched. H3: n/a. H4: the address file is rewritten idempotently. H5: `getifaddrs` is an OS API, not UI automation. H6: unaffected. H7: one more camera/commerce sentence in core (note, P53 family; not a platform concept in core tables). H8: the three new commands only read runtime or service state. Secrets: the camera login is Keychain-only; the FTP password leaves the Keychain only through the Show command and is redacted. No live services: the tests use loopback FTP and `FakeTransport`.

## Findings
None blocking (B1–B4 all clear). Verdict `APPROVED_WITH_NOTES`.

## Notes written to backlog
1. Book fold: 10E R-I21 (password length/alphabet, "shown once") and 09A (refusal text) still carry the pre-P68 wording. **Destination:** orchestrator book update.
2. Save settings stores the camera login before `saveSettings`. A refused settings save leaves the login stored and the fields cleared (partial save). **Destination:** next settings phase.
3. Typing only the camera user writes an empty Access Authentication password to the Keychain (A8). **Destination:** next settings phase.
4. In the header, the address-changed sentence hides needs-action labels such as `Confirm the camera's fingerprint` (A4). **Destination:** next header/camera-UI phase.
5. The upload address is taken from any accepted private-network connection before login (A1). A non-camera device on port 2121 can move it. **Destination:** next camera phase.
6. H7 family: `… set Host Name on the camera` and the Shopify error sentences live in core. **Destination:** same as the P53 note (adapter-owned strings).
7. Process: flow §4A "never in the background" collides with the agent tool's 10-minute foreground limit for the ~1 h workspace suite. **Destination:** orchestrator, flow.md.

## Lessons for the plans
- A plan that changes a ratified rule on the owner's word (here R-I21, 09A) should name the book sections it overrides. The implementer and reviewer can then treat the difference as settled rather than re-arguing it each phase.
- "The address the camera last uploaded to" was implementable two ways (connection vs. completed upload). A one-line boundary definition in the plan would have settled A1.

## Probe declaration
No production file was edited, even temporarily. Files written by this review: `docs/implementation/backlog.md` (7 notes appended), `docs/implementation/master_plan.md` (P68 tracker row only), `docs/implementation/plans/P68_setup_polish_address_password_shop.md` (Review log entry), `docs/implementation/narratives/P68_setup_polish_address_password_shop.md` (new), this handoff. Build artifacts only under `target/`. Nothing committed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
