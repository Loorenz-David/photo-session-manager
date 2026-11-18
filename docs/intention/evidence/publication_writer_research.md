# Publication writer: research and default writing guide

Research date 2026-11-18. [S#] refers to the source list at the end. "Inference" marks my reasoning where no source settles the point.

## 1. How respected sellers write listings

- **Two title traditions.** Marketplaces (1stDibs, Chairish, Pamono, Design Market) use long English titles: `[Origin/era adj] [Type] in [Material] by [Designer] for [Manufacturer], [Decade]`, plus "Set of 2" where relevant [S1][S2][S3][S38]. Auction houses (Auctionet, Lauritz, Bukowskis) lead with the designer, often in capitals, then terse comma-separated facts: type, material, maker, model number, year, country, quantity [S5][S7][S8][S9]. Both name the model number when known and end on the decade.
- **Attribution is hedged in words**: "attributed to", "in the manner of", Swedish "troligen" [S9][S40][S41]. The same Kirkegaard chair is spelled Kierkegaard, Høng and Hng across major sites [S1][S3], so canonical names must come from fields, not the model.
- **Description length and order.** 1stDibs: about 70 words, one paragraph: identity, materials and features, what was renewed, condition [S1]. Pamono: 60–70 words with most facts in structured fields (creator, maker, period, country, condition, restoration, material, colour, W/D/H) [S2]. Auction lots: two to four sentences, dimensions on labelled lines including seat height [S5][S7][S8].
- **Tone.** Marketplaces add one warm line; auction houses stay neutral. Both state flaws plainly [S1][S5][S7].

## 2. Condition report conventions

- **No shared scale.** Pamono shows one grade with a fixed definition (Good = may show scratches and other signs of age) [S2]. 1stDibs gives a grade, the stock line "Wear consistent with age and use", then specifics [S1]. Chairish grades originality instead (Original Unaltered, Original Design Modified, Restored, Unknown) [S4]. Vinterior's tiers run down to "Average" [S11]. A furniture guide uses Excellent / Very Good / Good / Fair / Restoration Needed and separates normal wear from defects that affect use [S12].
- **Auction pattern**: a general wear sentence, then each flaw with its location and a pointer to a detail photo [S7]; missing parts listed explicitly [S5]. Bukowskis' Swedish lots use one-word lines such as *Bruksslitage, Repor, Fläckar* [S9].
- Auction houses frame reports as guidance, not guarantee [S6][S10]; a consumer retailer cannot (inference), so precision is the protection.
- **Buyers ask about** joints, underside, repairs, replaced hardware, veneer and upholstery [S13]: the report's checklist.
- **Google's terms**: `refurbished` means restored to look new *with a warranty*; wear or modification means `used` [S16]. Restored vintage is **used** unless the shop gives a warranty.

## 3. SEO

- **Google Shopping titles**: up to 150 characters, about 70 shown, most important details first, no promotional text, capitals for emphasis or symbols; add the brand when it differentiates [S15][S37]. **AI-generated titles must be submitted as `structured_title` with `trained_algorithmic_media`** [S15]. `condition` is required for used items [S16]; vintage goods are a named case of products without GTINs [S17].
- **Shopify**: SEO title input allows 70 characters, Shopify recommends 60; about 160 recommended for the meta description [S18], with a 320-character input cap per third-party docs [S19]. Keywords early, unique per page, plain language, no keyword lists.
- **Google Search**: concise, unique titles without repeated variants; a shop name short, at start or end after a delimiter [S20]. Meta descriptions have no fixed limit, are truncated by width, and for products should carry facts such as age and manufacturer; programmatic generation from page data is acceptable [S21].
- **How buyers search** (inference): every major seller's titles converge on designer, maker, model number, type, material, decade and "Danish/mid-century" [S1][S3][S7][S38]. No query logs were found; this convergence is the best proxy.
- **Borrowing without stuffing**: take nouns from comparable listings (canonical maker, model number, type words such as "easy chair", "nesting tables"), not their adjectives; one use in the title, one in the description [S20]. Spelling variants (Hong, Kierkegaard) belong in Shopify search synonyms, not visible copy (inference).
- **Structured data and metafields**: themes emit Product JSON-LD via `structured_data`, often with `itemCondition` hard-coded or missing [S23]; set `UsedCondition` [S22]. Use Furniture category metafields (early access) for colour and material [S24], and custom metafields for designer, maker, model, decade and dimensions. SEO fields become translatable only once set explicitly [S25].

## 4. Making AI text not read as AI

**Evidenced**
- Excess vocabulary is measurable: words such as *delves, showcasing, pivotal, crucial* surged across 15 million abstracts after LLMs [S27]. Antislop found some patterns over 1,000× more frequent in LLM text than human text and suppresses them with banned-pattern lists at generation time [S28].
- Wikipedia's field guide [S26] lists puffery (*boasts, vibrant, rich, nestled, renowned*), significance inflation (*testament, pivotal*), trailing "-ing" clauses, rule of three, "not just X but Y", "serves as" for "is", em-dash overuse and synonym cycling. It warns that signals shift between models, humans detect AI text at chance, and detectors are unreliable [S26][S29].
- Shoppers did not reliably tell AI from human product texts; quality mattered more than disclosure [S30]. Saying "AI" in descriptions lowered purchase intent [S31], so the copy should never refer to itself.
- Few-shot examples are among the most reliable ways to steer tone and format (3–5, varied); say what to do rather than only what not to do; match the prompt's style to the output [S32].

**Folklore or weak**
- Retail ban lists (*elevate, timeless, seamless, effortless, versatile, must-have*) come from practitioner blogs without conversion data [S33][S34]; plausible given [S27][S28], but unproven.
- A ban list inside the prompt alone is not shown to work; positive instructions plus a deterministic post-check are better supported [S28][S32]. "Humanizer" rewrites and detector gates have no support [S26].
## 5. Swedish specifics

- Swedish auction lots use *fåtölj, karmstol*, "modell nr", "1950-60-tal", "ca" with decimal commas, *sitthöjd*, and condition words *bruksslitage, repor, fläckar* [S8][S9].
- Useful trade vocabulary (inference): *pall, fotpall, satsbord, matbord, skänk/sideboard; teak, bok, ek, massiv, fanér, fanérskada, nagg; nyklädd, slipad och oljad, stadig, inga glapp; dansk design*.
- **Search language**: no query data found. Tradera's 2025 interior report analysed searches by brand, not by language [S35]. Swedish marketplace titles mix Swedish nouns with untranslated designer and maker names and loanwords like "design" and "retro" [S36]. Working assumption: Swedish buyers search the designer and maker in the original spelling plus a Swedish type noun. Keep Høng Stolefabrik in Danish; verify later in Shopify search analytics and Search Console.

## 6. Assessment of the owner's texts

**Already good**: plain, short, no puffery (the trustworthy auction register). Titles carry a distinguishing detail (green woven seat, cream bouclé). The description leads with attribution and names real form features. Restoration is disclosed; the report names flaws and says "ready to use".

**Improve**
1. One title formula, not two; add country and decade, the commonest modifiers in comparable titles.
2. Canonical maker names ("Høng Stolefabrik", not "Høng Stole").
3. One decade instead of "during the 1960s and 1970s".
4. Add dimensions with seat height.
5. Say what "refinished" means (oil or lacquer) and what is new.
6. Give each flaw a location, size and photo reference; always state stability; open with a grade word.
7. The "desk, dining table or reading corner" triplet is fine once but becomes a tic at scale; allow two uses at most.

## 7. Proposed default writing guide

### 7.1 Instruction text (about 330 words)

> You write product texts for a small Swedish shop that sells restored Scandinavian mid-century furniture. You receive structured fields and a short note of what the photos show. Write three texts: a title, a description and a condition report.
>
> Use only facts in the fields and the photo note. If a field is empty, leave that fact out. Never guess a designer, model, date or material. If the attribution field says "attributed", write "attributed to". Copy names, model numbers and manufacturer names exactly, including Danish letters.
>
> Write like a careful dealer writing a catalogue entry: plain sentences, concrete nouns, verbs like "is", "has" and "was made". Prefer one precise detail to a general claim. Describe shape, material and what was done to the piece. Use only adjectives a buyer could check in the photos.
>
> Title: follow the title formula exactly. Commas only, no words of praise.
>
> Description: 60 to 120 words in short paragraphs, in this order. First, what it is, who made it, when and where. Second, one or two sentences on form and materials. Third, what our workshop did, naming the finish and any new parts. Fourth, at most one sentence on where it can be used. End with the dimensions line in the given format.
>
> Condition report: at most 45 words. Begin with the grade word from the fields. Then the restoration in one sentence. Then each remaining flaw with its location and approximate size, citing the photo number if given. End with a sentence on stability. Never soften a flaw and never omit one listed in the fields.
>
> Write in the requested language. In Swedish, use Swedish nouns, keep proper names in their original spelling, and write as a Swedish dealer would, not as a translation.
>
> Use commas and full stops. No dashes as punctuation, exclamation marks, questions or strings of three adjectives. Do not mention AI, the writing process or the shop's values.
>
> Follow the style of the examples below but never reuse their sentences.

The banned list (7.5) runs as a post-check, not in the prompt.

### 7.2 Title formula (≤70 characters, hard maximum 100)

- **Designer known**: `{Designer} [{Model}] {Type} in {Material} for {Manufacturer}, {Decade}`; add `with {Colour} {Upholstery}` only within 70 characters; `, Set of {n}` for sets.
- **Designer unknown**: `{Country adj} {Material} {Type} [with {Colour} {Upholstery/seat}], {Decade}`.
- **Attributed**: `{Type} in {Material} attributed to {Designer}, {Decade}`.
- **Swedish**: `{Designer} [{modell}] {typ} i {material}, {Tillverkare}, {decennium}-tal`; unknown designer: `Dansk {material}{typ} [i {färg} {klädsel}], {decennium}-tal`.
- Title case in English, sentence case in Swedish.

### 7.3 Description structure

60–120 words, 2–4 short paragraphs: identity, form and materials, restoration, optional use sentence, then the dimensions line: `Dimensions: W 68 × D 72 × H 76 cm, seat height 42 cm.` / `Mått: B 68 × D 72 × H 76 cm, sitthöjd 42 cm.`

### 7.4 Condition report rules

1. At most 45 words, 2–4 sentences.
2. Open with a fixed grade: Excellent / Very good / Good / Fair (SV: Utmärkt / Mycket gott / Gott / Godtagbart skick), plus "restored" when it applies.
3. Restoration as actions and materials (sanded, oiled, new foam, new papercord).
4. Each flaw as type, location, approximate size, and "(see photo n)" when shown.
5. Always state stability.
6. Never "minor" without a size, never "as expected for its age" alone, never "perfect", "like new" or "flawless".
7. Every damage field set to true must appear; the post-check verifies this.

### 7.5 Banned words and patterns (post-check, case-insensitive)

- **EN**: timeless, elevate, testament, boasts, stunning, iconic (unless a field says so), nestled, vibrant, rich (as praise), showcase, seamless, effortless, versatile, must-have, statement piece, exquisite, impeccable, meticulous, curated, delve, tapestry, embodies, exemplifies, beloved, gem, treasure, "perfect for", "adds a touch of", "warmth and character", "piece of history", "whether you", "look no further".
- **SV**: tidlös, ikonisk, fantastisk, otrolig, unik (unless a field says so), "en riktig pärla", "ett måste", "perfekt för", "lyfter rummet", "tillför värme".
- **Patterns**: em or en dash as punctuation; "!" and "?"; "not just … but", "it's not … it's"; three comma-separated adjectives; trailing participle clauses (", highlighting…", ", adding…"); "serves as"; more than one use sentence; any designer, model or year absent from the fields.
- On a hit, regenerate once naming the hits; on a second hit, flag for the owner.

### 7.6 SEO title and meta description

- **SEO title**: ≤60 characters (Shopify cap 70). `{Designer or Country adj} {Type} in {Material}, {Decade}`, with `| {Shop}` only if still ≤60.
- **Meta description**: 120–155 characters. One or two sentences: identity, the key restoration fact, condition grade; "Ships from Sweden" if room. Unique, no keyword lists.
- Write Swedish SEO fields explicitly. For Google Shopping: `structured_title` disclosure, `condition=used`.

## 8. Four example outputs (invented inputs)

### A. Known designer (EN)
*Fields: dining chair; Børge Mogensen; FDB Møbler; J39; 1960s; Denmark; solid beech, papercord; W 49 D 46 H 78, seat 45; light scratches; stable; sanded, oiled, seat rewoven; photo 4 shows scratch.*

**Title**: Børge Mogensen J39 Chair in Beech for FDB Møbler, 1960s

**Description**: Model J39 by Børge Mogensen, made in Denmark by FDB Møbler in the 1960s.

The frame is solid beech with a curved top rail, and the seat is woven papercord.

We sanded and oiled the frame and rewove the seat in new natural papercord. It works at a dining table or a desk.

Dimensions: W 49 × D 46 × H 78 cm, seat height 45 cm.

**Condition**: Very good, restored. Frame sanded and oiled, seat rewoven with new papercord. A few light scratches remain on the back rail (see photo 4). Stable, with no loose joints.

**SEO title (51)**: Børge Mogensen J39 Chair in Beech, FDB Møbler 1960s
**Meta (148)**: Børge Mogensen J39 dining chair in solid beech, made by FDB Møbler in Denmark in the 1960s. Oiled frame and new papercord seat. Very good condition.

### B. No designer (EN)
*Fields: nesting tables ×3; designer unknown; Denmark; 1960s; teak veneer tops, solid teak legs; light scratches, veneer chip; stable; cleaned, re-oiled; photo 6 shows chip.*

**Title**: Danish Teak Nesting Tables, Set of 3, 1960s

**Description**: A set of three nesting tables made in Denmark in the 1960s. There are no maker's marks.

The tops are teak veneer on solid teak legs, and the two smaller tables slide under the largest.

We cleaned and re-oiled all three, keeping the original finish. Use them stacked as one side table or apart as bedside tables.

Dimensions: largest W 54 × D 39 × H 50 cm, smallest W 40 × D 36 × H 42 cm.

**Condition**: Good. Cleaned and re-oiled, not refinished. Light scratches on the top of the largest table and a 1 cm veneer chip on the front edge of the middle table (see photo 6). All three stand firm.

### C. Notable wear (EN)
*Fields: sideboard; designer unknown; Denmark; 1960s; teak veneer; W 180 D 45 H 78; water marks, lifted veneer, veneer chips, stiff door; stable; cleaned only; photo 5 shows lifted veneer.*

**Title**: Danish Teak Sideboard with Sliding Doors, 1960s

**Description**: A teak sideboard made in Denmark in the 1960s, with two sliding doors over adjustable shelves and four drawers on the right.

It is sold as found. We cleaned it but did not refinish it or repair the veneer, and the price reflects that. It suits a buyer who plans to restore it or is happy with visible wear.

Dimensions: W 180 × D 45 × H 78 cm.

**Condition**: Fair, cleaned only. Water marks across the top and a 10 × 3 cm patch of lifted veneer at the back left corner (see photo 5). Two veneer chips along the plinth, the larger about 2 cm. The left door slides stiffly. Structurally sound and stable.

### D. Swedish
*Fält: fåtölj; formgivare okänd; Danmark; 1960-tal; massiv teak; gräddvit ullbouclé; B 68 D 72 H 76, sitthöjd 42; lätta repor; stadig; slipad, oljad, nyklädd med ny stoppning; bild 5 visar repor.*

**Titel**: Dansk teakfåtölj i gräddvit bouclé, 1960-tal

**Beskrivning**: Fåtölj tillverkad i Danmark på 1960-talet. Formgivare och tillverkare är okända.

Stommen är i massiv teak med svängda armstöd. Sits och rygg har ny stoppning och är klädda i gräddvit ullbouclé.

Vi har slipat och oljat stommen. Den passar i ett läshörn eller vid ett skrivbord.

Mått: B 68 × D 72 × H 76 cm, sitthöjd 42 cm.

**Skick**: Mycket gott renoverat skick. Stommen slipad och oljad, sits och rygg nyklädda. Lätta repor på vänster armstöd (se bild 5). Stadig, inga glapp i fogarna.

**SEO-titel (38 + butiksnamn)**: Dansk teakfåtölj i ullbouclé, 1960-tal | {Butik}
**Meta (136)**: Dansk fåtölj i massiv teak från 1960-talet, slipad och oljad, nyklädd i gräddvit ullbouclé. Mycket gott renoverat skick. Sitthöjd 42 cm.

## 9. Open questions for the owner

1. Adopt the four-step grade scale, and how do condition choices map to it?
2. Who confirms a designer, and when is "attributed to" acceptable?
3. Era field: production decade, design year, or both?
4. Will you keep canonical designer and maker spellings?
5. Is seat height measured for every seating piece?
6. Can restoration be fixed choices (sanded, oiled, lacquered, new foam, webbing, papercord, fabric)?
7. Do you sell "as found" pieces like example C?
8. Keep the use-case sentence?
9. Shop name in SEO titles; "Ships from Sweden" in metas?
10. Will products feed Google Merchant Center (`structured_title` disclosure)?
11. Who proofreads the first Swedish batch?
12. Which 3–5 existing listings per language, edited to this guide, become the examples?

## Sources

- [S1] 1stDibs, Kirkegaard/Høng armchair: https://www.1stdibs.com/furniture/seating/armchairs/vintage-danish-armchair-teak-wool-erik-kierkegaard-hng-1960s/id-f_15853391/
- [S2] Pamono listing (structure, condition definition): https://www.pamono.com/scale-by-yasu-sasamoto-for-dulton-ltd-1980s
- [S3] Chairish, Kirkegaard Model 52: https://www.chairish.com/product/30014095/vintage-danish-model-52-armchair-in-teak-and-leather-by-erik-kirkegaard-for-hng-stolefabrik-1960s
- [S4] Chairish help, condition classification (via search excerpt; page blocks fetch): https://support.chairish.com/hc/en-us/articles/44253577597073
- [S5] Lauritz, Grete Jalk armchairs: https://www.lauritz.com/en/auction/grete-jalk-for-glostrup-moebelfabrik-a-pair-of-teak-armchairs-1960s-2/6517825
- [S6] Lauritz, Before you buy: https://www.lauritz.com/en/buying/before-you-buy
- [S7] Auctionet, Kirkegaard model 43: https://auctionet.com/en/3300884-erik-kirkegaard-a-pair-of-armchairs-frame-of-teak-hong-stolefabrik-model-43-1956
- [S8] Bukowskis, Kofod-Larsen "Åre": https://www.bukowskis.com/en/auctions/640/534-ib-kofod-larsen-fatolj-are-modell-nr-809-olof-persons-fatoljindustri-ope-jonkoping-1950-60-tal
- [S9] Bukowskis, Swedish lot with condition terms: https://www.bukowskis.com/sv/lots/437057
- [S10] Bukowskis buyer terms: https://www.bukowskis.com/en/help/buyer_terms/hammer
- [S11] Vinterior seller help (via search excerpt): https://help.vinterior.co/support/solutions/articles/76000057365-listing-as-an-individual-seller
- [S12] Reperch, condition grades: https://reperch.com/blog/furniture-condition-grades-explained-for-used-buyers
- [S13] 1stDibs, buying vintage furniture (via search excerpt): https://www.1stdibs.com/blogs/the-study/?p=436202
- [S15] Google Merchant Center, title and structured_title: https://support.google.com/merchants/answer/6324415
- [S16] Google Merchant Center, condition: https://support.google.com/merchants/answer/6324469
- [S17] Google Merchant Center, GTIN/identifier_exists: https://support.google.com/merchants/answer/6324478
- [S18] Shopify Help, SEO keywords: https://help.shopify.com/en/manual/promoting-marketing/seo/adding-keywords
- [S19] Hextom help, Shopify 70/320 limits: https://tms-help-center.hextom.com/article/370-why-is-my-translated-meta-title-meta-description-cut-off-tms
- [S20] Google Search Central, title links: https://developers.google.com/search/docs/appearance/title-link
- [S21] Google Search Central, snippets: https://developers.google.com/search/docs/appearance/snippet
- [S22] Google Search Central, merchant listing structured data: https://developers.google.com/search/docs/appearance/structured-data/merchant-listing
- [S23] Shopify blog, ecommerce schema; Shero Commerce schema tutorial: https://www.shopify.com/blog/ecommerce-schema , https://sherocommerce.com/blogs/insights/shopify-product-schema-tutorial
- [S24] Shopify Help, product taxonomy and category metafields: https://help.shopify.com/en/manual/products/details/product-category
- [S25] Shopify Help, international SEO and localization: https://help.shopify.com/en/manual/markets/seo , https://help.shopify.com/en/manual/international/languages
- [S26] Wikipedia, Signs of AI writing: https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing
- [S27] Kobak et al., excess vocabulary, arXiv 2406.07016: https://arxiv.org/abs/2406.07016
- [S28] Paech et al., Antislop, arXiv 2510.15061: https://arxiv.org/abs/2510.15061
- [S29] TechCrunch, 2025-11-20: https://techcrunch.com/2025/11/20/the-best-guide-to-spotting-ai-writing-comes-from-wikipedia/
- [S30] Nordhoff & Zumstein, LLM-optimized product texts (abstract): https://econpapers.repec.org/RePEc:spr:prbchp:978-3-031-88052-0_33
- [S31] Washington State University, "AI" in product descriptions: https://news.wsu.edu/?p=244668
- [S32] Anthropic, prompting best practices: https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices
- [S33] Retail Insider, AI product descriptions: https://retail-insider.com/articles/2026/10/how-retail-brands-can-use-ai-for-product-descriptions-without-sounding-robotic/
- [S34] Knowledge Hub Media, banned-word lists: https://knowledgehubmedia.com/using-ai-to-write-product-descriptions-that-actually-sell
- [S35] Tradera Circular Brands Award 2025, Interior: https://www.klassbols.se/klassbols/uploads/Interior_CBA_25-final.pdf
- [S36] Tradera listing (Swedish title style): https://www.tradera.com/da/item/302537/742824731/jens-juul-eilersen-designsoffa-2-fatoljer
- [S37] Feedonomics, Shopping title examples: https://feedonomics.com/blog/how-to-create-and-optimize-your-google-shopping-product-titles-with-examples/
- [S38] Design Market, Kirkegaard Model 52: https://www.design-mkt.com/180584-mid-century-armchair-in-teak-model-52-by-erik-kirkegaard-for-høng-denmark-1960s.html
- [S40] Chairish, "attributed to" listing: https://www.chairish.com/product/8663664/mid-century-danish-model-52-armchair-in-teak-attributed-to-erik-kirkegaard-for-hng-1960s
- [S41] Chairish, "in the manner of" listing: https://www.chairish.com/product/35330716/restored-mid-century-scandinavian-modern-arm-chair-in-solid-teak-and-new-fabric-in-the-manner-of-d-scan-c-1970s
