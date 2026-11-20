# 09B — Conversations with Pricing (questions from the pricer, answered in the session)

**Status: RATIFIED** (owner David Loorenz, 2026-11-20: "perfect, let's implement it"; chapter 16 entry A27).
Shaper Claude. Branch `feature/agentic`.
Source of the external contract: the Pricing integration package
`~/Desktop/Developer/pricing-app/docs/integrations/session-manager/` (README, API, EVENTS_AND_WEBHOOKS, CHAT_UX,
`openapi-integration.json`; Pricing owner, verified live 2026-11-20). That package is the authority on Pricing's
API; this chapter is the authority on what Session Manager does with it. Where this chapter quotes the API it
names the package file.

## 1. Outcome

Beyo Pricing is the owner's second application, where a **pricer** prices one product at a time from what Session
Manager published to Shopify. While pricing, the pricer asks the photographer questions about a product ("Is the
glass original?", "Send a close-up of the dent"). Pricing owns those questions. Session Manager shows them to the
photographer and lets the photographer answer:

- a **chat icon in the header** with a count, like social apps: on the Sessions page it opens every conversation
  that is unresolved or has news; inside a session it opens only that session's conversations;
- a conversation view that looks like Pricing's (dark received bubbles, light sent ones, the highlighted "about
  this field" quote, photos, replies);
- **jumping** from a conversation to the session it belongs to and **coming back** to where the photographer was,
  without moving the camera's capture folder;
- a **sound** (mutable) and a **Mac banner** when a question or a pricer reply arrives.

## 2. Owner decisions (2026-11-20)

1. Build it as planned ("perfect, let's implement it").
2. **Store:** the Pricing service account is already tied to the **real shop** Session Manager publishes to (the
   package README still says "Dev store"; that is outdated). "If I change to dev then there is no match."
3. **Jumping visits:** opening a session from a conversation only visits it; new photos keep going to the session
   being shot, until the photographer chooses "Shoot here instead" (§7).
4. **Mac banners when Session Manager is in the background: on by default.**
5. The description switch ("AI agent writes it / Write manually") is retired (A26); unrelated to this chapter but
   decided the same day.

## 3. Vocabulary and boundaries

- **Conversation** = one Pricing *case*: a question thread about one product. It has a `topic` (Identification,
  Dimensions, Condition, Product details, Other), a `blocking` flag and a `status`: **open** (waiting for the
  photographer) → **answered** (the photographer replied) → **resolved** (the pricer closed it, or activated the
  product).
- **Message**: author (the pricer, or "me" = Session Manager's service account), text, optional **quote** (the
  text the pricer highlighted in a product field, with the field), optional **reply-to**, optional **photos**,
  `editedAt`.
- **Product = session.** A conversation belongs to the session whose item has the conversation's **SKU** (exact
  match) and that is not in the Trash. No such session on this Mac → the conversation still shows, marked "Not on
  this Mac" (the product may have been photographed on the other Mac).
- **Only photographer conversations** (`recipient = photographer`). Conversations addressed to Operations are
  never stored or shown.
- Architecture (01, 11): core gets a platform-neutral **conversations provider** port (list a product's
  conversations, read one, post a message, mark answered, read events after a cursor, list stores) and the
  core vocabulary *conversation / message*. The Beyo Pricing adapter lives in `integrations`. No Pricing names in
  core. Tauri commands stay thin.

## 4. Connection and settings

**Settings → Pricing** (a new section):
- **Server address**, prefilled `https://pricing.beyoworkaroundtheclock.com` (editable; https only).
- **API key**, pasted by the owner (Pricing → Settings → Service accounts issues it, scopes `cases:read`,
  `cases:write`, `events:read`). Stored only in the macOS Keychain through the existing `SecretStore` port, like
  the research key. Never written to the settings file, never in a settings export, never logged or shown again
  (the field shows "Saved" with Replace / Remove).
- **Test connection**: lists the account's stores and picks the one whose `shopDomain` equals the connected
  Shopify shop's domain (case-insensitive). Result shown in words: "Connected to <store name> (<shop>)". No match:
  "This Pricing key isn't tied to <shop>. It can see: <names>." and conversations stay off. No Shopify connection:
  "Connect Shopify first; conversations are matched to its products." The chosen store id is saved.
- **Sound for new questions and replies**: on by default. The mute button in the conversations panel is the same
  setting.
- **Mac banner when Session Manager is in the background**: on by default (owner decision 4).
- **Last checked**: the time of the last successful check, or the last error in words.

Without a key, or with the store unmatched, Session Manager makes **no** Pricing calls; the header icon stays and
its panel says "Connect Pricing in Settings → Pricing".

## 5. Keeping up to date (contract MC-C1)

Pricing offers webhooks, but only to a **public https address**; a desktop app has none, so Session Manager uses
Pricing's **event pull** only (EVENTS_AND_WEBHOOKS "Catch-up by pull", which the package calls the source of truth).
Webhook registration is not used.

