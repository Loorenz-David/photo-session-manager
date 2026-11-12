//! MC-A2: a versioned, Unicode-aware key for comparing names.

use unicode_casefold::UnicodeCaseFold;
use unicode_general_category::get_general_category;
use unicode_normalization::UnicodeNormalization;

pub const RESEARCH_NORMALISE_VERSION: u32 = 1;

pub fn name_key(input: &str) -> String {
    let folded: String = input.nfkc().collect::<String>().case_fold().collect();
    let mut key = String::new();
    let mut space = false;
    for ch in folded.chars() {
        let category = get_general_category(ch).abbreviation();
        if ch.is_whitespace() || category.starts_with('P') || category.starts_with('S') {
            space = !key.is_empty();
        } else {
            if space {
                key.push(' ');
                space = false;
            }
            key.push(ch);
        }
    }
    key
}

#[cfg(test)]
mod tests {
    use super::name_key;

    #[test]
    fn a03_c1a_punctuation() {
        assert_eq!(name_key("Hans J. Wegner"), "hans j wegner");
    }
    #[test]
    fn a03_c1b_diacritics_and_spaces() {
        assert_eq!(name_key("  Børge   Mogensen "), "børge mogensen");
        assert_ne!(name_key("Børge Mogensen"), name_key("Borge Mogensen"));
    }
    #[test]
    fn a03_c1c_nfkc() {
        assert_eq!(name_key("Ｗｅｇｎｅｒ"), "wegner");
        assert_eq!(name_key("ﬁnn"), "finn");
    }
    #[test]
    fn a03_c1d_full_fold() {
        assert_eq!(name_key("STRASSE"), name_key("Straße"));
    }
    #[test]
    fn a03_c1e_symbols() {
        assert_eq!(name_key("Fritz-Hansen & Co."), "fritz hansen co");
        assert_eq!(name_key("A/S"), "a s");
    }
    #[test]
    fn a03_c1f_no_aliases() {
        assert_ne!(name_key("Hans J. Wegner"), name_key("Hans Wegner"));
    }
    #[test]
    fn a03_c1g_empty() {
        assert_eq!(name_key("..."), "");
        assert_eq!(name_key(""), "");
    }
}
