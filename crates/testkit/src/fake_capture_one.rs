use psm_core::{Error, Result};
use psm_integrations::editing::capture_one::recipes::RecipeSpec;
use psm_integrations::editing::capture_one::scripting::{
    CaptureOneScripting, CollectionInfo, DocumentInfo, EditorInfo, RecipeEnsure, RecipeProps,
    StyleEnsure, StyleList,
};
use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

/// P65: a photo the app will file that Capture One lists only after `synchronizing = true` was set
/// on its folder (then after `sync_delay_ms`), or after `unsynced_listing_ms` on its own.
#[derive(Debug, Clone)]
pub struct PendingImage {
    pub variant: String,
    /// The RAW's path once the fake has seen it (a sync of its folder or a lookup).
    pub raw: Option<String>,
    pub first_seen_ms: Option<i64>,
    pub synced_ms: Option<i64>,
}

#[derive(Clone)]
pub struct FakeVariant {
    pub id: String,
    pub tag: i32,
    pub geometry: psm_core::domain::composition::EditorGeometry,
    pub styles: Vec<String>,
}
pub struct FakeState {
    pub document_id: String,
    pub kind: String,
    pub captures: String,
    pub current_collection: String,
    pub fail_set_current_collection: bool,
    pub capture_root: String,
    pub collections: Vec<CollectionInfo>,
    pub delay_document_info_ms: u64,
    pub call_delay_ms: u64,
    pub fail_process: bool,
    pub camera: Option<String>,
    pub recipes: HashMap<String, RecipeProps>,
    pub running: bool,
    pub pro: bool,
    pub automation_denied: bool,
    pub launch_fails: Option<String>,
    pub launched: bool,
    pub running_after_polls: Option<u32>,
    /// After `quit`, `running` turns false once this many `editor_info` calls asked (`None`:
    /// it never quits — a dialog is open). Default `Some(1)`.
    pub quit_after_polls: Option<u32>,
    /// The countdown of a `quit` in progress.
    pub quitting: Option<u32>,
    /// The `editor_info` after the first one that reports not-running after a quit reports
    /// `running = true` (the photographer opened it), with no `launch` op (P48 C5(e)).
    pub restart_after_quit: bool,
    /// The last `editor_info` was the first to report not-running after a quit.
    pub quit_seen: bool,
    /// `save` fails (P48 C5(b)).
    pub fail_save: bool,
    pub open_documents: Vec<String>,
    pub front_document: Option<String>,
    pub fail_make_document: Option<String>,
    pub fail_open_document: Option<String>,
    pub fail_front: bool,
    pub fail_make_recipe: Option<String>,
    pub styles: Vec<String>,
    pub next_capture_styles: Vec<String>,
    pub hooks: HashMap<String, String>,
    pub images: HashMap<String, Vec<FakeVariant>>,
    pub image_collections: HashMap<String, String>,
    pub folder_images: HashMap<String, Vec<FakeVariant>>,
    pub ops: Vec<String>,
    pub private_prefix: bool,
    pub fail_next_set_captures: bool,
    pub readback_garbage: bool,
    pub timeout_next_set_tag: bool,
    pub drop_tag_writes: bool,
    pub hooks_readback_null: bool,
    pub next_job: i64,
    /// Every `process` call in order: (job id, variant, recipe).
    pub issued: Vec<(String, String, String)>,
    /// P56 (P52 F12): Capture One's "Existing files: add suffix". An output whose name exists is
    /// written as `<stem> <n>.<ext>`, while the processing-done hook still reports `<stem>.<ext>`.
    pub add_suffix: bool,
    /// P56: processing a variant rewrites its settings sidecar (`.cos`) with the adjustments
    /// unchanged and Capture One's processing bookkeeping bumped, one second after the output.
    pub touch_settings_on_process: bool,
    /// The adjustments of each RAW's settings sidecar the fake writes (raw → exposure).
    pub settings: HashMap<String, i64>,
    /// Capture One's processing bookkeeping (`LowResProcessCounter`).
    pub process_counter: u64,
    /// P60 #22: an Apple event reaching a Capture One that is not running relaunches it (a script
    /// already under way when it quit): any document script, after its `call_delay_ms`, records
    /// op `launch` and sets `running`.
    pub apple_events_launch: bool,
    /// P63: the names of `styles` that are the photographer's own (listed first by `list_styles`).
    pub user_styles: Vec<String>,
    /// P63: `list_styles` fails (the list cannot be read).
    pub fail_list_styles: bool,
    /// P65: file name → a photo Capture One has not listed yet (see `PendingImage`).
    pub pending_images: HashMap<String, PendingImage>,
    /// P65: after a sync, a pending photo is listed this much later (default 3 s).
    pub sync_delay_ms: i64,
    /// P65: without a sync, a pending photo is listed this long after it was first seen (60 s,
    /// the dev Mac's folder check on 2026-11-11 took 17–80 s).
    pub unsynced_listing_ms: i64,
}
/// One look at whether Capture One runs (`editor_info` or `is_running`): advances a quit or a
/// launch in progress, as the dev Mac's Capture One does between two looks.
fn poll_running(s: &mut FakeState) {
    if std::mem::take(&mut s.quit_seen) && std::mem::take(&mut s.restart_after_quit) {
        s.running = true;
    }
    if let Some(polls) = s.quitting.as_mut() {
        if *polls != u32::MAX {
            *polls = polls.saturating_sub(1);
        }
        if *polls == 0 {
            s.quitting = None;
            s.running = false;
            s.quit_seen = true;
        }
    }
    if s.launched
        && let Some(polls) = s.running_after_polls.as_mut()
        && *polls > 0
    {
        *polls -= 1;
        if *polls == 0 {
            s.running = true;
        }
    }
}
pub struct FakeCaptureOneScripting {
    pub state: Mutex<FakeState>,
    /// P65: the test's clock (pending photos are listed by its time); unset = time 0.
    clock: Mutex<Option<Arc<dyn psm_core::ports::clock::Clock>>>,
}
impl FakeCaptureOneScripting {
    pub fn get_color_tag(&self, variant: &str) -> Result<i32> {
        let id = self.state.lock().unwrap().document_id.clone();
        CaptureOneScripting::get_color_tag(self, &id, variant)
    }
    pub fn new(document_id: String, capture_root: String) -> Self {
        let mut recipes = HashMap::new();
        for (name, spec) in [
            ("PSM Preview", RecipeSpec::PREVIEW),
            ("PSM Master", RecipeSpec::MASTER),
        ] {
            let mut props = spec.props();
            if name == "PSM Preview" {
                props.primary_scaling_value = 1600;
                props.jpeg_quality = 80;
                props.export_crop_method = "respect".into();
                props.bits = 16;
            }
            props.existing_files = "add suffix".into();
            props.output_sub_folder = String::new();
            props.root_folder_type = Some("output location".into());
            recipes.insert(name.into(), props);
        }
        Self {
            state: Mutex::new(FakeState {
                open_documents: vec![document_id.clone()],
                front_document: Some(document_id.clone()),
                document_id,
                running: true,
                pro: true,
                automation_denied: false,
                launch_fails: None,
                launched: false,
                running_after_polls: None,
                quit_after_polls: Some(1),
                quitting: None,
                restart_after_quit: false,
                quit_seen: false,
                fail_save: false,
                fail_make_document: None,
                fail_open_document: None,
                fail_front: false,
                fail_make_recipe: None,
                styles: vec!["PSM House".into()],
                next_capture_styles: Vec::new(),
                kind: "session".into(),
                captures: capture_root.clone(),
                current_collection: capture_root.clone(),
                fail_set_current_collection: false,
                collections: vec![
                    CollectionInfo {
                        id: "3".into(),
                        name: "Capture".into(),
                        folder: Some(capture_root.clone()),
                    },
                    CollectionInfo {
                        id: "capture".into(),
                        name: "Capture".into(),
                        folder: Some(capture_root.clone()),
                    },
                ],
                capture_root,
                delay_document_info_ms: 0,
                call_delay_ms: 0,
                fail_process: false,
                camera: Some("Sony ZV-E10".into()),
                recipes,
                hooks: HashMap::new(),
                images: HashMap::new(),
                image_collections: HashMap::new(),
                folder_images: HashMap::new(),
                ops: Vec::new(),
                private_prefix: false,
                fail_next_set_captures: false,
                readback_garbage: false,
                timeout_next_set_tag: false,
                drop_tag_writes: false,
                hooks_readback_null: false,
                next_job: 1,
                issued: Vec::new(),
                add_suffix: false,
                touch_settings_on_process: false,
                settings: HashMap::new(),
                process_counter: 0,
                apple_events_launch: false,
                user_styles: Vec::new(),
                fail_list_styles: false,
                pending_images: HashMap::new(),
                sync_delay_ms: 3_000,
                unsynced_listing_ms: 60_000,
            }),
            clock: Mutex::new(None),
        }
    }
    /// P65: the clock pending photos are listed by.
    pub fn set_clock(&self, clock: Arc<dyn psm_core::ports::clock::Clock>) {
        *self.clock.lock().unwrap() = Some(clock);
    }
    fn now_ms(&self) -> i64 {
        self.clock
            .lock()
            .unwrap()
            .as_ref()
            .map_or(0, |clock| clock.now_ms())
    }
    /// P65: the photo named `file_name` (once filed) is listed as variant `variant` only after a
    /// sync of its folder plus `sync_delay_ms`, else `unsynced_listing_ms` after it is first seen.
    pub fn list_after_sync(&self, file_name: &str, variant: &str) {
        self.state.lock().unwrap().pending_images.insert(
            file_name.into(),
            PendingImage {
                variant: variant.into(),
                raw: None,
                first_seen_ms: None,
                synced_ms: None,
            },
        );
    }
    /// Lists the pending photos whose time has come (as Capture One's folder check would).
    fn list_pending(&self, s: &mut FakeState, seen_raw: Option<&str>) {
        let now = self.now_ms();
        if let Some(raw) = seen_raw
            && let Some(name) = Path::new(raw).file_name().map(|n| n.to_string_lossy().into_owned())
            && let Some(pending) = s.pending_images.get_mut(&name)
        {
            pending.raw.get_or_insert_with(|| raw.to_string());
            pending.first_seen_ms.get_or_insert(now);
        }
        let (delay, unsynced) = (s.sync_delay_ms, s.unsynced_listing_ms);
        let ready: Vec<String> = s
            .pending_images
            .iter()
            .filter(|(_, p)| {
                p.raw.is_some()
                    && (p.synced_ms.is_some_and(|at| now >= at + delay)
                        || p.first_seen_ms.is_some_and(|at| now >= at + unsynced))
            })
            .map(|(name, _)| name.clone())
            .collect();
        for name in ready {
            let pending = s.pending_images.remove(&name).unwrap();
            let raw = pending.raw.unwrap();
            let folder = Path::new(&raw)
                .parent()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();
            s.ops.push(format!("listed:{raw}"));
            s.image_collections.insert(raw.clone(), folder);
            s.images.insert(
                raw,
                vec![FakeVariant {
                    id: pending.variant,
                    tag: 3,
                    geometry: Default::default(),
                    styles: Vec::new(),
                }],
            );
        }
    }
    /// The output file Capture One writes for render job `job`: a JPEG (SOI … EOI) whose bytes
    /// carry the job id, so every recorded or exported file can be traced to the job that made it.
    fn check_id(&self, id: &str) -> Result<()> {
        let (expected, delay) = {
            let s = self.state.lock().unwrap();
            (
                s.open_documents.iter().any(|open| open == id),
                s.call_delay_ms,
            )
        };
        if delay > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
        {
            let mut s = self.state.lock().unwrap();
            if s.apple_events_launch && !s.running {
                s.ops.push("launch".into());
                s.running = true;
            }
        }
        if !expected {
            Err(Error::DocumentNotOpen)
        } else {
            Ok(())
        }
    }
    pub fn output_bytes(job: &str) -> Vec<u8> {
        let mut bytes = vec![0xFF, 0xD8, 0xFF, 0xE0];
        bytes.extend_from_slice(format!("image from {job}").as_bytes());
        bytes.extend_from_slice(&[0xFF, 0xD9]);
        bytes
    }
    /// Writes a finished render's output as Capture One names it, into `dir`, with `mtime`, and
    /// returns `(written, reported)`: the file on disk and the path the processing-done hook
    /// reports. With `add_suffix` and `<stem>.<ext>` present the file is `<stem> <n>.<ext>` (the
    /// first free n) and the report is still `<stem>.<ext>` (P52 F12). With
    /// `touch_settings_on_process` the RAW's sidecar is rewritten one second later.
    pub fn write_output(
        &self,
        raw: &str,
        dir: &Path,
        extension: &str,
        bytes: &[u8],
        mtime: std::time::SystemTime,
    ) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
        let stem = Path::new(raw)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        fs::create_dir_all(dir)?;
        let reported = dir.join(format!("{stem}.{extension}"));
        let (add_suffix, touch) = {
            let s = self.state.lock().unwrap();
            (s.add_suffix, s.touch_settings_on_process)
        };
        let mut written = reported.clone();
        if add_suffix {
            let mut n = 1;
            while written.exists() {
                written = dir.join(format!("{stem} {n}.{extension}"));
                n += 1;
            }
        }
        write_with_mtime(&written, bytes, mtime)?;
        if touch {
            self.state.lock().unwrap().process_counter += 1;
            self.write_settings_file(raw, mtime + std::time::Duration::from_secs(1))?;
        }
        Ok((written, reported))
    }
    /// The photographer changes the RAW's adjustments in Capture One; its autosave writes the
    /// sidecar at `mtime`.
    pub fn edit_settings(
        &self,
        raw: &str,
        exposure: i64,
        mtime: std::time::SystemTime,
    ) -> Result<()> {
        self.state
            .lock()
            .unwrap()
            .settings
            .insert(raw.into(), exposure);
        self.write_settings_file(raw, mtime)
    }
    /// Capture One rewrites the RAW's sidecar without changing an adjustment (a document save).
    pub fn rewrite_settings(&self, raw: &str, mtime: std::time::SystemTime) -> Result<()> {
        self.state.lock().unwrap().process_counter += 1;
        self.write_settings_file(raw, mtime)
    }
    /// `<raw folder>/CaptureOne/Settings1680/<raw name>.cos` in Capture One's format: bookkeeping
    /// outside the adjustment list (`<DL>`), the adjustments inside it.
    fn write_settings_file(&self, raw: &str, mtime: std::time::SystemTime) -> Result<()> {
        let (exposure, counter) = {
            let s = self.state.lock().unwrap();
            (s.settings.get(raw).copied().unwrap_or(0), s.process_counter)
        };
        let raw = Path::new(raw);
        let folder = raw.parent().unwrap().join("CaptureOne/Settings1680");
        fs::create_dir_all(&folder)?;
        let text = format!(
            "<?xml version=\"1.0\"?>\n<IMG>\n<E K=\"Author\" V=\"16.8.6.25 Pro Mac\" />\n\t<VAR Engine=\"1300\">\n\t\t<E K=\"LastMetadataModificationDate\" V=\"{counter}.5\" />\n\t\t<E K=\"LowResProcessCounter\" V=\"{counter}\" />\n\t\t<DL>\n\t\t\t<E K=\"Basic_Rating\" V=\"0\" />\n\t\t\t<E K=\"Exposure\" V=\"{exposure}\" />\n\t\t</DL>\n\t</VAR>\n</IMG>\n"
        );
        write_with_mtime(
            &folder.join(format!(
                "{}.cos",
                raw.file_name().unwrap().to_string_lossy()
            )),
            text.as_bytes(),
            mtime,
        )
    }
    /// The job ids `process` returned for `variant`, oldest first.
    pub fn jobs_for(&self, variant: &str) -> Vec<String> {
        self.state
            .lock()
            .unwrap()
            .issued
            .iter()
            .filter(|(_, v, _)| v == variant)
            .map(|(job, _, _)| job.clone())
            .collect()
    }
    pub fn add_image(&self, raw: &str, variants: &[&str], in_folder: bool) {
        let mut s = self.state.lock().unwrap();
        let value = variants
            .iter()
            .map(|id| FakeVariant {
                id: (*id).into(),
                tag: 3,
                geometry: Default::default(),
                styles: Vec::new(),
            })
            .collect();
        if in_folder {
            s.folder_images.insert(raw.into(), value);
        } else {
            let current = s.current_collection.clone();
            s.image_collections.insert(raw.into(), current);
            s.images.insert(raw.into(), value);
        }
    }
    pub fn set_variant_geometry(
        &self,
        variant: &str,
        geometry: psm_core::domain::composition::EditorGeometry,
    ) {
        let mut s = self.state.lock().unwrap();
        for v in s.images.values_mut().flat_map(|v| v.iter_mut()) {
            if v.id == variant {
                v.geometry = geometry.clone();
            }
        }
        for v in s.folder_images.values_mut().flat_map(|v| v.iter_mut()) {
            if v.id == variant {
                v.geometry = geometry.clone();
            }
        }
    }
}
fn write_with_mtime(path: &Path, bytes: &[u8], mtime: std::time::SystemTime) -> Result<()> {
    fs::write(path, bytes)?;
    fs::File::options()
        .write(true)
        .open(path)?
        .set_times(fs::FileTimes::new().set_modified(mtime))?;
    Ok(())
}
impl CaptureOneScripting for FakeCaptureOneScripting {
    /// The process list, not a script: recorded as no op (P63 #22).
    fn is_running(&self) -> Result<bool> {
        let mut s = self.state.lock().unwrap();
        poll_running(&mut s);
        Ok(s.running)
    }
    fn list_styles(&self) -> Result<StyleList> {
        let mut s = self.state.lock().unwrap();
        s.ops.push("list_styles".into());
        if s.fail_list_styles || !s.running {
            return Err(Error::Provider("styles unreadable".into()));
        }
        let (user, built_in) = s
            .styles
            .iter()
            .cloned()
            .partition(|name| s.user_styles.contains(name));
        Ok(StyleList { user, built_in })
    }
    fn editor_info(&self) -> Result<EditorInfo> {
        let mut s = self.state.lock().unwrap();
        s.ops.push("editor_info".into());
        if s.automation_denied {
            return Err(Error::Provider(
                "execution error: Not authorized to send Apple events to Capture One. (-1743)"
                    .into(),
            ));
        }
        poll_running(&mut s);
        Ok(EditorInfo {
            running: s.running,
            version: Some("16.8.6".into()),
            pro: s.running.then_some(s.pro),
        })
    }
    fn launch(&self) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        s.ops.push("launch".into());
        if let Some(error) = &s.launch_fails {
            return Err(Error::Provider(error.clone()));
        }
        s.launched = true;
        Ok(())
    }
    fn make_document(&self, parent: &str, name: &str) -> Result<String> {
        let mut s = self.state.lock().unwrap();
        let folder = Path::new(parent).join(name);
        let id = folder.to_string_lossy().into_owned();
        s.ops.push(format!("make_document:{id}"));
        if let Some(error) = &s.fail_make_document {
            return Err(Error::Provider(error.clone()));
        }
        fs::create_dir_all(&folder)?;
        for sub in ["Capture", "Selects", "Output", "Trash"] {
            fs::create_dir_all(folder.join(sub))?;
        }
        fs::write(folder.join(format!("{name}.cosessiondb")), b"")?;
        s.open_documents.push(id.clone());
        s.front_document = Some(id.clone());
        s.document_id = id.clone();
        s.capture_root = folder.join("Capture").to_string_lossy().into_owned();
        s.captures = s.capture_root.clone();
        Ok(id)
    }
    fn open_document(&self, file: &str) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("open_document:{file}"));
        if let Some(error) = &s.fail_open_document {
            return Err(Error::Provider(error.clone()));
        }
        if !Path::new(file).is_file() {
            return Err(Error::DocumentNotOpen);
        }
        let id = Path::new(file)
            .parent()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        if !s.open_documents.contains(&id) {
            s.open_documents.push(id.clone());
        }
        s.front_document = Some(id);
        Ok(())
    }
    fn front_document(&self, id: &str) -> Result<String> {
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("front:{id}"));
        if !s.open_documents.iter().any(|d| d == id) {
            return Err(Error::DocumentNotOpen);
        }
        if !s.fail_front {
            s.front_document = Some(id.into());
        }
        Ok(s.front_document.clone().unwrap_or_default())
    }
    fn ensure_recipe(&self, id: &str, name: &str, want: &RecipeProps) -> Result<RecipeEnsure> {
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("ensure_recipe:{name}"));
        let created = !s.recipes.contains_key(name);
        if created {
            if let Some(error) = &s.fail_make_recipe {
                return Err(Error::Provider(error.clone()));
            }
            s.ops.push(format!("make_recipe:{name}"));
            let mut defaults = RecipeSpec::PREVIEW.props();
            defaults.enabled = false;
            defaults.output_format = "TIFF".into();
            defaults.bits = 16;
            defaults.tiff_compression = "Uncompressed".into();
            defaults.export_crop_method = "respect".into();
            defaults.color_profile = "Adobe RGB (1998)".into();
            defaults.scaling_method = "Fixed".into();
            defaults.scaling_unit = "percent".into();
            defaults.primary_scaling_value = 100;
            defaults.jpeg_quality = 100;
            defaults.existing_files = "add suffix".into();
            defaults.output_sub_folder = String::new();
            defaults.root_folder_type = Some("output location".into());
            s.recipes.insert(name.into(), defaults);
        }
        let mut props = s.recipes.get(name).unwrap().clone();
        let mut changed = Vec::new();
        macro_rules! repair {
            ($field:ident,$label:literal) => {
                if props.$field != want.$field {
                    props.$field = want.$field.clone();
                    changed.push($label.into());
                }
            };
        }
        repair!(enabled, "enabled");
        repair!(output_format, "outputFormat");
        repair!(bits, "bits");
        repair!(tiff_compression, "tiffCompression");
        repair!(export_crop_method, "exportCropMethod");
        repair!(color_profile, "colorProfile");
        repair!(scaling_method, "scalingMethod");
        repair!(scaling_unit, "scalingUnit");
        repair!(primary_scaling_value, "primaryScalingValue");
        repair!(jpeg_quality, "jpegQuality");
        repair!(existing_files, "existingFiles");
        repair!(output_sub_folder, "outputSubFolder");
        if !changed.is_empty() {
            s.ops
                .push(format!("set_recipe_props:{name}:{}", changed.join(",")));
        }
        s.recipes.insert(name.into(), props.clone());
        Ok(RecipeEnsure {
            created,
            changed,
            props,
        })
    }
    fn ensure_capture_style(&self, id: &str, name: &str) -> Result<StyleEnsure> {
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("ensure_capture_style:{name}"));
        let listed = s.styles.iter().any(|style| style == name);
        if listed && s.next_capture_styles != [name] {
            s.next_capture_styles = vec![name.into()];
            s.ops.push(format!("set_next_capture_styles:{name}"));
        }
        Ok(StyleEnsure {
            listed,
            applied: s.next_capture_styles.clone(),
        })
    }
    fn document_info(&self, id: &str) -> Result<DocumentInfo> {
        let delay = self.state.lock().unwrap().delay_document_info_ms;
        self.state
            .lock()
            .unwrap()
            .ops
            .push("document_info_started".into());
        if delay > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
        let mut s = self.state.lock().unwrap();
        if s.apple_events_launch && !s.running {
            s.ops.push("launch".into());
            s.running = true;
        }
        s.ops.push("document_info".into());
        if !s.open_documents.iter().any(|d| d == id) {
            return Err(Error::DocumentNotOpen);
        };
        if s.automation_denied {
            return Err(Error::Provider(
                "execution error: Not authorized to send Apple events to Capture One. (-1743)"
                    .into(),
            ));
        }
        if id != s.document_id {
            return Ok(DocumentInfo {
                kind: "session".into(),
                id: id.into(),
                captures: format!("{id}/Capture"),
                collections: Vec::new(),
                camera: None,
            });
        }
        Ok(DocumentInfo {
            kind: s.kind.clone(),
            id: s.document_id.clone(),
            captures: s.captures.clone(),
            collections: s.collections.clone(),
            camera: s.camera.clone(),
        })
    }
    fn save(&self, id: &str) -> Result<()> {
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        s.ops.push("save".into());
        if s.fail_save {
            return Err(Error::Provider(
                "Capture One could not save the document".into(),
            ));
        }
        Ok(())
    }
    fn set_captures(&self, id: &str, path: &str) -> Result<()> {
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("set_captures:{path}"));
        if s.fail_next_set_captures {
            s.fail_next_set_captures = false;
            s.captures = format!("{}/Capture", std::env::current_dir()?.display());
            return Err(Error::Provider(
                "Cannot set because there is no file at this location".into(),
            ));
        }
        if !Path::new(path).is_dir() {
            s.captures = format!("{}/Capture", std::env::current_dir()?.display());
            return Err(Error::Provider(
                "Cannot set because there is no file at this location".into(),
            ));
        }
        s.captures = path.into();
        s.collections[1].folder = Some(path.into());
        s.collections[1].name = Path::new(path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        Ok(())
    }
    fn set_current_collection(&self, id: &str, folder: &str) -> Result<()> {
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        if !s.running {
            return Err(Error::DocumentNotOpen);
        }
        if s.fail_set_current_collection {
            return Err(Error::Provider("current collection not set".into()));
        }
        if !Path::new(folder).is_dir() {
            return Err(Error::Provider("current collection not found".into()));
        }
        if s.current_collection != folder {
            s.ops.push(format!("set_current_collection:{folder}"));
            s.current_collection = folder.into();
        }
        Ok(())
    }
    /// P65: `synchronizing = true` on the folder's collection: pending photos filed in it are
    /// listed `sync_delay_ms` later. Op `sync_folder:<folder>`.
    fn sync_folder(&self, id: &str, folder: &str) -> Result<()> {
        self.check_id(id)?;
        let now = self.now_ms();
        let mut s = self.state.lock().unwrap();
        if !s.running {
            return Err(Error::Provider("Capture One is not running".into()));
        }
        s.ops.push(format!("sync_folder:{folder}"));
        for (name, pending) in s.pending_images.iter_mut() {
            let raw = Path::new(folder).join(name);
            if raw.is_file() {
                pending
                    .raw
                    .get_or_insert_with(|| raw.to_string_lossy().into_owned());
                pending.first_seen_ms.get_or_insert(now);
                pending.synced_ms = Some(now);
            }
        }
        Ok(())
    }
    fn apply_style(&self, id: &str, variant: &str, name: &str) -> Result<bool> {
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        if !s.styles.iter().any(|v| v == name) {
            return Ok(false);
        }
        let mut found = false;
        for value in s.images.values_mut().flatten() {
            if value.id == variant {
                if value.styles.contains(&name.to_string()) {
                    return Ok(true);
                }
                value.styles.push(name.into());
                found = true;
                break;
            }
        }
        if !found {
            for value in s.folder_images.values_mut().flatten() {
                if value.id == variant {
                    if value.styles.contains(&name.to_string()) {
                        return Ok(true);
                    }
                    value.styles.push(name.into());
                    found = true;
                    break;
                }
            }
        }
        if !found {
            return Err(Error::VariantNotFound);
        }
        s.ops.push(format!("apply_style:{variant}:{name}"));
        Ok(true)
    }
    fn get_captures(&self, id: &str) -> Result<String> {
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        s.ops.push("get_captures".into());
        if s.readback_garbage {
            return Ok(format!("{}/Capture", std::env::current_dir()?.display()));
        }
        if s.private_prefix {
            return Ok(fs::canonicalize(&s.captures)?
                .to_string_lossy()
                .into_owned());
        }
        Ok(s.captures.clone())
    }
    fn set_hook(&self, name: &str, path: &str) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("set_hook:{name}"));
        s.hooks.insert(name.into(), path.into());
        Ok(())
    }
    fn get_hook(&self, name: &str) -> Result<Option<String>> {
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("get_hook:{name}"));
        if s.hooks_readback_null {
            return Ok(None);
        };
        let value = s.hooks.get(name).cloned();
        if s.private_prefix {
            return value
                .map(|p| {
                    fs::canonicalize(p)
                        .map(|p| p.to_string_lossy().into_owned())
                        .map_err(Error::from)
                })
                .transpose();
        }
        Ok(value)
    }
    fn recipe_exists(&self, id: &str, name: &str) -> Result<bool> {
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("recipe_exists:{name}"));
        Ok(s.recipes.contains_key(name))
    }
    fn find_image(&self, id: &str, raw: &str) -> Result<Option<String>> {
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("find_image:{raw}"));
        self.list_pending(&mut s, Some(raw));
        if s.image_collections.get(raw) != Some(&s.current_collection) {
            return Ok(None);
        }
        Ok(s.images
            .get(raw)
            .and_then(|v| v.first())
            .map(|v| v.id.clone()))
    }
    fn find_image_in_folder(&self, id: &str, folder: &str, raw: &str) -> Result<Option<String>> {
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("find_image_in_folder:{raw}"));
        self.list_pending(&mut s, Some(raw));
        let images = if Path::new(raw)
            .parent()
            .is_some_and(|parent| parent == Path::new(folder))
        {
            s.folder_images.get(raw).or_else(|| s.images.get(raw))
        } else {
            None
        };
        Ok(images.and_then(|v| v.first()).map(|v| v.id.clone()))
    }
    fn get_color_tag(&self, id: &str, variant: &str) -> Result<i32> {
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("get_color_tag:{variant}"));
        s.images
            .values()
            .chain(s.folder_images.values())
            .flatten()
            .find(|v| v.id == variant)
            .map(|v| v.tag)
            .ok_or(Error::VariantNotFound)
    }
    fn set_color_tag(&self, id: &str, variant: &str, tag: i32) -> Result<()> {
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("set_color_tag:{variant}:{tag}"));
        let drop = s.drop_tag_writes;
        let timeout = s.timeout_next_set_tag;
        s.timeout_next_set_tag = false;
        if !drop {
            for v in s.images.values_mut().flatten() {
                if v.id == variant {
                    v.tag = tag;
                }
            }
            for v in s.folder_images.values_mut().flatten() {
                if v.id == variant {
                    v.tag = tag;
                }
            }
        }
        if timeout { Err(Error::Timeout) } else { Ok(()) }
    }
    fn process(
        &self,
        id: &str,
        variant: &str,
        recipe: &str,
    ) -> Result<psm_integrations::editing::capture_one::scripting::ProcessStarted> {
        self.state
            .lock()
            .unwrap()
            .ops
            .push(format!("process_started:{variant}:{recipe}"));
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("process:{variant}:{recipe}"));
        if s.fail_process {
            return Err(Error::Provider("process failed".into()));
        }
        if !s.recipes.contains_key(recipe) {
            return Err(Error::Provider(format!("recipe {recipe} not found")));
        }
        let id = format!("job-{}", s.next_job);
        s.next_job += 1;
        s.issued.push((id.clone(), variant.into(), recipe.into()));
        let geometry = s
            .images
            .values()
            .chain(s.folder_images.values())
            .flat_map(|v| v.iter())
            .find(|v| v.id == variant)
            .map(|v| v.geometry.clone())
            .unwrap_or_default();
        Ok(
            psm_integrations::editing::capture_one::scripting::ProcessStarted {
                job_id: id,
                geometry,
            },
        )
    }
    fn capture(&self) -> Result<()> {
        self.state.lock().unwrap().ops.push("capture".into());
        Ok(())
    }
    /// Like `quit.js`: records op `quit` either way; a running editor stops after
    /// `quit_after_polls` `editor_info` calls (never with `None`).
    fn quit(&self) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        s.ops.push("quit".into());
        if s.running {
            s.launched = false;
            match s.quit_after_polls {
                Some(0) => s.running = false,
                Some(n) => s.quitting = Some(n),
                // Never quits (a dialog is open): a countdown that never reaches zero.
                None => s.quitting = Some(u32::MAX),
            }
        }
        Ok(())
    }
}
