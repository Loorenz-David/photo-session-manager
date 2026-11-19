# Phase W3_writer_review: written texts arrive in AI review

## What you can do now
When you finish reviewing a session and press "Next to review", the writer drafts that session's title, description and condition report in the background. It also does this when you close Details while AI review says "All checked". You never wait for it, and it never interrupts you. The texts arrive in AI review like research's suggestions, each with its reason, the facts it used and the previous text. A long text is shown as paragraphs, and a text that failed the check is marked low with a "Check:" line. Questions and notes from the writer sit above the suggestions: answer a question and the texts are written again, or dismiss a note and it never comes back. The Product step has a **Write texts** button for writing again whenever you like. A suggested title only changes the item's title (and so the shop's title) when you accept it, and undo brings the old title back. The research panel now has a Writer section that shows what the writer used, why it wrote each text, what the check found and what it cost.

## Try it yourself
1. Start the app on scratch data with your research keys set and fields chosen in Settings → Research:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev
   ```
2. Let research finish on a session. Open Details → AI review, check every suggestion, and press **Next to review →**. The next session opens straight away.
3. After a minute, go back to the first session and open AI review. You should see a title, a description and a condition report, each with why it was written and which facts it used. A question may sit above them.
4. Type an answer to the question and press **Answer**. The texts are written again within about a minute.
5. Accept the title. The item's title changes. Press ⌘Z and the old title comes back.
6. In the Product step, press **Write texts**. New texts arrive, and a text you dismissed does not come back word for word. If research is still running on the session, a message says so instead.
7. Open the research panel. The Writer section shows the number of fields, observations and listing titles it used, its reasons, any check it failed, whether it rewrote once, and the cost.
8. To run the automated checks for this phase:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-testkit --test suite research::w3_
   cd apps/desktop && npx vitest run src/features/form/__tests__/w3_writer_review.test.tsx src/features/form/__tests__/w3_writer_panel.test.tsx src/features/sessions/__tests__/w3_writer_triggers.test.tsx
   ```

## Choices made for you
- A suggested title becomes the item's title only when you accept it. Until then the shop keeps the old title, even if you publish. Say "publish a suggested title without accepting it" to change it.
- A writer description that replaces the shop's description counts as a conflict at publish, like research's values: the item waits until you choose. Say "let writer descriptions publish over the shop's text" to change it.
- The writer starts by itself once per session: when you press "Next to review", or when you close Details while AI review says "All checked". After that only **Write texts** or answering a question writes again. Say "write again automatically every time I finish a review" to change it.
- "Leaving the session" means closing the Details window. AI review only exists inside Details, so leaving the session page with Details closed does not start the writer. Say "also start the writer when I leave the session page" to change it.
- "All checked" means no suggestion is left, the item has at least one AI value, and research is not running.
- Answering a question rewrites the texts from the fields alone, without photos, in one or two calls. The answer is kept for the item.
- A dismissed note never comes back.
- Dismiss on a suggested title for an item with no title only removes the suggestion. Clearing a title you already accepted empties the title and the suggestion together, in one undo step.
- "Accept all" still skips a suggested title that would replace an existing title, as it does for any value with a previous value. You accept that one yourself. Say "Accept all should take the title too" to change it.
- When you type over a suggested title, the suggestion goes away in one undo step, and further typing joins that same step.
- Undo or redo of a title accept is refused with "that title was changed" if the title has changed since. A plain title edit undoes as before.
- On the writer's texts, Accept does the same as "Use suggestion". Research's own conflicts keep their "Use suggestion" button.
- Asking the writer with an unknown reason is refused. The engine's internal "answer" reason cannot be sent from the window.
- The cost is shown in dollars with two decimals, for example "$0.02". Say "show costs to the tenth of a cent" to change it.
- The **Write texts** button sits at the top of the Product step with one line explaining what it does. When it queues, a message says "Writing texts. They arrive in AI review."
- Until Settings → Writer exists, the writer uses its shipped defaults. A writer setting typed by hand into the settings file is honoured.

## Not yet
- Settings → Writer (language, lengths, question types, the automatic start) comes in the next phase.
- The two search texts arrive with the search-fields phase, and the restoration choices after that.
- If you answer a question while research is still busy on the session, the answer is saved but the texts are not rewritten yet, and nothing tells you so. A message will be added with Settings → Writer.
- A writing run that costs less than half a cent shows "$0.00".
- Two rough edges from the previous phase remain for the planner. If you typed a description yourself, the writer's unused draft could still raise a question about it. A condition sentence that starts "A 1 cm chip…" is read as a model number and flagged.
- No live run with the real model was made in this phase's checks. Your first real texts come from the steps above.

## Needs you
Nothing.
