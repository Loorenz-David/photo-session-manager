use crate::{
    Error, Result,
    domain::session::SessionDto,
    mechanisms::time::now_ms,
    ports::{clock::Clock, editing::EditingProvider, store::Store},
    services::health::HealthService,
};

pub struct SessionsService<'a> {
    store: &'a dyn Store,
    clock: &'a dyn Clock,
    editing: Option<&'a dyn EditingProvider>,
    health: Option<&'a HealthService>,
}

impl<'a> SessionsService<'a> {
    pub fn new(store: &'a dyn Store, clock: &'a dyn Clock) -> Self {
        Self {
            store,
            clock,
            editing: None,
            health: None,
        }
    }

    pub fn with_editing(
        mut self,
        editing: &'a dyn EditingProvider,
        health: &'a HealthService,
    ) -> Self {
        self.editing = Some(editing);
        self.health = Some(health);
        self
    }

    pub fn activate(&self, id: i64) -> Result<()> {
        let editing = self.editing.ok_or(Error::DocumentNotOpen)?;
        let health = self.health.ok_or(Error::DocumentNotOpen)?;
        let session = self.store.session(id)?;
        let item = self
            .store
            .item(session.session.item_id.ok_or(Error::DocumentNotSession)?)?;
        let root = editing.inspect_document()?.capture_root;
        if let Err(e) = editing.set_capture_folder(&format!("{}/{}", root, item.folder_key)) {
            health.set("capture_folder_not_set", e.to_string());
            return Err(Error::CaptureFolderNotSet);
        }
        self.store.set_active_session_id(Some(id))?;
        health.clear("capture_folder_not_set");
        Ok(())
    }

    pub fn deactivate(&self) -> Result<()> {
        let editing = self.editing.ok_or(Error::DocumentNotOpen)?;
        let health = self.health.ok_or(Error::DocumentNotOpen)?;
        let root = editing.inspect_document()?.capture_root;
        if let Err(e) = editing.set_capture_folder(&format!("{root}/_Unassigned")) {
            health.set("capture_folder_not_set", e.to_string());
            return Err(Error::CaptureFolderNotSet);
        }
        self.store.set_active_session_id(None)?;
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
            tx.item(item_id)?;
            id = tx.insert_session(item_id, at)?;
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
        let photos = self
            .store
            .photos_in_session(id)?
            .into_iter()
            .filter(|photo| photo.classification != crate::domain::photo::Classification::Deleted)
            .collect();
        Ok(crate::domain::session::SessionView {
            session,
            item,
            photos,
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
            let label = row_label(input);
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
