# Phase P02_review_order: Sorting, ordering, deleting with undo, and moving photos between sessions

## What you can do now
Nothing new is visible yet: there is no screen for sorting photos until the workspace arrives. Underneath,
the app can now do everything you will do to a session's photos. It can mark a photo Main or Condition, put
photos in order, delete one (the file stays on disk), and undo the last delete. Undo puts the photo back in
its old spot. It can also move or copy photos into another session. Each list stays numbered 1, 2, 3 with no
gaps, so "Main 01" is always the cover. When you change a session you have already published, the app
remembers that the shop copy is now out of date. That is the "needs republishing" signal later screens will
show.

## Try it yourself
1. Run this phase's checks. They sort, reorder, delete, undo, move and copy photos in a throwaway folder and
   leave nothing behind:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p02_review_order
   ```
   The last line should read `test result: ok. 25 passed`.
2. Optional. Start the app. The first line selects Node 22, which the frontend tools need:
   ```
   export PATH="$HOME/.nvm/versions/node/v22.22.3/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
   The Health page opens as before. Nothing about it has changed in this phase.

## Choices made for you
- The app remembers your last delete as a single small note on the session. Nothing to change.
- Marking a photo with the class it already has, or dropping it on the spot it already holds, does
  nothing at all. It does not count as a change and does not make a published session "needs
  republishing". Say "count every tap as a change" to change this.
- You can move or copy photos into a published session once it has been changed since publishing.
  You cannot while it still matches the shop. Say "never allow published sessions as targets" to change this.
- You can move several photos from different sessions in one go. Each session involved counts as changed
  once. Nothing to change.
- After a move or copy, the app reports back on every session involved, in a fixed order. Nothing to change.

## Not yet
- There is no screen for any of this yet. Sorting with keys and swipes, dragging to reorder and the
  undo toast come with the workspace.
- Sorting a photo does not yet set its colour tag in Capture One. That comes with the Capture One connection.
- The sessions list, which shows each session's state and groups sessions by day, comes next.
- When you sort in the workspace, the sessions list does not refresh by itself yet. This is noted for the
  screen phases.

## Needs you
Nothing.
