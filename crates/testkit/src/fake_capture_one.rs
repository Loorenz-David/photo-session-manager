use psm_core::{Error, Result};
use psm_integrations::editing::capture_one::scripting::{
    CaptureOneScripting, CollectionInfo, DocumentInfo,
};
use std::{collections::HashMap, fs, path::Path, sync::Mutex};

#[derive(Clone)]
pub struct FakeVariant {
    pub id: String,
    pub tag: i32,
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
    pub recipes: HashMap<String, (String, String)>,
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
        recipes.insert("PSM Preview".into(), ("add suffix".into(), String::new()));
        recipes.insert("PSM Web".into(), ("add suffix".into(), String::new()));
        Self {
            state: Mutex::new(FakeState {
                document_id,
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
            (s.document_id.clone(), s.call_delay_ms)
        };
        if delay > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
        if id != expected {
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
            })
            .collect();
        if in_folder {
            s.folder_images.insert(raw.into(), value);
        } else {
            s.images.insert(raw.into(), value);
        }
    }
}
impl CaptureOneScripting for FakeCaptureOneScripting {
    fn document_info(&self, id: &str) -> Result<DocumentInfo> {
        let delay = self.state.lock().unwrap().delay_document_info_ms;
        if delay > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
        let mut s = self.state.lock().unwrap();
        s.ops.push("document_info".into());
        if id != s.document_id {
            return Err(Error::DocumentNotOpen);
        };
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
    fn set_recipe_props(
        &self,
        id: &str,
        name: &str,
        existing_files: &str,
        output_sub_folder: &str,
    ) -> Result<()> {
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("set_recipe_props:{name}"));
        s.recipes.insert(
            name.into(),
            (existing_files.into(), output_sub_folder.into()),
        );
        Ok(())
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
    fn process(&self, id: &str, variant: &str, recipe: &str) -> Result<String> {
        self.check_id(id)?;
        let mut s = self.state.lock().unwrap();
        s.ops.push(format!("process:{variant}:{recipe}"));
        if s.fail_process {
            return Err(Error::Provider("process failed".into()));
        }
        let id = format!("job-{}", s.next_job);
        s.next_job += 1;
        s.issued.push((id.clone(), variant.into(), recipe.into()));
        Ok(id)
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
