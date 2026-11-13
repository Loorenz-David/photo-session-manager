//! `replay <export> --run <id> --stage <stage> [--model m] [--effort e] [--instructions file]`
//! (R9·U1 task 3; 08B MC-A11 **Replay identity**, MA9, MA16).
//!
//! For each recorded call of the chosen stage the request body is rebuilt from the call record:
//! the recorded request (model, effort, language, instructions, input, images by SHA-256) in the
//! model adapter's wire form, with every image read back from the export's `images/`. Only the
//! model, effort or instructions named on the command line are substituted; every other byte is
//! the recorded one. A call that cannot be rebuilt whole is reported `not replayable: <reason>`
//! and is never sent, not even partially. The key comes only from the environment
//! (`OPENAI_API_KEY`); without it nothing is sent at all. Nothing is ever written to the export.

use psm_research::{
    adapters::openai::{
        adapter::{MAX_OUTPUT_TOKENS, RESPONSES_URL},
        schema::schema_for,
    },
    config::{DEFAULT_MODEL_PROVIDER, DEFAULT_PAGE_READER},
    domain::Stage,
    ports::ModelRequest,
};
use serde_json::{Map, Value as Json, json};
use std::{collections::BTreeMap, fmt::Write as _, time::Duration};

use crate::{
    export_reader::RunDir,
    http::{HttpSeam, Wire, base64},
};

/// The environment variable the replay key is read from.
pub const KEY_VARIABLE: &str = "OPENAI_API_KEY";
/// Why a stage 3 page read through a model-side page tool is not replayed (RESEARCH-VERIFY-8).
pub const PAGE_TOOL: &str = "the page content cannot be reproduced";
/// The time one replayed call may take (the app's model call timeout default).
pub const TIMEOUT: Duration = Duration::from_secs(180);

/// What the command line substitutes.
#[derive(Debug, Clone, Default)]
pub struct Substitutions {
    pub model: Option<String>,
    pub effort: Option<String>,
    /// The contents of the `--instructions` file.
    pub instructions: Option<String>,
}

/// One call of the stage, ready to send or refused.
#[derive(Debug, Clone)]
pub enum Plan {
    Send {
        call_id: i64,
        unit_key: String,
        body: Vec<u8>,
        recorded_output: Json,
    },
    NotReplayable {
        call_id: Option<i64>,
        reason: String,
    },
    /// A visual search or plain page download: no model, effort or instructions to substitute.
    Skipped { call_id: i64, why: String },
}

fn stage_of(text: &str) -> Result<Stage, String> {
    serde_json::from_value(json!(text)).map_err(|_| {
        format!("unknown stage {text:?} (observe, find, read, compare, labels, condition, decide)")
    })
}

/// Every `{"withheld": …}` and every `{"image_sha256": …}` under `value`.
fn image_refs(value: &Json, withheld: &mut bool, shas: &mut Vec<String>) {
    match value {
        Json::Object(map) => {
            if map.contains_key("withheld") {
                *withheld = true;
            }
            if let Some(sha) = map.get("image_sha256").and_then(Json::as_str) {
                shas.push(sha.to_owned());
            }
            map.values().for_each(|v| image_refs(v, withheld, shas));
        }
        Json::Array(items) => items.iter().for_each(|v| image_refs(v, withheld, shas)),
        _ => {}
    }
}

fn image_part(url: String) -> Json {
    json!({ "type": "input_image", "image_url": url })
}

/// The SHA-256s of a list of `{"image_sha256": …}` references, in order.
fn shas_of(list: &Json) -> Result<Vec<String>, String> {
    list.as_array()
        .into_iter()
        .flatten()
        .map(|r| {
            r["image_sha256"]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "an image reference has no SHA-256".to_owned())
        })
        .collect()
}

