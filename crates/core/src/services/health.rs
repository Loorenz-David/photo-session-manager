use crate::domain::{HealthItem, HealthReport};
use std::sync::Mutex;

#[derive(Default)]
pub struct HealthService {
    items: Mutex<Vec<HealthItem>>,
}

impl HealthService {
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
