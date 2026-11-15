//! Scoring of the visual-search benchmark (R1·U1 §Scoring definitions) and `<out>/report.md`;
//! and `report <export>` (R9·U1 task 2, 08A §10A): what the photographer did with each suggestion
//! of an exported run, per stage and overall, and the cost and time of its calls.

use crate::{
    Ctx,
    answer_key::{self, Bucket, Piece, SiteClass},
    bench_search::{self, first_ten, hits_from, piece_dir},
    pages,
    recording::call_files,
    serpapi::Hit,
};
use serde_json::{Value as Json, json};
use std::{collections::BTreeMap, fmt::Write as _, fs, path::Path};

pub const ROUTES: [(&str, &str); 2] = [
    ("serpapi_lens", "Lens via SerpApi"),
    ("cloud_vision", "Cloud Vision Web Detection"),
];

/// URL → page text, from every plain-HTTP page recording under `<out>/pages`.
pub fn page_texts(out: &Path) -> BTreeMap<String, (Option<u16>, String)> {
    let mut map = BTreeMap::new();
    for file in call_files(&bench_search::pages_dir(out)) {
        let Ok(rec) = serde_json::from_str::<Json>(&fs::read_to_string(&file).unwrap_or_default())
        else {
            continue;
        };
        let url = rec["request"]["url"].as_str().unwrap_or("").to_owned();
        let status = rec["response"]["status"].as_u64().map(|s| s as u16);
        let text = pages::strip_tags(&pages::body_text(&rec));
        map.insert(url, (status, text));
    }
    map
}

/// One piece through one route.
#[derive(Debug, Clone)]
pub struct RouteScore {
    pub ran: bool,
    pub ten: Vec<Hit>,
    pub own: usize,
    pub naming: Vec<bool>,
    pub site: Vec<SiteClass>,
    pub ms: Option<u64>,
}

impl RouteScore {
    pub fn any_naming(&self) -> bool {
        self.naming.iter().any(|n| *n)
    }
    pub fn naming_shop_trusted(&self) -> bool {
        self.naming
            .iter()
            .zip(&self.site)
            .any(|(n, s)| *n && !matches!(s, SiteClass::Other(_)))
    }
    pub fn first_naming(&self) -> Option<(usize, String)> {
        self.naming
            .iter()
            .position(|n| *n)
            .map(|i| (i + 1, site_label(&self.site[i])))
    }
    pub fn exact(&self) -> usize {
        self.ten.iter().filter(|h| h.exact).count()
    }
    /// The same counting with the exact copies of the photo removed.
    pub fn naming_without_exact(&self) -> bool {
        self.naming
            .iter()
            .zip(&self.ten)
            .any(|(n, h)| *n && !h.exact)
    }
}

pub fn site_label(s: &SiteClass) -> String {
    match s {
        SiteClass::Shop => "shop".into(),
        SiteClass::Trusted(e) => e.clone(),
        SiteClass::Other(h) => h.clone(),
    }
}

pub fn score_route(
    result: &Json,
    route: &str,
    piece: &Piece,
    names: &[String],
    shop: &[String],
    texts: &BTreeMap<String, (Option<u16>, String)>,
) -> RouteScore {
    let r = &result[route];
    let hits = hits_from(&r["hits"]);
    let (ten, own) = first_ten(&hits, piece, shop);
    let trusted = answer_key::trusted_sites();
    let naming = ten
        .iter()
        .map(|h| {
            answer_key::is_naming(&h.title, names)
                || texts
                    .get(&h.url)
                    .is_some_and(|(_, t)| answer_key::is_naming(t, names))
        })
        .collect();
    let site = ten
        .iter()
        .map(|h| answer_key::site_of(&h.url, shop, &trusted))
        .collect();
    RouteScore {
        ran: !r.is_null() && r["status"].as_u64() == Some(200),
        ten,
        own,
        naming,
        site,
        ms: r["ms"].as_u64(),
    }
}

fn median(mut v: Vec<usize>) -> String {
    if v.is_empty() {
        return "—".into();
    }
    v.sort_unstable();
    let m = v.len() / 2;
    if v.len() % 2 == 1 {
        v[m].to_string()
    } else {
        format!("{:.1}", (v[m - 1] + v[m]) as f64 / 2.0)
    }
}

