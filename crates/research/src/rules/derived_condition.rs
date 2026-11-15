//! Detailed condition derived from the saved condition levels, best first.

use psm_core::domain::attribute::{AiConfidence, Definition};
use serde_json::Value as Json;

use crate::{
    config::ConditionLevel,
    rules::value_map::{Mapped, map_output},
};

#[derive(Debug, Clone)]
pub struct LevelValue {
    pub label: String,
    pub value: Option<String>,
    pub ai_low: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Derived {
    pub value: Option<Json>,
    pub confidence: Option<AiConfidence>,
    pub reason: String,
}

fn undetermined(reason: impl Into<String>) -> Derived {
    Derived {
        value: None,
        confidence: None,
        reason: reason.into(),
    }
}

/// The latest applicable level wins. Missing or unknown levels refuse a grade rather than
/// guessing; a saved choice is remapped against the field's current choices.
pub fn derive(levels: &[ConditionLevel], values: &[LevelValue], field: &Definition) -> Derived {
    if values.is_empty() {
        return undetermined("no condition levels in this form");
    }
    let missing: Vec<&str> = values
        .iter()
        .filter(|v| v.value.is_none())
        .map(|v| v.label.as_str())
        .collect();
    if !missing.is_empty() {
        return undetermined(format!(
            "condition levels are incomplete: {}",
            missing.join(", ")
        ));
    }
    let mut worst: Option<(usize, &str)> = None;
    for value in values {
        let text = value.value.as_deref().unwrap_or_default();
        let Some(index) = levels.iter().position(|level| level.level == text) else {
            return undetermined(format!("no grade is set for {text}"));
        };
        if levels[index].choice.is_some() && worst.is_none_or(|(prior, _)| index > prior) {
            worst = Some((index, text));
        }
    }
    let Some((index, level)) = worst else {
        return undetermined("no condition level applies");
    };
    let choice = levels[index].choice.as_deref().unwrap_or_default();
    let Mapped::Value { value, .. } = map_output(field, &Json::String(choice.into())) else {
        return undetermined("the setting names a choice the field does not have");
    };
    let labels = values
        .iter()
        .filter(|v| v.value.as_deref() == Some(level))
        .map(|v| v.label.as_str())
        .collect::<Vec<_>>();
    let confidence = if values.iter().any(|v| v.ai_low) {
        AiConfidence::Low
    } else {
        AiConfidence::Medium
    };
    Derived {
        value: Some(value),
        confidence: Some(confidence),
        reason: format!(
            "From the condition levels: the worst is {level} ({}).",
            labels.join(", ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use psm_core::domain::attribute::{AttributeGroup, CoreType};

    fn field() -> Definition {
        Definition {
            integration: "shop".into(),
            key: "custom.grade".into(),
            label: "Detailed condition".into(),
            core_type: CoreType::Choice,
            choices: ["A", "B", "C", "D"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            validations: Json::Object(Default::default()),
            group: AttributeGroup::Details,
            description: None,
            writable: true,
        }
    }
    fn table() -> Vec<ConditionLevel> {
        [
            ("None", Some("A")),
            ("Minor", Some("B")),
            ("Moderate", Some("C")),
            ("Major", Some("D")),
            ("N/A", None),
        ]
        .into_iter()
        .map(|(level, choice)| ConditionLevel {
            level: level.into(),
            choice: choice.map(str::to_owned),
        })
        .collect()
    }
    fn values(names: &[&str]) -> Vec<LevelValue> {
        names
            .iter()
            .enumerate()
            .map(|(i, v)| LevelValue {
                label: format!("Field {i}"),
                value: Some((*v).into()),
                ai_low: false,
            })
            .collect()
    }
    fn got(names: &[&str]) -> Derived {
        derive(&table(), &values(names), &field())
    }
    #[test]
    fn a25_c12a_none_none() {
        assert_eq!(
            (
                got(&["None", "None"]).value,
                got(&["None", "None"]).confidence
            ),
            (Some(Json::String("A".into())), Some(AiConfidence::Medium))
        );
    }
    #[test]
    fn a25_c12b_none_minor() {
        assert_eq!(
            (
                got(&["None", "Minor"]).value,
                got(&["None", "Minor"]).confidence
            ),
            (Some(Json::String("B".into())), Some(AiConfidence::Medium))
        );
    }
    #[test]
    fn a25_c12c_minor_moderate_none() {
        assert_eq!(
            (
                got(&["Minor", "Moderate", "None"]).value,
                got(&["Minor", "Moderate", "None"]).confidence
            ),
            (Some(Json::String("C".into())), Some(AiConfidence::Medium))
        );
    }
    #[test]
    fn a25_c12d_moderate_major() {
        assert_eq!(
            (
                got(&["Moderate", "Major"]).value,
                got(&["Moderate", "Major"]).confidence
            ),
            (Some(Json::String("D".into())), Some(AiConfidence::Medium))
        );
    }
    #[test]
    fn a25_c12e_na_minor() {
        assert_eq!(
            (
                got(&["N/A", "Minor"]).value,
                got(&["N/A", "Minor"]).confidence
            ),
            (Some(Json::String("B".into())), Some(AiConfidence::Medium))
        );
    }
    #[test]
    fn a25_c12f_only_na() {
        assert_eq!(got(&["N/A", "N/A"]).reason, "no condition level applies");
    }
    #[test]
    fn a25_c12g_missing_value_names_label() {
        let mut v = values(&["Minor"]);
        v.push(LevelValue {
            label: "Veneer".into(),
            value: None,
            ai_low: false,
        });
        let result = derive(&table(), &v, &field());
        assert!(result.value.is_none());
        assert!(result.reason.contains("Veneer"));
    }
    #[test]
    fn a25_c12h_low_level_lowers_grade() {
        let mut v = values(&["None", "Minor"]);
        v[1].ai_low = true;
        let result = derive(&table(), &v, &field());
        assert_eq!(
            (result.value, result.confidence),
            (Some(Json::String("B".into())), Some(AiConfidence::Low))
        );
    }
    #[test]
    fn a25_c12i_unknown_level_refuses() {
        let t = table()
            .into_iter()
            .filter(|r| r.level != "Major")
            .collect::<Vec<_>>();
        assert_eq!(
            derive(&t, &values(&["Minor", "Major"]), &field()).reason,
            "no grade is set for Major"
        );
    }
    #[test]
    fn a25_c12j_no_level_fields() {
        assert_eq!(got(&[]).reason, "no condition levels in this form");
    }
    #[test]
    fn a25_c12k_missing_grade_choice() {
        let mut t = table();
        t[1].choice = Some("No such choice".into());
        assert_eq!(
            derive(&t, &values(&["Minor"]), &field()).reason,
            "the setting names a choice the field does not have"
        );
    }
}
