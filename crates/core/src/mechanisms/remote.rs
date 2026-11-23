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