/// A piece, its results, its key names and its score per route.
type Scored = (Piece, Json, Vec<(String, Bucket)>, Vec<RouteScore>);

pub fn group_name(published: bool, attributed: bool) -> &'static str {
    match (published, attributed) {
        (true, true) => "published, with attribution",
        (true, false) => "published, without attribution",
        (false, true) => "unpublished, with attribution",
        (false, false) => "unpublished, without attribution",
    }
}

pub fn write_search(ctx: &Ctx) -> Result<(), String> {
    let pieces = answer_key::load_pieces(&ctx.args.test_file)?;
    let corrections = answer_key::load_corrections(&ctx.args.corrections)?;
    let entries = answer_key::key_entries(&pieces, &corrections);
    let shop = bench_search::shop_hosts(&ctx.out, &pieces);
    let texts = page_texts(&ctx.out);
    let mut md = String::new();
    let mut scores: Vec<Scored> = Vec::new();
    for piece in &pieces {
        let Ok(text) = fs::read_to_string(piece_dir(&ctx.out, piece).join("results.json")) else {
            continue;
        };
        let result: Json = serde_json::from_str(&text).unwrap_or(Json::Null);
        let names = answer_key::key_names(piece.n, &entries);
        let plain: Vec<String> = names.iter().map(|(n, _)| n.clone()).collect();
        let routes = ROUTES
            .iter()
            .map(|(r, _)| score_route(&result, r, piece, &plain, &shop, &texts))
            .collect();
        scores.push((piece.clone(), result, names, routes));
    }
    let _ = writeln!(
        md,
        "# Benchmark 1 — visual search (R1·U1)\n\nGenerated by `psm-research score-search` from the recordings in this folder. Pieces scored: {} of {}. Shop hosts: {}. Trusted sites: {}.\n",
        scores.len(),
        pieces.len(),
        shop.join(", "),
        answer_key::TRUSTED_SITES.join(", ")
    );
    let _ = writeln!(
        md,
        "Definitions: the piece's own listing (shop host, path ending in `/products/<handle>`) is removed before ranking and counted apart. A **naming result** is a result whose title, or whose fetched page text, contains the piece's key designer or manufacturer (case-insensitive, punctuation and spaces ignored). A naming result is not proof of the right listing: see the check sheet.\n"
    );

    // Group table.
    let _ = writeln!(md, "## Per route and group\n");
    let _ = writeln!(
        md,
        "| Route | Group | Pieces | Naming result in first 10 | … only shop or trusted results | Median rank of first naming | Naming in first 10 without exact copies | Pieces with exact copies in first 10 | Own listing returned |\n|---|---|---|---|---|---|---|---|---|"
    );
    let mut groups_json = Vec::new();
    for (ri, (route, route_name)) in ROUTES.iter().enumerate() {
        for (published, attributed) in [(true, true), (false, true), (true, false), (false, false)]
        {
            let members: Vec<_> = scores
                .iter()
                .filter(|(p, ..)| p.published == published && p.has_attribution() == attributed)
                .collect();
            let n = members.len();
            let ran = members.iter().filter(|m| m.3[ri].ran).count();
            if ran == 0 {
                let _ = writeln!(
                    md,
                    "| {route_name} | {} | {n} | not run: no search of this route succeeded | — | — | — | — | — |",
                    group_name(published, attributed)
                );
                groups_json.push(json!({"route": route, "group": group_name(published, attributed), "pieces": n, "ran": 0}));
                continue;
            }
            let own = members.iter().filter(|m| m.3[ri].own > 0).count();
            let exact = members.iter().filter(|m| m.3[ri].exact() > 0).count();
            if attributed {
                let naming = members.iter().filter(|m| m.3[ri].any_naming()).count();
                let trusted = members
                    .iter()
                    .filter(|m| m.3[ri].naming_shop_trusted())
                    .count();
                let no_exact = members
                    .iter()
                    .filter(|m| m.3[ri].naming_without_exact())
                    .count();
                let ranks: Vec<usize> = members
                    .iter()
                    .filter_map(|m| m.3[ri].first_naming().map(|(r, _)| r))
                    .collect();
                let _ = writeln!(
                    md,
                    "| {route_name} | {} | {n} | {naming} | {trusted} | {} | {no_exact} | {exact} | {own} |",
                    group_name(published, attributed),
                    median(ranks.clone())
                );
                groups_json.push(json!({"route": route, "group": group_name(published, attributed), "pieces": n, "naming": naming, "naming_shop_trusted": trusted, "median_first_rank": median(ranks), "naming_without_exact": no_exact, "with_exact": exact, "own_listing": own}));
            } else {
                let _ = writeln!(
                    md,
                    "| {route_name} | {} | {n} | (no key: listed below) | — | — | — | {exact} | {own} |",
                    group_name(published, attributed)
                );
                groups_json.push(json!({"route": route, "group": group_name(published, attributed), "pieces": n, "with_exact": exact, "own_listing": own}));
            }
        }
    }

    // Corrected and from-text keys.
    let _ = writeln!(
        md,
        "\n## Keys outside the owner-verified bucket\n\n| Piece | Bucket | Key name | {} | {} |\n|---|---|---|---|---|",
        ROUTES[0].1, ROUTES[1].1
    );
    for (p, _, names, routes) in &scores {
        for (name, bucket) in names.iter().filter(|(_, b)| *b != Bucket::OwnerVerified) {
            let cell = |r: &RouteScore| match r.first_naming() {
                Some((rank, site)) => format!("rank {rank} ({site})"),
                None => "no naming result".into(),
            };
            let _ = writeln!(
                md,
                "| {} | {:?} | {name} | {} | {} |",
                p.n,
                bucket,
                cell(&routes[0]),
                cell(&routes[1])
            );
        }
    }

    // Per-piece table.
    let _ = writeln!(
        md,
        "\n## Per piece\n\n| Piece | Group | Key | Lens: first naming (site) | Lens: own | Lens: exact | Vision: first naming (site) | Vision: own | Vision: exact |\n|---|---|---|---|---|---|---|---|---|"
    );
    let mut pieces_json = Vec::new();
    for (p, _, names, routes) in &scores {
        let key = names
            .iter()
            .map(|(n, _)| n.as_str())
            .collect::<Vec<_>>()
            .join(" / ");
        let cell = |r: &RouteScore| {
            if !r.ran {
                return "no result".to_owned();
            }
            match r.first_naming() {
                Some((rank, site)) => format!("{rank} ({site})"),
                None if names.is_empty() => "—".into(),
                None => "none".into(),
            }
        };
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            p.n,
            group_name(p.published, p.has_attribution()),
            if key.is_empty() { "—" } else { &key },
            cell(&routes[0]),
            routes[0].own,
            routes[0].exact(),
            cell(&routes[1]),
            routes[1].own,
            routes[1].exact()
        );
        pieces_json.push(json!({
            "n": p.n, "group": group_name(p.published, p.has_attribution()), "key": key,
            "routes": ROUTES.iter().zip(routes).map(|((r, _), s)| json!({
                "route": r, "ran": s.ran, "first_naming": s.first_naming().map(|(rank, site)| json!({"rank": rank, "site": site})),
                "own": s.own, "exact": s.exact(), "ms": s.ms,
                "naming_shop_trusted": s.naming_shop_trusted(),
            })).collect::<Vec<_>>(),
        }));
    }

    // Cost and time.
    let _ = writeln!(md, "\n## Calls, cost and time per search\n");
    let _ = writeln!(
        md,
        "| Route | Successful searches | Mean seconds per search | Billing unit |\n|---|---|---|---|"
    );
    for (ri, (_, name)) in ROUTES.iter().enumerate() {
        let ms: Vec<u64> = scores
            .iter()
            .filter(|s| s.3[ri].ran)
            .filter_map(|s| s.3[ri].ms)
            .collect();
        let mean = if ms.is_empty() {
            0.0
        } else {
            ms.iter().sum::<u64>() as f64 / ms.len() as f64 / 1000.0
        };
        let unit = if ri == 0 {
            "1 search credit per search (upload: see V7); free plan 250 / month"
        } else {
            "1 unit per image; first 1,000 units / month free (see evidence probe 4)"
        };
        let _ = writeln!(md, "| {name} | {} | {mean:.2} | {unit} |", ms.len());
    }

    // Without attribution: first 10 for a human look.
    let _ = writeln!(
        md,
        "\n## Pieces without attribution: first 10 results (no score)\n"
    );
    for (p, _, _, routes) in scores.iter().filter(|s| !s.0.has_attribution()) {
        let _ = writeln!(md, "### Piece {} — {}\n", p.n, p.title);
        for ((_, name), r) in ROUTES.iter().zip(routes) {
            let _ = writeln!(md, "{name}:\n");
            if r.ten.is_empty() {
                let _ = writeln!(md, "- (no results)");
            }
            for (i, h) in r.ten.iter().enumerate() {
                let _ = writeln!(
                    md,
                    "{}. [{}]({}) — {}{}",
                    i + 1,
                    escape(&h.title),
                    h.url,
                    site_label(&r.site[i]),
                    if h.exact { " — exact copy" } else { "" }
                );
            }
            let _ = writeln!(md);
        }
    }

    // Check sheet.
    let _ = writeln!(
        md,
        "\n## Check sheet (12 pieces: the 4 lowest piece numbers of each test group)\n\nFor each result, mark in the last column whether it is the right listing (the same piece or the same model): yes / no.\n"
    );
    let mut per_group: BTreeMap<String, Vec<&Piece>> = BTreeMap::new();
    for p in &pieces {
        per_group.entry(p.test_group.clone()).or_default().push(p);
    }
    for (group, list) in &per_group {
        let mut list = list.clone();
        list.sort_by_key(|p| p.n);
        for p in list.into_iter().take(4) {
            let Some((_, _, names, routes)) = scores.iter().find(|s| s.0.n == p.n) else {
                continue;
            };
            let key = names
                .iter()
                .map(|(n, _)| n.as_str())
                .collect::<Vec<_>>()
                .join(" / ");
            let _ = writeln!(
                md,
                "### Piece {} ({group}) — {}\n\nKey: {}. Own listing returned: Lens {}, Vision {}.\n\n| Route | # | Result | Site | Naming | Exact copy | Right listing (yes / no) |\n|---|---|---|---|---|---|---|",
                p.n,
                p.title,
                if key.is_empty() { "—" } else { &key },
                routes[0].own,
                routes[1].own
            );
            for ((_, name), r) in ROUTES.iter().zip(routes) {
                for (i, h) in r.ten.iter().enumerate() {
                    let _ = writeln!(
                        md,
                        "| {name} | {} | [{}]({}) | {} | {} | {} | |",
                        i + 1,
                        escape(&h.title),
                        h.url,
                        site_label(&r.site[i]),
                        if r.naming[i] { "yes" } else { "" },
                        if h.exact { "yes" } else { "" }
                    );
                }
            }
            let _ = writeln!(md);
        }
    }

    // Disagreements.
    let _ = writeln!(
        md,
        "\n## Disagreements with the key (review candidates)\n\nFor pieces with a key: results in the first 10 on the shop or a trusted site that do not name the key (by title or fetched page text). A disagreement is a review candidate, not automatically an error of the route (`answer_key_corrections.json` `scoring_rules.disagreements`).\n\n| Piece | Key | Route | # | Result | Site | Page fetched |\n|---|---|---|---|---|---|---|"
    );
    let mut disagreements = 0;
    for (p, _, names, routes) in &scores {
        if names.is_empty() {
            continue;
        }
        let key = names
            .iter()
            .map(|(n, _)| n.as_str())
            .collect::<Vec<_>>()
            .join(" / ");
        for ((_, name), r) in ROUTES.iter().zip(routes) {
            for (i, h) in r.ten.iter().enumerate() {
                if !r.naming[i] && !matches!(r.site[i], SiteClass::Other(_)) {
                    disagreements += 1;
                    let fetched = texts
                        .get(&h.url)
                        .map_or("no".to_owned(), |(s, _)| format!("HTTP {}", s.unwrap_or(0)));
                    let _ = writeln!(
                        md,
                        "| {} | {key} | {name} | {} | [{}]({}) | {} | {fetched} |",
                        p.n,
                        i + 1,
                        escape(&h.title),
                        h.url,
                        site_label(&r.site[i])
                    );
                }
            }
        }
    }
    let _ = writeln!(
        md,
        "\nExact copies: a Lens result is marked an exact copy only when SerpApi sets `exact_matches: true` on it; with `type=all` no result of any piece carried the flag, so the with/without columns are equal. The unpublished group (photos not online) is the clean measure."
    );
    let summary = json!({"groups": groups_json, "pieces": pieces_json, "disagreements": disagreements, "shop_hosts": shop});
    fs::write(ctx.out.join("report.md"), md).map_err(|e| e.to_string())?;
    fs::write(
        ctx.out.join("search_scores.json"),
        serde_json::to_vec_pretty(&summary).unwrap_or_default(),
    )
    .map_err(|e| e.to_string())?;
    eprintln!("wrote {}", ctx.out.join("report.md").display());
    Ok(())
}

