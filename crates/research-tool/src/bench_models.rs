//! First per-stage model comparison (R1·U2): observe (visible fields from the cover photo),
//! read (what a listing page states), condition (severity fields from the condition photos).
//! Every call carries `meta` {stage, n, unit, …} so the scorer reads the recordings alone.

use crate::{
    Ctx,
    answer_key::{self, Bucket, KeyEntry, Piece},
    bench_search::{self, first_ten, hits_from},
    openai::{self, ModelCall},
    pages,
    recording::{ToolError, call_files},
    report::page_texts,
    score::{self, KeyKind, Outcome},
};
use serde_json::{Value as Json, json};
use std::{
    collections::BTreeMap,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

pub const STAGES: [&str; 3] = ["observe", "read", "condition"];
pub const OBSERVE_INSTRUCTIONS: &str = include_str!("instructions/observe.md");
pub const READ_INSTRUCTIONS: &str = include_str!("instructions/read.md");
pub const CONDITION_INSTRUCTIONS: &str = include_str!("instructions/condition.md");
/// Page text sent to the read stage, in characters.
pub const PAGE_TEXT_CHARS: usize = 12_000;
pub const MAX_CONDITION_IMAGES: usize = 6;

/// The live shop's definitions of the scored keys (`claude_design/reference/
/// SHOPIFY_PRODUCT_METAFIELDS.md`, read from Shopify on 2026-10-24): the choice strings given
/// to the model (MC-A2).
pub const WOOD_TYPES: [&str; 13] = [
    "Teak",
    "Dark Teak",
    "Oak",
    "Dark Oak",
    "Santos Rosewood",
    "Mahogany",
    "Walnut",
    "Cherry",
    "Beech",
    "Birch",
    "Elm",
    "Pine",
    "Chrome/Steel",
];
pub const SEVERITIES: [&str; 5] = ["N/A", "None", "Minor", "Moderate", "Major"];
pub const OBSERVE_KEYS: [&str; 1] = ["wood_type"];
pub const READ_KEYS: [&str; 2] = ["designer", "manufacturer"];

pub fn kind(field: &str) -> Option<KeyKind> {
    let list = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    match field {
        "wood_type" => Some(KeyKind::MultiChoice(list(&WOOD_TYPES))),
        f if f.starts_with("condition_") && f != "condition_report_description" => {
            Some(KeyKind::Choice(list(&SEVERITIES)))
        }
        "designer" | "manufacturer" => Some(KeyKind::Text),
        _ => None,
    }
}

/// The condition fields of a piece's layout (the condition metafields its key carries).
pub fn condition_fields(piece: &Piece) -> Vec<String> {
    piece
        .metafields
        .as_object()
        .into_iter()
        .flatten()
        .map(|(k, _)| k.clone())
        .filter(|k| matches!(kind(k), Some(KeyKind::Choice(_))))
        .collect()
}

fn choice_value(choices: &[&str], multi: bool) -> Json {
    let item = json!({"type": "string", "enum": choices});
    if multi {
        json!({"type": "array", "items": item})
    } else {
        json!({"anyOf": [item, {"type": "null"}]})
    }
}

pub fn observe_schema() -> Json {
    json!({
        "type": "object", "additionalProperties": false,
        "required": ["piece_type", "shows_marking", "fields"],
        "properties": {
            "piece_type": {"type": "string"},
            "shows_marking": {"type": "boolean"},
            "fields": {
                "type": "object", "additionalProperties": false,
                "required": ["wood_type"],
                "properties": {"wood_type": {
                    "type": "object", "additionalProperties": false, "required": ["value", "sure"],
                    "properties": {"value": choice_value(&WOOD_TYPES, true), "sure": {"type": "boolean"}}
                }}
            }
        }
    })
}

pub fn read_schema() -> Json {
    let key = json!({
        "type": "object", "additionalProperties": false, "required": ["stated", "value"],
        "properties": {"stated": {"type": ["string", "null"]}, "value": {"type": ["string", "null"]}}
    });
    json!({
        "type": "object", "additionalProperties": false, "required": ["keys"],
        "properties": {"keys": {
            "type": "object", "additionalProperties": false, "required": READ_KEYS,
            "properties": {"designer": key.clone(), "manufacturer": key}
        }}
    })
}

pub fn condition_schema(fields: &[String]) -> Json {
    let field = json!({
        "type": "object", "additionalProperties": false, "required": ["value", "sure"],
        "properties": {"value": choice_value(&SEVERITIES, false), "sure": {"type": "boolean"}}
    });
    let props: serde_json::Map<String, Json> =
        fields.iter().map(|f| (f.clone(), field.clone())).collect();
    json!({
        "type": "object", "additionalProperties": false, "required": ["fields"],
        "properties": {"fields": {
            "type": "object", "additionalProperties": false, "required": fields, "properties": props
        }}
    })
}

/// One model call of the matrix.
pub struct Planned {
    pub label: String,
    pub dir: PathBuf,
    pub mc: ModelCall,
    pub meta: Json,
}

fn chars(text: &str, n: usize) -> String {
    text.chars().take(n).collect()
}

/// A reduced copy: R1·U1's file when it exists, else made now (downloaded into this out folder).
fn reduced(ctx: &Ctx, bench1: &Path, url: &str, edge: u32) -> Result<Vec<u8>, String> {
    let name = format!(
        "{}-{edge}.jpg",
        &crate::recording::sha256_hex(url.as_bytes())[..24]
    );
    if let Ok(bytes) = fs::read(bench1.join("reduced").join(&name)) {
        return Ok(bytes);
    }
    bench_search::reduced(ctx, url, edge)
}

/// A page R1·U1 fetched: (text, names the key per key name).
pub struct PageInput {
    pub url: String,
    pub text: String,
    pub naming: bool,
}

/// Read-stage pages of a piece: the first naming result, then the first two non-naming results
/// on the shop or a trusted site, among R1·U1's first ten (Lens route), each fetched with HTTP 200.
pub fn read_pages(
    bench1: &Path,
    piece: &Piece,
    names: &[String],
    shop: &[String],
    texts: &BTreeMap<String, (Option<u16>, String)>,
) -> Vec<PageInput> {
    let Ok(text) = fs::read_to_string(bench_search::piece_dir(bench1, piece).join("results.json"))
    else {
        return Vec::new();
    };
    let result: Json = serde_json::from_str(&text).unwrap_or(Json::Null);
    let (ten, _) = first_ten(&hits_from(&result["serpapi_lens"]["hits"]), piece, shop);
    let trusted = answer_key::trusted_sites();
    let readable = |url: &str| {
        texts
            .get(url)
            .filter(|(s, t)| *s == Some(200) && !t.is_empty())
            .map(|(_, t)| t.clone())
    };
    let mut out = Vec::new();
    if let Some((h, t)) = ten
        .iter()
        .filter_map(|h| readable(&h.url).map(|t| (h, t)))
        .find(|(h, t)| answer_key::is_naming(&h.title, names) || answer_key::is_naming(t, names))
    {
        out.push(PageInput {
            url: h.url.clone(),
            naming: answer_key::is_naming(&t, names),
            text: t,
        });
    }
    for h in &ten {
        if out.len() >= 3 {
            break;
        }
        if matches!(
            answer_key::site_of(&h.url, shop, &trusted),
            answer_key::SiteClass::Other(_)
        ) || out.iter().any(|p| p.url == h.url)
        {
            continue;
        }
        if let Some(t) = readable(&h.url)
            && !answer_key::is_naming(&h.title, names)
            && !answer_key::is_naming(&t, names)
        {
            out.push(PageInput {
                url: h.url.clone(),
                naming: false,
                text: t,
            });
        }
    }
    out
}

fn effort_label(e: &Option<String>) -> String {
    e.clone().unwrap_or_else(|| "default".into())
}

pub fn plan(ctx: &Ctx) -> Result<Vec<Planned>, String> {
    let bench1 = ctx
        .args
        .bench1
        .clone()
        .ok_or("--bench1 <R1·U1 out folder> is required")?;
    let pieces = bench_search::selected(ctx)?;
    let all = answer_key::load_pieces(&ctx.args.test_file)?;
    let corrections = answer_key::load_corrections(&ctx.args.corrections)?;
    let entries = answer_key::key_entries(&all, &corrections);
    let shop = bench_search::shop_hosts(&bench1, &all);
    let texts = page_texts(&bench1);
    let models = if ctx.args.models.is_empty() {
        crate::probes::candidates(ctx)?
    } else {
        ctx.args.models.clone()
    };
    let efforts: Vec<Option<String>> = if ctx.args.efforts.is_empty() {
        vec![None]
    } else {
        ctx.args
            .efforts
            .iter()
            .map(|e| (e != "default").then(|| e.clone()))
            .collect()
    };
    let stages: Vec<String> = if ctx.args.stages.is_empty() {
        STAGES.iter().map(|s| (*s).to_owned()).collect()
    } else {
        ctx.args.stages.clone()
    };
    let mut planned = Vec::new();
    for piece in &pieces {
        let dir = ctx
            .out
            .join(format!("{}-{}", piece.n, piece.handle))
            .join("run-bench2");
        for stage in &stages {
            // Inputs per stage, then one call per (model, effort).
            let mut units: Vec<(String, ModelCall, Json)> = Vec::new();
            match stage.as_str() {
                "observe" => {
                    let cover = piece.images.first().ok_or("piece has no image")?;
                    let jpeg = reduced(ctx, &bench1, cover, 1024)?;
                    let mut mc = ModelCall::new(
                        "",
                        "Fields:\n- wood_type (multi choice): the visible wood or material of the piece.",
                    );
                    mc.instructions = Some(OBSERVE_INSTRUCTIONS.to_owned());
                    mc.images = vec![(jpeg, None)];
                    mc.schema = Some(("observe".into(), observe_schema()));
                    units.push(("cover".into(), mc, json!({})));
                }
                "read" if piece.has_attribution() => {
                    let names: Vec<String> = answer_key::key_names(piece.n, &entries)
                        .into_iter()
                        .map(|(n, _)| n)
                        .collect();
                    for (i, page) in read_pages(&bench1, piece, &names, &shop, &texts)
                        .into_iter()
                        .enumerate()
                    {
                        let text = format!(
                            "Keys:\n- designer (text): the person who designed the piece.\n- manufacturer (text): the company or workshop that made it.\n\nPage URL: {}\nPage text:\n{}",
                            page.url,
                            chars(&page.text, PAGE_TEXT_CHARS)
                        );
                        let mut mc = ModelCall::new("", &text);
                        mc.instructions = Some(READ_INSTRUCTIONS.to_owned());
                        mc.schema = Some(("read".into(), read_schema()));
                        let naming: BTreeMap<String, bool> = READ_KEYS
                            .iter()
                            .filter_map(|k| {
                                entries
                                    .iter()
                                    .find(|e| e.n == piece.n && e.field == *k)
                                    .and_then(|e| {
                                        e.value.as_str().map(|v| {
                                            (
                                                (*k).to_owned(),
                                                answer_key::is_naming(&page.text, &[v.to_owned()]),
                                            )
                                        })
                                    })
                            })
                            .collect();
                        units.push((
                            format!("page{}", i + 1),
                            mc,
                            json!({"url": page.url, "page_names_any_key": page.naming, "page_names": naming}),
                        ));
                    }
                }
                "condition" if !piece.condition_images.is_empty() => {
                    let fields = condition_fields(piece);
                    if fields.is_empty() {
                        continue;
                    }
                    let mut images = Vec::new();
                    for url in piece.condition_images.iter().take(MAX_CONDITION_IMAGES) {
                        images.push((reduced(ctx, &bench1, url, 2048)?, None));
                    }
                    let list = fields
                        .iter()
                        .map(|f| format!("- {f} (choice)"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let mut mc = ModelCall::new("", &format!("Condition fields:\n{list}"));
                    mc.instructions = Some(CONDITION_INSTRUCTIONS.to_owned());
                    mc.images = images;
                    mc.schema = Some(("condition".into(), condition_schema(&fields)));
                    mc.max_output_tokens = 6_000;
                    units.push(("condition".into(), mc, json!({"fields": fields})));
                }
                _ => {}
            }
            for (unit, mc, extra) in units {
                for model in &models {
                    for effort in &efforts {
                        let mut mc = mc.clone();
                        mc.model = model.clone();
                        mc.effort = effort.clone();
                        if mc.max_output_tokens < 6_000 {
                            mc.max_output_tokens = 4_000;
                        }
                        let mut meta = json!({"stage": stage, "n": piece.n, "unit": unit, "model": model, "effort": effort_label(effort)});
                        if let (Some(m), Some(x)) = (meta.as_object_mut(), extra.as_object()) {
                            m.extend(x.clone());
                        }
                        planned.push(Planned {
                            label: format!("{stage}-{unit}-{model}-{}", effort_label(effort)),
                            dir: dir.clone(),
                            mc,
                            meta,
                        });
                    }
                }
            }
        }
    }
    Ok(planned)
}

/// Makes the planned calls in order; a call recorded earlier is not made again (resume). Stops at
/// the first refusal (ceiling or cap) or secret hit.
pub fn execute(ctx: &Ctx, planned: Vec<Planned>, key: &str) -> Result<(), String> {
    let prices = ctx.prices()?;
    for p in planned {
        let mut call = openai::call(&p.label, &p.dir, key, &p.mc, &prices)?;
        call.meta = p.meta;
        match ctx.caller.call(call) {
            Ok(r) if !r.from_disk => eprintln!(
                "{}: HTTP {:?}, {} micro-USD",
                r.call_id,
                r.status(),
                r.record["cost_micro_usd"]
            ),
            Ok(_) => {}
            Err(e @ (ToolError::Refused(_) | ToolError::SecretInRecording { .. })) => {
                return Err(e.to_string());
            }
            Err(e) => eprintln!("{}: {e}", p.label),
        }
    }
    Ok(())
}

pub fn run(ctx: &Ctx) -> Result<(), String> {
    let planned = plan(ctx)?;
    eprintln!("bench-models: {} planned call(s)", planned.len());
    execute(ctx, planned, &bench_search::env("OPENAI_API_KEY")?)
}

/// The parsed structured output of a recorded reply, or why there is none.
pub fn parsed(record: &Json) -> Result<Json, &'static str> {
    let body = &record["response"]["body"];
    if record["response"].is_null() {
        return Err("no reply");
    }
    if record["response"]["status"].as_u64() != Some(200) {
        return Err("http error");
    }
    if openai::refusal(body).is_some() {
        return Err("refusal");
    }
    if body["status"] != "completed" {
        return Err("schema failure");
    }
    serde_json::from_str(&openai::output_text(body)).map_err(|_| "schema failure")
}

#[derive(Default, Clone)]
struct Tally {
    counts: BTreeMap<(Bucket, Outcome), u32>,
    schema_failures: u32,
    refusals: u32,
    http_errors: u32,
    calls: u32,
    cost: i64,
    ms: u64,
    pieces: std::collections::BTreeSet<u32>,
}

/// One scored answer for the disagreement list.
struct Wrong {
    stage: String,
    model: String,
    effort: String,
    n: u32,
    field: String,
    key: Json,
    answer: Json,
    bucket: Bucket,
    unit: String,
}

pub fn write_report(ctx: &Ctx) -> Result<(), String> {
    let all = answer_key::load_pieces(&ctx.args.test_file)?;
    let corrections = answer_key::load_corrections(&ctx.args.corrections)?;
    let entries = answer_key::key_entries(&all, &corrections);
    let mut tallies: BTreeMap<(String, String, String), Tally> = BTreeMap::new();
    let mut wrongs: Vec<Wrong> = Vec::new();
    // Read stage split by whether the page names the key: (model, effort, names) → outcome counts.
    let mut read_split: BTreeMap<(String, String, bool), BTreeMap<Outcome, u32>> = BTreeMap::new();
    for file in call_files(&ctx.out) {
        let Ok(rec) = serde_json::from_str::<Json>(&fs::read_to_string(&file).unwrap_or_default())
        else {
            continue;
        };
        let meta = &rec["meta"];
        let Some(stage) = meta["stage"].as_str() else {
            continue;
        };
        let model = meta["model"].as_str().unwrap_or("").to_owned();
        let effort = meta["effort"].as_str().unwrap_or("").to_owned();
        let n = meta["n"].as_u64().unwrap_or(0) as u32;
        let unit = meta["unit"].as_str().unwrap_or("").to_owned();
        let t = tallies
            .entry((stage.to_owned(), model.clone(), effort.clone()))
            .or_default();
        t.calls += 1;
        t.cost += rec["cost_micro_usd"].as_i64().unwrap_or(0);
        t.ms += rec["duration_ms"].as_u64().unwrap_or(0);
        t.pieces.insert(n);
        let out = match parsed(&rec) {
            Ok(o) => o,
            Err("refusal") => {
                t.refusals += 1;
                continue;
            }
            Err("schema failure") => {
                t.schema_failures += 1;
                continue;
            }
            Err(_) => {
                t.http_errors += 1;
                continue;
            }
        };
        let mut record =
            |field: &str, answer: Option<&Json>, outcome: Option<(Bucket, Outcome)>| {
                if let Some((bucket, o)) = outcome {
                    *t.counts.entry((bucket, o)).or_default() += 1;
                    if o == Outcome::Wrong {
                        wrongs.push(Wrong {
                            stage: stage.to_owned(),
                            model: model.clone(),
                            effort: effort.clone(),
                            n,
                            field: field.to_owned(),
                            key: key_value(&entries, n, field),
                            answer: answer.cloned().unwrap_or(Json::Null),
                            bucket,
                            unit: unit.clone(),
                        });
                    }
                }
            };
        match stage {
            "observe" => {
                for field in OBSERVE_KEYS {
                    let v = &out["fields"][field]["value"];
                    let answer = (!v.as_array().is_some_and(Vec::is_empty)).then_some(v);
                    let kind = kind(field).unwrap_or(KeyKind::Text);
                    record(
                        field,
                        answer,
                        score::score_field(&entries, n, field, &kind, answer),
                    );
                }
            }
            "read" => {
                for field in READ_KEYS {
                    let Some(entry) = entries.iter().find(|e| e.n == n && e.field == field) else {
                        continue;
                    };
                    let key = entry.value.as_str().unwrap_or("");
                    let names = meta["page_names"][field].as_bool().unwrap_or(false);
                    let answer = out["keys"][field]["value"].as_str();
                    let o = score::score_read(names, key, answer);
                    *read_split
                        .entry((model.clone(), effort.clone(), names))
                        .or_default()
                        .entry(o)
                        .or_default() += 1;
                    let answer_json = answer.map(|a| json!(a));
                    record(field, answer_json.as_ref(), Some((entry.bucket, o)));
                }
            }
            "condition" => {
                for field in meta["fields"].as_array().into_iter().flatten() {
                    let field = field.as_str().unwrap_or("");
                    let v = &out["fields"][field]["value"];
                    let answer = (!v.is_null()).then_some(v);
                    let kind = kind(field).unwrap_or(KeyKind::Text);
                    record(
                        field,
                        answer,
                        score::score_field(&entries, n, field, &kind, answer),
                    );
                }
            }
            _ => {}
        }
    }
    let mut md = String::new();
    let _ = writeln!(
        md,
        "# Benchmark 2 — first per-stage model comparison (R1·U2)\n\nGenerated by `psm-research score-models` from the recordings in this folder. Owner-verified and corrected-key answers are counted apart (`ov` / `ck`). Read stage: a page that does not name the key is correct when the model states nothing and wrong when it names anyone (MC-A2 \"what a listing states\").\n"
    );
    let _ = writeln!(
        md,
        "| Stage | Model | Effort | Calls | Pieces | Correct (ov) | Wrong (ov) | Not determined (ov) | Correct (ck) | Wrong (ck) | Not det. (ck) | Schema failures | Refusals | HTTP errors | USD per piece | Seconds per piece |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|"
    );
    let mut rows = Vec::new();
    for ((stage, model, effort), t) in &tallies {
        let c = |b, o| t.counts.get(&(b, o)).copied().unwrap_or(0);
        let pieces = t.pieces.len().max(1) as f64;
        let _ = writeln!(
            md,
            "| {stage} | {model} | {effort} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.4} | {:.1} |",
            t.calls,
            t.pieces.len(),
            c(Bucket::OwnerVerified, Outcome::Correct),
            c(Bucket::OwnerVerified, Outcome::Wrong),
            c(Bucket::OwnerVerified, Outcome::NotDetermined),
            c(Bucket::Corrected, Outcome::Correct) + c(Bucket::FromText, Outcome::Correct),
            c(Bucket::Corrected, Outcome::Wrong) + c(Bucket::FromText, Outcome::Wrong),
            c(Bucket::Corrected, Outcome::NotDetermined)
                + c(Bucket::FromText, Outcome::NotDetermined),
            t.schema_failures,
            t.refusals,
            t.http_errors,
            t.cost as f64 / 1e6 / pieces,
            t.ms as f64 / 1000.0 / pieces,
        );
        rows.push(json!({"stage": stage, "model": model, "effort": effort, "calls": t.calls, "pieces": t.pieces.len(),
            "ov": {"correct": c(Bucket::OwnerVerified, Outcome::Correct), "wrong": c(Bucket::OwnerVerified, Outcome::Wrong), "not_determined": c(Bucket::OwnerVerified, Outcome::NotDetermined)},
            "ck": {"correct": c(Bucket::Corrected, Outcome::Correct), "wrong": c(Bucket::Corrected, Outcome::Wrong), "not_determined": c(Bucket::Corrected, Outcome::NotDetermined)},
            "schema_failures": t.schema_failures, "refusals": t.refusals, "http_errors": t.http_errors,
            "cost_micro_usd": t.cost, "ms": t.ms}));
    }
    let _ = writeln!(
        md,
        "\n## Read stage by page kind (all buckets)\n\nOn a page that does not name the key, \"wrong\" means the model reported a name the page gives for another piece.\n\n| Model | Effort | Page names the key | Correct | Wrong | Not determined |\n|---|---|---|---|---|---|"
    );
    for ((model, effort, names), counts) in &read_split {
        let g = |o| counts.get(&o).copied().unwrap_or(0);
        let _ = writeln!(
            md,
            "| {model} | {effort} | {} | {} | {} | {} |",
            if *names { "yes" } else { "no" },
            g(Outcome::Correct),
            g(Outcome::Wrong),
            g(Outcome::NotDetermined)
        );
    }
    let _ = writeln!(
        md,
        "\n## Disagreements (every wrong answer, for a human look)\n\n| Stage | Model | Effort | Piece | Unit | Field | Key | Answer | Bucket |\n|---|---|---|---|---|---|---|---|---|"
    );
    wrongs.sort_by(|a, b| {
        (a.stage.as_str(), a.n, a.field.as_str(), a.model.as_str()).cmp(&(
            b.stage.as_str(),
            b.n,
            b.field.as_str(),
            b.model.as_str(),
        ))
    });
    for w in &wrongs {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {:?} |",
            w.stage, w.model, w.effort, w.n, w.unit, w.field, w.key, w.answer, w.bucket
        );
    }
    fs::write(ctx.out.join("report_models.md"), md).map_err(|e| e.to_string())?;
    fs::write(
        ctx.out.join("model_scores.json"),
        serde_json::to_vec_pretty(&json!({"rows": rows, "wrong": wrongs.len()}))
            .unwrap_or_default(),
    )
    .map_err(|e| e.to_string())?;
    eprintln!("wrote {}", ctx.out.join("report_models.md").display());
    Ok(())
}

fn key_value(entries: &[KeyEntry], n: u32, field: &str) -> Json {
    entries
        .iter()
        .find(|e| e.n == n && e.field == field)
        .map(|e| e.value.clone())
        .unwrap_or(Json::Null)
}

/// Pages module re-export for the read stage's text (plain tag stripping).
pub fn page_text(html: &str) -> String {
    pages::strip_tags(html)
}
