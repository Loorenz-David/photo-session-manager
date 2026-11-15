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
pub const INSTRUCTIONS_VERSION: u32 = 2;

/// How many photos stage 1 looks at by default (R12·U1).
pub const DEFAULT_OBSERVE_MAX_PHOTOS: u32 = 8;

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

/// The default model adapter id (R8·U1; the provider name in R1's captures).
pub const DEFAULT_MODEL_PROVIDER: &str = "openai";
/// The default visual-search adapter id: Google Lens through SerpApi (evidence "Owner picks" 1).
pub const DEFAULT_VISUAL_SEARCH_PROVIDER: &str = "serpapi_lens";
/// The default page reader id: plain HTTP download (evidence "Owner picks" 2).
pub const DEFAULT_PAGE_READER: &str = "page_http";

/// The adapter ids a run calls (the ids the price table is keyed by). A saved `""` means the
/// default ([`ProviderIds::resolved`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderIds {
    pub model: String,
    pub visual_search: String,
    pub page_reader: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DetailedCondition {
    pub key: String,
    pub levels: Vec<ConditionLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionLevel {
    pub level: String,
    pub choice: Option<String>,
}

impl Default for ProviderIds {
    fn default() -> Self {
        Self {
            model: DEFAULT_MODEL_PROVIDER.to_owned(),
            visual_search: DEFAULT_VISUAL_SEARCH_PROVIDER.to_owned(),
            page_reader: DEFAULT_PAGE_READER.to_owned(),
        }
    }
}

impl ProviderIds {
    /// The ids with every saved `""` (or blank) turned into its default; any other id is kept
    /// as it is (an unknown id gives no adapter).
    pub fn resolved(&self) -> Self {
        let pick = |given: &str, default: &str| {
            if given.trim().is_empty() {
                default.to_owned()
            } else {
                given.to_owned()
            }
        };
        Self {
            model: pick(&self.model, DEFAULT_MODEL_PROVIDER),
            visual_search: pick(&self.visual_search, DEFAULT_VISUAL_SEARCH_PROVIDER),
            page_reader: pick(&self.page_reader, DEFAULT_PAGE_READER),
        }
    }
}

/// The run's settings snapshot (MC-A6 start transaction; MC-A3, MC-A8 snapshots).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ResearchConfig {
    pub stages: BTreeMap<String, StageConfig>,
    pub max_pages: u32,
    pub observe_max_photos: u32,
    pub max_cost_per_run: i64,
    pub max_cost_per_day: i64,
    pub prices: Vec<PriceSnapshot>,
    pub fallbacks: Vec<FallbackSnapshot>,
    pub trusted_sites: Vec<String>,
    pub shop_hosts: Vec<String>,
    pub own_listing_url: Option<String>,
    pub own_listing_path_tail: Vec<String>,
    pub attribution_keys: Vec<String>,
    pub excluded_keys: Vec<String>,
    pub extra_shop_hosts: Vec<String>,
    pub detailed_condition: DetailedCondition,
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
            observe_max_photos: DEFAULT_OBSERVE_MAX_PHOTOS,
            max_cost_per_run: 500_000,
            max_cost_per_day: 15_000_000,
            prices: Vec::new(),
            fallbacks: Vec::new(),
            trusted_sites: Vec::new(),
            shop_hosts: Vec::new(),
            own_listing_url: None,
            own_listing_path_tail: Vec::new(),
            attribution_keys: Vec::new(),
            excluded_keys: Vec::new(),
            extra_shop_hosts: Vec::new(),
            detailed_condition: DetailedCondition::default(),
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
    pub source_wait_seconds: i64,
    /// The shop's designer and maker fields (08A §6), as the owner points them; empty by default
    /// (no field is treated as attribution until chosen).
    pub attribution_keys: Vec<String>,
    pub excluded_keys: Vec<String>,
    pub extra_shop_hosts: Vec<String>,
    pub detailed_condition: DetailedCondition,
    pub trusted_sites: Vec<String>,
    pub max_pages: u32,
    /// At most this many photos go to the one stage 1 call (Main photos first by position).
    pub observe_max_photos: u32,
    pub stages: BTreeMap<String, StageConfig>,
    pub max_cost_per_run: String,
    pub max_cost_per_day: String,
    pub prices: Vec<PriceRow>,
    pub language: String,
    pub record_days: i64,
    pub call_timeout_seconds: CallTimeouts,
    pub providers: ProviderIds,
}

