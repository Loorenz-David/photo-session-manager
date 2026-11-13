//! The probes of R1·U1 task 2 (RESEARCH-VERIFY-1…10 and evidence probe 4). Each records its
//! calls under `<out>/probes/<id>/` and writes `answer.json` with what was observed; the evidence
//! doc is written from those files.

use crate::{
    Ctx,
    answer_key::{self, SiteClass},
    bench_search::{self, env},
    budget::Prices,
    cloud_vision,
    openai::{self, ModelCall, WebSearch},
    pages,
    recording::{Recorded, ToolError},
    serpapi, shop,
};
use serde_json::{Value as Json, json};
use std::{fs, path::PathBuf, time::Duration};

/// `(jpeg bytes, detail)` as `ModelCall::images` holds them.
type ImageInput = (Vec<u8>, Option<String>);

fn dir(ctx: &Ctx, id: &str) -> PathBuf {
    ctx.out.join("probes").join(id)
}

fn tool(e: ToolError) -> String {
    e.to_string()
}

fn save(ctx: &Ctx, id: &str, answer: &Json) -> Result<(), String> {
    let path = dir(ctx, id).join("answer.json");
    let text = serde_json::to_vec_pretty(answer).unwrap_or_default();
    crate::recording::write_scanned(&ctx.caller.secrets.borrow(), &path, &text).map_err(tool)?;
    println!("{}", String::from_utf8_lossy(&text));
    Ok(())
}

/// The candidate models recorded in `prices.json` (`candidates.flagship|mid|small`).
pub fn candidates(ctx: &Ctx) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(ctx.out.join("prices.json")).map_err(|e| e.to_string())?;
    let json: Json = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let c = &json["candidates"];
    Ok(["flagship", "mid", "small"]
        .iter()
        .filter_map(|k| c[k].as_str().map(str::to_owned))
        .collect())
}

/// The probe image: the cover of test piece 2 (a JPEG on the owner's shop).
fn probe_image(ctx: &Ctx, long_edge: u32) -> Result<Vec<u8>, String> {
    let pieces = answer_key::load_pieces(&ctx.args.test_file)?;
    let piece = pieces.iter().find(|p| p.n == 2).ok_or("piece 2 missing")?;
    bench_search::reduced(ctx, &piece.images[0], long_edge)
}

fn observe_schema() -> Json {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["piece_type", "materials", "visible_marking"],
        "properties": {
            "piece_type": {"type": "string"},
            "materials": {"type": "array", "items": {"type": "string"}},
            "visible_marking": {"type": "boolean"}
        }
    })
}

const OBSERVE_TEXT: &str = "Look at the photo of a vintage furniture piece. Name the piece type, the visible materials, and whether a maker's label, stamp or marking is visible. Answer in English.";

fn model_call(
    ctx: &Ctx,
    id: &str,
    label: &str,
    mc: &ModelCall,
    prices: &Prices,
) -> Result<Recorded, String> {
    let key = env("OPENAI_API_KEY")?;
    let call = openai::call(label, &dir(ctx, id), &key, mc, prices)?;
    ctx.caller.call(call).map_err(tool)
}

fn summary(r: &Recorded) -> Json {
    let body = r.body();
    json!({
        "call": r.call_id,
        "status": r.status(),
        "response_status": body["status"],
        "incomplete_details": body["incomplete_details"],
        "usage": body["usage"],
        "cost_micro_usd": r.record["cost_micro_usd"],
        "duration_ms": r.record["duration_ms"],
        "error": if r.record["error"].is_null() { body["error"].clone() } else { r.record["error"].clone() },
        "output_types": body["output"].as_array().map(|a| a.iter().map(|o| o["type"].clone()).collect::<Vec<_>>()),
        "output_text": openai::output_text(body),
        "refusal": openai::refusal(body),
    })
}

pub fn run(ctx: &Ctx) -> Result<(), String> {
    let id = ctx
        .args
        .positional
        .first()
        .cloned()
        .ok_or("probe needs an id")?;
    match id.as_str() {
        "v1" => v1(ctx),
        "v2" => v2(ctx),
        "v4" => v4(ctx),
        "v5" => v5(ctx),
        "v6" => v6(ctx),
        "v7" => v7(ctx),
        "v8" => v8(ctx),
        "v9" => v9(ctx),
        "v10" => v10(ctx),
        "ep4" => ep4(ctx),
        other => Err(format!(
            "unknown probe {other} (v3 is answered from v1's incomplete reply and the docs)"
        )),
    }
}

