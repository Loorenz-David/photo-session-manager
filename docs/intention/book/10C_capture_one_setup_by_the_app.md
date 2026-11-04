# 10C — Capture One set up by the app (owner decision 2026-11-03)

Owner decision 2026-11-03 (chapter 15 R36, chapter 16 entry 24), when preparing the move to the work
Mac: **Session Manager sets up and keeps up everything it needs inside Capture One — its own
Capture One session document, the two process recipes, the next-capture style — so the photographer
only shoots, retouches when needed, and publishes.** Owner's words: "what I'm looking for is
Session Manager managing all that infrastructure and configuration so that the photographer only
focuses on take picture, retouch if needed, upload." Choices: "its own file" (not an existing
session), and when Capture One has another document open, "open ours alongside".

This chapter is the authority on provisioning the editing side. Chapter 10 (startup), 10A (MC-15
hooks, MC-18 capture folder, MC-19 addressing) and chapter 12 (settings, health) point here.

## What the photographer still does (not automatable)
Install and license Capture One Pro; answer macOS's one-time Automation prompt ("Session Manager
wants to control Capture One"); connect and power the camera; create the house style itself if
they want one (a creative choice; the app applies a style by name, it never makes one). Health names
each of these in plain words when it is the missing piece.

## How products are divided inside Capture One
One Capture One **session document** holds every product. A Capture One session is a folder on disk
(`<name>.cosessiondb` plus the folders `Capture`, `Selects`, `Output`, `Trash`) whose images are plain
files in those folders (a catalog, the other document kind, is a database referencing files anywhere;
the app does not use one). Each Session Manager item has its own folder `Capture/<folder_key>`
(05A MC-2, 05B MC-22); Capture One shows them as session folders in its Library. No change to that
rule.

## MC-26 — Provisioning the editing document

**Setting.** `document_id` keeps naming the session document by path. **Missing or null → the app's
own document** `~/Pictures/Session Manager/Session Manager.cosessiondb` (folder
`~/Pictures/Session Manager/`). Settings shows it as "Capture One session: Session Manager (created
by the app)" with "Use another session…" (pick an existing `.cosessiondb`; then that document is
provisioned the same way). Kind must be `session` (chapter 10).

**Ensure, step by step,** at startup, after a settings save, on Health "Retry checks", and before a
Session Manager session is activated (MC-18). Each step is idempotent; the first failing step stops
the sequence with its own health item; nothing blocks review, export or the rest of the app (H6).
1. **Capture One running.** If not, launch it and wait until it answers scripting (bounded wait).
   Not installed, not Pro, or Automation denied → health item naming that, no further steps.
2. **Document exists.** If no document is at the path, create it with Capture One's `make new
   document` (kind session) at that location. Never create over an existing folder that is not a
   Capture One session (health item instead).
3. **Document open.** If it is not among Capture One's open documents, open it **in a window of its own**
   (Capture One's `open` replaces the front document's window — evidence VERIFY-14; the app first makes a throwaway
   document, which opens in a new window, opens its own into that window, and deletes the throwaway folder). **Other open
   documents are never closed, saved, switched or modified.**
4. **Recipes.** `PSM Preview` and `PSM Web` (names from settings) exist with the
   properties the app depends on: JPEG, sRGB, long edge 1600 / q80 (preview) and 2048 / q90 (web),
   output location as the render contract needs (MC-16), **disabled** so the photographer's own
   batch processing never runs them. Missing → created; a dependent property drifted → set back;
   other properties are left alone. Recipes with other names are never touched. Capture One's recipes are application-wide (evidence
   VERIFY-14), so these two are shared by every document the photographer opens; being disabled, they are inert there.
5. **Next-capture style.** If `style_name` is set and Capture One lists it, the document's next
   capture settings apply that style; if it is not listed, health "Style '<name>' not found in
   Capture One" (capture still works). Empty `style_name` → the next-capture settings are left as
   the photographer set them. **Default empty** (owner 2026-11-04, changelog 26): a fresh setup applies no style
   and Health starts green; the photographer names a style in Settings only if they want one.
6. **Hooks** as MC-15 (unchanged).

**Front document.** When a Session Manager session is activated (MC-18), the app brings its document
to the front in Capture One before setting the capture folder, because tethered captures go to the
foreground document. The photographer's other documents stay open behind it.

**Grounding to verify before relying on it (VERIFY-13/14, evidence file):** `make new document`
creating a session at a given path and name; two session documents open at once and which one a
tethered capture lands in; how to bring a document to the front by script. **If Capture One cannot
keep two documents open, the implementer stops and the owner decides** (the "alongside" choice
assumed it can).

## Invariant
**M32** (chapter 14): after a successful ensure, the configured session document exists, is open
and frontmost when a session is active, holds both recipes with the dependent properties, and no
other Capture One document or recipe was closed, switched or edited by the app.
