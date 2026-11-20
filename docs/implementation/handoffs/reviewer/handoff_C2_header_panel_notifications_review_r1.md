---
plan: C2_header_panel_notifications
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-20
actor: opus (pipeline-reviewer)
---
# Review — C2_header_panel_notifications, round 1

**Gate:** passed. The Tracker line read `IMPLEMENTED`. The checkpoints `89809fe` (U1), `e3fd424` (U2), `2076b3d` (U3) and
the close commit `2777d99` are in `git log`. Authority: `docs/process/flow.md` (no mutation probes, blocking only on
B1–B4), the plan's **Review mode**, book 09B §3, §4, §6, §7, §9, §10, and book 01 H1–H8.

## Verified correct
**Perimeter.** `git show --stat` of each unit equals that unit's declared files: U1 has 11 paths, U2 15, U3 12. No path
is extra or missing. The close commit holds only the tracker, the Review log and the handoff.

**Checks.** Review mode, run once, one cargo command at a time, nothing piped or run in the background:
| Command | Result |
|---|---|
| `cargo test -p psm-core --lib mechanisms::conversation_rules` | 17 passed, 0 failed |
| `cargo test -p psm-testkit --test suite c2_conversations_lists::` | 17 passed, 0 failed |
| `cargo test -p psm-infra --lib chime` | 3 passed, 0 failed |
| `cargo test -p psm-desktop` | 79 passed, 0 failed (c2_tests 10, alerts 1) |
| `cargo clippy -p psm-core -p psm-infra -p psm-desktop -p psm-testkit --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npx vitest run src/features/conversations src/features/sessions src/features/shell src/features/settings` | 42 files, 224 passed |
| `npm run typecheck` | clean |

The implementer's full-suite stamp stands (workspace 2035/0, npm test 709/0).

**Criterion rows (66).** For each row I read the production code that makes it true, then its test. Every row has
one test at the boundary the row names, and each test would fail on a wrong outcome. Points that needed checking:
- **C2·C1 (session match).** `ConversationStore::item_sessions` uses `ORDER BY s.id DESC LIMIT 1`, the same rule as
  `Store::session_for_item`, and the service compares SKUs exactly. `items.sku` is `NOT NULL UNIQUE`, so the
  `HashMap` loses nothing. Session delete removes the `sessions` row outright, and the database has no Trash state, so
  09B §3's "not in the Trash" holds. C2·C1(a) is the positive control for (b) and (c).
- **C2·C2(e).** The order `[p, s, q, r]` separates news > blocking > activity. Leaving out any step of `list_order`
  changes the order.
- **C2·C3(b).** The 40-day row's activity (700) lies between the two kept rows (500, 900). A wrong window or a wrong
  sort fails the test.
- **C2·C5(b).** The resolved conversation still counts, because `has_news` ignores status. This is the hazard the plan names.
- **C2·C6(c), (d).** Both tests assert the copy holds the conversation before asserting there is no badge, so they
  cannot pass on an empty copy.
- **C2·C7(a).** `key_set(None)` deletes only the key and keeps the copy (read in `services/conversations.rs`). So
  "no rows" really comes from the hidden state, not from an empty copy.
- **C2·C7(c).** Asserts `here == 1` before the store change, so the zeros come from the change.
- **C2·C7(e).** Clears the fake's request record, then calls all three commands, then asserts the record is still empty.
- **C2·C8–C15 (UI).** Each test asserts what is shown and the service calls with their exact arguments.
  C2·C8(d) renders the real `App` on `#/settings` and `#/health`.
- **C2·C16(e).** Asserts `duration: Infinity`, then a re-issue with the same id at 10 000 ms after `focus`.
- **C2·C17.** Header fields, chunk sizes, duration, a non-zero sample, the peak limit, and deterministic bytes.
- **C2·C18, C2·C19.** A recording `Alerts`, a settable focus flag, and the real `TauriEvents` arm, with the window
  event observed through `listen_any`.

**Review-mode reading checks.**
- **List, count and badge calls never reach the provider or take the lane.** `visible()` reads `connection()` (the
  sync record, settings and the held key) and the store. No `provider` or lane call is on that path.
- **One state on every route.** `useConversations(sessionId)` is called once in `App.tsx`, before the early returns.
  `withConversations` wraps the Health, Settings and main branches with the same provider value, `ConversationsPanel`
  and `ConversationToasts`.
- **The chime never blocks the bridge.** `chime::play` writes the file if needed, `spawn`s `/usr/bin/afplay` and reaps
  it on its own thread. Failures are logged and swallowed. The banner path also returns at once:
  `tauri-plugin-notification` 2.8.1's desktop `show()` hands off to `tauri::async_runtime::spawn` (read in the
  installed source).
- **No sound file was added.** The diff `51d3dbb..2777d99` adds no audio file, and `git ls-files` lists none.
- **Mute.** The panel button saves `{sound}` through the same `conversations_settings_save` as Settings → Pricing.
  C1's `emit` already sets `chime = settings.sound && any`.

**Assumptions (handoff §Assumptions 1–12).** None contradicts book 09B. The owner-visible ones are in the narrative's
"Choices made for you": exact SKU match, newest session, the chime and banner details, Open switching to all
sessions, and the Unassigned page as a session route, with a sentence to change it.

**Hard constraints.**
- H1: nothing is written to Pricing; mute writes local settings only.
- H2–H5: not touched.
- H6: everything is hidden without a key or shop, and the photo workflow is unaffected.
- H7: no Pricing concept in core code or tables. Two doc comments quote the UI words "Pricing is waiting" (a note, below).
- H8: the rules live in `ConversationsService` and `conversation_rules`. The commands only map DTOs, and the bridge
  only applies the window-focus rule, which the plan places there.
- Secrets and live services: the key is not touched; there is no live Pricing or Shopify call in the tests.

**Guide (§7A).** `docs/guide/02_architecture.md` and `04_change_impact.md` have current rows for the commands, the
panel, the alerts seam, the chime file and the plugin.

## Findings
None blocking (no B1–B4).

## Notes written to backlog
All six are in `docs/implementation/backlog_conversations/C2_header_panel_notifications.md` (the plan's Review mode
names that file, not `backlog.md`):
1. **H7 wording.** Two doc comments in `crates/core/src/domain/conversation.rs` quote "Pricing is waiting".
2. **Contract fold.** Add `ConversationStore::item_sessions` to the master plan's names registry, so C3/C4 reuse it
   for the session match.
3. **UX, offered to the owner.** On the Unassigned page the count is always 0 and the panel shows only the footer.
4. **Toasts across routes.** Background toasts waiting for focus are lost if the route changes before focus. This is
   hardly reachable today.
5. **Test shape.** `c2_c10d` reads `parentElement.textContent`, which ties it to the row's markup nesting.
6. **Plugin.** 2.8.1 offers `on_action`. The live check should find out whether macOS reports banner clicks.

## Write perimeter (this session)
- `docs/implementation/plans/C2_header_panel_notifications.md`: the Tracker line and one Review log entry.
- `docs/implementation/backlog_conversations/C2_header_panel_notifications.md` (new).
- `docs/implementation/narratives/C2_header_panel_notifications.md` (new).
- This handoff.

No production or test file was edited, and no probe was applied. No commit (the runner commits).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
