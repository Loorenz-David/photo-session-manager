//! The product extract of a listing page (R12·U2; the contract is the R12 plan, Unit 2): what
//! the read model is given as the page's text instead of the whole visible text. Parts in this
//! order, each labelled; an empty part is left out, and a text equal to an earlier one after
//! collapsing white space is left out:
//!
//! 1. **Title:** `og:title`, else the `Product` block's name, else `<title>`.
//! 2. **Description:** the `Product` block's description; `og:description`; the `description`
//!    meta.
//! 3. **Stated:** from the `Product` block only, `creator`, `manufacturer`, `material` and
//!    `productionDate`, each as `<property>: <text>` (a string, or the `name` of an object).
//!    `brand` is never listed: on the recorded pages it names the seller.
//! 4. **Page text:** the visible text from the first place the text of the document's first
//!    `<h1>` occurs in it (else from its start), at most [`PRODUCT_TEXT_CHARS`] characters.
//!
//! The whole extract is cut at [`MAX_TEXT_CHARS`]. Nothing is decided from the page here.

use serde_json::Value as Json;

use super::{Collected, MAX_TEXT_CHARS, PRODUCT_TEXT_CHARS, collapse, collect, title_of};

/// The `Product` properties the stated part lists, in this order.
pub const STATED_PROPERTIES: [&str; 4] = ["creator", "manufacturer", "material", "productionDate"];

/// Labels of the parts.
pub const TITLE_LABEL: &str = "Title: ";
pub const DESCRIPTION_LABEL: &str = "Description: ";
pub const STATED_LABEL: &str = "Stated:";
pub const PAGE_TEXT_LABEL: &str = "Page text:";

/// The product extract of an HTML document.
pub fn product_extract(html: &str) -> String {
    let (c, product) = collect(html);
    let title = title_of(&c, product.as_ref());
    from_parts(&c, product.as_ref(), &title)
}

/// A property's text: a string, a number, the `name` of an object, or a list of those.
fn property_text(value: &Json) -> Option<String> {
    let text = match value {
        Json::String(s) => collapse(s),
        Json::Number(n) => n.to_string(),
        Json::Object(map) => map.get("name").and_then(property_text)?,
        Json::Array(items) => items
            .iter()
            .filter_map(property_text)
            .collect::<Vec<_>>()
            .join(", "),
        _ => return None,
    };
    (!text.is_empty()).then_some(text)
}

/// The visible text from the first `<h1>`'s text on, at most `PRODUCT_TEXT_CHARS` characters.
fn page_text(c: &Collected) -> String {
    let visible = collapse(&c.text);
    let heading = c.h1.as_deref().map(collapse).unwrap_or_default();
    let start = if heading.is_empty() {
        0
    } else {
        visible.find(&heading).unwrap_or(0)
    };
    visible[start..].chars().take(PRODUCT_TEXT_CHARS).collect()
}

