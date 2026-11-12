//! MC-A2: from a model's output to a stored value, and what a listing states for a key.
//!
//! Every mapped value is produced by core's `canonicalize`, so it is valid for its definition by
//! construction. Unit conversion uses exact integer arithmetic only.

use psm_core::domain::attribute::{CoreType, Definition};
use psm_core::mechanisms::{
    name_key::name_key,
    sameness::same,
    values::{canonical_decimal, canonicalize},
};
use serde_json::Value as Json;

/// The result of mapping one model output for one definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mapped {
    /// The canonical stored value; `dropped` names the elements of a multi_choice answer that
    /// matched no choice (for the trace).
    Value { value: Json, dropped: Vec<String> },
    /// Not determined for this key; one short sentence for the trace.
    Fail(String),
}

impl Mapped {
    pub fn value(&self) -> Option<&Json> {
        match self {
            Mapped::Value { value, .. } => Some(value),
            Mapped::Fail(_) => None,
        }
    }
}

/// What one listing states for a key (MC-A2 "What a listing states").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stated {
    /// The stated text maps to this stored value.
    Mapped(Json),
    /// The stated text maps to nothing; it counts as its own value, keyed by its name key, and is
    /// never written (R-IA24).
    Unmapped(String),
}

fn fail(text: &str) -> Mapped {
    Mapped::Fail(text.to_owned())
}

fn stored(definition: &Definition, raw: Json, dropped: Vec<String>) -> Mapped {
    match canonicalize(definition.core_type, definition, raw) {
        Ok(canonical) => match serde_json::from_str(&canonical.0) {
            Ok(value) => Mapped::Value { value, dropped },
            Err(_) => fail("the value has no stored form"),
        },
        Err(_) => fail("the value is not valid for this field"),
    }
}

/// The unique choice for a stated name: byte-equal first, else the unique choice with the same
/// name key. An empty name key never matches.
fn choice_for<'a>(definition: &'a Definition, text: &str) -> Option<&'a String> {
    if let Some(exact) = definition.choices.iter().find(|c| c.as_str() == text) {
        return Some(exact);
    }
    let key = name_key(text);
    if key.is_empty() {
        return None;
    }
    let mut found = definition.choices.iter().filter(|c| name_key(c) == key);
    match (found.next(), found.next()) {
        (Some(choice), None) => Some(choice),
        _ => None,
    }
}

/// A decimal written as a JSON number or a string of digits with an optional `.`; never an
/// exponent, a thousands separator or a decimal comma. Returns the MC-9 canonical decimal.
fn decimal_text(raw: &Json) -> Option<String> {
    let text = match raw {
        Json::Number(n) => n.to_string(),
        Json::String(s) => s.trim().to_owned(),
        _ => return None,
    };
    if text
        .bytes()
        .any(|b| !(b.is_ascii_digit() || b == b'.' || b == b'-'))
    {
        return None;
    }
    canonical_decimal(&text)
}

/// `(numerator, scale)` of a canonical decimal string: value = numerator / 10^scale.
fn exact(decimal: &str) -> Option<(i128, u32)> {
    let negative = decimal.starts_with('-');
    let digits = decimal.trim_start_matches('-');
    let (whole, fraction) = digits.split_once('.').unwrap_or((digits, ""));
    let n: i128 = format!("{whole}{fraction}").parse().ok()?;
    let scale = u32::try_from(fraction.len()).ok()?;
    Some((if negative { -n } else { n }, scale))
}

/// The size of one neutral unit in the base unit (mm, g), as a fraction.
fn factor(core_type: CoreType, neutral: &str) -> Option<(i128, i128)> {
    match (core_type, neutral) {
        (CoreType::Dimension, "mm") => Some((1, 1)),
        (CoreType::Dimension, "cm") => Some((10, 1)),
        (CoreType::Dimension, "m") => Some((1000, 1)),
        (CoreType::Dimension, "in") => Some((254, 10)),
        (CoreType::Weight, "g") => Some((1, 1)),
        (CoreType::Weight, "kg") => Some((1000, 1)),
        (CoreType::Weight, "lb") => Some((45_359_237, 100_000)),
        _ => None,
    }
}

/// `numerator / denominator` rounded half-even to an integer (`denominator > 0`).
fn round_half_even(numerator: i128, denominator: i128) -> i128 {
    let quotient = numerator.div_euclid(denominator);
    let remainder = numerator.rem_euclid(denominator);
    match (2 * remainder).cmp(&denominator) {
        std::cmp::Ordering::Less => quotient,
        std::cmp::Ordering::Greater => quotient + 1,
        std::cmp::Ordering::Equal => {
            if quotient % 2 == 0 {
                quotient
            } else {
                quotient + 1
            }
        }
    }
}

