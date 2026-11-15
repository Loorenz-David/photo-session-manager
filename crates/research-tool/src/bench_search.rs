//! Visual-search benchmark (R1·U1 task 3): the reduced copy of each piece's first image to both
//! routes, then the pages among the first 10 results that are on the shop or a trusted site, plus
//! the first three other pages, by plain HTTP (cached by URL across routes and pieces).

use crate::{
    Ctx,
    answer_key::{self, Piece, SiteClass},
    cloud_vision,
    http::Wire,
    pages,
    recording::{ToolError, sha256_hex},
    serpapi::{self, Hit},
};
use psm_core::ports::images::Imaging;
use serde_json::{Value as Json, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

pub const LONG_EDGE: u32 = 1024;

/// The owner's shop file, downloaded once (`<out>/source/<sha256 of url>.<ext>`). Not a provider
/// call, so not recorded as one.
pub fn source_image(ctx: &Ctx, url: &str) -> Result<PathBuf, String> {
    let ext = url
        .rsplit('.')
        .next()
        .filter(|e| e.len() <= 4)
        .unwrap_or("img")
        .to_ascii_lowercase();
    let path = ctx
        .out
        .join("source")
        .join(format!("{}.{ext}", &sha256_hex(url.as_bytes())[..24]));
    if path.exists() {
        return Ok(path);
    }
    let reply = ctx
        .seam
        .send(&Wire {
            method: "GET".into(),
            url: url.to_owned(),
            headers: Vec::new(),
            body: Vec::new(),
            timeout: Duration::from_secs(120),
            follow_redirects: true,
        })
        .map_err(|e| format!("download of a shop image failed: {}", e.text))?;
    if reply.status != 200 {
        return Err(format!("download of a shop image: HTTP {}", reply.status));
    }
    fs::create_dir_all(path.parent().unwrap_or(Path::new("."))).map_err(|e| e.to_string())?;
    fs::write(&path, &reply.body).map_err(|e| e.to_string())?;
    Ok(path)
}

/// The reduced copy (long edge `long_edge`) made by `psm_infra`'s thumbnail function (H3: no
/// new pixel code). `<out>/reduced/<sha prefix>-<long edge>.jpg`.
pub fn reduced(ctx: &Ctx, url: &str, long_edge: u32) -> Result<Vec<u8>, String> {
    let source = source_image(ctx, url)?;
    let dest = ctx.out.join("reduced").join(format!(
        "{}-{long_edge}.jpg",
        &sha256_hex(url.as_bytes())[..24]
    ));
    if !dest.exists() {
        let made = psm_infra::imaging::ImageOpsImpl
            .thumbnail(&source, &dest, long_edge)
            .map_err(|e| e.to_string())?;
        if !made {
            return Err(format!(
                "the thumbnail function could not reduce {}",
                source.display()
            ));
        }
    }
    fs::read(&dest).map_err(|e| e.to_string())
}

/// The reduced cover for SerpApi's 500 KB upload limit: 1024, lowered by 128 until it fits.
pub fn reduced_for_upload(ctx: &Ctx, url: &str) -> Result<(Vec<u8>, u32), String> {
    let mut edge = LONG_EDGE;
    loop {
        let bytes = reduced(ctx, url, edge)?;
        if bytes.len() <= serpapi::MAX_UPLOAD_BYTES || edge <= 512 {
            return Ok((bytes, edge));
        }
        edge -= 128;
    }
}

pub fn piece_dir(out: &Path, piece: &Piece) -> PathBuf {
    out.join(format!("{}-{}", piece.n, piece.handle))
        .join("run-bench")
}

pub fn selected(ctx: &Ctx) -> Result<Vec<Piece>, String> {
    let pieces = answer_key::load_pieces(&ctx.args.test_file)?;
    Ok(pieces
        .into_iter()
        .filter(|p| ctx.args.pieces.is_none_or(|(a, b)| p.n >= a && p.n <= b))
        .collect())
}

fn tool(e: ToolError) -> String {
    e.to_string()
}

pub fn env(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|_| format!("{name} is not set"))
}

