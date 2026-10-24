# Component inventory → shadcn/ui

The prototype is one file, so "component" here means a repeated UI part. The shadcn name is the suggested base; extra libraries are noted where shadcn has no equivalent.

## App shell
| Part | shadcn / library | Notes |
|---|---|---|
| Header bar (56px) | custom flex | Back to Sessions, item type, SKU and ⓘ; camera status, bell, form button, Publish |
| Camera status | `Button` variant ghost + dot | Green dot when connected, red when disconnected; click retries the connection |
| Notification bell + count | `Button` icon + `Badge` (destructive) | Count = failed jobs |
| Notifications drawer | `Sheet` (side right) or `Popover` | Rows: thumbnail, title, step label, `Progress`, Retry |
| Toasts | `Sonner` | Max 3; sticky for errors; action button (Undo, Retry, Open, Review) |
| Theme toggle | `Button` icon | Sessions page only; shadcn `.dark` class |

## Sessions page
| Part | shadcn / library | Notes |
|---|---|---|
| Search + filter button | `Input` + `Button` icon inside | Red dot on the filter button when a time filter is active |
| Time filter | `Popover` + list of options (or `DropdownMenu` with `DropdownMenuRadioGroup`) | Any time, Today, Last 7 days, Last 30 days, Reset |
| Open / Published / All | `ToggleGroup` type single (or `Tabs`) | With counts |
| Unassigned card | `Card` (dashed border) | Up to 5 thumbnails + "+N" |
| Session list | `Table` or CSS grid rows | Columns: checkbox, thumb, product, photos, status, last activity; group headers by date |
| Row checkbox | `Checkbox` | 42px invisible hit area; header box supports the indeterminate state |
| Status pill | `Badge` variants | In progress, Ready · Publish (button), Publishing n% + `Progress`, Publish failed (outline, destructive, clickable), Description failed, Published |
| Hover preview | `HoverCard` | 400px image + caption + properties; flips left near the edge; dims the page behind |
| Failure popover | `Popover` | Reason, progress bar, Retry / Ask the AI agent again, Open session |
| Batch bar | custom fixed bar (or `Toolbar`) | Count, Select all N, Review & publish N, clear |
| New session | `Dialog` + `Command` | Results with thumbnail + note; "Create '…'" item |

## Workspace
| Part | shadcn / library | Notes |
|---|---|---|
| Stage / panel split | `ResizablePanelGroup` | Panel 340px open, 64px folded (`collapsible`, `collapsedSize`); animate the width over 420ms |
| Photo stage | custom | Original/adjusted stacked images with a crossfade; prev/next `Button` icon |
| Swipe gestures | Motion (`drag`, `dragConstraints`, thresholds 120/110px) | Stamps fade in with drag distance |
| Approve / Condition / Delete | `Button` size icon, rounded-full | 56 / 56 / 68px; Approve is primary |
| Original/Adjusted | `Toggle` (pill) | Spinner while processing |
| Item details ⓘ | `Popover` from the header | Property grid + "Edit details →" |
| Photo ⋯ menu / right-click | `DropdownMenu` + `ContextMenu` | Main / Condition, Assign to product…, Move to session…, Copy to session…, Delete |
| Photo grid (Main, Condition) | dnd-kit `SortableContext` (rect strategy) + `ScrollArea` | Number badge, Cover badge, unreviewed dot, Processing, upload state badges |
| Folded strip | dnd-kit (vertical strategy) | 44×33 thumbs, "COND" divider |
| Drag overlay | dnd-kit `DragOverlay` | 1.25× in the strip, rotated 2.5° in the grid |
| Sessions panel (move target) | custom column + `Input` + droppable rows | dnd-kit `useDroppable` per row; highlights on hover |
| Publish popover | `Popover` | Main and Condition thumbnail rows, Product details count, Publish N images |
| Failure notice | `Alert` in a floating card | Auto-hides after 7s, dismissible |

## Product details form
| Part | shadcn / library | Notes |
|---|---|---|
| Container | `Dialog` (max 1120×820) | Header: type, SKU \| barcode (opens a `Popover` editor), Saved, Research chip, Customize fields, close |
| Batch queue strip | `ScrollArea` horizontal + buttons | ✓ when confirmed, n/N filled |
| Steps | `Tabs` (underline style) | Details, Condition, Product; each shows filled/total |
| Media column | CSS grid | Cover spans 2×2; the other set is shown smaller below |
| Field generator | custom `MetafieldInput` | See the mapping below |
| Customize mode | dnd-kit sortable list + `Command` (add metafield) | Grip, name, namespace.key, type, default value, remove |
| Footer | `Progress` + buttons | Save & close, ← previous step, next step →, Publish; batch: Publish N confirmed, Confirm & next |
| AI pill | `Badge` (ai variant) + 3-bar confidence | Opens a `Popover` |
| AI field popover | `Popover` | Confidence, value, reason, source links, Looks right / Clear / See research |
| Research panel | `Sheet` inside the dialog (or a positioned panel) | Timeline of steps; Accept all; focus chip |

## MetafieldInput kinds → shadcn
| Kind | Used for | shadcn |
|---|---|---|
| text | single_line_text_field | `Input` |
| textarea | multi_line_text_field, rich_text_field | `Textarea` |
| choice | ≤4 short choices | `ToggleGroup` (pills) |
| select | >4 choices | `Popover` + `Command` (searchable when >8) |
| radio | choices with " - " descriptions | `RadioGroup` as cards |
| severity | N/A, None, Minor, Moderate, Major | `ToggleGroup` with a severity colour ramp |
| multi | ≤4 short list choices | `ToggleGroup` type multiple |
| multiselect | list choices, metaobject and product refs | `Popover` + `Command` with checkboxes; selected values as removable tags |
| tags | list text without choices, product tags | Tag `Input` + suggestions |
| int | number_integer | `Input` + −/+ buttons |
| decimal / money | number_decimal, money | `Input` (currency prefix) |
| dimension / weight | dimension, weight | `Input` + unit `ToggleGroup` (cm, mm, in / kg, lb) |
| boolean | boolean | `ToggleGroup` Yes/No (or `Switch`) |
| date | date, date_time | `Popover` + `Calendar` |
| color | color | swatches + hex `Input` |
| rating | rating | 5 star buttons |
| url | url | `Input` with link icon |
| files | list.file_reference, file_reference | Thumbnail picker from the session's condition photos |
| aidesc | product description | `ToggleGroup` (AI agent / manual) + `Textarea` |
| unsupported | metaobject_reference and unknown types | Dashed note "Edited in Shopify admin" |

## Theme variables (tokens → shadcn)
- `--background` = `--color-bg`
- `--foreground` = `--color-text`
- `--card`, `--popover` = `--color-bg`
- `--muted` = `--color-surface`, `--muted-foreground` = `--muted`
- `--primary` = `--color-accent`, `--primary-foreground` = `--color-bg`
- `--secondary` / `--accent` = `--accent-tint`
- `--destructive` = `--danger`
- `--border` = `--hair`, `--input` = `--hair-strong`, `--ring` = `--color-text`
- `--radius` = 8px
- Add custom `--ai`, `--ai-foreground` (`--ai-700`) and `--ai-muted` (`--ai-tint`) for the agent layer.
