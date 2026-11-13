# Phase R1_research_benchmark: photo search, page reading and AI models measured on your 42 pieces

## What you can do now
You now have measured numbers for the three choices the research agent needs from you. Every one of your 42 test pieces was searched by its first photo through both photo-search routes: Google Lens through SerpApi, and Google Cloud Vision. Ten listing pages (one per trusted site, plus your shop) were read two ways. Three OpenAI models were compared on three research steps, with accuracy, cost and time per piece: what the cover photo shows, what a listing page states, and how worn the piece is. Nothing in the app changed. This was measuring only, with real calls, and it used USD 5.82 of the USD 50 allowed.

## Try it yourself
1. Read **Benchmark 1** (photo search) in the evidence document:
   ```
   open -e ~/Desktop/Developer/Photo-Session-Manager-agentic/docs/intention/evidence/research_providers.md
   ```
2. Open the full report and go to **Check sheet** (pieces 1–4, 21–24 and 31–34). For each route, mark whether the right listing really is among the first ten results:
   ```
   open -e ~/psm-research-bench/a01/report.md
   ```
3. In the evidence document, read **Page reading**: which of your sites a plain download can read, and which only OpenAI's page tool can read.
4. Read **Benchmark 2** in the evidence document. The full table, with every wrong answer, is here:
   ```
   open -e ~/psm-research-bench/a02/report_models.md
   ```
5. To see the tool's safety checks run with made-up data (spending limit, call limits, keys never written to disk), with no real calls:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-research-tool
   ```
6. Answer the three picks under **Needs you**.

## Choices made for you
- The three models compared were gpt-6.1-sol, gpt-6.1-terra and gpt-6-luna. gpt-6-astra was left out because of its price. Say so if Astra should be measured too.
- Prices were taken from OpenAI's pricing page on the day. Cached-write tokens were charged at their own (higher) rate.
- For the spending limit, each call was budgeted at its worst possible cost before it was sent. A call that got no reply was charged that worst case.
- The two search services counted as free (both were inside their free monthly allowance). Only their call limits applied.
- A result "names" the piece when its title or page contains the designer or maker, ignoring capitals, spaces and punctuation. Accents count, so Wørts is not Wörtz.
- Piece 26 has no maker in its fields, so the "most likely by Holger Christensens Møbelmagasiner" from its description was scored separately.
- Lens marked none of its results as exact copies of your photo, so no extra exact-copy search was bought.
- Lens searched as from Sweden, in English.
- On a re-run, a call the provider had refused (wrong key, billing, too many requests, server error) was tried again. Every other recorded call was reused, not repeated.
- Your shop's address for scoring is beyovintage.se.
- In the model comparison, the photo step was scored on wood type, the page step on designer and maker, and the condition step on your condition fields.
- The wood and condition choices given to the models are your live shop's own choices, not the test store's.
- The "best model per step" is the one with the most right answers minus wrong ones.
- The plain page download waits at least 2 seconds between requests to the same site. It waits once when a site says "too many requests".
- The tool reads its command line itself, with no extra library.
- Each choice can be changed by saying so. The tool and its recordings stay on this Mac, so a step can be measured again.

## Not yet
- Nothing in the app uses these results yet. The real search, page-reading and model connections are built in the next research phase, after your picks.
- A successful Cloud Vision reply is not yet saved among the test samples. The search ran after billing was turned on, so it can still be saved if you pick Vision.
- The final measurement of the whole research agent on your pieces comes after the connections are built. It confirms or changes the model picks.

## Needs you

**Card 1 — Which photo search should research use?**
- **Question:** Google Lens through SerpApi (A) or Google Cloud Vision (B)?
- **Story:** Seven of your pieces have photos that were never online. For 6 of them, Lens found a page naming the designer or maker, nearly always as the first result. Vision found none. On 25 published pieces, Lens found 24 and Vision 8, and only 4 once Vision's copies of your own photos are set aside. But Lens reaches Google through SerpApi, which Google is suing for scraping. Google's claims were dismissed twice, and it may refile until 30 November. No court has ruled the scraping lawful.
- **Branches:** A — research usually finds the maker from the photo; free for 250 searches a month, then USD 25 per 1,000; if SerpApi is blocked, photo search stops until Vision is switched in. B — Google's own service, free up to 1,000 a month and faster (about 1 s), but it rarely finds the maker.
- **Recommendation:** A, with Vision kept as the replacement: 6 of 7 against 0 of 7 decides it. Mark the check sheet first if you want to confirm the hits.
- **On silence:** the photo-search connection is not built; the gate holds.
- **Trace:** 08A §4, §12A.1 · evidence Benchmark 1 · R8·U2.

**Card 2 — How should research read a listing page?**
- **Question:** A plain download from the Mac (A) or OpenAI's page tool (B)?
- **Story:** A plain download read 5 of your 10 sites, with the designer and maker in the page: auctionet, bukowskis, pamono, maleforsimports and sweakronan. Etsy and vntg block it. Your shop, midcenturymobler and vgliving are Shopify stores and refuse after a quick burst, so they must be read slowly. OpenAI's page tool read 8 of 10, all but pamono and vntg, for about one cent a page. But it returns only the model's summary, never the page itself, so later you could not see what the page actually said.
- **Branches:** A — exact page text, kept and checkable later, free; etsy and vntg stay unread (their results still count by title). B — two more sites, a cost per page, no page text kept, and the page-reader design must change.
- **Recommendation:** A, with Shopify stores read slowly: it keeps evidence you can check.
- **On silence:** the page-reading connection is not built; the gate holds.
- **Trace:** 08A §4 page reader · evidence Page reading · R8·U3.

**Card 3 — First AI model per research step?**
- **Question:** Photo look: gpt-6.1-sol, low effort. Page reading: gpt-6.1-terra, low. Condition: gpt-6.1-terra, high. Agree?
- **Story:** On cover photos (36 pieces), Sol named the wood right 21 times and wrong 9. Terra and Luna mostly declined to answer. On pages that name the maker, all three models were right 28–31 times out of 41. On condition photos (203 values), Terra at high effort was right 150 times, Sol 146 and Luna 124. Most misses were one step off, such as None instead of Minor.
- **Branches:** Yes — the best measured accuracy, about USD 0.04 a piece for the model. Cheaper page reading — Luna: 56 right instead of 60, at an eighth of the cost. Faster condition — Terra at default effort: 145 right, about 12 s a piece instead of 21 s.
- **Recommendation:** Yes; the final measurement on the finished agent confirms or changes it.
- **On silence:** no model defaults are written; the gate holds.
- **Trace:** 08A §5, §9 · evidence Benchmark 2 · R9·U2, R10·U1.
