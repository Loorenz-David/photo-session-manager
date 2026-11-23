//! Pure rules of remote items (book 10H; master plan archive §4 "Pure rules", §5 MC-H5).

use crate::domain::{
    remote::{CallBudget, RemoteKind, RemoteState},
    storage::ArchivedImage,
};

/// MC-H5.3: background work waits while the budget is below this share of its maximum.
pub const BUDGET_FLOOR: f64 = 0.25;
/// MC-H5.5: the most media ids asked for in one URL read.
pub const URL_REFRESH_CHUNK: usize = 25;
/// MC-H1.3: the most picture downloads running at the same time.
pub const PARALLEL_DOWNLOADS: usize = 4;
/// MC-H1.2: the long edge of a row picture (cover), in px.
pub const COVER_EDGE: u32 = 320;
/// MC-H1.4: a picture's address is not asked for again within this time (monotonic).
pub const REFRESH_COOLDOWN_MS: u64 = 600_000;

/// MC-H4.3: a failed bring-back download is tried this many more times.
pub const DOWNLOAD_RETRIES: usize = 2;
/// MC-H3.5: the size assumed for a remote picture whose size the shop did not tell (10F's
/// measured output size).
pub const UNKNOWN_PICTURE_BYTES: i64 = 1_500_000;

/// MC-H2.5: the long edge of a shop search result's picture, in px.
pub const SEARCH_EDGE: u32 = 160;
/// MC-H2.6: the search-picture cache holds at most this many bytes.
pub const SEARCH_CACHE_LIMIT_BYTES: u64 = 200_000_000;

/// MC-H2.5: the search-picture cache file name of a picture address,
/// `<first 16 hex of sha256(url without its query)>.jpg` (a fresh address of the same picture
/// keeps its name).
pub fn search_name(url: &str) -> String {
    let bare = url.split('?').next().unwrap_or(url);
    format!("{}.jpg", hex_prefix(bare, 16))
}

/// MC-H2.6: the files to delete so the folder totals at most `limit` bytes: the oldest first (by
/// modified time, name ascending on a tie), until the rest fits. `entries` are `(name, bytes,
/// modified_ms)`.
pub fn evict<S: AsRef<str>>(entries: &[(S, u64, i64)], limit: u64) -> Vec<String> {
    let mut total: u64 = entries.iter().map(|(_, bytes, _)| *bytes).sum();
    let mut oldest: Vec<&(S, u64, i64)> = entries.iter().collect();
    oldest.sort_by(|a, b| a.2.cmp(&b.2).then_with(|| a.0.as_ref().cmp(b.0.as_ref())));
    let mut out = Vec::new();
    for (name, bytes, _) in oldest {
        if total <= limit {
            break;
        }
        total -= bytes;
        out.push(name.as_ref().to_owned());
    }
    out
}

/// MC-H2.1: the sessions list shows this many rows per page.
pub const PAGE_SIZE: usize = 25;

/// MC-H2.1: `(page, pages, rows)` for `total` rows and the requested page: `pages =
/// max(1, ceil(total / PAGE_SIZE))`, the page clamped to `1..=pages`, the rows' index range.
pub fn page_of(total: usize, page: u32) -> (u32, u32, std::ops::Range<usize>) {
    let pages = total.div_ceil(PAGE_SIZE).max(1) as u32;
    let page = page.clamp(1, pages);
    let start = (page as usize - 1) * PAGE_SIZE;
    (page, pages, start..(start + PAGE_SIZE).min(total))
}

/// MC-H1.2: the cover cache file name of a remote picture,
/// `cover-<first 12 hex of sha256(media_id)>.jpg`.
pub fn cover_name(media_id: &str) -> String {
    format!("cover-{}.jpg", hex_prefix(media_id, 12))
}

