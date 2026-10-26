//! 07A MC-13: the Shopify metafield type map, both directions, with the value shapes grounded by
//! SV-1 (`docs/intention/evidence/shopify_admin.md`).
//!
//! Platform → core (`from_platform_string`) produces the raw JSON that core's MC-9
//! `canonicalize` accepts: `currency_code` → `currency`, `scale_min`/`scale_max` → `min`/`max`,
//! dimension numbers → decimal strings, the rich-text tree → paragraphs, list JSON strings →
//! arrays. Core → platform (`to_platform_string`) is the inverse on canonical values.

use psm_core::domain::attribute::CoreType;
use serde_json::{Value as Json, json};

/// The unit codes Shopify stores in dimension values (SV-1).
pub const DIMENSION_UNITS: [&str; 6] = [
    "MILLIMETERS",
    "CENTIMETERS",
    "METERS",
    "INCHES",
    "FEET",
    "YARDS",
];
/// The unit codes Shopify stores in weight values (SV-1).
pub const WEIGHT_UNITS: [&str; 4] = ["OUNCES", "POUNDS", "GRAMS", "KILOGRAMS"];
/// The unit codes Shopify stores in volume values (SV-1).
pub const VOLUME_UNITS: [&str; 12] = [
    "MILLILITERS",
    "CENTILITERS",
    "LITERS",
    "CUBIC_METERS",
    "FLUID_OUNCES",
    "PINTS",
    "QUARTS",
    "GALLONS",
    "IMPERIAL_FLUID_OUNCES",
    "IMPERIAL_PINTS",
    "IMPERIAL_QUARTS",
    "IMPERIAL_GALLONS",
];

/// A definition's `validations` (`[{name, value}]` as Shopify returns them) looked up by name.
pub fn validation<'a>(validations: &'a Json, name: &str) -> Option<&'a str> {
    match validations {
        Json::Array(items) => items
            .iter()
            .find(|v| v.get("name").and_then(Json::as_str) == Some(name))
            .and_then(|v| v.get("value").and_then(Json::as_str)),
        Json::Object(map) => map.get(name).and_then(Json::as_str),
        _ => None,
    }
}

/// The `choices` validation as a list of strings (empty when absent).
pub fn choices(validations: &Json) -> Vec<String> {
    validation(validations, "choices")
        .and_then(|text| serde_json::from_str::<Vec<String>>(text).ok())
        .unwrap_or_default()
}

/// Whether the definition's `access` lets this app write (SV-1: `access.admin` of
/// `PUBLIC_READ_WRITE` means "the merchant and other apps have read and write access"; every
/// other value excludes other apps).
pub fn access_permits_write(access: &Json) -> bool {
    access.get("admin").and_then(Json::as_str) == Some("PUBLIC_READ_WRITE")
}

/// MC-13 table: the core type of a Shopify definition and whether it is writable (type-writable
/// ∧ access permits). `list.file_reference` is `reference` here; the adapter maps the one
/// configured condition-photos definition to `media_list` (09A MC-24).
pub fn to_core(platform_type: &str, validations: &Json, access: &Json) -> (CoreType, bool) {
    let has_choices = !choices(validations).is_empty();
    let core = match platform_type {
        "single_line_text_field" if has_choices => CoreType::Choice,
        "single_line_text_field" => CoreType::Text,
        "list.single_line_text_field" if has_choices => CoreType::MultiChoice,
        "list.single_line_text_field" => CoreType::Tags,
        "multi_line_text_field" => CoreType::Textarea,
        "rich_text_field" => CoreType::RichText,
        "number_integer" => CoreType::Integer,
        "number_decimal" => CoreType::Decimal,
        "dimension" | "volume" => CoreType::Dimension,
        "weight" => CoreType::Weight,
        "boolean" => CoreType::Boolean,
        "date" => CoreType::Date,
        "color" => CoreType::Color,
        "rating" => CoreType::Rating,
        "money" => CoreType::Money,
        "url" => CoreType::Url,
        _ => CoreType::Reference,
    };
    let writable = core != CoreType::Reference && access_permits_write(access);
    (core, writable)
}