1. **Local copy.** Session Manager keeps a local copy of each photographer conversation with all its messages,
   the saved event cursor, and per-conversation **read marks**. The copy is what the UI reads; screens never
   wait on the network.
2. **First connection** (no saved cursor), silently (no sound, no banner, no toast):
   (a) page the event log from `after=0` to its end (limit 500) only to reach the newest cursor; (b) list the
   store's unresolved photographer conversations (`GET …/cases?recipient=photographer&status=unresolved`) into the
   copy, and fetch any photographer conversation the paged events named that is not yet in it; (c) save the cursor.
   Paging first means a change made while listing is caught by the next check (planning, 2026-11-20).
   Read marks after the first connection: an **open** conversation counts as unread (it needs an answer); answered
   and resolved ones count as read.
3. **Every 10 seconds** while the app runs (focused or not), and at once on launch, on waking, and after the key
   or store changes: `GET …/events?after=<cursor>&limit=500`, repeated until a page is empty. For each event in
   order:
   - skip an event id already applied (at-least-once);
   - `data.recipient != photographer` → advance the cursor, nothing else;
   - otherwise re-read that conversation (`GET …/cases/{caseId}`) and replace the local copy of it and its
     messages; `case.updated` with `deleted: true`, or a 404 on the re-read, removes it from the copy;
   - the cursor advances in the same database transaction as the change it follows.
4. **Errors**: network or 5xx → keep the copy, retry with backoff (10 s, 20 s, 40 s … at most 5 min), back to 10 s
   after a success. 401/403 → stop checking, Settings and the panel say "Pricing refused the key: check Settings →
   Pricing"; checking resumes when the key is replaced. 404 on the store → "This key is no longer tied to <shop>".
   Never a crash, never a blocked screen.
5. **What notifies** (not during the first connection): `case.created`, and `message.created` with
   `authorKind = user`, for a photographer conversation. Several in one check make **one** sound and one toast per
   conversation (at most three toasts; then "and N more"). Own replies never notify.

## 6. Unread, counts and badges

- A message is **unread** when its author is the pricer and it is newer than its conversation's read mark.
  Opening a conversation on screen while the window is focused moves its read mark to its newest message. Read
  marks are per Mac.
- A conversation **has news** when it has an unread message.
- **Header count** = conversations with news: on the Sessions page (and Settings, Health) across all sessions;
  inside a session only that session's, plus a small dot on the icon when other sessions have news.
- **Sessions list**: a session with unresolved conversations shows a small badge "N open"; amber "Pricing is
  waiting" when one of them is blocking. A blocking conversation stays blocking until the pricer resolves it,
  even after the photographer answered (CHAT_UX §7 rule 2).

## 7. The conversations panel and the jump (contract MC-C3)

**Panel**: the header icon opens a right-side panel.
- **List**: tabs **Open** (unresolved) and **Resolved** (resolved in the last 30 days). Rows sorted: news first,
  then blocking, then latest activity. A row shows the SKU and product title, the topic, a status dot, "Pricing is
  waiting" (amber) for blocking unresolved ones, the last message's first line, its time, and an unread dot.
  Inside a session the list shows that session's conversations and a footer "N new in other sessions → All
  conversations", which switches the list to all.
- **Conversation**: a back arrow to the list; a header line with the status dot, topic, "· Blocking", the status
  in words, and **Open session** (when it belongs to a session other than the one on screen), or "Not on this
  Mac". Messages as in Pricing, mirrored to the viewer (CHAT_UX §2): the pricer's messages in **dark bubbles on the
  left** with name and time, the photographer's own in **light bubbles on the right** ("You"); "· edited" after the
  time when edited; photos as rounded thumbnails above the text (click opens the large image); the reply-to line
  ("↩ Elin: …"); and the **quote** as its own block (accent bar, contrasting fill, uppercase field chip, the
  quoted text in “…”). Missing information reads "not recorded", never 0 or invented.
- **Quote chip jump**: when the conversation's session is the one on screen, the chip opens the form at that
  field. Field mapping: `title` → `product.title`, `description` → `product.description`, `condition` →
  `custom.condition_report_description`, `mf:<namespace>.<key>` → the definition `<namespace>.<key>` if this Mac
  has it. Anything else: the chip is plain text.

**The jump and the way back** (owner decision 3):
1. **Open session** goes to that session as a **visit**: the session view opens, but the capture folder is **not**
   moved (no `session_activate`). The panel stays open on the same conversation.
2. While visiting, a slim bar under the header says: "Visiting <SKU>: new photos still go to <capture session SKU,
   or Unassigned>." with **← Back to <origin>** and **Shoot here instead**.
3. **← Back** returns to where the photographer was before the first jump (that session, or the Sessions page),
   as it was (the same step/tab where the app already remembers it). Jumping again from a visited session keeps
   the original origin.
4. **Shoot here instead** activates the visited session exactly like opening it normally (capture folder moves),
   and the bar goes away.
5. Any normal navigation (opening a session from the Sessions list, the Sessions breadcrumb, review navigation)
   ends the visit; normal navigation keeps today's capture-folder behaviour unchanged.

## 8. Answering (contract MC-C2)