/// The first `chars` lowercase hex characters of the SHA-256 of `text`.
fn hex_prefix(text: &str, chars: usize) -> String {
    use sha2::Digest;
    let mut hex: String = sha2::Sha256::digest(text.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    hex.truncate(chars);
    hex
}

/// MC-H5.3: whether a background step must wait for the call budget. Waits only on a known
/// budget with a positive maximum whose available part is below `BUDGET_FLOOR` of it.
pub fn must_wait(budget: Option<CallBudget>) -> bool {
    match budget {
        Some(b) => b.maximum > 0.0 && b.available < BUDGET_FLOOR * b.maximum,
        None => false,
    }
}

/// MC-H4.2: the file names of a bring back, `<folder_key>-main-<NN>.jpg` for each gallery
/// picture then `<folder_key>-condition-<NN>.jpg` for each condition picture (`NN` = position,
/// two digits at least). When any name of the set is `taken`, the whole set carries `-b2` before
/// `.jpg`, then `-b3`, …: the smallest suffix for which none is taken.
pub fn bring_back_names(
    folder_key: &str,
    gallery: usize,
    condition: usize,
    taken: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let set = |round: usize| -> Vec<String> {
        let suffix = if round == 1 {
            String::new()
        } else {
            format!("-b{round}")
        };
        (1..=gallery)
            .map(|n| format!("{folder_key}-main-{n:02}{suffix}.jpg"))
            .chain((1..=condition).map(|n| format!("{folder_key}-condition-{n:02}{suffix}.jpg")))
            .collect()
    };
    let mut round = 1;
    loop {
        let names = set(round);
        if !names.iter().any(|name| taken(name)) {
            return names;
        }
        round += 1;
    }
}

/// MC-H3.5: the bytes a bring back of `pictures` downloads, about: each picture's `bytes`, or
/// `UNKNOWN_PICTURE_BYTES` when the shop did not tell it.
pub fn estimate_bytes(pictures: &[ArchivedImage]) -> i64 {
    pictures
        .iter()
        .map(|picture| picture.bytes.unwrap_or(UNKNOWN_PICTURE_BYTES))
        .sum()
}

/// What the remote state rule needs to know about a session's item (MC-H3.1), read by
/// `Store::remote_input` in one read. A session without an item answers all-false.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RemoteInput {
    /// The item's local files were cleared (`item_retention.cleared_at` set).
    pub cleared: bool,
    /// The item has a link for the integration.
    pub linked: bool,
    /// The item came from the shop (its origin is `linked`).
    pub adopted: bool,
    /// Photo rows of the session, of any classification.
    pub photo_rows: usize,
    /// The item's remote pictures were read (`remote_checked_at` set).
    pub checked: bool,
    /// Rows of the item's record of remote pictures.
    pub pictures: usize,
}

/// What the row picture rule needs about one session's item besides its photos (MC-H1.1), read
/// for every session in one read (`Store::remote_covers`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteCover {
    pub session_id: i64,
    pub item_id: i64,
    pub sku: String,
    pub input: RemoteInput,
    /// The record's gallery picture with the lowest position (gone when its URL is empty).
    pub cover: Option<ArchivedImage>,
}

