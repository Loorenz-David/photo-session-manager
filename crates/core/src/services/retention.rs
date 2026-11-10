//! Retention after publish (10F §3, Rules, Q2, Q3, M41; P62): the read-only guard of a cleared
//! item, the verification that Shopify holds every published image of an item before anything
//! of it is deleted, and the archived view (the item's photos shown from Shopify, downloaded on
//! demand and cached small). The deletions themselves are the storage service's.

use crate::{
    Error, Result,
    domain::{
        photo::Classification,
        publish::MediaImage,
        session::PublishStep,
        storage::{ArchivedImage, ItemRetention},
    },
    ports::{commerce::CommerceAdapter, store::Store},
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

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
                .and_then(|m| m.url.clone().map(|url| (m.id.clone(), url)));
            match found {
                Some((media_id, url)) => images.push(ArchivedImage {
                    role: role.into(),
                    position: index as i64 + 1,
                    media_id,
                    url,
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

/// One archived picture as the workspace shows it: the Shopify URL and, once downloaded, the
/// cached file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchivedPicture {
    pub role: String,
    pub position: i64,
    pub url: String,
    pub path: Option<PathBuf>,
}

/// What the workspace needs to know about an item's retention (10F §3, Q2, Q3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveView {
    pub item_id: i64,
    pub retention: ItemRetention,
    pub pictures: Vec<ArchivedPicture>,
}

/// The archived view and the pin (P62).
pub struct ArchiveService<'a> {
    store: &'a dyn Store,
    commerce: Option<&'a dyn CommerceAdapter>,
    /// `<data>/previews/archived` (the small cache of downloaded Shopify images).
    cache_dir: &'a Path,
}

impl<'a> ArchiveService<'a> {
    pub fn new(
        store: &'a dyn Store,
        commerce: Option<&'a dyn CommerceAdapter>,
        cache_dir: &'a Path,
    ) -> Self {
        Self {
            store,
            commerce,
            cache_dir,
        }
    }

    fn cached_path(&self, item_id: i64, image: &ArchivedImage) -> PathBuf {
        use sha2::Digest;
        let tag: String = sha2::Sha256::digest(image.media_id.as_bytes())
            .iter()
            .take(6)
            .map(|b| format!("{b:02x}"))
            .collect();
        self.cache_dir
            .join(item_id.to_string())
            .join(format!("{}-{:02}-{tag}.jpg", image.role, image.position))
    }

    /// The session's item retention state; for a cleared item its pictures, each downloaded
    /// from Shopify once (at preview size) and then served from the cache. A download that
    /// fails leaves the picture without a file (tried again next time).
    pub fn view(&self, session_id: i64) -> Result<Option<ArchiveView>> {
        let Some(item_id) = self.store.session(session_id)?.session.item_id else {
            return Ok(None);
        };
        let retention = self.store.item_retention(item_id)?;
        let mut pictures = Vec::new();
        if retention.cleared_at.is_some() {
            for image in self.store.archived_images(item_id)? {
                let path = self.cached_path(item_id, &image);
                let cached = path.is_file() || self.download(&image, &path);
                pictures.push(ArchivedPicture {
                    role: image.role,
                    position: image.position,
                    url: image.url,
                    path: cached.then_some(path),
                });
            }
        }
        Ok(Some(ArchiveView {
            item_id,
            retention,
            pictures,
        }))
    }

    fn download(&self, image: &ArchivedImage, path: &Path) -> bool {
        let Some(commerce) = self.commerce else {
            return false;
        };
        let written = commerce
            .fetch_image(&image.url, ARCHIVED_IMAGE_EDGE)
            .and_then(|bytes| {
                let dir = path
                    .parent()
                    .ok_or_else(|| Error::InvalidData("cache path".into()))?;
                std::fs::create_dir_all(dir)?;
                let partial = path.with_extension("part");
                std::fs::write(&partial, bytes)?;
                std::fs::rename(&partial, path)?;
                Ok(())
            });
        if let Err(error) = &written {
            tracing::warn!(%error, media = image.media_id, "archived image not downloaded");
        }
        written.is_ok()
    }

    /// 10F Q2: "Keep files on this Mac" exempts the item from retention and the budget.
    pub fn set_keep_files(&self, item_id: i64, keep: bool) -> Result<ItemRetention> {
        self.store.item(item_id)?;
        self.store.set_item_keep_files(item_id, keep)?;
        self.store.item_retention(item_id)
    }
}

/// The set of sessions whose item was cleared (the preview service leaves their photos alone:
/// a cleared RAW is not "deleted outside Session Manager").
pub fn cleared_sessions(store: &dyn Store) -> Result<HashSet<i64>> {
    Ok(store.cleared_sessions()?.into_iter().collect())
}
