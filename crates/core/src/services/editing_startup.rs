use crate::{
    Error, Result,
    ports::{editing::EditingProvider, store::Store},
    services::health::HealthService,
};

pub struct EditingStartup<'a> {
    pub provider: &'a dyn EditingProvider,
    pub store: &'a dyn Store,
    pub health: &'a HealthService,
}
impl EditingStartup<'_> {
    pub fn run(&self) -> Result<()> {
        let doc = match self.provider.inspect_document() {
            Ok(doc) => {
                self.health.clear("document_not_open");
                self.health.clear("document_not_session");
                self.health.set_ok("document", "Session open");
                doc
            }
            Err(e) => {
                self.health.set(
                    if matches!(e, Error::DocumentNotSession) {
                        "document_not_session"
                    } else {
                        "document_not_open"
                    },
                    e.to_string(),
                );
                return Err(e);
            }
        };
        if let Err(e) = self.provider.install_hooks() {
            self.health.set("hooks_not_installed", e.to_string());
        } else {
            self.health.clear("hooks_not_installed");
            self.health.set_ok("hooks", "Installed");
        }
        match self.provider.check_recipes() {
            Ok(missing) => {
                self.health.clear("recipe_missing");
                if missing.is_empty() {
                    self.health.set_ok("recipes", "Ready");
                }
                for name in missing {
                    self.health.set("recipe_missing", name);
                }
            }
            Err(e) => self.health.set("recipe_missing", e.to_string()),
        }
        let active = self.store.active_session_id()?;
        let folder = if let Some(id) = active {
            let session = self.store.session(id)?;
            let item = self
                .store
                .item(session.session.item_id.ok_or(Error::DocumentNotSession)?)?;
            item.folder_key
        } else {
            "_Unassigned".into()
        };
        if let Err(e) = self
            .provider
            .set_capture_folder(&format!("{}/{}", doc.capture_root, folder))
        {
            self.health.set("capture_folder_not_set", e.to_string());
            return Err(Error::CaptureFolderNotSet);
        }
        self.health.clear("capture_folder_not_set");
        Ok(())
    }
}
