use crate::{
    Error, Result,
    domain::editing::ProvisionInputs,
    domain::session::SessionDto,
    mechanisms::time::now_ms,
    ports::{
        camera::{ArrivalFollow, CameraProvider},
        clock::Clock,
        editing::EditingProvider,
        store::Store,
    },
    services::{editing_startup::EDITOR_NOT_FOLLOWING, health::HealthService},
};

pub struct SessionsService<'a> {
    store: &'a dyn Store,
    clock: &'a dyn Clock,
    editing: Option<&'a dyn EditingProvider>,
    health: Option<&'a HealthService>,
    provision: Option<&'a ProvisionInputs>,
    commerce: Option<&'a dyn crate::ports::commerce::CommerceAdapter>,
    camera: Option<&'a dyn CameraProvider>,
    follow: Option<&'a dyn ArrivalFollow>,
}

impl<'a> SessionsService<'a> {
    pub fn new(store: &'a dyn Store, clock: &'a dyn Clock) -> Self {
        Self {
            store,
            clock,
            editing: None,
            health: None,
            provision: None,
            commerce: None,
            camera: None,
            follow: None,
        }
    }

    /// The app's camera provider: when it does not depend on the editing provider (the direct
    /// adapter), activation stores the session first and Capture One follows when it can
    /// (10E MC-34 R-I33). Without it, activation behaves as with the Capture One camera adapter.
    pub fn with_camera(mut self, camera: &'a dyn CameraProvider) -> Self {
        self.camera = Some(camera);
        self
    }
    pub fn with_follow(mut self, follow: &'a dyn ArrivalFollow) -> Self {
        self.follow = Some(follow);
        self
    }

    /// R-I33: the camera does not need Capture One to shoot.
    fn camera_independent(&self) -> bool {
        self.camera
            .is_some_and(|c| !c.capabilities().depends_on_editing_provider)
    }

    /// R-I33's follow-up after the session is stored: one health item says whether Capture One
    /// shows it (`editor_not_following`), never `capture_folder_not_set` (P48 L12).
    fn editor_follows(&self, health: &HealthService, applied: Result<()>) {
        health.clear("capture_folder_not_set");
        match applied {
            Ok(()) => health.clear("editor_not_following"),
            Err(error) => {
                tracing::warn!(%error, "Capture One does not follow the session");
                health.set("editor_not_following", EDITOR_NOT_FOLLOWING);
            }
        }
    }

    pub fn with_commerce(
        mut self,
        commerce: Option<&'a dyn crate::ports::commerce::CommerceAdapter>,
    ) -> Self {
        self.commerce = commerce;
        self
    }

    pub fn with_editing(
        mut self,
        editing: &'a dyn EditingProvider,
        health: &'a HealthService,
        provision: &'a ProvisionInputs,
    ) -> Self {
        self.editing = Some(editing);
        self.health = Some(health);
        self.provision = Some(provision);
        self
    }

    pub fn activate(&self, id: i64) -> Result<()> {
        let editing = self.editing.ok_or(Error::DocumentNotOpen)?;
        let health = self.health.ok_or(Error::DocumentNotOpen)?;
        let session = self.store.session(id)?;
        let item = self
            .store
            .item(session.session.item_id.ok_or(Error::DocumentNotSession)?)?;
        let provision = self.provision.ok_or(Error::DocumentNotOpen)?;
        if self.camera_independent() {
            self.store.set_active_session_id(Some(id))?;
            let applied = (|| {
                if editing.ensure(provision)?.deferred {
                    return Err(Error::EditorLaunchDeferred);
                }
                editing.bring_document_to_front()?;
                let root = editing.inspect_document()?.capture_root;
                let target = format!("{}/{}", root, item.folder_key);
                editing.set_capture_folder(&target)?;
                editing.set_current_collection(&target).and_then(|_| {
                    self.follow
                        .map_or(Ok(()), |follow| follow.collection_switched())
                })
            })();
            self.editor_follows(health, applied);
            return Ok(());
        }
        if editing.ensure(provision)?.deferred {
            return Err(Error::EditorLaunchDeferred);
        }
        editing.bring_document_to_front()?;
        let root = editing.inspect_document()?.capture_root;
        let target = format!("{}/{}", root, item.folder_key);
        if let Err(e) = editing
            .set_capture_folder(&target)
            .and_then(|_| editing.set_current_collection(&target))
        {
            health.set("capture_folder_not_set", e.to_string());
            return Err(Error::CaptureFolderNotSet);
        }
        self.store.set_active_session_id(Some(id))?;
        if let Some(follow) = self.follow {
            follow.collection_switched()?;
        }
        health.clear("capture_folder_not_set");
        Ok(())
    }

