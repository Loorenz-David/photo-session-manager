# Source evidence — Claude Design handoff (2026-10-24)

Location: `claude_design/` (README, NOTES, COMPONENTS, 36 desktop + 4 narrow PNGs, prototype
source, tokens, Shopify metafield reference). This file records the rules the book depends on,
read from `NOTES.md` and `source/session-manager.html`.

## Session status (prototype `status()`)
`publishing` if a job is in prepare/upload/update/describe → `failed` if the job failed and the
session was never published → `published` if published and not dirty → `ready` if any main photo
and every photo is adjusted and reviewed → else `in_progress`.
Any photo change sets `dirty`. Publish completion clears `dirty`, sets `published`.

## Review
Actions: approve (A, swipe right ≥120 px), condition (C, swipe up ≥110 px), delete (⌫, swipe
left). Next unreviewed opens after an action. Delete is instant; toast "Photo deleted · Undo ⌘Z"
4.5 s; ⌘Z until the next delete; only the last delete undoable; undo restores position and makes
the photo active. Condition photo's middle button = "Move to Main". Main 01 = cover.

## Original / Adjusted
One pill per session view, Space toggles, 700 ms crossfade, spinner while processing, adjusted
fades in automatically.

## Publish
Popover: main thumbs in order, condition thumbs, product-details count, note "N new photos
haven't been reviewed and will publish in Main" (book decision N2 may change this), button
"Publish N images". Steps: Creating Shopify product | Preparing images → Uploading n of N →
Updating Shopify product → AI agent is writing the description → Published. Progress fractions:
prepare 0–10 %, upload 10–75 %, update 75–85/100 %, describe 85–100 %. Failure keeps uploaded
images; retry re-runs the failed step only; per-image badges On Shopify / Not uploaded / Uploading.
Batch: tick rows → "Review & publish N" → form queue → "Confirm & next" → "Publish N confirmed".

## Sessions list
Grouped by last activity (Today, Yesterday, This week, Last week, months). Filters Open (default,
everything not published) / Published / All with counts. Search by title or SKU. Time filter
popover. Unassigned card on top only when unassigned photos exist. Hover preview card.

## New session
Search Shopify by title or SKU; "Create '…'" row creates a local item, product created on first
publish; the prototype assigns a new SKU `VF-nnnn` (real rule: owner's SKU scheme, TBD in settings).

## Form
Steps Details / Condition / Product with filled/total counts; media column with cover 2×2;
values save as typed; Customize fields per item type and step (reorder, remove, add from all shop
metafields, default values; Title/Description/Tags not removable); SKU and barcode editable in
the header; tags with shop suggestions; description "AI agent writes it" (default) or "Write
manually".

## Field kind map (prototype `KIND`)
single_line_text_field: no choices → text; choices == severity set → severity; any choice > 32
chars → radio cards; > 4 choices or joined length > 48 → select; else choice pills.
list.single_line_text_field: no choices → tags; > 4 or joined > 48 → multiselect; else multi
pills. multi_line/rich_text → textarea; number_integer → int; number_decimal/volume → decimal;
dimension, weight, boolean, date/date_time → date, color, rating, money, url; description →
aidesc; tags → tags; list.metaobject_reference / list.product_reference → multiselect (book:
`reference`, read-only); list.file_reference / file_reference → files. Unknown → unsupported.

## AI (UI states only in the MVP)
Pill with 3/2/1 confidence bars; Low reads "Check"; popover with reason, sources, Looks right,
Clear, See research; human edits win; Accept all skips Low; research panel timeline with focus.

## Keyboard
A, C, ⌫, ⌘Z, ←/→, Space, F (fold panel), I (item card), D (form), S (capture, prototype),
N (new session), Esc (close topmost), ↑/↓/Enter in lists.

## Tokens and components
`tokens/tokens.css|json`: light/dark colours (bg, surface, text, muted, stage, hair, accent,
danger, ai, ok), Archivo type scale 11–17, weights 400–800, spacing, radii (8 px base), shadows,
motion. `COMPONENTS.md` maps every part to shadcn/ui plus dnd-kit (sorting, droppables), Motion
(swipe), Sonner (toasts), and gives the token → shadcn theme variable map.

## Shop reference
`reference/SHOPIFY_PRODUCT_METAFIELDS.md`: 68 product metafield definitions read on 2026-10-24
(API 2026-01). Condition fields are `custom.condition_*` (severity set), `custom.damage_report`
(`list.file_reference`, images), `custom.condition_report` (text), `custom.detailed_condition`
(cards). No `custom.condition_images` definition exists.

## Not in the handoff
Phone layout (explicitly desktop only). Settings screen. Health/error states beyond camera
disconnected and publish failures. Export-without-integration state.
