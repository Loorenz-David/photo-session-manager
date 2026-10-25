# Phase P07_shell_sessions_ui: the app has its main window, the Sessions list and New session

## What you can do now
The app now opens on a real Sessions page instead of the old health report. Sessions are grouped by when you last worked on them. Open, Published and All filters show their counts, and you can search by SKU or title. Each row has a status label such as "In progress" (with how many photos are left to review), "Ready · Publish", "Published" or "Publish failed". The camera light in the header is green when Capture One sees the ZV-E10 and red when it does not. Clicking the light retries, and hovering over it tells you what to check. Press N (or click New session) to search your items. If nothing matches, a "Create '…'" row makes a new item with a proposed SKU you can edit, then opens its session. When photos are waiting in Unassigned, a card shows how many. Opening a session points Capture One's capture folder at it. Going back to Sessions sends new shots to Unassigned again.

## Try it yourself
1. Run this phase's checks. They use pretend services, so neither Capture One nor the camera is needed:
   ```
   export PATH="$HOME/.cargo/bin:$HOME/.nvm/versions/node/v22.22.3/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p07_shell_sessions_ui
   cd apps/desktop && npm test
   ```
   The first should end with `test result: ok. 4 passed`, the second with `Tests  19 passed (19)`.
2. Open your Capture One session, tether the camera, and start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
   You see the Sessions page. The camera light is green. Unplug the camera and it turns red. Click it to retry.
3. Press N and type "Brass floor lamp". Choose "Create 'Brass floor lamp'", type a SKU (for example `TEST-0001`), and click Create. The empty session view opens with "Ready for the first photo".
4. Quit the app while this session is open, then start it again as in step 2. It reopens the Brass floor lamp session, because that is still where Capture One saves new shots.
5. Click back to Sessions. The row shows "In progress", and the counts read one more under Open and All. Quit and start again: this time the app opens on the Sessions list.

## Choices made for you
- The fix changed one thing. If you quit with a session open, the app reopens that session at the next start. It no longer shows the Sessions list while Capture One still saves into that session's folder. The other option was to always open on the list and send new shots to Unassigned at start. Say so if you prefer that.
- Pages are addressed as `#/` (Sessions) and `#/session/<number>`. The old health report is still at `#/health`. Nothing to change.
- Search waits a moment after you stop typing (about a quarter second) before asking. Say so if it feels slow.
- "Last 7 days" means today and the six calendar days before it. Nothing to change.
- The light or dark choice is remembered by the app window until the Settings screen arrives. Then it moves to the settings file.
- A new item's type starts empty, so the list and header show its title until you set a type.
- If the SKU you type is already used by one of your items, the dialog says so with that item's title, and Create stays off.
- Only rows that are ready or failed get a selection checkbox.
- Thumbnails stay grey until the workspace phase turns on preview files.
- When the shop already has the proposed SKU, the dialog says so and blocks Create, but it does not yet suggest the next free number. This cannot happen until the Shopify connection exists, and that phase will add the suggestion.

## Not yet
- The session view is a placeholder. Shooting, sorting and reordering come with the workspace phase.
- The "Review & publish" button, the Ready and Retry buttons on rows, and the bell are visible but do nothing until the publish phase.
- Settings (including the SKU prefix that pre-fills new SKUs) and the health screen come in the last phase. Until then, type the SKU yourself.
- The Open, Published and All counts ignore the time filter. They count all sessions.

## Needs you
Nothing.