- **Composer** at the bottom of a conversation: a text box (1–4000 characters after trimming), **Enter sends**,
  Shift+Enter is a new line; **Reply** on a message sets the reply-to (shown above the box with ✕); a **photo
  button** offers the photos of *this* conversation's session that already have a Shopify media id (published
  gallery photos and published condition photos), at most 10; photos without one are not offered ("Publish first
  to attach photos"). The composer needs the conversation's session on this Mac only for photos; text replies
  work for "Not on this Mac" conversations too.
- **Sending feels instant and never lies** (CHAT_UX §5): the message shows at once, dimmed, "Sending…". Each send
  gets a UUID `clientMutationId` when the photographer presses send, stored with the pending message, so a retry
  (including after a restart) cannot duplicate it. Success → the message becomes the server's message (and the
  echoed event is recognised by its id). Failure → it stays with "Couldn't send · Retry" (same id), or
  **Remove** (deletes the local pending message, sends nothing; for a reply Pricing refuses for good). A pending
  message survives a restart.
- **Mark answered** (header of an open conversation): `PATCH status=answered` without a message, for when the fix
  was made in the data. 409 `case_resolved` → refresh and say "Pricing has already resolved this".
- **Resolved conversations**: the panel says "Resolved by the pricer" (or "Resolved when the product was
  activated"); the composer stays, with the note "Replying re-opens it as answered" (API.md, POST messages).
- Session Manager **never resolves**, never sets topic or blocking, never starts a question, never edits or
  deletes messages (not in the API).

## 9. Notifications

- **Toast**: "New question · <SKU> — <topic>" or "New reply · <SKU>", with **Open** (opens the panel on that
  conversation).
- **Sound**: one short, soft chime built into the app (no third-party sound file), when the sound setting is on.
- **Mac banner** (when on and the window is not focused): same words; clicking it brings Session Manager forward
  (macOS does not tell the app which banner was clicked, so the toast with **Open** stays until the window is
  focused again). macOS asks once to allow banners; the chime is played by the app, not the page, so it also
  sounds right after launch.
- Nothing notifies for the photographer's own replies or during the first connection.

## 10. Rules that must never break

1. Session Manager writes to Pricing only the photographer's replies and "Mark answered". Nothing else, ever.
2. The API key lives only in the Keychain; it is never printed, logged, exported or sent anywhere but the
   configured Pricing server.
3. Deleting or trashing a session never touches Pricing, and never deletes conversations from the local copy
   (they are keyed by SKU).
4. Opening a conversation never moves the capture folder; only normal opening or "Shoot here instead" does.
5. Pricing being down or the key missing never blocks or slows any other screen.
6. No duplicate replies: every send carries its `clientMutationId`; every retry reuses it.

## 11. Out of scope (this version)

Webhooks; asking a new question from Session Manager; editing or deleting the photographer's own messages; typing
indicators or read receipts back to Pricing; Operations conversations; read marks shared between the two Macs.

## 12. Acceptance scenarios

- **CV-1** The pricer asks a blocking Condition question with a quote on a product photographed on this Mac.
  Within about 10 s: one sound, a toast "New question · <SKU> — Condition", the header count goes up, the
  session's row shows "Pricing is waiting". Opening it shows the quote block; the count goes down.
- **CV-2** While in session A, a question arrives for session B. The icon in A shows a dot; the panel footer says
  "1 new in other sessions". The photographer opens it, presses Open session: B opens as a visit, the bar says new
  photos still go to A; a capture lands in A; ← Back returns to A.
- **CV-3** The photographer replies with text and two published photos; the reply shows "Sending…" then sent; the
  conversation becomes answered in Pricing; no sound for the own reply; the echoed event creates no duplicate.
- **CV-4** The network drops during a send: "Couldn't send · Retry"; after Retry the pricer sees exactly one
  message.
- **CV-5** Session Manager was closed while the pricer asked two questions and resolved a third: on launch the
  copy catches up, one sound, the resolved one moves to Resolved.
- **CV-6** Wrong key: Settings and the panel say Pricing refused the key; all other screens work normally.
- **CV-7** A question for a SKU not on this Mac shows "Not on this Mac"; a text reply still works.
- **CV-8** Mute on: no sound; toast and banner still follow their own settings.

## 13. Milestones (for the planner)

- **C1 Connection and sync**: port + adapter, Settings → Pricing (key in Keychain, store match, test), local copy,
  first connection, 10-second pull, read marks, error states. Tested against a fake Pricing server.
- **C2 Header icons, panel list, counts, notifications**: header icon and counts, panel list and filters,
  sessions-list badges, sound, toast, Mac banner, mute.
- **C3 Conversation view and answering**: thread rendering (quotes, photos, replies, edited), composer, photo
  picker, pending/failed/retry with `clientMutationId`, Mark answered.
- **C4 Jump and back**: visit mode, the bar, Back, Shoot here instead, quote chip jump.

The live check against the real Pricing server is the orchestrator's, after C4, with a test question the owner
creates (writes to Pricing are outward-facing).
