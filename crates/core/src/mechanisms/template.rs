/// Item types are the same despite casing or whitespace at their ends.
pub fn type_key(item_type: &str) -> String {
    item_type.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::type_key;

    #[test]
    fn t1_c4a_type_key() {
        assert_eq!(type_key("Dining chair"), "dining chair");
        assert_eq!(type_key("  STOL "), "stol");
        assert_eq!(type_key("Ö-stol"), "ö-stol");
    }
}
