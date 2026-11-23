//! Retention after publish (10F §3, Rules, Q2, Q3, M41; P62): the read-only guard of a cleared
//! item and the verification that Shopify holds every published image of an item before anything
//! of it is deleted. The deletions themselves are the storage service's; the archived view and
//! the pin are `RemoteItems`' (H4, MC-H3.5).

use crate::{
    Error, Result,
    domain::{
        photo::Classification, publish::MediaImage, session::PublishStep, storage::ArchivedImage,
    },
    ports::{commerce::CommerceAdapter, store::Store},
};
use std::collections::HashSet;

/// The integration whose links and media are verified.
pub const INTEGRATION: &str = "shopify";
/// Archived images are downloaded at preview size (delegated choice: the preview recipe's
/// 1600 px long edge, under the plan's 2560 px bound).
pub const ARCHIVED_IMAGE_EDGE: u32 = 1600;
pub const ROLE_GALLERY: &str = "gallery";
pub const ROLE_CONDITION: &str = "condition";

/// 10F Q3: refused for a session whose item's files were cleared (no session item → allowed).
pub fn ensure_session_not_cleared(store: &dyn Store, session_id: i64) -> Result<()> {
    let Some(item_id) = store.session(session_id)?.session.item_id else {
        return Ok(());
    };
    if store.item_retention(item_id)?.cleared_at.is_some() {
        return Err(Error::FilesCleared);
    }
    Ok(())
}

/// 10F Q3 for a photo: refused when the photo's session's item was cleared.
pub fn ensure_photo_not_cleared(store: &dyn Store, photo_id: i64) -> Result<()> {
    ensure_session_not_cleared(store, store.photo(photo_id)?.session_id)
}

/// The cache file name of one remote picture in `<previews>/archived/<item_id>/` (P62):
/// `<role>-<NN>-<first 12 hex of sha256(media_id)>.jpg`.
pub fn archived_file_name(image: &ArchivedImage) -> String {
    use sha2::Digest;
    let tag: String = sha2::Sha256::digest(image.media_id.as_bytes())
        .iter()
        .take(6)
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("{}-{:02}-{tag}.jpg", image.role, image.position)
}

/// The outcome of reading an item's published images back from the platform (M41).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verification {
    /// Every published image is there: the images, gallery then condition, each in order.
    Holds(Vec<ArchivedImage>),
    /// `n` published images are not on the platform (or the app cannot say which they are).
    Missing(usize),
}

/// The platform ids expected for one role, in order (`None` for a slot without a ready link).
type Expected = Vec<Option<String>>;

/// The published images of the item's last completed publish: the selected main photos'
/// output slots (gallery links) and condition photos' slots (condition-field links), in order.
/// `None` for a slot without a ready link.
fn published_images(
    store: &dyn Store,
    commerce: &dyn CommerceAdapter,
    item_id: i64,
    session_id: i64,
) -> Result<(Expected, Expected)> {
    let latest = store
        .latest_publish(session_id)?
        .filter(|latest| latest.step == PublishStep::Done)
        .ok_or_else(|| Error::InvalidData("no completed publish".into()))?;
    let publish = store.publish_item(latest.item_id)?;
    let condition_role = format!("attribute:{}", commerce.condition_photos_key());
    let mut groups = (Vec::new(), Vec::new());
    for (ids, role, out) in [
        (&publish.frozen_main, ROLE_GALLERY, &mut groups.0),
        (
            &publish.frozen_condition,
            condition_role.as_str(),
            &mut groups.1,
        ),
    ] {
        for &photo_id in ids {
            let photo = store.photo(photo_id)?;
            if photo.session_id != session_id || photo.classification == Classification::Deleted {
                continue;
            }
            for slot in store.composition(photo_id)?.kind.slots() {
                let link = store.photo_link(photo_id, item_id, role, *slot)?;
                out.push(
                    link.filter(|l| l.upload_state == "ready")
                        .and_then(|l| l.external_id),
                );
            }
        }
    }
    Ok(groups)
}

/// 10F Rules / M41: reads the item's media back from the platform and checks that every
/// published image is present (with a URL to show it from). An error means the read failed.
pub fn verify(
    store: &dyn Store,
    commerce: &dyn CommerceAdapter,
    item_id: i64,
    session_id: i64,
) -> Result<Verification> {
    let external = store
        .item_link(item_id, INTEGRATION)?
        .ok_or_else(|| Error::NotFound(format!("shop link of item {item_id}")))?
        .external_id;
    let (gallery, condition) = published_images(store, commerce, item_id, session_id)?;
    let readback = commerce.media_readback(&external)?;
    let mut missing = 0;
    let mut images = Vec::new();
    for (expected, held, role) in [
        (&gallery, &readback.gallery, ROLE_GALLERY),
        (&condition, &readback.condition, ROLE_CONDITION),
    ] {
        for (index, id) in expected.iter().enumerate() {
            let found = id
                .as_ref()
                .and_then(|id| held.iter().find(|m: &&MediaImage| &m.id == id))
                .and_then(|m| m.url.clone().map(|url| (m.id.clone(), url, m.bytes)));
            match found {
                Some((media_id, url, bytes)) => images.push(ArchivedImage {
                    role: role.into(),
                    position: index as i64 + 1,
                    media_id,
                    url,
                    bytes,
                }),
                None => missing += 1,
            }
        }
    }
    Ok(if missing == 0 {
        Verification::Holds(images)
    } else {
        Verification::Missing(missing)
    })
}

/// The set of sessions whose item was cleared (the preview service leaves their photos alone:
/// a cleared RAW is not "deleted outside Session Manager").
pub fn cleared_sessions(store: &dyn Store) -> Result<HashSet<i64>> {
    Ok(store.cleared_sessions()?.into_iter().collect())
}
