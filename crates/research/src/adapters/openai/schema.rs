//! The strict output schema of each stage (R8·U1 task 3): exactly the keys that stage's parser
//! reads in `stages.rs` / `decide.rs`, keyed by the definition keys of `request.input["fields"]`,
//! every property required, `additionalProperties: false` at every level. A value follows 08B
//! MC-A2's core-type table; no field for a confidence exists (MC-A4).
//!
//! [`check`] is the hand-written check of a reply against the schema this module generated.

use serde_json::{Map, Value as Json, json};

use crate::{domain::Stage, ports::ModelRequest};

fn object(properties: Vec<(String, Json)>) -> Json {
    let required: Vec<Json> = properties.iter().map(|(k, _)| json!(k)).collect();
    let properties: Map<String, Json> = properties.into_iter().collect();
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false,
    })
}

fn nullable(schema: Json) -> Json {
    json!({ "anyOf": [schema, { "type": "null" }] })
}

fn string() -> Json {
    json!({ "type": "string" })
}

fn boolean() -> Json {
    json!({ "type": "boolean" })
}

fn measurement(units: &[&str]) -> Json {
    object(vec![
        ("value".to_owned(), json!({ "type": "number" })),
        (
            "unit".to_owned(),
            json!({ "type": "string", "enum": units }),
        ),
    ])
}

/// The value schema of one field descriptor (`key`, `core_type`, `choices`), MC-A2.
fn value_schema(field: &Json) -> Json {
    let choices: Vec<Json> = field["choices"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|c| c.is_string())
        .cloned()
        .collect();
    let choice = || {
        if choices.is_empty() {
            string()
        } else {
            json!({ "type": "string", "enum": choices })
        }
    };
    match field["core_type"].as_str() {
        Some("choice") => choice(),
        Some("multi_choice") => json!({ "type": "array", "items": choice() }),
        Some("integer") => json!({ "type": "integer" }),
        Some("decimal") => json!({ "type": "number" }),
        Some("dimension") => measurement(&["mm", "cm", "m", "in"]),
        Some("weight") => measurement(&["g", "kg", "lb"]),
        Some("boolean") => boolean(),
        _ => string(),
    }
}

/// The field descriptors of a request, in order, by key.
fn fields(request: &ModelRequest) -> Vec<(String, Json)> {
    request.input["fields"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|f| Some((f["key"].as_str()?.to_owned(), f.clone())))
        .collect()
}

fn per_field(request: &ModelRequest, entry: impl Fn(&Json) -> Json) -> Json {
    object(
        fields(request)
            .into_iter()
            .map(|(key, field)| (key, entry(&field)))
            .collect(),
    )
}

/// `{value, sure}`: a photo read; `value` null = the photo does not show it.
fn photo_read(field: &Json) -> Json {
    object(vec![
        ("value".to_owned(), nullable(value_schema(field))),
        ("sure".to_owned(), boolean()),
    ])
}

/// `{stated, value}`: what a page or a label states.
fn statement(field: &Json) -> Json {
    object(vec![
        ("stated".to_owned(), nullable(string())),
        ("value".to_owned(), nullable(value_schema(field))),
    ])
}