/// MC-H3.1: cleared → archived; an adopted, linked item without photo rows is unknown until
/// read, then shop-only when the shop holds a picture, else local; everything else is local.
pub fn remote_state(input: RemoteInput) -> RemoteState {
    if input.cleared {
        return RemoteState::Remote(RemoteKind::Archived);
    }
    if input.adopted && input.linked && input.photo_rows == 0 {
        return if !input.checked {
            RemoteState::Unknown
        } else if input.pictures >= 1 {
            RemoteState::Remote(RemoteKind::ShopOnly)
        } else {
            RemoteState::Local
        };
    }
    RemoteState::Local
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none_taken(_: &str) -> bool {
        false
    }

    #[test]
    fn h3_c3a_names_main_then_condition() {
        assert_eq!(
            bring_back_names("BEYO-12", 2, 1, &none_taken),
            vec![
                "BEYO-12-main-01.jpg",
                "BEYO-12-main-02.jpg",
                "BEYO-12-condition-01.jpg"
            ]
        );
    }

    #[test]
    fn h3_c3b_one_taken_name_suffixes_the_whole_set() {
        let taken = |name: &str| name == "BEYO-12-main-02.jpg";
        assert_eq!(
            bring_back_names("BEYO-12", 2, 1, &taken),
            vec![
                "BEYO-12-main-01-b2.jpg",
                "BEYO-12-main-02-b2.jpg",
                "BEYO-12-condition-01-b2.jpg"
            ]
        );
    }

    #[test]
    fn h3_c3c_the_smallest_free_suffix() {
        let taken =
            |name: &str| name == "BEYO-12-main-02.jpg" || name == "BEYO-12-condition-01-b2.jpg";
        let names = bring_back_names("BEYO-12", 2, 1, &taken);
        assert_eq!(
            names,
            vec![
                "BEYO-12-main-01-b3.jpg",
                "BEYO-12-main-02-b3.jpg",
                "BEYO-12-condition-01-b3.jpg"
            ]
        );
    }

    #[test]
    fn h3_c3d_twelve_names_two_digits() {
        let names = bring_back_names("K", 12, 0, &none_taken);
        assert_eq!(names.len(), 12);
        assert_eq!(names.last().map(String::as_str), Some("K-main-12.jpg"));
    }

    #[test]
    fn h3_c3e_estimate_uses_sizes_or_the_assumed_size() {
        let picture = |bytes| ArchivedImage {
            role: "gallery".into(),
            position: 1,
            media_id: "m".into(),
            url: "u".into(),
            bytes,
        };
        assert_eq!(
            estimate_bytes(&[picture(Some(1_000_000)), picture(None)]),
            2_500_000
        );
        assert_eq!(estimate_bytes(&[]), 0);
    }

    const ARCHIVED: RemoteState = RemoteState::Remote(RemoteKind::Archived);
    const SHOP_ONLY: RemoteState = RemoteState::Remote(RemoteKind::ShopOnly);

    /// H1·C7(c)'s input: not cleared, adopted, linked, no photo rows, not checked.
    fn adopted() -> RemoteInput {
        RemoteInput {
            adopted: true,
            linked: true,
            ..RemoteInput::default()
        }
    }

    #[test]
    fn h1_c7a_cleared_is_archived() {
        let input = RemoteInput {
            cleared: true,
            ..RemoteInput::default()
        };
        assert_eq!(remote_state(input), ARCHIVED);
    }

    #[test]
    fn h1_c7b_cleared_wins() {
        let input = RemoteInput {
            cleared: true,
            adopted: true,
            linked: true,
            photo_rows: 4,
            checked: true,
            pictures: 0,
        };
        assert_eq!(remote_state(input), ARCHIVED);
    }

    #[test]
    fn h1_c7c_adopted_not_checked_is_unknown() {
        assert_eq!(remote_state(adopted()), RemoteState::Unknown);
    }

    #[test]
    fn h1_c7d_checked_with_pictures_is_shop_only() {
        let input = RemoteInput {
            checked: true,
            pictures: 3,
            ..adopted()
        };
        assert_eq!(remote_state(input), SHOP_ONLY);
    }

    #[test]
    fn h1_c7e_checked_without_pictures_is_local() {
        let input = RemoteInput {
            checked: true,
            pictures: 0,
            ..adopted()
        };
        assert_eq!(remote_state(input), RemoteState::Local);
    }

    #[test]
    fn h1_c7f_a_photo_row_makes_it_local() {
        let input = RemoteInput {
            checked: true,
            pictures: 3,
            photo_rows: 1,
            ..adopted()
        };
        assert_eq!(remote_state(input), RemoteState::Local);
    }

    #[test]
    fn h1_c7g_not_adopted_or_not_linked_is_local() {
        let shop_only = RemoteInput {
            checked: true,
            pictures: 3,
            ..adopted()
        };
        let not_adopted = RemoteInput {
            adopted: false,
            ..shop_only
        };
        let not_linked = RemoteInput {
            linked: false,
            ..shop_only
        };
        assert_eq!(
            (remote_state(not_adopted), remote_state(not_linked)),
            (RemoteState::Local, RemoteState::Local)
        );
    }

    fn b(available: f64, maximum: f64) -> Option<CallBudget> {
        Some(CallBudget { available, maximum })
    }

    #[test]
    fn h2_c6a_pages_of_twenty_five() {
        assert_eq!(
            (page_of(60, 1), page_of(60, 3)),
            ((1, 3, 0..25), (3, 3, 50..60))
        );
    }

    #[test]
    fn h2_c6b_the_page_is_clamped() {
        assert_eq!(
            (page_of(60, 9), page_of(60, 0)),
            ((3, 3, 50..60), (1, 3, 0..25))
        );
    }

    #[test]
    fn h2_c6c_at_least_one_page() {
        assert_eq!(page_of(0, 1), (1, 1, 0..0));
    }

    #[test]
    fn h2_c6d_the_boundary() {
        assert_eq!(
            (page_of(25, 1), page_of(26, 2)),
            ((1, 1, 0..25), (2, 2, 25..26))
        );
    }

    #[test]
    fn h2_c13a_the_oldest_goes_first() {
        assert_eq!(
            evict(&[("a", 120, 1), ("b", 60, 2), ("c", 50, 3)], 200),
            vec!["a"]
        );
    }

    #[test]
    fn h2_c13b_at_the_limit_nothing_goes() {
        assert!(evict(&[("a", 120, 1), ("b", 80, 2)], 200).is_empty());
    }

    #[test]
    fn h2_c13c_name_ascending_on_a_tie() {
        assert_eq!(
            evict(&[("a", 150, 1), ("b", 150, 1), ("c", 150, 2)], 200),
            vec!["a", "b"]
        );
    }

    #[test]
    fn h2_c13e_search_name_ignores_the_query() {
        use sha2::Digest;
        let digest: String = sha2::Sha256::digest(b"https://cdn.shopify.com/a/f.jpg")
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let want = format!("{}.jpg", &digest[..16]);
        assert_eq!(
            (
                search_name("https://cdn.shopify.com/a/f.jpg?v=1"),
                search_name("https://cdn.shopify.com/a/f.jpg?v=2")
            ),
            (want.clone(), want)
        );
    }

    #[test]
    fn h2_c5d_cover_name() {
        use sha2::Digest;
        let digest: String = sha2::Sha256::digest(b"gid://shopify/MediaImage/1001")
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(
            cover_name("gid://shopify/MediaImage/1001"),
            format!("cover-{}.jpg", &digest[..12])
        );
    }

    #[test]
    fn h1_c3a_below_a_quarter_waits() {
        assert!(must_wait(b(249.0, 1000.0)));
    }

    #[test]
    fn h1_c3b_a_quarter_does_not_wait() {
        assert!(!must_wait(b(250.0, 1000.0)));
    }

    #[test]
    fn h1_c3c_unknown_budget_does_not_wait() {
        assert!(!must_wait(None));
    }

    #[test]
    fn h1_c3d_zero_maximum_does_not_wait() {
        assert!(!must_wait(b(0.0, 0.0)));
    }
}
