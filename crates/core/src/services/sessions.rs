use crate::{
    Error, Result,
    domain::editing::ProvisionInputs,
    domain::session::SessionDto,
    events::AppEvent,
    mechanisms::time::now_ms,
    ports::{
        camera::{ArrivalFollow, CameraProvider},
        clock::Clock,
        editing::EditingProvider,
        events::EventSink,
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
    events: Option<&'a dyn EventSink>,
    /// H2 (MC-H1.1): the row picture rule's remote half and the cover queue.
    remote: Option<&'a crate::services::remote::RemoteItems>,
    /// A16: the filing target was already claimed for this request (`claim_target`), so the
    /// switch does not set it again (an older request finishing late must not take it back).
    target_claimed: bool,
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
            events: None,
            remote: None,
            target_claimed: false,
        }
    }

    /// A16: the request's filing target was claimed before it waited for its turn.
    pub fn with_target_claimed(mut self, claimed: bool) -> Self {
        self.target_claimed = claimed;
        self
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

    pub fn with_events(mut self, events: &'a dyn EventSink) -> Self {
        self.events = Some(events);
        self
    }

    /// H2: rows follow the whole row picture rule (MC-H1.1) and `page` requests pending covers.
    pub fn with_remote(mut self, remote: &'a crate::services::remote::RemoteItems) -> Self {
        self.remote = Some(remote);
        self
    }

    pub fn arrange(&self, id: i64) -> Result<bool> {
        let session = self.store.session(id)?;
        if session.session.item_id.is_none() {
            return Err(Error::InvalidData(
                "the Unassigned session cannot be arranged".into(),
            ));
        }
        let changed = self.store.arrange_session_once(id, self.clock.now_ms())?;
        if changed && let Some(events) = self.events {
            events.emit(AppEvent::SessionChanged { session_id: id });
        }
        Ok(changed)
    }

    pub fn next_unarranged(&self, after: i64) -> Result<Option<i64>> {
        let current = self.store.session(after)?.session;
        let cleared = crate::services::retention::cleared_sessions(self.store)?;
        let mut candidates = Vec::new();
        for record in self.store.session_list_records()? {
            let session = &record.session.session;
            if session.id == after
                || session.arranged_at.is_some()
                || session.published_at.is_some()
                || cleared.contains(&session.id)
            {
                continue;
            }
            if !self
                .store
                .photos_in_session(session.id)?
                .iter()
                .any(|p| p.classification != crate::domain::photo::Classification::Deleted)
            {
                continue;
            }
            candidates.push((session.created_at, session.id));
        }
        candidates.sort();
        Ok(candidates
            .iter()
            .find(|(at, id)| (*at, *id) > (current.created_at, current.id))
            .or_else(|| candidates.first())
            .map(|(_, id)| *id))
    }

    /// R-I33: the camera does not need Capture One to shoot.
    fn camera_independent(&self) -> bool {
        self.camera
            .is_some_and(|c| !c.capabilities().depends_on_editing_provider)
    }

    /// A16 (owner 2026-11-18): where the camera's photos are filed follows the request at once,
    /// before Capture One is asked for anything: a slow Capture One switch can never leave the
    /// next session's photos in the previous session. Only with a camera that does not shoot
    /// through Capture One (with a tethered one, Capture One's folder is where photos land).
    /// `true` when the target was set here.
    pub fn claim_target(&self, id: Option<i64>) -> Result<bool> {
        if !self.camera_independent() {
            return Ok(false);
        }
        let folder = match id {
            // MC-H3.4 (H4 projection r0, L1): a remote item is never the filing target.
            Some(id) if self.is_remote(id)? => return Ok(false),
            Some(id) => {
                let session = self.store.session(id)?;
                let item = self
                    .store
                    .item(session.session.item_id.ok_or(Error::DocumentNotSession)?)?;
                item.folder_key
            }
            None => "_Unassigned".to_owned(),
        };
        self.store_target(id, &folder)?;
        Ok(true)
    }

    /// Stores the filing target, with one `arrival.target` line when it changes.
    fn set_target(&self, id: Option<i64>, folder: &str) -> Result<()> {
        if self.target_claimed {
            return Ok(());
        }
        self.store_target(id, folder)
    }

    fn store_target(&self, id: Option<i64>, folder: &str) -> Result<()> {
        let before = self.store.active_session_id()?;
        self.store.set_active_session_id(id)?;
        if before != id {
            tracing::info!(
                target: "psm::arrival",
                "arrival.target session={} folder={} from={}",
                id.map_or("none".to_owned(), |id| id.to_string()),
                folder,
                before.map_or("none".to_owned(), |id| id.to_string()),
            );
        }
        Ok(())
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

    /// MC-H3.1: whether the session's item is `Remote(_)`: from `with_remote` when attached,
    /// else from the store when the service has a commerce integration, else never (H6).
    fn remote_state(&self, id: i64) -> Result<crate::domain::remote::RemoteState> {
        if let Some(remote) = self.remote {
            return remote.state(id);
        }
        if self.commerce.is_some() {
            return Ok(crate::mechanisms::remote::remote_state(
                self.store.remote_input(id)?,
            ));
        }
        Ok(crate::domain::remote::RemoteState::Local)
    }

    fn is_remote(&self, id: i64) -> Result<bool> {
        Ok(matches!(
            self.remote_state(id)?,
            crate::domain::remote::RemoteState::Remote(_)
        ))
    }

    pub fn activate(&self, id: i64) -> Result<()> {
        // MC-H3.4: opening a remote item never moves the camera's capture folder: no editing
        // call, no camera target change, the active session unchanged.
        if self.is_remote(id)? {
            tracing::info!(
                target: "psm::capture_one",
                "capture_one.switch_skipped request=\"open session {id}\" reason=remote_item"
            );
            return Ok(());
        }
        let editing = self.editing.ok_or(Error::DocumentNotOpen)?;
        let health = self.health.ok_or(Error::DocumentNotOpen)?;
        let session = self.store.session(id)?;
        let item = self
            .store
            .item(session.session.item_id.ok_or(Error::DocumentNotSession)?)?;
        let provision = self.provision.ok_or(Error::DocumentNotOpen)?;
        let from = self.store.active_session_id()?;
        let to = format!("session {id}");
        let log = SwitchLog::new("open_session", from, &to, &item.folder_key, editing);
        if self.camera_independent() {
            self.set_target(Some(id), &item.folder_key)?;
            let applied = (|| {
                if log.step("ensure", || editing.ensure(provision))?.deferred {
                    return Err(Error::EditorLaunchDeferred);
                }
                log.step("bring_document_to_front", || {
                    editing.bring_document_to_front()
                })?;
                let root = log
                    .step("inspect_document", || editing.inspect_document())?
                    .capture_root;
                let target = format!("{}/{}", root, item.folder_key);
                log.step("set_capture_folder", || editing.set_capture_folder(&target))?;
                log.step("set_current_collection", || {
                    editing.set_current_collection(&target)
                })?;
                log.step("collection_switched", || {
                    self.follow
                        .map_or(Ok(()), |follow| follow.collection_switched())
                })
            })();
            log.done(&applied);
            self.editor_follows(health, applied);
            return Ok(());
        }
        let result = (|| {
            if log.step("ensure", || editing.ensure(provision))?.deferred {
                return Err(Error::EditorLaunchDeferred);
            }
            log.step("bring_document_to_front", || {
                editing.bring_document_to_front()
            })?;
            let root = log
                .step("inspect_document", || editing.inspect_document())?
                .capture_root;
            let target = format!("{}/{}", root, item.folder_key);
            if let Err(e) = log
                .step("set_capture_folder", || editing.set_capture_folder(&target))
                .and_then(|_| {
                    log.step("set_current_collection", || {
                        editing.set_current_collection(&target)
                    })
                })
            {
                health.set("capture_folder_not_set", e.to_string());
                return Err(Error::CaptureFolderNotSet);
            }
            self.store.set_active_session_id(Some(id))?;
            if let Some(follow) = self.follow {
                log.step("collection_switched", || follow.collection_switched())?;
            }
            health.clear("capture_folder_not_set");
            Ok(())
        })();
        log.done(&result);
        result
    }

    pub fn deactivate(&self) -> Result<()> {
        let editing = self.editing.ok_or(Error::DocumentNotOpen)?;
        let health = self.health.ok_or(Error::DocumentNotOpen)?;
        let from = self.store.active_session_id()?;
        let log = SwitchLog::new("leave_session", from, "Unassigned", "_Unassigned", editing);
        if self.camera_independent() {
            self.set_target(None, "_Unassigned")?;
            health.clear("document_not_front");
            let applied = log
                .step("inspect_document", || editing.inspect_document())
                .and_then(|doc| {
                    let target = format!("{}/_Unassigned", doc.capture_root);
                    log.step("set_capture_folder", || editing.set_capture_folder(&target))?;
                    log.step("set_current_collection", || {
                        editing.set_current_collection(&target)
                    })?;
                    log.step("collection_switched", || {
                        self.follow
                            .map_or(Ok(()), |follow| follow.collection_switched())
                    })
                });
            log.done(&applied);
            self.editor_follows(health, applied);
            return Ok(());
        }
        let result = (|| {
            let root = log
                .step("inspect_document", || editing.inspect_document())?
                .capture_root;
            let target = format!("{root}/_Unassigned");
            if let Err(e) = log
                .step("set_capture_folder", || editing.set_capture_folder(&target))
                .and_then(|_| {
                    log.step("set_current_collection", || {
                        editing.set_current_collection(&target)
                    })
                })
            {
                health.set("capture_folder_not_set", e.to_string());
                return Err(Error::CaptureFolderNotSet);
            }
            self.store.set_active_session_id(None)?;
            if let Some(follow) = self.follow {
                log.step("collection_switched", || follow.collection_switched())?;
            }
            health.clear("document_not_front");
            health.clear("capture_folder_not_set");
            Ok(())
        })();
        log.done(&result);
        result
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
        // MC-H3.5 (M-H4): a remote item's photo rows never reach the screen (an archived item's
        // cleared rows name deleted previews); its pictures come from `RemoteItems::view`.
        let remote = self.is_remote(id)?;
        let photos: Vec<_> = self
            .store
            .photos_in_session(id)?
            .into_iter()
            .filter(|_| !remote)
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
        use crate::mechanisms::{status::row_label, time::group_by_activity};
        let records = self.store.session_list_records()?;
        // MC-H1.1: one read of the remote inputs for every row (none without remote items).
        let pictures = self
            .remote
            .map(|remote| remote.row_pictures())
            .transpose()?;
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
            // MC-H3.7: a remote item's row counts its record (the list and pages only;
            // `ready_session_ids` keeps the raw input, H4 projection r0, L8).
            let remote_row = pictures
                .as_ref()
                .and_then(|pictures| pictures.remote_row(dto.session.id));
            let mut input = status_input(&record);
            let (main_count, condition_count, unreviewed_count) = match remote_row {
                Some(row) => {
                    input.main_count = row.gallery as i64;
                    input.unreviewed_count = 0;
                    input.non_deleted_missing_preview_count = 0;
                    (row.gallery as i64, row.condition as i64, 0)
                }
                None => (dto.main_count, dto.condition_count, dto.unreviewed_count),
            };
            let cleared_or_remote = cleared.contains(&dto.session.id) || remote_row.is_some();
            let label = if cleared_or_remote {
                // MC-H3.7: "Archived in Shopify" for both kinds, unless a publish of theirs is
                // running, held, failed or cancelled.
                use crate::domain::session::RowLabel;
                match row_label(input) {
                    label @ (RowLabel::WaitingForYou(_)
                    | RowLabel::Publishing(_)
                    | RowLabel::PublishFailed
                    | RowLabel::PublishCancelled
                    | RowLabel::DescriptionFailed) => label,
                    _ => RowLabel::Archived,
                }
            } else {
                row_label(input)
            };
            // MC-H3.7A (09C §14.3): a shop-only row whose label is "Archived in Shopify" is
            // Published; while a publish of it shows through, `derive_status` as for any row.
            let status = row_status(remote_row.as_ref(), label, input);
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
            let label = if !cleared_or_remote
                && status == SessionStatus::Publishing
                && record.waiting_for_editor
            {
                // Changelog 45 (P65): the publish waits for Capture One to load its photos.
                crate::domain::session::RowLabel::WaitingForEditor
            } else {
                label
            };
            let progress = if let crate::domain::session::RowLabel::Publishing(pct) = label {
                Some(pct)
            } else {
                None
            };
            let adjusted = record.cover_adjusted_preview_path.as_deref();
            let original = record.cover_original_preview_path.as_deref();
            let (cover, cover_pending) = match &pictures {
                Some(pictures) => pictures.row_picture(dto.session.id, adjusted, original),
                None => (
                    crate::services::remote::local_picture(adjusted, original),
                    false,
                ),
            };
            rows.push(SessionRow {
                id: dto.session.id,
                sku: record.sku,
                item_type: record.item_type,
                title: record.title,
                main_count,
                condition_count,
                unreviewed_count,
                status,
                label,
                progress,
                last_activity_at: dto.session.last_activity_at,
                created_at: dto.session.created_at,
                arranged: dto.session.arranged_at.is_some(),
                cover_preview_path: cover.map(|path| path.to_string_lossy().into_owned()),
                cover_pending,
                publish: record.publish,
                changes: dto.changes,
            });
        }
        // Changelog 46 (1): creation order, newest created on top, grouped by creation time;
        // opening, editing, capturing into or publishing a session never moves it.
        let group_ids = group_by_activity(
            &rows
                .iter()
                .map(|r| (r.id, r.created_at))
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

impl SessionsService<'_> {
    /// MC-H2.1: one page of `list(filter, query)`: its rows kept when created at or after
    /// `created_from` and whose id is in `only_ids` (each when given; `Some([])` keeps none), in
    /// the list's order, cut to the page (`page_of`); each group keeps its row count before the
    /// cut, empty groups are left out. Then the page's pending covers are requested (MC-H1.3);
    /// nothing is downloaded here.
    pub fn page(
        &self,
        filter: crate::domain::session::ListFilter,
        query: Option<String>,
        created_from: Option<i64>,
        only_ids: Option<&[i64]>,
        page: u32,
    ) -> Result<crate::domain::session::SessionPage> {
        use crate::domain::session::{PageGroup, SessionPage};
        let list = self.list(filter, query)?;
        let kept = |row: &crate::domain::session::SessionRow| {
            created_from.is_none_or(|from| row.created_at >= from)
                && only_ids.is_none_or(|ids| ids.contains(&row.id))
        };
        let groups: Vec<(String, Vec<crate::domain::session::SessionRow>)> = list
            .groups
            .into_iter()
            .map(|group| (group.label, group.rows.into_iter().filter(kept).collect()))
            .collect();
        let total = groups.iter().map(|(_, rows)| rows.len()).sum();
        let (page, pages, range) = crate::mechanisms::remote::page_of(total, page);
        let mut index = 0;
        let mut cut = Vec::new();
        for (label, rows) in groups {
            let group_total = rows.len();
            let start = index;
            index += group_total;
            let rows: Vec<_> = rows
                .into_iter()
                .enumerate()
                .filter(|(i, _)| range.contains(&(start + i)))
                .map(|(_, row)| row)
                .collect();
            if !rows.is_empty() {
                cut.push(PageGroup {
                    label,
                    total: group_total,
                    rows,
                });
            }
        }
        if let Some(remote) = self.remote {
            let pending: Vec<i64> = cut
                .iter()
                .flat_map(|group| &group.rows)
                .filter(|row| row.cover_pending)
                .map(|row| row.id)
                .collect();
            remote.request_covers(crate::services::remote::CoverSource::Page, &pending)?;
        }
        Ok(SessionPage {
            groups: cut,
            counts: list.counts,
            unassigned_count: list.unassigned_count,
            page,
            pages,
            total,
        })
    }
}

/// MC-H3.7A (09C §14.3): the list status of a row. A `Remote(ShopOnly)` row labelled
/// "Archived in Shopify" is `Published`; every other row is `derive_status(input)`.
fn row_status(
    remote_row: Option<&crate::services::remote::RemoteRow>,
    label: crate::domain::session::RowLabel,
    input: crate::mechanisms::status::StatusInput,
) -> crate::domain::session::SessionStatus {
    use crate::domain::remote::{RemoteKind, RemoteState};
    use crate::domain::session::{RowLabel, SessionStatus};
    match remote_row {
        Some(row)
            if row.state == RemoteState::Remote(RemoteKind::ShopOnly)
                && label == RowLabel::Archived =>
        {
            SessionStatus::Published
        }
        _ => crate::mechanisms::status::derive_status(input),
    }
}

/// The status inputs of one session-list record (07).
fn status_input(
    record: &crate::domain::session::SessionListRecord,
) -> crate::mechanisms::status::StatusInput {
    let dto = &record.session;
    crate::mechanisms::status::StatusInput {
        latest_step: record.latest_step,
        failed_step: record.failed_step,
        published_at: dto.session.published_at,
        dirty: dto.dirty,
        main_count: dto.main_count,
        unreviewed_count: dto.unreviewed_count,
        non_deleted_missing_preview_count: record.non_deleted_missing_preview_count,
        uploaded: record.uploaded,
        total: record.total,
        unresolved_conflicts: record.unresolved_conflicts,
    }
}

/// P69 (changelog 47 (2)): the item sessions whose status is Ready (07) and whose files were not
/// cleared after publishing, newest created first (the list's order, changelog 46 (1)).
pub fn ready_session_ids(store: &dyn Store) -> Result<Vec<i64>> {
    use crate::domain::session::SessionStatus;
    let cleared = crate::services::retention::cleared_sessions(store)?;
    let mut ready: Vec<(i64, i64)> = store
        .session_list_records()?
        .iter()
        .filter(|record| {
            record.session.session.item_id.is_some()
                && !cleared.contains(&record.session.session.id)
                && crate::mechanisms::status::derive_status(status_input(record))
                    == SessionStatus::Ready
        })
        .map(|record| (record.session.session.created_at, record.session.session.id))
        .collect();
    ready.sort_by(|a, b| b.cmp(a));
    Ok(ready.into_iter().map(|(_, id)| id).collect())
}

/// A10 (owner 2026-11-18): one log line per Capture One step of a capture-target switch (opening
/// a session, going back to the list), and one for the whole switch, so the owner's logs show
/// where "Capture One stops opening the session" fails. Folder names only, never full paths.
struct SwitchLog<'s> {
    reason: &'static str,
    from: String,
    to: &'s str,
    folder: &'s str,
    started: std::time::Instant,
    /// A20: the editor whose scripts the switch counts, and its count at the start.
    editing: &'s dyn EditingProvider,
    calls_before: Option<u64>,
}
impl<'s> SwitchLog<'s> {
    fn new(
        reason: &'static str,
        from: Option<i64>,
        to: &'s str,
        folder: &'s str,
        editing: &'s dyn EditingProvider,
    ) -> Self {
        Self {
            reason,
            from: from.map_or_else(|| "Unassigned".into(), |id| format!("session {id}")),
            to,
            folder,
            started: std::time::Instant::now(),
            editing,
            calls_before: editing.script_calls(),
        }
    }
    fn step<T>(&self, step: &str, run: impl FnOnce() -> Result<T>) -> Result<T> {
        let started = std::time::Instant::now();
        let result = run();
        tracing::info!(
            target: "psm::capture_one",
            "capture_one.switch_step reason={} to=\"{}\" folder={} step={} ms={} result={}",
            self.reason,
            self.to,
            self.folder,
            step,
            started.elapsed().as_millis(),
            result
                .as_ref()
                .map_or_else(|e| format!("error: {e}"), |_| "ok".into()),
        );
        result
    }
    fn done(&self, result: &Result<()>) {
        tracing::info!(
            target: "psm::capture_one",
            "capture_one.switch reason={} from=\"{}\" to=\"{}\" folder={} ms={} result={}",
            self.reason,
            self.from,
            self.to,
            self.folder,
            self.started.elapsed().as_millis(),
            result
                .as_ref()
                .map_or_else(|e| format!("error: {e}"), |_| "ok".into()),
        );
        // A20: how many scripts the switch sent to the editor (the cost per switch must stay flat).
        if let (Some(before), Some(after)) = (self.calls_before, self.editing.script_calls()) {
            tracing::info!(
                target: "psm::capture_one",
                "capture_one.switch_scripts reason={} to=\"{}\" scripts={}",
                self.reason,
                self.to,
                after.saturating_sub(before),
            );
        }
    }
}

/// A10 (owner 2026-11-18, "Capture One sometimes stops opening the session"): opening sessions
/// one after another sent overlapping switch requests (each on its own worker), and whichever
/// finished last won, so Capture One and the capture target could end on a session the screen no
/// longer showed. Requests now take a ticket in the order they are made and run one at a time; a
/// request that a newer one has overtaken is skipped, so the last request asked for is the one
/// that holds.
pub struct SwitchOrder {
    lock: std::sync::Mutex<()>,
    /// A16: the short lock the filing-target claims take (never held while Capture One works).
    claim_lock: std::sync::Mutex<()>,
    latest: std::sync::atomic::AtomicU64,
}
impl SwitchOrder {
    pub const fn new() -> Self {
        Self {
            lock: std::sync::Mutex::new(()),
            claim_lock: std::sync::Mutex::new(()),
            latest: std::sync::atomic::AtomicU64::new(0),
        }
    }
    /// A16 (owner 2026-11-18, "the pictures I took for the next session arrive in the previous
    /// one"): sets where photos are filed for this request at once, without waiting for the
    /// Capture One switches before it (which held the lock for minutes). A claim a newer request
    /// has overtaken is skipped, so the last request asked for holds here too.
    pub fn claim(&self, ticket: u64, claim: impl FnOnce() -> Result<bool>) -> Result<bool> {
        let _turn = self.claim_lock.lock().unwrap_or_else(|e| e.into_inner());
        if self.latest.load(std::sync::atomic::Ordering::SeqCst) != ticket {
            return Ok(false);
        }
        claim()
    }
    /// Taken when the request is made (before any waiting).
    pub fn ticket(&self) -> u64 {
        self.latest
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            + 1
    }
    /// Runs `switch` unless a newer request has been made since `ticket` was taken.
    pub fn run(
        &self,
        ticket: u64,
        request: &str,
        switch: impl FnOnce() -> Result<()>,
    ) -> Result<()> {
        let _turn = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        if self.latest.load(std::sync::atomic::Ordering::SeqCst) != ticket {
            tracing::info!(
                target: "psm::capture_one",
                "capture_one.switch_skipped request=\"{request}\" reason=a_newer_request_follows"
            );
            return Ok(());
        }
        switch()
    }
}
impl Default for SwitchOrder {
    fn default() -> Self {
        Self::new()
    }
}
/// The one order every capture-target switch of the app goes through.
pub static CAPTURE_TARGET_ORDER: SwitchOrder = SwitchOrder::new();
