//! OpenAI Responses API requests (evidence doc: `developers.openai.com/api/docs/guides/…`).

use crate::{
    budget::{ModelPrice, Prices, cost_micro},
    recording::{Call, Costing, OutBody, image_marker},
};
use serde_json::{Value as Json, json};
use std::{path::Path, time::Duration};

pub const RESPONSES_URL: &str = "https://api.openai.com/v1/responses";
pub const PROVIDER: &str = "openai";
pub const CAP: &str = "openai";

#[derive(Debug, Clone)]
pub struct WebSearch {
    pub allowed_domains: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ModelCall {
    pub model: String,
    /// `None` = the provider's default effort (not sent).
    pub effort: Option<String>,
    pub instructions: Option<String>,
    pub text: String,
    /// `(jpeg bytes, detail)`; detail `None` = not sent.
    pub images: Vec<(Vec<u8>, Option<String>)>,
    /// `(name, schema)` for a strict JSON-schema output.
    pub schema: Option<(String, Json)>,
    pub max_output_tokens: u64,
    pub web_search: Option<WebSearch>,
    pub max_tool_calls: Option<u64>,
    pub include: Vec<String>,
    pub timeout: Duration,
}

impl ModelCall {
    pub fn new(model: &str, text: &str) -> Self {
        Self {
            model: model.to_owned(),
            effort: None,
            instructions: None,
            text: text.to_owned(),
            images: Vec::new(),
            schema: None,
            max_output_tokens: 4_000,
            web_search: None,
            max_tool_calls: None,
            include: Vec::new(),
            timeout: Duration::from_secs(180),
        }
    }
}

pub fn body(mc: &ModelCall) -> Json {
    let mut content = vec![json!({"type": "input_text", "text": mc.text})];
    for (i, (_, detail)) in mc.images.iter().enumerate() {
        let mut item = json!({
            "type": "input_image",
            "image_url": format!("data:image/jpeg;base64,{}", image_marker(i)),
        });
        if let Some(d) = detail {
            item["detail"] = json!(d);
        }
        content.push(item);
    }
    let mut body = json!({
        "model": mc.model,
        "input": [{"role": "user", "content": content}],
        "max_output_tokens": mc.max_output_tokens,
        "store": false,
    });
    if let Some(i) = &mc.instructions {
        body["instructions"] = json!(i);
    }
    if let Some(e) = &mc.effort {
        body["reasoning"] = json!({"effort": e});
    }
    if let Some((name, schema)) = &mc.schema {
        body["text"] = json!({"format": {"type": "json_schema", "name": name, "strict": true, "schema": schema}});
    }
    if let Some(ws) = &mc.web_search {
        let mut tool = json!({"type": "web_search"});
        if !ws.allowed_domains.is_empty() {
            tool["filters"] = json!({"allowed_domains": ws.allowed_domains});
        }
        body["tools"] = json!([tool]);
    }
    if let Some(n) = mc.max_tool_calls {
        body["max_tool_calls"] = json!(n);
    }
    if !mc.include.is_empty() {
        body["include"] = json!(mc.include);
    }
    body
}

/// Worst case of a call: every text byte counted as a token, each image at the image bound, each
/// allowed web search at its call price plus the content bound, and `max_output_tokens` output.
pub fn worst_case(mc: &ModelCall, prices: &Prices) -> Result<i64, String> {
    let price = prices.model(&mc.model)?;
    let text_bytes = mc.text.len()
        + mc.instructions.as_ref().map_or(0, String::len)
        + mc.schema.as_ref().map_or(0, |(_, s)| s.to_string().len())
        + 200;
    let searches = if mc.web_search.is_some() {
        mc.max_tool_calls.unwrap_or(10)
    } else {
        0
    };
    // Every input token at the cache-write rate when that is the higher one.
    let in_rate = price.input.max(price.cache_write);
    let input = text_bytes as u64
        + mc.images.len() as u64 * prices.image_tokens_upper_bound
        + searches * prices.web_search_content_tokens_upper_bound;
    Ok(cost_micro(&[
        (input, in_rate),
        (mc.max_output_tokens, price.output),
        (searches, prices.web_search_call),
    ]))
}

/// Web searches of a reply: `tool_usage.web_search.num_requests` (RESEARCH-VERIFY-1), else the
/// number of `web_search_call` items in its output.
pub fn web_search_calls(body: &Json) -> u64 {
    if let Some(n) = body["tool_usage"]["web_search"]["num_requests"].as_u64() {
        return n;
    }
    body["output"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|o| o["type"] == "web_search_call")
        .count() as u64
}

/// Cost from the reported usage: uncached input, cached input, cache writes, output (reasoning
/// included) and web search calls. A reply without usage (an error reply) costs 0.
pub fn costing(price: ModelPrice, web_search_call: i64) -> Costing {
    Box::new(move |_status, body| {
        let usage = &body["usage"];
        if usage.is_null() {
            return (Json::Null, 0);
        }
        let input = usage["input_tokens"].as_u64().unwrap_or(0);
        let cached = usage["input_tokens_details"]["cached_tokens"]
            .as_u64()
            .unwrap_or(0);
        let written = usage["input_tokens_details"]["cache_write_tokens"]
            .as_u64()
            .unwrap_or(0);
        let output = usage["output_tokens"].as_u64().unwrap_or(0);
        let searches = web_search_calls(body);
        let cost = cost_micro(&[
            (
                input.saturating_sub(cached).saturating_sub(written),
                price.input,
            ),
            (cached, price.cached_input),
            (written, price.cache_write),
            (output, price.output),
            (searches, web_search_call),
        ]);
        let mut recorded = usage.clone();
        recorded["web_search_call"] = json!(searches);
        (recorded, cost)
    })
}

pub fn call(
    label: &str,
    dir: &Path,
    key: &str,
    mc: &ModelCall,
    prices: &Prices,
) -> Result<Call, String> {
    let price = prices.model(&mc.model)?;
    let mut call = Call::new(label, dir, PROVIDER, CAP, "POST", RESPONSES_URL);
    call.model = mc.model.clone();
    call.effort = mc.effort.clone().unwrap_or_else(|| "default".into());
    call.headers = vec![("authorization".into(), format!("Bearer {key}"))];
    call.body = OutBody::Json(body(mc));
    call.images = mc.images.iter().map(|(b, _)| b.clone()).collect();
    call.worst_case_micro = worst_case(mc, prices)?;
    call.timeout = mc.timeout;
    call.costing = costing(price, prices.web_search_call);
    Ok(call)
}

/// The concatenated `output_text` of a reply.
pub fn output_text(body: &Json) -> String {
    let mut out = String::new();
    for item in body["output"].as_array().into_iter().flatten() {
        for c in item["content"].as_array().into_iter().flatten() {
            if c["type"] == "output_text"
                && let Some(t) = c["text"].as_str()
            {
                out.push_str(t);
            }
        }
    }
    out
}

/// The refusal text of a reply, if the model refused.
pub fn refusal(body: &Json) -> Option<String> {
    body["output"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|i| i["content"].as_array().into_iter().flatten())
        .find(|c| c["type"] == "refusal")
        .map(|c| c["refusal"].as_str().unwrap_or("").to_owned())
}

/// Recomputes `cost_micro_usd` of every recorded OpenAI call under `out` from its recorded
/// usage and the current `prices.json` (after a price-table correction). Returns the new total.
pub fn recost(
    out: &Path,
    prices: &Prices,
    secrets: &crate::recording::Secrets,
) -> Result<i64, String> {
    let mut total = 0;
    for file in crate::recording::call_files(out) {
        let text = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
        let mut rec: Json = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if rec["provider"] != PROVIDER || rec["response"].is_null() {
            total += rec["cost_micro_usd"].as_i64().unwrap_or(0);
            continue;
        }
        let model = rec["model"].as_str().unwrap_or("").to_owned();
        let price = prices.model(&model)?;
        let status = rec["response"]["status"].as_u64().unwrap_or(0) as u16;
        let (usage, cost) =
            costing(price, prices.web_search_call)(status, &rec["response"]["body"]);
        rec["usage"] = usage;
        rec["cost_micro_usd"] = json!(cost);
        total += cost;
        let bytes = serde_json::to_vec_pretty(&rec).unwrap_or_default();
        crate::recording::write_scanned(secrets, &file, &bytes).map_err(|e| e.to_string())?;
    }
    Ok(total)
}
