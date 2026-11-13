//! Scoring of the visual-search benchmark (R1·U1 §Scoring definitions) and `<out>/report.md`.

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
