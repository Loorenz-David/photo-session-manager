//! 05A MC-9 (attribute values: core canonical forms and equality) and 07A MC-14 (condition photo
//! selection). Equality of values is byte equality of the canonical JSON produced here.

use crate::domain::attribute::{CoreType, Definition};
use serde_json::Value as Json;
use unicode_normalization::UnicodeNormalization;

pub type PhotoId = i64;

/// Canonical JSON text: compact, object keys sorted (MC-9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalJson(pub String);

/// A value that does not fit its definition's canonical form; it is stored verbatim with
/// `valid = false` (MC-9 "invalid values").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invalid(pub String);

/// The five strings of the severity set (MC-9 field kind).
pub const SEVERITY_SET: [&str; 5] = ["N/A", "None", "Minor", "Moderate", "Major"];

/// Compact JSON with object keys in sorted order at every level.
pub fn canonical_json(value: &Json) -> String {
    sorted(value).to_string()
}

fn sorted(value: &Json) -> Json {
    match value {
        Json::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            Json::Object(
                keys.into_iter()
                    .map(|k| (k.clone(), sorted(&map[k])))
                    .collect(),
            )
        }
        Json::Array(items) => Json::Array(items.iter().map(sorted).collect()),
        other => other.clone(),
    }
}

fn invalid(core_type: CoreType, why: &str) -> Invalid {
    Invalid(format!("{}: {why}", core_type.as_str()))
}

fn nfc(text: &str) -> String {
    text.nfc().collect()
}

/// `-?\d+(\.\d+)?` → no trailing zeros after the point, no point without digits after it, no
/// redundant leading zeros, `-0` written `0`. Anything else (exponent, `+`, spaces) is `None`.
pub fn canonical_decimal(text: &str) -> Option<String> {
    let (negative, unsigned) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let (int, frac) = match unsigned.split_once('.') {
        Some((int, frac)) => (int, Some(frac)),
        None => (unsigned, None),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(int) || frac.is_some_and(|f| !digits(f)) {
        return None;
    }
    let int = match int.trim_start_matches('0') {
        "" => "0",
        trimmed => trimmed,
    };
    let frac = frac.map(|f| f.trim_end_matches('0')).unwrap_or("");
    let mut out = String::new();
    if negative && !(int == "0" && frac.is_empty()) {
        out.push('-');
    }
    out.push_str(int);
    if !frac.is_empty() {
        out.push('.');
        out.push_str(frac);
    }
    Some(out)
}

fn decimal_field(core_type: CoreType, value: Option<&Json>, name: &str) -> Result<Json, Invalid> {
    value
        .and_then(Json::as_str)
        .and_then(canonical_decimal)
        .map(Json::from)
        .ok_or_else(|| invalid(core_type, &format!("{name} is not a decimal string")))
}

/// The object's keys are exactly `names`.
fn exact_object<'a>(
    core_type: CoreType,
    raw: &'a Json,
    names: &[&str],
) -> Result<&'a serde_json::Map<String, Json>, Invalid> {
    let map = raw
        .as_object()
        .ok_or_else(|| invalid(core_type, "not an object"))?;
    if map.len() != names.len() || names.iter().any(|n| !map.contains_key(*n)) {
        return Err(invalid(
            core_type,
            &format!("keys must be exactly {}", names.join(", ")),
        ));
    }
    Ok(map)
}

fn string_of(core_type: CoreType, raw: &Json) -> Result<&str, Invalid> {
    raw.as_str()
        .ok_or_else(|| invalid(core_type, "not a string"))
}

fn trimmed_text(raw: &str) -> String {
    nfc(raw).trim_end().to_owned()
}

