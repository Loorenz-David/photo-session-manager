use crate::{
    Error, Result,
    domain::editing::ProvisionInputs,
    ports::{camera::CameraProvider, editing::EditingProvider, store::Store},
    services::health::HealthService,
};

/// The `editor_not_following` detail (10E MC-34 R-I33, master plan §6.6).
pub const EDITOR_NOT_FOLLOWING: &str =
    "Capture One will not show this session's new photos until it is reachable";

pub struct EditingStartup<'a> {
    pub provider: &'a dyn EditingProvider,
    pub store: &'a dyn Store,
    pub health: &'a HealthService,
    pub provision: &'a ProvisionInputs,
    /// The app's camera provider; with one that does not depend on the editing provider (the
    /// direct adapter) a failed capture-folder step is `editor_not_following` (R-I33, P48 L12).
    pub camera: Option<&'a dyn CameraProvider>,
}
impl EditingStartup<'_> {
    pub fn run(&self) -> Result<()> {
        if self.provider.ensure(self.provision)?.deferred {
            // "Take the camera back" holds the editor-launch gate (R-I32, P48 L1): nothing after
            // the ensure may address Capture One, and nothing is reported about it.
            return Ok(());
        }
        let camera_independent = self
            .camera
            .is_some_and(|c| !c.capabilities().depends_on_editing_provider);
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
            if camera_independent {
                self.health.clear("capture_folder_not_set");
                self.health
                    .set("editor_not_following", EDITOR_NOT_FOLLOWING);
            } else {
                self.health.set("capture_folder_not_set", e.to_string());
            }
            return Err(Error::CaptureFolderNotSet);
        }
        self.health.clear("capture_folder_not_set");
        self.health.clear("editor_not_following");
        Ok(())
    }
}
