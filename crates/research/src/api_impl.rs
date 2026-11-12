//! `impl ResearchApi for ResearchService` (the surface frozen in R2·U2): status with its derived
//! counts, Run again, keys into the secrets port, trusted-site editing and settings validation.

use std::path::{Path, PathBuf};

use psm_core::{Error, Result};
use serde_json::Value as Json;

use crate::{
    api::{ExportSelection, ResearchApi, ResearchStatus, SecretsInput, SecretsState},
    config::{ResearchSettings, validate},
    rules::sites::TrustedSites,
    service::{MODEL_ACCOUNT, READER_ACCOUNT, ResearchService, SEARCH_ACCOUNT},
};

impl ResearchService {
    /// The shop's hosts (refusals of trusted entries name them).
    fn shop_hosts(&self) -> Result<Vec<String>> {
        match &self.shop {
            Some(shop) => Ok(shop.facts(None)?.hosts),
            None => Ok(Vec::new()),
        }
    }

    /// The live `research` value with `trusted_sites` replaced: what the caller saves.
    fn with_sites(&self, sites: &TrustedSites) -> Json {
        let mut value = match (self.settings)() {
            Some(Json::Object(map)) => Json::Object(map),
            _ => serde_json::to_value(ResearchSettings::default()).unwrap_or_default(),
        };
        value["trusted_sites"] = Json::from(sites.entries().to_vec());
        value
    }

    fn sites(&self) -> TrustedSites {
        TrustedSites::new(self.settings().trusted_sites)
    }

    fn is_set(&self, account: &str) -> Result<bool> {
        Ok(self
            .secrets
            .get(account)?
            .is_some_and(|v| !v.trim().is_empty()))
    }
}

impl ResearchApi for ResearchService {
    fn status(&self, session_id: i64) -> Result<ResearchStatus> {
        ResearchService::status(self, session_id)
    }

    fn run(&self, session_id: i64) -> Result<()> {
        self.run_again(session_id)
    }

    fn session_changed(&self, session_id: i64) {
        ResearchService::session_changed(self, session_id);
    }

    fn settings_changed(&self) {
        ResearchService::settings_changed(self);
    }

    fn validate_settings(&self, settings: &Json) -> Result<()> {
        validate(settings).map_err(Error::InvalidData)
    }

    /// Writes each given key under its account (an empty key removes it); never returns one.
    fn secrets_set(&self, input: SecretsInput) -> Result<()> {
        for (account, value) in [
            (MODEL_ACCOUNT, input.model_key),
            (SEARCH_ACCOUNT, input.search_key),
            (READER_ACCOUNT, input.reader_key),
        ] {
            match value.as_deref().map(str::trim) {
                Some("") => self.secrets.delete(account)?,
                Some(key) => self.secrets.set(account, key)?,
                None => {}
            }
        }
        ResearchService::settings_changed(self);
        Ok(())
    }

    fn secrets_state(&self) -> Result<SecretsState> {
        Ok(SecretsState {
            model: self.is_set(MODEL_ACCOUNT)?,
            search: self.is_set(SEARCH_ACCOUNT)?,
            reader: self.is_set(READER_ACCOUNT)?,
        })
    }

    fn site_add(&self, site: &str) -> Result<Json> {
        let mut sites = self.sites();
        sites
            .add(site, &self.shop_hosts()?)
            .map_err(Error::InvalidData)?;
        Ok(self.with_sites(&sites))
    }

    fn site_remove(&self, site: &str) -> Result<Json> {
        let mut sites = self.sites();
        sites.remove(site);
        Ok(self.with_sites(&sites))
    }

    fn sites_reorder(&self, sites: &[String]) -> Result<Json> {
        let mut current = self.sites();
        current.reorder(sites).map_err(Error::InvalidData)?;
        Ok(self.with_sites(&current))
    }

    /// Export arrives with retention (R7·U2).
    fn export(&self, _selection: ExportSelection, _dest: &Path) -> Result<PathBuf> {
        Err(Error::InvalidData(
            "research export is not available in this version".into(),
        ))
    }

    /// Retention arrives in R7·U2; nothing is purged yet.
    fn purge_expired(&self, _now_ms: i64) -> Result<()> {
        Ok(())
    }
}
