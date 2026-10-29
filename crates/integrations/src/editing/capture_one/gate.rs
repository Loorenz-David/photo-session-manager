use super::scripting::{CaptureOneScripting, DocumentInfo};
use psm_core::domain::editing::ShapeOutcome;
use psm_core::{Error, Result};
use std::sync::{Arc, Condvar, Mutex};

#[derive(Clone, Copy)]
pub enum Lane {
    User,
    Background,
}

#[derive(Default)]
struct GateState {
    active: bool,
    user_waiting: usize,
}

pub struct GatedScripting {
    inner: Arc<dyn CaptureOneScripting>,
    lane: Lane,
    gate: Arc<(Mutex<GateState>, Condvar)>,
}
impl GatedScripting {
    pub fn pair(
        inner: Arc<dyn CaptureOneScripting>,
    ) -> (Arc<dyn CaptureOneScripting>, Arc<dyn CaptureOneScripting>) {
        let gate = Arc::new((Mutex::new(GateState::default()), Condvar::new()));
        (
            Arc::new(Self {
                inner: inner.clone(),
                lane: Lane::User,
                gate: gate.clone(),
            }),
            Arc::new(Self {
                inner,
                lane: Lane::Background,
                gate,
            }),
        )
    }
    fn call<T>(&self, f: impl FnOnce(&dyn CaptureOneScripting) -> Result<T>) -> Result<T> {
        let (lock, cv) = &*self.gate;
        let mut state = lock.lock().map_err(|e| Error::InvalidData(e.to_string()))?;
        if matches!(self.lane, Lane::User) {
            state.user_waiting += 1;
        }
        while state.active || (matches!(self.lane, Lane::Background) && state.user_waiting > 0) {
            state = cv
                .wait(state)
                .map_err(|e| Error::InvalidData(e.to_string()))?;
        }
        if matches!(self.lane, Lane::User) {
            state.user_waiting -= 1;
        }
        state.active = true;
        drop(state);
        struct Release<'a>(&'a (Mutex<GateState>, Condvar));
        impl Drop for Release<'_> {
            fn drop(&mut self) {
                if let Ok(mut state) = self.0.0.lock() {
                    state.active = false;
                    self.0.1.notify_all();
                }
            }
        }
        let _release = Release(&self.gate);
        f(self.inner.as_ref())
    }
}
impl CaptureOneScripting for GatedScripting {
    fn document_info(&self, id: &str) -> Result<DocumentInfo> {
        self.call(|i| i.document_info(id))
    }
    fn save(&self, id: &str) -> Result<()> {
        self.call(|i| i.save(id))
    }
    fn set_captures(&self, id: &str, path: &str) -> Result<()> {
        self.call(|i| i.set_captures(id, path))
    }
    fn get_captures(&self, id: &str) -> Result<String> {
        self.call(|i| i.get_captures(id))
    }
    fn set_hook(&self, name: &str, path: &str) -> Result<()> {
        self.call(|i| i.set_hook(name, path))
    }
    fn get_hook(&self, name: &str) -> Result<Option<String>> {
        self.call(|i| i.get_hook(name))
    }
    fn recipe_exists(&self, id: &str, name: &str) -> Result<bool> {
        self.call(|i| i.recipe_exists(id, name))
    }
    fn set_recipe_props(
        &self,
        id: &str,
        name: &str,
        existing_files: &str,
        output_sub_folder: &str,
    ) -> Result<()> {
        self.call(|i| i.set_recipe_props(id, name, existing_files, output_sub_folder))
    }
    fn find_image(&self, id: &str, raw: &str) -> Result<Option<String>> {
        self.call(|i| i.find_image(id, raw))
    }
    fn find_image_in_folder(&self, id: &str, folder: &str, raw: &str) -> Result<Option<String>> {
        self.call(|i| i.find_image_in_folder(id, folder, raw))
    }
    fn get_color_tag(&self, id: &str, variant: &str) -> Result<i32> {
        self.call(|i| i.get_color_tag(id, variant))
    }
    fn set_color_tag(&self, id: &str, variant: &str, tag: i32) -> Result<()> {
        self.call(|i| i.set_color_tag(id, variant, tag))
    }
    fn crop_aspect_ratios(&self, id: &str) -> Result<Vec<String>> {
        self.call(|i| i.crop_aspect_ratios(id))
    }
    fn apply_crop_if_untouched(
        &self,
        id: &str,
        variant: &str,
        ratio: &str,
    ) -> Result<ShapeOutcome> {
        self.call(|i| i.apply_crop_if_untouched(id, variant, ratio))
    }
    fn process(&self, id: &str, variant: &str, recipe: &str) -> Result<String> {
        self.call(|i| i.process(id, variant, recipe))
    }
    fn capture(&self) -> Result<()> {
        self.call(|i| i.capture())
    }
    fn import(&self, id: &str, paths: &[String]) -> Result<()> {
        self.call(|i| i.import(id, paths))
    }
}