fn escape(text: &str) -> String {
    text.replace('|', "/")
        .replace('[', "(")
        .replace(']', ")")
        .replace('\n', " ")
}

// ---------------------------------------------------------------------------------------------
// `report <export>` (R9·U1)
// ---------------------------------------------------------------------------------------------

/// The counts of one stage, or of all stages.
///
/// A **suggestion** is a `writes.json` entry whose outcome is `written`, `unchanged` or
/// `lowered`. Its **action** is the first feedback row for its key (by time) whose suggested
/// value is the suggestion under `S` (`psm_core::mechanisms::sameness`); none is "not acted on".
/// **Not determined** counts the `not_determined` entries other than "not researched". A
/// suggestion or a not-determined field counts in every stage whose steps name its key (stage 7
/// when no earlier step does). For the run's attribution keys a suggestion is **right** when the
/// photographer acted and the final value is the suggestion under `S`, **wrong** when the action
/// left another value or none, **open** when nobody acted. Cost and time are the sums of the
/// stage's call records (`cost_micro_usd`, `duration_ms`).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct Counts {
    pub suggestions: u32,
    pub accepted_unchanged: u32,
    pub edited: u32,
    pub cleared: u32,
    pub kept_previous: u32,
    pub not_acted_on: u32,
    pub not_determined: u32,
    pub attribution_right: u32,
    pub attribution_wrong: u32,
    pub attribution_open: u32,
    pub calls: u32,
    pub cost_micro_usd: i64,
    pub duration_ms: i64,
}

