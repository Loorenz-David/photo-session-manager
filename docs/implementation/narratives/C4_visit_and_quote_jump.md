# Phase C4_visit_and_quote_jump: Look at the product a question is about, then go back to your shoot

## What you can do now
When a conversation in the panel belongs to a session other than the one on screen, its header line shows **Open session**. Pressing it opens that session as a visit. You see the product and the panel stays open on the question, but the camera's photos keep going to the session you were shooting. A slim bar under the header says so, for example "Visiting BEYO-7: new photos still go to BEYO-3.". It offers **← Back to BEYO-3**, which returns you to where you were before the first jump, and **Shoot here instead**, which points the camera at the visited session and removes the bar. In the session on screen, a quote's field chip (for example CONDITION or DESIGNER) is now a link when your form has that field. Clicking it opens the product form on the right step, scrolled to that field.

## Try it yourself
1. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   npm run tauri -- dev
   ```
2. Open session A and keep shooting. In Pricing, ask a question about product B (also photographed on this Mac).
3. Open the conversations panel and press "1 new in other sessions → All conversations". Open the question and press **Open session**.
4. Session B opens, and the panel still shows the question. The bar reads "Visiting B: new photos still go to A." Take a photo: it lands in A.
5. Press **← Back to A**: you are in A again and the bar is gone. Jump to B once more and press **Shoot here instead**. The bar goes away and the next photo lands in B.
6. In B, open a question that quotes the Condition text and click the CONDITION chip. The form opens on the Condition step, scrolled to that field.
7. Optional: in Settings, change something without saving. Then open a conversation of another session and press **Open session**. "Save your changes?" appears. Discard opens the session with the bar, and photos still go where they went before.
8. To run the automated checks for this phase:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-integrations --lib conversations::
   cargo test -p psm-testkit --test suite c4_conversations_visit::
   cd apps/desktop && npx vitest run src/features/conversations src/features/form
   ```

## Choices made for you
- A visit never moves the camera's folder. New photos keep going to the session you were shooting, or to Unassigned if you came from the Sessions page or Settings, until you press "Shoot here instead".
- Back returns to where you were before the first jump, also after you jump through several conversations.
- From Settings with unsaved changes, Open session first asks "Save your changes?". Discard or Save then opens the session as a visit, and Cancel keeps you in Settings.
- Opening the session you are shooting from a conversation ends the visit, exactly like Back.
- Any ordinary way of moving around ends the visit and behaves as it always has: the Sessions back link, the Sessions list, the notifications' Open, review navigation, the back keys. Opening a session that way points the camera at it.
- A visit does not survive closing the app. At the next start you land in the session the camera points at, as before.
- The bar asks the app which session the camera really points at, so a disagreement would show. Until that answer arrives, the name is blank for a moment. Unassigned is called "Unassigned".
- The bar appears only on a session's page, because a visit is always a session.
- The field chip is a link only for fields this Mac's form has for that product, and only while that product's session is on screen. Otherwise it is plain text.
- Every click on the chip jumps, also a second click on the same chip. If that field holds an AI suggestion, its suggestion popover opens too.
- A chip jump wins over the "open on AI review" rule: the form opens at the field even when suggestions wait. A field the form doesn't have opens the form as usual.
- For a quote naming a custom field like "mf:a.b.c", everything after the first dot counts as the field's name.
- The bar is a light amber strip that stays under the header. The chip link is underlined and says "Open the form at this field" on hover.

To change any of these, say which one and how, for example: "Back should go to the last session I visited, not the first".

## Not yet
- Nobody has tried this on the real app with the camera. The automated checks use stand-ins for Pricing, Capture One and the camera, so steps 2–6 above are the first real test.
- The app does not remember more of a session's screen than it did before. Back returns to that session the way the app normally reopens it.
- Three small points went to the backlog:
  - If the app can't tell which session the camera points at, the bar shows a blank name instead of saying so.
  - The number that identifies Unassigned is now written in one more place in the code.
  - One automatic check doesn't cover one safety line in the form. The behaviour itself is checked.

## Needs you
Nothing.