/// The shipped model per stage (evidence `research_providers.md` "Owner pick of models",
/// 2026-11-15): `(stage, model, effort)`. Find calls no model (it is the visual search).
const DEFAULT_STAGE_MODELS: [(Stage, &str, &str); 6] = [
    (Stage::Observe, "gpt-6.1-sol", "low"),
    (Stage::Read, "gpt-6-luna", "low"),
    (Stage::Compare, "gpt-6.1-terra", "low"),
    (Stage::Labels, "gpt-6.1-sol", "low"),
    (Stage::Condition, "gpt-6.1-terra", "high"),
    (Stage::Decide, "gpt-6.1-terra", "low"),
];

/// OpenAI prices of the shipped models, USD per 1M tokens: input, cached input, cache write
/// (30 min), output (evidence "Candidate models", pricing page read 2026-11-12, Standard tier).
const DEFAULT_MODEL_PRICES: [(&str, &str, &str, &str, &str); 3] = [
    ("gpt-6.1-sol", "2.00", "0.10", "2.50", "10.00"),
    ("gpt-6.1-terra", "1.00", "0.05", "1.25", "5.00"),
    ("gpt-6-luna", "0.10", "0.01", "0.125", "0.50"),
];

/// The web search tool: USD 10 per 1,000 calls (evidence "Model provider: OpenAI API").
const WEB_SEARCH_USD_PER_MILLION: &str = "10000";
/// A model call whose usage is unknown (the acceptance measurement's worst case per call).
const MODEL_FALLBACK_CALL_USD: &str = "0.05";
/// SerpApi Google Lens: USD 25 per 1,000 searches (cheapest paid plan, evidence "Reverse image
/// search", read 2026-11-12); an upload costs no credit (V7).
const LENS_USD_PER_MILLION: &str = "25000";
const LENS_FALLBACK_CALL_USD: &str = "0.025";
/// A plain page download costs nothing; its fallback is the smallest positive amount (1 µUSD).
const PAGE_FALLBACK_CALL_USD: &str = "0.000001";

fn default_stages() -> BTreeMap<String, StageConfig> {
    let mut stages = ResearchConfig::default().stages;
    for (stage, model, effort) in DEFAULT_STAGE_MODELS {
        if let Some(entry) = stages.get_mut(stage.as_str()) {
            entry.model = model.to_owned();
            entry.effort = effort.to_owned();
        }
    }
    stages
}

fn default_prices() -> Vec<PriceRow> {
    let row = |provider: &str, model: &str, unit: &str, usd: &str| PriceRow {
        provider: provider.to_owned(),
        model: model.to_owned(),
        unit: Some(unit.to_owned()),
        usd_per_million: Some(usd.to_owned()),
        fallback_call_usd: None,
    };
    let fallback = |provider: &str, model: &str, usd: &str| PriceRow {
        provider: provider.to_owned(),
        model: model.to_owned(),
        unit: None,
        usd_per_million: None,
        fallback_call_usd: Some(usd.to_owned()),
    };
    let mut prices = Vec::new();
    for (model, input, cached, write, output) in DEFAULT_MODEL_PRICES {
        for (unit, usd) in [
            ("input_token", input),
            ("cached_input_token", cached),
            ("cache_write_token", write),
            ("output_token", output),
            ("web_search_call", WEB_SEARCH_USD_PER_MILLION),
        ] {
            prices.push(row(DEFAULT_MODEL_PROVIDER, model, unit, usd));
        }
        prices.push(fallback(
            DEFAULT_MODEL_PROVIDER,
            model,
            MODEL_FALLBACK_CALL_USD,
        ));
    }
    prices.push(row(
        DEFAULT_VISUAL_SEARCH_PROVIDER,
        "",
        "image_search_call",
        LENS_USD_PER_MILLION,
    ));
    prices.push(fallback(
        DEFAULT_VISUAL_SEARCH_PROVIDER,
        "",
        LENS_FALLBACK_CALL_USD,
    ));
    prices.push(row(DEFAULT_PAGE_READER, "", "page_fetch", "0"));
    prices.push(fallback(DEFAULT_PAGE_READER, "", PAGE_FALLBACK_CALL_USD));
    prices
}

