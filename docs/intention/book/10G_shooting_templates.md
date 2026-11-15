# 10G — Shooting templates (mechanical photo arrangement)

**Status: RATIFIED** (owner David Loorenz, 2026-11-16 00:20, in chat; owner answers to four questions recorded in
§9). Written by the orchestrator from the owner's own description; this chapter is the authority for the feature.
Branch `feature/agentic` (synced with `main` at `23cd3c6`). Deadline: usable on the work Mac on the morning of
2026-11-16.

This is **mechanical**, not an assistant: no model call, no network, no cost. A later assistant (adjustments,
automatic recovery) builds on it and is a non-goal here. It is the "shot template" that 08A §12 deferred, and the
"existing ordered shot template" the raw idea assumed.

## 1. Outcome (the owner's words, condensed)

For each item type the person keeps a **shooting template**: the angles of that type, in **final order** (the
order of the session's Main photos and of the upload), each angle a **container**. The person also records the
**order in which they physically shoot** those angles, which differs from the final order. When a session of that
type is open, its photo area shows the containers **faded and numbered, in final order, waiting to be filled**. As
photos arrive, each fills **the container of its place in the shooting order, by when it was shot, not when it
arrived**: if the third shot arrives before the second, it still goes to the third shot's container, and the second
fills its own container seconds later. A container can carry **framing presets** (for example split frame, and a
zoom-in crop of some amount); a photo that fills it gets them automatically. After the last Main container's shot,
the following photos are **classified Condition automatically**. **Manual changes always win**: the app never
moves a photo back or undoes an edit by itself; it shows a **warning** with a way back to the template ("Back to
template"). Everything in the template editor **saves on every change**.

The owner's example (the acceptance story, §8): a dining-chair template made from uploaded example photos (shown in
black and white so they read as a template, not as the person's photos); container 3 set to split and zoom; the
shooting order set; a new dining-chair session shows the faded containers; shots 1–3 fill 1–3 although shot 3
arrives before shot 2; shot 3 gets split and zoom; after the last Main shot the photos become Condition; the person
moves four condition photos to Main on purpose, sees the warning, ignores it. On the next chair the person
mistakenly puts the split third photo in second place, sees the warning, taps Back to template, and it returns.

## 2. Vocabulary

- **Shooting template** — per item type (the session's `item_type`, 05 / changelog 42): an ordered list of
  **containers** (final order 1…n), a **shooting order** (a permutation of the containers), and per container an
  optional example image and optional **framing presets**. At most one template per item type.
- **Container** — one slot of the final Main order. Shown by number only in the session (the example image is not
  shown there; §5). Its presets are values of the existing composition model (10D MC-27…MC-29): split, shape, a
  centred zoom-in crop amount; nothing outside 10D.
- **Shot index** — the place of a photo among the session's shots in shutter order (§4).
- **Placed by template / placed by hand** — whether a photo's current place (classification, position) and
  composition are the template's; any person action on it makes it "by hand" (§6).

## 3. Settings

- **Settings becomes sectioned**: a side drawer (list of sections) with one page per section; the existing settings
  keep their content and move into sections unchanged. A new section **Shooting templates**. (Owner: "start
  organizing settings into a drawer like domain"; tonight only the drawer and the moves, no redesign of the pages.)
- **Shooting templates section**: choose an item type (the existing item-type list); create its template; **upload
  example photos** (one per container; any image file; stored in the app's data folder, never in the repo; shown in
  black and white everywhere); **drag the containers into final order**; per container set presets (split on/off,
  shape, zoom-in amount); **set the shooting order** by pressing "Set shooting order" and **tapping the containers
  in the order they are shot** (each shows a small second number; can be redone); the number of containers is the
  number of Main containers. **Every change is saved immediately** (no Save button; a failure is shown and nothing
  already saved is lost). A template can be deleted. Templates are local settings of this Mac (not in Shopify).

## 4. Placement on arrival (the rule)

Applies only to a session whose item type has a template; every other session behaves exactly as today (P66:
arrives Main, end of list).

1. **Shot order.** The session's shots are ordered by shutter moment: `captured_at` per 05A MC-4 (direct camera:
   the matched exposure's `exposed_at`; Capture One tether: file mtime, which is sequential there). Where the
   direct camera's file numbering (the camera's filename counter, which increases with every shot) is available
   for the session's deliveries, it decides the order among received files, because an out-of-order Wi-Fi
   delivery can be matched to the wrong exposure by the ledger's FIFO rule (10E). Exposures the direct camera has
   reported for the session whose file has not arrived yet **hold their place**: a later shot that arrives first
   goes to its own container, and the container of the missing shot stays empty until its file arrives. The planner
   verifies which of these signals exist in code and states the exact order key; the outcome above is the contract.
2. **Container.** The photo with shot index *k* (1-based, counting only shots of this session that are not deleted
   by a hand action described in §6 rule 4) goes to the container that is *k*-th in the **shooting order**; its Main
   position is that container's final-order number, with empty containers skipped when the Main list is read (the
   publish/export order is the Main list as today).
3. **Presets.** On filling a container the photo's composition is set to the container's presets (as if the person
   had set them in Compose mode; 10D rules apply, including "needs_review" if a preset no longer fits).
4. **Condition.** A shot whose index is greater than the number of containers is classified **Condition**,
   appended in shot order (as if the person pressed C; 06).
5. **Late earlier shot.** When a shot arrives that belongs before photos already placed by template, those photos
   move to their correct containers (they were placed by the rule, so moving them is not "fighting back"). A photo
   placed by hand is never moved by this rule.
6. Arrival, previews, Capture One, retention, research and publish are otherwise unchanged. Placement is one
   transaction per arrival and emits the existing photo events so the UI updates live.

## 5. The session view

- In a templated session the Main area shows the containers **faded, numbered, in final order**; a filled container
  shows the photo as today. Empty containers are visible placeholders only (not photos; never exported/published).
- An **info button** opens the template (the black-and-white examples with their numbers and presets) so the person
  can check the placement.
- Untemplated sessions look exactly as today.

## 6. Manual changes win; warnings and recovery

1. Any person action on a templated photo — drag to another place, move between Main/Condition, Deny, compose edit,
   move/copy to another session — marks it **placed by hand**. The app never reverts it.
2. A session with photos placed by hand that differ from the template shows a **warning** (unobtrusive, on the
   session's photo area; also a small mark on each differing photo) with **Back to template**: it re-applies §4 to
   those photos (place and presets) as one undoable step (05A MC-30). Ignoring the warning is fine; it stays visible
   until the photos match again or Back to template is used.
3. **Retake (owner answer 2):** dropping a photo onto a filled container puts it there and marks the photo that was
   there **deleted** (red, undoable as today).
4. **Bad shot (owner answer 1, "both happen"):** deleting (Deny) a templated photo **leaves its container empty**
   (later photos do not move), and shows a **"Shift the rest back"** button on that session: pressing it moves
   every later templated photo back one container in shooting order (for the case where the person reshot at
   once), as one undoable step. The button disappears when there is nothing to shift.
5. Undo/redo (05A MC-30) covers the person's actions and Back to template / Shift the rest back; automatic
   placement on arrival is not an undo step (like arrival today).

## 7. Non-goals (tonight)

Any model or image analysis (matching a photo's content to an angle, auto-straighten, duplicate detection) — the
later assistant; per-container shape for Condition photos; templates shared between Macs or stored in Shopify;
editing a template moving photos already placed (a changed template applies to new arrivals; Back to template uses
the current template); a graph editor for the shooting order (owner chose tap-in-order).

## 8. Acceptance (the owner's story as checks)

A1 template editor: create a dining-chair template from uploaded examples (shown B/W), reorder, set container 3 to
split + zoom, set the shooting order by tapping; quit and reopen: all kept. A2 session: a new dining-chair session
shows the faded numbered containers and the info button. A3 arrival: shots 1, 3, 2 arriving in that order end in
containers by shooting order, shot 3 with split + zoom. A4 after the last Main container's shot, photos arrive as
Condition. A5 moving four Condition photos to Main shows the warning and nothing reverts. A6 dragging the split photo
to place 2 shows the warning; Back to template returns it (and undo restores the hand placement). A7 Deny leaves the
container empty; Shift the rest back moves later photos back one. A8 retake dropped onto a filled container: the old
one is deleted (red). A9 a session whose item type has no template behaves exactly as before (P66).

## 9. Owner answers (2026-11-16 00:20)

1. Bad shot: **both happen** → §6 rule 4. 2. Retake onto a filled container: **old one deleted** → §6 rule 3.
3. Shooting order editor: **tap in order** → §3. 4. Delivery: when built and all checks pass, **join into `main` on
GitHub** for the work Mac, as with research.

## 10. Process for this chapter (owner: "open for you to modify the flow")

No inventory gate and no projection; the planner writes the plans straight from this chapter; Codex implements,
Opus reviews, one fix round; the orchestrator chains the phases overnight and joins into `main` when all are
approved and the full checks pass.
