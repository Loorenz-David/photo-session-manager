//! MC-A3: hosts, trusted entries, the site of a page, page identity, the item's own listing and
//! reading order.

use std::net::IpAddr;

use url::{Host, Url};

use crate::domain::SiteClass;

/// The parsed URL when it is an absolute `http`/`https` URL with a host.
fn web_url(text: &str) -> Option<Url> {
    let url = Url::parse(text.trim()).ok()?;
    matches!(url.scheme(), "http" | "https").then_some(())?;
    url.host()?;
    Some(url)
}

fn host_of(url: &Url) -> Option<String> {
    let host = match url.host()? {
        Host::Domain(domain) => domain.to_ascii_lowercase(),
        Host::Ipv4(ip) => ip.to_string(),
        Host::Ipv6(ip) => format!("[{ip}]"),
    };
    let host = host.strip_suffix('.').unwrap_or(&host).to_owned();
    (!host.is_empty()).then_some(host)
}

/// `H(url)`: lowercase, IDNA to ASCII, trailing dot stripped, port ignored; `None` unless the URL
/// is absolute `http`/`https`.
pub fn host(url: &str) -> Option<String> {
    host_of(&web_url(url)?)
}

fn is_ip_literal(host: &str) -> bool {
    host.starts_with('[') || host.parse::<IpAddr>().is_ok()
}

fn without_www(host: &str) -> &str {
    host.strip_prefix("www.").unwrap_or(host)
}

/// `h` is `e` or one of its subdomains.
fn covers(entry: &str, host: &str) -> bool {
    host == entry
        || host
            .strip_suffix(entry)
            .is_some_and(|rest| rest.ends_with('.'))
}

pub fn overlaps(a: &str, b: &str) -> bool {
    covers(a, b) || covers(b, a)
}

/// The ordered, disjoint list of trusted entries (`research.trusted_sites`); priority is the index.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrustedSites {
    entries: Vec<String>,
}

impl TrustedSites {
    /// Entries already in their stored form, in priority order.
    pub fn new(entries: Vec<String>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    /// The stored form of a typed domain or URL: `H` of it with one leading `www.` removed. It must
    /// contain a dot and must not be an IP address.
    pub fn parse_entry(text: &str) -> Result<String, String> {
        let text = text.trim();
        let host = if text.contains("://") {
            host(text)
        } else {
            host(&format!("https://{text}"))
        };
        let Some(host) = host else {
            return Err("not a web address".to_owned());
        };
        if is_ip_literal(&host) {
            return Err("an IP address cannot be a trusted site".to_owned());
        }
        let entry = without_www(&host).to_owned();
        if !entry.contains('.') {
            return Err("a site needs a dot in its name".to_owned());
        }
        Ok(entry)
    }

    /// Appends an entry (lowest priority). Refused when it equals, is a parent of, or is a
    /// subdomain of an existing entry or a shop host.
    pub fn add(&mut self, text: &str, shop_hosts: &[String]) -> Result<String, String> {
        let entry = Self::parse_entry(text)?;
        let taken = self
            .entries
            .iter()
            .chain(shop_hosts.iter())
            .find(|other| covers(other, &entry) || covers(&entry, other));
        if let Some(other) = taken {
            return Err(format!("already covered by {other}"));
        }
        self.entries.push(entry.clone());
        Ok(entry)
    }

    /// Removes the entry a typed domain or URL names; false when it is not in the list.
    pub fn remove(&mut self, text: &str) -> bool {
        let Ok(entry) = Self::parse_entry(text) else {
            return false;
        };
        let before = self.entries.len();
        self.entries.retain(|e| *e != entry);
        self.entries.len() != before
    }

    /// Replaces the order; the new order must hold exactly the current entries.
    pub fn reorder(&mut self, order: &[String]) -> Result<(), String> {
        let mut current = self.entries.clone();
        let mut proposed = order.to_vec();
        current.sort();
        proposed.sort();
        if current != proposed {
            return Err("the new order must list exactly the current sites".to_owned());
        }
        self.entries = order.to_vec();
        Ok(())
    }

    /// A host matches an entry iff `h = e` or `h` ends with `"." + e`.
    pub fn matches(host: &str, entry: &str) -> bool {
        covers(entry, host)
    }

    /// The priority of an entry (0 first).
    pub fn priority(&self, entry: &str) -> Option<usize> {
        self.entries.iter().position(|e| e == entry)
    }

    /// The entry a host belongs to, with its priority. IP literals never match.
    pub fn entry_for(&self, host: &str) -> Option<(usize, &str)> {
        if is_ip_literal(host) {
            return None;
        }
        self.entries
            .iter()
            .enumerate()
            .find(|(_, e)| Self::matches(host, e))
            .map(|(i, e)| (i, e.as_str()))
    }
}

/// The site a page belongs to: its class and identity (MC-A3 "Site of a page").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Site {
    pub class: SiteClass,
    pub identity: String,
}

