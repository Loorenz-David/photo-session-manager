//! CS4 (`master_plan_live.md` §5): the next container in shooting order that is not taken.
use std::collections::HashSet;

/// The first container after `after` in `order` that is not taken, else the first in `order`
/// that is not taken, else `None`. `after = None` (or a container not in `order`) starts at the
/// beginning.
pub fn next_untaken(order: &[i64], taken: &HashSet<i64>, after: Option<i64>) -> Option<i64> {
    let start = after
        .and_then(|id| order.iter().position(|candidate| *candidate == id))
        .map_or(0, |index| index + 1);
    order[start..]
        .iter()
        .chain(order[..start].iter())
        .copied()
        .find(|id| !taken.contains(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORDER: [i64; 3] = [2, 1, 3];

    #[test]
    fn the_first_not_taken_after_the_given_container() {
        assert_eq!(next_untaken(&ORDER, &HashSet::new(), Some(2)), Some(1));
        assert_eq!(next_untaken(&ORDER, &HashSet::from([1]), Some(2)), Some(3));
    }

    #[test]
    fn past_the_end_it_wraps_to_the_first_not_taken() {
        assert_eq!(next_untaken(&ORDER, &HashSet::from([1]), Some(3)), Some(2));
        assert_eq!(next_untaken(&ORDER, &HashSet::from([2]), Some(3)), Some(1));
    }

    #[test]
    fn without_a_start_the_first_not_taken() {
        assert_eq!(next_untaken(&ORDER, &HashSet::new(), None), Some(2));
        assert_eq!(next_untaken(&ORDER, &HashSet::from([2]), None), Some(1));
        assert_eq!(next_untaken(&ORDER, &HashSet::from([2]), Some(99)), Some(1));
    }

    #[test]
    fn every_container_taken_is_none() {
        assert_eq!(next_untaken(&ORDER, &HashSet::from(ORDER), None), None);
        assert_eq!(next_untaken(&ORDER, &HashSet::from(ORDER), Some(1)), None);
        assert_eq!(next_untaken(&[], &HashSet::new(), None), None);
    }
}
