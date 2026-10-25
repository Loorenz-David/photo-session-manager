# 05A — Mechanism contracts: core (identity, arrival, order, state, naming, values)

Written by the mechanism inventory (2026-10-25, round 1). Each contract is the authority for its
mechanism; chapters 05–08 remain the authority for meaning. Every contract names its invariant,
registered in `14_measurement_ledger.md` as an `M8+` entry: a criterion row cites the `M` id or
the `MC` id. Publish contracts are in `07A_publish_contracts.md`; Capture One contracts in
`10A_capture_one_contracts.md`. Items marked **(ratify Rn)** resolve a contradiction in the
ratified text; the list is in the inventory handoff and in chapter 16 entry 14.

Conventions for every contract: "canonical" means the one form stored and compared; all
comparisons are on canonical forms, byte for byte, unless the contract says otherwise.
`MECHANISMS_VERSION = 1` is stored in `settings` at first start; a later change to any contract
that alters a stored canonical form bumps it and ships a migration.

---

## MC-1 — Path canonicalization and RAW identity

**Inputs.** RAW paths arrive from four producers, in different shapes: the capture-done hook argv
(text), the processing-done hook argv 2 (text), FSEvents (UTF-8 bytes, may be NFD), and user
import (file picker or drag; may traverse symlinks). Capture One returns paths through `/private`
when the input went through a symlink (evidence: `/tmp/x` reads back as `/private/tmp/x`).

**Canonical form** `canon(p)`:
1. absolute; relative input is an error, never resolved against a working directory;
2. the **parent directory** resolved to its on-disk path: symlinks and `..` removed and every
   component in its on-disk spelling (on macOS `fcntl(F_GETPATH)` on an open handle of the
   directory gives exactly this); the file name itself is not resolved through a symlink (a
   symlinked RAW is identified by its link location);
3. the whole string normalized to Unicode **NFC**;
4. no trailing slash; no case folding. The file-name component is replaced by the spelling of
   the matching entry in the parent directory listing (the on-disk spelling), so two inputs
   differing only in case on a case-insensitive volume converge to one string.

**Identity.** A RAW is identified by `canon(raw_path)`. `photos.raw_path` stores only canonical
paths. The `photos` uniqueness key is `(session_id, canon(raw_path))`.

**Invariant (M8).** For any two producer inputs that name the same file (NFC vs NFD spelling,
via `/tmp` vs `/private/tmp`, differing case on a case-insensitive volume), `canon` returns the
same string; the arrival pipeline therefore creates at most one row per (session, file).

---

## MC-2 — Capture arrival: candidate filter, stability, session attribution, row creation

**Capture root.** The document's capture root is the `file` of the document collection whose
`id` is `capture` (evidence: stable, independent of the current `captures` value, which we
change per session). Per-session capture folders are `<capture root>/<folder_key(SKU)>/`
(`folder_key` in MC-8). When no session is active the app sets the capture folder to
`<capture root>/_Unassigned/` **(ratify R-I1)**; `_Unassigned` cannot collide with a folder key
because folder keys never start with `_` (MC-8).

**Candidate filter** (watcher and startup scan; hook paths pass the same filter):
a candidate is a regular file whose parent directory is the capture root or exactly one level
below it; whose name does not start with `.`; that is not inside any directory named
`CaptureOne` or starting with `.` (Capture One writes captures through `Capture/.capture/` and
keeps settings and caches under `CaptureOne/`); whose extension, compared case-insensitively,
is in the camera adapter's declared RAW extension set (Sony: `arw`). Everything else is
ignored without a `capture_events` row.

**Stability.** A hook-delivered path is complete (Capture One calls the hook after writing). A
watcher or scan candidate is processed only when two `stat` reads at least 1 s apart return the
same size and mtime; otherwise it is re-checked every 1 s, giving up after 120 s with a
`capture_events` error row.

