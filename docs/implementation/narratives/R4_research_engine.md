# Phase R4_research_engine: a complete research run works from start to finish, against stand-in providers

## What you can do now
Nothing new appears in the app yet. Underneath, the research agent can now do a whole run on its own:
- It looks at your photos, searches for matching listings and reads them, trusted sites first.
- It compares the listings with your cover photo, reads labels and condition, and then fills the form with suggestions and a step-by-step trail of what it did.

Every call to a provider is written down before it is sent. A photo you deleted is never sent. A photo only ever leaves the Mac as a small, cropped copy with no metadata.

Runs start on their own after a quiet moment, one at a time across the app, and stop at your cost limits. If the app is killed in the middle, the run picks up where it was and ends exactly as if nothing had happened. Adding a photo later only redoes the work that photo changes.

All of this is proven against stand-in providers that answer from a script. Real providers and the screens come in later phases.

## Try it yourself
1. Open Terminal in the project folder and run whole research runs against the stand-ins. One of them fills the designer (medium, from two trusted sites), the wood and the scratches; others show the cost cap stopping the reading, and refusals and failures ending as they should. Every line should say `ok`:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-research --test engine stages::
   ```
2. Watch runs being killed at six different moments and resumed. Each one is compared with a run that was never interrupted. The same command shows that follow-up runs only redo what changed:
   ```
   cargo test -p psm-research --test engine resume::
   ```
3. See when runs are created and started: the quiet period, one at a time, the daily limit, being offline:
   ```
   cargo test -p psm-research --test engine lifecycle::
   ```

## Choices made for you
**What research knows about your shop and the form**
- The designer field research treats as "who made it" is your shop's **Designer** field. Your shop has no manufacturer field, so there is only one. Say so if another field should count as designer or maker.
- When it is told which fields to look for, research looks for what is visible in the photos, reads designer, measurements and visible features from listings, and reads condition fields from the condition photos. Say so if a field should be looked for somewhere else.
- A photo's look at a visible feature comes from its first photo, or from a clearly read label if no photo shows it. Say so if a later photo should win.

**What a run sends and records**
- Research uses a photo only once its Capture One preview has caught up with its latest edits, framed by its current crop. Before that, research waits. Say so if it should use an older preview.
- A call whose photos were all deleted before it left sends nothing and records nothing. Say so if you want such a call kept in the record.
- When a page can't be opened, it is noted as "could not be read" and the run goes on. Say so if a dead page should stop the run.
- A listing page that turns out to be this item's own page in your shop is noted and skipped as evidence. Say so if you want it read anyway.
- When the model refuses to answer, that answer is recorded as failed, and the call is still paid for. Say so if refusals should cost nothing.
- When the app is offline, the call that couldn't leave is not kept as "sent" anywhere. Say so if you want offline attempts listed.
- If a key shows up in a provider's reply, it is hidden in the record, and one warning is logged per call. Say so if you want a warning per attempt.
- Attempts are counted separately for each page reader and each model call of a step. Say so if they should share one count.
- When a run fails, its message reads "key rejected by …" for a bad key, and otherwise names the step that failed. Say so if you want different wording.

**How a run decides and writes**
- The model is asked for its explanations only when at least one field was decided. Fields that stay empty are listed together in one final step. Say so if you'd like one step per empty field.
- An earlier suggestion that a new run no longer supports is lowered to low, with a note saying why. That also happens when the run stopped at your cost limit before it could check that field again. Say so if a run that hit the limit should leave earlier suggestions alone.
- The final "decide" step always runs again in a follow-up run, so it is checked against the form as it is now. Every other step is reused when its inputs did not change. Say so if decide should be reused too.
- A new cover photo that finds the same listing pages reuses what was already read from them. Say so if a new cover should re-read every page.
- "n fields filled" counts only fields the latest run actually changed. A follow-up that confirms the same values shows 0. Say so if it should count every field the run supports.

**When runs start**
- Entering your keys does not start research on sessions that already have photos. The next photo change or **Run again** does. Say so if entering keys should start research on the session you have open.
- Pressing **Run again** while something essential is missing (no shop, no key, no price) shows that reason and queues nothing. Say so if it should queue anyway and wait.
- When a photo's preview is missing, research waits up to 2 minutes, counted from the session's last change. If no photo has a preview at all, it keeps waiting, even after **Run again**. Say so if Run again should skip that wait.
- Being offline doesn't stop a run from starting. It pauses at its first call and retries at least once a minute. Say so if it should not start while offline.
- The Unassigned session never gets research and shows no Research button. Say so if it should.
- "Today" for the daily cost limit is this Mac's calendar day, ending at local midnight. Say so if you want a different day boundary.

**Keys, settings and what's left for later**
- Saving an empty key removes it. Removing a site that isn't listed changes nothing. Say so if either should show an error.
- A price in the settings can have a per-unit price, a fixed price per call, or both. Say so if every price must have both.
- Exporting research records and deleting old ones arrive with the retention phase; for now export says it isn't available yet.
- The places in the app that will tell research "this session changed" are chosen: a new photo, sorting, reordering, deleting, moving or copying photos, cropping, and a new Capture One preview. They are connected in the wiring phase. Say so if another action should count as a change.

## Not yet
- Nothing in the app starts or shows research yet. The panel, Settings → Research and the wiring come in later phases.
- No real model, search or page-reading provider is contacted. The real adapters come later, after the benchmark and your picks.
- Models and prices are empty by default, so a real run cannot start until the defaults phase fills them in.
- A few small rough edges are set aside for later:
  - "Waiting for Capture One" can appear a little before the 2-minute wait is over.
  - After a crash, a step gets one fewer retry.

## Needs you
Nothing.
