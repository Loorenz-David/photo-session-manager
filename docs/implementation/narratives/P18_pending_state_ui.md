# Phase P18_pending_state_ui: buttons that show they're working, placeholders while pages load, and faster item search

## What you can do now
When you press a button that talks to Capture One or Shopify, such as Create session, Export, Publish, Save settings or Retry checks, it dims at once with a "…" mark, and a second click does nothing until the first one finishes. If the action fails, the button comes back and you get a message. The sessions list, an open session's photos and stage, Health, Settings, the item form, the publish preview and the item card show grey placeholder blocks while they load, instead of "Loading…" text or a brief "No sessions yet". A page that refreshes keeps showing what it had, so there is no flicker. Item search in New session and Assign product now shows your own items as you type, and adds Shopify matches a moment later. The letters never wait on the shop, and a slow shop answer for something you already typed past is thrown away. Choosing a Shopify match in Assign product now brings that product into the app first, so the photos go to the right item.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or shop needed):
   ```
   export PATH="$HOME/.cargo/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p18_item_search
   cd apps/desktop
   npx vitest run src/features/pending src/features/sessions/__tests__/p18_item_search.test.tsx
   ```
   The first should read `test result: ok. 1 passed` and the second `Tests  10 passed (10)`.
2. Start the app with a few sessions already in it:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
   The sessions list shows grey placeholder rows for a moment, then your sessions, with no "No sessions yet" flash. Open a session: placeholder tiles, then the photos.
3. Click New session and type part of a product title. Your own items appear as you type, with "searching the shop…" in the heading. Shopify matches are added a moment later and the heading changes to "n in the shop". Keep typing: the earlier shop matches disappear until the shop answers the new text.
4. Press Create session, Save settings, Retry checks, Export or Publish. The button dims with "…" at once, and clicking again does nothing until it finishes.
5. Open Health and Settings: grey placeholders appear briefly, never "Loading…".

## Choices made for you
- Your own items are searched 75 thousandths of a second after you stop typing, and the shop 300 thousandths after. Say "search the shop sooner/later" to change it.
- A busy button shows a "…" after its label rather than a spinner. Say "use a spinner" to change it.
- If the shop search fails, the heading says "shop unavailable". Say so if you want other wording.
- The photo review buttons and keys (keep, condition photo, delete, and swiping) don't show a busy state, because they finish instantly and must not interrupt your rhythm. Nothing to change.
- The placeholders are plain grey blocks with a gentle pulse. The real design can be applied later in one place.
- The fix after the first review: when you kept typing, the previous text's Shopify matches came back on screen, labelled as finished, while the shop was still answering the new text. Shop matches are now shown only for the exact text they were found for, in both New session and Assign product.

## Not yet
- During a real Shopify outage the search usually says "0 in the shop" rather than "shop unavailable". You could then create a local item for a product the shop already has.
- In the item form, if you accept two AI suggestions in quick succession, the second can be dropped without a message.
- An open session's page still uses its own loading logic rather than the shared one. It behaves the same.
- The placeholders and busy marks have no visual design yet.

## Needs you
Nothing.
