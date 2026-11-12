//! Research settings and the run's settings snapshot.
//!
//! [`ResearchConfig`] is what a run executes from: it is written into
//! `research_runs.settings_snapshot` by the start transaction (MC-A6) and never read from the live
//! settings afterwards (MC-A3: edits apply to runs that start later).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    domain::Stage,
    rules::{
        cost::{Fallback, Price, PriceTable},
        sites::OwnListing,
    },
};

/// The version of the instructions each stage sends (part of every unit identity, MC-A7).
pub const INSTRUCTIONS_VERSION: u32 = 1;

/// Model, effort and image size of one stage.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct StageConfig {
    pub model: String,
    pub effort: String,
    pub image_long_edge: u32,
}

/// One parsed price row: micro-USD per million units.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceSnapshot {
    pub provider: String,
    pub model: String,
    pub unit: String,
    pub micro_usd_per_million: i64,
}

/// One parsed fallback: micro-USD per call whose usage is unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FallbackSnapshot {
    pub provider: String,
    pub model: String,
    pub micro_usd: i64,
}

/// The adapter ids a run calls (the ids the price table is keyed by).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderIds {
    pub model: String,
    pub visual_search: String,
    pub page_reader: String,
}

/// The run's settings snapshot (MC-A6 start transaction; MC-A3, MC-A8 snapshots).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ResearchConfig {
    pub stages: BTreeMap<String, StageConfig>,
    pub max_pages: u32,
    pub max_cost_per_run: i64,
    pub max_cost_per_day: i64,
    pub prices: Vec<PriceSnapshot>,
    pub fallbacks: Vec<FallbackSnapshot>,
    pub trusted_sites: Vec<String>,
    pub shop_hosts: Vec<String>,
    pub own_listing_url: Option<String>,
    pub own_listing_path_tail: Vec<String>,
    pub attribution_keys: Vec<String>,
    pub language: String,
    pub providers: ProviderIds,
    pub normalise_version: u32,
    pub fingerprint_version: u32,
    pub instructions_version: u32,
}

impl Default for ResearchConfig {
    fn default() -> Self {
        let stage = |edge| StageConfig {
            model: String::new(),
            effort: String::new(),
            image_long_edge: edge,
        };
        let stages = Stage::ALL
            .into_iter()
            .map(|s| {
                let edge = match s {
                    Stage::Labels | Stage::Condition => 2048,
                    _ => 1024,
                };
                (s.as_str().to_owned(), stage(edge))
            })
            .collect();
        Self {
            stages,
            max_pages: 10,
            max_cost_per_run: 500_000,
            max_cost_per_day: 15_000_000,
            prices: Vec::new(),
            fallbacks: Vec::new(),
            trusted_sites: Vec::new(),
            shop_hosts: Vec::new(),
            own_listing_url: None,
            own_listing_path_tail: Vec::new(),
            attribution_keys: Vec::new(),
            language: "English".to_owned(),
            providers: ProviderIds::default(),
            normalise_version: psm_core::mechanisms::name_key::RESEARCH_NORMALISE_VERSION,
            fingerprint_version: crate::rules::fingerprint::RESEARCH_FINGERPRINT_VERSION,
            instructions_version: INSTRUCTIONS_VERSION,
        }
    }
}

impl ResearchConfig {
    pub fn stage(&self, stage: Stage) -> StageConfig {
        self.stages.get(stage.as_str()).cloned().unwrap_or_default()
    }

    pub fn price_table(&self) -> PriceTable {
        PriceTable {
            prices: self
                .prices
                .iter()
                .map(|p| Price {
                    provider: p.provider.clone(),
                    model: p.model.clone(),
                    unit: p.unit.clone(),
                    micro_usd_per_million: p.micro_usd_per_million,
                })
                .collect(),
            fallbacks: self
                .fallbacks
                .iter()
                .map(|f| Fallback {
                    provider: f.provider.clone(),
                    model: f.model.clone(),
                    micro_usd: f.micro_usd,
                })
                .collect(),
        }
    }

