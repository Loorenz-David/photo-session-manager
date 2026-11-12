//! `psm-research fixtures --out <a01> --dest crates/research/fixtures`: copies sanitized captures
//! into the research crate's fixtures folder (charter rule 17: provider shapes are captures, not
//! hand-written). Sanitizing: the test store's domain and name become `example.myshopify.com` /
//! `example`; account identifiers in headers and error details become placeholders; long request
//! strings are cut; result lists are cut to 3; a third-party page keeps only status, headers,
//! final URL and sizes. Every written file is scanned for every key value first (fail closed).

use crate::{
    Ctx,
    recording::{Secrets, call_files, write_scanned},
};
use serde_json::{Value as Json, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

const ACCOUNT_HEADERS: [&str; 3] = ["openai-organization", "openai-project", "cf-ray"];

fn find(out: &Path, sub: &str, prefix: &str) -> Result<Json, String> {
    let dir = out.join(sub);
    for file in call_files(&dir) {
        let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with(prefix) {
            let text = fs::read_to_string(&file).map_err(|e| e.to_string())?;
            return serde_json::from_str(&text).map_err(|e| e.to_string());
        }
    }
    Err(format!("no capture {sub}/…/{prefix}*"))
}

fn cut_strings(v: &mut Json, max: usize) {
    match v {
        Json::String(s) if s.chars().count() > max => {
            let n = s.chars().count();
            *s = format!(
                "{}…<cut: {} more characters>",
                s.chars().take(max).collect::<String>(),
                n - max
            );
        }
        Json::Array(a) => a.iter_mut().for_each(|x| cut_strings(x, max)),
        Json::Object(m) => m.values_mut().for_each(|x| cut_strings(x, max)),
        _ => {}
    }
}

fn cut_lists(v: &mut Json, keys: &[&str], n: usize) {
    if let Json::Object(m) = v {
        for (k, x) in m.iter_mut() {
            if keys.contains(&k.as_str())
                && let Json::Array(a) = x
            {
                a.truncate(n);
            }
            cut_lists(x, keys, n);
        }
    } else if let Json::Array(a) = v {
        a.iter_mut().for_each(|x| cut_lists(x, keys, n));
    }
}

/// The text replacements every fixture gets.
fn replacements() -> Vec<(String, String)> {
    let mut r = Vec::new();
    if let Ok(shop) = std::env::var("SHOPIFY_SHOP") {
        let shop = shop.trim().to_owned();
        if let Some(handle) = shop.strip_suffix(".myshopify.com") {
            r.push((shop.clone(), "example.myshopify.com".into()));
            r.push((handle.to_owned(), "example".into()));
        }
    }
    r.push(("test-item-position-scanner".into(), "example".into()));
    if let Ok(id) = std::env::var("SHOPIFY_CLIENT_ID") {
        r.push((id.trim().to_owned(), "<client-id>".into()));
    }
    r
}

fn sanitize(mut rec: Json, source: &str, probe: &str) -> Json {
    for side in ["request", "response"] {
        if let Some(h) = rec[side]["headers"].as_object_mut() {
            for k in ACCOUNT_HEADERS {
                if h.contains_key(k) {
                    h.insert(k.into(), json!(format!("<{k}>")));
                }
            }
        }
    }
    cut_strings(&mut rec["request"]["body"], 2_000);
    let mut text = rec.to_string();
    for (from, to) in replacements() {
        text = text.replace(&from, &to);
    }
    // Google project numbers (after "project", e.g. `projects/123…`, `project=123…`) everywhere.
    let mut numbers = Vec::new();
    let mut rest = text.as_str();
    while let Some(i) = rest.find("project") {
        rest = &rest[i + 7..];
        let skip = rest
            .find(|c: char| !matches!(c, '=' | '/' | 's' | ' ' | '#'))
            .unwrap_or(0);
        let digits: String = rest[skip..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if digits.len() >= 6 {
            numbers.push(digits);
        }
    }
    let mut out = text.clone();
    for n in numbers {
        out = out.replace(&n, "<project-number>");
    }
    let mut rec: Json = serde_json::from_str(&out).unwrap_or(Json::Null);
    rec["fixture"] = json!({"source": source, "probe": probe, "captured": rec["started"].clone(), "sanitized": "R1·U1 psm-research fixtures"});
    rec
}

pub fn run(ctx: &Ctx) -> Result<(), String> {
    let out = &ctx.out;
    let dest: PathBuf = ctx
        .args
        .positional
        .first()
        .map(PathBuf::from)
        .ok_or("fixtures needs the destination folder")?;
    let secrets = {
        let mut s = Secrets::from_env();
        if let Ok(id) = std::env::var("SHOPIFY_CLIENT_ID") {
            s.add(&id);
        }
        s
    };
    let mut readme = String::from(
        "# Research fixtures (sanitized provider captures)\n\nWritten by `psm-research fixtures` (R1, 2026-11-12) from the recordings in `~/psm-research-bench/a01/`; never edited by hand (charter rule 17). Cite as `fixtures/<provider>/<file> (RESEARCH-VERIFY-n)`. Sanitizing: test-store domain → `example.myshopify.com`, account headers and Google project numbers → placeholders, request strings over 2,000 characters cut, result lists cut to 3, third-party page bodies omitted. Every file passed a scan for the three provider keys and the shop's client id and secret.\n\n| File | Probe | What it is | Captured |\n|---|---|---|---|\n",
    );
    let mut items: Vec<(String, Json, String, String)> = Vec::new();
    let mut add = |file: &str, rec: Json, probe: &str, what: &str| {
        items.push((file.to_owned(), rec, probe.to_owned(), what.to_owned()))
    };
    let models = ["gpt-6.1-sol", "gpt-6.1-terra", "gpt-6-luna"];
    for m in models {
        add(
            &format!("openai/image_strict_schema_{m}.json"),
            find(out, "probes/v1", &format!("v1-image-schema-{m}"))?,
            "RESEARCH-VERIFY-1, -3, -4",
            "ok reply: one image + strict JSON schema; usage of this candidate model",
        );
    }
    add(
        "openai/incomplete_max_output_tokens.json",
        find(out, "probes/v1", "v1-incomplete")?,
        "RESEARCH-VERIFY-1, -3",
        "`status: incomplete`, reason max_output_tokens, usage reported",
    );
    add(
        "openai/web_search.json",
        find(out, "probes/v1", "v1-web-search")?,
        "RESEARCH-VERIFY-1",
        "web_search reply: `tool_usage.web_search.num_requests`, sources",
    );
    add(
        "openai/image_and_web_search.json",
        find(out, "probes/v5", "v5-image-and-web-search")?,
        "RESEARCH-VERIFY-5",
        "image + web_search in one request, allowed_domains = trusted list",
    );
    add(
        "openai/error_invalid_key.json",
        find(out, "probes/v2", "v2-invalid-key")?,
        "RESEARCH-VERIFY-2",
        "HTTP 401 invalid_api_key (a made-up key string)",
    );
    add(
        "openai/error_context_length.json",
        find(out, "probes/v2", "v2-oversized-input")?,
        "RESEARCH-VERIFY-2",
        "HTTP 400 context_length_exceeded (request text cut)",
    );
    add(
        "openai/page_open_tool.json",
        find(out, "probes/v8", "v8-model-auctionet_com")?,
        "RESEARCH-VERIFY-8",
        "web_search open_page restricted to one domain",
    );
    // U2 replies (one per stage) when present.
    for stage in ["observe", "read", "condition"] {
        let completed: Vec<Json> =
            call_files(Path::new(&ctx.args.bench1.clone().unwrap_or_default()))
                .into_iter()
                .filter_map(|f| fs::read_to_string(f).ok())
                .filter_map(|t| serde_json::from_str::<Json>(&t).ok())
                .filter(|r| {
                    r["meta"]["stage"] == stage
                        && r["response"]["status"] == 200
                        && r["response"]["body"]["status"] == "completed"
                })
                .collect();
        // A read reply about the owner's own shop when there is one (no third-party page text).
        let pick = completed
            .iter()
            .find(|r| {
                r["meta"]["url"]
                    .as_str()
                    .is_some_and(|u| u.contains("beyovintage.se"))
            })
            .or_else(|| completed.first())
            .cloned();
        if let Some(mut rec) = pick {
            if stage == "read"
                && !rec["meta"]["url"]
                    .as_str()
                    .is_some_and(|u| u.contains("beyovintage.se"))
            {
                omit_page_text(&mut rec["request"]["body"]);
            }
            let model = rec["model"].as_str().unwrap_or("").to_owned();
            add(
                &format!("openai/stage_{stage}_{model}.json"),
                rec,
                "RESEARCH-VERIFY-3 (R1·U2)",
                &format!("stage {stage}, version-0 instructions, strict schema reply"),
            );
        }
    }
    let mut search = find(out, "2-square-teak-dining-table", "serpapi-search")?;
    cut_lists(
        &mut search["response"]["body"],
        &[
            "visual_matches",
            "organic_results",
            "references",
            "text_blocks",
            "related_content",
            "short_videos",
        ],
        3,
    );
    add(
        "serpapi_lens/search_cut_to_3.json",
        search,
        "RESEARCH-VERIFY-6",
        "Google Lens search by image_id, type=all; lists cut to 3",
    );
    add(
        "serpapi_lens/upload.json",
        find(out, "probes/v7", "v7-upload-d")?,
        "RESEARCH-VERIFY-7",
        "image upload reply (`image_id`)",
    );
    add(
        "serpapi_lens/error_upload_too_large.json",
        find(out, "probes/v7", "v7-upload-oversized")?,
        "RESEARCH-VERIFY-7",
        "HTTP 400, file over 500 KB",
    );
    add(
        "serpapi_lens/error_image_id_expired.json",
        find(out, "probes/v7", "v7-search-late2")?,
        "RESEARCH-VERIFY-7",
        "HTTP 400 \"Image not found.\" 11.9 min after upload",
    );
    add(
        "serpapi_lens/account.json",
        find(out, "probes/v7", "v7-account-after-late")?,
        "RESEARCH-VERIFY-7",
        "account figures (credits)",
    );
    add(
        "cloud_vision/error_billing_disabled.json",
        find(out, "2-square-teak-dining-table", "cloud-vision")?,
        "RESEARCH-VERIFY-6",
        "HTTP 403 BILLING_DISABLED (no web detection reply could be captured)",
    );
    add(
        "shop/hosts_query.json",
        find(out, "probes/v9", "v9-hosts")?,
        "RESEARCH-VERIFY-9",
        "read-only Admin query on the test store: primaryDomain, myshopifyDomain; webPresences needs read_markets",
    );
    add(
        "shop/token.json",
        find(out, "probes/v9", "v9-token")?,
        "RESEARCH-VERIFY-9",
        "client credentials grant reply (token redacted)",
    );
    // Pages: the shop's own listing page in full; third-party pages without body.
    let mut shop_page = None;
    let mut others = Vec::new();
    for file in call_files(&out.join("pages"))
        .into_iter()
        .chain(call_files(&out.join("probes/v8")))
    {
        let rec: Json = serde_json::from_str(&fs::read_to_string(&file).unwrap_or_default())
            .unwrap_or(Json::Null);
        if rec["provider"] != crate::pages::PROVIDER {
            continue;
        }
        let url = rec["request"]["url"].as_str().unwrap_or("").to_owned();
        let host = crate::answer_key::host(&url).unwrap_or_default();
        if host == "beyovintage.se" {
            if shop_page.is_none() && rec["response"]["status"] == 200 && url.contains("/products/")
            {
                shop_page = Some(rec);
            }
        } else if file.starts_with(out.join("probes/v8")) {
            others.push((host, rec));
        }
    }
    if let Some(rec) = shop_page {
        add(
            "pages/shop_listing_page.json",
            rec,
            "RESEARCH-VERIFY-8",
            "the owner's shop: a product page read by plain HTTP, in full",
        );
    }
    let mut shop_429 = find(out, "probes/v8", "v8-http-beyovintage_se")?;
    shop_429["response"]["body"] = json!(null);
    shop_429["response"]["body_omitted"] = json!("throttle page");
    add(
        "pages/shop_429_throttled.json",
        shop_429,
        "RESEARCH-VERIFY-8",
        "the owner's shop: HTTP 429 with Retry-After after a burst of plain fetches",
    );
    for (host, mut rec) in others {
        rec["response"]["body"] = Json::Null;
        rec["response"]["body_omitted"] = json!("third-party page body not committed");
        add(
            &format!("pages/{}.json", host.replace('.', "_")),
            rec,
            "RESEARCH-VERIFY-8",
            "plain HTTP GET: status, headers, final URL, sizes (no body)",
        );
    }
    for (file, rec, probe, what) in items {
        let captured = rec["started"].as_str().unwrap_or("").to_owned();
        let clean = sanitize(rec, &file, &probe);
        let bytes = serde_json::to_vec_pretty(&clean).unwrap_or_default();
        write_scanned(&secrets, &dest.join(&file), &bytes).map_err(|e| format!("{file}: {e}"))?;
        readme.push_str(&format!("| `{file}` | {probe} | {what} | {captured} |\n"));
    }
    write_scanned(&secrets, &dest.join("README.md"), readme.as_bytes())
        .map_err(|e| e.to_string())?;
    eprintln!("wrote fixtures to {}", dest.display());
    Ok(())
}

/// Replaces everything after "Page text:" in every input text with an omission note.
fn omit_page_text(v: &mut Json) {
    match v {
        Json::String(t) => {
            if let Some(i) = t.find("Page text:") {
                t.truncate(i + "Page text:".len());
                t.push_str(" <third-party page text omitted>");
            }
        }
        Json::Array(a) => a.iter_mut().for_each(omit_page_text),
        Json::Object(m) => m.values_mut().for_each(omit_page_text),
        _ => {}
    }
}
