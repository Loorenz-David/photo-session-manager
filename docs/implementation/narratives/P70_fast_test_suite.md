# Phase P70_fast_test_suite: the full test run takes about 8 minutes instead of over an hour

## What you can do now
The full set of automatic checks now runs in about 8 minutes on this Mac, down from 65 minutes. The reviewer measured 7 min 59 s, with all 946 checks passing. Most of the old hour was not the tests themselves. macOS checks every newly built program the first time it starts, and the checks used to be 63 separate programs. They are now one program, so that check happens once. One phase's checks can still be run on their own. The app itself is unchanged: the only change to the app's code is that the camera helper's 3-second "quit, then force-close" wait can now be shortened by the tests. In the app it is still 3 seconds.

## Try it yourself
1. Run one phase's checks on their own:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test suite p66_approve_on_arrival_and_session_order::
   ```
   You should see `6 passed; 0 failed; … 806 filtered out`.
2. Time the full run twice in a row. The second run is the normal time:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   time cargo test --workspace
   time cargo test --workspace
   ```
   The second run should finish in under 10 minutes, with `812 passed` for the main program. The first run after the code changes can take about 17 minutes, because macOS checks every newly built program before it starts.
3. Open the app and use it as before. Nothing should look or behave differently:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop && npm run tauri -- dev
   ```

## Choices made for you
- The "before" time was measured by running each of the 63 old programs once, recording separately the macOS start-up check and the test time. Say "measure it another way" to change that.
- A few checks change something shared by the whole program, such as the app log, or need the system's file-watching service to themselves. Each of these now runs in its own separate process, as before. Say "fold them back in" if you'd rather not have that.
- The stand-in camera helper used by the tests is now one shared copy instead of a fresh copy for every check. A fresh copy cost a macOS check each time. Say "copy it per test again" to undo this.
- Only the camera helper's quit wait was made adjustable. The other waits the plan listed already run on the tests' simulated clock, so they cost no real time. Say "make the other timings adjustable too" if you want them changed anyway.
- "Warm" is read as the second of two runs back to back. The first run after a change pays the macOS check.
- The libraries the app uses (image, database, hashing) are now built optimised in test and development builds. This made the checks noticeably faster: the main program's time went from 188 s to 115 s. Your own code still rebuilds quickly, and the shipped app is not affected. The first build after this change took a long time once. Say "drop the optimised libraries" to undo it.
- The interface checks (`npm test`) were left as they were. They ran in about 21 seconds without the load timeouts seen before. One older, known intermittent failure in the photo split grid showed up once.

## Not yet
- Switching between running one phase's checks and running everything currently rebuilds part of the code each time. Each rebuild pays the macOS start-up check again, so it can add a few minutes. That is noted for a later fix.
- Every full run also rebuilds the desktop app's test program, because one check rewrites the camera helper file. That adds a minute or two and is noted.
- A new phase's check file must be added to the list of checks that make up the one program. A file left off that list would never run, and nothing would warn about it. A guard for this is noted.

## Needs you
Nothing.
