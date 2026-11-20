# Phase C3_conversation_and_answering: You read a pricer's question and answer it, with photos, without leaving Session Manager

## What you can do now
Click a conversation in the panel and it opens in place of the list. It looks the way it does in Pricing: the pricer's messages are dark bubbles on the left, yours are light bubbles on the right with "You". You also see the highlighted quote with its field, the photos (click one to see it large), the line a message replies to, and "edited". Opening it while the window is in front marks it as read, so the number on the header icon goes down. You answer in the box at the bottom. Enter sends, Shift+Enter starts a new line, and a counter appears near the 4000-character limit. You can reply to one particular message, and the photo button lets you attach up to 10 photos already published for that product. A reply shows at once as "Sending…" and is never sent twice. If it fails it reads "Couldn't send", with **Retry** and **Remove**. On an open question, **Mark answered** tells Pricing the fix is already in the data.

## Try it yourself
1. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   npm run tauri -- dev
   ```
2. In Pricing, ask a question about a product you photographed and published from this Mac. Open the panel from the header icon and click the conversation. The question is a dark bubble on the left with its quote, and the header number goes down.
3. Type an answer and press Enter. It shows at once as "Sending…", then as sent. Pricing shows it, and the conversation reads "Answered". There is no chime for your own reply.
4. Press the photo button, tick two published photos and send. Pricing shows the photos.
5. Turn Wi-Fi off and send a reply: it reads "Couldn't send". Turn Wi-Fi back on and press **Retry**. Pricing shows the message exactly once.
6. On another open question, press **Mark answered**. It reads "Answered" in both apps.
7. To run the automated checks for this phase:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-core --lib mechanisms::conversation_rules
   cargo test -p psm-testkit --test suite c3_conversations_answer::
   cargo test -p psm-desktop
   cd apps/desktop && npx vitest run src/features/conversations
   ```

## Choices made for you
- A reply is saved on this Mac the moment you press send and goes out within a quarter of a second. If the app closes first, it is sent once at the next start.
- A failed reply is never sent again by itself. It waits with "Couldn't send", **Retry** and **Remove**.
- Sometimes a reply arrives but the confirmation is lost. The conversation then shows the delivered message and also "Couldn't send · Retry", until you press Retry (nothing is sent twice) or Remove.
- The photos you can attach are the ones already published for that product: gallery photos in gallery order, then condition photos, and both halves of a split photo. Say "let me attach any photo of the session" to change it.
- **Mark answered** is offered only on an open question. Replying to a resolved one re-opens it as answered, as Pricing does.
- A conversation counts as read when it is on screen while the window is in front.
- A photo opens large inside Session Manager, not in a browser.
- If Pricing refuses the key while replies are going out, every reply still waiting in that batch also shows the refusal and needs Retry.
- **Remove** works even when Pricing is not connected, because it only deletes the unsent reply on this Mac.
- Two replies sent within the same millisecond leave in the order you pressed them.
- When Mark answered finds the question already resolved by the pricer, the "Resolved by …" words appear at the next check, not at once.
- Retry shows the reply as "Sending…" again.
- The box empties as soon as you press send. Say "keep my text until the reply is accepted" to change it.
- **Reply** is offered on every message, yours included.
- The photo button is disabled while replies can't be sent.
- What the fix changed: the automatic check for "Shift+Enter starts a new line" could not notice if Shift+Enter sent the reply instead. It now waits a moment before checking, and it also checks that your text is still in the box. Only that check changed. The app itself was already right.

## Not yet
- Jumping from a conversation to its session (an **Open session** button, the visit bar and Back) and making the quote a link to its field come in the next step.
- Nobody has sent a real reply yet. The automated checks use a stand-in for Pricing, so steps 2–6 above are the first real test. The orchestrator plans a live check against the real Pricing server after the last conversations step.
- Four small points went to the backlog:
  - If a reply is refused, the text you typed is gone. This is rare, because the app checks your text before sending.
  - A reply can stay "Sending…" while the key is refused, until you reconnect. The rule for this still has to be written down.
  - Four of the choices above still have to be added to the conversations rules.
  - One line in the developer guide needs a correction.

## Needs you
Nothing.
