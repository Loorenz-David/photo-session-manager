/// Match the upload name with or without Shopify's UUID uniquifier (MC-11).
pub fn matches(stem: &str, name: &str) -> bool {
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