/// The body the model adapter sent for this recorded request, with the substitutions applied:
/// the adapter's form (`psm_research::adapters::openai`), field for field and in its order.
pub fn rebuild(
    run: &RunDir,
    stage: Stage,
    unit_key: &str,
    recorded: &Json,
    with: &Substitutions,
) -> Result<Vec<u8>, String> {
    let text = |key: &str| recorded[key].as_str().unwrap_or_default().to_owned();
    let model = with.model.clone().unwrap_or_else(|| text("model"));
    let effort = with.effort.clone().unwrap_or_else(|| text("effort"));
    let instructions = with
        .instructions
        .clone()
        .unwrap_or_else(|| text("instructions"));
    let language = text("language");
    let mut data_urls = Vec::new();
    for sha in shas_of(&recorded["images"])?
        .into_iter()
        .chain(shas_of(&recorded["listing_images"])?)
    {
        let path = run
            .image(&sha)
            .ok_or_else(|| format!("image {sha} is missing from images/"))?;
        let bytes = std::fs::read(&path).map_err(|e| format!("image {sha}: {e}"))?;
        data_urls.push(format!("data:image/jpeg;base64,{}", base64(&bytes)));
    }
    let request = ModelRequest {
        stage,
        unit_key: unit_key.to_owned(),
        model: model.clone(),
        effort: effort.clone(),
        language: language.clone(),
        instructions: instructions.clone(),
        input: recorded["input"].clone(),
        images: Vec::new(),
        listing_images: Vec::new(),
    };
    let schema = schema_for(&request);
    let mut input = request.input.clone();
    match input.as_object_mut() {
        Some(object) => {
            object.insert("language".into(), json!(language));
        }
        None => input = json!({ "input": input, "language": language }),
    }
    let mut content = vec![json!({ "type": "input_text", "text": input.to_string() })];
    content.extend(data_urls.into_iter().map(image_part));
    let mut body = json!({
        "input": [{ "role": "user", "content": content }],
        "instructions": instructions,
        "max_output_tokens": MAX_OUTPUT_TOKENS,
        "model": model,
        "store": false,
        "text": { "format": {
            "type": "json_schema",
            "name": stage.as_str(),
            "strict": true,
            "schema": schema,
        } },
    });
    if !effort.is_empty() {
        body["reasoning"] = json!({ "effort": effort });
    }
    serde_json::to_vec(&body).map_err(|e| e.to_string())
}

/// The structured output of a Responses reply body (the concatenated `output_text`, parsed).
pub fn output_of(body: &Json) -> Json {
    let mut text = None::<String>;
    for item in body["output"].as_array().into_iter().flatten() {
        for content in item["content"].as_array().into_iter().flatten() {
            if content["type"] == "output_text"
                && let Some(t) = content["text"].as_str()
            {
                text.get_or_insert_with(String::new).push_str(t);
            }
        }
    }
    match text {
        Some(t) => serde_json::from_str(&t).unwrap_or(Json::String(t)),
        None => Json::Null,
    }
}

/// The recorded output of a call: from its recorded response (`{"body": {status, headers,
/// body}}`), or its error.
fn recorded_output(call: &Json) -> Json {
    let response = call["response"]
        .as_str()
        .and_then(|t| serde_json::from_str::<Json>(t).ok());
    match response {
        Some(r) => output_of(&r["body"]["body"]),
        None => json!({ "error": call["error_text"], "state": call["state"] }),
    }
}

/// What will happen to each call of `stage` in `run`.
pub fn plan(run: &RunDir, stage: &str, with: &Substitutions) -> Result<Vec<Plan>, String> {
    let stage = stage_of(stage)?;
    let calls: Vec<&Json> = run
        .calls
        .iter()
        .filter(|c| c["stage"].as_str() == Some(stage.as_str()))
        .collect();
    if run.expired() {
        let reason = psm_research::export::CALL_RECORD_EXPIRED.to_owned();
        if calls.is_empty() {
            return Ok(vec![Plan::NotReplayable {
                call_id: None,
                reason,
            }]);
        }
        return Ok(calls
            .iter()
            .map(|c| Plan::NotReplayable {
                call_id: c["id"].as_i64(),
                reason: reason.clone(),
            })
            .collect());
    }
    let mut out = Vec::new();
    for call in calls {
        let call_id = call["id"].as_i64().unwrap_or(0);
        let refuse = |reason: String| Plan::NotReplayable {
            call_id: Some(call_id),
            reason,
        };
        let Some(recorded) = call["request"]
            .as_str()
            .and_then(|t| serde_json::from_str::<Json>(t).ok())
        else {
            out.push(refuse("the recorded request is missing".into()));
            continue;
        };
        let provider = call["provider"].as_str().unwrap_or_default();
        if recorded.get("url").is_some() {
            if provider == DEFAULT_PAGE_READER {
                out.push(Plan::Skipped {
                    call_id,
                    why: "a page download, not a model call".into(),
                });
            } else {
                out.push(refuse(PAGE_TOOL.into()));
            }
            continue;
        }
        if recorded.get("instructions").is_none() {
            out.push(Plan::Skipped {
                call_id,
                why: "a visual search, not a model call".into(),
            });
            continue;
        }
        let (mut withheld, mut shas) = (false, Vec::new());
        image_refs(&recorded, &mut withheld, &mut shas);
        if withheld {
            out.push(refuse(format!(
                "an image was withheld ({})",
                psm_research::export::PHOTO_DELETED
            )));
            continue;
        }
        if provider != DEFAULT_MODEL_PROVIDER {
            out.push(refuse(format!(
                "provider {provider:?} has no replay form in this tool"
            )));
            continue;
        }
        let unit_key = call["unit_key"].as_str().unwrap_or_default().to_owned();
        match rebuild(run, stage, &unit_key, &recorded, with) {
            Ok(body) => out.push(Plan::Send {
                call_id,
                unit_key,
                body,
                recorded_output: recorded_output(call),
            }),
            Err(reason) => out.push(refuse(reason)),
        }
    }
    Ok(out)
}

