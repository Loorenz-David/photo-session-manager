# Phase P68_setup_polish_address_password_shop: setup is easier — the camera steps show this Mac's address, the FTP password is easy to type and can be shown again, the shop settings clean themselves up, and the camera login is saved with Save settings

## What you can do now
Settings → Camera always lists the camera's FTP steps. Step 3 now shows this Mac's real address on your network, with a **Copy address** button, so you no longer have to look up the host name. If the Mac later gets a different address (another Wi-Fi, a new router lease), the header camera line and Health say "This Mac's address changed to … — set Host Name on the camera". The message goes away by itself when the Mac is back on the old address.

**Generate password** now makes 16 lowercase letters and digits, without the look-alikes 0, o, 1, l and i, shown in groups of four. **Show password** brings the same password back from the Keychain any time, and your current password keeps working until you generate a new one. The shop domain is cleaned up when you save: a pasted `https://YourShop.myshopify.com/admin/` or an `admin.shopify.com/store/yourshop` address becomes `yourshop.myshopify.com`, and stray spaces around the client ID and secret are removed. A domain that can't work now says "The shop domain looks wrong: …" instead of "http: builder error", and wrong keys say "The Shopify client ID or secret is wrong". The camera's Access Authentication login is saved with **Save settings**. When the camera asks you to confirm its fingerprint, the header's camera card has a button that opens the fingerprint box directly.

## Try it yourself
1. Run this phase's checks. They use a stand-in shop and a local test connection, so nothing real is touched:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p68_setup_polish_address_password_shop
   cd apps/desktop && npx vitest run src/features/settings/__tests__/p68_setup_polish.test.tsx
   ```
   You should see `3 passed; 0 failed`, then 4 passing checks.
2. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop && npm run tauri -- dev
   ```
3. Open Settings → Camera (with camera support installed). Step 3 shows this Mac's address. Click **Copy address** and paste it anywhere: it's the same address.
4. Click **Generate password**: 16 letters and digits in groups of four. Leave Settings, come back and click **Show password** (macOS may ask to allow Keychain access). The same password appears. On the camera, type it without the spaces.
5. Type the camera's Access Authentication user and password, then click **Health**. "Save your changes?" appears. Choose **Save**. The fields empty, and the login is in Keychain Access under `com.loorenz.sessionmanager.camera`.
6. In the Shopify section, type ` https://YourShop.myshopify.com/admin/ ` as the domain and click **Save settings**. The field now reads `yourshop.myshopify.com`.
7. After at least one photo has arrived over FTP, switch the Mac to another network. Within a few seconds the header camera line and Health say "This Mac's address changed to … — set Host Name on the camera".
8. When the camera asks for its fingerprint, open the header's camera card and click **Confirm the camera's fingerprint**. The fingerprint box opens there.

## Choices made for you
- "The address the camera last uploaded to" is the Mac's address on the camera's last connection to the FTP receiver. It's kept in the app's data folder, so the warning also appears after a restart on a new network, and it's never written to a log. Say "only count a connection that logged in" if you want it stricter.
- When the camera's address is gone, step 3 shows the Mac's first home-network address (192.168…, 10…), then any other address. Say "prefer the cable/Wi-Fi address" if the wrong one shows up.
- The address is checked again every 5 seconds while Settings or the header is on screen, rather than the instant the network changes. Say "check faster" if 5 seconds feels slow.
- In the header, the "address changed" sentence replaces the camera's other messages except "taking/receiving a photo". The card still shows both. Say "show the camera's own message first" to change that.
- The password is stored and typed without spaces. The groups of four are only for reading.
- A shop domain is checked when the app talks to Shopify, not when you save, so a domain saved before this change is also named in the message. Say "refuse a bad domain on save" to change that.
- The wrong-keys message replaces the old "Shopify credentials rejected" text. Nothing else about that error changed.
- If both camera login fields are empty, Save settings leaves the stored login alone. If you type either one, both are saved, so typing only a new user name also replaces the stored password with an empty one. Say "save only the field I typed" to change that.

## Not yet
- Nothing here has been tried on your work Mac's real network or with the camera. Steps 3, 7 and 8 are the first real checks.
- If Save settings fails (for example, a wrong port), the camera login has already been saved and its fields have emptied. The rest of the settings are not saved. This is recorded for a later fix.
- A printer or other device that touches the camera's FTP port could, in rare cases, show or clear the "address changed" message wrongly. This is recorded for a later fix.
- The project's reference book still describes the old 32-character password that is shown only once, and the old "Shopify credentials rejected" text. The orchestrator will update it.
- The full test run was the implementer's. The review re-ran this phase's checks, the checks for every part the change touches, the formatting check, the code-style check (clippy) and the type check, and all passed.

## Needs you
Nothing.
