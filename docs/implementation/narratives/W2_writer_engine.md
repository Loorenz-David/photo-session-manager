# Phase W2_writer_engine: the text writer, built and proven behind the scenes

## What you can do now
Nothing new to click yet. This phase builds the whole writer inside the research engine and proves it with stand-in providers. The next phase connects it to the app. What now exists: the writer's settings with the shipped writing guide in English and Swedish and the four style examples; exactly what the writer gets to see; a fixed check of every text it writes; and the rules for when a writing run starts. It also writes its texts as suggestions under the same rules research uses, and it raises questions and notes that never come back to something you already decided. Its cost, day limit, call record and export are shared with research. When it is connected, a text that passes the check arrives as a normal suggestion. A text that fails the check twice arrives marked low, with what failed written in its reason.

## Try it yourself
1. Read the shipped writing guide and examples. The app ships exactly the text in the writer master plan, section 9:
   ```
   open ~/Desktop/Developer/Photo-Session-Manager-agentic/docs/implementation/master_plan_writer.md
   ```
2. Run the writer's own checks: settings, what the writer sees, the check of every text (all four shipped examples pass it) and the question filter:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-research --lib writer::
   ```
3. Run whole writing runs on stand-in providers: one call, the one rewrite, the writes and their refusals, questions never asked twice, cost and export:
   ```
   cargo test -p psm-research --test engine w2_
   ```

## Choices made for you
- The writer never sees a photo. It reads the fields you let it see, research's note with your decisions, and the titles of matching listings. Say "let the writer see photos" to change it.
- A clean text arrives as a normal suggestion. A text that failed the check twice arrives marked low with what failed, so Accept all leaves it for you. Say "flagged texts should arrive as normal suggestions" to change it.
- The writer rewrites once at most. A run costs one call, or two when the first texts fail the check. Say "allow two rewrites" to change it.
- Automatic writing happens once per session. After that, only "Write texts" or answering a question writes again. A run that failed or was cancelled may run again. Say "write again automatically every time I finish a review" to change it.
- A run that hit the cost limit before its rewrite still writes its texts, marked "not checked again: run cost limit", and counts as that session's automatic writing. Say "a run stopped by the cost limit should not count" to change it.
- Nothing is written while research or Look again is still busy with the session. Run again cancels a writing run that is only waiting.
- "Write texts" goes ahead of waiting research runs, because it is one short call. It needs no photo previews and does respect the day limit.
- Pressing "Write texts" again while texts are being written queues exactly one more run. Pressing it more often adds nothing.
- A question or note is never raised about a field you decided in review, and never asked again once answered or dismissed.
- At most three questions per write. Extra findings become notes, and extra questions are dropped and listed in the run's steps. Say "ask up to five" to change it.
- Questions and notes come only from the writer's first answer. The rewrite only fixes the texts.
- Each writing run replaces the item's earlier open questions and notes. Answered and dismissed ones are kept, so they are never asked again and show up in the counts.
- Answering a question writes the texts again with your answer. A blank answer is refused. Dismissing writes nothing.
- The listing titles the writer sees come from the session's latest research run, Match ones first, at most eight.
- Your earlier answers are given to the writer in the order you answered them.
- The texts you dismissed or kept-previous are given to the writer so it writes something different.
- The writer starts fresh every time and reuses nothing from an earlier writing run.
- The instructions sent are the guide, the title formula, the dimensions line and each example written out in full.
- The settings check also refuses unknown length or question-type names, a non-text effort, a visible-fields value that is not a list, and a blank condition report field.
- A description is suggested again when it differs from one you dismissed only in capital letters. Descriptions are rich text, and research's rule for rich text compares them exactly. Say "case-only changes to a dismissed text should count as dismissed" to change it.
- A missing price names your model provider and the writer's model, for example "No price is set for openai gpt-6.1-sol".

## Not yet
- Nothing in the app runs the writer yet. The next phase adds the "Write texts" button, the automatic start after "Next to review", the writer block in AI review with its questions, and the writer block in the research panel.
- Settings → Writer comes after that. Until then the writer uses the defaults above.
- The two search texts arrive with the search-fields phase, and the restoration choices after it.
- Rough edges to fix before you see texts, recorded for the next phase:
  - If you typed a description yourself, the writer's unused draft could still raise a question about it.
  - A condition sentence that starts "A 1 cm chip…" is read as a model number and flagged.
- No live run with the real model was done. The first real texts come with the next phase.

## Needs you
Nothing.
