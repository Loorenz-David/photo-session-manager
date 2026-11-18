# STATUS, research track — read first (updated by the research orchestrator at every stop)

Branch `feature/agentic`, worktree `~/Desktop/Developer/Photo-Session-Manager-agentic`. Never work in or run git
against the sibling checkout `~/Desktop/Developer/Photo-Session-Manager` (`main`, the owner's other work).
`docs/implementation/STATUS.md` belongs to `main`'s orchestrator; do not edit it from this track.

**2026-11-15: R1–R13 APPROVED (notes in `backlog_research/`); research joined `main` on GitHub (PR #1, `04f26ad`);
the agent guide `docs/guide/` covers research (PR #2). Nothing running.** Intention 08A RATIFIED and contracts 08B
RATIFIED (owner, 2026-11-12); later owner decisions are 08B changelog 12–18 and the "supersedes" sections of the R11–R13
plans. Thirteen milestones in `docs/implementation/phases_research.txt`; rules in `docs/process/flow.md` §10; skeleton,
tracks and driver loop in `docs/implementation/master_plan_research.md` (its §3 "Scratch data only" and parts of §6.6
predate the merge; `docs/guide/` notes where). Next for the owner: try it on the work Mac (backup, keys, chosen fields).
- **2026-11-17: owner tried research on the work Mac.** Nothing ran until "Fields research fills" was ticked (fresh
  install: empty list; `arranged` queues nothing while a creation prerequisite fails). **Owner corrections A6**
  (changelog A6, 08A §7 amended): web links open in the default browser (`tauri-plugin-opener` 2.7.2, http/https only,
  `components/ExternalLink.tsx`); "n to review" opens a list of the fields and goes to the chosen one's popover;
  `research_overview` shows each session's research state in the sessions list and the session header; a queued run
  says whom it waits for and its place (`ResearchStatus.queue`). One run at a time unchanged. Joined: PR #7 merged into
  `main` as `da08fe3`; release build from `da08fe3` (SDK helper 352928 bytes in the bundle) installed in /Applications;
  the previous (A5) build and zip moved to `~/Desktop/Session Manager old (2026-11-12)/` as "(2026-11-16 evening)"; new
  `~/Desktop/Session Manager.zip` 12,550,675 bytes. **Waiting on the owner:** the work Mac's log folder and
  `session_manager.db` for the many "could not be read" pages on the owner's own shop (page reader, not touched yet);
  the deferred batch idea (several sessions at once + OpenAI Flex), read later by the owner.
- **2026-11-17: owner corrections A7** (changelog A7; 08B MC-A9/MC-A11 and 08A §7 amended). Export research stopped
  with "…images/<sha>.jpg contains a key": the fail-closed check read JPEG pixel data as text and a short stored
  value (the camera's access user name or a short password) turned up by chance. Now a JPEG is checked up to its
  first start-of-scan marker (`export::scanned_part`; other files and unwalkable JPEGs whole) and
  `research_secret_values` leaves out `access-auth-user` (a name, not a secret). Research steps have distinct
  looks (icon, label, rail; conclusions stand out, `StepView.unread` pages are a muted "Could not read" warning).
  Joined: PR #8 merged into `main` as `c6e662c`; release build installed in /Applications (helper 352928 bytes in
  the bundle); the A6 build and zip moved to the old folder as "(2026-11-17 A6)"; new
  `~/Desktop/Session Manager.zip` 12,552,228 bytes. Still waiting on the owner's work-Mac files for "could not be
  read" on the owner's shop.
- **2026-11-18: owner correction A8** (changelog A8; 08B MC-A11 amended). With A7 on the work Mac, Export research
  still stopped on a listing image: those are stored as the shop sent them (PNG/WebP/GIF/AVIF under `.jpg`), and
  only JPEGs were trimmed. `rules::image_text::scanned_parts` now recognises each format by its leading bytes and
  reads only its text/metadata parts (JPEG header; PNG text chunks inflated, `eXIf`, bytes after `IEND`; WebP
  `EXIF`/`XMP `; GIF comment/plain-text/application extensions; AVIF/HEIF all top-level boxes but `mdat`);
  anything else or unwalkable is read whole. A choice with no run refuses with "No research runs match this
  choice." (`Error::NothingToExport`) instead of an empty 22-byte zip (the owner's first try gave one). Listing
  images keep the `.jpg` name (export layout fixed in master plan §6.7). Under load the full run had two failures
  that passed when rerun: p33 C4(d) and `p50…c5a`, which shares the global log-redaction registry. Joined: PR #9
  merged into `main` as `430f156`; release build installed in /Applications (helper 352928 bytes in the bundle);
  the A7 build and zip moved to the old folder as "(2026-11-17 A7)"; new `~/Desktop/Session Manager.zip`
  12,555,683 bytes. Next: the owner exports on the work Mac and sends the zip (the "could not be read" pages).
- **2026-11-18: owner corrections A9** (changelog A9; 08B MC-A3, MC-A9, MC-A11 amended; 08A §6), from the work
  Mac's data (database, logs and the stuck image, brought by the owner). Export still stopped on the same image: a
  real JPEG with a 42 KB header (EXIF and Photoshop thumbnails, XMP), so A8's premise did not hold for it; the cause
  was the camera's short FTP code on research's secret list (it had also redacted one search record).
  `research_secret_values` is now the three research keys plus the client secret. 6 runs took 4–8 min, 3–6 min of
  it waiting: the owner's shop answered 21 of 23 page reads 429 with `Retry-After: 60`. The shop's own product pages
  are now read through Shopify (`CommerceAdapter::shop_product`, `productByIdentifier`; `ShopPageReader` wraps the
  reader in the app wiring; read identity route `+shop_api`). `Retry-After` is waited only up to 10 s; a 403 or a
  long 429 is `blocked` and the run skips that host ("Skipped: <site> is blocking automated reading"). Locale and
  collection spellings of one product are read once; collection and category pages are not read. Checks: workspace
  1579 passed, app crate 30, clippy (after boxing `Answer::Page`'s reply) and fmt clean, tsc, vitest 560/560,
  frontend build, key scan 0. Joined: PR #10 merged into `main` as `4228653`; release build installed in
  /Applications (helper 352928 bytes in the bundle); the A8 build and zip moved to the old folder as
  "(2026-11-18 A8)"; new `~/Desktop/Session Manager.zip` 12,570,736 bytes. Not verified: the API read against the
  owner's real shop (the query was accepted by the dev store, whose products have no fields), and a run on the work
  Mac. Next: the owner installs the zip on the work Mac, re-exports and runs research again.
- **2026-11-18: owner correction A11** (changelog A11; 08A §7 amended). Checking suggestions one by one ("n to
  review" → field → AI pill) was slow. The form has a fourth tab, **AI review** (`AiReviewTab.tsx`), with the
  number still to check. It lists every `ai` + `suggested` field, each with its step, what the field held before,
  the suggestion, confidence, reason, sources and the pill's own actions (`SuggestionBody`, shared with
  `AiPopover`; same rules and undo), plus Show field. Accept all is at the top. At 0 the tab says why. The form's
  "n to review" chip and the header's "AI · n to review" open it; A6's popover list is gone. No separate "Confirm
  all high-confidence": Accept all already is that. Checks: fmt and clippy clean; `cargo test --workspace` 1586
  passed, 0 failed; tsc; vitest 571/571 (new: a11_ai_review_tab 6, a11_header_review 2); frontend build; key scan 0.
  Joined: PR #12 merged into `main` as `37edffe` (code `28a97ab`). Release build installed in /Applications (helper
  352928 bytes in the bundle). The A10 build and zip moved to the old folder as "(2026-11-18 A10)". New
  `~/Desktop/Session Manager.zip` 12,606,400 bytes. Not verified on screen (jsdom tests only).
- **2026-11-18: owner correction A12** (changelog A12; 08A §7 amended). The owner's manual check (screenshot →
  Google Lens → copy facts back) is now a **Search this photo** window. Clicking a photo in the form's media preview
  opens it.
  - **Layout:** one window per session through Tauri's multiwebview (`unstable` feature). The app's page is
    `search-ui-*`, which renders `SearchWindow.tsx` from `#/search/<s>/<p>`. The web pages are in a separate child
    webview, `search-web-*`, placed over the browser area.
  - **Google route:** the photo (`browse::lens_image`: framed, 1600 px, no metadata) is posted from Google's image
    page to `lens.google.com/v3/upload`. The route was probed on 2026-11-18 with an icon-derived image: 303 to
    `www.google.com/search?…udm=26`, after Google's EU cookie page.
  - **Fallback:** with no results within 30 s, the window lists the session's recorded visual matches, or runs one
    visual search.
  - **Saved pages:** stored per item in `saved_pages` (migration 018).
  - **Selection actions:** Put in (a human edit with undo) and Ask AI (a `read`-stage call; `medium` suggestions
    with the page as source, via `writes::suggest`).
  - **`browse` runs:** `RunTrigger::Browse` gives each call a gateway record, cost, day limit and export entry.
    `store::runs` leaves browse runs out; `store::all_runs` is used for the export.
  - **Trust boundary:** `allowed_navigation` limits every frame to public http/https and never allows the app's
    origins or local hosts. No remote capability exists. The selection is read by the app's page asking
    (`SELECTION_SCRIPT`) and is checked by `parse_selection_report`.
  - **Checks:** fmt and clippy clean. `cargo test --workspace` 1602 passed, 0 failed; new: engine `browse` 8, app
    crate `a12_` 8. tsc passed. vitest 584/584 (new: `a12_search_window` 13). Frontend build passed. Key scan 0.
    Break-the-fix: letting any http(s) host through `allowed_navigation` failed 3 tests; the rule was restored.
  - **Joined:** PR #13 merged into `main` as `b0dd647` (code `1007af4`, WIP `1c16732`). The release build is
    installed in /Applications (helper 352928 bytes in the bundle). The A11 build and zip moved to the old folder
    as "(2026-11-18 A11)". New `~/Desktop/Session Manager.zip` 12,835,563 bytes.
  - **Not verified on screen:** the window, the Google route inside WKWebView, real sites, and the selection read
    on real pages. Next: the owner tries it, and logs `psm::search` lines if something is off.
- **2026-11-18: owner correction A13** (changelog A13). In the owner's first live test, Google looked old, the
  route ended on an empty Google page after the cookie choice, and the toolbar was hidden. All three causes were
  confirmed and fixed:
  - **User agent:** WebKit's default UA got Google's basic pages, and the Lens post landed on `/search` without
    `udm=26` (probed). The fix is `PAGE_USER_AGENT`, desktop Safari.
  - **Lost post:** the cookie page swallowed the first post (seen live), and A12 never re-posted. The photo is now
    kept and re-posted on a non-results Google page, at most `MAX_POSTS` = 2 times.
  - **Hidden toolbar:** the window's content view includes the 32 pt title bar (probed natively), so both webviews
    sat 32 pt high. The app's page is now laid out below `contentLayoutRect` on open and on every resize, with no
    `auto_resize`. The browser is placed at area + offset, clamped, and re-placed when any column row changes.
  - **Logs:** `search_window.navigation host= kind=` and `step=photo_posted_again`.
  - **Checks:** fmt and clippy clean. `cargo test --workspace` 1607 passed, 0 failed (new: 5 `a13_`). tsc and
    build passed. vitest 586/586 (new: `a13_search_placement` 2). Break-the-fix: the offset, the re-post and the
    row watching each fail their tests when removed. Key scan 0.
  - **Live, by a probe through the real `open()`:** the layout is exact before and after a resize. Post 2 reached
    the `udm=26` results, and the cookie choice persisted.
  - **Joined:** PR #14 merged into `main` as `39c17f9` (code `509fa4d`) and installed in /Applications (helper
    352928 bytes). The A12 build and zip moved to the old folder as "(2026-11-18 A12)". New
    `~/Desktop/Session Manager.zip` 12,839,012 bytes.
  - **Not verified:** Google's matches rendering. From this Mac, Google answered the repeated test uploads with its
    "unusual traffic" page, which a person can solve in the window. The selection bar on real pages is also
    unverified. If the owner sees that page, it is Google's check, not the app.
- **2026-11-18: owner correction A14** (changelog A14). The owner confirmed A13 works, and asked for two things:
  a nicer selection bar, and for links in Google's AI Overview to open.
  - **Selection bar:** `SelectionBar.tsx` is a floating bar in a fixed-height band under the page.
    - It holds the snippet, Manufacturer/Designer, an inline searchable "Other field…" list, an inline Ask AI
      prompt with "Open AI review", dismiss, and a hint pill when nothing is selected.
    - "Open AI review" calls `search_open_review`, then `useReviewFromSearch` opens the main window on the AI
      review tab.
    - "Put in" shows a toast with Undo: `search_put_text` answers the previous value, and `search_put_back`
      restores it.
    - The toolbar and the saved-page pills were restyled.
    - Nothing is drawn over the page webview: toasts go over the photo panel and tooltips are native titles.
  - **Links:** an empty `window.open('')` followed by setting the address never reached the app, because Tauri
    drops the `""` URL; `about:blank` would have blanked the page. This was probed in a wry 0.57.1 harness.
    `page_popups.js` now runs in every frame and opens new-window links, forms and `window.open` in place,
    http/https only, and reads and sends nothing.
    - Not confirmed: that this is the pattern Google's AI Overview uses. Google shows "unusual traffic" to the
      dev Mac.
  - **Checks:** fmt and clippy clean.
    - `cargo test --workspace`: 1611 passed, 0 failed (new: 4 `a14_`).
    - tsc and the build pass. vitest: 598/598 (new: 12).
    - Break-the-fix: the `window.open` override and the link retarget each fail their tests when removed.
    - Key scan: 0.
  - **Joined:** PR #15 is merged into `main` as `2f35a32` (code `61c3365`) and installed in /Applications
    (helper 352928 bytes). The A13 build and zip moved to the old folder as "(2026-11-18 A13)". New
    `~/Desktop/Session Manager.zip`: 12,868,382 bytes.
  - **Not verified on screen:** the new bar and toolbar, and the AI Overview links on live Google.
- **2026-11-18: owner correction A15** (changelog A15). The owner liked the A14 bar and found one bug: going back
  from "Other field…" without picking a field lost the selection and its actions.
  - **Cause (from the code):** the 0.7 s selection reading set the selection to `null` whenever the page answered
    "nothing". The page's selection goes once focus moves into the app's bar, and the bar's own copy (`held`) only
    covered the field list and the prompt, not the way back.
  - **Fix:** `keptSelection()` / `samePage()` in `SearchWindow.tsx`. An empty reading never drops the selection
    taken; it goes only on another selection, a refused one, ✕ (dismiss now lives in the window), or another page
    (a #fragment change is the same page). "Put in" and an AI answer keep it for a second action.
  - **Checks:** fmt and clippy clean. `cargo test --workspace`: 1611 passed, 0 failed (no Rust change). tsc and the
    build pass. vitest: 605/605 (new: 7 in `a15_keep_selection.test.tsx`). Break-the-fix: an empty reading dropping
    the selection again fails 5 of the 7. Key scan: 0.
  - **Joined:** PR #16 is merged into `main` as `d6a807b` (code `c5dd481`) and installed in /Applications
    (helper 352928 bytes). The A14 build and zip moved to the old folder as "(2026-11-18 A14)". New
    `~/Desktop/Session Manager.zip`: 12,868,442 bytes.
  - **Not verified on screen:** the bar with real pages.
- **2026-11-18: owner correction A18 and owner decisions A19** (changelog A18, A19).
  - **A18:**
    - Pinning another photo no longer searches. Each new search opens its own tab: one web view per tab, at most 8.
    - Ask AI has an "Answer me" mode (the default): a short answer plus "Put … in …" chips that write only when
      tapped, like "Put in". The selection limit is now 6,000 characters.
    - Page zoom is per tab (toolbar and View-menu ⌘+/⌘−/⌘0). The pinned photo zooms and pans.
    - AI review rows: Keep previous whenever a prior value exists (the backend now allows it outside conflicts),
      Dismiss otherwise, and a calmer layout.
  - **A19 (owner):**
    - Research starts automatically at most once per session (queued, running, finished or stopped at the cap blocks
      Arranged). Only Run again researches again. A failed or cancelled run can still be retried by arranging.
    - Arranged — next goes to Sessions with New session focused.
    - Review queue: "To review (n)", Details lands on AI review, "All checked ✓" leads to "Next to review", and the
      header shows "Next to review (n left)".
  - **Checks:**
    - `cargo test`: 1,630. clippy and fmt: clean.
    - vitest 629/629, tsc and the build: green.
    - Break-the-fix: every deliberate break was caught.
    - Key scan: 0.
  - **Joined:** PR #19 is merged into `main` as `7640a4b` (code `7c1b359`, `fb430dd`).
  - **Not installed yet:** it will be installed together with A20 (Capture One switch speed) and A21 (moving photos
    moves their files; repair for T1-181126). Those are being built in an isolated worktree branched at `fed8796`.
  - **Not verified on screen:** second-tab Lens, page zoom on Google, and ⌘+/⌘− while the page has the keyboard.
- **Done:** `R2_research_foundation` APPROVED_WITH_NOTES 2026-11-12 (implemented by Codex, the runner's default, before the
  Opus preference below was applied; reviewed by Opus). Narrative relayed to the owner.
- **Done:** `R3_research_rules` APPROVED_WITH_NOTES 2026-11-12 (Opus implementer). Notes folded: PriceMissing scope → R4
  plan task 2; MC-A4 invariant scoped → 08B changelog 4. Other notes stay in `backlog_research/R3_research_rules.md`
  (→ R6/R4·U3 Details+Product field; → R8 quota-vs-key order). Narrative relayed.
- **Done:** `R4_research_engine` APPROVED_WITH_NOTES 2026-11-12. All routed notes of R2–R4 folded into R5–R10 plan files
  ("Carried from earlier reviews" sections; implementers never read `backlog_research/`), master plan v3, 08B
  changelog 5 (MA12 sent requests). Two notes are the owner's (below, "Needs the owner").
- **Done:** `R5_research_app_seams` APPROVED_WITH_NOTES 2026-11-12 18:44 (attempt 8; seven projection rounds before it,
  all archived; 08B changelogs 6, 8, 9, 10). Notes folded into R6·U3 (release events, undo after release, p15 fixture).
- **Done:** `R6_research_ui` APPROVED_WITH_NOTES 2026-11-12 19:12. Notes folded into R7·U1 (settings save, release signal
  incl. undo/redo, wait clock, `conflict` field + bindings allowance, Accept all count); the rest stays in
  `backlog_research/R6_research_ui.md` (popover Edit for non-text fields, shop public domain shown).
- **Done:** `R7_research_wiring_and_export` APPROVED_WITH_NOTES 2026-11-12 19:52. Notes folded into R8 (export secret scan,
  `wait_ends_at` row, long calls) and R9 (TZ pin); event→change rule stays in `backlog_research/`.
- **Owner, 2026-11-12 night:** attached billing (card A) and delegated: "take care of this and continue resolve
  questions given the intention" while asleep. The orchestrator verified the link in the console, ran the Vision
  benchmark (Lens 24/25 + 6/7 vs Vision 8/25 + 0/7), merged `research/track-b` into `feature/agentic` (`3297b17`,
  workspace check green), set the R1 implement handoff to DONE with a resolution note.
- **Done:** `R1_research_benchmark` APPROVED_WITH_NOTES 2026-11-13 01:12, 3 owner cards. Picks made by the orchestrator under
  the owner's delegation, all as recommended, recorded in the evidence doc "Owner picks": Lens via SerpApi (Vision
  the replacement, not built), plain HTTP with slow Shopify hosts, models observe Sol low / read Terra low /
  condition Terra high. R1 notes folded into R8·U3 and R9·U1/U2. Owner answers 2026-11-13 morning (evidence doc
  "Owner answers"): piece 30's wood is "Santos Rosewood" (corrections file updated); pamono.se/.eu/.co.uk trusted
  (tool list now 12); shop addresses listed; designer and maker are the shop's "Designer"/"Manufacturer" fields and
  R10 makes them a settings picker like the condition-photos field; USD 40 is OpenAI API spend. Track B's build folder: the owner said delete it, but the
  permission prompt denied the command again; the owner runs `rm -rf ../Photo-Session-Manager-agentic-b/target`.
- **Done:** `R8_research_real_adapters` APPROVED_WITH_NOTES 2026-11-13 02:30 (attempt 3; two projection rounds archived;
  08B changelog 11; master plan v5–v6). Page-reader notes (charset, proxy, address ranges) folded into R9·U1 task 0;
  §3 Isolation lesson applied (v7). Not yet run live against the real services.
- **Done:** `R9_research_tool_and_acceptance` APPROVED_WITH_NOTES 2026-11-13 03:07 (tool, `accept`, `score-accept`,
  page-reader fixes). Owner card in its narrative: models per stage after the measurement (→ R10).
- **2026-11-15 11:11: R10 APPROVED_WITH_NOTES** (defaults: picked models, prices incl. cache-write and Lens USD 0.025,
  auto off, attribution keys empty; notes → R11: reset-settings action, app tests set attribution keys). R11 and R12 plans
  written (planner handoff `handoffs/reviewer/handoff_research_plan_r3.md`; 08B changelog 14). R11 APPROVED_WITH_NOTES 13:02
  (Codex built U1–U5, hit its usage limit until 16:42; Claude finished U6, one fix round). R12 APPROVED_WITH_NOTES 13:37
  (Claude, Codex limited). **Second measurement DONE 2026-11-15 20:56** (evidence doc "Acceptance measurement 2": USD 0.11 per piece, was 0.30; designer/maker sure answers 0 really wrong; detailed condition waits for all levels; finding: research fills price/location/link fields → owner) — was (R12·U4 tasks 3–6, ceiling USD 20, script `scratchpad/accept_run3.sh`, log `scratchpad/a25_run.log`,
  `accept --settings docs/test-items/accept_settings_owner.json` into `~/psm-research-bench/a25`, then `score-accept`). Owner: camera routing stays as is (08B changelog 16). Owner card open (not blocking): detailed-condition table (recommended A).
- **2026-11-15 21:10: owner decision after the second measurement** (08B changelog 17, master plan v11): research
  fills only chosen fields (allow-list replaces "leave alone"), detailed condition derived from the levels that have a
  value. **R13_research_chosen_fields** planned (4 units, 15 criteria, 40 rows `A27·…`, planner handoff r4 DONE, 0 owner
  cards; 08B changelog 18). **R13 APPROVED_WITH_NOTES 23:19** (Codex built, no fix round; 5 notes in
  `backlog_research/R13_…md`, incl. restore `spawn_blocking` in `settings_set`). U4 task 3 done (0 values outside the 22
  fields, detailed condition = table grade on 4/4, USD 0.40). Full suite, clippy, fmt, vitest 515, frontend build
  green on `feature/agentic`; real-key scan of tip and branch history: 0. **Joined into `main` 2026-11-15 23:30: PR #1 merged on GitHub (`04f26ad`)** after merging `main` `e4d5c06` (docs/guide, backlog conflict kept both sides) into this branch. The local `main` checkout was not touched; the owner pulls it.
- **After R13 (owner approved 2026-11-15): bring research into `main` for the work PC.** `main` has not moved since the
  merge base `9dc94dc` (P70), so the join is a fast-forward and this worktree is the combined version: run the full suite
  and a release build here, scan the branch for secrets, then push `feature/agentic`, open a PR to `main` and merge it on
  GitHub (the sibling `main` checkout stays untouched; the owner pulls it). Test data in `docs/test-items/` stays (owner:
  removing it is "not necessary"). Remind the owner: back up the work PC data folder before the first start (one-way
  schema), enter both research keys, set the chosen fields; owner adds mark/stamp and contains key by hand as a test.
- **2026-11-15: owner picked the models** (observe Sol low, read Luna low, compare Terra low, condition Terra high) and
  moved the trigger to an "Arranged — next" button (08B changelog 13, master plan v9). R10 amended (crate-only,
  auto off, attribution keys empty) and launched with Claude (Opus); **owner 2026-11-15: from R11 on, Codex implements again
  (runner default, no `PSM_IMPLEMENTER`)**; R11 (trigger + pickers + derived condition + look-alike) and R12
  (cost + re-measure) need plans.
- **Owner decisions after the measurement (2026-11-13):** 08B changelog 12, master plan v8: follow-up phase R11
  (fields research leaves alone, condition deferred, photo look by shoot template, light parser, look-alike fix).
  Task 5 comparison done (96 replays, evidence doc "Model comparison"); waits for the owner's model pick → R10.
- **Measurement tasks 3–4 done (2026-11-13):** 42/42 finished (a23 pieces 1–15, a24 pieces 16–42 after the owner added
  credit); results in the evidence doc "Acceptance measurement". Real spend ≈ USD 12.6; ledgers 8.77 + 7.89 = 16.66
  of the 40. Next: relay; task 5 model comparison (replay, budget by count) when the owner nods; owner asked about
  pieces 7, 11 keys. Earlier note — measurement stopped part-way: OpenAI account out of credit at piece 16 ("You have no credits
  remaining"); pieces 1–15 finished (all published group), 16–42 failed at observe. Real spend ≈ USD 4.65 (15 × median
  0.30, 148 s median… 5–7.5 min each); the folder's ledger shows USD 8.77 because each refused call is charged its
  worst case (0.15 per failed run), which eats the per-folder ceiling. First look at the 15: designer/maker at
  medium/high 11 right, 6 "wrong", of which 2 are name forms (Wørts/Wörtz, B./Bertil Fridhagen), 2 doubtful keys
  (piece 7 maker "Architecture & Design", piece 11 maker), 2 real (piece 6: matched the shop's look-alike "model 77"
  listing). Waits for the owner to add OpenAI credit; then run pieces 16–42 in a fresh folder (a24) with the
  remaining ceiling. Earlier note: `accept` → `score-accept` into `~/psm-research-bench/a23`,
  ceiling USD 40 (script `scratchpad/accept_run.sh`). Original hold note: the measurement (R9·U2 tasks 3–6), owner-attended per the
  plan, real keys, ceiling USD 40 total for tasks 3–5.** Not run overnight although the owner delegated questions:
  the plan requires the owner nearby and it spends money. Run-book when the owner says go:
  1. `set -a; source ~/.psm_research.env; set +a` (via a script file; this session's guard refuses `.`/`source` inline),
     then `cargo run -p psm-research-tool -- accept --pieces docs/test-items/beyo_test_samples_1.json --data
     ~/psm-research-bench/a23 --ceiling-usd 40`, then `score-accept --data ~/psm-research-bench/a23`.
  2. Check `scores.json` has both groups and 42 runs before copying tables (R9 review note 3).
  3. Task 5 comparisons: `replay` has no ceiling (R9 review note 1) and `accept`'s ceiling is per folder (note 4):
     budget by invocation count and pass the remainder of USD 40 as the next folder's ceiling.
  4. Record in the evidence doc "Acceptance measurement" and the R9 Review log; relay the tables; owner picks models.
- **Next on track A:** the owner-attended measurement, then R10 (models from its result). Schedule changed
  (master plan v3): R5 and R6 moved to track A because track A was idle and B is busy with R1. Track B after R1:
  merge `feature/agentic` into `research/track-b` first (R4 and plan edits), then R8 once the owner's picks arrive.
- **Track B (owner said yes 2026-11-12):** worktree `~/Desktop/Developer/Photo-Session-Manager-agentic-b`, branch
  `research/track-b`, created at `2cb6454` (R3 approved; before any R4 commit), `npm ci` done.
  **R1 stopped there (exit 10, 18:02): implementer OWNER_DECISIONS_PENDING, 1 card (Google Cloud Vision billing).**
  Both units done and checkpointed (U1 `60f6d6c`, U2 `7b3a2f8`, closing `0446282`); USD 5.82 spent of 50. Not reviewed yet.
  Handoff `docs/implementation/handoffs/implementer/handoff_R1_research_benchmark_implement_r1.md` on track B. Was started
  2026-11-12 15:27. Real paid calls, ceiling USD 50 (plan). Keys `~/.psm_research.env` (0600) and `~/.psm_shopify.env`
  present (presence only). Then R8 on track B (R5, R6 moved to track A). Merge rule (master plan §5): after each approved B milestone
  merge `research/track-b` into `feature/agentic` here; after each approved A milestone merge `feature/agentic` into
  track B before B's next milestone. Track B's copy of this file is not maintained; this one is.
- **Owner answered 2026-11-12 (both "yes"):** a capped run lowers nothing (→ R7·U1 carried note); saving keys or
  Automatic on starts nothing (owner withdrew the "start the open session" pick: no extra work). 08B changelog 7.
- **Blocked on the owner:** R10 (models per stage,
  picked from R9's report). Picks arrive as decision cards in the R1 and R9 narratives.
- **Unverified, watch for it:** this worktree has never been built (first cargo run is a full compile); the three
  template edits (`tools/pipeline/templates/{implement,review,fix}.md`) have never run; the research crate using the
  testkit as a dev-dependency while the testkit depends on it (first exercised in R3 unit 5); the planner assembled
  the 29 units from the earlier A-plans by script and spot-checked only some, so a unit's "What changed" list wins
  over older task wording inside it.
- **Owner preferences for this track:** implementation on Opus (Codex first only if the owner says so); speed over
  ceremony; app tests only where app files changed (flow §10); plain-language narratives per milestone.
