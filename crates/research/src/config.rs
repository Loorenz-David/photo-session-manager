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