/// V1 usage (and V3 incomplete): one image + strict-schema call per candidate; one web search
/// call; one `incomplete` reply.
fn v1(ctx: &Ctx) -> Result<(), String> {
    let prices = ctx.prices()?;
    let image = probe_image(ctx, 1024)?;
    let mut answers = Vec::new();
    for model in candidates(ctx)? {
        let mut mc = ModelCall::new(&model, OBSERVE_TEXT);
        mc.images = vec![(image.clone(), None)];
        mc.schema = Some(("observe".into(), observe_schema()));
        mc.max_output_tokens = 4_000;
        let r = model_call(ctx, "v1", &format!("v1-image-schema-{model}"), &mc, &prices)?;
        answers
            .push(json!({"case": "image + strict schema", "model": model, "result": summary(&r)}));
    }
    let small = candidates(ctx)?.last().cloned().ok_or("no candidates")?;
    let mut ws = ModelCall::new(
        &small,
        "Who designed the 'CH24 Wishbone' chair and which company makes it? Answer in one sentence with a source.",
    );
    ws.web_search = Some(WebSearch {
        allowed_domains: Vec::new(),
    });
    ws.max_tool_calls = Some(2);
    ws.max_output_tokens = 2_000;
    ws.include = vec!["web_search_call.action.sources".into()];
    let r = model_call(ctx, "v1", &format!("v1-web-search-{small}"), &ws, &prices)?;
    answers.push(json!({"case": "web search", "model": small, "result": summary(&r),
        "web_search_items": r.body()["output"].as_array().map(|a| a.iter().filter(|o| o["type"] == "web_search_call").cloned().collect::<Vec<_>>())}));
    let mut inc = ModelCall::new(&small, OBSERVE_TEXT);
    inc.images = vec![(image, None)];
    inc.schema = Some(("observe".into(), observe_schema()));
    inc.max_output_tokens = 16;
    let r = model_call(ctx, "v1", &format!("v1-incomplete-{small}"), &inc, &prices)?;
    answers.push(
        json!({"case": "incomplete (max_output_tokens 16)", "model": small, "result": summary(&r)}),
    );
    save(
        ctx,
        "v1",
        &json!({"probe": "V1 usage / V3 incomplete", "answers": answers}),
    )
}

/// V2 errors: an invalid key string (never the real one) and an oversized input.
fn v2(ctx: &Ctx) -> Result<(), String> {
    let prices = ctx.prices()?;
    let small = candidates(ctx)?.last().cloned().ok_or("no candidates")?;
    let mut answers = Vec::new();
    let mc = ModelCall::new(&small, "Say OK.");
    let mut bad = openai::call(
        "v2-invalid-key",
        &dir(ctx, "v2"),
        "sk-proj-INVALID-probe-key-not-a-real-key-000000",
        &mc,
        &prices,
    )?;
    // An invalid-key call is never billed: its worst case is 0.
    bad.worst_case_micro = 0;
    let r = ctx.caller.call(bad).map_err(tool)?;
    answers.push(json!({"case": "invalid key", "result": summary(&r), "headers": r.record["response"]["headers"]}));
    // Over the 922,000-token input limit: ~1.2 M tokens of text.
    let big = " word".repeat(1_200_000);
    let mut over = ModelCall::new(&small, &big);
    over.max_output_tokens = 16;
    let r = model_call(ctx, "v2", "v2-oversized-input", &over, &prices)?;
    answers.push(json!({"case": "oversized input (~1.2M tokens)", "result": summary(&r), "headers": r.record["response"]["headers"]}));
    save(
        ctx,
        "v2",
        &json!({"probe": "V2 errors", "answers": answers}),
    )
}

