---
plan: P68_setup_polish_address_password_shop
role: implement
round: 1
state: DONE
date: 2026-11-11
actor: codex
---
# Handoff — P68 implement r1 (Claude Opus in the `codex` actor slot)

Gate: intention header RATIFIED; tracker P67 APPROVED, P68 NOT_STARTED → passed.

## What was built
- **This Mac's address in the FTP steps.** Settings → Camera now always lists the camera's FTP steps (not only after Generate). Step 3
  reads `[Destination Settings] › [Host Name] = <this Mac's address>; [Port] = 2121; …` with a **Copy address** button. The address is
  re-read every 5 s, so a network change while Settings is open shows up. Without a network the step keeps the old placeholder text.
- **"Address changed" warning.** The FTP receiver remembers the address of this Mac that the camera last connected to (the local address
  of its last accepted connection, kept in the app's data folder so a restart on another network still notices it). When this Mac no
  longer has that address, Health (item "Camera") and the header camera line say
  `This Mac's address changed to <new> — set Host Name on the camera`. The header shows the first 48 characters with the full sentence in
  its tooltip and in its card. The warning goes away by itself once the Mac is back on the old address. The address is never written to a
  log.
- **Easy FTP password.** Generate password now makes 16 characters from lowercase letters and digits without `0 o 1 l i`
  (`abcdefghjkmnpqrstuvwxyz23456789`, about 79 bits). It is shown in groups of 4 (`k7mx 4rtp 9qad 2wen`) with "Type it without the
  spaces". **Show password** reads the stored password back from the Keychain (macOS may ask to allow that). Copy copies it without
  spaces. Existing passwords keep working until regenerated: the receiver still uses whatever the Keychain holds.
- **Shop settings cleaned on save.** The shop domain loses spaces, its scheme and anything after the host (`/admin…`, a trailing `/`),
  and is lowercased. `admin.shopify.com/store/<x>` becomes `<x>.myshopify.com`. The client ID and secret are trimmed. A stored domain that
  cannot be a host name now fails with `The shop domain looks wrong: <value>` before any request is built, so `http: builder error`
  no longer appears. A refused client ID or secret reads `The Shopify client ID or secret is wrong`.
- **Camera login saved with Save settings.** The Access Authentication user and password fields stay in Settings → Camera, but the
  separate "Save camera login" button is gone. **Save settings** stores them in the Keychain (never settings.json) and then clears the
  fields. A typed login counts as an unsaved change, so leaving Settings asks "Save your changes?".
- **Fingerprint reachable from the header.** When the camera waits for its fingerprint to be confirmed (unconfirmed or changed), the
  header camera card has **Confirm the camera's fingerprint**. It opens the same box as Settings → Camera (fingerprint, Confirm, Not mine).
  The box now also asks the backend for a fingerprint reported before it was opened (new command `camera_fingerprint_pending`), so it no
  longer waits for the next event.

## Tests → criterion rows
| Row | Test | Boundary |
|---|---|---|
| C1(a) | `crates/testkit/tests/p68_setup_polish_address_password_shop.rs::c1a_address_shown_and_change_named` | real FTP receiver connection on A (loopback), fake network A → B: `CameraRuntime::ftp_address()` = B with the exact sentence; Health item `camera_address_changed` = the sentence; back on A it clears |
| C1(a) | `apps/desktop/src/features/settings/__tests__/p68_setup_polish.test.tsx` "C1(a) …" | step 3 list item shows B, Copy address copies B; header dot label/tooltip/card carry the sentence |
| C2(a) | `p68_…shop.rs::c2a_easy_password_generated_and_shown_again` | the old password logs in; Generate → 16 chars from the allowed set, none of `0 o 1 l i`; `CameraService::ftp_password` (Show) returns the same; the receiver accepts it |
| C2(a) | `p68_setup_polish.test.tsx` "C2(a) …" | Generate shows `k7mx 4rtp 9qad 2wen`; after remount, hidden until Show password, which shows the same |
| C3(a) | `p68_…shop.rs::c3a_shop_domain_normalised_and_bad_domain_named` | `settings_set` → settings.json holds `x.myshopify.com` (from `https://X.myshopify.com/admin/ ` and from `admin.shopify.com/store/x`), keys trimmed; adapter with `my shop^.myshopify.com` → exactly `The shop domain looks wrong: my shop^.myshopify.com`, no request sent |
| C4(a) | `p68_setup_polish.test.tsx` "C4(a) …" | typed login → leaving asks "Save your changes?"; Save settings → `cameraSecretsSet` gets user/password, the `saveSettings` payload holds neither; fields cleared |
| C4(b) | `p68_setup_polish.test.tsx` "C4(b) …" | status `fingerprint_unconfirmed` + pending `ab:cd:ef` → header card's Confirm the camera's fingerprint opens the box; Confirm sends `ab:cd:ef`, box closes |
| C5(a) | closing gate (Stamp) | whole workspace |

Core mechanism unit test: `psm_core::mechanisms::secrets::tests::c6a_password_shape_and_rejection` (the existing P50 mechanism test,
updated to the new alphabet and length).

## Assumptions
- A1. "The camera last uploaded to" = the local address of the receiver's **last accepted connection** (after the private-network peer
  filter, before login), as the plan words it. It is kept in `<data>/camera/last_upload_address` (data folder, never committed, never
  logged) so the warning also appears after a restart on a new network.