pub(super) fn from_parts(c: &Collected, product: Option<&Json>, title: &str) -> String {
    // Every text kept so far, white space collapsed, for the "equal to an earlier part" rule.
    let mut seen: Vec<String> = Vec::new();
    let mut keep = |text: &str| -> Option<String> {
        let collapsed = collapse(text);
        if collapsed.is_empty() || seen.contains(&collapsed) {
            return None;
        }
        seen.push(collapsed.clone());
        Some(collapsed)
    };
    let mut parts: Vec<String> = Vec::new();
    if let Some(title) = keep(title) {
        parts.push(format!("{TITLE_LABEL}{title}"));
    }
    let descriptions: Vec<String> = [
        product.and_then(|p| p["description"].as_str()),
        c.og_description.as_deref(),
        c.meta_description.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter_map(&mut keep)
    .map(|d| format!("{DESCRIPTION_LABEL}{d}"))
    .collect();
    if !descriptions.is_empty() {
        parts.push(descriptions.join("\n"));
    }
    let stated: Vec<String> = STATED_PROPERTIES
        .iter()
        .filter_map(|property| {
            let text = property_text(product?.get(*property)?)?;
            keep(&format!("{property}: {text}"))
        })
        .collect();
    if !stated.is_empty() {
        parts.push(format!("{STATED_LABEL}\n{}", stated.join("\n")));
    }
    // The page text is already collapsed; it is kept exactly as cut (no second trim).
    let text = page_text(c);
    if keep(&text).is_some() {
        parts.push(format!("{PAGE_TEXT_LABEL}\n{text}"));
    }
    parts.join("\n\n").chars().take(MAX_TEXT_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The part after `label` up to the next blank line (the page text runs to the end).
    fn part<'a>(extract: &'a str, label: &str) -> Option<&'a str> {
        let at = extract
            .split("\n\n")
            .find(|block| block.starts_with(label))?;
        Some(at.strip_prefix(label)?.trim_start_matches('\n'))
    }

    fn page_part(extract: &str) -> &str {
        let at = extract
            .find(&format!("{PAGE_TEXT_LABEL}\n"))
            .expect("a page-text part");
        &extract[at + PAGE_TEXT_LABEL.len() + 1..]
    }

    fn descriptions(extract: &str) -> Vec<&str> {
        extract
            .lines()
            .filter_map(|l| l.strip_prefix(DESCRIPTION_LABEL))
            .collect()
    }

    /// `word` repeated, separated by single spaces, exactly `n` characters.
    fn words(n: usize, word: &str) -> String {
        let mut out = String::new();
        while out.len() < n {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(word);
        }
        out.truncate(n);
        // A cut inside a word is fine; a trailing space is not (collapse would drop it).
        if out.ends_with(' ') {
            out.pop();
            out.push('z');
        }
        out
    }

    #[test]
    fn a26_c8a_the_committed_shop_page() {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/pages/shop_listing_page.json");
        let capture: Json = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
        let body = capture["response"]["body"].as_str().unwrap();
        let extract = product_extract(body);
        assert_eq!(
            extract.lines().next(),
            Some("Title: Swedish bookshelf in oak")
        );
        assert!(
            descriptions(&extract)
                .iter()
                .any(|d| d.contains("Bookshelf manufactured in Sweden in the 1950s-60s.")),
            "{extract}"
        );
        assert!(
            part(&extract, STATED_LABEL).is_none_or(|stated| !stated.contains("brand")),
            "{extract}"
        );
        let page = page_part(&extract);
        assert!(page.starts_with("Swedish bookshelf in oak"), "{page}");
        assert!(page.chars().count() <= PRODUCT_TEXT_CHARS);
    }

    #[test]
    fn a26_c8b_page_text_starts_at_the_heading() {
        let heading = "Swedish bookshelf in oak";
        let prefix = words(7_799, "menu");
        let rest = words(20_000 - 7_799 - 1 - heading.len() - 1, "listing");
        let visible = format!("{prefix} {heading} {rest}");
        assert_eq!(visible.len(), 20_000);
        assert_eq!(visible.find(heading), Some(7_800));
        let html =
            format!("<html><body><p>{prefix}</p><h1>{heading}</h1><p>{rest}</p></body></html>");
        let extract = product_extract(&html);
        assert_eq!(page_part(&extract), &visible[7_800..17_800]);
    }

    #[test]
    fn a26_c8c_no_heading_takes_the_first_10000_characters() {
        let visible = words(12_000, "auction");
        let html = format!("<html><body><div>{visible}</div></body></html>");
        let extract = product_extract(&html);
        assert_eq!(page_part(&extract), &visible[..10_000]);
    }

    #[test]
    fn a26_c8d_a_first_heading_not_in_the_visible_text_starts_at_the_start() {
        let html = "<html><body><template><h1>Hidden heading</h1></template>\
                    <nav>Menu Shop All products</nav><h1>Real heading</h1>\
                    <p>The listing text.</p></body></html>";
        let extract = product_extract(html);
        assert_eq!(
            page_part(&extract),
            "Menu Shop All products Real heading The listing text."
        );
    }

    #[test]
    fn a26_c8e_stated_lists_creator_manufacturer_material_never_brand() {
        let ld = serde_json::json!({
            "@context": "https://schema.org",
            "@type": "Product",
            "name": "Armchair",
            "brand": {"@type": "Brand", "name": "Bukowskis"},
            "material": "Teak",
            "creator": {"@type": "Person", "name": "Niels Otto Møller"},
            "manufacturer": "J.L. Møllers Møbelfabrik",
        });
        let html = format!(
            "<html><head><script type=\"application/ld+json\">{ld}</script></head>\
             <body><p>Lot 12</p></body></html>"
        );
        let extract = product_extract(&html);
        assert_eq!(
            part(&extract, STATED_LABEL),
            Some(
                "creator: Niels Otto Møller\nmanufacturer: J.L. Møllers Møbelfabrik\nmaterial: Teak"
            ),
            "{extract}"
        );
        assert!(!extract.contains("Bukowskis"), "{extract}");
    }

    #[test]
    fn a26_c8f_a_description_equal_but_for_white_space_is_kept_once() {
        let ld = serde_json::json!({
            "@type": "Product",
            "name": "Sideboard",
            "description": "A teak sideboard\n  made in Denmark.",
        });
        let html = format!(
            "<html><head><script type=\"application/ld+json\">{ld}</script>\
             <meta property=\"og:description\" content=\"A teak   sideboard made in Denmark.\">\
             <meta name=\"description\" content=\"Buy vintage furniture online.\">\
             </head><body><p>Sideboard</p></body></html>"
        );
        let extract = product_extract(&html);
        assert_eq!(
            descriptions(&extract),
            vec![
                "A teak sideboard made in Denmark.",
                "Buy vintage furniture online."
            ]
        );
    }

    #[test]
    fn a26_c8g_a_product_block_that_is_not_json_is_ignored() {
        let html = "<html><head>\
             <script type=\"application/ld+json\">{\"@type\": \"Product\", \"name\": \"X\",}</script>\
             <meta property=\"og:title\" content=\"Dealer chair\"></head>\
             <body><p>Chair in beech.</p></body></html>";
        let extract = product_extract(html);
        assert_eq!(
            extract,
            "Title: Dealer chair\n\nPage text:\nChair in beech."
        );
    }

    #[test]
    fn a26_c8h_the_whole_extract_is_cut_at_30000_characters() {
        let description = words(40_000, "veneer");
        let ld =
            serde_json::json!({"@type": "Product", "name": "Desk", "description": description});
        let html = format!(
            "<html><head><script type=\"application/ld+json\">{ld}</script></head>\
             <body><p>Desk</p></body></html>"
        );
        assert_eq!(product_extract(&html).chars().count(), 30_000);
    }
}