**Session attribution (ratify R-I1).** The session is decided by the RAW's parent folder, not by
the event's processing time, because the folder records which session was active when the
shutter fired (D1 switches the folder with the session):
1. parent = `<capture root>/_Unassigned/` → Unassigned;
2. parent = `<capture root>/<k>/` and exactly one session's item has `folder_key(sku) = k` →
   that session (if that session is `published`, the photo still joins it and makes it dirty;
   captures are never redirected);
3. otherwise (capture root itself, an unknown folder, an import) → the session that is active
   when the event is processed, or Unassigned if none.

**Row creation.** Each hook call, watcher event and scan hit writes one `capture_events` row
(raw, append-only). Then, in one SQLite transaction: if a `photos` row with
`(session, canon(path))` exists, nothing else happens; otherwise insert it with
`classification = unreviewed`, `position = null` (MC-5), `captured_at` per MC-4,
`source = capture` (or `import` for the import path), and enqueue original-preview extraction
(MC-20), hashing (MC-3) and the adjusted-preview render (MC-16). The transaction is the only
dedupe; hook and watcher may race freely.

**Startup reconciliation** scans the capture root and each of its immediate subfolders (not
only the current `captures` folder) **(ratify R-I16)**.

**Move and copy.** A move re-parents the row; a copy inserts a second row. Both are refused,
changing nothing and telling the user, when the target already has a row on the same RAW
**(ratify R-I12)**, or the target's status is `published`. Moving rows **out of** any session,
including a published one, is allowed (S-K) **(ratify R-I3)**.

**Invariant (M9).** For every RAW the candidate filter accepts, the number of `photos` rows
created by arrival is exactly one, in the session given by the attribution rule, whether the
hook, the watcher, the startup scan, or any combination delivered it, in any order; files the
filter rejects create none.

---

## MC-3 — RAW hash

**Algorithm.** SHA-256 over every byte of the file, streamed in chunks, file opened read-only
(`O_RDONLY`; no write handle to a RAW is ever opened, H2). Canonical form: 64 lowercase hex
characters. Computed **after** stability (MC-2) and only once per canonical path: rows on the
same RAW (copies) share the value; a second row copies it instead of re-hashing.

**Checks.** The hash is re-verified (a) in `prepare` for every photo in the run's frozen lists,
before its file is rendered, and (b) on demand from the health screen. A mismatch marks the photo
`raw_changed`, fails the publish item at `prepare` with that reason, and never overwrites
`raw_hash`. A missing file marks `raw_missing` the same way.

**Invariant (M10).** `raw_hash` equals SHA-256 of the file bytes at stability time; it is never
recomputed in place; a later mismatch is surfaced as `raw_changed` and blocks that photo's
export and publish.

---

## MC-4 — Time

- **Storage.** Every timestamp is a UTC instant stored as integer **milliseconds since the Unix
  epoch** (`*_at` columns). No local-time strings are stored.
- **captured_at** = the RAW file's mtime at first stability check. EXIF capture time is **never**
  used: the camera writes local wall time with no zone (evidence: EXIF time 2 h 0 m 30 s ahead of
  the file mtime on the dev Mac).
- **Durations and timers** (15 s preview delay, undo window, stability checks, job backoff) use
  the monotonic clock, never wall-clock differences.
- **Sessions-list grouping** is computed at render time in the Mac's current time zone on
  **calendar dates**, never by subtracting 24 h (a DST change day has 23 or 25 hours):
  Today = local date of `last_activity_at` equals today; Yesterday = today − 1 calendar day;
  This week = same ISO-8601 week (Monday start) and not Today/Yesterday; Last week = previous ISO
  week; older = one group per calendar month, newest first. Within a group: `last_activity_at`
  descending, then session id descending.
- **last_activity_at** is set to now by: any photo change (MC-6), any attribute edit, and a
  publish item reaching `done` or `failed`.
- **Ordering ties** anywhere the book orders by time break by the row's integer id ascending.

