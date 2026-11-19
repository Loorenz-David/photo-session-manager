# Phase W4_writer_settings: you can now tune the writer in Settings

## What you can do now
Settings has a new **Writer** page, right after Research. On it you choose the writer's model and effort, the language it writes in (English or Swedish), and whether it starts by itself when you press "Next to review". You can edit its instruction. **Preview what is sent** shows exactly what would go to the model for a built-in example chair; nothing is sent and it costs nothing. Every time you change the instruction, the old text is kept: the last 20 per language are listed with their date, and **Reset to default** brings back the shipped instruction. You also pick which fields the writer may read, edit the style examples for each language, add banned words, change the target lengths, set each kind of doubt to off, note or ask (with how often you answered or dismissed each), set how many questions it may ask per write, and type the shop name used in search titles.

## Try it yourself
1. Start the app on scratch data with your research keys and prices set:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev
   ```
2. Open Settings → **Writer**. Add a line to the instruction, for example "Keep descriptions short.", and press **Save writer settings**.
3. Press **Preview what is sent**. The window shows your instruction first, then the example chair's fields as the input, then the answer format.
4. In a session press **Write texts** (Product step). The new texts follow your instruction.
5. Back in Settings → Writer, **Earlier versions** lists the shipped text with today's date. Press **Reset to default**: the shipped text is back, and your text joins the list. **Use this** on a version puts it back in the editor; it is kept only when you press Save.
6. Set **Writing language** to Swedish, save, and press **Write texts** again: the texts arrive in Swedish.
7. Set **Field vs field** to off, untick a field under **Fields the writer may see**, save, and write again. The research panel's Writer section counts one field fewer.
8. To run the automated checks for this phase:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-desktop w4_tests
   cd apps/desktop && npx vitest run src/features/settings/__tests__/w4_writer_settings.test.tsx
   ```

## Choices made for you
- The Writer page saves only when you press Save, like the Research page, and the writer uses the saved values at its next run.
- **Preview what is sent** shows the saved settings, so an edit shows in the preview only after you save. Say "preview my unsaved changes" to change it.
- The preview uses a built-in example chair (the fields of shipped example A) and asks for every text, so it needs no item and costs nothing.
- Every save that changes an instruction keeps the text it replaced; the last 20 per language are kept.
- **Reset to default** saves at once and only changes the instruction of the language shown. Text you typed since the last save in that same editor is lost; other unsaved changes on the page stay. Say "ask before Reset throws away my typing" to change it.
- If you save an instruction or an example list that is exactly the shipped one, it is stored as "shipped", so later improvements to the shipped text reach you.
- Only models with a price in Settings → Research are offered. A saved model that loses its price stays chosen, marked "(no price)", so a save never swaps it.
- Effort is typed as text, as on the Research page.
- Extra banned words are entered one per line.
- The style examples of both languages are shown at once; the instruction editor shows the chosen language only.
- With no list saved, the fields the writer may see are all Details and Condition fields except prices, media and references. Ticking or unticking one saves your own list from then on.
- The Writer page appears only when the Research page does, since the writer needs research's prices and keys.
- The shop name is typed once here; it is added to a search title only when the title still fits.

## Not yet
- The two search texts (search title and search description) and their publishing come in the next phase; the shop name and the search-length settings are used then.
- Restoration as fixed choices comes after that.
- Three small points from the previous phase are still open: answering a question while research is busy saves the answer but gives no message; a writing run under half a cent shows "$0.00"; one line in the developer guide is out of date.
- No live run with the real model was made in this phase's checks. Your first real check is steps 2–7 above.

## Needs you
Nothing.
