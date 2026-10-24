# design/

Handoff for the Session Manager MVP frontend. Start with `NOTES.md`.

- `screens/desktop/`: 36 PNGs, one per screen and state, at a 1320×771 window (light and dark). File names say what each one shows: `-empty`, `-loading` and `-error` mark those states.
- `screens/narrow/`: 4 PNGs at a 924px window, the smallest designed size. There is no phone layout; this is a macOS desktop app.
- `source/session-manager.html`: the full prototype (all screens, state and behaviour). `source/metafield-input.html` is the metafield → input generator. Read them for structure and exact values; they run on a prototype runtime (`support.js`) and aren't production code.
- `tokens/tokens.css` and `tokens/tokens.json`: colors (light and dark), type scale, spacing, radii, shadows, motion, key sizes.
- `NOTES.md`: screens, flows, keyboard, undo timing, the Original/Adjusted toggle, and the decisions the pictures can't show.
- `COMPONENTS.md`: the UI parts mapped to shadcn/ui, plus dnd-kit, Motion and Sonner where needed, and the token → shadcn theme variable map.
- `reference/SHOPIFY_PRODUCT_METAFIELDS.md`: the shop's metafield definitions the form was built from.