/// First match of shop, trusted, untrusted; `None` when the URL has no valid host.
pub fn site_of(url: &str, shop_hosts: &[String], trusted: &TrustedSites) -> Option<Site> {
    let host = host(url)?;
    if shop_hosts.iter().any(|shop| covers(shop, &host)) {
        return Some(Site {
            class: SiteClass::Shop,
            identity: "shop".to_owned(),
        });
    }
    if let Some((_, entry)) = trusted.entry_for(&host) {
        return Some(Site {
            class: SiteClass::Trusted,
            identity: entry.to_owned(),
        });
    }
    Some(Site {
        class: SiteClass::Untrusted,
        identity: without_www(&host).to_owned(),
    })
}

fn tracking(name: &str) -> bool {
    name.starts_with("utm_") || matches!(name, "srsltid" | "gclid" | "fbclid" | "mc_cid" | "mc_eid")
}

/// `U(url)`: scheme lowercased, host `H`, default port dropped, empty path `/`, fragment dropped,
/// tracking parameters dropped and the rest sorted by name, then value.
pub fn page_identity(url: &str) -> Option<String> {
    let parsed = web_url(url)?;
    let host = host_of(&parsed)?;
    let mut out = format!("{}://{host}", parsed.scheme());
    if let Some(port) = parsed.port() {
        out.push_str(&format!(":{port}"));
    }
    let path = parsed.path();
    out.push_str(if path.is_empty() { "/" } else { path });
    let mut params: Vec<(&str, &str)> = parsed
        .query()
        .unwrap_or("")
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|p| p.split_once('=').unwrap_or((p, "")))
        .filter(|(name, _)| !tracking(name))
        .collect();
    params.sort();
    if !params.is_empty() {
        let query: Vec<String> = params
            .iter()
            .map(|(name, value)| {
                if value.is_empty() {
                    (*name).to_owned()
                } else {
                    format!("{name}={value}")
                }
            })
            .collect();
        out.push('?');
        out.push_str(&query.join("&"));
    }
    Some(out)
}

/// A locale path segment as shops put it before their paths: `en`, `fr`, `sv-se`, `en_GB`.
fn is_locale(segment: &str) -> bool {
    let bytes = segment.as_bytes();
    let letters = |part: &[u8]| part.len() == 2 && part.iter().all(u8::is_ascii_alphabetic);
    match bytes.len() {
        2 => letters(bytes),
        5 => letters(&bytes[..2]) && matches!(bytes[2], b'-' | b'_') && letters(&bytes[3..]),
        _ => false,
    }
}

/// The path segments of a web URL with one leading locale segment dropped.
fn segments_without_locale(url: &Url) -> Vec<String> {
    let mut segments: Vec<String> = url
        .path_segments()
        .map(|s| s.filter(|seg| !seg.is_empty()).map(str::to_owned).collect())
        .unwrap_or_default();
    if segments.len() > 1 && is_locale(&segments[0]) {
        segments.remove(0);
    }
    segments
}