/// An attribution suggestion against the photographer's final value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Right,
    Wrong,
    Open,
}

/// One attribution suggestion of a run and its verdict.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Attribution {
    pub run_id: i64,
    pub key: String,
    pub suggested: Json,
    pub confidence: String,
    /// The feedback action (`accepted`, `edited`, `cleared`, `kept_previous`), if any.
    pub action: Option<String>,
    pub final_value: Json,
    pub verdict: Verdict,
}

/// The report of an export.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct ExportReport {
    pub runs: u32,
    /// Runs whose call records had expired before the export.
    pub expired_runs: u32,
    /// Stage name → counts, in stage order (stages with nothing to count are absent).
    pub stages: Vec<(String, Counts)>,
    pub overall: Counts,
    /// Σ of the runs' recorded `cost_micro_usd` (kept after the call records expire).
    pub run_cost_micro_usd: i64,
    /// Σ of `finished_at − started_at` of the finished runs.
    pub run_ms: i64,
    /// Every attribution suggestion, run by run in `writes.json` order.
    pub attributions: Vec<Attribution>,
}

fn definitions_of(
    run: &crate::export_reader::RunDir,
) -> BTreeMap<String, psm_core::domain::attribute::Definition> {
    serde_json::from_value::<psm_research::service::FormSnapshot>(run.run["form_snapshot"].clone())
        .map(|form| {
            form.fields
                .into_iter()
                .map(|f| (f.definition.key.clone(), f.definition))
                .collect()
        })
        .unwrap_or_default()
}

