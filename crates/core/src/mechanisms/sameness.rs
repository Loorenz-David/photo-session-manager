//! MC-A1: equality for a suggested value and a stored value.

use std::collections::BTreeSet;

use serde_json::Value as Json;

use crate::domain::attribute::{CoreType, Definition};
use crate::mechanisms::{name_key::name_key, values::canonical_json};

pub fn same(definition: &Definition, a: &Json, b: &Json) -> bool {
    match definition.core_type {
        CoreType::Text | CoreType::Textarea | CoreType::Url => match (a.as_str(), b.as_str()) {
            (Some(a), Some(b)) => name_key(a) == name_key(b),
            _ => false,
        },
        CoreType::MultiChoice | CoreType::Tags => {
            let keys = |value: &Json| -> Option<BTreeSet<String>> {
                value
                    .as_array()?
                    .iter()
                    .map(|v| {
                        let s = v.as_str()?;
                        Some(if definition.core_type == CoreType::Tags {
                            name_key(s)
                        } else {
                            s.to_owned()
                        })
                    })
                    .collect()
            };
            match (keys(a), keys(b)) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            }
        }
        CoreType::Dimension | CoreType::Weight => same_measurement(definition, a, b),
        _ => canonical_json(a) == canonical_json(b),
    }
}

fn decimal(value: &str) -> Option<(i128, i128)> {
    let negative = value.starts_with('-');
    let digits = value.strip_prefix('-').unwrap_or(value);
    let (whole, fraction) = digits.split_once('.').unwrap_or((digits, ""));
    if whole.is_empty()
        || !whole.bytes().all(|c| c.is_ascii_digit())
        || !fraction.bytes().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    let n: i128 = format!("{whole}{fraction}").parse().ok()?;
    let d = 10_i128.checked_pow(u32::try_from(fraction.len()).ok()?)?;
    Some((if negative { -n } else { n }, d))
}

fn factor(ty: CoreType, neutral: &str) -> Option<(i128, i128)> {
    match (ty, neutral) {
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

fn measurement(value: &Json) -> Option<(&str, (i128, i128))> {
    Some((
        value.get("unit")?.as_str()?,
        decimal(value.get("value")?.as_str()?)?,
    ))
}

fn same_measurement(definition: &Definition, a: &Json, b: &Json) -> bool {
    let (Some((a_unit, (a_num, a_den))), Some((b_unit, (b_num, b_den)))) =
        (measurement(a), measurement(b))
    else {
        return false;
    };
    if a_unit == b_unit {
        return a_num.checked_mul(b_den) == b_num.checked_mul(a_den);
    }
    let Some(names) = definition.validations.get("unit_names") else {
        return false;
    };
    let neutral = |code: &str| -> Option<&str> {
        ["mm", "cm", "m", "in", "g", "kg", "lb"]
            .into_iter()
            .find(|name| names.get(name).and_then(Json::as_str) == Some(code))
    };
    let (Some((a_factor, a_factor_den)), Some((b_factor, b_factor_den))) = (
        neutral(a_unit).and_then(|n| factor(definition.core_type, n)),
        neutral(b_unit).and_then(|n| factor(definition.core_type, n)),
    ) else {
        return false;
    };
    let left = a_num
        .checked_mul(a_factor)
        .and_then(|n| n.checked_mul(b_den))
        .and_then(|n| n.checked_mul(b_factor_den));
    let right = b_num
        .checked_mul(b_factor)
        .and_then(|n| n.checked_mul(a_den))
        .and_then(|n| n.checked_mul(a_factor_den));
    left.is_some() && left == right
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::attribute::AttributeGroup;
    use serde_json::json;

    fn def(core_type: CoreType) -> Definition {
        Definition {
            integration: "test".into(),
            key: "k".into(),
            label: "K".into(),
            core_type,
            choices: vec![],
            validations: json!({}),
            group: AttributeGroup::Details,
            description: None,
            writable: true,
        }
    }
    #[test]
    fn a03_c7a_text() {
        let d = def(CoreType::Text);
        assert!(same(&d, &json!("Hans J. Wegner"), &json!("hans j wegner")));
        assert!(!same(&d, &json!("Hans J. Wegner"), &json!("Hans Wegner")));
    }
    #[test]
    fn a03_c7b_choice() {
        let d = def(CoreType::Choice);
        assert!(same(&d, &json!("Teak"), &json!("Teak")));
        assert!(!same(&d, &json!("Teak"), &json!("teak")));
    }
    #[test]
    fn a03_c7c_multi_choice() {
        let d = def(CoreType::MultiChoice);
        assert!(same(&d, &json!(["Oak", "Teak"]), &json!(["Teak", "Oak"])));
        assert!(!same(&d, &json!(["Oak"]), &json!(["Oak", "Teak"])));
    }
    #[test]
    fn a03_c7d_dimension() {
        let mut d = def(CoreType::Dimension);
        d.validations = json!({"unit_names":{"mm":"millimetre", "cm":"centimetre"}});
        assert!(same(
            &d,
            &json!({"value":"25.4","unit":"millimetre"}),
            &json!({"value":"2.54","unit":"centimetre"})
        ));
        assert!(!same(
            &d,
            &json!({"value":"25","unit":"millimetre"}),
            &json!({"value":"2.54","unit":"centimetre"})
        ));
    }
    #[test]
    fn a03_c7e_boolean_integer() {
        assert!(same(&def(CoreType::Boolean), &json!(true), &json!(true)));
        assert!(!same(&def(CoreType::Integer), &json!(12), &json!(13)));
    }
}