/// Tenths as a decimal string with one fractional digit (`254` → `25.4`, `-5` → `-0.5`).
fn tenths_text(tenths: i128) -> String {
    let sign = if tenths < 0 { "-" } else { "" };
    let magnitude = tenths.unsigned_abs();
    format!("{sign}{}.{}", magnitude / 10, magnitude % 10)
}

fn measurement(definition: &Definition, raw: &Json) -> Mapped {
    let Some(object) = raw.as_object() else {
        return fail("a measurement needs a value and a unit");
    };
    let Some(unit) = object.get("unit").and_then(Json::as_str) else {
        return fail("a measurement needs a value and a unit");
    };
    let Some(source) = factor(definition.core_type, unit.trim()) else {
        return fail("the unit is not one research can convert");
    };
    let Some((numerator, scale)) = object
        .get("value")
        .and_then(decimal_text)
        .and_then(|d| exact(&d))
    else {
        return fail("the measurement's value is not a number");
    };
    let Some(first_code) = definition.units().and_then(|u| u.first().copied()) else {
        return fail("the field lists no unit");
    };
    let names = definition.validations.get("unit_names");
    let target = ["mm", "cm", "m", "in", "g", "kg", "lb"]
        .into_iter()
        .filter(|n| factor(definition.core_type, n).is_some())
        .find(|n| names.and_then(|m| m.get(n)).and_then(Json::as_str) == Some(first_code));
    let Some(target) = target.and_then(|n| factor(definition.core_type, n)) else {
        return fail("the field's unit has no known name");
    };
    // value × source / target, in tenths: n × 10 × s_num × t_den / (10^scale × s_den × t_num).
    let Some(power) = 10_i128.checked_pow(scale) else {
        return fail("the measurement's value is too long");
    };
    let top = numerator
        .checked_mul(10)
        .and_then(|v| v.checked_mul(source.0))
        .and_then(|v| v.checked_mul(target.1));
    let bottom = power
        .checked_mul(source.1)
        .and_then(|v| v.checked_mul(target.0));
    let (Some(top), Some(bottom)) = (top, bottom) else {
        return fail("the measurement's value is too large");
    };
    let tenths = round_half_even(top, bottom);
    let Some(value) = canonical_decimal(&tenths_text(tenths)) else {
        return fail("the measurement's value is not a number");
    };
    stored(
        definition,
        serde_json::json!({ "value": value, "unit": first_code }),
        Vec::new(),
    )
}

/// MC-A2's mapping table: the model's output for one definition → its stored value, or a fail.
pub fn map_output(definition: &Definition, output: &Json) -> Mapped {
    use CoreType as T;
    match definition.core_type {
        T::Choice => match output.as_str().and_then(|s| choice_for(definition, s)) {
            Some(choice) => stored(definition, Json::from(choice.clone()), Vec::new()),
            None => fail("the answer is not one of the field's choices"),
        },
        T::MultiChoice => {
            let Some(items) = output.as_array() else {
                return fail("the answer is not a list");
            };
            let mut picked: Vec<&String> = Vec::new();
            let mut dropped: Vec<String> = Vec::new();
            for item in items {
                let Some(text) = item.as_str() else {
                    return fail("the answer's list holds something that is not text");
                };
                match choice_for(definition, text) {
                    Some(choice) => picked.push(choice),
                    None => dropped.push(text.to_owned()),
                }
            }
            let ordered: Vec<Json> = definition
                .choices
                .iter()
                .filter(|c| picked.contains(c))
                .map(|c| Json::from(c.clone()))
                .collect();
            if ordered.is_empty() {
                return fail("no element of the answer is one of the field's choices");
            }
            stored(definition, Json::Array(ordered), dropped)
        }
        T::Text | T::Textarea => match output.as_str().map(str::trim) {
            Some(text) if !text.is_empty() => stored(definition, Json::from(text), Vec::new()),
            _ => fail("the answer is empty"),
        },
        T::Url => match output.as_str().map(str::trim) {
            Some(text) if super::sites::host(text).is_some() => {
                stored(definition, Json::from(text), Vec::new())
            }
            _ => fail("the answer is not an http or https address"),
        },
        T::Integer => {
            let parsed = match output {
                Json::Number(n) => n.as_i64(),
                Json::String(s) => {
                    let s = s.trim();
                    let digits = s.strip_prefix('-').unwrap_or(s);
                    if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
                        s.parse::<i64>().ok()
                    } else {
                        None
                    }
                }
                _ => None,
            };
            match parsed {
                Some(n) => stored(definition, Json::from(n), Vec::new()),
                None => fail("the answer is not a whole number"),
            }
        }
        T::Decimal => match decimal_text(output) {
            Some(text) => stored(definition, Json::from(text), Vec::new()),
            None => fail("the answer is not a number"),
        },
        T::Dimension | T::Weight => measurement(definition, output),
        T::Boolean => match output {
            Json::Bool(_) => stored(definition, output.clone(), Vec::new()),
            _ => fail("the answer is not true or false"),
        },
        T::Date | T::Color => match output.as_str() {
            Some(text) => stored(definition, Json::from(text.trim()), Vec::new()),
            None => fail("the answer is not text"),
        },
        T::Tags | T::RichText | T::Money | T::Rating | T::MediaList | T::Reference => {
            fail("research does not fill this kind of field")
        }
    }
}