    pub fn own_listing(&self) -> OwnListing {
        OwnListing {
            url: self.own_listing_url.clone(),
            path_tail: self.own_listing_path_tail.clone(),
        }
    }

    pub fn from_snapshot(text: Option<&str>) -> psm_core::Result<Self> {
        match text {
            Some(text) => serde_json::from_str(text)
                .map_err(|e| psm_core::Error::InvalidData(format!("settings snapshot: {e}"))),
            None => Ok(Self::default()),
        }
    }
}

/// One price row of `research.prices` (decimal strings, MC-8): a per-unit price, a fallback per
/// call, or both, for one `(provider, model)` (`model` is `""` for non-model providers).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PriceRow {
    pub provider: String,
    pub model: String,
    pub unit: Option<String>,
    pub usd_per_million: Option<String>,
    pub fallback_call_usd: Option<String>,
}

/// Call timeouts in seconds (MC-A9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CallTimeouts {
    pub model: u32,
    pub other: u32,
}

impl Default for CallTimeouts {
    fn default() -> Self {
        Self {
            model: 180,
            other: 60,
        }
    }
}

/// The `research` slot of the settings file (`master_plan_research.md` §6.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ResearchSettings {
    pub auto: bool,
    pub quiet_seconds: i64,
    pub source_wait_seconds: i64,
    /// The shop's designer field (08A §6: the shop's designer and manufacturer keys; the
    /// connected shop has no manufacturer field).
    pub attribution_keys: Vec<String>,
    pub trusted_sites: Vec<String>,
    pub max_pages: u32,
    pub stages: BTreeMap<String, StageConfig>,
    pub max_cost_per_run: String,
    pub max_cost_per_day: String,
    pub prices: Vec<PriceRow>,
    pub language: String,
    pub record_days: i64,
    pub call_timeout_seconds: CallTimeouts,
    pub providers: ProviderIds,
}

impl Default for ResearchSettings {
    fn default() -> Self {
        Self {
            auto: true,
            quiet_seconds: 1,
            source_wait_seconds: 120,
            attribution_keys: vec!["beyo.designer".to_owned()],
            trusted_sites: Vec::new(),
            max_pages: 10,
            stages: ResearchConfig::default().stages,
            max_cost_per_run: "0.50".to_owned(),
            max_cost_per_day: "15".to_owned(),
            prices: Vec::new(),
            language: "English".to_owned(),
            record_days: 30,
            call_timeout_seconds: CallTimeouts::default(),
            providers: ProviderIds::default(),
        }
    }
}

impl ResearchSettings {
    /// Reads the slot tolerantly: a missing slot or field takes its default, and a field an older
    /// (or damaged) file holds in another shape is ignored with a warning.
    pub fn from_slot(slot: Option<&serde_json::Value>) -> Self {
        let Some(serde_json::Value::Object(given)) = slot else {
            return Self::default();
        };
        if let Ok(settings) =
            serde_json::from_value::<Self>(serde_json::Value::Object(given.clone()))
        {
            return settings;
        }
        let mut merged = serde_json::to_value(Self::default()).unwrap_or_default();
        for (key, value) in given {
            let mut trial = merged.clone();
            trial[key] = value.clone();
            if serde_json::from_value::<Self>(trial.clone()).is_ok() {
                merged = trial;
            } else {
                tracing::warn!(key = %key, "research setting has an unexpected shape; default used");
            }
        }
        serde_json::from_value(merged).unwrap_or_default()
    }

    pub fn stage(&self, stage: Stage) -> StageConfig {
        self.stages.get(stage.as_str()).cloned().unwrap_or_default()
    }

    /// Fallback per `(provider, model)` as micro-USD, for the pairs whose text parses.
    pub fn fallbacks(&self) -> Vec<FallbackSnapshot> {
        self.prices
            .iter()
            .filter_map(|row| {
                Some(FallbackSnapshot {
                    provider: row.provider.clone(),
                    model: row.model.clone(),
                    micro_usd: crate::rules::cost::parse_micro_usd(
                        row.fallback_call_usd.as_deref()?,
                    )?,
                })
            })
            .collect()
    }