- A2. "This Mac's current address" = the address the camera last used while this Mac still has it. Otherwise it is the first private IPv4
  address of an interface that is up (not loopback, not IPv6 link-local), then any IPv4, then IPv6. "Changed" = the camera uploaded before,
  this Mac has an address, and the last upload address is not among this Mac's addresses. With no network there is no warning.
- A3. "Refreshed on network change" = the UI re-reads the address every 5 s (step 3 and the header). The backend compares at most once a
  second on the camera tick and updates Health then. There is no OS network-change notification.
- A4. The header uses the existing camera-line rule: the first 48 characters plus "…", the full sentence in the tooltip and the card. The
  sentence replaces the status label unless the camera is busy (taking or receiving a photo).
- A5. The password is stored and typed without spaces; the groups are display only.
- A6. The domain is checked before any request: dot-separated labels of letters, digits and `-`. The check runs at request time rather
  than as a save refusal, so a domain saved before P68 is named in the sentence.
- A7. `The Shopify client ID or secret is wrong` is the new text of the existing `credentials_rejected` error (grant refused, or 401
  twice). The error code is unchanged.
- A8. If both login fields are empty, Save settings leaves the Keychain untouched. If either one is typed, both are written, as the old
  button did.

## Deviations
- **Book wording now differs from the owner's 2026-11-11 rule (the orchestrator should fold it in; I did not edit the book):** 10E R-I21
  says the FTP password is "32 random characters from `[A-Za-z0-9]`" and "shows it once". 09A says a refused grant reads "Shopify
  credentials rejected". The plan (owner backlog 2026-11-11) changes all three, and I followed the plan. The P66 precedent was a
  reviewer note, not BLOCKED.
- Files outside the plan's named list, each needed by the rows: `crates/core/src/error.rs` (new `ShopDomainInvalid`, new
  `CredentialsRejected` text), `crates/core/src/ports/camera.rs` (`CaptureSink::connection_accepted`, default no-op), `crates/infra/src/net.rs`
  (new; `getifaddrs`), `crates/infra/src/lib.rs`, `crates/integrations/src/camera/{runtime.rs, direct/adapter.rs}`, `crates/integrations/src/lib.rs`
  (normalising in `SettingsSetContext::prepare`), `apps/desktop/src-tauri/src/{lib.rs, commands/items.rs}`, `apps/desktop/src/services/{camera.ts,
  bindings.ts}`, `apps/desktop/src/features/shell/{CameraDot.tsx, CameraCard.tsx}`, `apps/desktop/src/features/health/HealthPage.tsx` (label),
  `apps/desktop/src/features/settings/FingerprintBox.tsx` (new, shared box).
