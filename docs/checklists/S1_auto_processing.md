# S1 — Auto-processing on arrival (photographer, no code)

Goal: every new capture arrives already styled, masked and cropped, and two process recipes exist
with the names the app expects.

## House style
1. Pick one well-shot RAW. Adjust until it is the house look (exposure, white balance, levels, clarity…).
2. Adjustments tool tab → Styles and Presets → "…" → Save User Style. Name: **PSM House**
   (or your own name; write it here: `________________`).

## AI masks and AI Crop on arrival
3. Capture tool tab → **Next Capture Adjustments**:
   - Styles: add **PSM House**.
   - Adjustments: "Copy from last" or "Defaults" — choose and note here: `__________`.
4. Enable AI Crop / auto-alignment for next captures if your version exposes it in Next Capture
   Adjustments; otherwise note where it lives: `__________________________`.
5. Shoot 5 frames. Each must arrive styled; masks recalculated per image; crop applied.
   Frames OK: ____ / 5.

## Process recipes (Output tool tab → Process Recipes)
6. Recipe **PSM Preview**: JPEG, quality 80, sRGB, scale long edge **1600 px**,
   Root folder: Output location, Sub folder: `_preview`, Existing files: **overwrite**.
7. Recipe **PSM Web**: JPEG, quality 90, sRGB, scale long edge **3000 px** (Shopify limit is
   20 MP, 3000 px is plenty for listings; change if you want), Root folder: Output location,
   Sub folder: empty, Existing files: **overwrite**.
8. Leave both recipes **disabled** in the recipe list (the app triggers them by name).
9. Select one frame, right-click → Process with → PSM Preview. Confirm a JPEG appears in
   `Studio/Output/_preview/`. Path of the file: `_______________________________`.

## Record
| Item | Value |
|---|---|
| Style name | |
| Preview recipe name / folder | |
| Web recipe name / folder | |
| 5-frame arrival test | / 5 |
| Shot-to-styled-preview time | s |

These names go into `.env` as `CO_STYLE_NAME`, `CO_PREVIEW_RECIPE`, `CO_WEB_RECIPE`.
