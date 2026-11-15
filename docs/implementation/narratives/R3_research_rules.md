# Phase R3_research_rules: the research agent's rulebook, and the one door it writes your form through

## What you can do now
Nothing new appears in the app yet. Underneath, the research agent now has its full rulebook. It can turn a model's answer into a valid form value, or give up on it ("not determined"). It can tell which website a page belongs to, so `notetsy.com` is never mistaken for Etsy. It decides how sure a suggestion is: a designer reaches "medium" only when two different trusted sites agree. It prices every call exactly in micro-dollars and knows what to do when a call fails. It also has the one door through which research writes into an item's form. That door never overwrites a value you typed, accepted or cleared, and it never brings back a value you cleared before. The next research phase is what actually runs these rules on your items.

## Try it yourself
1. Open Terminal in the project folder and run the rules. Each rule prints by name, and all of them say `ok`:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-research --lib rules
   ```
   Look for names such as `a05_c1d_two_trusted_sites_medium`, `a04_c3c_suffix_without_dot_is_untrusted` and `a06_c4b_twenty_five_hour_day`.
2. Watch the write door work on a real app database. These 25 checks cover refusals, cleared values, saved previous values, lowering and repeat writes:
   ```
   cargo test -p psm-research --test engine writes::
   ```

## Choices made for you
- A text answer is trimmed at both ends, so " Oak table " becomes "Oak table". Say so if leading spaces should be kept.
- In a list-of-choices answer, one item that is not text throws out the whole answer; text items that match no choice are only dropped. Say so if you'd rather keep the matching items in that case too.
- Sizes and weights must arrive in mm, cm, m, in, g, kg or lb, and are converted into the field's first unit. Say so if research should accept other units (for example feet).
- A website address with an IP number (like `192.168.1.5`) is never a trusted site. This is fixed by the rules, not a setting.
- The shop's own website counts as one of the "two different sites" a designer needs for medium confidence. Say so if only outside trusted sites should count.
- If a provider says both "bad key" and "no credit", the app reports a rejected key. Both stop the run. Say so if "no credit" should win.
- If a call's price is missing from the price table, it is counted as free. The next phase must refuse to start a run until every price is filled in. Say so if you'd rather charge a fixed amount.
- A value you once cleared, or chose to keep over a suggestion, is never suggested again, even when the field is now empty. Say so if a cleared value should be allowed back after some time.
- Research treats a field as part of the Product step (title, description, tags and anything you placed there) and never fills it. Say so if some Product field should be researched.
- A research write does not appear in Undo. You will act on suggestions with Accept and Clear instead.

## Not yet
- Nothing in the app calls these rules yet. Research runs, the panel that shows what research did, and the settings come in later phases.
- No real model or search provider is contacted. That comes with the real adapters.
- One small rough edge, set aside for later: a field placed in both the Details step and the Product step would still be researched. You only get there by building such a layout on purpose.

## Needs you
Nothing.