- Existing tests changed because their behaviour changed: `p12_shopify_adapter.rs` c9f (refusal text), `p50_camera_settings_and_support.rs`
  c6b (length 16), `apps/desktop/src-tauri/src/lib.rs` command test (length 16), `p51_camera_section.test.tsx` C5(a) (groups, new sentence)
  and C6(a) (no separate button).
- New commands: `camera_ftp_password_get`, `camera_ftp_address`, `camera_fingerprint_pending` (registered and in the bindings).
- The full `cargo test --workspace` ran with Bash's background mode: a foreground call stops at 10 minutes and the suite takes about an
  hour. Nothing else touched cargo while it ran (typecheck and `npm test` ran then; they do not use cargo).

## Files changed
apps/desktop/src-tauri/src/commands/camera.rs · apps/desktop/src-tauri/src/commands/items.rs · apps/desktop/src-tauri/src/lib.rs ·
apps/desktop/src/features/health/HealthPage.tsx · apps/desktop/src/features/settings/CameraSection.tsx · apps/desktop/src/features/settings/FingerprintBox.tsx (new) ·
apps/desktop/src/features/settings/SettingsPage.tsx · apps/desktop/src/features/settings/cameraSteps.ts ·
apps/desktop/src/features/settings/__tests__/p51_camera_section.test.tsx · apps/desktop/src/features/settings/__tests__/p68_setup_polish.test.tsx (new) ·
apps/desktop/src/features/shell/CameraCard.tsx · apps/desktop/src/features/shell/CameraDot.tsx · apps/desktop/src/services/bindings.ts ·
apps/desktop/src/services/camera.ts · crates/core/src/error.rs · crates/core/src/mechanisms/secrets.rs · crates/core/src/ports/camera.rs ·
crates/core/src/services/camera.rs · crates/infra/src/ftp/server.rs · crates/infra/src/lib.rs · crates/infra/src/net.rs (new) ·
crates/integrations/src/camera/direct/adapter.rs · crates/integrations/src/camera/runtime.rs · crates/integrations/src/commerce/shopify/adapter.rs ·
crates/integrations/src/lib.rs · crates/testkit/tests/p12_shopify_adapter.rs · crates/testkit/tests/p50_camera_settings_and_support.rs ·
crates/testkit/tests/p68_setup_polish_address_password_shop.rs (new) · docs/implementation/master_plan.md (P68 row) ·
docs/implementation/plans/P68_setup_polish_address_password_shop.md (Review log)

## Checkpoint SHA
`e39c105` (perimeter only; this handoff is left for the runner to commit)

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace` | exit 0: 67 test binaries, 938 passed, 0 failed (no flakes this run) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npm run typecheck` | exit 0 |
| `npm test` | exit 0: 68 files, 463 tests passed |
| `cargo test -p psm-testkit --test p68_setup_polish_address_password_shop` | 3 passed |

## Manual try-out
1. Start the app (`cd apps/desktop && npm run tauri -- dev`), open Settings → Camera (camera support installed).
2. Step 3 of the FTP steps shows this Mac's address. Click **Copy address** and paste it anywhere: it is the same address.
3. Click **Generate password**: 16 lowercase letters and digits in groups of 4. Leave Settings, come back, click **Show password**: the same
   password appears (macOS may ask to allow Keychain access).
4. Type the camera's Access Authentication user and password, then click **Health**: "Save your changes?" appears. Choose Save. The
   fields empty and the login is in the Keychain (Keychain Access → `com.loorenz.sessionmanager.camera`).
5. Shopify: type ` https://YourShop.myshopify.com/admin/ ` as the domain and Save settings. After reload the field reads
   `yourshop.myshopify.com`.
6. After one photo has arrived over FTP, move the Mac to another network (or change its IP). Within a few seconds the header camera line
   and Health say `This Mac's address changed to <new> — set Host Name on the camera`.
7. When the camera asks for its fingerprint, the header camera card shows **Confirm the camera's fingerprint**, which opens the box.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
