use psm_core::{Error, Result};
use psm_integrations::editing::capture_one::recipes::RecipeSpec;
use psm_integrations::editing::capture_one::scripting::{
    CaptureOneScripting, CollectionInfo, DocumentInfo, EditorInfo, RecipeEnsure, RecipeProps,
    StyleEnsure,
};
use std::{collections::HashMap, fs, path::Path, sync::Mutex};

#[derive(Clone)]
pub struct FakeVariant {
    pub id: String,
    pub tag: i32,
    pub geometry: psm_core::domain::composition::EditorGeometry,
}
pub struct FakeState {
    pub document_id: String,
    pub kind: String,
    pub captures: String,
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
}
pub struct FakeCaptureOneScripting {
    pub state: Mutex<FakeState>,
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
                fail_make_document: None,
                fail_open_document: None,
                fail_front: false,
                fail_make_recipe: None,
                styles: vec!["PSM House".into()],
                next_capture_styles: Vec::new(),
                kind: "session".into(),
                captures: capture_root.clone(),
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
            }),
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
            })
            .collect();
        if in_folder {
            s.folder_images.insert(raw.into(), value);
        } else {
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
impl CaptureOneScripting for FakeCaptureOneScripting {
    fn editor_info(&self) -> Result<EditorInfo> {
        let mut s = self.state.lock().unwrap();
        s.ops.push("editor_info".into());
        if s.automation_denied {
            return Err(Error::Provider(
                "execution error: Not authorized to send Apple events to Capture One. (-1743)"
                    .into(),
            ));
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
        if delay > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
        let mut s = self.state.lock().unwrap();
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
        self.state.lock().unwrap().ops.push("save".into());
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
        Ok(s.images
            .get(raw)
            .and_then(|v| v.first())
            .map(|v| v.id.clone()))
    }
    fn find_image_in_folder(&self, id: &str, _folder: &str, raw: &str) -> Result<Option<String>> {
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("find_image_in_folder:{raw}"));
        Ok(s.folder_images
            .get(raw)
            .and_then(|v| v.first())
            .map(|v| v.id.clone()))
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
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("process:{variant}:{recipe}"));
        if s.fail_process {
            return Err(Error::Provider("process failed".into()));
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
    fn import(&self, id: &str, paths: &[String]) -> Result<()> {
        self.check_id(id)?;
        self.state
            .lock()
            .unwrap()
            .ops
            .push(format!("import:{}", paths.len()));
        Ok(())
    }
}