/// The schema of the request's stage. An Observe request whose input carries `photos` (R12·U1:
/// several photos in one call) answers the indexes of the marking photos; one without it (a
/// replay of a run exported before R12) keeps the one-photo schema.
pub fn schema_for(request: &ModelRequest) -> Json {
    let reads = || ("reads".to_owned(), per_field(request, photo_read));
    match request.stage {
        Stage::Observe if request.input.get("photos").is_some() => object(vec![
            reads(),
            (
                "marking_photos".to_owned(),
                json!({ "type": "array", "items": { "type": "integer" } }),
            ),
        ]),
        Stage::Observe => object(vec![reads(), ("shows_marking".to_owned(), boolean())]),
        Stage::Condition => object(vec![reads()]),
        // A18: "Ask AI…" answering in words also answers the owner's question in `answer`.
        Stage::Read if request.input.get("answer").is_some() => object(vec![
            ("answer".to_owned(), string()),
            ("keys".to_owned(), per_field(request, statement)),
        ]),
        // R12·U3: a Read request whose input carries the `piece` also answers whether the page
        // lists the same kind of piece (null: cannot tell) and the kind it lists.
        Stage::Read if request.input.get("piece").is_some() => object(vec![
            ("keys".to_owned(), per_field(request, statement)),
            ("same_kind".to_owned(), nullable(boolean())),
            ("page_kind".to_owned(), nullable(string())),
        ]),
        Stage::Read => object(vec![("keys".to_owned(), per_field(request, statement))]),
        Stage::Compare => object(vec![(
            "verdict".to_owned(),
            json!({ "type": "string", "enum": ["match", "similar", "different"] }),
        )]),
        Stage::Labels => object(vec![
            ("labels".to_owned(), per_field(request, statement)),
            ("text".to_owned(), string()),
        ]),
        Stage::Decide => object(vec![(
            "reasons".to_owned(),
            per_field(request, |_| string()),
        )]),
        Stage::Find => object(Vec::new()),
    }
}

fn kind_matches(kind: &str, value: &Json) -> bool {
    match kind {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.is_i64() || value.is_u64(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => false,
    }
}

/// Checks `value` against a schema made by [`schema_for`]: required keys present, no extra keys,
/// value kinds and enums. The text names the first path that fails.
pub fn check(schema: &Json, value: &Json) -> Result<(), String> {
    check_at(schema, value, "$")
}

fn check_at(schema: &Json, value: &Json, path: &str) -> Result<(), String> {
    if let Some(any) = schema["anyOf"].as_array() {
        return if any.iter().any(|s| check_at(s, value, path).is_ok()) {
            Ok(())
        } else {
            Err(format!("{path} has an unexpected value"))
        };
    }
    if let Some(kind) = schema["type"].as_str()
        && !kind_matches(kind, value)
    {
        return Err(format!("{path} is not of type {kind}"));
    }
    if let Some(allowed) = schema["enum"].as_array()
        && !allowed.contains(value)
    {
        return Err(format!("{path} is not one of the allowed values"));
    }
    if let (Some(properties), Some(given)) = (schema["properties"].as_object(), value.as_object()) {
        for key in schema["required"].as_array().into_iter().flatten() {
            if let Some(key) = key.as_str()
                && !given.contains_key(key)
            {
                return Err(format!("{path}.{key} is missing"));
            }
        }
        for (key, item) in given {
            match properties.get(key) {
                Some(property) => check_at(property, item, &format!("{path}.{key}"))?,
                None if schema["additionalProperties"] == json!(false) => {
                    return Err(format!("{path}.{key} is not expected"));
                }
                None => {}
            }
        }
    }
    if let (Some(items), Some(given)) = (schema.get("items"), value.as_array()) {
        for (i, item) in given.iter().enumerate() {
            check_at(items, item, &format!("{path}[{i}]"))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_request(input: Json) -> ModelRequest {
        ModelRequest {
            stage: Stage::Read,
            unit_key: "browse".into(),
            model: "m".into(),
            effort: "low".into(),
            language: "English".into(),
            instructions: String::new(),
            input,
            images: Vec::new(),
            listing_images: Vec::new(),
        }
    }

    #[test]
    fn a18_an_answer_in_words_is_required_only_when_asked_for() {
        let fields = json!([{"key": "maker", "core_type": "text", "choices": []}]);
        let answering = schema_for(&read_request(json!({"fields": fields, "answer": true})));
        let ok = json!({"answer": "Kai is the designer.", "keys": {"maker": {"stated": null, "value": null}}});
        assert_eq!(check(&answering, &ok), Ok(()));
        assert!(check(&answering, &json!({"keys": ok["keys"].clone()})).is_err());
        let plain = schema_for(&read_request(json!({"fields": fields})));
        assert_eq!(check(&plain, &json!({"keys": ok["keys"].clone()})), Ok(()));
        assert!(check(&plain, &ok).is_err(), "no answer outside answer mode");
    }
}
