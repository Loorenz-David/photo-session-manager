//! MC-A8: money as integer micro-USD, the cost of one call, the stop rule and today's spend. No
//! floating point anywhere in this module.

use chrono::{TimeZone, Utc};

use crate::domain::Stage;

/// An MC-9 decimal string of USD with at most 6 fractional digits → micro-USD, exactly. Signs,
/// exponents, separators and the empty string give `None`.
pub fn parse_micro_usd(text: &str) -> Option<i64> {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    let digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    if whole.is_empty()
        || !digits(whole)
        || !digits(fraction)
        || (text.contains('.') && fraction.is_empty())
        || fraction.len() > 6
    {
        return None;
    }
    let whole: i64 = whole.parse().ok()?;
    let padded = format!("{fraction:0<6}");
    let fraction: i64 = padded.parse().ok()?;
    whole.checked_mul(1_000_000)?.checked_add(fraction)
}

/// One row of the price table: micro-USD per million units (`usd_per_million × 10^6`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Price {
    pub provider: String,
    /// `""` for non-model providers.
    pub model: String,
    pub unit: String,
    pub micro_usd_per_million: i64,
}

/// The charge for a call whose usage is unknown, per `(provider, model)`, in micro-USD.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fallback {
    pub provider: String,
    pub model: String,
    pub micro_usd: i64,
}

/// `research.prices`, already parsed (snapshotted at run start).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PriceTable {
    pub prices: Vec<Price>,
    pub fallbacks: Vec<Fallback>,
}

impl PriceTable {
    pub fn price(&self, provider: &str, model: &str, unit: &str) -> Option<i64> {
        self.prices
            .iter()
            .find(|p| p.provider == provider && p.model == model && p.unit == unit)
            .map(|p| p.micro_usd_per_million)
    }

    pub fn fallback(&self, provider: &str, model: &str) -> Option<i64> {
        self.fallbacks
            .iter()
            .find(|f| f.provider == provider && f.model == model)
            .map(|f| f.micro_usd)
    }
}

/// `ceil(Σ usage × p / 10^6)` micro-USD in `i128`. Never sent → 0. No usage reported, or a unit
/// missing from the table → the fallback for the whole call.
pub fn call_cost(
    usage: Option<&[(String, i64)]>,
    sent: bool,
    provider: &str,
    model: &str,
    prices: &PriceTable,
) -> i64 {
    if !sent {
        return 0;
    }
    let fallback = || prices.fallback(provider, model).unwrap_or(0);
    let Some(usage) = usage else {
        return fallback();
    };
    let mut total: i128 = 0;
    for (unit, count) in usage {
        let Some(price) = prices.price(provider, model, unit) else {
            return fallback();
        };
        total += i128::from(*count) * i128::from(price);
    }
    let cost = (total + 999_999).div_euclid(1_000_000);
    i64::try_from(cost).unwrap_or(i64::MAX)
}

/// What the stop rule allows before a call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextCall {
    Call,
    /// Skip the remaining reading units and go to Decide.
    SkipToDecide,
    /// The run ends `stopped_at_cap`.
    EndAtCap,
}

/// MC-A8 stop rule: stages 1–6 call iff `run_cost + reserve < cap`; Decide (and, W2·WC7, the
/// writer's stage) calls iff `run_cost < cap`.
pub fn next_call(run_cost: i64, cap: i64, reserve: i64, stage: Stage) -> NextCall {
    if matches!(stage, Stage::Decide | Stage::Write) {
        if run_cost < cap {
            NextCall::Call
        } else {
            NextCall::EndAtCap
        }
    } else if run_cost.saturating_add(reserve) < cap {
        NextCall::Call
    } else {
        NextCall::SkipToDecide
    }
}

