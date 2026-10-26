//! MC-12 gallery diff; remote ids remain opaque to core.
use std::collections::HashSet;

pub type MediaId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub deletes: HashSet<MediaId>,
    pub moves: Vec<(MediaId, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconcileError {
    Foreign(usize),
}

pub fn plan(
    desired: &[MediaId],
    current: &[MediaId],
    own: &HashSet<MediaId>,
    announced: &HashSet<MediaId>,
) -> Result<Plan, ReconcileError> {
    let desired_set: HashSet<_> = desired.iter().collect();
    let unwanted: Vec<_> = current
        .iter()
        .filter(|id| !desired_set.contains(id))
        .collect();
    let foreign = unwanted
        .iter()
        .filter(|id| !own.contains(id.as_str()) && !announced.contains(id.as_str()))
        .count();
    if foreign != 0 {
        return Err(ReconcileError::Foreign(foreign));
    }
    Ok(Plan {
        deletes: unwanted.into_iter().cloned().collect(),
        moves: desired
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, id)| (id, i))
            .collect(),
    })
}
