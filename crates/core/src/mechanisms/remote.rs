//! Pure rules of remote items (book 10H; master plan archive §4 "Pure rules", §5 MC-H5).

use crate::domain::remote::CallBudget;

/// MC-H5.3: background work waits while the budget is below this share of its maximum.
pub const BUDGET_FLOOR: f64 = 0.25;
/// MC-H5.5: the most media ids asked for in one URL read.
pub const URL_REFRESH_CHUNK: usize = 25;

/// MC-H5.3: whether a background step must wait for the call budget. Waits only on a known
/// budget with a positive maximum whose available points are below `BUDGET_FLOOR` of it.
pub fn must_wait(budget: Option<CallBudget>) -> bool {
    match budget {
        Some(b) => b.maximum > 0.0 && b.available < BUDGET_FLOOR * b.maximum,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(available: f64, maximum: f64) -> Option<CallBudget> {
        Some(CallBudget { available, maximum })
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
