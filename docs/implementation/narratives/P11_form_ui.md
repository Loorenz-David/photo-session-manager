# Phase P11_form_ui: the product details form

## What you can do now
In a session, press `D` (or the **Details** button on the stage) and the product details form opens over the workspace. It has three steps, Details, Condition and Product, and each step shows how many of its fields are filled. Every field type the shop can send has its own input: severity scales, pill choices, dropdowns, dimensions with units, dates, colours, ratings, tags, and a picker that chooses which condition photos go into the damage report. Your typing is saved on its own a moment after you stop. A stored value that does not fit its field is shown as typed, with an "Invalid" marker, until you correct it. With **Customize fields** you can reorder, remove and add fields; Title, Description and Tags always stay. The Title in the form is the item's title: change it there and the sessions list, the workspace and the export all show the new title.

## Try it yourself
1. Run this phase's checks (no camera or Capture One needed):
   ```
   export PATH="$HOME/.nvm/versions/node/v22.22.3/bin:$HOME/.cargo/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p11_title_binding
   cd apps/desktop && npx vitest run src/features/form
   ```
   The first should end with `test result: ok. 1 passed`, the second with all form tests passed.
2. Start the app with a scratch data folder:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   PSM_DATA_DIR=/tmp/psm-p11 npm run tauri -- dev
   ```
3. Create a session for a new item, for example SKU `P11-1` with title `Oak`. Open it and press `D`.
4. Go to the **Product** step. The Title field already says `Oak`. Change it to `Oak sideboard` and click into Description. "Saved" appears. Press **Save & close**: the workspace's item card, if you have it open, shows `Oak sideboard`. Press `D` again to reopen the form.
5. Type some tags and a description, then press **Save & close**. Reopen with `D`: everything is still there, and the Product step counts the filled fields.
6. Click **Customize fields**, drag Tags above Title, then press **Done**. Close and reopen the form: the new order stays. Title, Description and Tags have no Remove control.
7. Go back to the sessions list: the session shows the new title.

## Choices made for you
- The Title field is the item's title, the one you typed when you created the item, so there is only ever one title. Emptying the field leaves the item without a title. Nothing to change.
- Only you can change the title from the form. A field layout cannot give it a default value, and a future AI assistant cannot write it. To change this, say "let the AI suggest a title".
- A field is saved 300 ms after you stop typing, and emptying a field clears it. To change the timing, say "save after N milliseconds".
- Numbers are sent as you typed them. If an entry does not fit, for example `12,5` with a comma, it is marked "Invalid". The box then empties and your text survives only in the marker. Nothing to change.
- The description opens in "AI agent writes it" mode unless one is already stored. Until the agent exists that mode writes nothing, so pick "Write manually" to type one. To change this, say "open the description in manual mode".
- When the shop does not say which unit, currency or scale a field uses, the form offers cm/mm/in and kg/lb, SEK, and a 0 to 5 rating. To change these, say which defaults you want.
- Dates use the Mac's built-in date picker. Nothing to change.
- Thumbnails in the form show the edited preview, or the camera original if there is no edited one. Nothing to change.
- Photos you have not reviewed yet are listed after the main photos, because they publish with them. Nothing to change.
- Fields only the shop can edit show "Edited in Shopify admin". Nothing to change.
- A Customize layout is saved when you press Done, change step or close the form, not on every drag. Nothing to change.
- The form opens from a "Details" button on the stage, not from the top bar of the app. To move it, say "put Details in the header".
- Items you create in the app have no item type yet, so one Customize layout is shared by all of them. This changes once item types can be set. Nothing to change now.

## Not yet
- Details and Condition steps are empty until the shop is connected; then the shop's fields appear there.
- No AI assistant fills fields. The AI suggestion badges and their "Looks right" / "Clear" actions are ready but nothing produces suggestions yet.
- The barcode can be corrected from the form header but not removed.
- Nothing is sent to the shop.
- If you close the form with Escape right after changing the Title, the item card in the workspace shows the old title until the workspace refreshes. The new title is saved.

## Needs you
Nothing.