/// `path = value` for every leaf of `value`.
fn leaves(value: &Json, path: String, out: &mut BTreeMap<String, String>) {
    match value {
        Json::Object(map) if !map.is_empty() => {
            for (k, v) in map {
                let p = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                leaves(v, p, out);
            }
        }
        other => {
            out.insert(path, other.to_string());
        }
    }
}

/// The difference of two outputs, one line per leaf that differs: `path: recorded → new`.
pub fn difference(recorded: &Json, new: &Json) -> Vec<String> {
    let (mut a, mut b) = (BTreeMap::new(), BTreeMap::new());
    leaves(recorded, String::new(), &mut a);
    leaves(new, String::new(), &mut b);
    let keys: std::collections::BTreeSet<&String> = a.keys().chain(b.keys()).collect();
    keys.into_iter()
        .filter(|k| a.get(*k) != b.get(*k))
        .map(|k| {
            let show = |m: &BTreeMap<String, String>| m.get(k).cloned().unwrap_or("—".into());
            format!("{k}: {} → {}", show(&a), show(&b))
        })
        .collect()
}

/// Runs the replay: plans every call, then (with a key) sends each replayable one. The printed
/// text is returned; the key appears in no part of it.
pub fn run(
    run: &RunDir,
    stage: &str,
    with: &Substitutions,
    key: Option<&str>,
    seam: &dyn HttpSeam,
) -> Result<(String, Json), String> {
    let key = key
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .ok_or_else(|| format!("{KEY_VARIABLE} is not set; nothing was sent"))?;
    let plans = plan(run, stage, with)?;
    let mut text = String::new();
    let mut results = Vec::new();
    let _ = writeln!(
        text,
        "replay of run {} stage {stage}{}{}{}",
        run.id(),
        with.model
            .as_deref()
            .map(|m| format!(" model={m}"))
            .unwrap_or_default(),
        with.effort
            .as_deref()
            .map(|e| format!(" effort={e}"))
            .unwrap_or_default(),
        if with.instructions.is_some() {
            " instructions=<file>"
        } else {
            ""
        },
    );
    for p in plans {
        match p {
            Plan::NotReplayable { call_id, reason } => {
                let who = call_id.map_or(format!("run {}", run.id()), |id| format!("call {id}"));
                let _ = writeln!(text, "{who}: not replayable: {reason}");
                results.push(json!({"call_id": call_id, "not_replayable": reason}));
            }
            Plan::Skipped { call_id, why } => {
                let _ = writeln!(text, "call {call_id}: skipped: {why}");
                results.push(json!({"call_id": call_id, "skipped": why}));
            }
            Plan::Send {
                call_id,
                unit_key,
                body,
                recorded_output,
            } => {
                let wire = Wire {
                    method: "POST".into(),
                    url: RESPONSES_URL.into(),
                    headers: vec![
                        ("authorization".into(), format!("Bearer {key}")),
                        ("content-type".into(), "application/json".into()),
                    ],
                    body,
                    timeout: TIMEOUT,
                    follow_redirects: false,
                };
                let (new_output, status) = match seam.send(&wire) {
                    Ok(reply) => {
                        let body: Json = serde_json::from_slice(&reply.body).unwrap_or(Json::Null);
                        let output = if (200..300).contains(&reply.status) {
                            output_of(&body)
                        } else {
                            json!({ "error": body["error"]["message"] })
                        };
                        (output, Some(reply.status))
                    }
                    Err(e) => (json!({ "error": e.text }), None),
                };
                let diff = difference(&recorded_output, &new_output);
                let _ = writeln!(
                    text,
                    "call {call_id} ({unit_key}): {}",
                    if diff.is_empty() {
                        "same output".to_owned()
                    } else {
                        format!("{} difference(s)", diff.len())
                    }
                );
                let pretty = |v: &Json| serde_json::to_string_pretty(v).unwrap_or_default();
                let _ = writeln!(text, "  recorded: {}", pretty(&recorded_output));
                let _ = writeln!(text, "  new:      {}", pretty(&new_output));
                for line in &diff {
                    let _ = writeln!(text, "  {line}");
                }
                results.push(json!({
                    "call_id": call_id,
                    "unit_key": unit_key,
                    "status": status,
                    "recorded": recorded_output,
                    "new": new_output,
                    "difference": diff,
                }));
            }
        }
    }
    let mut summary = Map::new();
    summary.insert("run".into(), json!(run.id()));
    summary.insert("stage".into(), json!(stage));
    summary.insert("model".into(), json!(with.model));
    summary.insert("effort".into(), json!(with.effort));
    summary.insert("instructions".into(), json!(with.instructions));
    summary.insert("calls".into(), Json::Array(results));
    Ok((text, Json::Object(summary)))
}