fn same_value(
    definition: Option<&psm_core::domain::attribute::Definition>,
    a: &Json,
    b: &Json,
) -> bool {
    match definition {
        Some(d) => psm_core::mechanisms::sameness::same(d, a, b),
        None => a == b,
    }
}

/// The stages whose steps (other than stage 7's) name `key`; stage 7 when none does.
fn stages_naming(run: &crate::export_reader::RunDir, key: &str) -> Vec<String> {
    let decide = psm_research::domain::Stage::Decide.as_str();
    let mut stages: Vec<String> = Vec::new();
    for step in &run.steps {
        let stage = step["stage"].as_str().unwrap_or_default();
        let names = step["field_keys"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|k| k.as_str() == Some(key));
        if names && stage != decide && !stages.iter().any(|s| s == stage) {
            stages.push(stage.to_owned());
        }
    }
    if stages.is_empty() {
        stages.push(decide.to_owned());
    }
    stages
}

/// The report of every run of an export.
pub fn export_report(runs: &[crate::export_reader::RunDir]) -> ExportReport {
    let mut by_stage: BTreeMap<String, Counts> = BTreeMap::new();
    let mut overall = Counts::default();
    let mut report = ExportReport::default();
    for run in runs {
        report.runs += 1;
        report.expired_runs += u32::from(run.expired());
        report.run_cost_micro_usd += run.run["run"]["cost_micro_usd"].as_i64().unwrap_or(0);
        if let (Some(start), Some(end)) = (
            run.run["run"]["started_at"].as_i64(),
            run.run["run"]["finished_at"].as_i64(),
        ) {
            report.run_ms += (end - start).max(0);
        }
        let definitions = definitions_of(run);
        let attribution: Vec<String> = run.run["settings_snapshot"]["attribution_keys"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|k| k.as_str().map(str::to_owned))
            .collect();
        let mut feedback: Vec<&Json> = run.feedback.iter().collect();
        feedback.sort_by_key(|f| (f["at"].as_i64().unwrap_or(0), f["id"].as_i64().unwrap_or(0)));
        let mut add = |stages: &[String], edit: &dyn Fn(&mut Counts)| {
            edit(&mut overall);
            for stage in stages {
                edit(by_stage.entry(stage.clone()).or_default());
            }
        };
        for (key, write) in run.writes["writes"].as_object().into_iter().flatten() {
            let outcome = write["outcome"].as_str().unwrap_or_default();
            if !matches!(outcome, "written" | "unchanged" | "lowered") {
                continue;
            }
            let definition = definitions.get(key);
            let suggested = &write["value"];
            let action = feedback.iter().find(|f| {
                f["key"].as_str() == Some(key.as_str())
                    && same_value(definition, &f["suggested_value"], suggested)
            });
            let stages = stages_naming(run, key);
            let is_attribution = attribution.iter().any(|k| k == key);
            let verb = action.and_then(|f| f["action"].as_str()).unwrap_or("");
            let right = action.map(|f| {
                let fin = &f["final_value"];
                if verb == "accepted" && fin.is_null() {
                    true
                } else {
                    !fin.is_null() && same_value(definition, fin, suggested)
                }
            });
            if is_attribution {
                report.attributions.push(Attribution {
                    run_id: run.id(),
                    key: key.clone(),
                    suggested: suggested.clone(),
                    confidence: write["confidence"].as_str().unwrap_or_default().to_owned(),
                    action: action.and_then(|f| f["action"].as_str().map(str::to_owned)),
                    final_value: action
                        .map(|f| f["final_value"].clone())
                        .unwrap_or(Json::Null),
                    verdict: match right {
                        Some(true) => Verdict::Right,
                        Some(false) => Verdict::Wrong,
                        None => Verdict::Open,
                    },
                });
            }
            add(&stages, &|c: &mut Counts| {
                c.suggestions += 1;
                match verb {
                    "accepted" => c.accepted_unchanged += 1,
                    "edited" => c.edited += 1,
                    "cleared" => c.cleared += 1,
                    "kept_previous" => c.kept_previous += 1,
                    _ => c.not_acted_on += 1,
                }
                if is_attribution {
                    match right {
                        Some(true) => c.attribution_right += 1,
                        Some(false) => c.attribution_wrong += 1,
                        None => c.attribution_open += 1,
                    }
                }
            });
        }
        for (key, reason) in run.writes["not_determined"]
            .as_object()
            .into_iter()
            .flatten()
        {
            if reason.as_str() == Some("not researched") {
                continue;
            }
            let stages = stages_naming(run, key);
            add(&stages, &|c: &mut Counts| c.not_determined += 1);
        }
        for call in &run.calls {
            let stage = call["stage"].as_str().unwrap_or_default().to_owned();
            let cost = call["cost_micro_usd"].as_i64().unwrap_or(0);
            let ms = call["duration_ms"].as_i64().unwrap_or(0);
            add(std::slice::from_ref(&stage), &|c: &mut Counts| {
                c.calls += 1;
                c.cost_micro_usd += cost;
                c.duration_ms += ms;
            });
        }
    }
    let order = |name: &str| {
        psm_research::domain::Stage::ALL
            .iter()
            .position(|s| s.as_str() == name)
            .unwrap_or(usize::MAX)
    };
    let mut stages: Vec<(String, Counts)> = by_stage.into_iter().collect();
    stages.sort_by_key(|(name, _)| (order(name), name.clone()));
    report.stages = stages;
    report.overall = overall;
    report
}