/// Today's spend: the sum over calls `(started_at_ms, cost)` whose local calendar date in `tz`
/// equals today's (05A MC-4: calendar dates, never "now − 24 h").
pub fn day_spend<Tz: TimeZone>(calls: &[(i64, i64)], now_ms: i64, tz: &Tz) -> i64 {
    let date = |ms: i64| {
        Utc.timestamp_millis_opt(ms)
            .single()
            .map(|at| at.with_timezone(tz).date_naive())
    };
    let Some(today) = date(now_ms) else {
        return 0;
    };
    calls
        .iter()
        .filter(|(at, _)| date(*at) == Some(today))
        .map(|(_, cost)| *cost)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::Europe::Stockholm;

    fn table(rows: &[(&str, &str)], fallback: &str) -> PriceTable {
        PriceTable {
            prices: rows
                .iter()
                .map(|(unit, usd)| Price {
                    provider: "openai".into(),
                    model: "m".into(),
                    unit: (*unit).into(),
                    micro_usd_per_million: parse_micro_usd(usd).unwrap(),
                })
                .collect(),
            fallbacks: vec![Fallback {
                provider: "openai".into(),
                model: "m".into(),
                micro_usd: parse_micro_usd(fallback).unwrap(),
            }],
        }
    }

    fn usage(rows: &[(&str, i64)]) -> Vec<(String, i64)> {
        rows.iter().map(|(u, n)| ((*u).to_owned(), *n)).collect()
    }

    fn cost(prices: &PriceTable, used: Option<&[(String, i64)]>, sent: bool) -> i64 {
        call_cost(used, sent, "openai", "m", prices)
    }

    #[test]
    fn a06_c1a_parse_exact() {
        assert_eq!(parse_micro_usd("0.50"), Some(500_000));
        assert_eq!(parse_micro_usd("15"), Some(15_000_000));
        assert_eq!(parse_micro_usd("0.000001"), Some(1));
    }

    #[test]
    fn a06_c1b_parse_refused() {
        for text in ["0.0000001", "1e3", "1,5", "-1", ""] {
            assert_eq!(parse_micro_usd(text), None, "{text:?}");
        }
    }

    #[test]
    fn a06_c2a_two_units() {
        let prices = table(
            &[("input_token", "2.50"), ("output_token", "10.00")],
            "0.02",
        );
        let used = usage(&[("input_token", 1000), ("output_token", 500)]);
        assert_eq!(cost(&prices, Some(&used), true), 7500);
    }

    #[test]
    fn a06_c2b_rounded_up() {
        let prices = table(&[("input_token", "0.15")], "0.02");
        assert_eq!(cost(&prices, Some(&usage(&[("input_token", 1)])), true), 1);
    }

    #[test]
    fn a06_c2c_summed_then_rounded_once() {
        let prices = table(&[("input_token", "0.15"), ("output_token", "0.15")], "0.02");
        let used = usage(&[("input_token", 3), ("output_token", 4)]);
        assert_eq!(cost(&prices, Some(&used), true), 2);
    }

    #[test]
    fn a06_c2d_no_usage_fallback() {
        let prices = table(&[("input_token", "2.50")], "0.02");
        assert_eq!(cost(&prices, None, true), 20_000);
    }

    #[test]
    fn a06_c2e_never_sent_free() {
        let prices = table(&[("input_token", "2.50")], "0.02");
        assert_eq!(
            cost(&prices, Some(&usage(&[("input_token", 1000)])), false),
            0
        );
        assert_eq!(cost(&prices, None, false), 0);
    }

    #[test]
    fn a06_c2f_unknown_unit_fallback() {
        let prices = table(&[("input_token", "2.50")], "0.02");
        let used = usage(&[("input_token", 1000), ("web_search_call", 1)]);
        assert_eq!(cost(&prices, Some(&used), true), 20_000);
    }

    #[test]
    fn a06_c2g_no_overflow() {
        let prices = table(&[("output_token", "1000.00")], "0.02");
        let used = usage(&[("output_token", 1_000_000_000_000)]);
        assert_eq!(cost(&prices, Some(&used), true), 1_000_000_000_000_000);
    }

    #[test]
    fn a06_c3a_reading_stage_reserve() {
        assert_eq!(
            next_call(479_999, 500_000, 20_000, Stage::Read),
            NextCall::Call
        );
        assert_eq!(
            next_call(480_000, 500_000, 20_000, Stage::Read),
            NextCall::SkipToDecide
        );
    }

    #[test]
    fn a06_c3b_decide_stage_cap() {
        assert_eq!(
            next_call(499_999, 500_000, 20_000, Stage::Decide),
            NextCall::Call
        );
        assert_eq!(
            next_call(500_000, 500_000, 20_000, Stage::Decide),
            NextCall::EndAtCap
        );
    }

    #[test]
    fn w2_c14c_the_writer_stage_calls_while_below_the_cap() {
        assert_eq!(
            next_call(499_999, 500_000, 50_000, Stage::Write),
            NextCall::Call
        );
        assert_eq!(
            next_call(500_000, 500_000, 50_000, Stage::Write),
            NextCall::EndAtCap
        );
    }

    fn local(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: u32, ms: i64) -> i64 {
        Stockholm
            .with_ymd_and_hms(y, mo, d, h, mi, s)
            .earliest()
            .unwrap()
            .timestamp_millis()
            + ms
    }

    fn c4a_calls() -> (Vec<(i64, i64)>, i64) {
        (
            vec![
                (local(2026, 11, 11, 23, 59, 59, 999), 1),
                (local(2026, 11, 12, 0, 0, 0, 0), 10),
            ],
            local(2026, 11, 12, 10, 0, 0, 0),
        )
    }

    fn c4b_calls() -> (Vec<(i64, i64)>, i64) {
        let second_half_past_two = Stockholm
            .with_ymd_and_hms(2026, 10, 25, 2, 30, 0)
            .latest()
            .unwrap()
            .timestamp_millis();
        (
            vec![
                (local(2026, 10, 25, 0, 30, 0, 0), 1),
                (second_half_past_two, 10),
                (local(2026, 10, 26, 0, 10, 0, 0), 100),
            ],
            local(2026, 10, 25, 23, 30, 0, 0),
        )
    }

    #[test]
    fn a06_c4a_local_midnight() {
        let (calls, now) = c4a_calls();
        assert_eq!(day_spend(&calls, now, &Stockholm), 10);
    }

    #[test]
    fn a06_c4b_twenty_five_hour_day() {
        let (calls, now) = c4b_calls();
        assert_eq!(day_spend(&calls, now, &Stockholm), 11);
    }

    #[test]
    fn a06_c4c_evaluation_zone_decides() {
        let (calls, now) = c4a_calls();
        assert_eq!(day_spend(&calls, now, &Utc), 0);
        let (calls, now) = c4b_calls();
        assert_eq!(day_spend(&calls, now, &Utc), 110);
    }
}
