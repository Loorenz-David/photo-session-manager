//! MC-A1: the fields research never writes, the write table per stored state (fail closed), the
//! cleared set's effect, and the conflict a prior value defines.

use psm_core::domain::attribute::{
    AiPrior, AiStatus, AttributeGroup, CoreType, Definition, Provenance, Value,
};
use psm_core::mechanisms::sameness::same;
use serde_json::Value as Json;

/// The built-in title field: the item's own title, never researched.
const PRODUCT_TITLE: &str = "product.title";

/// MC-A1 "Fields never written by research". `step` is the layout step holding the key in the
/// item's current layout (`None`: not in the layout).
pub fn never_written(definition: &Definition, step: Option<AttributeGroup>) -> bool {
    step.is_none()
        || step == Some(AttributeGroup::Product)
        || definition.group == AttributeGroup::Product
        || matches!(
            definition.core_type,
            CoreType::MediaList
                | CoreType::Reference
                | CoreType::RichText
                | CoreType::Money
                | CoreType::Rating
        )
        || !definition.writable
        || definition.key == PRODUCT_TITLE
}

/// What the write table allows for one proposed value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteDecision {
    /// Write the value as `ai` + `suggested`, with this `ai_prior`.
    Write { ai_prior: Option<AiPrior> },
    /// The value is in the cleared set: treated as not determined.
    MatchesCleared,
    /// A live `default` or `integration` row already holds the same value.
    AlreadyHolds,
    /// The table forbids any research write over the stored row.
    Refused(&'static str),
}

fn parsed(text: &str) -> Json {
    serde_json::from_str(text).unwrap_or_else(|_| Json::String(text.to_owned()))
}

/// MC-A1 write table for the stored row (read in the writing transaction), the proposed value
/// (already mapped by MC-A2) and the cleared set `C(item, key)`. Every combination the table does
/// not name is refused.
pub fn decide_write(
    definition: &Definition,
    stored: Option<&Value>,
    proposed: &Json,
    cleared: &[Json],
) -> WriteDecision {
    use AiStatus::{Accepted, Cleared, Suggested};
    use Provenance::{Ai, Default, Human, Integration};
    let ai_prior = match stored.map(|s| (s.provenance, s.ai_status)) {
        None => None,
        Some((Default | Integration, None)) => stored.map(|s| AiPrior {
            value: s.value.clone(),
            valid: s.valid,
            provenance: s.provenance,
        }),
        Some((Ai, Some(Suggested))) => stored.and_then(|s| s.ai_prior.clone()),
        Some((Ai | Default | Integration, Some(Cleared))) => None,
        Some((Ai, Some(Accepted))) => {
            return WriteDecision::Refused("the photographer accepted this value");
        }
        Some((Human, Some(Cleared))) => {
            return WriteDecision::Refused("the photographer cleared this field");
        }
        Some((Human, _)) => return WriteDecision::Refused("the photographer wrote this value"),
        Some(_) => return WriteDecision::Refused("the stored value cannot be changed by research"),
    };
    if cleared.iter().any(|c| same(definition, c, proposed)) {
        return WriteDecision::MatchesCleared;
    }
    if let Some(s) = stored
        && matches!(s.provenance, Default | Integration)
        && s.ai_status.is_none()
        && same(definition, &parsed(&s.value), proposed)
    {
        return WriteDecision::AlreadyHolds;
    }
    WriteDecision::Write { ai_prior }
}

