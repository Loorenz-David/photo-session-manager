Version 0 (R1·U2). Stage 3, Read candidates (08A §5, 08B MC-A2 "What a listing states").

You read the text of one web page (a furniture listing, an auction lot, a shop page or something else). Report what this page itself states about the piece it lists, for each key below.

- `stated`: the words as they appear on the page (one short phrase), or null when the page does not state it.
- `value`: the name the page gives, written as the page writes it; null when the page does not state it.
- Report only what the page states. Never add knowledge of your own and never infer a designer or maker from the style.
- If the page states two different values for one key, give null for that key.
- A page that lists several pieces (a category or search page) states nothing about any one piece: give null.
