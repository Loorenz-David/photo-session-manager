# Phase P03_session_state_list: Session status, the row label, grouping by day, and the sessions list

## What you can do now
There is nothing new on screen yet. The sessions list screen arrives with the first UI phase. Underneath, the
app now works out the state of every session. A session is In progress while photos are still to sort, Ready
once every photo is sorted and previewed, then Publishing (with a percentage), Publish failed or Published. It
also decides the short label each row shows, such as "3 to review", "Publishing 42%" or "Publish failed". The
list groups sessions by when you last worked on them: Today, Yesterday, This week, Last week, then by month
("October 2026"). The groups use your Mac's own calendar days, so a session from 23:30 counts as that day,
including on the night the clocks change. You can filter by Open, Published or All, and search by title or SKU.

## Try it yourself
1. Run this phase's checks. They build sessions in a throwaway folder, set each one up in a different state,
   and check the status, label, grouping, filters and search:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p03_session_state_list
   ```
   The last line should read `test result: ok. 28 passed`.
2. Optional. Start the app. The first line selects Node 22, which the frontend tools need:
   ```
   export PATH="$HOME/.nvm/versions/node/v22.22.3/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
   The Health page opens as before. Nothing about it has changed in this phase.

## Choices made for you
- Only publish attempts you confirmed count towards a session's state. A publish you started but never
  confirmed is ignored. Nothing to change.
- The numbers on the Open / Published / All tabs count every session, not just the ones matching your search,
  so they stay steady while you type. Say "make the tab counts follow the search" to change this.
- A row shows a percentage only while it is publishing. Nothing to change.
- A session's picture in the list is Main 01's edited preview, or its original preview while no edited one
  exists. Say "use a different cover" to change this.
- The Unassigned bucket never appears as a session row or in any day group. It gets its own count instead.
  Nothing to change.
- The first review found one check that could not catch a mistake. The check was meant to prove that the
  newest publish attempt decides a session's state, but it would have passed even if the app picked an older
  failed attempt. The fix added one line to that check, so it now catches that mistake. The app's behaviour
  did not change.

## Not yet
- There is no sessions list screen yet. It comes with the first UI phase.
- The Unassigned count also includes photos you deleted, so the Unassigned card could show while the bucket
  holds only hidden photos. This is noted for the screen phase.
- The list does not refresh by itself yet when you sort photos. This is noted for the screen phases.
- Publishing itself comes later. For now the app only reads publish attempts recorded by the checks.

## Needs you
Nothing.
