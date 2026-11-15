//! Which photos stage 1 looks at, in which order and with which role (R12·U1; MC-A7 as the R12
//! plan supersedes it, 08B changelog 12 (d)). Until a shoot template names the Main positions, the
//! only fact is the owner's: the first Main photo is the wide one; every other chosen photo is a
//! closer one. Plain values in, plain values out.

use psm_core::domain::photo::Classification;
use serde::{Deserialize, Serialize};

/// What a photo is in the one Observe call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// The Main photo at the lowest position: says what the piece is and how many there are.
    Wide,
    /// Every other chosen photo: wood, material, colour and surface.
    Closer,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Wide => "wide",
            Role::Closer => "closer",
        }
    }
}

/// A usable photo of the run (eligible, with a research source).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoCandidate {
    pub id: i64,
    pub classification: Classification,
    pub position: Option<i64>,
    pub revision: i64,
}

/// One photo chosen for the Observe call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chosen {
    pub id: i64,
    pub revision: i64,
    pub role: Role,
}

/// The photos to look at, in call order, and how many Main or unreviewed photos the limit left
/// out.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhotoPlan {
    pub chosen: Vec<Chosen>,
    pub left_out: usize,
}

/// Main photos by position (ties by id; no position last), then unreviewed photos by id, cut at
/// `limit`; Condition (and every other class of) photos never. The Main photo at the lowest
/// position is `Wide`; every other chosen photo is `Closer`.
pub fn look_at(photos: &[PhotoCandidate], limit: usize) -> PhotoPlan {
    let mut mains: Vec<&PhotoCandidate> = photos
        .iter()
        .filter(|p| p.classification == Classification::Main)
        .collect();
    mains.sort_by_key(|p| (p.position.unwrap_or(i64::MAX), p.id));
    let mut unreviewed: Vec<&PhotoCandidate> = photos
        .iter()
        .filter(|p| p.classification == Classification::Unreviewed)
        .collect();
    unreviewed.sort_by_key(|p| p.id);
    let ordered: Vec<&PhotoCandidate> = mains.into_iter().chain(unreviewed).collect();
    let take = ordered.len().min(limit);
    let chosen = ordered[..take]
        .iter()
        .enumerate()
        .map(|(i, p)| Chosen {
            id: p.id,
            revision: p.revision,
            role: if i == 0 && p.classification == Classification::Main {
                Role::Wide
            } else {
                Role::Closer
            },
        })
        .collect();
    PhotoPlan {
        chosen,
        left_out: ordered.len() - take,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn photo(id: i64, classification: Classification, position: Option<i64>) -> PhotoCandidate {
        PhotoCandidate {
            id,
            classification,
            position,
            revision: 1,
        }
    }

    fn ids_and_roles(plan: &PhotoPlan) -> Vec<(i64, Role)> {
        plan.chosen.iter().map(|c| (c.id, c.role)).collect()
    }

    #[test]
    fn a26_c1a_mains_by_position_then_unreviewed_never_condition() {
        let photos = vec![
            photo(30, Classification::Condition, None),
            photo(12, Classification::Main, Some(3)),
            photo(5, Classification::Unreviewed, None),
            photo(11, Classification::Main, Some(2)),
            photo(31, Classification::Condition, None),
            photo(14, Classification::Main, Some(1)),
        ];
        let plan = look_at(&photos, 8);
        assert_eq!(
            ids_and_roles(&plan),
            vec![
                (14, Role::Wide),
                (11, Role::Closer),
                (12, Role::Closer),
                (5, Role::Closer)
            ]
        );
        assert_eq!(plan.left_out, 0);
    }

    #[test]
    fn a26_c1b_ten_mains_limit_eight() {
        // Ids in reverse of position, so position (not id) decides the order.
        let photos: Vec<PhotoCandidate> = (1..=10)
            .map(|pos| photo(100 - pos, Classification::Main, Some(pos)))
            .collect();
        let plan = look_at(&photos, 8);
        assert_eq!(
            plan.chosen.iter().map(|c| c.id).collect::<Vec<_>>(),
            (1..=8).map(|pos| 100 - pos).collect::<Vec<_>>()
        );
        assert_eq!(plan.chosen[0].role, Role::Wide);
        assert!(plan.chosen[1..].iter().all(|c| c.role == Role::Closer));
        assert_eq!(plan.left_out, 2);
    }

    #[test]
    fn a26_c1c_limit_one_keeps_the_wide_photo() {
        let photos = vec![
            photo(2, Classification::Main, Some(2)),
            photo(3, Classification::Main, Some(3)),
            photo(1, Classification::Main, Some(1)),
        ];
        let plan = look_at(&photos, 1);
        assert_eq!(ids_and_roles(&plan), vec![(1, Role::Wide)]);
        assert_eq!(plan.left_out, 2);
    }

    #[test]
    fn a26_c1d_only_condition_photos_none() {
        let photos = vec![
            photo(1, Classification::Condition, None),
            photo(2, Classification::Condition, None),
        ];
        assert_eq!(look_at(&photos, 8), PhotoPlan::default());
    }
}
