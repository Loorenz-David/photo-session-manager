use super::{adapter::CaptureOneAdapter, recipes::RecipeSpec};
use psm_core::{
    Error, Result,
    domain::editing::{ProvisionInputs, ProvisionReport, StyleOutcome},
    mechanisms::canon::canon,
    ports::editing::EditingProvider,
};
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorFailure {
    NotInstalled,
    AutomationDenied,
}

pub fn classify_editor_error(message: &str) -> Option<EditorFailure> {
    if message.contains("Unable to find application") {
        Some(EditorFailure::NotInstalled)
    } else if message.contains("-1743") || message.contains("Not authorized to send Apple events") {
        Some(EditorFailure::AutomationDenied)
    } else {
        None
    }
}

pub fn editor_wait_poll(wait: Duration) -> Duration {
    Duration::from_millis(500).min(wait / 4)
}

fn session_file(folder: &Path) -> Result<Option<PathBuf>> {
    if !folder.is_dir() {
        return Ok(None);
    }
    let base = folder
        .file_name()
        .ok_or(Error::DocumentNotSession)?
        .to_string_lossy();
    let named = folder.join(format!("{base}.cosessiondb"));
    let mut found = fs::read_dir(folder)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "cosessiondb") && p.is_file())
        .collect::<Vec<_>>();
    found.sort();
    Ok(if named.is_file() {
        Some(named)
    } else {
        found.into_iter().next()
    })
}