    pub fn deactivate(&self) -> Result<()> {
        let editing = self.editing.ok_or(Error::DocumentNotOpen)?;
        let health = self.health.ok_or(Error::DocumentNotOpen)?;
        if self.camera_independent() {
            self.store.set_active_session_id(None)?;
            health.clear("document_not_front");
            let applied = editing.inspect_document().and_then(|doc| {
                let target = format!("{}/_Unassigned", doc.capture_root);
                editing.set_capture_folder(&target)?;
                editing.set_current_collection(&target).and_then(|_| {
                    self.follow
                        .map_or(Ok(()), |follow| follow.collection_switched())
                })
            });
            self.editor_follows(health, applied);
            return Ok(());
        }
        let root = editing.inspect_document()?.capture_root;
        let target = format!("{root}/_Unassigned");
        if let Err(e) = editing
            .set_capture_folder(&target)
            .and_then(|_| editing.set_current_collection(&target))
        {
            health.set("capture_folder_not_set", e.to_string());
            return Err(Error::CaptureFolderNotSet);
        }
        self.store.set_active_session_id(None)?;
        if let Some(follow) = self.follow {
            follow.collection_switched()?;
        }
        health.clear("document_not_front");
        health.clear("capture_folder_not_set");
        Ok(())
    }

    /// The session captures are currently routed to, if any (what `activate` last stored).
    pub fn active(&self) -> Result<Option<i64>> {
        self.store.active_session_id()
    }

    pub fn create(&self, item_id: i64) -> Result<SessionDto> {
        if let Some(id) = self.store.session_for_item(item_id)? {
            return self.store.session(id);
        }
        let mut id = 0;
        let at = now_ms(self.clock);
        self.store.transaction(&mut |tx| {
            let item = tx.item(item_id)?;
            id = tx.insert_session(item_id, at)?;
            crate::services::attributes::prefill_defaults(tx, item_id, &item.item_type, at)?;
            Ok(())
        })?;
        self.store.session(id)
    }

