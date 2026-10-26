//! Confirmed attribute diff and product-tag merge (07A MC-13).
use crate::{
    domain::attribute::{CoreType, Definition},
    mechanisms::values::canonicalize,
};
use serde_json::Value as Json;
use std::collections::{BTreeMap, HashSet};

pub fn write_set(
    frozen: &BTreeMap<String, Json>,
    definitions: &[Definition],
    remote: &[(String, Json)],
) -> Vec<(String, Json)> {
    definitions
        .iter()
        .filter_map(|definition| {
            if !definition.writable
                || matches!(
                    definition.core_type,
                    CoreType::Reference | CoreType::MediaList
                )
            {
                return None;
            }
            let local = frozen.get(&definition.key)?;
            let local = canonicalize(definition.core_type, definition, local.clone()).ok()?;
            let remote_value = remote.iter().find(|(key, _)| key == &definition.key);
            let remote = remote_value.and_then(|(_, value)| {
                canonicalize(definition.core_type, definition, value.clone()).ok()
            });
            if definition.core_type == CoreType::RichText
                && remote_value.is_some()
                && remote.is_none()
            {
                return None;
            }
            if remote.as_ref() == Some(&local) {
                return None;
            }
            serde_json::from_str(&local.0)
                .ok()
                .map(|value| (definition.key.clone(), value))
        })
        .collect()
}

pub fn merge_tags(base: &[String], local: &[String], remote_now: &[String]) -> Option<Vec<String>> {
    let removed: HashSet<&str> = base
        .iter()
        .filter(|tag| !local.contains(tag))
        .map(String::as_str)
        .collect();
    let base_set: HashSet<&str> = base.iter().map(String::as_str).collect();
    let mut result: Vec<String> = remote_now
        .iter()
        .filter(|tag| !removed.contains(tag.as_str()))
        .cloned()
        .collect();
    for tag in local {
        if !base_set.contains(tag.as_str()) && !result.contains(tag) {
            result.push(tag.clone());
        }
    }
    (result != remote_now).then_some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::attribute::AttributeGroup;
    #[test]
    fn mc13_write_set_canonical_and_writable() {
        let definitions = ["years", "country", "designer", "reference"]
            .into_iter()
            .map(|key| Definition {
                integration: "shopify".into(),
                key: key.into(),
                label: key.into(),
                core_type: if key == "reference" {
                    CoreType::Reference
                } else {
                    CoreType::Text
                },
                choices: vec![],
                validations: Json::Null,
                group: AttributeGroup::Details,
                description: None,
                writable: true,
            })
            .collect::<Vec<_>>();
        let frozen = BTreeMap::from([
            ("years".into(), Json::from("1950-1960s")),
            ("country".into(), Json::from("Sweden")),
            ("reference".into(), Json::from("x")),
        ]);
        let remote = vec![
            ("years".into(), Json::from("1950-1960s")),
            ("country".into(), Json::from("Denmark")),
            ("designer".into(), Json::from("X")),
        ];
        assert_eq!(
            write_set(&frozen, &definitions, &remote),
            vec![("country".into(), Json::from("Sweden"))]
        );
    }
    #[test]
    fn mc13_tags_preserve_remote_additions_and_local_removals() {
        let base = vec!["a".into(), "b".into()];
        assert_eq!(
            merge_tags(&base, &base, &["a".into(), "b".into(), "c".into()]),
            None
        );
        assert_eq!(
            merge_tags(&base, &["a".into()], &["a".into(), "b".into(), "c".into()]),
            Some(vec!["a".into(), "c".into()])
        );
        assert_eq!(
            merge_tags(&[], &["x".into()], &["y".into()]),
            Some(vec!["y".into(), "x".into()])
        );
    }
}
