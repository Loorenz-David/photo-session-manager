//! The labels wiring (book 09C; `master_plan_labels.md` §4 Service): builds `LabelsService` over
//! the Beyo Pricing adapter the conversations service already holds, and tells the conversations
//! service where to send what it learns (`set_labels_link`, a `Weak`: no cycle).
//!
//! L3: the label files (`DiskLabelFiles` on the previews folder and the labels folder, both from
//! `AppPaths`), with the suite's seam `with_test_label_files`.

use psm_core::{
    ports::{
        clock::Clock,
        commerce::CommerceAdapter,
        events::EventSink,
        labels::{LabelFiles, LabelsLink, LabelsProvider},
    },
    services::{conversations::ConversationsService, labels::LabelsService, remote::RemoteItems},
};
use psm_infra::{label_files::DiskLabelFiles, paths::AppPaths, sqlite::store::SqliteStore};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock, Weak},
};

/// The labels folder under the home folder (`master_plan_labels.md` §4 Files).
pub const LABELS_FOLDER: &str = "Documents/Session Manager Labels";

type TestFiles = Mutex<HashMap<PathBuf, Arc<dyn LabelFiles>>>;
static TEST_FILES: OnceLock<TestFiles> = OnceLock::new();

/// The suite's stand-in for the label files of the app at `paths` (keyed by the data folder,
/// survives a reopen): no test runs `sips` or `open` through the app, nor writes into the real
/// Documents folder.
pub fn with_test_label_files(paths: &AppPaths, files: Arc<dyn LabelFiles>) {
    if let Ok(mut all) = TEST_FILES.get_or_init(Default::default).lock() {
        all.insert(paths.root.clone(), files);
    }
}

/// The label files of the app at `paths`: the test seam when one is installed, else the disk
/// (previews under `<data>/previews`, the labels folder under `AppPaths.home`: the one place the
/// labels folder is derived).
pub fn files_for(paths: &AppPaths) -> Arc<dyn LabelFiles> {
    if let Some(files) = TEST_FILES
        .get()
        .and_then(|all| all.lock().ok()?.get(&paths.root).cloned())
    {
        return files;
    }
    Arc::new(DiskLabelFiles::new(
        paths.root.join("previews"),
        paths.home.join(LABELS_FOLDER),
    ))
}

/// The pieces `App::build` hands the labels wiring.
pub struct ServiceParts {
    pub conversations: Arc<ConversationsService>,
    /// The same adapter instance as the conversations service's.
    pub provider: Arc<dyn LabelsProvider>,
    pub store: Arc<SqliteStore>,
    /// L2: the row pictures of matched sessions and the search-picture cache of the others.
    pub remote: Arc<RemoteItems>,
    /// L2: the shop an unmatched row is adopted from (none without a commerce integration).
    pub commerce: Option<Arc<dyn CommerceAdapter>>,
    /// L3: the preview pictures and saved label files ([`files_for`]).
    pub files: Arc<dyn LabelFiles>,
    pub clock: Arc<dyn Clock>,
    pub events: Arc<dyn EventSink>,
}

pub fn build_service(parts: ServiceParts) -> Arc<LabelsService> {
    let labels = Arc::new(LabelsService::new(
        parts.conversations.clone(),
        parts.provider,
        parts.store.clone(),
        parts.store,
        parts.remote,
        parts.commerce,
        parts.files,
        parts.clock,
        parts.events,
    ));
    let weak: Weak<LabelsService> = Arc::downgrade(&labels);
    let link: Weak<dyn LabelsLink> = weak;
    parts.conversations.set_labels_link(link);
    labels
}
