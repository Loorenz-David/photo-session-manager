use crate::{
    Error, Result,
    domain::editing::ProvisionInputs,
    ports::{editing::EditingProvider, store::Store},
    services::health::HealthService,
};

pub struct EditingStartup<'a> {
    pub provider: &'a dyn EditingProvider,
    pub store: &'a dyn Store,
    pub health: &'a HealthService,
    pub provision: &'a ProvisionInputs,
}
impl EditingStartup<'_> {
    pub fn run(&self) -> Result<()> {
        self.provider.ensure(self.provision)?;
        let doc = match self.provider.inspect_document() {
            Ok(doc) => {
                self.health.clear("document_not_open");
                self.health.clear("document_not_session");
                doc
            }
            Err(e) => {
                if matches!(e, Error::CaptureFolderNotSet) {
                    if !self.health.has("capture_folder_not_set") {
                        self.health.set("capture_folder_not_set", e.to_string());
                    }
                    return Err(e);
                }
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
        let active = self.store.active_session_id()?;
        if active.is_some() {
            self.provider.bring_document_to_front()?;
        } else {
            self.health.clear("document_not_front");
        }
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
