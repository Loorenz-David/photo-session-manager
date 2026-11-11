---
phase: P68_setup_polish_address_password_shop
title: Setup polish from the work-Mac install — the FTP steps show this Mac's address and warn when it changed, the FTP password is easy to type and can be shown again, the shop domain and keys are cleaned up with plain errors, and the camera login is saved with Save settings
difficulty: normal
projection: no
depends_on: [P67_form_values_defaults_and_batch_step]
---
# P68 — Setup polish (owner 2026-11-11, backlog entries of that day)

Written by the orchestrator (Opus) on 2026-11-11 from the work-Mac install. Owner: "how to know the host name? also the password is a pain to write, this
should change on the fixes".

## Goal
1. **Address shown.** Settings → Camera FTP step 3 shows this Mac's current address on the active network (live, refreshed on network change) with a Copy
   button. When the address differs from the one the camera last uploaded to (from the receiver's last accepted connection), the header camera line and
   Health say `This Mac's address changed to <new> — set Host Name on the camera` (address in the UI only, never in committed docs or logs at INFO).
2. **Easy FTP password.** Generate makes a 16-character password from lowercase letters and digits without look-alikes (`0 o 1 l i`), shown in groups of 4;
   **Show password** in Settings reveals it again (from the Keychain; a Keychain prompt is fine). Existing passwords keep working until regenerated.
3. **Shop settings cleaned.** The shop domain is normalised on save (strip scheme, `/admin…`, trailing `/`, spaces; lowercase); `admin.shopify.com/store/<x>`
   becomes `<x>.myshopify.com`; client ID/secret trimmed. A transport/builder error names the field: `The shop domain looks wrong: <value>` /
   `The Shopify client ID or secret is wrong` (never `http: builder error`).
4. **Camera login with Save settings.** The camera user/password are saved by **Save settings** (and covered by the P63 unsaved-changes prompt); the
   separate button goes. The fingerprint box appears in Settings → Camera and from the header when confirmation is needed (`Confirm the camera's fingerprint`
   opens it).

## Read first
Backlog 2026-11-11 entries; P50, P55, P63 narratives; book 10E MC-31/MC-32 (FTP, secrets), 09A. Code: `apps/desktop/src/features/settings/{CameraSection.tsx,
cameraSteps.ts, SettingsPage.tsx}`, `crates/core/src/services/{camera.rs, settings.rs}`, `crates/infra/src/ftp/*`, `crates/integrations/src/commerce/shopify/{adapter.rs,
transport.rs, auth.rs}`, header camera line. Tests: p50, p55, p63 files; settings UI tests.

## Files expected to change (write perimeter)
The files above as needed; `crates/testkit/tests/p68_setup_polish_address_password_shop.rs` (new); settings UI tests. ≈ 14 paths, ≈ 400 lines.

## Criteria (5 criteria, 6 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | address shown / changed | fake network address A, camera last uploaded to A; address becomes B | step 3 shows B; header + Health sentence with B | `[rule]` backlog 2026-11-11 |
| C2(a) | easy password | Generate; Show password | 16 chars from the allowed set, groups of 4; shown again equal | `[rule]` backlog 2026-11-11 |
| C3(a) | domain normalised | `https://X.myshopify.com/admin/ `, `admin.shopify.com/store/x`, ` key ` | stored `x.myshopify.com`; keys trimmed; a bad domain → the field sentence, no `builder error` | `[rule]` backlog 2026-11-11 |
| C4(a) | camera login with Save | type user/password, Save settings; leave with unsaved login | saved to the Keychain (not settings.json); the prompt asks | `[rule]` backlog 2026-11-11 |
| C4(b) | fingerprint reachable | fingerprint pending | header action opens the box with Confirm | `[rule]` backlog 2026-11-11 |
| C5(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (known intermittent `p33_split_grid` aside), clippy, fmt, typecheck, sidecar stub tests green | `[rule]` master plan §7.2 |

## Notes
- **Hard rules:** never change the camera's settings; secrets only in the Keychain; no address in committed files. Default implementer (Codex; Claude when out).

## Review mode (owner 2026-11-10: fast; kept)
- **Implementer:** high effort; run the full closing gate once (`cargo test --workspace`, clippy, fmt, typecheck, `npm test`) and put the stamp in the handoff.
- **Reviewer (time-boxed, still correct):** do not re-run `cargo test --workspace`. Run this phase's new test file(s), the test files of the crates and features the diff touches, clippy, fmt, typecheck and the touched UI test files; trust the implementer's full-suite stamp unless it is missing or contradicts what you see. Read every changed production path against the rows; only B1–B4 findings block. Aim to finish in about 15 minutes.

## Review log
- **2026-11-11 implement r1 (Claude Opus in the `codex` slot).** Built: Settings → Camera always lists the FTP steps; step 3 carries this
  Mac's address (re-read every 5 s) with **Copy address**. The receiver remembers the local address of its last accepted connection
  (`<data>/camera/last_upload_address`, never logged); when this Mac no longer has it, Health item `camera_address_changed` and the header
  camera line say `This Mac's address changed to <new> — set Host Name on the camera`. FTP password: 16 characters from
  `abcdefghjkmnpqrstuvwxyz23456789`, shown in groups of 4; **Show password** reads it back from the Keychain; stored passwords keep working.
  Shop domain normalised and client ID/secret trimmed on save (`SettingsSetContext::prepare`); the adapter refuses a domain that cannot be a
  host name with `The shop domain looks wrong: <value>` before any request; a refused grant reads `The Shopify client ID or secret is wrong`.
  The camera login is saved by **Save settings** (Keychain) and counts as an unsaved change; the separate button is gone. The fingerprint box
  is one component, shown in Settings → Camera and from the header's **Confirm the camera's fingerprint**; it also reads a fingerprint that
  arrived before it opened (`camera_fingerprint_pending`). Tests: `p68_setup_polish_address_password_shop.rs` (C1a, C2a, C3a) and
  `p68_setup_polish.test.tsx` (C1a, C2a, C4a, C4b). Assumptions and deviations: see the handoff (book 10E R-I21 and 09A wording now differ
  from the owner's rule; orchestrator to fold).
