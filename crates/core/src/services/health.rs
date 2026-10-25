use crate::domain::{HealthItem, HealthReport};

pub struct HealthService;

impl HealthService {
    pub fn report(data_dir: &str, hooks_compiled: bool, nudge_listening: bool) -> HealthReport {
        HealthReport {
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
        }
    }
}
