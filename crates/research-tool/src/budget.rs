//! Spending ceiling, per-provider call caps and model prices (R1 plan §The tool, Budget).
//!
//! Money is integer micro-USD. Prices come from `prices.json` in the out folder (filled from the
//! provider's pricing page on the day, with its URL and date) as decimal strings in USD per million
//! units, parsed exactly. A call is refused when `spent + worst case of this call ≥ ceiling`, or
//! when its cap is reached. The micro-USD helpers are temporary: R9·U1 replaces them with
//! `psm_research::rules::cost`.

use serde_json::Value as Json;
use std::{collections::BTreeMap, fmt, path::Path};

pub const MICRO: i64 = 1_000_000;

/// `"12.5"` → 12 500 000 micro-USD. At most 6 fractional digits; no sign, no exponent.
pub fn parse_micro_usd(text: &str) -> Result<i64, String> {
    let text = text.trim();
    let (whole, frac) = text.split_once('.').unwrap_or((text, ""));
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(whole) || (!frac.is_empty() && !digits(frac)) || frac.len() > 6 {
        return Err(format!("not a USD amount: {text:?}"));
    }
    let whole: i64 = whole.parse().map_err(|_| format!("too large: {text:?}"))?;
    let mut frac_micro = 0i64;
    for (i, b) in frac.bytes().enumerate() {
        frac_micro += i64::from(b - b'0') * 10i64.pow(5 - i as u32);
    }
    whole
        .checked_mul(MICRO)
        .and_then(|w| w.checked_add(frac_micro))
        .ok_or_else(|| format!("too large: {text:?}"))
}

pub fn format_usd(micro: i64) -> String {
    format!("{}.{:06}", micro / MICRO, (micro % MICRO).abs())
}

/// `ceil(Σ units × micro_usd_per_million / 10^6)`, in `i128`.
pub fn cost_micro(lines: &[(u64, i64)]) -> i64 {
    let total: i128 = lines
        .iter()
        .map(|(units, per_million)| i128::from(*units) * i128::from(*per_million))
        .sum();
    let micro = (total + 999_999).div_euclid(1_000_000);
    micro as i64
}

/// One model's prices, micro-USD per million tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelPrice {
    pub input: i64,
    pub cached_input: i64,
    /// Cache writes (30-minute lifetime) are billed at this rate instead of the input rate.
    pub cache_write: i64,
    pub output: i64,
}

#[derive(Debug, Clone, Default)]
pub struct Prices {
    pub models: BTreeMap<String, ModelPrice>,
    /// micro-USD per million web search calls.
    pub web_search_call: i64,
    /// Worst-case input tokens assumed per image when no measurement is known.
    pub image_tokens_upper_bound: u64,
    /// Worst-case search content tokens per web search call.
    pub web_search_content_tokens_upper_bound: u64,
}

impl Prices {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let json: Json = serde_json::from_str(&text).map_err(|e| format!("prices.json: {e}"))?;
        Self::from_json(&json)
    }

    pub fn from_json(json: &Json) -> Result<Self, String> {
        let money = |v: &Json, what: &str| -> Result<i64, String> {
            parse_micro_usd(
                v.as_str()
                    .ok_or_else(|| format!("prices.json: {what} must be a decimal string"))?,
            )
        };
        let mut models = BTreeMap::new();
        for (id, row) in json["models"].as_object().into_iter().flatten() {
            models.insert(
                id.clone(),
                ModelPrice {
                    input: money(&row["input_token"], "input_token")?,
                    cached_input: money(&row["cached_input_token"], "cached_input_token")?,
                    cache_write: match row.get("cache_write_token") {
                        Some(v) => money(v, "cache_write_token")?,
                        None => money(&row["input_token"], "input_token")?,
                    },
                    output: money(&row["output_token"], "output_token")?,
                },
            );
        }
        Ok(Self {
            models,
            web_search_call: money(&json["tools"]["web_search_call"], "web_search_call")?,
            image_tokens_upper_bound: json["worst_case"]["image_input_tokens"]
                .as_u64()
                .unwrap_or(6_000),
            web_search_content_tokens_upper_bound: json["worst_case"]["web_search_content_tokens"]
                .as_u64()
                .unwrap_or(40_000),
        })
    }

    pub fn model(&self, model: &str) -> Result<ModelPrice, String> {
        self.models
            .get(model)
            .copied()
            .ok_or_else(|| format!("no price for model {model} in prices.json"))
    }
}

