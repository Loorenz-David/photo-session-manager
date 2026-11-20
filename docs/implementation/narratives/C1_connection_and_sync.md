# Phase C1_connection_and_sync: Session Manager connects to Pricing and keeps a copy of your conversations

## What you can do now
Settings has a new **Pricing** page, right after Shopify. The server address is already filled in. You paste the API key from Pricing and press **Save key**. The key goes into the macOS Keychain and is never shown again: the page then says **Saved**, with **Replace** and **Remove**. A line tells you in words whether it worked, for example "Connected to Dev store (your-shop.myshopify.com)", or which Pricing stores the key can see when none of them is your Shopify shop. Once connected, Session Manager quietly keeps a copy of the questions the pricers send to the photographer. It checks Pricing every 10 seconds in the background and never holds up a screen. **Last checked** shows when the last check succeeded, or the problem in words when the last check failed. Two switches, **Sound for new questions and replies** and **Mac banner when Session Manager is in the background**, are saved as soon as you flip them. They do nothing you can hear or see yet.

## Try it yourself
1. Start the app (it uses your real Keychain, so this is the place to paste your real key):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   npm run tauri -- dev
   ```
2. Open Settings → **Pricing**. The address is filled in. Paste the API key from Pricing (Settings → Service accounts) and press **Save key**. The field turns into "Saved" and the line reads "Connected to <your store> (<your shop>)".
3. Press **Test connection**: you get the same line. "Last checked" shows a clock time that moves forward every 10 seconds or so.
4. Turn Wi-Fi off for a minute. "Last checked" now says "Could not reach Pricing", and every other screen works as before. Turn Wi-Fi back on: within a few minutes at most, the time is back.
5. Press **Remove**: the key is deleted from the Keychain and the connection line disappears. Paste it again to reconnect. The copy is kept, because it is the same store.
6. To run the automated checks for this phase:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-testkit --test suite c1_conversations_connection::
   cargo test -p psm-testkit --test suite c1_conversations_sync::
   cargo test -p psm-desktop c1_tests
   cd apps/desktop && npx vitest run src/features/settings/__tests__/c1_pricing_settings.test.tsx
   ```

## Choices made for you
- Saving the key tests the connection at once; **Test connection** repeats the test.
- In this step Session Manager only reads from Pricing; nothing can be sent yet.
- If your Shopify shop is changed on this Mac, Session Manager makes no call to Pricing until you press Test connection again.
- If the key belongs to a different Pricing store, the local copy is emptied; the same store keeps it.
- The first connection is silent: it fetches the 200 oldest open questions plus everything mentioned in the last 30 days of Pricing's activity (Pricing's own limits).
- When Pricing can't be reached, the wait between tries doubles from 10 seconds up to 5 minutes. The first success brings it back to 10 seconds.
- The address must start with https. A key is never shown again after it is saved.
- If you save a different server address, Session Manager asks for Test connection again before it makes any call. Saving an empty address goes back to the default one.
- A wrong key on a Mac that was never connected says "Pricing refused the key: check Settings → Pricing", not "press Test connection". Say "show the press-Test-connection line instead" to change it.
- If Pricing can't be reached while you save the key or press Test connection, the line shows "Could not reach Pricing" for up to 10 seconds, and the previous connection is kept. Say "keep the error on screen until I close the page" to change it.
- "Last checked" shows a clock time (like 14:02:05), and the page refreshes it every 10 seconds while it is open. Say "show it as '2 minutes ago'" to change it.
- If you type the default address yourself, it is saved as "the default". A future change to the default address then reaches you without you doing anything.
- The two switches are simple On/Off buttons. Say "use the same switch style as elsewhere" once a shared switch exists.
- A check that clears an earlier error also tells the window, so "Last checked" updates at once.

## Not yet
- You can't see any conversation yet. The icon, the list, the counts, the toasts, the chime and the Mac banner come in the next step.
- Replying, "Mark answered" and the conversation view come after that, followed by jumping from a question to its item.
- Nobody has tried this against the real Pricing server yet. The orchestrator does a live check after the last conversations step, using a test question you create. Steps 2–4 above are the first real check, if you want to do it earlier.
- Five small points went to the backlog: three places where the written rules should be updated to match what was built, a "Last checked" write that does not wait its turn, and one test variable to rename.

## Needs you
Nothing.
