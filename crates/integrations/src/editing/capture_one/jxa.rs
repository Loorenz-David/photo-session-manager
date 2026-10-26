use super::scripting::{CaptureOneScripting, DocumentInfo};
use psm_core::{Error, Result};
use psm_infra::{osascript::OsaRunner, paths::AppPaths};
use serde::de::DeserializeOwned;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Duration,
};

const SCRIPTS: [(&str, &str); 14] = [
    ("document_info", include_str!("jxa/document_info.js")),
    ("save", include_str!("jxa/save.js")),
    ("set_captures", include_str!("jxa/set_captures.js")),
    ("get_captures", include_str!("jxa/get_captures.js")),
    ("set_hook", include_str!("jxa/set_hook.js")),
    ("get_hook", include_str!("jxa/get_hook.js")),
    ("recipe_exists", include_str!("jxa/recipe_exists.js")),
    ("set_recipe_props", include_str!("jxa/set_recipe_props.js")),
    ("find_image", include_str!("jxa/find_image.js")),
    (
        "find_image_in_folder",
        include_str!("jxa/find_image_in_folder.js"),
    ),
    ("get_color_tag", include_str!("jxa/get_color_tag.js")),
    ("set_color_tag", include_str!("jxa/set_color_tag.js")),
    ("process", include_str!("jxa/process.js")),
    ("capture", include_str!("jxa/capture.js")),
];
const IMPORT: &str = include_str!("jxa/import.js");

pub struct JxaScripting {
    runner: OsaRunner,
    dir: PathBuf,
    variant_raws: Mutex<HashMap<String, String>>,
}
impl JxaScripting {
    pub fn new(paths: &AppPaths) -> Result<Self> {
        let dir = paths.hooks().join("jxa");
        fs::create_dir_all(&dir)?;
        for (name, code) in SCRIPTS {
            fs::write(
                dir.join(format!("{name}.js")),
                format!("{}\n{code}", include_str!("jxa/lib.js")),
            )?;
        }
        fs::write(
            dir.join("import.js"),
            format!("{}\n{IMPORT}", include_str!("jxa/lib.js")),
        )?;
        Ok(Self {
            runner: OsaRunner::new(&paths.root),
            dir,
            variant_raws: Mutex::new(HashMap::new()),
        })
    }
    fn call<T: DeserializeOwned>(&self, op: &str, args: &[&str]) -> Result<T> {
        let output = self.runner.run(
            &self.dir.join(format!("{op}.js")),
            args,
            Duration::from_secs(30),
        )?;
        serde_json::from_str(&output)
            .map_err(|e| Error::InvalidData(format!("{op}: {e}; {output}")))
    }
    fn remember_variant(&self, raw: &str, variant: &Option<String>) -> Result<()> {
        if let Some(variant) = variant {
            self.variant_raws
                .lock()
                .map_err(|e| Error::InvalidData(e.to_string()))?
                .insert(variant.clone(), raw.into());
        }
        Ok(())
    }
    fn variant_location(&self, variant: &str) -> Result<(String, String)> {
        let raw = self
            .variant_raws
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .get(variant)
            .cloned()
            .ok_or(Error::VariantNotFound)?;
        let folder = Path::new(&raw)
            .parent()
            .ok_or(Error::VariantNotFound)?
            .to_string_lossy()
            .into_owned();
        Ok((raw, folder))
    }
}
pub fn parse_process_result(value: &str) -> Result<String> {
    if value.starts_with("ERROR") {
        Err(Error::Provider(value.into()))
    } else {
        Ok(value.into())
    }
}
impl CaptureOneScripting for JxaScripting {
    fn document_info(&self, id: &str) -> Result<DocumentInfo> {
        self.call("document_info", &[id])
    }
    fn save(&self, id: &str) -> Result<()> {
        self.call("save", &[id])
    }
    fn set_captures(&self, id: &str, path: &str) -> Result<()> {
        self.call("set_captures", &[id, path])
    }
    fn get_captures(&self, id: &str) -> Result<String> {
        self.call("get_captures", &[id])
    }
    fn set_hook(&self, name: &str, path: &str) -> Result<()> {
        self.call("set_hook", &[name, path])
    }
    fn get_hook(&self, name: &str) -> Result<Option<String>> {
        self.call("get_hook", &[name])
    }
    fn recipe_exists(&self, id: &str, name: &str) -> Result<bool> {
        self.call("recipe_exists", &[id, name])
    }
    fn set_recipe_props(
        &self,
        id: &str,
        name: &str,
        existing_files: &str,
        output_sub_folder: &str,
    ) -> Result<()> {
        self.call(
            "set_recipe_props",
            &[id, name, existing_files, output_sub_folder],
        )
    }
    fn find_image(&self, id: &str, raw: &str) -> Result<Option<String>> {
        let found = self.call("find_image", &[id, raw])?;
        self.remember_variant(raw, &found)?;
        Ok(found)
    }
    fn find_image_in_folder(&self, id: &str, folder: &str, raw: &str) -> Result<Option<String>> {
        let found = self.call("find_image_in_folder", &[id, folder, raw])?;
        self.remember_variant(raw, &found)?;
        Ok(found)
    }
    fn get_color_tag(&self, id: &str, variant: &str) -> Result<i32> {
        let (raw, folder) = self.variant_location(variant)?;
        self.call("get_color_tag", &[id, &raw, &folder, variant])
    }
    fn set_color_tag(&self, id: &str, variant: &str, tag: i32) -> Result<()> {
        let (raw, folder) = self.variant_location(variant)?;
        self.call(
            "set_color_tag",
            &[id, &raw, &folder, variant, &tag.to_string()],
        )
    }
    fn process(&self, id: &str, variant: &str, recipe: &str) -> Result<String> {
        let (raw, folder) = self.variant_location(variant)?;
        parse_process_result(
            &self.call::<String>("process", &[id, &raw, &folder, variant, recipe])?,
        )
    }
    fn capture(&self) -> Result<()> {
        self.call("capture", &[])
    }
    fn import(&self, id: &str, paths: &[String]) -> Result<()> {
        self.call(
            "import",
            &[
                id,
                &serde_json::to_string(paths).map_err(|e| Error::InvalidData(e.to_string()))?,
            ],
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mc16_process_result() {
        assert_eq!(parse_process_result("job-1").unwrap(), "job-1");
        assert!(matches!(
            parse_process_result("ERROR failed"),
            Err(Error::Provider(_))
        ));
    }
}
