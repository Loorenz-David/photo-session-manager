# Owner ideas (research track)

## Cost reduction: light local parsers and fewer model calls (owner 2026-11-13)

The owner asks for local tools (light parsers) that cut what the agent sends to the API. Measured on the 15 finished
pieces of the first measurement (`~/psm-research-bench/a23`, published group), per piece: observe USD 0.101 (32 %,
one Sol call per main photo, ~0.011 each), read 0.081 (26 %, 166 s, the slowest step), compare 0.071 (23 %, one
image call per candidate page), rest ≤ 0.02; total ≈ 0.31, 4–7.5 min. Candidate levers, to size after the full
measurement and the model comparison:

1. Read: parse the page locally first (JSON-LD / Shopify product JSON / og tags / the product block) and send only
   that, or skip the model when structured data names the field outright.
2. Observe: send a few chosen main photos, or several photos in one call, instead of one call per photo.
3. Compare: drop candidate pages locally (title or text clearly another kind of piece) before the image comparison.
4. A cheaper model per stage (Luna for read was near Terra in R1's matrix at about 1/8 of the cost).
