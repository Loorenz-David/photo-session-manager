use crate::domain::photo::Classification;
use crate::domain::publish::PhotoLink;
use crate::domain::publish::PublishPreview;
use std::collections::HashMap;

/// P64: uploads in flight at once for one publish item.
pub const PARALLEL_UPLOADS: usize = 4;

/// P64: the waves of a publish item's pending uploads: consecutive runs of at most
/// [`PARALLEL_UPLOADS`] indexes into the pending list, in order. Each wave is uploaded in parallel
/// and registered on the platform in one request before the next wave starts.
pub fn upload_waves(pending: usize) -> Vec<std::ops::Range<usize>> {
    (0..pending)
        .step_by(PARALLEL_UPLOADS)
        .map(|start| start..(start + PARALLEL_UPLOADS).min(pending))
        .collect()
}

/// 07A MC-10 `frozen_main` (changelog 44): the `main` photos in their order, followed by the
/// `unreviewed` photos in their order; each in the order the workspace shows them (position, then
/// capture time, then id: an unreviewed photo has no position, so it is its arrival order).
/// Condition and deleted photos are not main photos.
/// Input: (classification, position, captured_at, photo id) of the session's photos, any order.
pub fn frozen_main(photos: &[(Classification, Option<i64>, Option<i64>, i64)]) -> Vec<i64> {
    let mut ordered: Vec<(u8, Option<i64>, Option<i64>, i64)> = photos
        .iter()
        .filter_map(|&(class, position, captured_at, id)| match class {
            Classification::Main => Some((0, position, captured_at, id)),
            Classification::Unreviewed => Some((1, position, captured_at, id)),
            Classification::Condition | Classification::Deleted => None,
        })
        .collect();
    ordered.sort();
    ordered.into_iter().map(|(_, _, _, id)| id).collect()
}

/// Ready links to replace in place (07B MC-21, 10D MC-29 §Links): the RAW has a newer retouch
/// generation than the link's, **or** the photo's composition revision is greater than the
/// link's `composition_rev`. Returns the photo ids of those links, in link order.
pub fn replacement_candidates(
    frozen_links: &[PhotoLink],
    raw_retouch_gens: &HashMap<i64, i64>,
    composition_revs: &HashMap<i64, i64>,
) -> Vec<i64> {
    frozen_links
        .iter()
        .filter(|link| {
            link.upload_state == "ready"
                && (raw_retouch_gens
                    .get(&link.photo_id)
                    .is_some_and(|generation| *generation > link.retouch_gen)
                    || composition_revs
                        .get(&link.photo_id)
                        .is_some_and(|revision| *revision > link.composition_rev))
        })
        .map(|link| link.photo_id)
        .collect()
}

pub fn publish_summary(preview: &PublishPreview) -> String {
    let images: usize = preview
        .items
        .iter()
        .map(|item| item.main.len() + item.condition.len())
        .sum();
    let unreviewed: usize = preview.items.iter().map(|item| item.unreviewed).sum();
    let removals: usize = preview
        .items
        .iter()
        .map(|item| item.announced_removals.len())
        .sum();
    let creates = preview.items.iter().filter(|item| item.create_item).count();
    let mut text = format!("Publish {images} images");
    if unreviewed > 0 {
        text.push_str(&format!(
            " · {unreviewed} unreviewed photos will be published after the main photos"
        ));
    }
    if removals > 0 {
        text.push_str(&format!(
            " · {removals} photo{} will be removed from the gallery",
            if removals == 1 { "" } else { "s" }
        ));
    }
    if creates > 0 {
        text.push_str(if creates == 1 {
            " · The product will be created on Shopify"
        } else {
            " · Products will be created on Shopify"
        });
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn upload_waves_are_runs_of_at_most_four_in_order() {
        assert!(upload_waves(0).is_empty());
        assert_eq!(upload_waves(3), vec![0..3]);
        assert_eq!(upload_waves(8), vec![0..4, 4..8]);
        assert_eq!(upload_waves(11), vec![0..4, 4..8, 8..11]);
    }
    #[test]
    fn frozen_main_is_main_then_unreviewed_each_in_display_order() {
        use Classification as C;
        // A=1, B=2 main at positions 2, 1; C=3, D=4 unreviewed (no position) arriving at 10, 20;
        // E=5 condition; F=6 deleted.
        let photos = [
            (C::Unreviewed, None, Some(20), 4),
            (C::Main, Some(2), Some(5), 1),
            (C::Deleted, None, Some(1), 6),
            (C::Unreviewed, None, Some(10), 3),
            (C::Condition, Some(1), Some(1), 5),
            (C::Main, Some(1), Some(9), 2),
        ];
        assert_eq!(frozen_main(&photos), vec![2, 1, 3, 4]);
        // Same capture time: id order (the workspace's last tie-break).
        assert_eq!(
            frozen_main(&[
                (C::Unreviewed, None, Some(7), 9),
                (C::Unreviewed, None, Some(7), 8)
            ]),
            vec![8, 9]
        );
        assert!(
            frozen_main(&[
                (C::Condition, Some(1), None, 1),
                (C::Deleted, None, None, 2)
            ])
            .is_empty()
        );
    }
    #[test]
    fn mc21_replacement_candidates_only_newer_ready_links() {
        let link = |id, state: &str, generation, composition_rev| PhotoLink {
            photo_id: id,
            item_id: 1,
            role: "gallery".into(),
            slot: crate::domain::composition::OutputSlot::Single,
            external_id: Some(format!("m{id}")),
            upload_state: state.into(),
            export_sha256: None,
            superseded_ids: vec![],
            revision: 0,
            retouch_gen: generation,
            composition_rev,
        };
        let links = vec![
            link(1, "ready", 0, 0),
            link(2, "ready", 2, 0),
            link(3, "exported", 0, 0),
        ];
        assert_eq!(
            replacement_candidates(
                &links,
                &HashMap::from([(1, 1), (2, 2), (3, 1)]),
                &HashMap::new()
            ),
            vec![1]
        );
    }

    #[test]
    fn mc29_replacement_on_a_newer_composition_revision() {
        let link = |id, state: &str, composition_rev| PhotoLink {
            photo_id: id,
            item_id: 1,
            role: "gallery".into(),
            slot: crate::domain::composition::OutputSlot::Left,
            external_id: Some(format!("m{id}")),
            upload_state: state.into(),
            export_sha256: None,
            superseded_ids: vec![],
            revision: 0,
            retouch_gen: 0,
            composition_rev,
        };
        let links = vec![
            link(1, "ready", 1),
            link(2, "ready", 2),
            link(3, "ready", 3),
            link(4, "uploaded", 0),
        ];
        let retouch = HashMap::from([(1, 0), (2, 0), (3, 0), (4, 0)]);
        assert_eq!(
            replacement_candidates(
                &links,
                &retouch,
                &HashMap::from([(1, 2), (2, 2), (3, 1), (4, 5)])
            ),
            vec![1]
        );
    }
}
