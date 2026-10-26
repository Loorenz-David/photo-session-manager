use crate::domain::{HealthItem, HealthReport};
use crate::{
    Result,
    domain::attribute::CoreType,
    ports::{commerce::CommerceAdapter, store::Store},
};
use std::sync::Mutex;

#[derive(Default)]
pub struct HealthService {
    items: Mutex<Vec<HealthItem>>,
}

impl HealthService {
    /// ch12 §Health, 09A MC-23/MC-24: whether a Shopify token can be obtained (detail = the
    /// MC-23 message) and whether the condition-photos field exists in the stored definitions
    /// as the one `media_list` field. Without an integration both items are absent.
    pub fn check_integration(
        &self,
        store: &dyn Store,
        commerce: Option<&dyn CommerceAdapter>,
    ) -> Result<()> {
        let Some(adapter) = commerce else {
            self.clear("integration_token_invalid");
            self.clear("condition_field_missing");
            return Ok(());
        };
        match adapter.check_token() {
            Ok(()) => self.set_ok("integration_token_invalid", "Shopify token obtainable"),
            Err(error) => self.set("integration_token_invalid", error.to_string()),
        }
        let key = adapter.condition_photos_key().to_owned();
        let mut definitions = Vec::new();
        store.transaction(&mut |tx| {
            definitions = tx.attribute_definitions("shopify")?;
            Ok(())
        })?;
        match definitions.iter().find(|definition| definition.key == key) {
            None => self.set(
                "condition_field_missing",
                format!("Condition photos field {key} not found in the shop"),
            ),
            Some(definition) if definition.core_type != CoreType::MediaList => self.set(
                "condition_field_missing",
                format!("Condition photos field {key} is not a file list"),
            ),
            Some(_) => self.set_ok(
                "condition_field_missing",
                format!("Condition photos field {key} found"),
            ),
        }
        Ok(())
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set(&self, code: &str, detail: impl Into<String>) {
        if let Ok(mut items) = self.items.lock() {
            items.retain(|item| item.code != code || code == "recipe_missing");
            items.push(HealthItem {
                code: code.into(),
                ok: false,
                detail: detail.into(),
            });
        }
    }
    pub fn set_ok(&self, code: &str, detail: impl Into<String>) {
        if let Ok(mut items) = self.items.lock() {
            items.retain(|item| item.code != code);
            items.push(HealthItem {
                code: code.into(),
                ok: true,
                detail: detail.into(),
            });
        }
    }
    pub fn clear(&self, code: &str) {
        if let Ok(mut items) = self.items.lock() {
            items.retain(|item| item.code != code);
        }
    }
    pub fn has(&self, code: &str) -> bool {
        self.items
            .lock()
            .is_ok_and(|items| items.iter().any(|i| i.code == code && !i.ok))
    }
    pub fn report(
        &self,
        data_dir: &str,
        hooks_compiled: bool,
        nudge_listening: bool,
    ) -> HealthReport {
        let mut report = HealthReport {
            items: vec![
                HealthItem {
                    code: "data_dir".into(),
                    ok: true,
                    detail: data_dir.into(),
                },
                HealthItem {
                    code: "hooks_compiled".into(),
                    ok: hooks_compiled,
                    detail: if hooks_compiled {
                        "Compiled"
                    } else {
                        "Unavailable"
                    }
                    .into(),
                },
                HealthItem {
                    code: "nudge_listening".into(),
                    ok: nudge_listening,
                    detail: if nudge_listening {
                        "Listening"
                    } else {
                        "Unavailable"
                    }
                    .into(),
                },
            ],
        };
        if let Ok(extra) = self.items.lock() {
            report.items.extend(extra.iter().cloned());
        }
        report
    }
}
