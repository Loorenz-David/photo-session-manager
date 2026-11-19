# Phase W5_seo_publish: search title and search description, written by the writer and published to Shopify

## What you can do now
Every item now has a **Search title** and a **Search description** at the end of its Product step, for every item type, even types whose Product step you arranged earlier. You cannot remove them, but you can move them. For an item linked to a Shopify product, the search texts already in the shop are read into the form as "from Shopify", the same way the description is. **Write texts** now also writes both search texts and checks them like the other texts. A search title over 60 characters is flagged. When you publish, the search texts go to Shopify's "Search engine listing" right after the description, but only when they differ from what the shop already has. A search text you left empty never clears the shop's own. This phase also fixes some rough edges from the earlier writer phases (listed at the end of the next section).

## Try it yourself
1. Start the app on scratch data with the dev store connected:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev
   ```
2. Open a session's Product step. "Search title" and "Search description" are the last two fields. In Customize fields they have no Remove button.
3. Open a session linked to a Shopify product that already has search texts. They appear in the two fields, marked as from Shopify.
4. Press **Write texts**. AI review now also lists the two search texts. A search title longer than 60 characters is flagged.
5. Accept them and publish. In Shopify's admin, open the product. Its "Search engine listing" shows them.
6. Publish again without changes. Nothing is written to the search fields. The app's log has a line `publish.seo item=<n> written=false`:
   ```
   grep "publish.seo" ~/Library/Application\ Support/com.loorenz.sessionmanager/logs/psm.*
   ```
7. Rough edges fixed in this phase:
   - A condition sentence like "A 1 cm chip" no longer counts as a model number.
   - The writer no longer asks about a text it did not write, such as your own description.
   - Answering a question while research is busy now shows why nothing started.
   - Writing costs under a cent show "< $0.01" instead of "$0.00".
   - The "Photos changed" condition banner no longer appears when research fills no Condition field. Look again could not clear it.
8. To run the automated checks for this phase:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-testkit --test suite -- research::w5_seo_publish:: research::w5_carried:: p12_shopify_adapter::
   cd apps/desktop && npx vitest run src/features/form/__tests__/w5_carried.test.tsx
   ```

## Choices made for you
- The two search fields show at the end of every item type's Product step, even where you arranged that step before. You can move them but not remove them, like title, description and tags.
- A layout you saved earlier is not rewritten. The fields are added when the step is shown, with no default value.
- A shop's existing search text is read into the form. If the writer's text would replace it, publishing waits until you choose, the same way it does for the description.
- A search text the writer flags (for example, a title over 60 characters) and that has nothing in the shop to replace is not published until you accept it.
- Publishing writes the search fields only when one differs from the shop's (spaces at the ends are ignored). It never empties one you left blank, because the shop's own text is sent back unchanged.
- After publishing, the app reads the search texts back. If Shopify kept something different, the item fails at "update" and names the field.
- Shopify stores a search title longer than its suggested 70 characters as it is. It neither cuts nor refuses it (tested on your store with a temporary draft product, which was then deleted). So a long title you keep is published as you wrote it, and search engines may cut it. Say "warn me about long search titles in the form" to add a warning.
- To check this, the app created one draft test product on your real store ("PSM-PROBE (test, do not publish)"), wrote and read its search texts, and deleted it. It touched no other product.
- The publish log line names the publish item number, as the other publish lines do. It is written only when there was a search text to publish.
- The message after an answer that started nothing is the app's own sentence, shown as an error toast.
- Cost display stays in the writer panel only. No other screen shows money yet.
- Shopify's shop name stays in one place in the code, and the writer reads it there.

## Not yet
- Restoration as fixed choices (the next phase).
- Google Merchant Center fields are not written. That was set aside in the book.
- No live publish of search texts was made in this phase's checks. The Shopify replies were captured once on your store and are replayed in tests. Your first real check is steps 3–6 above.

## Needs you
Nothing.