/// The owner's detailed-condition table (answer A to R11's card, 2026-11-15): the worst condition
/// level None or Minor gives Very Good, Moderate gives Good, Major gives Worn; Excellent is never
/// written. N/A is not a level of wear and counts for nothing. Choice strings as the shop defines
/// `custom.detailed_condition`.
fn default_detailed_condition() -> DetailedCondition {
    let level = |level: &str, choice: Option<&str>| ConditionLevel {
        level: level.to_owned(),
        choice: choice.map(str::to_owned),
    };
    const VERY_GOOD: &str =
        "Very Good - This vintage item has no defects, but it may show slight traces of use.";
    const GOOD: &str =
        "Good - This vintage item may have some wear such as scratches and other signs of ageing";
    const WORN: &str = "Worn - This vintage item has visible damages, defects, or small missing \
        parts. It may be in need of restoration.";
    DetailedCondition {
        key: "custom.detailed_condition".to_owned(),
        levels: vec![
            level("N/A", None),
            level("None", Some(VERY_GOOD)),
            level("Minor", Some(VERY_GOOD)),
            level("Moderate", Some(GOOD)),
            level("Major", Some(WORN)),
        ],
    }
}

impl Default for ResearchSettings {
    fn default() -> Self {
        Self {
            source_wait_seconds: 120,
            // Owner 2026-11-15: no field is attribution until the owner points at one.
            attribution_keys: Vec::new(),
            excluded_keys: Vec::new(),
            extra_shop_hosts: Vec::new(),
            detailed_condition: default_detailed_condition(),
            trusted_sites: Vec::new(),
            max_pages: 10,
            observe_max_photos: DEFAULT_OBSERVE_MAX_PHOTOS,
            stages: default_stages(),
            max_cost_per_run: "0.50".to_owned(),
            max_cost_per_day: "15".to_owned(),
            prices: default_prices(),
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
            observe_max_photos: settings.observe_max_photos.max(1),
            max_cost_per_run: money(&settings.max_cost_per_run, &defaults.max_cost_per_run),
            max_cost_per_day: money(&settings.max_cost_per_day, &defaults.max_cost_per_day),
            prices: settings.unit_prices(),
            fallbacks: settings.fallbacks(),
            trusted_sites: settings.trusted_sites.clone(),
            shop_hosts: {
                let mut hosts = shop.hosts.clone();
                for typed in &settings.extra_shop_hosts {
                    if let Ok(host) = crate::rules::sites::TrustedSites::parse_entry(typed)
                        && !hosts.contains(&host)
                    {
                        hosts.push(host);
                    }
                }
                hosts
            },
            own_listing_url: shop.own_listing.url.clone(),
            own_listing_path_tail: shop.own_listing.path_tail.clone(),
            attribution_keys: settings.attribution_keys.clone(),
            excluded_keys: settings.excluded_keys.clone(),
            extra_shop_hosts: settings.extra_shop_hosts.clone(),
            detailed_condition: settings.detailed_condition.clone(),
            language: settings.language.clone(),
            providers: settings.providers.resolved(),
            ..Self::default()
        }
    }
}

/// Refuses a research settings value (whole or a patch of top-level keys) that holds a cap or a
/// price that is not a decimal string with at most 6 fractional digits, `record_days` below 2, a
/// negative `source_wait_seconds`, an `observe_max_photos` below 1, or a long edge that is not a positive
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
    if let Some(limit) = object.get("observe_max_photos")
        && !limit
            .as_u64()
            .is_some_and(|n| n >= 1 && n <= u64::from(u32::MAX))
    {
        return Err("observe_max_photos must be a whole number of at least 1".into());
    }
    if let Some(seconds) = object.get("source_wait_seconds")
        && !seconds.as_i64().is_some_and(|s| s >= 0)
    {
        return Err("source_wait_seconds must be a whole number of 0 or more".into());
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
    let settings = ResearchSettings::from_slot(Some(value));
    for typed in &settings.extra_shop_hosts {
        let host = crate::rules::sites::TrustedSites::parse_entry(typed)
            .map_err(|reason| format!("extra shop address {typed:?}: {reason}"))?;
        for trusted in &settings.trusted_sites {
            let trusted_host = crate::rules::sites::TrustedSites::parse_entry(trusted)
                .unwrap_or_else(|_| trusted.clone());
            if crate::rules::sites::overlaps(&host, &trusted_host) {
                return Err(format!(
                    "extra shop address {typed:?} is already a trusted site"
                ));
            }
        }
    }
    Ok(())
}