/// The product handle of a shop's product page: the segment after `products`
/// (`/products/<h>`, `/<locale>/products/<h>`, `/collections/<c>/products/<h>`); `None` for any
/// other page (owner correction A9, 2026-11-18).
pub fn product_handle(url: &str) -> Option<String> {
    let parsed = web_url(url)?;
    let segments = segments_without_locale(&parsed);
    let at = segments.iter().position(|s| s == "products")?;
    let handle = segments.get(at + 1)?.trim();
    (!handle.is_empty()).then(|| handle.to_owned())
}

/// One product however a shop spells its page: `scheme://host/products/<handle>` for a product
/// page, so its locale variants and collection paths are read once; `None` for any other page.
pub fn product_identity(url: &str) -> Option<String> {
    let parsed = web_url(url)?;
    let handle = product_handle(url)?;
    Some(format!(
        "{}://{}/products/{handle}",
        parsed.scheme(),
        host_of(&parsed)?
    ))
}

/// A page that lists products rather than describing one: a shop collection without a product
/// (`/collections/<c>`), or a category path (`/category/…`, `/categories/…`,
/// `/product-category/…`). Such pages are not read (owner correction A9, 2026-11-18).
pub fn is_listing_page(url: &str) -> bool {
    let Some(parsed) = web_url(url) else {
        return false;
    };
    let segments = segments_without_locale(&parsed);
    if segments.iter().any(|s| s == "products") {
        return false;
    }
    matches!(
        segments.first().map(String::as_str),
        Some("collections" | "category" | "categories" | "product-category")
    )
}

/// The item's own storefront listing, as the shop reports it. Empty when the item has none.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OwnListing {
    pub url: Option<String>,
    pub path_tail: Vec<String>,
}

/// True when the page is the item's own listing: the same `U` as its storefront URL, or a page on
/// a shop host whose path ends with the listing's path segments.
pub fn is_own_listing(url: &str, own: &OwnListing, shop_hosts: &[String]) -> bool {
    let Some(identity) = page_identity(url) else {
        return false;
    };
    if own
        .url
        .as_deref()
        .and_then(page_identity)
        .is_some_and(|own_identity| own_identity == identity)
    {
        return true;
    }
    if own.path_tail.is_empty() {
        return false;
    }
    let Some(parsed) = web_url(url) else {
        return false;
    };
    let on_shop = host_of(&parsed).is_some_and(|h| shop_hosts.iter().any(|s| covers(s, &h)));
    let segments: Vec<&str> = parsed
        .path_segments()
        .map(|s| s.filter(|seg| !seg.is_empty()).collect())
        .unwrap_or_default();
    on_shop && segments.ends_with(&own.path_tail.iter().map(String::as_str).collect::<Vec<_>>())
}

/// One search result to consider reading: its URL and the provider's rank (1 best).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub url: String,
    pub rank: u32,
}

/// A page to read, in reading order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub url: String,
    pub identity: String,
    pub rank: u32,
    pub site: Site,
}

/// The reading order, plus what was set apart.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadingOrder {
    pub pages: Vec<Page>,
    /// Candidates that are the item's own listing ("this item's own listing" in the trace).
    pub own_listing: Vec<Candidate>,
    /// Candidates without a valid absolute `http`/`https` URL.
    pub discarded: Vec<Candidate>,
    /// Candidates that list products instead of describing one ([`is_listing_page`]): not read.
    pub listings: Vec<Candidate>,
}