**Invariant (M11).** Grouping of a fixed set of `last_activity_at` instants is identical for
every evaluation instant inside the same local calendar day, including DST change days; no
stored instant depends on the camera clock.

---

## MC-5 — Order and position math

**Groups.** `main` and `condition` per session. Within each, `position` is 1-based and
contiguous (`1..n`). `unreviewed` and `deleted` photos have `position = null` **(ratify R-I2:
chapter 06's "joins at the end of Main" is a display rule)**: the UI shows unreviewed photos
after the last main photo, ordered by `captured_at`, then id.

**Operations** (one SQLite transaction each; `n_g` = size of group g before the operation):
| Operation | Effect |
|---|---|
| classify into g (from unreviewed or from the other group) | remove from old group if positioned (below); `position = n_g + 1` in g |
| remove photo at position k from g (to deleted, unreviewed, or other group; move out) | every photo in g with position > k gets position − 1 |
| reorder within g from i to j (`j` clamped to `1..n_g`) | i < j: positions i+1..j get −1; i > j: positions j..i−1 get +1; photo gets j |
| move or copy into another session | same classification in the target, appended at the end of its group (moved/copied set appended in its source order: group, then position, then `captured_at`); unreviewed stays null |
| delete | remember `(prior classification, prior position k)` as the session's single undo slot (replacing any previous slot); remove as above |
| undo (only the slot; valid until the next delete) | classification restored; if positioned, insert at `min(k, n_g + 1)`, photos at ≥ that position get +1; photo becomes active; slot cleared |

Drag between groups uses "classify into g" (end of target), whatever the drop index (chapter 06).
Provider sort order is never read.

**Invariant (M12).** After any sequence of the operations above, each (session, group) holds
positions exactly `1..n` with no gaps or duplicates, `unreviewed`/`deleted` rows hold null, and
undo of the last delete restores the prior classification and the prior index (clamped).

---

## MC-6 — Change sequence and dirty

`sessions.change_seq` is an integer incremented by 1 in the same transaction as any **photo
change** (add, classify, reorder, move in or out, copy in, delete, undo). `dirty` is **derived**:
`dirty = published_at is not null and change_seq > published_seq`, where `published_seq` is the
`change_seq` recorded in the publish item at confirmation (07A MC-10) and copied to the session
when that item reaches `done` **(ratify R-I13)**. A change made while a publish is running
therefore keeps the session dirty after that publish completes. Attribute edits do not change
`change_seq` (chapter 07). Pending owner card I1 (Capture One retouch after publish) would add
one event to this list.

**Invariant (M13).** A session whose photos changed after the confirmation of its latest
completed publish is dirty; one with no photo change since that confirmation is not.

---

## MC-7 — Session status and row label

**Inputs.** The session's photos, `published_at`, `dirty` (MC-6), and its **latest publish
item** = the item for this session whose run has the greatest `confirmed_at`, ties broken by
greater run id. The Unassigned session has no status (it is shown as the Unassigned card).

**Status** (chapter 07, first match wins; total over all states):
1. `publishing` — latest item's step ∈ {prepare, upload, update, describe} (queued items count).
2. `failed` — latest item's step = `failed` and `published_at` is null.
3. `published` — `published_at` not null and not `dirty`.
4. `ready` — at least one `main` photo, no `unreviewed` photo, and every non-deleted photo has
   `adjusted_preview_path` not null (a *stale* preview still counts, MC-17).
5. `in_progress` — otherwise, including an empty session.

**Row label** (separate function; chapter 07's precedence list) **(ratify R-I7)**:
`Publishing n%` if status is publishing → `Publish failed` (+ Retry) if the latest item is
`failed` at a step other than `describe`, whether or not the session was published before →
`Description failed` if failed at `describe` → `Published` → `Ready · Publish` →
`In progress` (+ "n to review" = count of unreviewed). Progress `n%` = prepare 0–10, upload
10 + 65 × uploaded/total, update 75–85, describe 85–100, floored to an integer.

**Invariant (M14).** For every combination of (latest item step ∈ the six steps or none,
`published_at` set or null, dirty, main count 0 or >0, unreviewed present, preview missing),
status equals the first matching rule and the row label equals the first matching label.

---

## MC-8 — SKU folder key and export file naming

**folder_key(SKU)**: NFC-normalize; replace every character outside `[A-Za-z0-9._-]` with `_`;
replace a leading `.` or `_` with `-`; the result must be 1–100 characters. Two items whose
folder keys are equal **ignoring case** cannot coexist (the default macOS volume is
case-insensitive): creating or linking the second is refused with a message naming the first
**(ratify R-I11)**.
The folder key names both `Capture/<key>/` (MC-2) and the export folder.

**Export layout** (M7): `<export folder>/<key>/<key>_<NN>_<main|condition>.jpg` plus
`<export folder>/<key>/item.json`. `NN` is the photo's position, zero-padded to
`max(2, digits(n_g))` for its group. The export is written into
`<export folder>/.<key>.partial-<run id>/` and then swapped in place of `<key>/` with renames;
the previous `<key>/` is removed only after the swap succeeds, so no stale file from an earlier
export with more photos survives (M7 "exactly").

**item.json**: UTF-8, keys sorted, two-space indent, trailing newline:
`{"schema": 1, "sku", "barcode", "title", "item_type", "attributes": {key: canonical value
(MC-9)}, "main": [file names in order], "condition": [file names in order]}`. Absent values are
omitted, never written as empty strings. Deleted and unreviewed photos appear nowhere.

**Invariant (M15).** For any session, the export folder holds exactly one file per main and
condition photo named by the rule, no other image file, and an `item.json` whose lists match the
file names in order; two items never share a capture or export folder.

---

## MC-9 — Attribute values: core canonical forms and equality

Values are stored in `attribute_values.value` as JSON in the canonical form of the definition's
core type. Equality (for "changed?", diff writes in 07A MC-13, and AI-vs-human checks) is byte
equality of canonical JSON.

| Core type | Canonical JSON | Notes |
|---|---|---|
| text, textarea, url, color | string | NFC; text and textarea trimmed of trailing whitespace only; color `#rrggbb` lowercase |
| rich_text | string, paragraphs separated by one blank line | formatting the core cannot represent makes the field read-only (adapter) |
| integer | JSON integer | range of i64 |
| decimal | string matching `-?\d+(\.\d+)?`, no exponent, no trailing zeros after the point | never a float |
| money | `{"amount": decimal string, "currency": ISO-4217 upper}` | |
| dimension, weight | `{"value": decimal string, "unit": unit code}` | unit codes are the adapter's, listed in the definition |
| boolean | `true` / `false` | |
| date | string `YYYY-MM-DD` | a date-time from the platform becomes a date only if its definition is `date`; else `reference` |
| rating | `{"value": decimal string, "min": decimal string, "max": decimal string}` | |
| choice | string, exactly one of `choices` | case-sensitive; numbers in choices stay strings (`"1"`…`"8"` in the shop) |
| multi_choice, tags | array of strings, de-duplicated, in the user's order | |
| media_list | `{"excluded": [photo ids]}` | 07A MC-14 |
| reference | the platform's raw value, opaque | never written |

A value read from the integration that does not fit its canonical form (or a `choice` value not
in the current choices) is stored verbatim with `valid = false`, shown, and never written back
unless the photographer edits it.

**Severity set** (field kind): a `choice` whose choices are exactly the five strings `N/A`,
`None`, `Minor`, `Moderate`, `Major` (case-sensitive, any order, no others).

**Invariant (M16).** Reading a value from the integration and writing it back unchanged produces
no write (canonical equality); numeric-looking choices are never coerced to numbers.
