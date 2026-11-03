use super::scripting::{
    CaptureOneScripting, DocumentInfo, EditorInfo, RecipeEnsure, RecipeProps, StyleEnsure,
};
use psm_core::domain::editing::ShapeOutcome;
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

const SCRIPTS: [(&str, &str); 21] = [
    ("editor_info", include_str!("jxa/editor_info.js")),
    ("make_document", include_str!("jxa/make_document.js")),
    ("open_document", include_str!("jxa/open_document.js")),
    ("front_document", include_str!("jxa/front_document.js")),
    ("ensure_recipe", include_str!("jxa/ensure_recipe.js")),
    (
        "ensure_capture_style",
        include_str!("jxa/ensure_capture_style.js"),
    ),
    ("document_info", include_str!("jxa/document_info.js")),
    ("save", include_str!("jxa/save.js")),
    ("set_captures", include_str!("jxa/set_captures.js")),
    ("get_captures", include_str!("jxa/get_captures.js")),
    ("set_hook", include_str!("jxa/set_hook.js")),
    ("get_hook", include_str!("jxa/get_hook.js")),
    ("recipe_exists", include_str!("jxa/recipe_exists.js")),
    ("find_image", include_str!("jxa/find_image.js")),
    (
        "find_image_in_folder",
        include_str!("jxa/find_image_in_folder.js"),
    ),
    ("get_color_tag", include_str!("jxa/get_color_tag.js")),
    ("set_color_tag", include_str!("jxa/set_color_tag.js")),
    ("process", include_str!("jxa/process.js")),
    ("capture", include_str!("jxa/capture.js")),
    (
        "crop_aspect_ratios",
        include_str!("jxa/crop_aspect_ratios.js"),
    ),
    ("apply_crop", include_str!("jxa/apply_crop.js")),
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
    fn editor_info(&self) -> Result<EditorInfo> {
        self.call("editor_info", &[])
    }
    fn launch(&self) -> Result<()> {
        let output = std::process::Command::new("open")
            .args(["-g", "-a", "Capture One"])
            .output()?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Error::Provider(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ))
        }
    }
    fn make_document(&self, parent: &str, name: &str) -> Result<String> {
        self.call("make_document", &[parent, name])
    }
    fn open_document(&self, file: &str) -> Result<()> {
        let scratch = self.dir.join(format!(
            "open-window-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::create_dir_all(&scratch)?;
        let scratch = scratch.to_string_lossy().into_owned();
        let result = self.call("open_document", &[file, &scratch]);
        let _ = fs::remove_dir_all(&scratch);
        result
    }
    fn front_document(&self, id: &str) -> Result<String> {
        self.call("front_document", &[id])
    }
    fn ensure_recipe(&self, id: &str, name: &str, want: &RecipeProps) -> Result<RecipeEnsure> {
        let json = serde_json::to_string(want).map_err(|e| Error::InvalidData(e.to_string()))?;
        self.call("ensure_recipe", &[id, name, &json])
    }
    fn ensure_capture_style(&self, id: &str, name: &str) -> Result<StyleEnsure> {
        self.call("ensure_capture_style", &[id, name])
    }
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
    fn crop_aspect_ratios(&self, id: &str) -> Result<Vec<String>> {
        self.call("crop_aspect_ratios", &[id])
    }
    fn apply_crop_if_untouched(
        &self,
        id: &str,
        variant: &str,
        ratio: &str,
    ) -> Result<ShapeOutcome> {
        let (raw, folder) = self.variant_location(variant)?;
        let answer: serde_json::Value =
            self.call("apply_crop", &[id, &raw, &folder, variant, ratio])?;
        match answer["outcome"].as_str() {
            Some("applied") if answer["ratio"].as_str() == Some(ratio) => Ok(ShapeOutcome::Applied),
            Some("applied") => Err(Error::Provider("crop read-back mismatch".into())),
            Some("skipped") => Ok(ShapeOutcome::Skipped),
            Some("unknown_ratio") => Ok(ShapeOutcome::UnknownRatio),
            _ => Err(Error::InvalidData("apply_crop outcome".into())),
        }
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
