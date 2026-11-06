pub fn is_hidden_or_staging(name: &str) -> bool {
    name.starts_with('.') || name.to_ascii_lowercase().ends_with(".part")
}
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidName;
pub fn base_name(path: &str) -> Result<String, InvalidName> {
    if path.split('/').any(|part| part == "..") {
        return Err(InvalidName);
    }
    let name = path.rsplit('/').next().ok_or(InvalidName)?;
    if name.is_empty()
        || name == "."
        || is_hidden_or_staging(name)
        || name.contains('\\')
        || name.contains('\0')
    {
        return Err(InvalidName);
    }
    Ok(name.to_owned())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_contain_staging_and_traversal() {
        assert_eq!(base_name("sub/DSC1.ARW"), Ok("DSC1.ARW".into()));
        for name in [
            "../escape.ARW",
            "/../../escape.ARW",
            ".hidden",
            "Q.ARW.part",
        ] {
            assert!(base_name(name).is_err());
        }
    }
}