/// One stated text for a key → its mapped value, or its name key when it maps to nothing.
pub fn statement(definition: &Definition, stated: &str) -> Stated {
    match map_output(definition, &Json::from(stated)) {
        Mapped::Value { value, .. } => Stated::Mapped(value),
        Mapped::Fail(_) => Stated::Unmapped(name_key(stated)),
    }
}

/// Two statements name the same value: mapped values by sameness, unmapped ones by name key.
pub fn same_statement(definition: &Definition, a: &Stated, b: &Stated) -> bool {
    match (a, b) {
        (Stated::Mapped(a), Stated::Mapped(b)) => same(definition, a, b),
        (Stated::Unmapped(a), Stated::Unmapped(b)) => a == b,
        _ => false,
    }
}

/// Everything one page states for a key → the single statement, or none when the page states two
/// values that are not the same (or nothing at all).
pub fn page_statement(definition: &Definition, texts: &[&str]) -> Option<Stated> {
    let statements: Vec<Stated> = texts.iter().map(|t| statement(definition, t)).collect();
    let first = statements.first()?;
    statements
        .iter()
        .all(|s| same_statement(definition, first, s))
        .then(|| first.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use psm_core::domain::attribute::AttributeGroup;
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

    fn plain(core_type: CoreType) -> Definition {
        def(core_type, &[], json!({}))
    }

    fn value(mapped: Mapped) -> Json {
        match mapped {
            Mapped::Value { value, .. } => value,
            Mapped::Fail(why) => panic!("expected a value, got fail: {why}"),
        }
    }

    fn fails(mapped: Mapped) -> bool {
        matches!(mapped, Mapped::Fail(_))
    }

    fn length() -> Definition {
        def(
            CoreType::Dimension,
            &[],
            json!({"units": ["CENTIMETERS", "MILLIMETERS"],
                   "unit_names": {"cm": "CENTIMETERS", "mm": "MILLIMETERS"}}),
        )
    }

    #[test]
    fn a03_c2a_choice_byte_equal() {
        let d = def(CoreType::Choice, &["Oak", "Teak"], json!({}));
        assert_eq!(value(map_output(&d, &json!("Teak"))), json!("Teak"));
    }

    #[test]
    fn a03_c2b_choice_by_name_key() {
        let d = def(CoreType::Choice, &["Oak", "Teak"], json!({}));
        assert_eq!(value(map_output(&d, &json!("teak "))), json!("Teak"));
    }

    #[test]
    fn a03_c2c_byte_equal_wins_shared_key_fails() {
        let d = def(CoreType::Choice, &["Oak", "oak"], json!({}));
        assert_eq!(value(map_output(&d, &json!("oak"))), json!("oak"));
        assert!(fails(map_output(&d, &json!("OAK"))));
    }

    #[test]
    fn a03_c2d_choice_unknown_or_not_text() {
        let d = def(CoreType::Choice, &["Oak", "Teak"], json!({}));
        assert!(fails(map_output(&d, &json!("Pine"))));
        assert!(fails(map_output(&d, &json!(12))));
    }

    #[test]
    fn a03_c3a_multi_choice_order_and_dropped() {
        let d = def(
            CoreType::MultiChoice,
            &["Oak", "Teak", "Rosewood"],
            json!({}),
        );
        assert_eq!(
            map_output(&d, &json!(["teak", "Oak", "Plastic"])),
            Mapped::Value {
                value: json!(["Oak", "Teak"]),
                dropped: vec!["Plastic".to_owned()],
            }
        );
    }

    #[test]
    fn a03_c3b_multi_choice_fails() {
        let d = def(
            CoreType::MultiChoice,
            &["Oak", "Teak", "Rosewood"],
            json!({}),
        );
        assert!(fails(map_output(&d, &json!(["Plastic"]))));
        assert!(fails(map_output(&d, &json!([]))));
        assert!(fails(map_output(&d, &json!("Teak"))));
    }

    #[test]
    fn a03_c4a_text() {
        let d = plain(CoreType::Text);
        assert_eq!(
            value(map_output(&d, &json!("  Oak table "))),
            json!("Oak table")
        );
        assert!(fails(map_output(&d, &json!("   "))));
    }

    #[test]
    fn a03_c4b_url() {
        let d = plain(CoreType::Url);
        assert_eq!(
            value(map_output(&d, &json!("https://a.example/x"))),
            json!("https://a.example/x")
        );
        assert!(fails(map_output(&d, &json!("ftp://a.example"))));
        assert!(fails(map_output(&d, &json!("a.example/x"))));
    }

    #[test]
    fn a03_c4c_integer() {
        let d = plain(CoreType::Integer);
        assert_eq!(value(map_output(&d, &json!(12))), json!(12));
        assert_eq!(value(map_output(&d, &json!("12"))), json!(12));
        assert!(fails(map_output(&d, &json!(12.5))));
        assert!(fails(map_output(&d, &json!("1e3"))));
        assert!(fails(map_output(&d, &json!("1,200"))));
    }

    #[test]
    fn a03_c4d_decimal() {
        let d = plain(CoreType::Decimal);
        assert_eq!(value(map_output(&d, &json!("12.5"))), json!("12.5"));
        assert_eq!(value(map_output(&d, &json!(12.5))), json!("12.5"));
        assert!(fails(map_output(&d, &json!("12,5"))));
    }

    #[test]
    fn a03_c4e_boolean_date_color() {
        assert_eq!(
            value(map_output(&plain(CoreType::Boolean), &json!(true))),
            json!(true)
        );
        let date = plain(CoreType::Date);
        assert!(fails(map_output(&date, &json!("2026-02-30"))));
        assert_eq!(
            value(map_output(&date, &json!("2026-02-28"))),
            json!("2026-02-28")
        );
        let color = plain(CoreType::Color);
        assert_eq!(
            value(map_output(&color, &json!("#A1B2C3"))),
            json!("#a1b2c3")
        );
        assert!(fails(map_output(&color, &json!("red"))));
    }

    #[test]
    fn a03_c5a_inches_to_centimetres() {
        assert_eq!(
            value(map_output(&length(), &json!({"value": 10, "unit": "in"}))),
            json!({"value": "25.4", "unit": "CENTIMETERS"})
        );
    }

    #[test]
    fn a03_c5b_half_even_at_one_decimal() {
        assert_eq!(
            value(map_output(
                &length(),
                &json!({"value": "2.5", "unit": "mm"})
            )),
            json!({"value": "0.2", "unit": "CENTIMETERS"})
        );
        assert_eq!(
            value(map_output(
                &length(),
                &json!({"value": "3.5", "unit": "mm"})
            )),
            json!({"value": "0.4", "unit": "CENTIMETERS"})
        );
    }

    #[test]
    fn a03_c5c_pounds_to_kilograms() {
        let d = def(
            CoreType::Weight,
            &[],
            json!({"units": ["KILOGRAMS"], "unit_names": {"kg": "KILOGRAMS"}}),
        );
        assert_eq!(
            value(map_output(&d, &json!({"value": 1, "unit": "lb"}))),
            json!({"value": "0.5", "unit": "KILOGRAMS"})
        );
    }

    #[test]
    fn a03_c5d_measurement_fails() {
        assert!(fails(map_output(
            &length(),
            &json!({"value": 10, "unit": "ft"})
        )));
        let unnamed = def(
            CoreType::Dimension,
            &[],
            json!({"units": ["CENTIMETERS"], "unit_names": {"mm": "MILLIMETERS"}}),
        );
        assert!(fails(map_output(
            &unnamed,
            &json!({"value": 10, "unit": "cm"})
        )));
        assert!(fails(map_output(&length(), &json!({"value": 10}))));
    }

    #[test]
    fn a03_c6a_statement_maps_by_name_key() {
        let d = def(CoreType::Choice, &["Hans J. Wegner"], json!({}));
        assert_eq!(
            statement(&d, "Hans J Wegner"),
            Stated::Mapped(json!("Hans J. Wegner"))
        );
    }

    #[test]
    fn a03_c6b_statement_unmapped() {
        let d = def(CoreType::Choice, &["Hans J. Wegner"], json!({}));
        assert_eq!(
            statement(&d, "Unknown Maker"),
            Stated::Unmapped("unknown maker".to_owned())
        );
    }

    #[test]
    fn a03_c6c_page_with_two_values_states_none() {
        let d = plain(CoreType::Text);
        assert_eq!(page_statement(&d, &["Wegner", "Mogensen"]), None);
        assert_eq!(
            page_statement(&d, &["Wegner", "WEGNER"]),
            Some(Stated::Mapped(json!("Wegner")))
        );
    }
}
