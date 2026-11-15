# Phase R9_research_tool_and_acceptance: a dev-Mac tool that grades research, and everything ready to measure it on your 42 pieces

## What you can do now
On the dev Mac, a command-line tool now reads the research file you export from the app. It tells you, stage by stage, how many suggestions you accepted as they were, edited, cleared or left alone. It also shows how many fields research left open, and what each stage cost in money and seconds. For designer and maker it says whether the suggestion was right or wrong against what you finally kept. It can also re-send the AI calls of one stage with another model, effort or instructions, and show the old and new answers side by side. A call whose photo you deleted, whose picture is missing or whose record has expired is never re-sent. A third command runs the finished research agent on your 42 test pieces in a throwaway data folder, with a stand-in shop that refuses every write. A fourth command grades those runs against your answer key. Separately, the page reader now reads older Swedish dealer pages (ä, å, ö) correctly and treats a PDF as "could not be read". It also refuses a few more kinds of private network addresses.

## Try it yourself
1. Export research from the app on scratch data (Settings → Research → Export research), then:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo run -p psm-research-tool -- report ~/Downloads/research-export-<stamp>.zip
   ```
   Read which stage loses the most suggestions to edits. Add `--json` for the raw numbers.
2. The measurement on the 42 pieces (the orchestrator runs it with you nearby; it spends real OpenAI money, at most USD 40):
   ```
   set -a; source ~/.psm_research.env; set +a
   cargo run -p psm-research-tool -- accept --pieces docs/test-items/beyo_test_samples_1.json --data ~/psm-research-bench/a23 --ceiling-usd 40
   cargo run -p psm-research-tool -- score-accept --data ~/psm-research-bench/a23
   ```
3. Afterwards, read "Acceptance measurement" in the research evidence document. Of the designer and maker suggestions the agent was fairly sure about, how many were right? How many pieces did it leave open? What does one piece cost?
4. Open the disagreement list and look at three pieces you know well.
5. To see the tool's behaviour checked without spending anything:
   ```
   cargo test -p psm-research-tool
   ```

## Choices made for you
- A suggestion counts as "right" when you acted on it and the value you kept is the same as the suggestion. Spelling and case differences don't count. Say so if editing the value counts differently for you.
- The re-send command only re-sends AI calls. Image searches and page downloads are listed as skipped, because there is no model to swap. Say so if you want those repeated too.
- The measurement treats your shop's "Designer" and "Manufacturer" fields as the designer and maker. The shipped app default names a different designer field. Say which fields your live shop really uses, and the defaults will follow.
- The measurement uses your picks: observe on Sol (low), read on Terra (low), condition on Terra (high). The other stages use Sol (low) where pictures are involved and Terra (low) for the final text-only step. Say so to try others.
- Because your test file has no field layouts, every field research can fill is asked for, for every item type. Say so if you want your shop's real layouts used.
- All shop pictures of a piece are treated as main photos, with the first as cover. The condition-report pictures are treated as condition photos.

## Not yet
- The measurement itself has not run. It needs you nearby and spends up to USD 40. Until it runs there are no numbers for the model choice.
- The one trial re-send with a real key waits until the measurement has produced real exports.
- The re-send command has no spending limit of its own yet. The model comparison after the measurement re-sends many calls, so it should be budgeted first.
- The test pieces' photos are finished shop pictures, not your studio previews. Results on fresh captures may differ.

## Needs you
**Question:** After the 42-piece measurement, which model and effort should each research stage ship with?
**Story:** Research is about to run on all 42 pieces with your current picks, and then compare other models on 12 of them. The tables will show, per stage and model, how often it agreed with your answer key, what a piece costs and how long it takes. Whatever you choose becomes what every new installation starts with.
**Branches:** Keep your picks if the tables agree → defaults ship as they are. · Change a stage → that stage ships the model the tables favour. · Decide later → no defaults ship; each user picks models by hand.
**Recommendation:** Decide when the tables are in. They exist to answer exactly this, and USD 40 is already set aside for them.
**On silence:** The gate holds; no default models ship.
**Trace:** 08A §9, §11; R9 plan U2 tasks 3–5; R10·U1 shipped defaults.
