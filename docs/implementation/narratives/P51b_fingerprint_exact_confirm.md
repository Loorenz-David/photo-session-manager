# Phase P51b_fingerprint_exact_confirm: Confirm trusts only the fingerprint you actually compared

## What you can do now
When the camera connects over Wi-Fi for the first time, Settings → Camera shows its fingerprint. Pressing **Confirm** now trusts exactly that fingerprint and nothing else. Before, Confirm meant "trust whatever key the camera shows next time", so a different device on the network could have been trusted in its place. If the camera reports a different fingerprint before you press Confirm, the app refuses and tells you: "The camera shows a different fingerprint now. Compare the new one." The card stays open so you can compare again. If a camera you already trusted shows a new key while Settings is open, the card is now titled **Fingerprint changed** at once, not only when you reopen Settings.

## Try it yourself
1. Run this phase's checks. They use stand-ins, so no camera is needed:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p51b_fingerprint_exact_confirm
   tools/sidecar/build.sh --stub --test
   cd apps/desktop && npx vitest run src/features/settings/__tests__/p51_camera_section.test.tsx
   ```
   You should see `4 passed`, then `21 tests, 0 failed`, then all Settings → Camera checks passing.
2. (Camera, optional; the hands-on phase covers it too.) Remove the stored fingerprint, connect the camera over Wi-Fi and open Settings → Camera. Compare the fingerprint with the camera's menu (MENU › Network › Network Option › Access Authen. Info) and press **Confirm**. At the next connect the helper reports a match and the card does not come back.

## Choices made for you
- Starting the camera helper by hand with its `--accept-fingerprint` option still trusts the first key it sees, as a tool for diagnosis at the terminal. The app never uses it. Say "make the terminal option ask too" to change it.
- If the camera's key already matches the stored one, a pending Confirm stays waiting instead of being discarded. It only takes effect if the camera later shows exactly that key. Say "drop a pending Confirm on every connect" to change it.
- If the app cannot write the trusted fingerprint to disk, it still connects this once and asks again next time, as before. Say "refuse to connect when the fingerprint cannot be saved" to change it.
- The helper compares fingerprints in the same lower-case form the card shows you, and stores the camera's original form on disk, as before. Nothing to decide unless a future Sony update changes the format.

## Not yet
- **Not mine** still does not stop the app from trying that network again. The card comes back at the next retry, within about 30 seconds.
- Nobody has tried this with the real camera yet. The hands-on phase does.

## Needs you
Nothing.
