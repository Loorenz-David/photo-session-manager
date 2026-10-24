# 05 — Domain model

Ownership: **U** photographer (UI), **S** system, **P** imaging provider (read by us; written only
through its API), **I** integration adapter (read by us; written only inside a confirmed publish
run), **AI** agent (post-MVP; never overwrites a U value).

## items
| Field | Owner | Notes |
|---|---|---|
| id | S | |
| sku | U/I | required, unique; immutable once a photo exists (it names folders and files) |
| barcode | U/I | optional |
| item_type | U/I | display name in lists ("Lounge Chair"); drives attribute presets |
| title | U/I/AI | product title |
| origin | S | `linked` (found on an integration) or `local` (created here) |
| created_at, updated_at | S | |

## item_links (one per item per integration — the only place external IDs live, H7)
item_id, integration (`shopify`), external_id, external_url, last_synced_at, create_pending
(true until the adapter creates it).

## sessions
| Field | Owner | Notes |
|---|---|---|
| id | S | |
| item_id | S/U | null only for the built-in Unassigned bucket |
| published_at | S | null until the first successful publish |
| dirty | S | true after any photo change since the last publish |
| last_activity_at | S | drives list grouping |
Status is **derived**, never stored (chapter 07).

## photos (one row per RAW per session; a copy to another session is a second row on the same RAW)
| Field | Owner | Notes |
|---|---|---|
| id | S | |
| session_id | S/U | reassignable by move/assign until published |
| raw_path | P | absolute path; unique together with session_id; never changed by us |
| raw_hash | S | SHA-256 computed once (M6) |
| provider_ref | P | Capture One variant id |
| original_preview_path | S | embedded camera JPEG, written by us once |
| adjusted_preview_path | S | provider's preview-recipe output; null while "Processing" |
| classification | U | `unreviewed` / `main` / `condition` / `deleted` (chapter 06) |
| position | U/S | 1-based contiguous within (session, main) and (session, condition); null otherwise |
| captured_at | P | file mtime at arrival |
| source | S | `capture` or `import` |

## photo_links (per photo per integration; H7)
photo_id, integration, external_media_id or external_file_id, role (`gallery` / `attribute:<key>`),
upload_state (`none` → `exported` → `staged` → `uploaded` → `attached` → `ready` / `failed`), error.

## attribute schema and values (chapter 08 defines semantics)
- `attribute_definitions`: integration, key, label, type, choices (JSON), validations (JSON),
  group (`details` / `condition` / `product`), description.
- `attribute_presets`: item_type, group, ordered keys with default values (Customize fields).
- `attribute_values`: item_id, key, value (JSON), provenance (`human` / `ai` / `default`),
  ai_confidence (`high` / `medium` / `low`), ai_status (`suggested` / `accepted` / `cleared`),
  ai_reason, ai_sources (JSON), updated_at. A `human` value is never overwritten by AI.

## publish_runs and publish_items (H1, M5)
`publish_runs`: id, integration, created_at, confirmed_at, confirmed_summary (exact text shown).
`publish_items`: run_id, session_id, options frozen at confirmation (create_item, title), step
(`prepare` / `upload` / `update` / `describe` / `done` / `failed`), progress (uploaded, total),
error, finished_at.

## jobs, capture_events, settings
- `jobs`: durable in-process queue: type (`preview` / `export` / `publish_step` / `verify`),
  target, status, attempts, next_attempt_at, error.
- `capture_events`: every provider hook call and watcher event, raw (M1 proof).
- `settings`: key/value, including the active session id, provider document path, recipe and
  style names, integration config, export folder.

## Facts vs derived
Stored facts: paths, hashes, provider refs, classification, position, external IDs, confirmation
text, job and step states, attribute values with provenance. Derived and recomputed, never stored:
session status, "n to review", "ready" counts, form fill counts, the publish summary (frozen only
into the run at confirmation). Missing data is never inferred.
