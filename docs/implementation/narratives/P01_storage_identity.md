# Phase P01_storage_identity: The app's memory for items, sessions and photos

## What you can do now
Nothing new is visible yet: the app still opens on the Health page. Underneath, it now has the
database every later feature writes into. An item can be created with its own SKU and gets its first
session at the same moment, and more sessions can be added to it later. A built-in "Unassigned"
bucket exists from the first launch. The app recognises one photo file however its path is spelled
(accents typed two ways, upper or lower case, through a shortcut folder), so the same shot is never
listed twice. It can fingerprint a RAW file without touching it, and later tell you if that file
was changed or has gone missing. Two items can never end up sharing a capture folder, even when
their SKUs differ only in upper and lower case. Once an item has photos, changing its SKU no longer
moves its capture folder.

## Try it yourself
1. Start the app. The first line selects Node 22, which the frontend tools need:
   ```
   export PATH="$HOME/.nvm/versions/node/v22.22.3/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
   The Health page opens as before. Leave the app running.
2. In a second Terminal window, list the tables the app created:
   ```
   sqlite3 ~/Library/Application\ Support/com.loorenz.sessionmanager/session_manager.db '.tables'
   ```
   You should see `items`, `sessions`, `photos` and the other tables, alongside `settings` and `capture_events`.
3. Look at the sessions:
   ```
   sqlite3 ~/Library/Application\ Support/com.loorenz.sessionmanager/session_manager.db 'select id, item_id from sessions;'
   ```
   A fresh database shows one line, `1|`. That is the Unassigned bucket: session 1, with no item.
4. Optional. Run this phase's checks, which create items, photos and SKU changes in a throwaway
   folder:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p01_storage_identity
   ```
   The last line should read `test result: ok. 20 passed`.

## Choices made for you
- When you edit an item's SKU and leave the barcode empty, the old barcode is kept. There is no
  way to remove a barcode yet. Say "let me clear a barcode" to change this.
- Creating a new item requires a title and an item type as well as the SKU. Say "let me create an
  item from just a SKU" if the New session screen should allow that.
- When a photo is added to a session, the session's "last activity" becomes the moment the photo
  arrived. This decides where the session appears in the sessions list. Nothing to change.
- An item and its first session are saved together: either both exist or neither does. Nothing to change.
- Item and session numbers reach the screen as ordinary numbers. That is safe far beyond any
  number of sessions you will ever create. Nothing to change.

## Not yet
- No screen shows items or sessions yet. The sessions list and New session arrive with the app
  shell.
- New captures do not create photos yet. That comes with capture arrival, after the Capture One
  connection.
- Proposing a SKU from your prefix, and warning that a SKU is already in the shop, come later.
- The file-fingerprint check runs only when publishing or the health screen asks for it. Both come
  later.

## Needs you
Nothing.