/// Paragraphs (runs of non-blank lines) joined by exactly one blank line; trailing whitespace of
/// every line removed; leading and trailing blank lines dropped.
fn paragraphs(raw: &str) -> String {
    let text = nfc(raw).replace("\r\n", "\n");
    let mut out: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in text.split('\n') {
        let line = line.trim_end();
        if line.trim().is_empty() {
            if !current.is_empty() {
                out.push(current.join("\n"));
                current.clear();
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        out.push(current.join("\n"));
    }
    out.join("\n\n")
}

fn is_date(text: &str) -> bool {
    let b = text.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d").is_ok()
}

fn string_list(core_type: CoreType, raw: &Json) -> Result<Vec<String>, Invalid> {
    let items = raw
        .as_array()
        .ok_or_else(|| invalid(core_type, "not an array"))?;
    let mut out: Vec<String> = Vec::with_capacity(items.len());
    for item in items {
        let text = item
            .as_str()
            .ok_or_else(|| invalid(core_type, "an element is not a string"))?;
        if !out.iter().any(|seen| seen == text) {
            out.push(text.to_owned());
        }
    }
    Ok(out)
}

/// MC-9: the canonical JSON of `raw` for a definition of `core_type`, or `Invalid` when the value
/// does not fit (stored verbatim with `valid = false` by the caller).
pub fn canonicalize(
    core_type: CoreType,
    definition: &Definition,
    raw: Json,
) -> Result<CanonicalJson, Invalid> {
    use CoreType as T;
    let canonical: Json = match core_type {
        T::Text | T::Textarea | T::Url => Json::from(trimmed_text(string_of(core_type, &raw)?)),
        T::Color => {
            let text = nfc(string_of(core_type, &raw)?);
            let hex = text
                .strip_prefix('#')
                .filter(|h| h.len() == 6 && h.bytes().all(|b| b.is_ascii_hexdigit()))
                .ok_or_else(|| invalid(core_type, "not #rrggbb"))?;
            Json::from(format!("#{}", hex.to_ascii_lowercase()))
        }
        T::RichText => Json::from(paragraphs(string_of(core_type, &raw)?)),
        T::Integer => match &raw {
            Json::Number(n) if n.is_i64() => raw.clone(),
            _ => return Err(invalid(core_type, "not a JSON integer in the i64 range")),
        },
        T::Decimal => decimal_field(core_type, Some(&raw), "value")?,
        T::Money => {
            let map = exact_object(core_type, &raw, &["amount", "currency"])?;
            let currency = map["currency"]
                .as_str()
                .filter(|c| c.len() == 3 && c.bytes().all(|b| b.is_ascii_alphabetic()))
                .ok_or_else(|| invalid(core_type, "currency is not an ISO-4217 code"))?;
            serde_json::json!({
                "amount": decimal_field(core_type, map.get("amount"), "amount")?,
                "currency": currency.to_ascii_uppercase(),
            })
        }
        T::Dimension | T::Weight => {
            let map = exact_object(core_type, &raw, &["value", "unit"])?;
            let unit = map["unit"]
                .as_str()
                .filter(|u| !u.is_empty())
                .ok_or_else(|| invalid(core_type, "unit is not a string"))?;
            if let Some(units) = definition.units()
                && !units.contains(&unit)
            {
                return Err(invalid(
                    core_type,
                    "unit is not one of the definition's units",
                ));
            }
            serde_json::json!({
                "unit": unit,
                "value": decimal_field(core_type, map.get("value"), "value")?,
            })
        }
        T::Boolean => match raw {
            Json::Bool(_) => raw.clone(),
            _ => return Err(invalid(core_type, "not true or false")),
        },
        T::Date => {
            let text = string_of(core_type, &raw)?;
            if !is_date(text) {
                return Err(invalid(core_type, "not YYYY-MM-DD"));
            }
            Json::from(text)
        }
        T::Rating => {
            let map = exact_object(core_type, &raw, &["value", "min", "max"])?;
            serde_json::json!({
                "max": decimal_field(core_type, map.get("max"), "max")?,
                "min": decimal_field(core_type, map.get("min"), "min")?,
                "value": decimal_field(core_type, map.get("value"), "value")?,
            })
        }
        T::Choice => {
            let text = string_of(core_type, &raw)?;
            if !definition.choices.iter().any(|c| c == text) {
                return Err(invalid(core_type, "not one of the choices"));
            }
            Json::from(text)
        }
        T::MultiChoice | T::Tags => {
            let list = string_list(core_type, &raw)?;
            if core_type == T::MultiChoice
                && list
                    .iter()
                    .any(|item| !definition.choices.iter().any(|c| c == item))
            {
                return Err(invalid(core_type, "an element is not one of the choices"));
            }
            Json::from(list)
        }
        T::MediaList => {
            let map = exact_object(core_type, &raw, &["excluded"])?;
            let ids = map["excluded"]
                .as_array()
                .filter(|ids| ids.iter().all(|id| id.as_i64().is_some()))
                .ok_or_else(|| invalid(core_type, "excluded is not a list of photo ids"))?;
            serde_json::json!({ "excluded": ids })
        }
        T::Reference => raw,
    };
    Ok(CanonicalJson(canonical_json(&canonical)))
}

/// MC-9 severity set: exactly the five strings `N/A`, `None`, `Minor`, `Moderate`, `Major`
/// (case-sensitive, any order, no others, none twice).
pub fn is_severity_set(choices: &[String]) -> bool {
    choices.len() == SEVERITY_SET.len()
        && SEVERITY_SET
            .iter()
            .all(|s| choices.iter().filter(|c| c.as_str() == *s).count() == 1)
}

/// 07A MC-14: the session's condition photos in condition order minus the excluded ids. Excluded
/// ids that are not condition photos are ignored.
pub fn selected_condition_photos(
    condition_in_order: &[PhotoId],
    excluded: &[PhotoId],
) -> Vec<PhotoId> {
    condition_in_order
        .iter()
        .copied()
        .filter(|id| !excluded.contains(id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::attribute::AttributeGroup;
    use serde_json::json;

    fn def(core_type: CoreType, choices: &[&str], validations: Json) -> Definition {
        Definition {
            integration: "test".into(),
            key: "k".into(),
            label: "K".into(),
            core_type,
            choices: choices.iter().map(|c| c.to_string()).collect(),
            validations,
            group: AttributeGroup::Details,
            description: None,
            writable: true,
        }
    }

    fn canon(core_type: CoreType, raw: Json) -> Result<String, Invalid> {
        canon_with(def(core_type, &[], json!({})), raw)
    }

    fn canon_with(definition: Definition, raw: Json) -> Result<String, Invalid> {
        canonicalize(definition.core_type, &definition, raw).map(|c| c.0)
    }

    #[test]
    fn mc9_text_nfc_trailing_whitespace_only() {
        assert_eq!(
            canon(CoreType::Text, json!(" He\u{301}j \t ")).unwrap(),
            "\" H\u{e9}j\""
        );
        assert_eq!(
            canon(CoreType::Textarea, json!("a \nb \n")).unwrap(),
            "\"a \\nb\""
        );
        assert!(canon(CoreType::Text, json!(3)).is_err());
    }

    #[test]
    fn mc9_rich_text_one_blank_line_between_paragraphs() {
        assert_eq!(
            canon(CoreType::RichText, json!("\n a\nb  \n \n\n\nc\n\n")).unwrap(),
            "\" a\\nb\\n\\nc\""
        );
    }

    #[test]
    fn mc9_color_lowercase_hex_only() {
        assert_eq!(
            canon(CoreType::Color, json!("#A1b2C3")).unwrap(),
            "\"#a1b2c3\""
        );
        assert!(canon(CoreType::Color, json!("#abc")).is_err());
        assert!(canon(CoreType::Color, json!("#gggggg")).is_err());
    }

    #[test]
    fn mc9_integer_is_a_json_integer() {
        assert_eq!(canon(CoreType::Integer, json!(-7)).unwrap(), "-7");
        assert!(canon(CoreType::Integer, json!(1.0)).is_err());
        assert!(canon(CoreType::Integer, json!(u64::MAX)).is_err());
    }

    #[test]
    fn mc9_decimal_forms() {
        for (input, out) in [
            ("1.50", Some("1.5")),
            ("2.0", Some("2")),
            ("-0.10", Some("-0.1")),
            ("007.25", Some("7.25")),
            ("-0.0", Some("0")),
            ("0", Some("0")),
            ("1e3", None),
            ("+1", None),
            ("1.", None),
            (".5", None),
            ("", None),
            ("1 ", None),
        ] {
            assert_eq!(canonical_decimal(input).as_deref(), out, "{input}");
        }
        assert!(canon(CoreType::Decimal, json!(1.5)).is_err());
    }

    #[test]
    fn mc9_money_dimension_rating_objects() {
        assert_eq!(
            canon(CoreType::Money, json!({"currency":"eur","amount":"3.00"})).unwrap(),
            r#"{"amount":"3","currency":"EUR"}"#
        );
        assert!(canon(CoreType::Money, json!({"amount":"3","currency":"EURO"})).is_err());
        assert!(canon(CoreType::Money, json!({"amount":"3"})).is_err());
        let units = def(CoreType::Weight, &[], json!({"units":["kg","g"]}));
        assert_eq!(
            canon_with(units.clone(), json!({"value":"2.50","unit":"kg"})).unwrap(),
            r#"{"unit":"kg","value":"2.5"}"#
        );
        assert!(canon_with(units, json!({"value":"2","unit":"lb"})).is_err());
        assert_eq!(
            canon(
                CoreType::Rating,
                json!({"value":"3.50","min":"0","max":"10.0"})
            )
            .unwrap(),
            r#"{"max":"10","min":"0","value":"3.5"}"#
        );
    }

    #[test]
    fn mc9_boolean_date() {
        assert_eq!(canon(CoreType::Boolean, json!(false)).unwrap(), "false");
        assert!(canon(CoreType::Boolean, json!(0)).is_err());
        assert_eq!(
            canon(CoreType::Date, json!("2024-02-29")).unwrap(),
            "\"2024-02-29\""
        );
        assert!(canon(CoreType::Date, json!("2023-02-29")).is_err());
        assert!(canon(CoreType::Date, json!("2026-1-05")).is_err());
    }

    #[test]
    fn mc9_choices_are_exact_strings() {
        let quantity = def(CoreType::Choice, &["1", "2", "Minor"], json!({}));
        assert_eq!(canon_with(quantity.clone(), json!("2")).unwrap(), "\"2\"");
        assert!(canon_with(quantity.clone(), json!(2)).is_err());
        assert!(canon_with(quantity, json!("minor")).is_err());
        let wood = def(CoreType::MultiChoice, &["Oak", "Teak"], json!({}));
        assert_eq!(
            canon_with(wood.clone(), json!(["Teak", "Oak", "Teak"])).unwrap(),
            r#"["Teak","Oak"]"#
        );
        assert!(canon_with(wood, json!(["Pine"])).is_err());
        assert_eq!(
            canon(CoreType::Tags, json!(["b", "a", "b"])).unwrap(),
            r#"["b","a"]"#
        );
    }

    #[test]
    fn mc9_media_list_and_reference() {
        assert_eq!(
            canon(CoreType::MediaList, json!({"excluded":[5,2]})).unwrap(),
            r#"{"excluded":[5,2]}"#
        );
        assert!(canon(CoreType::MediaList, json!({"excluded":["5"]})).is_err());
        assert_eq!(
            canon(CoreType::Reference, json!({"b":[1,{"z":0,"a":1}],"a":null})).unwrap(),
            r#"{"a":null,"b":[1,{"a":1,"z":0}]}"#
        );
    }

    #[test]
    fn mc9_severity_set() {
        let set =
            |c: &[&str]| is_severity_set(&c.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        assert!(set(&["Major", "None", "N/A", "Moderate", "Minor"]));
        assert!(!set(&["N/A", "None", "Minor", "Moderate"]));
        assert!(!set(&["N/A", "None", "Minor", "Moderate", "major"]));
        assert!(!set(&[
            "N/A", "None", "Minor", "Moderate", "Major", "Other"
        ]));
        assert!(!set(&["N/A", "None", "Minor", "Moderate", "Moderate"]));
    }

    #[test]
    fn mc14_selection_is_condition_order_minus_exclusions() {
        assert_eq!(selected_condition_photos(&[4, 2, 9], &[]), vec![4, 2, 9]);
        assert_eq!(selected_condition_photos(&[4, 2, 9], &[2, 77]), vec![4, 9]);
        assert_eq!(selected_condition_photos(&[], &[1]), Vec::<PhotoId>::new());
    }
}
