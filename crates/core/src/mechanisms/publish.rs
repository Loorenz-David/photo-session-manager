use crate::domain::publish::PhotoLink;
use crate::domain::publish::PublishPreview;
use std::collections::HashMap;

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
            " · {unreviewed} unreviewed photos will not be published"
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
