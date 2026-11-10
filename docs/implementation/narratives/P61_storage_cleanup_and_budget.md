# Phase P61_storage_cleanup_and_budget: the app now frees disk space by itself, and Settings shows what fills your disk

## What you can do now
Session Manager now cleans up three kinds of files that only waste space, and never touches your RAWs:
- the second copy of every photo that lands in the duplicates folder when the camera sends it both over the cable and over Wi-Fi;
- Capture One's preview exports, once the app has copied them;
- the large master TIFFs that publishing and export need.

A duplicate goes 24 hours after it arrived, and only when its bytes are exactly the same as the photo that was kept. Settings now has a **Storage** section. It shows the free space on your disk, how much space RAWs, duplicates, Capture One's preview exports, previews and masters use, and a **Minimum free space** (20 GB to start). When free space drops below that minimum, the app clears duplicates first, then preview exports, then masters, and stops as soon as there is enough room again. If that still isn't enough, the header and the Health page say `Disk almost full: N GB free; nothing more can be cleared safely`. All of this runs quietly in the background, a few files at a time, and waits while a photo is being taken or transferred, a preview is rendering, or an export or publish is running.

## Try it yourself
1. Run this phase's checks. They use a stand-in disk, camera and Capture One, so nothing on your real disk is deleted:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p61_storage_cleanup_and_budget
   cd apps/desktop && npx vitest run src/features/settings/__tests__/p61_storage_section.test.tsx
   ```
   You should see `7 passed; 0 failed` and then `3 passed`.
2. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop && npm run tauri -- dev
   ```
3. Open Settings (the gear in the header) and scroll to **Storage**. After a few seconds you should see your free space and one line each for RAW files, Duplicates, Capture One preview exports, Previews and Masters.
4. Take a few photos on the cable with Wi-Fi transfer on. The Duplicates line grows. A day later it shrinks again. The app's log shows a line starting `storage.cleared rule=duplicate_24h` for each file removed.
5. Once previews have rendered, the "Capture One preview exports" line should drop back to about 0 MB.
6. Set **Minimum free space (GB)** higher than your current free space and press **Save minimum**. Within a few seconds, duplicates and preview exports are removed and the numbers drop. If that isn't enough, the header shows the "Disk almost full" message. Set it back to 20 afterwards. Your RAWs stay where they are.

## Choices made for you
- A duplicate's 24 hours start when the app received it. For old duplicates the app has no record of, they start at the file's date. Say "count from the file's date" if you prefer that for all of them.
- A duplicate is removed only while the photo it duplicates is still on disk. If you deleted that photo yourself, the duplicate stays, since it may be your only copy.
- A preview export is removed only when it is byte-for-byte the same as the app's copy. An export Capture One has rewritten since waits until the app copies it again.
- The app looks for leftover preview exports from earlier days in the session's `Output/_session_manager` folder.
- Under pressure, a master is kept if an export run still needs it or if its session is publishing.
- "Busy" means one of these: a photo is being taken on the cable or arriving over Wi-Fi, Capture One is rendering, the camera take-back is running, or an export or publish is running. While busy, the app removes nothing.
- Files are removed in batches of at most 20, two seconds apart. Sizes are recalculated at most every 5 minutes and after each clean-up. While the app is busy for a long time, Settings may say "Calculating…".
- Minimum free space is whole gigabytes, from 0 to 100 000. 0 turns the minimum off. A new value takes effect within a second or two, with no restart.
- The "N GB free" in the warning is what will be free after the app has cleared what it can, with one decimal.
- The warning sits as a small line of text in the header on both pages and is re-checked every 30 seconds. The Storage section refreshes every 5 seconds while open.
- Storage has its own Save button and does not use the main "Save settings" button.

## Not yet
- None of this has run on your Mac with real photos yet. Steps 3–6 are for your next hands-on run.
- Clearing published items' RAWs after a few days, with Shopify as the archive, and the per-item "keep files" pin come in the next phase.
- While a publish is running, the clean-up waits completely, even for duplicates. If a publish is stuck, nothing is cleared until it finishes or fails.
- While the app is busy for a long time, for example during a shoot with renders queued, the header's "Disk almost full" message is not updated.
- The first time it starts on a Mac that already has many old duplicates, the app checks them a few at a time. That keeps the disk busy for a few minutes in the background.

## Needs you
Nothing.
