# Phase P10_attributes: the product information behind each item

## What you can do now
Each item can now hold its product information: title, description and tags, plus every field the shop will provide once it is connected (wood type, dimensions, condition scratches and so on). Values are stored in one fixed form, so "1.50" and "1.5" count as the same value and saving it again changes nothing. A value that does not fit its field, such as "minor" where the choices are "Minor", "Moderate"…, is kept as received and marked as not fitting. It stays out of the export until you correct it. You can set up a field layout per item type (which fields, in which order, with default values), and a new item of that type starts with those defaults filled in. The damage report selects every condition photo automatically, including ones you shoot later, unless you take one out. Exporting now writes the item's filled-in values into `item.json`. There is no form screen yet; that is the next UI phase.

## Try it yourself
1. Run this phase's checks (no camera or Capture One needed):
   ```
   export PATH="$HOME/.cargo/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p10_attributes
   ```
   The last line should read `test result: ok. 31 passed`.
2. Start the app with a scratch data folder and create a session for a new item, for example SKU `P10-1`:
   ```
   export PATH="$HOME/.nvm/versions/node/v22.22.3/bin:$HOME/.cargo/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   PSM_DATA_DIR=/tmp/psm-p10 npm run tauri -- dev
   ```
3. Right-click in the window, choose **Inspect**, open the Console and run these two lines:
   ```
   await window.__TAURI_INTERNALS__.invoke('attribute_values_set', {itemId: 1, key: 'product.tags', value: '["Oak","Teak","Oak"]'})
   await window.__TAURI_INTERNALS__.invoke('form_get', {itemId: 1})
   ```
   The second answer shows three steps. Product lists Title, Description and Tags. Tags holds `["Oak","Teak"]` (the repeated Oak is gone) and the step counts 1 of 3 filled.
4. In Terminal, look at what was stored:
   ```
   sqlite3 /tmp/psm-p10/session_manager.db 'select key, value, valid, provenance from attribute_values;'
   ```
   It prints `product.tags|["Oak","Teak"]|1|human`.
5. With Capture One open, shoot a photo or two, review them and press **Export**. The `item.json` in the export folder now contains `"attributes": {"product.tags": ["Oak", "Teak"]}`.

## Choices made for you
- Title, Description and Tags are built in. They exist with no shop connected and cannot be removed from a layout. Nothing to change.
- The Title field in the form is not yet the item's title you typed when creating it. A new item named "Oak" shows an empty Title field, and the sessions list keeps showing "Oak". This must be joined up before the form screen ships, and it is noted for that phase. To decide it now, say "the form's Title is the item's title".
- A web link has trailing spaces removed, like plain text. The written rules only mention text, so they will be updated to match. To change this, say "keep spaces at the end of links".
- Numbers drop leading zeros and a minus zero: "007.25" is stored as "7.25", "-0" as "0". Forms like "1e3", ".5" or "+1" do not fit. Nothing to change.
- A dimension accepts any unit when the shop lists none for that field. Ratings are not checked against their minimum and maximum. Nothing to change.
- Tags and multi-choice lists drop repeats but keep your spelling exactly as typed, including how accented letters are encoded. Nothing to change.
- Clearing a field keeps a hidden record that it was cleared. Once you have typed or cleared a field yourself, a future AI assistant can never fill it again. To change this, say "after I clear a field, AI may suggest it again".
- If you type the same value the AI suggested, the field becomes yours. Nothing to change.
- Every edit, AI suggestions included, moves the session to the top of the sessions list. Edits never mark a published session as changed. Nothing to change.
- A field shows as a default only while it still holds the untouched preset value. "Filled" counts fields that hold a fitting value. Nothing to change.
- Without a saved layout, a step shows all its fields: the built-in ones first, then the shop's in the shop's order. Nothing to change.
- Field values travel between the window and the app as text. That is an internal detail. Nothing to change.
- Until the shop is connected, the app offers only Title, Description and Tags. Nothing to change.

## Not yet
- No form screen. You can only see and set values from the developer console as above. The form arrives in the next UI phase.
- No shop fields. The shop's 68 fields come when the Shopify connection is built. They were modelled here for testing only.
- Nothing is sent to the shop.
- No AI assistant runs. Only the places where its suggestions will be stored are ready.

## Needs you
Nothing.
