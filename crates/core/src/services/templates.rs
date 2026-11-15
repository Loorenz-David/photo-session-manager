use crate::{
    Error, Result,
    domain::{
        composition::Shape,
        template::{Presets, Template, TemplateSummary},
    },
    mechanisms::template::type_key,
    ports::{clock::Clock, images::Imaging, store::Store},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct TemplatesService<'a> {
    store: &'a dyn Store,
    clock: &'a dyn Clock,
    imaging: &'a dyn Imaging,
    folder: PathBuf,
}

impl<'a> TemplatesService<'a> {
    pub fn new(
        store: &'a dyn Store,
        clock: &'a dyn Clock,
        imaging: &'a dyn Imaging,
        folder: PathBuf,
    ) -> Self {
        Self {
            store,
            clock,
            imaging,
            folder,
        }
    }
    fn present(&self, mut template: Template) -> Template {
        for c in &mut template.containers {
            c.example_path = c.example_file.as_ref().map(|name| {
                self.folder
                    .join(template.id.to_string())
                    .join(name)
                    .to_string_lossy()
                    .into_owned()
            });
        }
        template
    }
    pub fn list(&self) -> Result<Vec<TemplateSummary>> {
        self.store.template_list()
    }
    pub fn get(&self, item_type: &str) -> Result<Option<Template>> {
        self.store
            .template_get(&type_key(item_type))
            .map(|v| v.map(|t| self.present(t)))
    }
    pub fn create(&self, item_type: &str) -> Result<Template> {
        let trimmed = item_type.trim();
        if trimmed.is_empty() {
            return Err(Error::InvalidValue("item type is empty".into()));
        }
        self.store
            .template_create(trimmed, &type_key(trimmed), self.clock.now_ms())
            .map(|t| self.present(t))
    }
    pub fn delete(&self, template_id: i64) -> Result<()> {
        self.store.template_delete(template_id)?;
        let folder = self.folder.join(template_id.to_string());
        if folder.exists() {
            fs::remove_dir_all(folder)?;
        }
        Ok(())
    }
    fn copy_example(&self, template_id: i64, source: &Path) -> Result<String> {
        let file = format!("{}.jpg", uuid::Uuid::new_v4());
        let dest = self.folder.join(template_id.to_string()).join(&file);
        match self.imaging.thumbnail(source, &dest, 1600) {
            Ok(true) => Ok(file),
            Ok(false) => Err(Error::TemplateImageUnreadable),
            Err(error) => {
                let _ = fs::remove_file(dest);
                Err(error)
            }
        }
    }
    pub fn container_add(&self, template_id: i64, source: Option<&Path>) -> Result<Template> {
        self.store.template_get_by_id(template_id)?;
        let file = source
            .map(|s| self.copy_example(template_id, s))
            .transpose()?;
        match self
            .store
            .template_container_add(template_id, file.as_deref(), self.clock.now_ms())
        {
            Ok(t) => Ok(self.present(t)),
            Err(error) => {
                if let Some(file) = file {
                    let _ = fs::remove_file(self.folder.join(template_id.to_string()).join(file));
                }
                Err(error)
            }
        }
    }
    pub fn container_example_set(&self, container_id: i64, source: &Path) -> Result<Template> {
        let (template_id, _) = self.locate(container_id)?;
        let file = self.copy_example(template_id, source)?;
        match self
            .store
            .template_container_example_set(container_id, &file, self.clock.now_ms())
        {
            Ok((t, old)) => {
                if let Some(old) = old {
                    fs::remove_file(self.folder.join(template_id.to_string()).join(old))?;
                }
                Ok(self.present(t))
            }
            Err(error) => {
                let _ = fs::remove_file(self.folder.join(template_id.to_string()).join(file));
                Err(error)
            }
        }
    }
    pub fn container_remove(&self, container_id: i64) -> Result<Template> {
        let (template_id, _) = self.locate(container_id)?;
        let (t, old) = self
            .store
            .template_container_remove(container_id, self.clock.now_ms())?;
        if let Some(old) = old {
            fs::remove_file(self.folder.join(template_id.to_string()).join(old))?;
        }
        Ok(self.present(t))
    }
    fn locate(&self, container_id: i64) -> Result<(i64, Template)> {
        for summary in self.list()? {
            let t = self.store.template_get_by_id(summary.id)?;
            if t.containers.iter().any(|c| c.id == container_id) {
                return Ok((summary.id, t));
            }
        }
        Err(Error::NotFound("container".into()))
    }
    pub fn containers_reorder(&self, template_id: i64, ids: &[i64]) -> Result<Template> {
        self.store
            .template_order_set(template_id, ids, false, self.clock.now_ms())
            .map(|t| self.present(t))
    }
    pub fn shooting_order_set(&self, template_id: i64, ids: &[i64]) -> Result<Template> {
        self.store
            .template_order_set(template_id, ids, true, self.clock.now_ms())
            .map(|t| self.present(t))
    }
    pub fn container_presets_set(&self, container_id: i64, presets: Presets) -> Result<Template> {
        if presets.zoom > 80 || presets.shape == Some(Shape::Free) {
            return Err(Error::InvalidValue("preset is out of range".into()));
        }
        self.store
            .template_container_presets_set(container_id, &presets, self.clock.now_ms())
            .map(|t| self.present(t))
    }
}
