---
plan: conversations
role: plan
round: 1
state: DONE
date: 2026-11-20
actor: Claude (pipeline-planner, implementation-planner doctrine; orchestrated)
---
# Conversations with Pricing — plans for C1–C4 (book 09B)

**Summary.** A short master plan (`master_plan_conversations.md`) with the contracts that stand in for the mechanism
inventory (MC-C1 sync, MC-C2 send, MC-C3 visit, §5), four milestone plans in the units format of flow §10, and the
phase list. Gate: 09B's status header reads **RATIFIED** (owner, 2026-11-20, changelog A27). The mechanism-inventory
gate is replaced by §5 at the orchestrator's instruction (recorded as a process delta, as for the writer). Every plan
was grounded in the code at `78869e5`: every cited existing path resolves (checked by script; the only unresolved
paths are the files these milestones create). The Pricing shapes were read from the package's `API.md`,
`EVENTS_AND_WEBHOOKS.md`, `CHAT_UX.md` and `openapi-integration.json` (read-only; nothing was called).

Counts (computed from the plan files by script): 4 milestones, 12 units, 73 criteria, 293 rows; every unit ≤ 8 criteria.
| Milestone | Units | Criteria | Rows | App files | Projection |
|---|---|---|---|---|---|
| C1_connection_and_sync | 4 | 21 | 110 | yes | **yes** |
| C2_header_panel_notifications | 3 | 19 | 66 | yes | no |
| C3_conversation_and_answering | 3 | 21 | 73 | yes | **yes** |
| C4_visit_and_quote_jump | 2 | 12 | 44 | yes | **yes** |

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner. 09B determines every behaviour the plans build; the points below are technical choices, each
carried in plain words by the plans' "Points recorded by planning" for the narratives.

## Write perimeter (this session)
Documents only, all new: `docs/implementation/master_plan_conversations.md`,
`docs/implementation/phases_conversations.txt`,
`docs/implementation/plans/{C1_connection_and_sync.md, C2_header_panel_notifications.md,
C3_conversation_and_answering.md, C4_visit_and_quote_jump.md}`, this handoff. No code, no migration, no tool-recorded
state (the repo has no architecture graph). Not touched: `STATUS.md`, `STATUS_conversations.md` (the orchestrator's,
untracked when this session started), book 09B, the Pricing repository, the sibling checkout.

## Points the orchestrator should know
1. **No `implementer:` override.** The task offered Fable for frontend milestones under the 2026-11-04 rule in the
   runner; `docs/process/flow.md` log v1.3 (2026-11-08: "no new plan may carry `implementer: fable`") and
   `docs/guide/03_rules_and_principles.md` are later and are the process authority, so no plan sets it. To use Fable
   the owner's rule has to change first; then add `implementer: fable` to C2 and C4 (and C3 if wanted).
2. **First-connection order differs from 09B §5.2's lettering, not from its calls.** 09B lists (a) unresolved list,
   (b) page the events, (c) save the cursor. MC-C1.5 pages the events first, then lists, then fetches the named
   conversations, then saves the cursor: a list taken first could miss a change made while the log is being paged,
   and that conversation would stay stale until its next event. Same requests, same result otherwise. If you want the
   book to say it, it is a one-line edit of §5.2.
3. **"Repeated until a page is empty" is taken literally** (one extra empty request per check with news), because the
   package does not promise that a short page means the end.
4. **Things the package's schema shows that 09B and `API.md` do not mention, all handled by contract:** the event log
   also carries `workflow.updated`, `finalization.updated` and `inbox.updated` events with no case and no recipient
   (skipped, cursor advances); a case's `sku` can be null ("Not on this Mac"); a message's `text` can be null and it
   has a `deleted` flag (not stored); `nextCursor` stays unchanged on an empty page; errors can arrive as 502 with a
   non-JSON body. For the Pricing owner, not for us: `API.md` writes the query `productId`, the OpenAPI file
   `product_id` (we use neither).
5. **A failed reply can be removed, not only retried** (09B §8 names Retry only). Without it a reply Pricing refuses
   for good (for example a photo it does not accept) would stay in the conversation forever. It deletes a local row
   and sends nothing. Say so if the owner should not have it.
6. **Lost confirmation.** If Pricing stored a reply but the answer was lost, the conversation shows the delivered
   message (from the next check) and still the "Couldn't send · Retry" line until Retry confirms it. Pricing's
   messages do not carry the `clientMutationId`, so the two cannot be matched without guessing; nothing is ever sent
   twice (MC-C2.5). The live check should try this once (send with Wi-Fi cut just after pressing Enter).
7. **Sound and banner are native, not in the page** (MC-C1.9, C2·U3): a page may not play sound before a click, so a
   chime at launch (CV-5) would be silent. The chime is a tone the app generates (no sound file in the repo) played
   with macOS's `afplay`; banners use `tauri-plugin-notification` from Rust (a new dependency, C2). The plugin does
   not report which banner was clicked on macOS as far as planning could tell without installing it; a click brings
   the app forward (macOS does that) and the toast with **Open** stays until the window is focused again. C2 asks the
   implementer to record what the installed version offers.
8. **macOS permission for banners** is asked by the system at the first banner; a refusal is invisible to the app.
   Settings → Pricing gets one hint line. Under `tauri dev` the request is attributed to the terminal, so the banner
   can only be judged on the installed app.
9. **No call to Pricing can come from a session.** Besides the fake at the HTTP seam: a data folder that was never
   tested makes no call even when the Keychain holds a key (MC-C1.2), and the master plan forbids sessions to launch
   the built app, because the Keychain is shared by every build of the app.
10. **A numbered migration (`019_conversations.sql`)**, not the research crate's `ensure` step: these are core tables.
    If something else takes 019 first, the next number is used.
11. **09B §3 "not in the Trash":** there is no session Trash today ("Delete session" removes the session); recorded
    as structurally held, with its trigger, in master plan §6.
12. **09B §6 "Settings, Health":** those pages do not render the app header today; C2 puts the icon in their page
    header and mounts the panel once for all routes.
13. **Banner while unfocused plus mute:** the mute button is the sound setting only; toast and banner are untouched by
    it (CV-8).

## Risks
- **C4 is the one that can put photos in the wrong product.** The capture folder follows a React effect keyed on the
  session id; a visit works only if route and visit change in one step (MC-C3.4). The plan makes that a pure reducer
  and asserts the whole list of activate/deactivate calls per scenario; projection is on. The existing navigation
  tests must pass unchanged.
- **C1·U3 is large** (50 rows in one unit, eight criteria): the pull has many event kinds and each needs its row. If
  the implementer session runs long, the unit boundary before U4 is a safe place to stop and resume.
- **All Pricing shapes come from the package's documents, not from a capture.** Projection on C1 should re-read the
  fixtures against `openapi-integration.json`; the orchestrator's live check after C4 is the first real contact.
- **Does a reply also log a `case.updated`?** The package states only `message.created`. The handler re-reads on any
  event, so either answer works; the fake logs only what the package states.
- **Images in bubbles load from Shopify's CDN** in the webview (the app has no content-security policy today, so they
  load); offline they are blank.
- **Two Macs** each get the chime and banner for the same question, and read marks differ per Mac (09B §11, accepted).
- **The Shopify shop domain** in settings must equal the Pricing store's `shopDomain` ignoring case; a shop configured
  under another spelling of the same shop (for example a custom domain) would not match and conversations stay off,
  with the store names shown.

## Next
Tracker lines are all `NOT_STARTED`. Start with `PSM_IMPLEMENTER=claude tools/pipeline/run_phase.sh phase
C1_connection_and_sync` (projection runs first).
