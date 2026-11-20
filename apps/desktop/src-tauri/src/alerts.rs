//! C2 (book 09B §9; `master_plan_conversations.md` §4 Event, §6): the native side of a
//! conversations notice: the chime (played by the app, not the page) and the Mac banner (through
//! the notification plugin, used from Rust only). The event bridge decides when; this seam does it.

use std::path::PathBuf;
use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::NotificationExt;

/// The banner's title.
pub const BANNER_TITLE: &str = "Session Manager";

pub trait Alerts: Send + Sync {
    /// One chime; returns at once.
    fn chime(&self);
    /// One Mac banner.
    fn banner(&self, title: &str, body: &str);
}

/// The real alerts: `chime.wav` in the data folder played by macOS, banners by the plugin.
pub struct SystemAlerts<R: Runtime> {
    app: AppHandle<R>,
    data_dir: PathBuf,
}

impl<R: Runtime> SystemAlerts<R> {
    pub fn new(app: AppHandle<R>, data_dir: PathBuf) -> Self {
        Self { app, data_dir }
    }
}

impl<R: Runtime> Alerts for SystemAlerts<R> {
    fn chime(&self) {
        psm_infra::chime::play(&self.data_dir);
    }
    fn banner(&self, title: &str, body: &str) {
        if let Err(error) = self
            .app
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show()
        {
            tracing::warn!(target: "psm::conversations", %error, "banner not shown");
        }
    }
}

/// Alerts that do nothing (tests that do not look at alerts).
#[cfg(test)]
pub struct NoAlerts;

#[cfg(test)]
impl Alerts for NoAlerts {
    fn chime(&self) {}
    fn banner(&self, _title: &str, _body: &str) {}
}

/// The banner bodies of one event: the toast lines in order, then `and <more> more` when `more` > 0
/// (09B §5.5: the banners use the same words as the window's toasts).
pub fn banners(toasts: &[String], more: usize) -> Vec<String> {
    let mut lines = toasts.to_vec();
    if more > 0 {
        lines.push(format!("and {more} more"));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c2_c19c_banner_bodies_are_the_lines_then_the_rest_counted() {
        let three = [
            "New question · BEYO-1 — Condition".to_owned(),
            "New reply · BEYO-2".to_owned(),
            "New reply · BEYO-3".to_owned(),
        ];
        assert_eq!(
            banners(&three, 2),
            [
                "New question · BEYO-1 — Condition",
                "New reply · BEYO-2",
                "New reply · BEYO-3",
                "and 2 more"
            ]
        );
        let one = ["New reply · BEYO-9".to_owned()];
        assert_eq!(banners(&one, 0), ["New reply · BEYO-9"]);
    }
}
