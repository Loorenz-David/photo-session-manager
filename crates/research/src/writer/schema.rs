//! W2·U3: the writer call's strict output schema (`master_plan_writer.md` §5 WC4.1, WC6): exactly
//! the requested texts (the description as paragraphs, every other text as one string), each with
//! the facts it used and a reason; the safeguard findings; the questions. Every property is
//! required and `additionalProperties` is false at every level.

use serde_json::{Map, Value as Json, json};

use crate::writer::config::FINDING_TYPES;

fn object(properties: Vec<(&str, Json)>) -> Json {
    let required: Vec<Json> = properties.iter().map(|(k, _)| json!(k)).collect();
    let properties: Map<String, Json> = properties
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect();
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false,
    })
}

fn strings() -> Json {
    json!({ "type": "array", "items": { "type": "string" } })
}

/// The schema of a writer request for the output names `texts`.
pub fn schema_for_write(texts: &[String]) -> Json {
    let mut by_name: Vec<(&str, Json)> = Vec::new();
    for name in texts {
        let body = if name == "description" {
            ("paragraphs", strings())
        } else {
            ("text", json!({ "type": "string" }))
        };
        by_name.push((
            name.as_str(),
            object(vec![
                body,
                ("facts", strings()),
                ("reason", json!({ "type": "string" })),
            ]),
        ));
    }
    object(vec![
        ("texts", object(by_name)),
        (
            "findings",
            json!({
                "type": "array",
                "items": object(vec![
                    ("type", json!({ "type": "string", "enum": FINDING_TYPES })),
                    ("keys", strings()),
                    ("text", json!({ "type": "string" })),
                ]),
            }),
        ),
        (
            "questions",
            json!({
                "type": "array",
                "items": object(vec![("keys", strings()), ("text", json!({ "type": "string" }))]),
            }),
        ),
    ])
}

/// The output names a request asks for (its input's `texts`).
pub fn requested(input: &Json) -> Vec<String> {
    input["texts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|t| t.as_str().map(str::to_owned))
        .collect()
}
