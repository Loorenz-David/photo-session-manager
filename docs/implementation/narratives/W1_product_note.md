# Phase W1_product_note: what research saw, corrected by your decisions

## What you can do now
When research looks at a session's photos, it now also writes a short factual note of what it saw: what the piece is, its form, wood and details from the main photos, and the wear on each condition photo. You find it in the research panel under the steps, headed "What research saw". Condition observations say which condition photo they mean. When you decide a field in AI review, your decision replaces what research saw. Say research saw oak and you set Teak: the line now reads "Wood: Teak (owner's decision)" with a small "Your decision" mark. If you recompose a main photo or change the condition photos afterwards, the block says "Photos changed since they were looked at" and offers **Look again**. Look again redoes only the two photo looks. It does no listing search and adds no new suggestions. This note is what the text writer will read in the next phases.

## Try it yourself
1. Start the app on scratch data with your research keys set and a few fields chosen, including one condition field:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev
   ```
2. Shoot or import a session with main and condition photos, press "Arranged — next" and let research finish.
3. Open Details and open the research panel. Under the steps, "What research saw" lists short observations. The condition ones carry "condition photo 1", "condition photo 2" and so on.
4. In AI review, accept or correct one value, for example the wood. The matching observation now shows your value with "Your decision".
5. Recompose a main photo. The banner "Photos changed since they were looked at" appears. Press **Look again**. Only the photo looks run (the cost grows by a few cents at most), no new suggestions appear, and the banner goes when it finishes. If you press it again while it runs, it says "Research is still working on this session".

## Choices made for you
- The note comes from the calls research already makes. There is no extra call, just a paragraph more of output. Say "make the note a separate call" to change it.
- Research's instructions changed, so on the next research of each existing session every step is looked at once more. That is a one-time cost.
- Your decision replaces what research saw. That covers accepted, corrected, kept-previous and dismissed values, and a value you typed by hand. Say "a typed value should not count as my decision" to change it.
- A dismissed value reads "Wood: not stated (the owner dismissed Oak)", so the writer knows not to bring it back.
- "Photos changed" means a photo research would look at now differs from the ones it looked at: another photo among the first ones, a recompose, or other condition photos. A new photo beyond the photo limit is not a change.
- Look again runs only when the photos changed (or research never looked at them) and nothing else is running on the session. It is refused with a sentence otherwise, or when the shop, the model key or a price is missing.
- Units whose photos did not change are copied, not paid for again.
- Condition photos are numbered among the photos your damage report keeps. An observation about a photo you took out of the damage report stays visible in the panel, marked "photo no longer in the condition photos", and the writer will not get it.
- A condition observation that names no valid photo is shown without a photo number and still counts.
- A Look again run stopped by the cost limit ends as "stopped at the cost limit", like research.
- Pressing "Research" / Run again, or "Arranged — next", while a Look again waits in the queue cancels the Look again. A full research run replaces it.
- A Look again run starts like a run you asked for by hand. It does not wait for missing previews, but it does wait when no photo has a preview at all.
- The "What research saw" block refreshes whenever the research panel does. It cannot be collapsed. Say "make it collapsible" to change it.

## Not yet
- Nothing writes texts from the note yet. The writer (title, description, condition report and search texts) comes in the next phases.
- The research panel does not yet say that the photos were looked at again after a Look again. That arrives with the writer block.
- One rough edge: if you un-choose every condition field after a note exists and then change a condition photo, the "Photos changed" banner can stay, because Look again then has no condition field to look for. It costs nothing, but the banner won't clear.
- No live try-out with the camera and the real model was done in the build sessions. Your try-out above is the first.

## Needs you
Nothing.
