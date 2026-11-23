//! L3 of `LabelsService` (book 09C §6; `master_plan_labels.md` §5 MC-L2): the check (what
//! generating would do, chunked and shown as one), the preview picture of one label, and the one
//! kept check. Nothing here is stored (M-L1): the check lives in the service's memory and is gone
//! at quit. Platform-neutral (H7): the provider is a port, the files are a port.

use super::labels::{LabelsFailure, LabelsService};
use crate::{
    Error,
    domain::label::{LabelCheck, LabelPlan, LabelsError, Unresolved},
    mechanisms::labels::{RUN_CHUNK, chunks, group_check, merge_plans},
};
use std::{collections::HashMap, path::PathBuf};

const LOG: &str = "psm::labels";

type Answer<T> = std::result::Result<T, LabelsFailure>;

/// MC-L2.3: the one check the service keeps (memory only).
pub struct KeptCheck {
    id: String,
    plan: LabelPlan,
    /// The preview picture made for a product of the check (MC-L2.5).
    previews: HashMap<String, PathBuf>,
}

/// The reason text of a file failure (projection r0, D2): the inner `io::Error`'s text for an
/// I/O error, else the error's own text.
pub(super) fn reason(error: &Error) -> String {
    match error {
        Error::Io(inner) => inner.to_string(),
        other => other.to_string(),
    }
}

/// A preview picture's name: the check id and the product id, keeping only characters that
/// cannot leave the previews folder.
fn picture_name(check_id: &str, product_id: &str) -> String {
    let safe: String = product_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    format!("{check_id}-{safe}")
}

impl LabelsService {
    fn kept(&self) -> std::sync::MutexGuard<'_, Option<KeptCheck>> {
        self.kept.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// D10: the kept check is dropped and its preview pictures removed.
    fn drop_check(&self) {
        *self.kept() = None;
        if let Err(error) = self.files.clear_previews() {
            tracing::warn!(target: LOG, %error, "label previews not cleared");
        }
    }

    /// MC-L2.1–.4 (a press; changes nothing at the provider): the products without repeats
    /// (first kept), checked by the provider in chunks of `RUN_CHUNK` and merged; one templates
    /// read names each group's product types (a failed read leaves them empty). The new check
    /// replaces the kept one and clears the preview pictures. A chunk the provider found none of
    /// lists its products as not found; when every chunk does, `NoProducts`. Any other provider
    /// error is answered and nothing is kept (D10: an earlier check is dropped too).
    pub fn check(&self, product_ids: &[String]) -> Answer<LabelCheck> {
        let mut ids: Vec<String> = Vec::with_capacity(product_ids.len());
        for id in product_ids {
            if !ids.contains(id) {
                ids.push(id.clone());
            }
        }
        if ids.is_empty() {
            return Err(Error::InvalidValue("no products to check".into()).into());
        }
        let store = self.gate()?;
        let ranges = chunks(ids.len(), RUN_CHUNK);
        let mut answers = Vec::with_capacity(ranges.len());
        for (index, range) in ranges.iter().enumerate() {
            let chunk = &ids[range.clone()];
            match self.learn(self.provider.plan(&store, chunk)) {
                Ok(plan) => answers.push((index, Ok(plan))),
                Err(LabelsFailure::Provider(LabelsError::NoProducts)) => answers.push((
                    index,
                    Err(chunk
                        .iter()
                        .map(|id| Unresolved {
                            product_id: Some(id.clone()),
                            sku: None,
                            message: String::new(),
                        })
                        .collect()),
                )),
                Err(failure) => {
                    self.drop_check();
                    return Err(failure);
                }
            }
        }
        if answers.iter().all(|(_, answer)| answer.is_err()) {
            self.drop_check();
            return Err(LabelsFailure::Provider(LabelsError::NoProducts));
        }
        let templates = self
            .learn(self.provider.templates(&store))
            .unwrap_or_default();
        let plan = merge_plans(answers);
        let groups = group_check(&plan, &templates);
        let id = uuid::Uuid::new_v4().to_string();
        tracing::info!(
            target: LOG,
            "labels.check products={} chunks={} ready={} failing={}",
            ids.len(),
            ranges.len(),
            plan.summary.ready,
            plan.summary.failing
        );
        let check = LabelCheck {
            id: id.clone(),
            groups,
            unresolved: plan.unresolved.clone(),
            summary: plan.summary,
            sizes: plan.sizes.clone(),
        };
        self.drop_check();
        *self.kept() = Some(KeptCheck {
            id,
            plan,
            previews: HashMap::new(),
        });
        Ok(check)
    }

    /// MC-L2.5: the preview picture of one product of the kept check (else `NotFound`, no
    /// request). The first ask fetches the provider's PDF and turns its page into a picture; a
    /// later ask answers that picture without a request while the file exists. A picture that
    /// cannot be made is `PreviewFailed`; a preview never blocks Generate.
    pub fn preview(&self, check_id: &str, product_id: &str) -> Answer<PathBuf> {
        {
            let kept = self.kept();
            let check = kept
                .as_ref()
                .filter(|c| c.id == check_id)
                .ok_or(LabelsFailure::NotFound)?;
            if !check.plan.items.iter().any(|i| i.product_id == product_id) {
                return Err(LabelsFailure::NotFound);
            }
            if let Some(path) = check.previews.get(product_id).filter(|p| p.is_file()) {
                return Ok(path.clone());
            }
        }
        let store = self.gate()?;
        let pdf = self.learn(self.provider.preview(&store, product_id))?;
        let path = self
            .files
            .raster(&pdf, &picture_name(check_id, product_id))
            .map_err(|error| LabelsFailure::PreviewFailed(reason(&error)))?;
        if let Some(check) = self.kept().as_mut().filter(|c| c.id == check_id) {
            check.previews.insert(product_id.to_owned(), path.clone());
        }
        Ok(path)
    }
}