fn usd(micro: i64) -> String {
    crate::budget::format_usd(micro)
}

fn seconds(ms: i64) -> String {
    format!("{}.{:01}", ms / 1000, (ms % 1000) / 100)
}

/// The report as plain-text tables.
pub fn render_export(report: &ExportReport) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "runs: {} ({} with expired call records); recorded run cost USD {}; run time {} s",
        report.runs,
        report.expired_runs,
        usd(report.run_cost_micro_usd),
        seconds(report.run_ms)
    );
    let _ = writeln!(
        out,
        "\n{:<10} {:>5} {:>8} {:>6} {:>7} {:>6} {:>9} {:>7} | {:>5} {:>5} {:>4} | {:>5} {:>12} {:>9}",
        "stage",
        "sugg",
        "accepted",
        "edited",
        "cleared",
        "kept",
        "not acted",
        "not det",
        "right",
        "wrong",
        "open",
        "calls",
        "USD",
        "seconds"
    );
    let row = |out: &mut String, name: &str, c: &Counts| {
        let _ = writeln!(
            out,
            "{:<10} {:>5} {:>8} {:>6} {:>7} {:>6} {:>9} {:>7} | {:>5} {:>5} {:>4} | {:>5} {:>12} {:>9}",
            name,
            c.suggestions,
            c.accepted_unchanged,
            c.edited,
            c.cleared,
            c.kept_previous,
            c.not_acted_on,
            c.not_determined,
            c.attribution_right,
            c.attribution_wrong,
            c.attribution_open,
            c.calls,
            usd(c.cost_micro_usd),
            seconds(c.duration_ms)
        );
    };
    for (name, counts) in &report.stages {
        row(&mut out, name, counts);
    }
    row(&mut out, "overall", &report.overall);
    let _ = writeln!(
        out,
        "\nA suggestion counts in every stage whose steps name its field; right / wrong / open are the run's attribution keys only."
    );
    out
}