/// V4 image tokens: a text-only baseline, then the same photo at 1024 and 2048 with `detail`
/// high and low, per candidate model.
fn v4(ctx: &Ctx) -> Result<(), String> {
    let prices = ctx.prices()?;
    let small_img = probe_image(ctx, 1024)?;
    let large_img = probe_image(ctx, 2048)?;
    let text = "Reply with the single word OK.";
    let mut answers = Vec::new();
    for model in candidates(ctx)? {
        let cases: Vec<(&str, Option<ImageInput>)> = vec![
            ("text only", None),
            ("1024 detail auto", Some((small_img.clone(), None))),
            (
                "1024 detail high",
                Some((small_img.clone(), Some("high".into()))),
            ),
            (
                "1024 detail low",
                Some((small_img.clone(), Some("low".into()))),
            ),
            (
                "2048 detail high",
                Some((large_img.clone(), Some("high".into()))),
            ),
            (
                "2048 detail original",
                Some((large_img.clone(), Some("original".into()))),
            ),
        ];
        for (name, image) in cases {
            let mut mc = ModelCall::new(&model, text);
            mc.effort = Some("low".into());
            mc.max_output_tokens = 300;
            if let Some(i) = image {
                mc.images = vec![i];
            }
            let label = format!("v4-{}-{model}", name.replace(' ', "-"));
            let r = model_call(ctx, "v4", &label, &mc, &prices)?;
            answers.push(json!({"model": model, "case": name, "input_tokens": r.body()["usage"]["input_tokens"], "result": summary(&r)}));
        }
    }
    save(
        ctx,
        "v4",
        &json!({"probe": "V4 image input", "image_bytes": {"1024": small_img.len(), "2048": large_img.len()}, "answers": answers}),
    )
}

/// V5: an image and web search in one request (trusted list as `allowed_domains`); and a
/// subdomain entry in `allowed_domains`.
fn v5(ctx: &Ctx) -> Result<(), String> {
    let prices = ctx.prices()?;
    let image = probe_image(ctx, 1024)?;
    let mid = candidates(ctx)?.get(1).cloned().ok_or("no mid candidate")?;
    let mut answers = Vec::new();
    let mut mc = ModelCall::new(
        &mid,
        "Identify the designer and maker of the furniture piece in the photo by searching the web for listings of the same model. Give the listing URLs you used.",
    );
    mc.images = vec![(image, None)];
    mc.web_search = Some(WebSearch {
        allowed_domains: answer_key::trusted_sites(),
    });
    mc.max_tool_calls = Some(3);
    mc.effort = Some("low".into());
    mc.max_output_tokens = 3_000;
    mc.include = vec!["web_search_call.action.sources".into()];
    let r = model_call(ctx, "v5", "v5-image-and-web-search", &mc, &prices)?;
    answers.push(json!({"case": "image + web_search, allowed_domains = the nine trusted sites", "result": summary(&r), "sources": sources(r.body())}));
    let mut sub = ModelCall::new(
        &mid,
        "Find a teak sideboard listing on this site and give its URL.",
    );
    sub.web_search = Some(WebSearch {
        allowed_domains: vec!["auktion.sweakronan.se".into()],
    });
    sub.max_tool_calls = Some(2);
    sub.effort = Some("low".into());
    sub.max_output_tokens = 2_000;
    sub.include = vec!["web_search_call.action.sources".into()];
    let r = model_call(ctx, "v5", "v5-allowed-subdomain", &sub, &prices)?;
    answers.push(json!({"case": "allowed_domains = [auktion.sweakronan.se] (a subdomain)", "result": summary(&r), "sources": sources(r.body())}));
    save(
        ctx,
        "v5",
        &json!({"probe": "V5 image + web search", "answers": answers}),
    )
}

/// Every web-search action of a reply with its URL and sources.
pub fn sources(body: &Json) -> Json {
    Json::Array(
        body["output"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|o| o["type"] == "web_search_call")
            .map(|o| json!({"status": o["status"], "action": o["action"]}))
            .collect(),
    )
}

