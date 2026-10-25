pub type Positioned = Vec<(i64, i64)>;

fn number(ids: impl IntoIterator<Item = i64>) -> Positioned {
    ids.into_iter()
        .enumerate()
        .map(|(index, id)| (id, index as i64 + 1))
        .collect()
}

pub fn classify_into(group: &[(i64, i64)], id: i64) -> Positioned {
    number(group.iter().map(|(id, _)| *id).chain(std::iter::once(id)))
}

pub fn remove_at(group: &[(i64, i64)], id: i64) -> Positioned {
    number(
        group
            .iter()
            .filter(|(other, _)| *other != id)
            .map(|(id, _)| *id),
    )
}

pub fn reorder(group: &[(i64, i64)], id: i64, requested: i64) -> Positioned {
    let mut ids: Vec<i64> = group.iter().map(|(id, _)| *id).collect();
    if let Some(old) = ids.iter().position(|other| *other == id) {
        ids.remove(old);
        let target = requested.clamp(1, group.len() as i64) as usize - 1;
        ids.insert(target, id);
    }
    number(ids)
}

pub fn undo_insert(group: &[(i64, i64)], id: i64, prior_position: i64) -> Positioned {
    let mut ids: Vec<i64> = group.iter().map(|(id, _)| *id).collect();
    let target = prior_position.clamp(1, ids.len() as i64 + 1) as usize - 1;
    ids.insert(target, id);
    number(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mc5_position_operations() {
        let group = vec![(1, 1), (2, 2), (3, 3)];
        assert_eq!(
            classify_into(&group, 4),
            vec![(1, 1), (2, 2), (3, 3), (4, 4)]
        );
        assert_eq!(remove_at(&group, 2), vec![(1, 1), (3, 2)]);
        assert_eq!(reorder(&group, 1, 3), vec![(2, 1), (3, 2), (1, 3)]);
        assert_eq!(undo_insert(&remove_at(&group, 2), 2, 2), group);
    }
}
