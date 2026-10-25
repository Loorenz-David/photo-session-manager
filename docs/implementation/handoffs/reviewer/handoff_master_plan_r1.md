---
plan: master
role: plan
round: 1
state: OWNER_DECISIONS_PENDING
date: 2026-10-25
actor: Claude Fable 5.1 (pipeline-planner, implementation-planner doctrine under docs/process/flow.md)
---
# Master plan — planning handoff, round 1

**Summary.** Gate passed: `intention.md` reads RATIFIED (2026-10-24); the inventory handoff reports
`OWNER_DECISIONS_PENDING` with a single card I1, which the owner answered on 2026-10-25 (chapter 15 R30, chapter 16
entry 15, chapter 07B) — nothing from the inventory remains open, so I treated the inventory as done. I read the
whole book (01–16 plus 05A/07A/07B/10A), both evidence docs, the design handoff and the three `claude_design` docs,
flow.md, the runner and its templates. I wrote one master plan, 17 phase plans and `phases.txt`. Every one of the 28
ledger measurements is served by at least one criterion row; every row traces to an M/MC/H id or is marked `[rule]`.
Two things need the owner (cards below); one environment fact changes the first step: **the Rust toolchain is not
installed on this Mac** (chapter 03 assumed it). Nothing was committed.

## ⚠ OWNER DECISIONS REQUIRED (2)

**Card 1 — Rust is not installed on the dev Mac.**
- **Question:** Will you install the Rust toolchain yourself before the first phase runs (one command, ~2 minutes), or
  should the orchestrator run the installer for you?
- **Story:** The first build session starts tomorrow morning, types `cargo --version`, and gets "command not found".
  It cannot install Rust itself: it runs in a sandbox that may not write to your home folder. It stops, writes
  "blocked", and the whole run waits until someone notices. Every later phase waits behind it.
- **Branches:** *I install it* — run `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y` in
  Terminal, open a new Terminal, done. *Orchestrator installs it* — it runs the same command in your session before
  launching P00. *Neither* — P00 cannot start.
- **Recommendation:** install it yourself (you see what is installed, and the permission prompt lands on you).
- **On silence:** the gate holds — P00 is not launched until `cargo --version` works.
- **Trace:** master plan §7.1 (Rust row), §5 (preconditions); P00 task 0.

**Card 2 — How does a new local item get its SKU?**
- **Question:** When you create an item that is not yet in Shopify ("Create 'Brass floor lamp'"), should you type the
  SKU, or should the app generate the next one from a prefix (like `VF-0421`)?
- **Story:** Thursday you photograph a lamp nobody has catalogued. You type "Brass floor lamp", the app creates the
  session and names its capture folder after the SKU. If the app invents `VF-0421` and your warehouse later labels
  the lamp `BFL-17`, the SKU is locked once the first frame lands, and the Shopify product is created with the wrong
  one at first publish. If you type it, you need the number at hand while shooting.
- **Branches:** *Type it* — a required SKU box in the create row; nothing generated. *Generate it* — you set a prefix
  once in Settings; the app proposes `<prefix>-<next number>`, editable before confirming.
- **Recommendation:** Generate with an editable proposal, because the design shows this flow and it never blocks a
  shoot; the prefix stays yours.
- **On silence:** the gate holds for that detail only — P07 builds the typed-SKU box (the smallest option) and the
  generator waits for your answer.
- **Trace:** ch04 S-D; evidence design_handoff §New session; master plan §6.8 `sku_prefix`; P07 task 4, row C5(b).