/// V6: one search per route on piece 2 (the same request the benchmark makes).
fn v6(ctx: &Ctx) -> Result<(), String> {
    let pieces = answer_key::load_pieces(&ctx.args.test_file)?;
    let piece = pieces.iter().find(|p| p.n == 2).ok_or("piece 2 missing")?;
    let result = bench_search::search_piece(ctx, piece)?;
    let keys = |v: &Json| {
        v.as_object()
            .map(|m| m.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default()
    };
    // Field names of the raw replies, read from the recordings.
    let piece_dir = bench_search::piece_dir(&ctx.out, piece);
    let mut fields = json!({});
    for file in crate::recording::call_files(&piece_dir) {
        let rec: Json = serde_json::from_str(&fs::read_to_string(&file).unwrap_or_default())
            .unwrap_or(Json::Null);
        let body = &rec["response"]["body"];
        let provider = rec["provider"].as_str().unwrap_or("");
        let entry = json!({
            "top_level": keys(body),
            "visual_match_fields": keys(&body["visual_matches"][0]),
            "web_detection_fields": keys(&body["responses"][0]["webDetection"]),
            "page_fields": keys(&body["responses"][0]["webDetection"]["pagesWithMatchingImages"][0]),
            "search_metadata": body["search_metadata"],
        });
        fields[format!("{provider}:{}", rec["id"].as_str().unwrap_or(""))] = entry;
    }
    save(
        ctx,
        "v6",
        &json!({"probe": "V6 result shape and rank", "piece": 2, "result": result, "fields": fields}),
    )
}

/// V7 SerpApi upload: account figures before and after; an upload and a search; an oversized
/// upload; the same `image_id` searched again (no cache) after 11 minutes.
fn v7(ctx: &Ctx) -> Result<(), String> {
    let key = env("SERPAPI_API_KEY")?;
    let d = dir(ctx, "v7");
    let stage = ctx.args.positional.get(1).cloned().unwrap_or_default();
    let mut answers = Vec::new();
    if !stage.starts_with("late") {
        let before = ctx
            .caller
            .call(serpapi::account("v7-account-before", &d, &key))
            .map_err(tool)?;
        let image = probe_image(ctx, 1024)?;
        let up = ctx
            .caller
            .call(serpapi::upload("v7-upload", &d, &key, image.clone()))
            .map_err(tool)?;
        let id = serpapi::image_id(up.body()).unwrap_or_default();
        let search = ctx
            .caller
            .call(serpapi::search(
                "v7-search",
                &d,
                &key,
                &id,
                "visual_matches",
            ))
            .map_err(tool)?;
        let mid = ctx
            .caller
            .call(serpapi::account("v7-account-after-search", &d, &key))
            .map_err(tool)?;
        let big = probe_image(ctx, 2560)?;
        let up_big = ctx
            .caller
            .call(serpapi::upload(
                "v7-upload-oversized",
                &d,
                &key,
                big.clone(),
            ))
            .map_err(tool)?;
        let after = ctx
            .caller
            .call(serpapi::account("v7-account-after", &d, &key))
            .map_err(tool)?;
        answers.push(json!({
            "account_before": account_figures(before.body()),
            "upload": {"call": up.call_id, "status": up.status(), "body": up.body(), "bytes": image.len(), "ms": up.record["duration_ms"]},
            "search": {"call": search.call_id, "status": search.status(), "search_metadata": search.body()["search_metadata"], "error": search.body()["error"], "hits": serpapi::hits(search.body()).len()},
            "account_after_upload_and_search": account_figures(mid.body()),
            "oversized_upload": {"call": up_big.call_id, "status": up_big.status(), "body": up_big.body(), "bytes": big.len()},
            "account_after_oversized": account_figures(after.body()),
            "image_id": id,
            "uploaded_at": up.record["finished"],
        }));
        save(
            ctx,
            "v7",
            &json!({"probe": "V7 SerpApi upload", "answers": answers}),
        )
    } else {
        let early: Json = serde_json::from_str(
            &fs::read_to_string(d.join("answer.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let id = early["answers"][0]["image_id"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        // `probe v7 late` (first try) and `probe v7 late2` (a later try; `output=json`, the
        // default, makes it a distinct request so the first one is not reused).
        let mut late = serpapi::search(
            &format!("v7-search-{stage}"),
            &d,
            &key,
            &id,
            "visual_matches",
        );
        late.url.push_str("&no_cache=true");
        if stage == "late2" {
            late.url.push_str("&output=json");
        }
        let r = ctx.caller.call(late).map_err(tool)?;
        let after = ctx
            .caller
            .call(serpapi::account("v7-account-after-late", &d, &key))
            .map_err(tool)?;
        let mut out = early.clone();
        out[stage.as_str()] = json!({
            "call": r.call_id, "started": r.record["started"], "uploaded_at": early["answers"][0]["uploaded_at"],
            "status": r.status(), "error": r.body()["error"], "search_metadata": r.body()["search_metadata"],
            "hits": serpapi::hits(r.body()).len(), "account_after": account_figures(after.body()),
        });
        save(ctx, "v7", &out)
    }
}

fn account_figures(body: &Json) -> Json {
    json!({
        "plan_name": body["plan_name"], "searches_per_month": body["searches_per_month"],
        "plan_searches_left": body["plan_searches_left"], "total_searches_left": body["total_searches_left"],
        "this_month_usage": body["this_month_usage"], "this_hour_searches": body["this_hour_searches"],
    })
}

/// V8 page reading: for each trusted site and the shop, one listing page by both routes.
/// The page list is `<out>/probes/v8/pages.json` (`[{"site", "url"}]`), chosen from the
/// benchmark's results.
fn v8(ctx: &Ctx) -> Result<(), String> {
    let prices = ctx.prices()?;
    let list: Json = serde_json::from_str(
        &fs::read_to_string(dir(ctx, "v8").join("pages.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mid = candidates(ctx)?.get(1).cloned().ok_or("no mid candidate")?;
    let pieces = answer_key::load_pieces(&ctx.args.test_file)?;
    let shop = bench_search::shop_hosts(&ctx.out, &pieces);
    let mut answers = Vec::new();
    for entry in list.as_array().into_iter().flatten() {
        let site = entry["site"].as_str().unwrap_or("");
        let url = entry["url"].as_str().unwrap_or("");
        let expect: Vec<String> = entry["expect"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect();
        // Route a: plain HTTP.
        let r = ctx
            .caller
            .call(pages::get(
                &format!("v8-http-{}", site.replace('.', "_")),
                &dir(ctx, "v8"),
                url,
            ))
            .map_err(tool)?;
        let html = pages::body_text(&r.record);
        let analysis = pages::analyse(&html);
        let text = pages::strip_tags(&html);
        let http = json!({
            "call": r.call_id, "status": r.status(), "final_url": r.record["response"]["final_url"],
            "redirects": r.record["response"]["redirects"], "analysis": analysis,
            "expected_text_found": expect.iter().map(|e| json!({"text": e, "in_html": answer_key::is_naming(&html, std::slice::from_ref(e)), "in_text": answer_key::is_naming(&text, std::slice::from_ref(e))})).collect::<Vec<_>>(),
            "error": r.record["error"],
            "site_class": format!("{:?}", answer_key::site_of(url, &shop, &answer_key::trusted_sites())),
        });
        // Route b: the model provider's page-open tool restricted to the site's domain.
        let domain = if matches!(
            answer_key::site_of(url, &shop, &answer_key::trusted_sites()),
            SiteClass::Shop
        ) {
            answer_key::host(url).unwrap_or_default()
        } else {
            site.to_owned()
        };
        let mut mc = ModelCall::new(
            &mid,
            &format!(
                "Open this exact page with the web tool and report: (1) the final URL you read, (2) the page title, (3) the designer and maker the page states, word for word, (4) how many product images the page shows. If you cannot open it, say so. Page: {url}"
            ),
        );
        mc.web_search = Some(WebSearch {
            allowed_domains: vec![domain.clone()],
        });
        mc.max_tool_calls = Some(2);
        mc.effort = Some("low".into());
        mc.max_output_tokens = 2_000;
        mc.include = vec!["web_search_call.action.sources".into()];
        let m = model_call(
            ctx,
            "v8",
            &format!("v8-model-{}", site.replace('.', "_")),
            &mc,
            &prices,
        )?;
        answers.push(json!({
            "site": site, "url": url, "http": http,
            "model": {"call": m.call_id, "allowed_domain": domain, "result": summary(&m), "actions": sources(m.body())},
        }));
    }
    save(
        ctx,
        "v8",
        &json!({"probe": "V8 page reading", "answers": answers}),
    )
}

/// V9: the shop's public hosts and a product's storefront URL and handle (one read-only query).
fn v9(ctx: &Ctx) -> Result<(), String> {
    let shop_domain = env("SHOPIFY_SHOP")?;
    let client_id = env("SHOPIFY_CLIENT_ID")?;
    let secret = env("SHOPIFY_CLIENT_SECRET")?;
    let d = dir(ctx, "v9");
    let token_call = shop::token("v9-token", &d, &shop_domain, &client_id, &secret);
    let t = ctx.caller.call(token_call).map_err(tool)?;
    // The token is read from the live reply only (the recording holds `<redacted>`).
    let token = t
        .live_body
        .as_ref()
        .and_then(|b| b["access_token"].as_str())
        .unwrap_or("")
        .to_owned();
    if token.is_empty() || token == crate::recording::REDACTED {
        return Err("no access token (V9 must run fresh; delete probes/v9 to re-run)".into());
    }
    ctx.caller.secrets.borrow_mut().add(&token);
    let pieces = answer_key::load_pieces(&ctx.args.test_file)?;
    let piece = pieces.iter().find(|p| p.n == 2).ok_or("piece 2 missing")?;
    let gid = fs::read_to_string(&ctx.args.test_file)
        .ok()
        .and_then(|t| serde_json::from_str::<Json>(&t).ok())
        .and_then(|j| {
            j.as_array()
                .and_then(|a| a.iter().find(|p| p["n"] == 2).map(|p| p["id"].clone()))
        })
        .unwrap_or(Json::Null);
    let q = ctx
        .caller
        .call(shop::graphql(
            "v9-hosts",
            &d,
            &shop_domain,
            &token,
            shop::HOSTS_QUERY,
            json!({"id": gid}),
        ))
        .map_err(tool)?;
    let data = &q.body()["data"];
    let mut hosts = Vec::new();
    for h in [
        data["shop"]["primaryDomain"]["host"].as_str(),
        data["shop"]["myshopifyDomain"].as_str(),
    ]
    .into_iter()
    .flatten()
    {
        hosts.push(h.to_owned());
    }
    for n in data["webPresences"]["nodes"]
        .as_array()
        .into_iter()
        .flatten()
    {
        if let Some(h) = n["domain"]["host"].as_str() {
            hosts.push(h.to_owned());
        }
    }
    let mut hosts: Vec<String> = hosts
        .iter()
        .map(|h| h.trim_end_matches('.').to_ascii_lowercase())
        .map(|h| h.strip_prefix("www.").map(str::to_owned).unwrap_or(h))
        .collect();
    hosts.sort();
    hosts.dedup();
    save(
        ctx,
        "v9",
        &json!({"probe": "V9 shop hosts", "token_status": t.status(), "query_status": q.status(),
            "errors": q.body()["errors"], "data": data, "hosts": hosts,
            "test_file_handle": piece.handle, "test_file_url": piece.url}),
    )
}

/// V10: a model call cut by a 1 s client timeout.
fn v10(ctx: &Ctx) -> Result<(), String> {
    let prices = ctx.prices()?;
    let image = probe_image(ctx, 1024)?;
    let mid = candidates(ctx)?.get(1).cloned().ok_or("no mid candidate")?;
    let mut mc = ModelCall::new(&mid, OBSERVE_TEXT);
    mc.images = vec![(image, None)];
    mc.schema = Some(("observe".into(), observe_schema()));
    mc.max_output_tokens = 2_000;
    mc.timeout = Duration::from_secs(1);
    let r = model_call(ctx, "v10", "v10-timeout-1s", &mc, &prices)?;
    save(
        ctx,
        "v10",
        &json!({"probe": "V10 client timeout", "call": r.call_id, "sent": r.record["sent"], "error": r.record["error"],
            "duration_ms": r.record["duration_ms"], "charged_in_ledger_micro_usd": r.record["cost_micro_usd"]}),
    )
}

/// Evidence probe 4: the Cloud Vision price page, read by the tool; also one Web Detection error
/// reply (an empty image) for the fixtures.
fn ep4(ctx: &Ctx) -> Result<(), String> {
    let d = dir(ctx, "ep4");
    let r = ctx
        .caller
        .call(pages::get(
            "ep4-vision-pricing",
            &d,
            "https://cloud.google.com/vision/pricing",
        ))
        .map_err(tool)?;
    let text = pages::strip_tags(&pages::body_text(&r.record));
    let snippets: Vec<String> = [
        "Web Detection",
        "First 1000",
        "1001",
        "units/month",
        "per 1000",
        "Free",
    ]
    .iter()
    .filter_map(|k| text.find(k).map(|i| around(&text, i)))
    .collect();
    let key = env("GOOGLE_VISION_API_KEY")?;
    let bad = ctx
        .caller
        .call(cloud_vision::web_detection(
            "ep4-vision-error-empty-image",
            &d,
            &key,
            Vec::new(),
            10,
        ))
        .map_err(tool)?;
    save(
        ctx,
        "ep4",
        &json!({"probe": "Evidence probe 4: Cloud Vision price", "page_call": r.call_id, "status": r.status(),
            "snippets": snippets, "vision_error_reply": {"call": bad.call_id, "status": bad.status(), "body": bad.body()}}),
    )
}

/// About 400 characters of `text` around byte `at` (on character boundaries).
fn around(text: &str, at: usize) -> String {
    let mut start = at.saturating_sub(120);
    while !text.is_char_boundary(start) {
        start -= 1;
    }
    text[start..].chars().take(420).collect()
}
