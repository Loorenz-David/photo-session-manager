use crate::{Error, Result};
use unicode_normalization::UnicodeNormalization;

pub fn folder_key(sku: &str) -> Result<String> {
    let mut key: String = sku
        .nfc()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if key.starts_with(['.', '_']) {
        key.replace_range(..1, "-");
    }
    if key.is_empty() || key.len() > 100 {
        return Err(Error::InvalidValue(
            "sku folder key must be 1–100 characters".into(),
        ));
    }
    Ok(key)
}

/// The two ordered groups an export writes (05A MC-8); deleted and unreviewed photos never export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportGroup {
    Main,
    Condition,
}

impl ExportGroup {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Condition => "condition",
        }
    }
}

/// MC-8 export file name: `<key>_<NN>_<main|condition>.jpg`, `NN` = `position` zero-padded to
/// `max(2, digits(n_g))` where `n_g` is the number of photos in the group.
pub fn export_file_name(key: &str, position: usize, n_g: usize, group: ExportGroup) -> String {
    let width = n_g.max(1).to_string().len().max(2);
    format!("{key}_{position:0width$}_{}.jpg", group.as_str())
}

/// MC-8 `item.json`: UTF-8, keys sorted at every level, two-space indent, trailing newline; absent
/// (or empty) identity values are omitted, never written as empty strings.
pub fn item_json(
    item: &crate::domain::item::Item,
    attributes: &std::collections::BTreeMap<String, serde_json::Value>,
    main_names: &[String],
    condition_names: &[String],
) -> String {
    use serde_json::Value;
    let mut root = std::collections::BTreeMap::<&str, Value>::new();
    root.insert("schema", Value::from(1));
    root.insert("sku", Value::from(item.sku.as_str()));
    let optional = [
        ("barcode", item.barcode.as_deref()),
        ("title", Some(item.title.as_str())),
        ("item_type", Some(item.item_type.as_str())),
    ];
    for (key, value) in optional {
        if let Some(value) = value.filter(|v| !v.is_empty()) {
            root.insert(key, Value::from(value));
        }
    }
    root.insert(
        "attributes",
        Value::Object(
            attributes
                .iter()
                .map(|(k, v)| (k.clone(), sorted(v)))
                .collect(),
        ),
    );
    root.insert("main", Value::from(main_names.to_vec()));
    root.insert("condition", Value::from(condition_names.to_vec()));
    let ordered: serde_json::Map<String, Value> =
        root.into_iter().map(|(k, v)| (k.to_owned(), v)).collect();
    let mut text = serde_json::to_string_pretty(&Value::Object(ordered))
        .expect("a JSON value always serializes");
    text.push('\n');
    text
}

/// Rebuilds objects with keys inserted in sorted order, so the output is sorted whether or not
/// serde_json's map preserves insertion order.
fn sorted(value: &serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            Value::Object(
                keys.into_iter()
                    .map(|k| (k.clone(), sorted(&map[k])))
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.iter().map(sorted).collect()),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::item::{Item, ItemOrigin};

    #[test]
    fn mc8_export_file_name_pads_to_group_width() {
        assert_eq!(
            export_file_name("K", 1, 3, ExportGroup::Main),
            "K_01_main.jpg"
        );
        assert_eq!(
            export_file_name("K", 10, 10, ExportGroup::Main),
            "K_10_main.jpg"
        );
        assert_eq!(
            export_file_name("K", 7, 99, ExportGroup::Condition),
            "K_07_condition.jpg"
        );
        assert_eq!(
            export_file_name("K", 1, 100, ExportGroup::Main),
            "K_001_main.jpg"
        );
        assert_eq!(
            export_file_name("K", 100, 100, ExportGroup::Main),
            "K_100_main.jpg"
        );
        assert_eq!(
            export_file_name("K", 1, 1000, ExportGroup::Main),
            "K_0001_main.jpg"
        );
    }

    fn item(barcode: Option<&str>, title: &str) -> Item {
        Item {
            id: 1,
            sku: "K".into(),
            folder_key: "K".into(),
            barcode: barcode.map(Into::into),
            item_type: "Sideboard".into(),
            title: title.into(),
            origin: ItemOrigin::Local,
            external_id: None,
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn mc8_item_json_sorted_indented_absent_omitted() {
        let mut attributes = std::collections::BTreeMap::new();
        attributes.insert(
            "wood".to_string(),
            serde_json::json!({"z": 1, "a": ["Ö", 2]}),
        );
        let text = item_json(
            &item(Some("123"), ""),
            &attributes,
            &["K_01_main.jpg".into()],
            &[],
        );
        assert_eq!(
            text,
            "{\n  \"attributes\": {\n    \"wood\": {\n      \"a\": [\n        \"Ö\",\n        2\n      ],\n      \"z\": 1\n    }\n  },\n  \"barcode\": \"123\",\n  \"condition\": [],\n  \"item_type\": \"Sideboard\",\n  \"main\": [\n    \"K_01_main.jpg\"\n  ],\n  \"schema\": 1,\n  \"sku\": \"K\"\n}\n"
        );
    }
}