/// The core type used to convert values of a platform type when choices do not matter.
pub fn value_core_type(platform_type: &str) -> CoreType {
    match platform_type {
        "list.file_reference" => CoreType::MediaList,
        other => to_core(other, &Json::Null, &Json::Null).0,
    }
}

/// Unit codes for the core `validations.units` of a dimension, weight or volume definition.
pub fn units_for(platform_type: &str) -> Option<&'static [&'static str]> {
    match platform_type {
        "dimension" => Some(&DIMENSION_UNITS),
        "weight" => Some(&WEIGHT_UNITS),
        "volume" => Some(&VOLUME_UNITS),
        _ => None,
    }
}

fn number_text(value: &Json) -> Option<String> {
    match value {
        Json::Number(n) => Some(n.to_string()),
        Json::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn plain_paragraphs(tree: &Json) -> Option<String> {
    let root = tree.as_object()?;
    if root.get("type")?.as_str()? != "root" || root.len() != 2 {
        return None;
    }
    let mut paragraphs = Vec::new();
    for paragraph in root.get("children")?.as_array()? {
        let p = paragraph.as_object()?;
        if p.get("type")?.as_str()? != "paragraph" || p.len() != 2 {
            return None;
        }
        let mut text = String::new();
        for node in p.get("children")?.as_array()? {
            let n = node.as_object()?;
            if n.get("type")?.as_str()? != "text" || n.len() != 2 {
                return None;
            }
            text.push_str(n.get("value")?.as_str()?);
        }
        paragraphs.push(text);
    }
    Some(paragraphs.join("\n\n"))
}

/// Platform value string → the raw JSON core canonicalizes (MC-9). A value that does not fit is
/// returned in a shape `canonicalize` rejects (the platform's own JSON, verbatim), so the caller
/// stores it `valid = false`; for rich text this is the tree when it has nodes or marks other than
/// `root` → `paragraph` → `text`.
pub fn from_platform_string(core_type: CoreType, platform_type: &str, s: &str) -> Json {
    let _ = platform_type;
    let parsed = || serde_json::from_str::<Json>(s).ok();
    match core_type {
        CoreType::Text
        | CoreType::Textarea
        | CoreType::Url
        | CoreType::Color
        | CoreType::Choice
        | CoreType::Date
        | CoreType::Decimal
        | CoreType::Reference => Json::from(s),
        CoreType::RichText => match parsed() {
            Some(tree) => plain_paragraphs(&tree).map(Json::from).unwrap_or(tree),
            None => json!({ "verbatim": s }),
        },
        CoreType::Integer => s
            .parse::<i64>()
            .map(Json::from)
            .unwrap_or_else(|_| json!({ "verbatim": s })),
        CoreType::Boolean => match s {
            "true" => Json::Bool(true),
            "false" => Json::Bool(false),
            _ => json!({ "verbatim": s }),
        },
        CoreType::Dimension | CoreType::Weight => match parsed() {
            Some(Json::Object(map)) => {
                match (map.get("value").and_then(number_text), map.get("unit")) {
                    (Some(value), Some(unit)) if map.len() == 2 => {
                        json!({"value": value, "unit": unit})
                    }
                    _ => Json::Object(map),
                }
            }
            other => other.unwrap_or_else(|| json!({ "verbatim": s })),
        },
        CoreType::Rating => match parsed() {
            Some(Json::Object(map)) => match (
                map.get("value").and_then(number_text),
                map.get("scale_min").and_then(number_text),
                map.get("scale_max").and_then(number_text),
            ) {
                (Some(value), Some(min), Some(max)) if map.len() == 3 => {
                    json!({"value": value, "min": min, "max": max})
                }
                _ => Json::Object(map),
            },
            other => other.unwrap_or_else(|| json!({ "verbatim": s })),
        },
        CoreType::Money => match parsed() {
            Some(Json::Object(map)) => match (
                map.get("amount").and_then(number_text),
                map.get("currency_code"),
            ) {
                (Some(amount), Some(currency)) if map.len() == 2 => {
                    json!({"amount": amount, "currency": currency})
                }
                _ => Json::Object(map),
            },
            other => other.unwrap_or_else(|| json!({ "verbatim": s })),
        },
        CoreType::Tags | CoreType::MultiChoice | CoreType::MediaList => match parsed() {
            Some(list @ Json::Array(_)) => list,
            _ => json!({ "verbatim": s }),
        },
    }
}

fn str_of(canonical: &Json) -> String {
    match canonical {
        Json::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Canonical core value → the string Shopify stores (MC-13 "Value sent").
pub fn to_platform_string(core_type: CoreType, canonical: &Json) -> String {
    match core_type {
        CoreType::Text
        | CoreType::Textarea
        | CoreType::Url
        | CoreType::Color
        | CoreType::Choice
        | CoreType::Date
        | CoreType::Decimal
        | CoreType::Reference => str_of(canonical),
        CoreType::Integer | CoreType::Boolean => canonical.to_string(),
        CoreType::RichText => {
            let text = str_of(canonical);
            let paragraphs: Vec<Json> = if text.is_empty() {
                Vec::new()
            } else {
                text.split("\n\n")
                    .map(|p| json!({"type": "paragraph", "children": [{"type": "text", "value": p}]}))
                    .collect()
            };
            json!({"type": "root", "children": paragraphs}).to_string()
        }
        CoreType::Dimension | CoreType::Weight => {
            let value = canonical.get("value").map(str_of).unwrap_or_default();
            let unit = canonical.get("unit").cloned().unwrap_or(Json::Null);
            // The canonical decimal string is valid JSON number text (`-?\d+(\.\d+)?`).
            format!("{{\"value\":{value},\"unit\":{unit}}}")
        }
        CoreType::Rating => json!({
            "value": canonical.get("value").map(str_of).unwrap_or_default(),
            "scale_min": canonical.get("min").map(str_of).unwrap_or_default(),
            "scale_max": canonical.get("max").map(str_of).unwrap_or_default(),
        })
        .to_string(),
        CoreType::Money => json!({
            "amount": canonical.get("amount").map(str_of).unwrap_or_default(),
            "currency_code": canonical.get("currency").map(str_of).unwrap_or_default(),
        })
        .to_string(),
        CoreType::Tags | CoreType::MultiChoice | CoreType::MediaList => canonical.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RW: &str = r#"{"admin":"PUBLIC_READ_WRITE"}"#;

    fn access(text: &str) -> Json {
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn mc13_to_core_table() {
        let choices = json!([{"name": "choices", "value": "[\"a\",\"b\"]"}]);
        let none = json!([]);
        let rw = access(RW);
        for (t, v, expected, writable) in [
            ("single_line_text_field", &choices, CoreType::Choice, true),
            ("single_line_text_field", &none, CoreType::Text, true),
            (
                "list.single_line_text_field",
                &choices,
                CoreType::MultiChoice,
                true,
            ),
            ("list.single_line_text_field", &none, CoreType::Tags, true),
            ("multi_line_text_field", &none, CoreType::Textarea, true),
            ("rich_text_field", &none, CoreType::RichText, true),
            ("number_integer", &none, CoreType::Integer, true),
            ("number_decimal", &none, CoreType::Decimal, true),
            ("dimension", &none, CoreType::Dimension, true),
            ("volume", &none, CoreType::Dimension, true),
            ("weight", &none, CoreType::Weight, true),
            ("boolean", &none, CoreType::Boolean, true),
            ("date", &none, CoreType::Date, true),
            ("color", &none, CoreType::Color, true),
            ("rating", &none, CoreType::Rating, true),
            ("money", &none, CoreType::Money, true),
            ("url", &none, CoreType::Url, true),
            ("date_time", &none, CoreType::Reference, false),
            ("list.file_reference", &none, CoreType::Reference, false),
            ("metaobject_reference", &none, CoreType::Reference, false),
            ("json", &none, CoreType::Reference, false),
        ] {
            assert_eq!(to_core(t, v, &rw), (expected, writable), "{t}");
        }
        assert_eq!(
            to_core(
                "number_integer",
                &none,
                &access(r#"{"admin":"MERCHANT_READ_WRITE"}"#)
            ),
            (CoreType::Integer, false)
        );
        assert_eq!(
            to_core("url", &none, &access(r#"{"admin":"PUBLIC_READ"}"#)),
            (CoreType::Url, false)
        );
    }

    #[test]
    fn mc13_from_platform_shapes() {
        assert_eq!(
            from_platform_string(
                CoreType::Money,
                "money",
                r#"{"amount":"12.50","currency_code":"USD"}"#
            ),
            json!({"amount": "12.50", "currency": "USD"})
        );
        assert_eq!(
            from_platform_string(
                CoreType::Rating,
                "rating",
                r#"{"scale_min":"1.0","scale_max":"5.0","value":"4.5"}"#
            ),
            json!({"value": "4.5", "min": "1.0", "max": "5.0"})
        );
        assert_eq!(
            from_platform_string(
                CoreType::Dimension,
                "volume",
                r#"{"value":1.5,"unit":"LITERS"}"#
            ),
            json!({"value": "1.5", "unit": "LITERS"})
        );
        assert_eq!(
            from_platform_string(
                CoreType::Tags,
                "list.single_line_text_field",
                r#"["teak","oak"]"#
            ),
            json!(["teak", "oak"])
        );
        assert_eq!(
            from_platform_string(CoreType::Integer, "number_integer", "42"),
            json!(42)
        );
        assert_eq!(
            from_platform_string(CoreType::Boolean, "boolean", "false"),
            json!(false)
        );
        let tree = r#"{"type":"root","children":[{"type":"paragraph","children":[{"type":"text","value":"a"},{"type":"text","value":"b"}]},{"type":"paragraph","children":[{"type":"text","value":"c"}]}]}"#;
        assert_eq!(
            from_platform_string(CoreType::RichText, "rich_text_field", tree),
            json!("ab\n\nc")
        );
        let bold = r#"{"type":"root","children":[{"type":"paragraph","children":[{"type":"text","value":"a","bold":true}]}]}"#;
        assert_eq!(
            from_platform_string(CoreType::RichText, "rich_text_field", bold),
            serde_json::from_str::<Json>(bold).unwrap()
        );
    }

    #[test]
    fn mc13_to_platform_shapes() {
        assert_eq!(
            to_platform_string(
                CoreType::Dimension,
                &json!({"unit": "CENTIMETERS", "value": "45.5"})
            ),
            r#"{"value":45.5,"unit":"CENTIMETERS"}"#
        );
        assert_eq!(
            to_platform_string(
                CoreType::Money,
                &json!({"amount": "12.5", "currency": "USD"})
            ),
            r#"{"amount":"12.5","currency_code":"USD"}"#
        );
        assert_eq!(
            to_platform_string(
                CoreType::Rating,
                &json!({"max": "5", "min": "1", "value": "4.5"})
            ),
            r#"{"scale_max":"5","scale_min":"1","value":"4.5"}"#
        );
        assert_eq!(to_platform_string(CoreType::Integer, &json!(42)), "42");
        assert_eq!(to_platform_string(CoreType::Boolean, &json!(true)), "true");
        assert_eq!(
            to_platform_string(CoreType::MediaList, &json!(["f1", "f2"])),
            r#"["f1","f2"]"#
        );
        assert_eq!(
            to_platform_string(CoreType::RichText, &json!("a\n\nb")),
            r#"{"children":[{"children":[{"type":"text","value":"a"}],"type":"paragraph"},{"children":[{"type":"text","value":"b"}],"type":"paragraph"}],"type":"root"}"#
        );
    }
}
