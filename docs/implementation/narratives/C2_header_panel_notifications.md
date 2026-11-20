# Phase C2_header_panel_notifications: You see which products have questions, and you are told when one arrives

## What you can do now
Every page now has a chat icon in the header: Sessions, a session, Settings and Health. It shows how many conversations have a pricer message you haven't read yet. Inside a session it counts only that session's conversations, and a small dot tells you when other sessions have news. Click the icon and a panel opens on the right, with an **Open** tab and a **Resolved** tab (the last 30 days). Rows with news come first, then the blocking ones, then the most recent. Each row shows the SKU and product, the topic, a status dot, amber "Pricing is waiting" when the pricer is blocked, the first line of the newest message and its time, an unread dot, and "Not on this Mac" when no session here has that SKU. On the Sessions page, a session with open questions shows "N open", or amber "Pricing is waiting". When a question or a pricer reply arrives you get a toast with **Open**, one soft chime, and a Mac banner if Session Manager is in the background. The bell button in the panel mutes the chime.

## Try it yourself
1. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   npm run tauri -- dev
   ```
2. In Pricing, ask a blocking Condition question about a product you photographed on this Mac. Within about 10 seconds you should hear a chime and see a toast "New question · <SKU> — Condition". The header icon shows 1, and that session's row on the Sessions page shows "Pricing is waiting".
3. Click the icon. The panel lists the conversation under **Open**, with the first line of the question. Open that session: the panel now lists only that session's conversations. Clicking a row shows a short "arrives with the next step" note for now.
4. Put Session Manager in the background and ask a second question. A Mac banner appears. The first time, macOS asks whether Session Manager may show notifications. Click the banner: Session Manager comes forward, and the toast with **Open** is still there.
5. Press the bell button in the panel (it turns into a crossed-out bell) and ask again. No chime this time, but the toast still appears. The **Sound** switch in Settings → Pricing has turned off too: it is the same setting.
6. To run the automated checks for this phase:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-core --lib mechanisms::conversation_rules
   cargo test -p psm-testkit --test suite c2_conversations_lists::
   cargo test -p psm-infra --lib chime
   cargo test -p psm-desktop c2_tests
   cd apps/desktop && npx vitest run src/features/conversations src/features/sessions/__tests__/c2_session_badges.test.tsx
   ```

## Choices made for you
- The icon is on every page, and its number counts the conversations with a pricer message you haven't read, resolved ones included.
- A conversation belongs to the session whose SKU is exactly its SKU (capital letters matter). Otherwise it reads "Not on this Mac". Say "match SKUs ignoring capitals" to change it.
- If a product was photographed in more than one session, its conversations go to the newest of those sessions, the same rule the rest of the app uses.
- The Resolved tab lists conversations whose last update is at most 30 days old.
- The chime is a short two-note tone the app makes itself, under half a second, played through macOS, so it also sounds right after launch. It plays at the Mac's normal volume. Say "make the chime quieter" or "use another tone" to change it.
- The mute button in the panel and the Sound switch in Settings → Pricing are the same setting.
- The banner's title is "Session Manager", and its text uses the toast's words. Say which title you would prefer to change it.
- macOS asks once whether Session Manager may show notifications. If you refuse, banners never appear and the app cannot tell. Settings → Pricing now has a line saying where to turn them back on (System Settings → Notifications).
- Clicking a banner brings Session Manager forward. A toast that arrived while the app was in the background stays until you come back, then 10 seconds more. A toast that arrives while you are looking stays 8 seconds.
- **Open** on a toast opens the panel on that conversation and switches the list to all sessions, because the question may belong to another session.
- On the Unassigned page the icon behaves as it does inside a session. Unassigned photos have no product, so its number is always 0, and you only see the dot when other sessions have news. Say "treat the Unassigned page like the Sessions page" to show the count of all conversations there instead.
- Opening a conversation comes with the next step. Until then, clicking a row shows its SKU, product and topic with a short note.

## Not yet
- Reading a conversation, replying, attaching photos and "Mark answered" come in the next step. Until then, nothing marks a conversation as read, so the header number doesn't go down yet, even after the pricer resolves the question. It will go down once you can open a conversation.
- Jumping from a conversation to its session (Open session, the visit bar, Back) comes after that.
- Nobody has heard the real chime or seen a real banner yet: the automated checks stand in for them, and steps 2–5 above are the first real check. The orchestrator plans a live check against the real Pricing server after the last conversations step.
- Six small points went to the backlog: two code comments to reword, a rule to write down for the next steps, the Unassigned-page count (offered above), toasts that could be lost if the page changed while you were away, one test to loosen, and whether macOS reports banner clicks (to check live).

## Needs you
Nothing.
