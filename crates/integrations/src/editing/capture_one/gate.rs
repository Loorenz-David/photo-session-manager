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

/// The user and background lanes over one scripting implementation, and the one gate every
/// script to Capture One passes (10E MC-34 R-I32; P60, P63 #22). Off the thread the take-back
/// permits, while Capture One must stay closed — "Take the camera back" holds the gate (from its
/// first step to its end), or a relaunch waits for the camera helper's USB session (P54) and
/// Capture One is not running — no script runs: the caller gets `EditorLaunchDeferred` ("not
/// now", neither a failure nor an attempt). `is_running` is no script (the process list) and is
/// always answered; `launch` keeps its start-order wait (the one explicit launch step); while the
/// gate is merely held, the launch-capable and render scripts are refused too. The take-back
/// waits for the scripts already running to finish before it quits Capture One.
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
    /// Capture One must stay closed for this thread (P63 #22): the take-back holds the gate on
    /// another thread, or a relaunch waits for the camera helper and Capture One is not running
    /// (asked of the process list, never of Capture One).
    fn closed_here(&self) -> bool {
        if self.launch_gate.permitted_here() {
            return false;
        }
        self.launch_gate.quiet_here()
            || (self.start_order.needs_relaunch_wait() && !self.inner.is_running().unwrap_or(false))
    }
    /// A script that would launch Capture One or render: also refused while the gate is held
    /// elsewhere (R-I32).
    fn launching<T>(&self, f: impl FnOnce(&dyn CaptureOneScripting) -> Result<T>) -> Result<T> {
        if self.launch_gate.refuses_here() {
            return Err(Error::EditorLaunchDeferred);
        }
        self.call(|inner| {
            if self.launch_gate.refuses_here() {
                return Err(Error::EditorLaunchDeferred);
            }
            f(inner)
        })
    }
    /// Every script but `launch` and `is_running`: refused while Capture One must stay closed,
    /// checked before the lane is taken and again once it is (P60, P63 #22: an Apple event
    /// addressed to a quit Capture One relaunches it).
    fn call<T>(&self, f: impl FnOnce(&dyn CaptureOneScripting) -> Result<T>) -> Result<T> {
        if self.closed_here() {
            return Err(Error::EditorLaunchDeferred);
        }
        self.lane(|inner| {
            if self.closed_here() {
                return Err(Error::EditorLaunchDeferred);
            }
            f(inner)
        })
    }
    /// Runs `f` on this lane (one script at a time across both lanes, user first), counted as in
    /// flight for the take-back's drain.
    fn lane<T>(&self, f: impl FnOnce(&dyn CaptureOneScripting) -> Result<T>) -> Result<T> {
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
        let _in_flight = self.launch_gate.enter();
        f(self.inner.as_ref())
    }
}
impl CaptureOneScripting for GatedScripting {
    /// No script: always answered, never waits for a lane.
    fn is_running(&self) -> Result<bool> {
        self.inner.is_running()
    }
    /// Only while Capture One runs (asking would launch it).
    fn list_styles(&self) -> Result<super::scripting::StyleList> {
        if !self.inner.is_running()? {
            return Err(Error::EditorLaunchDeferred);
        }
        self.call(|i| i.list_styles())
    }
    /// Only the ensure asks this on the gated lanes: Capture One found running satisfies the start
    /// order for the rest of the process (R-I31 "already running at app start", P48 L4b). Refused
    /// while the take-back holds the gate (P63 #22).
    fn editor_info(&self) -> Result<EditorInfo> {
        if self.launch_gate.quiet_here() {
            return Err(Error::EditorLaunchDeferred);
        }
        let info = self.lane(|i| i.editor_info())?;
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
            if self.start_order.state() == crate::camera::direct::start_order::StartState::Waiting {
                self.start_order.wait(self.clock.as_ref());
            } else {
                self.start_order.wait_for_relaunch(self.clock.as_ref());
            }
        }
        if self.launch_gate.refuses_here() {
            return Err(Error::EditorLaunchDeferred);
        }
        if self.launch_gate.quiet_here() {
            return Err(Error::EditorLaunchDeferred);
        }
        self.lane(|i| {
            if self.launch_gate.refuses_here() {
                return Err(Error::EditorLaunchDeferred);
            }
            i.launch()
        })
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
        self.launching(|i| i.recipe_exists(id, name))
    }
    fn find_image(&self, id: &str, raw: &str) -> Result<Option<String>> {
        self.launching(|i| i.find_image(id, raw))
    }
    fn find_image_in_folder(&self, id: &str, folder: &str, raw: &str) -> Result<Option<String>> {
        self.launching(|i| i.find_image_in_folder(id, folder, raw))
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
        self.launching(|i| i.process(id, variant, recipe))
    }
    fn capture(&self) -> Result<()> {
        self.call(|i| i.capture())
    }
    fn set_current_collection(&self, id: &str, folder: &str) -> Result<()> {
        self.call(|i| i.set_current_collection(id, folder))
    }
    fn apply_style(&self, id: &str, variant: &str, name: &str) -> Result<bool> {
        self.call(|i| i.apply_style(id, variant, name))
    }
    /// Refused like a launch-capable script while the take-back holds the gate (P65).
    fn sync_folder(&self, id: &str, folder: &str) -> Result<()> {
        self.launching(|i| i.sync_folder(id, folder))
    }
    /// `quit.js` checks `running()` first (L13); refused while the take-back holds the gate (it
    /// quits through its own ungated scripting).
    fn quit(&self) -> Result<()> {
        if self.launch_gate.quiet_here() {
            return Err(Error::EditorLaunchDeferred);
        }
        self.lane(|i| i.quit())
    }
}