/// A conflict: `ai` + `suggested`, with a prior, and the value not the same as the prior's.
pub fn is_conflict(definition: &Definition, row: &Value) -> bool {
    row.provenance == Provenance::Ai
        && row.ai_status == Some(AiStatus::Suggested)
        && row
            .ai_prior
            .as_ref()
            .is_some_and(|prior| !same(definition, &parsed(&row.value), &parsed(&prior.value)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use psm_core::domain::attribute::AiConfidence;
    use serde_json::json;

    fn def(core_type: CoreType, group: AttributeGroup) -> Definition {
        Definition {
            integration: "shop".into(),
            key: "custom.maker".into(),
            label: "Maker".into(),
            core_type,
            choices: vec![],
            validations: json!({}),
            group,
            description: None,
            writable: true,
        }
    }

    fn text() -> Definition {
        def(CoreType::Text, AttributeGroup::Details)
    }

    fn row(value: &str, provenance: Provenance, status: Option<AiStatus>) -> Value {
        Value {
            item_id: 1,
            key: "custom.maker".into(),
            value: json!(value).to_string(),
            valid: true,
            provenance,
            ai_confidence: status.map(|_| AiConfidence::Medium),
            ai_status: status,
            ai_reason: None,
            ai_sources: None,
            ai_prior: None,
            updated_at: 0,
        }
    }

    #[test]
    fn a07_c5_never_written_fields() {
        let details = Some(AttributeGroup::Details);
        assert!(!never_written(&text(), details));
        assert!(never_written(&text(), None));
        assert!(never_written(&text(), Some(AttributeGroup::Product)));
        for t in [
            CoreType::MediaList,
            CoreType::Reference,
            CoreType::RichText,
            CoreType::Money,
            CoreType::Rating,
        ] {
            assert!(
                never_written(&def(t, AttributeGroup::Details), details),
                "{t:?}"
            );
        }
        let mut locked = text();
        locked.writable = false;
        assert!(never_written(&locked, details));
        let mut title = text();
        title.key = PRODUCT_TITLE.into();
        assert!(never_written(&title, details));
    }

    #[test]
    fn a07_c1_decide_write_table() {
        let d = text();
        let a = json!("A");
        let w = |stored: Option<&Value>| decide_write(&d, stored, &a, &[]);
        assert_eq!(w(None), WriteDecision::Write { ai_prior: None });
        assert_eq!(
            w(Some(&row("D", Provenance::Default, None))),
            WriteDecision::Write {
                ai_prior: Some(AiPrior {
                    value: "\"D\"".into(),
                    valid: true,
                    provenance: Provenance::Default
                })
            }
        );
        assert!(matches!(
            w(Some(&row("X", Provenance::Ai, Some(AiStatus::Accepted)))),
            WriteDecision::Refused(_)
        ));
        assert!(matches!(
            w(Some(&row("X", Provenance::Human, None))),
            WriteDecision::Refused(_)
        ));
        assert!(matches!(
            w(Some(&row("X", Provenance::Human, Some(AiStatus::Cleared)))),
            WriteDecision::Refused(_)
        ));
        assert!(matches!(
            w(Some(&row("X", Provenance::Ai, None))),
            WriteDecision::Refused(_)
        ));
        assert!(matches!(
            w(Some(&row(
                "X",
                Provenance::Default,
                Some(AiStatus::Suggested)
            ))),
            WriteDecision::Refused(_)
        ));
        let cleared = row("A", Provenance::Ai, Some(AiStatus::Cleared));
        assert_eq!(
            decide_write(&d, Some(&cleared), &a, &[json!("a")]),
            WriteDecision::MatchesCleared
        );
        assert_eq!(
            decide_write(&d, Some(&row("a ", Provenance::Default, None)), &a, &[]),
            WriteDecision::AlreadyHolds
        );
    }

    #[test]
    fn a07_c2_conflict_needs_a_differing_prior() {
        let d = text();
        let mut r = row("A", Provenance::Ai, Some(AiStatus::Suggested));
        assert!(!is_conflict(&d, &r));
        r.ai_prior = Some(AiPrior {
            value: "\"D\"".into(),
            valid: true,
            provenance: Provenance::Default,
        });
        assert!(is_conflict(&d, &r));
        r.value = "\"d\"".into();
        assert!(!is_conflict(&d, &r));
    }
}