    pub fn get(&self, id: i64) -> Result<SessionDto> {
        self.store.session(id)
    }
    /// The workspace view of one session: its item and its photos without the deleted ones.
    pub fn view(&self, id: i64) -> Result<crate::domain::session::SessionView> {
        let session = self.store.session(id)?;
        let item = match session.session.item_id {
            Some(item_id) => Some(self.store.item(item_id)?),
            None => None,
        };
        let photos: Vec<_> = self
            .store
            .photos_in_session(id)?
            .into_iter()
            .filter(|photo| {
                photo.classification != crate::domain::photo::Classification::Deleted
                    && photo.integrity != crate::domain::photo::Integrity::RawMissing
            })
            .collect();
        let publish = self.store.latest_publish(id)?;
        let upload_states = photos
            .iter()
            .map(|photo| {
                let Some(commerce) = self.commerce else {
                    return Ok(None);
                };
                let Some(item_id) = session.session.item_id else {
                    return Ok(None);
                };
                let role = match photo.classification {
                    crate::domain::photo::Classification::Main => "gallery".to_string(),
                    crate::domain::photo::Classification::Condition => {
                        format!("attribute:{}", commerce.condition_photos_key())
                    }
                    _ => return Ok(None),
                };
                // The photo's first output slot (`single` or `left`) stands for its upload.
                let slot = self.store.composition(photo.id)?.kind.slots()[0];
                Ok(self
                    .store
                    .photo_link(photo.id, item_id, &role, slot)?
                    .map(|link| link.upload_state))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(crate::domain::session::SessionView {
            session,
            item,
            compositions: self
                .store
                .compositions_in_session(id)?
                .into_iter()
                .filter(|c| photos.iter().any(|p| p.id == c.photo_id))
                .collect(),
            photos,
            publish,
            upload_states,
        })
    }
    pub fn unassigned(&self) -> Result<SessionDto> {
        self.get(1)
    }
}

impl SessionsService<'_> {
    pub fn list(
        &self,
        filter: crate::domain::session::ListFilter,
        query: Option<String>,
    ) -> Result<crate::domain::session::SessionList> {
        use crate::domain::session::{
            ListCounts, ListFilter, ListGroup, SessionList, SessionRow, SessionStatus,
        };
        use crate::mechanisms::{
            status::{StatusInput, derive_status, row_label},
            time::group_by_activity,
        };
        let records = self.store.session_list_records()?;
        let cleared = crate::services::retention::cleared_sessions(self.store)?;
        let unassigned_count = self.store.session(1)?.photo_count;
        let query = query.unwrap_or_default().to_lowercase();
        let mut counts = ListCounts {
            open: 0,
            published: 0,
            all: 0,
        };
        let mut rows = Vec::new();
        for record in records {
            let dto = &record.session;
            let input = StatusInput {
                latest_step: record.latest_step,
                failed_step: record.failed_step,
                published_at: dto.session.published_at,
                dirty: dto.dirty,
                main_count: dto.main_count,
                unreviewed_count: dto.unreviewed_count,
                non_deleted_missing_preview_count: record.non_deleted_missing_preview_count,
                uploaded: record.uploaded,
                total: record.total,
            };
            let status = derive_status(input);
            counts.all += 1;
            if status == SessionStatus::Published {
                counts.published += 1;
            } else {
                counts.open += 1;
            }
            if (filter == ListFilter::Open && status == SessionStatus::Published)
                || (filter == ListFilter::Published && status != SessionStatus::Published)
                || (!query.is_empty()
                    && !record.title.to_lowercase().contains(&query)
                    && !record.sku.to_lowercase().contains(&query))
            {
                continue;
            }
            let label = if cleared.contains(&dto.session.id) {
                crate::domain::session::RowLabel::Archived
            } else if status == SessionStatus::Publishing && record.waiting_for_editor {
                // Changelog 45 (P65): the publish waits for Capture One to load its photos.
                crate::domain::session::RowLabel::WaitingForEditor
            } else {
                row_label(input)
            };
            let progress = if let crate::domain::session::RowLabel::Publishing(pct) = label {
                Some(pct)
            } else {
                None
            };
            rows.push(SessionRow {
                id: dto.session.id,
                sku: record.sku,
                item_type: record.item_type,
                title: record.title,
                main_count: dto.main_count,
                condition_count: dto.condition_count,
                unreviewed_count: dto.unreviewed_count,
                status,
                label,
                progress,
                last_activity_at: dto.session.last_activity_at,
                cover_preview_path: record.cover_preview_path,
                publish: record.publish,
            });
        }
        let group_ids = group_by_activity(
            &rows
                .iter()
                .map(|r| (r.id, r.last_activity_at))
                .collect::<Vec<_>>(),
            self.clock.now_ms(),
            &chrono::Local,
        );
        let groups = group_ids
            .into_iter()
            .map(|group| ListGroup {
                label: group.label,
                rows: group
                    .rows
                    .into_iter()
                    .filter_map(|id| rows.iter().find(|r| r.id == id).cloned())
                    .collect(),
            })
            .collect();
        Ok(SessionList {
            groups,
            counts,
            unassigned_count,
        })
    }
}