/// Cost and time of one group of benchmark calls.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct GroupCost {
    pub calls: u32,
    pub cost_micro_usd: i64,
    pub duration_ms: i64,
    pub max_ms: i64,
}

/// A benchmark recording's calls grouped by provider, model and (when recorded) stage.
pub fn bench_report(calls: &[Json]) -> Vec<(String, GroupCost)> {
    let mut groups: BTreeMap<String, GroupCost> = BTreeMap::new();
    for call in calls {
        let mut name = call["provider"].as_str().unwrap_or("unknown").to_owned();
        for part in [call["model"].as_str(), call["meta"]["stage"].as_str()]
            .into_iter()
            .flatten()
            .filter(|p| !p.is_empty())
        {
            name.push(' ');
            name.push_str(part);
        }
        let g = groups.entry(name).or_default();
        let ms = call["duration_ms"].as_i64().unwrap_or(0);
        g.calls += 1;
        g.cost_micro_usd += call["cost_micro_usd"].as_i64().unwrap_or(0);
        g.duration_ms += ms;
        g.max_ms = g.max_ms.max(ms);
    }
    groups.into_iter().collect()
}

pub fn render_bench(groups: &[(String, GroupCost)]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{:<40} {:>6} {:>12} {:>10} {:>10}",
        "call group", "calls", "USD", "seconds", "max s"
    );
    let mut total = GroupCost::default();
    for (name, g) in groups {
        let _ = writeln!(
            out,
            "{:<40} {:>6} {:>12} {:>10} {:>10}",
            name,
            g.calls,
            usd(g.cost_micro_usd),
            seconds(g.duration_ms),
            seconds(g.max_ms)
        );
        total.calls += g.calls;
        total.cost_micro_usd += g.cost_micro_usd;
        total.duration_ms += g.duration_ms;
        total.max_ms = total.max_ms.max(g.max_ms);
    }
    let _ = writeln!(
        out,
        "{:<40} {:>6} {:>12} {:>10} {:>10}",
        "total",
        total.calls,
        usd(total.cost_micro_usd),
        seconds(total.duration_ms),
        seconds(total.max_ms)
    );
    out
}