/// MC-A3 reading order: shop by rank; trusted by entry priority, then rank; untrusted by rank;
/// ties by `U` ascending. Equal `U` merge, keeping the best rank, and so do the spellings of one
/// product page ([`product_identity`]: locale variants, collection paths). Listing pages are set
/// apart, not read (owner correction A9, 2026-11-18).
pub fn reading_order(
    candidates: &[Candidate],
    shop_hosts: &[String],
    trusted: &TrustedSites,
    own: &OwnListing,
) -> ReadingOrder {
    let mut order = ReadingOrder::default();
    for candidate in candidates {
        let (Some(identity), Some(site)) = (
            page_identity(&candidate.url),
            site_of(&candidate.url, shop_hosts, trusted),
        ) else {
            order.discarded.push(candidate.clone());
            continue;
        };
        if is_own_listing(&candidate.url, own, shop_hosts) {
            order.own_listing.push(candidate.clone());
            continue;
        }
        if is_listing_page(&candidate.url) {
            order.listings.push(candidate.clone());
            continue;
        }
        let same = product_identity(&candidate.url).unwrap_or_else(|| identity.clone());
        match order
            .pages
            .iter_mut()
            .find(|p| product_identity(&p.url).unwrap_or_else(|| p.identity.clone()) == same)
        {
            Some(page) if page.rank > candidate.rank => {
                page.rank = candidate.rank;
                page.url = candidate.url.clone();
                page.identity = identity;
            }
            Some(_) => {}
            None => order.pages.push(Page {
                url: candidate.url.clone(),
                identity,
                rank: candidate.rank,
                site,
            }),
        }
    }
    let sort_key = |page: &Page| {
        let (class, priority) = match page.site.class {
            SiteClass::Shop => (0, 0),
            SiteClass::Trusted => (
                1,
                trusted.priority(&page.site.identity).unwrap_or(usize::MAX),
            ),
            SiteClass::Untrusted => (2, 0),
        };
        (class, priority, page.rank, page.identity.clone())
    };
    order.pages.sort_by_key(sort_key);
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(items: &[&str]) -> Vec<String> {
        items.iter().map(|i| i.to_string()).collect()
    }

    fn c(url: &str, rank: u32) -> Candidate {
        Candidate {
            url: url.to_owned(),
            rank,
        }
    }

    #[test]
    fn a04_c1a_host_lowercase_port_ignored() {
        assert_eq!(
            host("HTTPS://WWW.Example.COM:8443/x").as_deref(),
            Some("www.example.com")
        );
    }

    #[test]
    fn a04_c1b_host_trailing_dot() {
        assert_eq!(
            host("https://example.com./a").as_deref(),
            Some("example.com")
        );
    }

    #[test]
    fn a04_c1c_host_idna() {
        assert_eq!(
            host("https://bücher.example/").as_deref(),
            Some("xn--bcher-kva.example")
        );
    }

    #[test]
    fn a04_c1d_host_not_web() {
        assert_eq!(host("ftp://example.com/"), None);
        assert_eq!(host("example.com/x"), None);
        assert_eq!(host("mailto:a@example.com"), None);
    }

    #[test]
    fn a04_c2a_parse_entry() {
        assert_eq!(
            TrustedSites::parse_entry("https://www.Pamono.com/x").as_deref(),
            Ok("pamono.com")
        );
        assert_eq!(
            TrustedSites::parse_entry("auktion.sweakronan.se").as_deref(),
            Ok("auktion.sweakronan.se")
        );
        assert!(TrustedSites::parse_entry("localhost").is_err());
    }

    #[test]
    fn a04_c2b_add_refuses_subdomain_and_equal() {
        let mut sites = TrustedSites::new(s(&["etsy.com"]));
        assert_eq!(
            sites.add("shop.etsy.com", &[]),
            Err("already covered by etsy.com".to_owned())
        );
        assert_eq!(
            sites.add("etsy.com", &[]),
            Err("already covered by etsy.com".to_owned())
        );
        assert_eq!(sites.entries(), s(&["etsy.com"]).as_slice());
    }

    #[test]
    fn a04_c2c_add_refuses_parent() {
        let mut sites = TrustedSites::new(s(&["shop.etsy.com"]));
        assert_eq!(
            sites.add("etsy.com", &[]),
            Err("already covered by shop.etsy.com".to_owned())
        );
    }

    #[test]
    fn a04_c2d_add_refuses_shop_host_and_no_dot() {
        let shop = s(&["beyo.example"]);
        let mut sites = TrustedSites::default();
        assert_eq!(
            sites.add("beyo.example", &shop),
            Err("already covered by beyo.example".to_owned())
        );
        assert_eq!(
            sites.add("www.beyo.example", &shop),
            Err("already covered by beyo.example".to_owned())
        );
        let refused = sites.add("example", &shop);
        assert!(refused.is_err());
        assert!(!refused.unwrap_err().starts_with("already covered"));
        assert!(sites.entries().is_empty());
    }

    #[test]
    fn a04_c2e_reorder_and_remove() {
        let mut sites = TrustedSites::new(s(&["a.example", "b.example", "c.example"]));
        sites
            .reorder(&s(&["c.example", "a.example", "b.example"]))
            .unwrap();
        assert!(sites.remove("a.example"));
        assert_eq!(sites.entries(), s(&["c.example", "b.example"]).as_slice());
        assert_eq!(sites.priority("c.example"), Some(0));
        assert_eq!(sites.priority("b.example"), Some(1));
    }

    fn fixture() -> (Vec<String>, TrustedSites) {
        (
            s(&["beyo.example"]),
            TrustedSites::new(s(&["etsy.com", "auktion.sweakronan.se"])),
        )
    }

    fn site(class: SiteClass, identity: &str) -> Option<Site> {
        Some(Site {
            class,
            identity: identity.to_owned(),
        })
    }

    #[test]
    fn a04_c3a_shop_and_its_subdomain() {
        let (shop, trusted) = fixture();
        assert_eq!(
            site_of("https://beyo.example/p", &shop, &trusted),
            site(SiteClass::Shop, "shop")
        );
        assert_eq!(
            site_of("https://en.beyo.example/p", &shop, &trusted),
            site(SiteClass::Shop, "shop")
        );
    }

    #[test]
    fn a04_c3b_trusted_entry_and_subdomain() {
        let (shop, trusted) = fixture();
        assert_eq!(
            site_of("https://www.etsy.com/l/1", &shop, &trusted),
            site(SiteClass::Trusted, "etsy.com")
        );
        assert_eq!(
            site_of("https://shop.etsy.com/l/1", &shop, &trusted),
            site(SiteClass::Trusted, "etsy.com")
        );
    }

    #[test]
    fn a04_c3c_suffix_without_dot_is_untrusted() {
        let (shop, trusted) = fixture();
        assert_eq!(
            site_of("https://notetsy.com/x", &shop, &trusted),
            site(SiteClass::Untrusted, "notetsy.com")
        );
        assert_eq!(
            site_of("https://sweakronan.se/x", &shop, &trusted),
            site(SiteClass::Untrusted, "sweakronan.se")
        );
    }

    #[test]
    fn a04_c3d_untrusted_www_and_ip_never_trusted() {
        let (shop, trusted) = fixture();
        assert_eq!(
            site_of("https://www.dealer.example/x", &shop, &trusted),
            site(SiteClass::Untrusted, "dealer.example")
        );
        assert_eq!(
            site_of("http://192.168.1.5/x", &shop, &trusted),
            site(SiteClass::Untrusted, "192.168.1.5")
        );
        let mut with_ip = trusted.clone();
        assert!(with_ip.add("192.168.1.5", &shop).is_err());
        assert_eq!(
            site_of("http://192.168.1.5/x", &shop, &with_ip),
            site(SiteClass::Untrusted, "192.168.1.5")
        );
    }

    #[test]
    fn a04_c4a_page_identity_normalised() {
        assert_eq!(
            page_identity("HTTP://WWW.Example.com:80/a?utm_source=x&b=2&a=1#frag").as_deref(),
            Some("http://www.example.com/a?a=1&b=2")
        );
    }

    #[test]
    fn a04_c4b_default_port_and_empty_path() {
        let a = page_identity("https://example.com");
        let b = page_identity("https://example.com:443/");
        assert_eq!(a.as_deref(), Some("https://example.com/"));
        assert_eq!(a, b);
        assert_eq!(
            page_identity("https://example.com:8443/").as_deref(),
            Some("https://example.com:8443/")
        );
    }

    #[test]
    fn a04_c4c_tracking_parameters_dropped() {
        assert_eq!(
            page_identity(
                "https://example.com/p?utm_campaign=a&srsltid=b&gclid=c&id=7&fbclid=d&mc_cid=e&mc_eid=f"
            )
            .as_deref(),
            Some("https://example.com/p?id=7")
        );
    }

    #[test]
    fn a04_c4d_query_sorted_by_name_then_value() {
        assert_eq!(
            page_identity("https://example.com/p?b=1&a=2&a=1").as_deref(),
            Some("https://example.com/p?a=1&a=2&b=1")
        );
    }

    fn own() -> OwnListing {
        OwnListing {
            url: Some("https://beyo.example/products/oak-table".to_owned()),
            path_tail: s(&["products", "oak-table"]),
        }
    }

    #[test]
    fn a04_c5a_own_listing_by_url_and_by_path() {
        let shop = s(&["beyo.example"]);
        assert!(is_own_listing(
            "https://beyo.example/products/oak-table?utm_source=x",
            &own(),
            &shop
        ));
        assert!(is_own_listing(
            "https://beyo.example/en/products/oak-table",
            &own(),
            &shop
        ));
    }

    #[test]
    fn a04_c5b_other_handle_or_other_host_not_own() {
        let shop = s(&["beyo.example"]);
        assert!(!is_own_listing(
            "https://beyo.example/products/oak-table-2",
            &own(),
            &shop
        ));
        assert!(!is_own_listing(
            "https://etsy.com/products/oak-table",
            &own(),
            &shop
        ));
    }

    #[test]
    fn a04_c5c_no_listing_data_never_own() {
        let shop = s(&["beyo.example"]);
        let none = OwnListing::default();
        assert!(!is_own_listing(
            "https://beyo.example/products/oak-table",
            &none,
            &shop
        ));
        assert!(!is_own_listing("https://beyo.example/", &none, &shop));
    }

    fn order_fixture() -> (Vec<String>, TrustedSites) {
        (
            s(&["beyo.example"]),
            TrustedSites::new(s(&["b.example", "a.example"])),
        )
    }

    fn urls(order: &ReadingOrder) -> Vec<&str> {
        order.pages.iter().map(|p| p.url.as_str()).collect()
    }

    #[test]
    fn a04_c6a_shop_trusted_by_priority_untrusted() {
        let (shop, trusted) = order_fixture();
        let order = reading_order(
            &[
                c("https://dealer.example/1", 1),
                c("https://a.example/2", 2),
                c("https://b.example/3", 3),
                c("https://beyo.example/4", 4),
            ],
            &shop,
            &trusted,
            &OwnListing::default(),
        );
        assert_eq!(
            urls(&order),
            vec![
                "https://beyo.example/4",
                "https://b.example/3",
                "https://a.example/2",
                "https://dealer.example/1"
            ]
        );
    }

    #[test]
    fn a04_c6b_same_entry_by_rank() {
        let (shop, trusted) = order_fixture();
        let order = reading_order(
            &[
                c("https://a.example/five", 5),
                c("https://a.example/two", 2),
            ],
            &shop,
            &trusted,
            &OwnListing::default(),
        );
        assert_eq!(
            urls(&order),
            vec!["https://a.example/two", "https://a.example/five"]
        );
    }

    #[test]
    fn a04_c6c_equal_rank_by_identity() {
        let (shop, trusted) = order_fixture();
        let order = reading_order(
            &[
                c("https://zeta.example/x", 3),
                c("https://alpha.example/x", 3),
            ],
            &shop,
            &trusted,
            &OwnListing::default(),
        );
        assert_eq!(
            urls(&order),
            vec!["https://alpha.example/x", "https://zeta.example/x"]
        );
    }

    #[test]
    fn a04_c6d_merge_own_listing_and_invalid() {
        let (shop, trusted) = order_fixture();
        let own = OwnListing {
            url: Some("https://beyo.example/products/oak-table".to_owned()),
            path_tail: s(&["products", "oak-table"]),
        };
        let order = reading_order(
            &[
                c("https://dealer.example/p?utm_source=x", 4),
                c("https://dealer.example/p", 1),
                c("https://beyo.example/products/oak-table", 1),
                c("not a url", 2),
            ],
            &shop,
            &trusted,
            &own,
        );
        assert_eq!(order.pages.len(), 1);
        assert_eq!(order.pages[0].identity, "https://dealer.example/p");
        assert_eq!(order.pages[0].rank, 1);
        assert_eq!(
            order.own_listing,
            vec![c("https://beyo.example/products/oak-table", 1)]
        );
        assert_eq!(order.discarded, vec![c("not a url", 2)]);
    }
    #[test]
    fn a9_locale_and_collection_spellings_of_one_product_are_read_once() {
        let shop = s(&["shop.example"]);
        let order = reading_order(
            &[
                c("https://shop.example/en/products/teak-table-4", 2),
                c("https://shop.example/fr/products/teak-table-4", 1),
                c(
                    "https://shop.example/collections/tables/products/teak-table-4",
                    3,
                ),
                c("https://other.example/en/products/teak-table-4", 4),
            ],
            &shop,
            &TrustedSites::default(),
            &OwnListing::default(),
        );
        let urls: Vec<&str> = order.pages.iter().map(|p| p.url.as_str()).collect();
        assert_eq!(
            urls,
            [
                "https://shop.example/fr/products/teak-table-4",
                "https://other.example/en/products/teak-table-4"
            ]
        );
        assert_eq!(
            order.pages[0].identity,
            "https://shop.example/fr/products/teak-table-4"
        );
    }

    #[test]
    fn a9_listing_pages_are_set_apart() {
        for listing in [
            "https://shop.example/en/collections/sideboards",
            "https://shop.example/collections",
            "https://dealer.example/product-category/chairs/",
            "https://dealer.example/sv-se/category/teak",
        ] {
            assert!(is_listing_page(listing), "{listing}");
        }
        for page in [
            "https://shop.example/collections/all-products/products/teak-table",
            "https://shop.example/en/products/teak-table",
            "https://auction.example/en/search?q=teak",
            "https://dealer.example/item/123",
        ] {
            assert!(!is_listing_page(page), "{page}");
        }
        let order = reading_order(
            &[
                c("https://shop.example/en/collections/sideboards", 1),
                c("https://shop.example/en/products/teak-sideboard", 2),
            ],
            &s(&["shop.example"]),
            &TrustedSites::default(),
            &OwnListing::default(),
        );
        assert_eq!(order.pages.len(), 1);
        assert_eq!(order.listings.len(), 1);
    }

    #[test]
    fn a9_product_handle_ignores_locale_and_collection() {
        for url in [
            "https://shop.example/products/bsh1-040826",
            "https://shop.example/es/collections/bookshelves/products/bsh1-040826",
            "https://shop.example/en/products/bsh1-040826?variant=1",
        ] {
            assert_eq!(product_handle(url).as_deref(), Some("bsh1-040826"), "{url}");
        }
        assert_eq!(
            product_handle("https://shop.example/en/collections/x"),
            None
        );
        assert_eq!(product_handle("https://shop.example/products/"), None);
    }
}
