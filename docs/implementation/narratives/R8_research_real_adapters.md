# Phase R8_research_real_adapters: research now uses real services (OpenAI, Google Lens, and reading listing pages)

## What you can do now
Research can now run for real. It sends your photos to OpenAI to look at the piece. It sends the cover photo to Google Lens (through SerpApi) to find similar pieces online. It downloads the listing pages Lens finds and reads them, the way a browser would, but without running any of the page's scripts. It asks your shop for its own web addresses and for the listing of a piece that is already in the shop. That listing is never used as evidence for itself.

A page that cannot be read is marked "could not be read" and the run goes on. That covers a site that refuses downloads (Etsy does), a page that no longer exists, a dead website, and a page that needs scripts to show its text. Your shop's pages are read slowly, 2 seconds between requests, because Shopify blocks fast readers. If a site asks the app to wait, it waits once and tries again. The keys are never written into any research record or export. The page reader never reaches computers on your own network.

## Try it yourself
1. Start the app on scratch data:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev
   ```
2. Open **Settings → Research** and paste the model key (OpenAI) and the visual-search key (SerpApi). Add your trusted sites in your order. Choose a model for each stage and enter its prices. **Include a cache-write price:** until a price table ships with the app, a call that reports cache writes is otherwise charged the fallback price. Before a key is saved, the panel says "Add the model key". After that it says "No price is set for openai …" until prices are entered. Both messages are expected.
3. Create a session and shoot one photo. Open the product form and its research panel. Steps appear as research goes: looked, searched, read, compared, concluded.
4. When it says **Finished**, check a suggested designer. Its popover shows the confidence, the reason and the links it relied on.
5. Research a piece that is already in your shop. Its own listing appears as "this item's own listing" and is not used.
6. Research a piece whose search finds an Etsy page. That page shows "could not be read" and the run still finishes.
7. To see the same behaviour checked with recorded replies, without spending any credit:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-research --test engine adapter_
   cargo test -p psm-testkit --test suite research::r8_shop_facts::
   ```

## Choices made for you
- Lens results are asked for in English for the Swedish market, and only Lens's "visual matches" list is used. Say so if you want another market or other result lists.
- From each listing page, the reader downloads at most 4 pictures and keeps JPEG pictures only. Say so if you want more pictures or other formats.
- A page over 4 MB or a picture over 3 MB counts as unreadable, and a page's text is cut at 30,000 characters. Say so if any of these limits should change.
- A picture that redirects to another address is skipped rather than followed. Say so if pictures should follow redirects like pages do.
- Besides your own network, the reader also refuses broadcast and "this network" addresses. Say so if that ever blocks a real site.
- The model must answer number fields with real numbers, not with numbers written as text. Say so if you see number fields left empty.
- Volume fields get no unit names, because research has no neutral volume units. Research therefore never fills a volume field. Say so if you need volumes researched.
- If a research copy of your photo cannot be read from disk, the run fails with an error instead of waiting. Say so if you would rather it wait.
- Several internal choices were made too (how requests are recorded, how the test stand-ins match requests, how the search key is encoded). None of them changes what you see.

## Not yet
- Default model names and a built-in price table. Until then you choose the models and enter the prices yourself.
- Google Cloud Vision as a second visual-search option. It is the named replacement and is not built.
- Your shop's other web addresses, besides the main domain and the myshopify one. This needs the Shopify permission `read_markets`, which the app does not ask for. Tell us if your shop answers on other domains and you want to grant it.
- Recognising "out of credit" answers from OpenAI or SerpApi. An empty account shows as "key rejected", or as a temporary error that ends the run after three tries.
- Weight fields that use ounces as their first unit cannot be converted yet. Your shop has no weight field today, so nothing fails.
- Checks so far used recorded replies only. Nobody has run a live research run against the real services since this phase.

## Needs you
Nothing.