/// One piece through the selected routes; writes (or updates) `results.json` beside the calls.
pub fn search_piece(ctx: &Ctx, piece: &Piece) -> Result<Json, String> {
    let dir = piece_dir(&ctx.out, piece);
    let cover = piece.images.first().ok_or("piece has no image")?;
    let (jpeg, edge) = reduced_for_upload(ctx, cover)?;
    let wants = |r: &str| ctx.args.routes.is_empty() || ctx.args.routes.iter().any(|x| x == r);
    let mut result: Json = fs::read_to_string(dir.join("results.json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| json!({}));
    result["n"] = json!(piece.n);
    result["handle"] = json!(piece.handle);
    result["long_edge"] = json!(edge);
    result["image_bytes"] = json!(jpeg.len());
    result["image_sha256"] = json!(sha256_hex(&jpeg));
    if wants("serpapi_lens") {
        let serp_key = env("SERPAPI_API_KEY")?;
        let upload = ctx
            .caller
            .call(serpapi::upload(
                "serpapi-upload",
                &dir,
                &serp_key,
                jpeg.clone(),
            ))
            .map_err(tool)?;
        result["serpapi_lens"] = match serpapi::image_id(upload.body()) {
            Some(id) => {
                let search = ctx
                    .caller
                    .call(serpapi::search(
                        "serpapi-search",
                        &dir,
                        &serp_key,
                        &id,
                        "all",
                    ))
                    .map_err(tool)?;
                json!({
                    "upload_call": upload.call_id, "search_call": search.call_id,
                    "status": search.status(), "ms": search.record["duration_ms"],
                    "upload_ms": upload.record["duration_ms"],
                    "hits": hits_json(&serpapi::hits(search.body())),
                    "error": search.body()["error"],
                })
            }
            None => {
                json!({"upload_call": upload.call_id, "status": upload.status(), "error": upload.body(), "hits": []})
            }
        };
    }
    if wants("cloud_vision") {
        let vision_key = env("GOOGLE_VISION_API_KEY")?;
        let vision = ctx
            .caller
            .call(cloud_vision::web_detection(
                "cloud-vision",
                &dir,
                &vision_key,
                jpeg.clone(),
                10,
            ))
            .map_err(tool)?;
        result["cloud_vision"] = json!({
            "call": vision.call_id, "status": vision.status(), "ms": vision.record["duration_ms"],
            "hits": hits_json(&cloud_vision::hits(vision.body())),
            "error": if vision.ok() { vision.body()["responses"][0]["error"].clone() } else { vision.body()["error"].clone() },
        });
    }
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(
        dir.join("results.json"),
        serde_json::to_vec_pretty(&result).unwrap_or_default(),
    )
    .map_err(|e| e.to_string())?;
    Ok(result)
}

fn hits_json(hits: &[Hit]) -> Json {
    Json::Array(
        hits.iter()
            .map(|h| json!({"rank": h.rank, "title": h.title, "url": h.url, "exact": h.exact}))
            .collect(),
    )
}

pub fn hits_from(value: &Json) -> Vec<Hit> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(|h| Hit {
            rank: h["rank"].as_u64().unwrap_or(0) as u32,
            title: h["title"].as_str().unwrap_or("").to_owned(),
            url: h["url"].as_str().unwrap_or("").to_owned(),
            exact: h["exact"].as_bool().unwrap_or(false),
        })
        .collect()
}

/// The shop hosts: the host of the test file's storefront URLs (the owner's live shop), plus the
/// hosts V9 read from the test store's Admin API.
pub fn shop_hosts(out: &Path, pieces: &[Piece]) -> Vec<String> {
    let mut from_v9 = Vec::new();
    if let Ok(text) = fs::read_to_string(out.join("probes/v9/answer.json"))
        && let Ok(json) = serde_json::from_str::<Json>(&text)
        && let Some(hosts) = json["hosts"].as_array()
    {
        from_v9 = hosts
            .iter()
            .filter_map(|h| h.as_str().map(str::to_owned))
            .collect();
    }
    let mut hosts: Vec<String> = from_v9;
    hosts.extend(
        pieces
            .iter()
            .filter_map(|p| answer_key::host(&p.url))
            .map(|h| h.strip_prefix("www.").unwrap_or(&h).to_owned()),
    );
    hosts.sort();
    hosts.dedup();
    hosts
}

/// The first 10 results after the piece's own listing is removed, and the own-listing count.
pub fn first_ten(hits: &[Hit], piece: &Piece, shop: &[String]) -> (Vec<Hit>, usize) {
    let own = hits
        .iter()
        .filter(|h| answer_key::is_own_listing(&h.url, &piece.handle, shop))
        .count();
    let rest: Vec<Hit> = hits
        .iter()
        .filter(|h| !answer_key::is_own_listing(&h.url, &piece.handle, shop))
        .take(10)
        .cloned()
        .collect();
    (rest, own)
}

/// The URLs to fetch for one route's first ten: shop and trusted pages, then the first three others.
pub fn pages_to_fetch(ten: &[Hit], shop: &[String]) -> Vec<String> {
    let trusted = answer_key::trusted_sites();
    let mut urls = Vec::new();
    let mut others = 0;
    for h in ten {
        match answer_key::site_of(&h.url, shop, &trusted) {
            SiteClass::Shop | SiteClass::Trusted(_) => urls.push(h.url.clone()),
            SiteClass::Other(_) if others < 3 => {
                others += 1;
                urls.push(h.url.clone());
            }
            SiteClass::Other(_) => {}
        }
    }
    urls
}

pub fn pages_dir(out: &Path) -> PathBuf {
    out.join("pages")
}

/// Fetches one page by plain HTTP (recorded under `<out>/pages`, one fetch per URL ever).
pub fn fetch_page(ctx: &Ctx, url: &str) -> Result<crate::recording::Recorded, String> {
    let host = answer_key::host(url).unwrap_or_else(|| "page".into());
    ctx.caller
        .call(pages::get(
            &format!("page-{}", host.replace('.', "_")),
            &pages_dir(&ctx.out),
            url,
        ))
        .map_err(tool)
}

pub fn run(ctx: &Ctx) -> Result<(), String> {
    let pieces = selected(ctx)?;
    let all = answer_key::load_pieces(&ctx.args.test_file)?;
    let shop = shop_hosts(&ctx.out, &all);
    let mut failures = Vec::new();
    for piece in &pieces {
        match search_piece(ctx, piece) {
            Ok(r) => eprintln!(
                "piece {}: lens {} hits, vision {} hits",
                piece.n,
                r["serpapi_lens"]["hits"].as_array().map_or(0, Vec::len),
                r["cloud_vision"]["hits"].as_array().map_or(0, Vec::len)
            ),
            Err(e) if e.contains("refused") || e.contains("key value") => return Err(e),
            Err(e) => {
                eprintln!("piece {}: {e}", piece.n);
                failures.push(piece.n);
            }
        }
    }
    for piece in &pieces {
        let Ok(text) = fs::read_to_string(piece_dir(&ctx.out, piece).join("results.json")) else {
            continue;
        };
        let result: Json = serde_json::from_str(&text).unwrap_or(Json::Null);
        for route in ["serpapi_lens", "cloud_vision"] {
            let (ten, _) = first_ten(&hits_from(&result[route]["hits"]), piece, &shop);
            for url in pages_to_fetch(&ten, &shop) {
                if let Err(e) = fetch_page(ctx, &url) {
                    if e.contains("refused") || e.contains("key value") {
                        return Err(e);
                    }
                    eprintln!("page fetch failed: {e}");
                }
            }
        }
    }
    if !failures.is_empty() {
        eprintln!("pieces without a result: {failures:?}");
    }
    Ok(())
}