/// The phase's hard limits.
#[derive(Debug, Clone)]
pub struct Limits {
    pub ceiling_micro: i64,
    /// Cap name → maximum number of calls (`openai`, `serpapi_search`, `serpapi_upload`,
    /// `cloud_vision`, …). A cap name absent from the map has no count limit.
    pub caps: BTreeMap<String, u32>,
}

impl Limits {
    /// R1·U1: SerpApi ≤ 130 searches and ≤ 130 uploads, Cloud Vision ≤ 130 units, OpenAI ≤ 80 calls.
    pub fn u1(ceiling_micro: i64) -> Self {
        let caps = [
            ("openai", 80),
            ("serpapi_search", 130),
            ("serpapi_upload", 130),
            ("cloud_vision", 130),
            ("shop", 4),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect();
        Self {
            ceiling_micro,
            caps,
        }
    }

    /// R1·U2: model calls only, cap 700.
    pub fn u2(ceiling_micro: i64) -> Self {
        Self {
            ceiling_micro,
            caps: [("openai".to_owned(), 700)].into_iter().collect(),
        }
    }
}

/// What has been spent and counted so far (from the recordings in the out folder).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ledger {
    pub spent_micro: i64,
    pub counts: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    Ceiling {
        spent: i64,
        worst_case: i64,
        ceiling: i64,
    },
    Cap {
        cap: String,
        limit: u32,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::Ceiling {
                spent,
                worst_case,
                ceiling,
            } => write!(
                f,
                "refused: spent USD {} + worst case of this call USD {} reaches the ceiling USD {}",
                format_usd(*spent),
                format_usd(*worst_case),
                format_usd(*ceiling)
            ),
            Refusal::Cap { cap, limit } => {
                write!(f, "refused: call cap {cap} = {limit} reached")
            }
        }
    }
}

impl Ledger {
    pub fn new(spent_micro: i64) -> Self {
        Self {
            spent_micro,
            counts: BTreeMap::new(),
        }
    }

    /// Refuses when the cap of `cap` is reached, or when `spent + worst_case ≥ ceiling`.
    pub fn check(&self, limits: &Limits, cap: &str, worst_case: i64) -> Result<(), Refusal> {
        if let Some(&limit) = limits.caps.get(cap)
            && self.counts.get(cap).copied().unwrap_or(0) >= limit
        {
            return Err(Refusal::Cap {
                cap: cap.to_owned(),
                limit,
            });
        }
        if self.spent_micro + worst_case >= limits.ceiling_micro {
            return Err(Refusal::Ceiling {
                spent: self.spent_micro,
                worst_case,
                ceiling: limits.ceiling_micro,
            });
        }
        Ok(())
    }

    pub fn add(&mut self, cap: &str, cost: i64) {
        self.spent_micro += cost;
        *self.counts.entry(cap.to_owned()).or_default() += 1;
    }

    /// Sum of every call file under `out` (`…/calls/*.json`, fields `cap` and `cost_micro_usd`).
    pub fn from_recordings(out: &Path) -> Self {
        let mut ledger = Ledger::default();
        for file in crate::recording::call_files(out) {
            if let Ok(text) = std::fs::read_to_string(&file)
                && let Ok(json) = serde_json::from_str::<Json>(&text)
                && json["sent"].as_bool().unwrap_or(true)
            {
                let cap = json["cap"].as_str().unwrap_or("").to_owned();
                ledger.add(&cap, json["cost_micro_usd"].as_i64().unwrap_or(0));
            }
        }
        ledger
    }
}
