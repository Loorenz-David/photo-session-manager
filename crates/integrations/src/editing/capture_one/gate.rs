use super::{
    launch_gate::LaunchGate,
    scripting::{
        CaptureOneScripting, DocumentInfo, EditorInfo, RecipeEnsure, RecipeProps, StyleEnsure,
    },
};
use crate::camera::direct::start_order::{StartOrder, StartReason};
use psm_core::{Error, Result, ports::clock::Clock};
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

/// The user and background lanes over one scripting implementation. Both lanes also enforce the
/// editor-launch gate (10E MC-34 R-I32): the six scripts that do not check `running()` first —
/// `launch`, `front_document`, `make_document`, `open_document`, `ensure_recipe`,
/// `ensure_capture_style` — answer `EditorLaunchDeferred` without running while "Take the camera
/// back" holds the gate on another thread. `launch` also waits for the start order (R-I31).
pub struct GatedScripting {
    inner: Arc<dyn CaptureOneScripting>,
    lane: Lane,
    gate: Arc<(Mutex<GateState>, Condvar)>,
    launch_gate: Arc<LaunchGate>,
    start_order: Arc<StartOrder>,
    clock: Arc<dyn Clock>,
}
impl GatedScripting {
    pub fn pair(
        inner: Arc<dyn CaptureOneScripting>,
        launch_gate: Arc<LaunchGate>,
        start_order: Arc<StartOrder>,
        clock: Arc<dyn Clock>,
    ) -> (Arc<dyn CaptureOneScripting>, Arc<dyn CaptureOneScripting>) {
        let gate = Arc::new((Mutex::new(GateState::default()), Condvar::new()));
        (
            Arc::new(Self {
                inner: inner.clone(),
                lane: Lane::User,
                gate: gate.clone(),
                launch_gate: launch_gate.clone(),
                start_order: start_order.clone(),
                clock: clock.clone(),
            }),
            Arc::new(Self {
                inner,
                lane: Lane::Background,
                gate,
                launch_gate,
                start_order,
                clock,
            }),
        )
    }
    /// A script that would launch Capture One: refused while the gate is held elsewhere.
    fn launching<T>(&self, f: impl FnOnce(&dyn CaptureOneScripting) -> Result<T>) -> Result<T> {
        if self.launch_gate.refuses_here() {
            return Err(Error::EditorLaunchDeferred);
        }
        self.call(f)
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
    /// Only the ensure asks this on the gated lanes: Capture One found running satisfies the start
    /// order for the rest of the process (R-I31 "already running at app start", P48 L4b).
    fn editor_info(&self) -> Result<EditorInfo> {
        let info = self.call(|i| i.editor_info())?;
        if info.running {
            self.start_order.satisfy(StartReason::AlreadyRunning);
        }
        Ok(info)
    }
    /// Gate check; then, off the permitted thread, the start-order wait outside the lane mutex (so
    /// no script on either lane stalls behind it, L5); then the gate again; then the launch.
    fn launch(&self) -> Result<()> {
        if self.launch_gate.refuses_here() {
            return Err(Error::EditorLaunchDeferred);
        }
        if !self.launch_gate.permitted_here() {
            self.start_order.wait(self.clock.as_ref());
        }
        self.launching(|i| i.launch())
    }
    fn make_document(&self, parent: &str, name: &str) -> Result<String> {
        self.launching(|i| i.make_document(parent, name))
    }
    fn open_document(&self, file: &str) -> Result<()> {
        self.launching(|i| i.open_document(file))
    }
    fn front_document(&self, id: &str) -> Result<String> {
        self.launching(|i| i.front_document(id))
    }
    fn ensure_recipe(&self, id: &str, name: &str, want: &RecipeProps) -> Result<RecipeEnsure> {
        self.launching(|i| i.ensure_recipe(id, name, want))
    }
    fn ensure_capture_style(&self, id: &str, name: &str) -> Result<StyleEnsure> {
        self.launching(|i| i.ensure_capture_style(id, name))
    }
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
    fn process(
        &self,
        id: &str,
        variant: &str,
        recipe: &str,
    ) -> Result<super::scripting::ProcessStarted> {
        self.call(|i| i.process(id, variant, recipe))
    }
    fn capture(&self) -> Result<()> {
        self.call(|i| i.capture())
    }
    fn import(&self, id: &str, paths: &[String]) -> Result<()> {
        self.call(|i| i.import(id, paths))
    }
    /// Not gated: `quit.js` checks `running()` first (L13).
    fn quit(&self) -> Result<()> {
        self.call(|i| i.quit())
    }
}