## Gate check
- `docs/intention/intention.md` line 3: `Status: RATIFIED (owner David Loorenz, 2026-10-24; chapter 16 entry 13)`.
- `handoffs/reviewer/handoff_intention_inventory_r1.md`: `state: OWNER_DECISIONS_PENDING`, one card (I1). Resolved by the
  owner: chapter 15 "Open owner cards: None. I1 … answered Yes on 2026-10-25: R30"; chapter 07B written; chapter 16
  entry 15. I read the gate as satisfied (the prompt's "reports DONE" was literally false but materially met); the
  orchestrator may disagree and re-run the planner — nothing else would change.

## Files written (write perimeter — documents only; no code, no commits, no changes outside `docs/implementation/`)
- `docs/implementation/master_plan.md`
- `docs/implementation/phases.txt`
- `docs/implementation/plans/P00_spike.md`, `P01_storage_identity.md`, `P02_review_order.md`, `P03_session_state_list.md`,
  `P04_capture_one_adapter.md`, `P05_capture_arrival.md`, `P06_live_previews.md`, `P07_shell_sessions_ui.md`,
  `P08_workspace_ui.md`, `P09_local_export.md`, `P10_attributes.md`, `P11_form_ui.md`, `P12_shopify_adapter.md`,
  `P13_publish_runs.md`, `P14_publish_update.md`, `P15_publish_ui.md`, `P16_operations.md`
- this handoff.
Not touched: the book, evidence, flow.md, runner, templates, STATUS.md, backlog, any prompt. Probe commands I ran were
read-only (`--version`, `ls`, `cat`, `npm view`, `curl` to crates.io/npm/docs); nothing was installed.

## Phase map (five lines each: what it builds · what the owner can try · size · difficulty/projection · depends_on)
1. **P00_spike** — Cargo workspace, Tauri app, data dir + settings (0600), SQLite, osascript runner (cwd = data dir),
   compiled hook scripts, spool + loopback nudge, FSEvents watcher, typed bindings, Health page; spike checklist
   (Automation permission, hook reaching the app, ad-hoc build) recorded in new evidence `packaging_spike.md`.
   Try: launch the window, allow the Capture One permission once, build and open the `.app`. ~45 files / 2 500 lines.
   high / projection no / depends on nothing (plus Rust installed — card 1).
2. **P01_storage_identity** — domain schema, Store, path canonicalization, RAW hash + integrity, folder key with
   case-insensitive collision refusal, items/sessions services, Unassigned built-in. Try: `sqlite3 .tables`.
   ~14 files / 1 400 lines. high / no / P00.
3. **P02_review_order** — classify, reorder, delete, undo, move, copy with exact position math; change sequence and
   derived dirty flag. Try: nothing visible yet (P08). ~8 files / 1 100 lines. normal / no / P01.
4. **P03_session_state_list** — status and row label (complete rule order), calendar grouping incl. DST day,
   sessions list query with filters/search/counts. Try: nothing visible yet (P07). ~7 files / 900 lines. normal / no / P02.
5. **P04_capture_one_adapter** — typed Capture One scripting layer (JXA) + in-memory fake, capture-folder switch with
   read-back and revert, hook install, recipe fixes, variant resolution, coalesced colour-tag mirror with read-back,
   render issuing, camera status/capture, durable job runner. Try: Health shows document/hooks/recipes/camera; hook
   properties visible in Capture One. ~30 files / 2 200 lines. high / **projection yes** / P00, P01.
6. **P05_capture_arrival** — hook dispatch, candidate filter, stability, session attribution by folder, one row per
   RAW however delivered, embedded-JPEG original preview (byte-exact), captured_at from mtime, import, assign
   Unassigned. Try: shoot → one `photos` row, `original.jpg` on disk. ~12 files / 1 800 lines. high / no / P04.
7. **P06_live_previews** — render completion matched to the job, generation-numbered preview files, staleness with the
   10–30 s delay and focus trigger, sidecar fallback, retouch → dirty (own tag writes exempt). Try: edit in Capture
   One, see the path change after 15 s or at once on focus. ~9 files / 1 500 lines. high / no / P05.
8. **P07_shell_sessions_ui** — app shell, camera dot with retry, grouped sessions list, filters, search, Unassigned
   card, New session dialog with "Create '…'". Try: press N, create an item, see the row. ~22 files / 1 900 lines.
   normal / no / P03, P04.
9. **P08_workspace_ui** — stage with crossfade, Original/Adjusted pill, A/C/⌫/⌘Z, swipes, next-unreviewed rule, undo
   toast, dnd-kit grids and folded strip, menus, Unassigned workspace with drops, import, arrival toast. **Local
   review usable.** Try: shoot and sort a session end to end. ~22 files / 2 600 lines. high / no / P06, P07.
10. **P09_local_export** — export job (hash check, web renders into staging, exact names, item.json, atomic swap),
    Export header action without an integration. **Local workflow complete.** Try: Export → `~/Pictures/Session
    Manager Export/<SKU>/`. ~10 files / 1 300 lines. high / no / P06, P02.
11. **P10_attributes** — definitions, canonical value forms per type, invalid-but-shown values, provenance rules,
    presets/customize, damage-report exclusions, item.json attributes. Try: nothing visible yet (P11). ~9 files /
    1 600 lines. high / no / P09.
12. **P11_form_ui** — form dialog: steps and counts, field-kind generator, customize mode, SKU/barcode editor, AI
    pill/popover/research states, damage-report picker. Try: press D, fill Title, customize fields. ~32 files /
    2 800 lines. normal / no / P10, P08.
13. **P12_shopify_adapter** — grounding probe against the dev store (writes `evidence/shopify_admin.md` + sanitized
    fixtures), GraphQL adapter with every capability primitive, type map both ways. Try: New session finds shop
    products; the form shows the shop's fields. ~16 files / 2 600 lines. high / **projection yes** / P10.
    **Precondition: the Shopify token** (`SHOPIFY_TOKEN`, `SHOPIFY_SHOP`) must be available to the session.
14. **P13_publish_runs** — confirmation freeze, mutation token, prepare (create/adopt, hash, export), upload with
    crash-safe adoption, gallery mirror with fail-before-delete, done/retry/resume. Try (after P15): publish, kill the
    app mid-upload, relaunch, one media per photo. ~10 files / 2 400 lines. high / no / P09, P12.
15. **P14_publish_update** — attribute diff writes with read-back, tags three-way merge, title/description, damage
    report files, retouch replacement in place, the summary text. Try: retouch a published photo → Ready → publish →
    same gallery slot. ~6 files / 1 400 lines. high / no / P13.
16. **P15_publish_ui** — Publish popover, progress, failure toast/popover/Retry, badges, batch queue, notifications
    drawer. Try: publish one, publish three, retry a failure. ~12 files / 1 500 lines. normal / no / P14, P11.
17. **P16_operations** — settings and health screens, backups (30 kept), daily logs with durations and run ids,
    secrets proven absent from logs, final ad-hoc build and operational measurements. Try: Settings, Health, logs,
    the `.app` on the work Mac. ~12 files / 1 200 lines. normal / no / P15.

Derived totals: 17 phases; criteria per phase 7, 7, 6, 5, 8, 8, 6, 6, 8, 7, 8, 6, 8, 8, 7, 6, 5 (none above 8);
rows per phase 16, 20, 25, 29, 21, 33, 18, 17, 22, 10, 31, 30, 53, 28, 19, 11, 11 (total 394); `[rule]` rows 3, 3, 0,
3, 2, 1, 0, 10, 12, 1, 7, 28, 0, 0, 0, 10, 7 (total 87, concentrated in the three frontend phases and P16).

## Ledger coverage (every M served)
M1 P05 · M2 P04 · M3 P13, P14 · M4 P13 · M5 P13, P16 · M6 P01, P09 · M7 P09, P10 · M8 P01 · M9 P05 · M10 P01, P09,
P13 · M11 P01, P03, P05, P07 · M12 P02, P08 · M13 P02, P06, P10, P13 · M14 P03, P07, P13 · M15 P01, P09, P13 ·
M16 P10, P12 · M17 P13, P14, P15 · M18 P12, P13 · M19 P12, P13, P14 · M20 P12, P14 · M21 P10, P12, P14 · M22 P00, P04 ·
M23 P06, P09 · M24 P06, P08 · M25 P04 · M26 P04 · M27 P05 · M28 P06, P14. Operational measurements (ch14 last paragraph):
P00 spike record and P16 final measurements.

## Planner decisions recorded (not owner cards; the orchestrator can overturn by amending the master plan)
- Crate and bundle names (`psm-*`, `com.loorenz.sessionmanager`); `settings` **table** = app state, `settings.json` =
  user configuration (ch05 and ch12 both name "settings"; this split keeps both true).
- `raw_state` table for the per-RAW counters MC-17/19/21 declare "per RAW"; `notifications` table for ch07/12 rows.
- One session per item in the MVP (`session_create` returns the existing session) — P07 Notes; R6.
- Projection on P04 and P12 only (externally owned fixtures). P05's unprobed capture-done argv is absorbed by design.
- Trace conventions allow `[rule]` rows (chapter/design rules without a ledger entry) for UI rendering, ch12
  operations and ch11 packaging, always marked and counted — master plan §3.
- Export is also the publish's step one, so the publish phases depend on P09.

## Preconditions and facts for the orchestrator
- **Rust absent** (card 1). After installation, P00's implementer verifies and amends master plan §7.2 with the real
  toolchain version; cargo commands in §7.2 are the standard ones and could not be executed in this session.
- **Codex sandbox** is `workspace-write`; `~/.cargo` and `~/.npm` caches may be unwritable. P00 task 0 handles it and
  records the outcome; consider adding `writable_roots` in `~/.codex/config.toml` to avoid a BLOCKED first session.
- **Shopify token** before P12 (STATUS.md already tracks this). The P12 probe writes once to the dev store and deletes
  what it created.
- **Owner checks still owed** (not decisions): camera set to release without focus (VERIFY-12), colour-tag visual check
  (VERIFY-4). P00's spike checklist item (g) re-runs the capture probes when possible.
- `.pipeline/runs/` contains dry-run leftovers named `P0_*` from the runner's self-test; phase ids here are `P00_…`,
  so there is no collision.
- Inventory handoff state is `OWNER_DECISIONS_PENDING` with the card resolved; the runner's `plan` command checks only
  the planner's state, so nothing blocks.
