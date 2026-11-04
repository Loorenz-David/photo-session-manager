use crate::domain::composition::OutputSlot;

/// The upload stem of one output (07A MC-11, 07B MC-21, 10D MC-29 §Upload filename):
/// `<folder_key>-<first 12 hex of uid>` (the stored folder key as is, the hex lowercase), then
/// `-L` / `-R` for a split half, then `-r<n>` when the link's revision `n ≥ 1`. The filename is
/// the stem plus `.jpg`.
pub fn stem(folder_key: &str, uid: &str, slot: OutputSlot, revision: i64) -> String {
    let hex: String = uid
        .chars()
        .filter(|c| *c != '-')
        .take(12)
        .collect::<String>()
        .to_lowercase();
    let half = match slot {
        OutputSlot::Single => "",
        OutputSlot::Left => "-L",
        OutputSlot::Right => "-R",
    };
    let suffix = if revision >= 1 {
        format!("-r{revision}")
    } else {
        String::new()
    };
    format!("{folder_key}-{hex}{half}{suffix}")
}

/// Match the upload name with or without Shopify's UUID uniquifier (MC-11), ignoring case (names
/// uploaded before 10D were lowercased by the adapter; folder keys are unique ignoring case).
pub fn matches(stem: &str, name: &str) -> bool {
    let (stem, name) = (stem.to_lowercase(), name.to_lowercase());
    if name == format!("{stem}.jpg") {
        return true;
    }
    let Some(suffix) = name
        .strip_prefix(&format!("{stem}_"))
        .and_then(|s| s.strip_suffix(".jpg"))
    else {
        return false;
    };
    uuid::Uuid::parse_str(suffix).is_ok_and(|id| id.get_version_num() == 4)
}
