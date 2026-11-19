//! `impl ResearchApi for ResearchService` (the surface frozen in R2·U2): status with its derived
//! counts, Run again, keys into the secrets port, trusted-site editing and settings validation.

use std::path::{Path, PathBuf};

use psm_core::{Error, Result};
use serde_json::Value as Json;

use crate::{
    api::{
        ExportSelection, ResearchApi, ResearchStatus, SecretsInput, SecretsState, SessionResearch,
    },
    config::{ResearchSettings, validate},
    rules::sites::TrustedSites,
    service::{MODEL_ACCOUNT, READER_ACCOUNT, ResearchService, SEARCH_ACCOUNT},
};

impl ResearchService {
    /// The shop's hosts (refusals of trusted entries name them).
    fn shop_hosts(&self) -> Result<Vec<String>> {
        let mut hosts = match &self.shop {
            Some(shop) => shop.facts(None)?.hosts,
            None => Vec::new(),
        };
        for typed in &self.settings().extra_shop_hosts {
            if let Ok(host) = TrustedSites::parse_entry(typed)
                && !hosts.contains(&host)
            {
                hosts.push(host);
            }
        }
        Ok(hosts)
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

    fn overview(&self, session: Option<i64>) -> Result<Vec<SessionResearch>> {
        ResearchService::overview(self, session)
    }

    fn run(&self, session_id: i64) -> Result<()> {
        self.run_again(session_id)
    }

    fn session_changed(&self, session_id: i64) {
        ResearchService::session_changed(self, session_id);
    }

    fn arranged(&self, session_id: i64) -> Result<bool> {
        ResearchService::arranged(self, session_id)
    }

    fn settings_changed(&self) {
        ResearchService::settings_changed(self);
    }

    fn settings_reset(&self) -> Result<Json> {
        let live = self.settings();
        let reset = ResearchSettings {
            trusted_sites: live.trusted_sites,
            attribution_keys: live.attribution_keys,
            fill_keys: live.fill_keys,
            extra_shop_hosts: live.extra_shop_hosts,
            detailed_condition: live.detailed_condition,
            language: live.language,
            ..ResearchSettings::default()
        };
        serde_json::to_value(reset).map_err(|e| Error::InvalidData(e.to_string()))
    }

    fn validate_settings(&self, settings: &Json) -> Result<()> {
        let mut merged =
            serde_json::to_value(self.settings()).map_err(|e| Error::InvalidData(e.to_string()))?;
        if let Some(patch) = settings.as_object() {
            for (key, value) in patch {
                merged[key] = value.clone();
            }
        }
        validate(&merged).map_err(Error::InvalidData)
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

    /// MC-A11 Export research (R7·U2): the selected runs as one folder and one zip; the zip's path.
    fn export(&self, selection: ExportSelection, dest: &Path) -> Result<PathBuf> {
        ResearchService::export(self, selection, dest)
    }

    /// MC-A11 retention (R7·U2): call records older than `record_days` and their only images.
    fn purge_expired(&self, now_ms: i64) -> Result<()> {
        ResearchService::purge_expired(self, now_ms)
    }

    fn browse_matches(&self, session_id: i64) -> Result<Vec<crate::browse::VisualMatch>> {
        ResearchService::browse_matches(self, session_id)
    }

    fn browse_search(
        &self,
        session_id: i64,
        photo_id: i64,
    ) -> Result<Vec<crate::browse::VisualMatch>> {
        ResearchService::browse_search(self, session_id, photo_id)
    }

    fn browse_ask(&self, request: &crate::browse::AskRequest) -> Result<crate::browse::AskAnswer> {
        ResearchService::browse_ask(self, request)
    }
}

/// W1: the writer's surface, on the same service (`master_plan_writer.md` §4 "API").
impl crate::writer::api::WriterApi for ResearchService {
    fn note(&self, session_id: i64) -> Result<crate::writer::note::NoteView> {
        ResearchService::assemble(self, session_id)
    }

    fn look_again(&self, session_id: i64) -> Result<crate::writer::api::Requested> {
        ResearchService::look_again(self, session_id)
    }

    fn request(
        &self,
        session_id: i64,
        origin: crate::writer::run::Origin,
    ) -> Result<crate::writer::api::Requested> {
        ResearchService::writer_request(self, session_id, origin)
    }

    fn answer(&self, finding_id: i64, answer: &str) -> Result<crate::writer::api::Requested> {
        ResearchService::writer_answer(self, finding_id, answer)
    }

    fn dismiss(&self, finding_id: i64) -> Result<()> {
        ResearchService::writer_dismiss(self, finding_id)
    }

    fn status(&self, session_id: i64) -> Result<crate::writer::api::WriterStatus> {
        ResearchService::writer_status(self, session_id)
    }

    fn counts(&self) -> Result<Vec<crate::writer::run::FindingCount>> {
        ResearchService::writer_counts(self)
    }

    fn preview(
        &self,
        settings: &serde_json::Value,
        language: &str,
    ) -> Result<crate::writer::api::Preview> {
        ResearchService::writer_preview(self, settings, language)
    }
}