    pub fn unit_prices(&self) -> Vec<PriceSnapshot> {
        self.prices
            .iter()
            .filter_map(|row| {
                Some(PriceSnapshot {
                    provider: row.provider.clone(),
                    model: row.model.clone(),
                    unit: row.unit.clone()?,
                    micro_usd_per_million: crate::rules::cost::parse_micro_usd(
                        row.usd_per_million.as_deref()?,
                    )?,
                })
            })
            .collect()
    }
}

impl ResearchConfig {
    /// The snapshot a run starts with: the live settings plus the shop facts of the item.
    pub fn from_settings(settings: &ResearchSettings, shop: &crate::shop::ShopFacts) -> Self {
        let defaults = ResearchSettings::default();
        let money = |text: &str, default: &str| {
            crate::rules::cost::parse_micro_usd(text)
                .or_else(|| crate::rules::cost::parse_micro_usd(default))
                .unwrap_or(0)
        };
        Self {
            stages: settings.stages.clone(),
            max_pages: settings.max_pages,
            max_cost_per_run: money(&settings.max_cost_per_run, &defaults.max_cost_per_run),
            max_cost_per_day: money(&settings.max_cost_per_day, &defaults.max_cost_per_day),
            prices: settings.unit_prices(),
            fallbacks: settings.fallbacks(),
            trusted_sites: settings.trusted_sites.clone(),
            shop_hosts: shop.hosts.clone(),
            own_listing_url: shop.own_listing.url.clone(),
            own_listing_path_tail: shop.own_listing.path_tail.clone(),
            attribution_keys: settings.attribution_keys.clone(),
            language: settings.language.clone(),
            providers: settings.providers.clone(),
            ..Self::default()
        }
    }
}

/// Refuses a research settings value (whole or a patch of top-level keys) that holds a cap or a
/// price that is not a decimal string with at most 6 fractional digits, `record_days` below 2, a
/// negative `quiet_seconds` or `source_wait_seconds`, or a long edge that is not a positive
/// integer. The text names the field.
pub fn validate(value: &serde_json::Value) -> Result<(), String> {
    use serde_json::Value as Json;
    let Some(object) = value.as_object() else {
        return Err("research settings must be an object".to_owned());
    };
    let money = |field: &str, value: &Json| -> Result<(), String> {
        match value.as_str().and_then(crate::rules::cost::parse_micro_usd) {
            Some(_) => Ok(()),
            None => Err(format!(
                "{field} must be an amount in USD like 0.50, with at most 6 decimals"
            )),
        }
    };
    for field in ["max_cost_per_run", "max_cost_per_day"] {
        if let Some(value) = object.get(field) {
            money(field, value)?;
        }
    }
    if let Some(prices) = object.get("prices") {
        let Some(rows) = prices.as_array() else {
            return Err("prices must be a list".to_owned());
        };
        for row in rows {
            for field in ["usd_per_million", "fallback_call_usd"] {
                if let Some(value) = row.get(field).filter(|v| !v.is_null()) {
                    money(field, value)?;
                }
            }
        }
    }
    if let Some(days) = object.get("record_days")
        && !days.as_i64().is_some_and(|d| d >= 2)
    {
        return Err("record_days must be a whole number of at least 2".to_owned());
    }
    for field in ["quiet_seconds", "source_wait_seconds"] {
        if let Some(seconds) = object.get(field)
            && !seconds.as_i64().is_some_and(|s| s >= 0)
        {
            return Err(format!("{field} must be a whole number of 0 or more"));
        }
    }
    if let Some(stages) = object.get("stages") {
        let Some(stages) = stages.as_object() else {
            return Err("stages must be an object".to_owned());
        };
        for (name, stage) in stages {
            if let Some(edge) = stage.get("image_long_edge")
                && !edge
                    .as_u64()
                    .is_some_and(|e| e > 0 && e <= u64::from(u32::MAX))
            {
                return Err(format!(
                    "stages.{name}.image_long_edge must be a whole number above 0"
                ));
            }
        }
    }
    Ok(())
}