impl CaptureOneAdapter {
    fn clear_after(&self, step: u8) {
        if step <= 2 {
            self.health.clear("document");
            self.health.clear("document_not_open");
        }
        if step <= 4 {
            self.health.clear("recipes");
            self.health.clear("recipe_missing");
        }
        if step <= 5 {
            self.health.clear("hooks");
            self.health.clear("hooks_not_installed");
        }
    }
    pub(crate) fn ensure_provision(&self, inputs: &ProvisionInputs) -> Result<ProvisionReport> {
        let _guard = self
            .provision_lock
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut editor = self.user.editor_info().inspect_err(|e| {
            self.clear_after(1);
            if classify_editor_error(&e.to_string()) == Some(EditorFailure::AutomationDenied) {
                self.health.set("editor_automation_denied", "macOS has not allowed Session Manager to control Capture One. Allow it under System Settings › Privacy & Security › Automation");
            } else { self.health.set("editor_not_running", e.to_string()); }
        })?;
        if !editor.running {
            self.user.launch().inspect_err(|e| {
                self.clear_after(1);
                if classify_editor_error(&e.to_string()) == Some(EditorFailure::NotInstalled) {
                    self.health.set(
                        "editor_not_installed",
                        "Capture One is not installed on this Mac",
                    );
                } else {
                    self.health.set("editor_not_running", e.to_string());
                }
            })?;
            let started = Instant::now();
            while !editor.running && started.elapsed() < inputs.wait {
                thread::sleep(editor_wait_poll(inputs.wait));
                if let Ok(info) = self.user.editor_info() {
                    editor = info;
                }
            }
            if !editor.running {
                self.clear_after(1);
                self.health.set(
                    "editor_not_running",
                    format!(
                        "Capture One did not start within {} s",
                        inputs.wait.as_secs()
                    ),
                );
                return Err(Error::DocumentNotOpen);
            }
        }
        if editor.pro == Some(false) {
            self.clear_after(1);
            self.health.set(
                "editor_not_pro",
                "Capture One Pro is required; the installed edition cannot be scripted",
            );
            return Err(Error::DocumentNotOpen);
        }
        for code in [
            "editor_not_installed",
            "editor_not_running",
            "editor_not_pro",
            "editor_automation_denied",
        ] {
            self.health.clear(code);
        }

        let folder = Path::new(&self.document_id);
        let base = folder
            .file_name()
            .ok_or(Error::DocumentNotSession)?
            .to_string_lossy();
        let expected = folder.join(format!("{base}.cosessiondb"));
        let mut created = false;
        let file = if !folder.exists() {
            let parent = folder.parent().ok_or(Error::DocumentNotSession)?;
            fs::create_dir_all(parent)?;
            let made = self
                .user
                .make_document(&parent.to_string_lossy(), &base)
                .map_err(|e| {
                    self.clear_after(2);
                    self.health.set(
                        "document_not_created",
                        format!(
                            "Capture One could not create the session {}: {e}",
                            expected.display()
                        ),
                    );
                    e
                })?;
            if made != self.document_id || !expected.is_file() {
                self.clear_after(2);
                self.health.set(
                    "document_not_created",
                    format!(
                        "Capture One could not create the session {}: no session file appeared",
                        expected.display()
                    ),
                );
                return Err(Error::DocumentNotOpen);
            }
            created = true;
            expected
        } else {
            match session_file(folder)? {
                Some(path) => path,
                None => {
                    self.clear_after(2);
                    self.health.set(
                        "document_not_session",
                        format!(
                            "{} exists but is not a Capture One session",
                            folder.display()
                        ),
                    );
                    return Err(Error::DocumentNotSession);
                }
            }
        };
        self.health.clear("document_not_created");
        self.health.clear("document_not_session");
        fs::create_dir_all(folder.join("Capture"))?;

        if self.user.document_info(&self.document_id).is_err() {
            let open_error = self.user.open_document(&file.to_string_lossy()).err();
            let started = Instant::now();
            let mut opened = self.user.document_info(&self.document_id).is_ok();
            while !opened && open_error.is_none() && started.elapsed() < inputs.wait {
                thread::sleep(editor_wait_poll(inputs.wait));
                opened = self.user.document_info(&self.document_id).is_ok();
            }
            if !opened {
                self.clear_after(3);
                let suffix = open_error.map(|e| format!(": {e}")).unwrap_or_default();
                self.health.set(
                    "document_not_open",
                    format!("Capture One could not open {}{suffix}", file.display()),
                );
                return Err(Error::DocumentNotOpen);
            }
        }
        self.health.clear("document_not_open");
        if self.user.document_info(&self.document_id)?.kind != "session" {
            self.clear_after(3);
            self.health.set(
                "document_not_session",
                "Capture One document is not a session",
            );
            return Err(Error::DocumentNotSession);
        }
        self.health.set_ok(
            "document",
            format!(
                "{} open",
                file.file_name().unwrap_or_default().to_string_lossy()
            ),
        );

        let mut recipes_created = Vec::new();
        for (name, spec) in [
            (&self.preview_recipe, RecipeSpec::PREVIEW),
            (&self.master_recipe, RecipeSpec::MASTER),
        ] {
            let want = spec.props();
            let checked = self.user.ensure_recipe(&self.document_id, name, &want);
            match checked {
                Ok(value)
                    if {
                        let mut got = value.props.clone();
                        got.root_folder_type = None;
                        got == want
                    } =>
                {
                    if value.created {
                        recipes_created.push(name.clone());
                    }
                }
                outcome => {
                    self.clear_after(4);
                    let cause = match outcome {
                        Err(e) => e.to_string(),
                        Ok(_) => "property read-back mismatch".into(),
                    };
                    self.health.set(
                        "recipe_missing",
                        format!("{name} could not be created or repaired: {cause}"),
                    );
                    return Err(Error::RecipeMissing { name: name.clone() });
                }
            }
        }
        self.health.clear("recipe_missing");
        self.health.set_ok("recipes", "Ready");

        let name = inputs.style_name.trim();
        let style = if name.is_empty() {
            self.health.clear("style_not_found");
            StyleOutcome::NotRequested
        } else {
            match self.user.ensure_capture_style(&self.document_id, name) {
                Ok(found) if found.listed && found.applied == [name] => {
                    self.health.clear("style_not_found");
                    StyleOutcome::Applied
                }
                Ok(found) if !found.listed => {
                    self.health.set(
                        "style_not_found",
                        format!("Style '{name}' not found in Capture One"),
                    );
                    StyleOutcome::NotListed
                }
                Ok(_) => {
                    self.health.set(
                        "style_not_found",
                        format!("Style '{name}' could not be applied: read-back mismatch"),
                    );
                    StyleOutcome::NotListed
                }
                Err(e) => {
                    self.health.set(
                        "style_not_found",
                        format!("Style '{name}' could not be applied: {e}"),
                    );
                    StyleOutcome::NotListed
                }
            }
        };

        if let Err(e) = EditingProvider::install_hooks(self) {
            self.health.clear("hooks");
            self.health.set("hooks_not_installed", e.to_string());
            return Err(e);
        }
        self.health.set_ok("hooks", "Installed");
        Ok(ProvisionReport {
            document_created: created,
            recipes_created,
            style,
        })
    }

    pub(crate) fn front_provision_document(&self) -> Result<()> {
        let match_front = self
            .user
            .front_document(&self.document_id)
            .ok()
            .and_then(|front| canon(Path::new(&front)).ok())
            == canon(Path::new(&self.document_id)).ok();
        if !match_front {
            self.health.set(
                "document_not_front",
                "Capture One is showing another session; captures would land there",
            );
            return Err(Error::DocumentNotFront);
        }
        self.health.clear("document_not_front");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mc26_classifies_editor_failures() {
        assert_eq!(
            classify_editor_error("Unable to find application named 'Capture One'"),
            Some(EditorFailure::NotInstalled)
        );
        assert_eq!(
            classify_editor_error("Not authorized to send Apple events to Capture One. (-1743)"),
            Some(EditorFailure::AutomationDenied)
        );
        assert_eq!(classify_editor_error("busy"), None);
    }
}
