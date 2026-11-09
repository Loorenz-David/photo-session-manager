//! "Take the camera back" (10E MC-34 §Take it back, R-I32; M39, M40): on the photographer's
//! click or the default-on plug-in trigger, save the Capture One document, ask Capture One to quit (never kill it), retry the
//! USB session, relaunch Capture One and reopen the document — each step bounded, any failure
//! stops the sequence and releases the editor-launch gate.
use super::{
    CAMERA_SERVICE_STUCK, CAPTURE_ONE_WINDOW_OPEN, DirectCameraAdapter, TAKE_BACK_DRAIN_WAIT_MS,
    TAKE_BACK_EXPOSURE_GRACE_MS, TAKE_BACK_FREE_WAIT_MS, TAKE_BACK_QUIT_WAIT_MS,
    TAKE_BACK_READY_WAIT_MS, TAKE_BACK_STUCK_SERVICE_MS,
    ownership::{HolderReport, known_running, name_programs},
};
use crate::{
    camera::{runtime::CameraRuntime, sidecar::protocol::Command},
    editing::capture_one::{launch_gate::LaunchGate, scripting::CaptureOneScripting},
};
use psm_core::{
    Error, Result,
    domain::{camera::ExposureStatus, editing::ProvisionInputs},
    ports::{camera::CameraProvider, clock::Clock, editing::EditingProvider, store::Store},
    services::{editing_startup::EditingStartup, health::HealthService},
};
use serde::Serialize;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

/// Step 3 asks whether Capture One still runs this often (clock time).
const QUIT_POLL_MS: u64 = 500;
/// Step 4 re-checks that nobody started Capture One this often (clock time).
const STARTED_POLL_MS: u64 = 500;
/// A `usb.claimed_by` during the free wait is answered by another `usb-retry` after this long.
const RETRY_AFTER_CLAIM_MS: u64 = 1_000;
/// Every wait polls the clock and sleeps this long in real time between polls.
const REAL_POLL: Duration = Duration::from_millis(10);

/// The `running` jobs that hold Capture One renders or a publish (L25: `queued` rows do not).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JobsRunning {
    pub render_master: i64,
    pub render_output: i64,
    pub publish_step: i64,
}

/// What the preconditions read (R-I32: "enabled only when …").
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TakeBackInputs {
    /// FTP uploads the receiver has open.
    pub uploads_in_progress: usize,
    /// A Save-to-PC download is expected.
    pub expected_transfer: bool,
    /// Age of the youngest `expecting` exposure, if any.
    pub youngest_expecting_exposure_age_ms: Option<i64>,
    pub jobs_running: JobsRunning,
    pub renders_in_flight: usize,
    pub holder_is_capture_one: bool,
}

/// Why the take-back cannot run now (master plan §6.6, in check order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TakeBackBlocked {
    pub reason: &'static str,
}

/// The second `run` while one is in progress.
pub const ALREADY_RUNNING: &str = "already running";

/// R-I32's enabling conditions, checked in the §6.6 order.
pub fn preconditions(inputs: &TakeBackInputs) -> std::result::Result<(), TakeBackBlocked> {
    let reason = if inputs.uploads_in_progress > 0 || inputs.expected_transfer {
        "a file is still arriving"
    } else if inputs
        .youngest_expecting_exposure_age_ms
        .is_some_and(|age| age < TAKE_BACK_EXPOSURE_GRACE_MS)
    {
        "a photo is still expected"
    } else if inputs.jobs_running.render_master + inputs.jobs_running.render_output > 0 {
        "an export is running"
    } else if inputs.jobs_running.publish_step > 0 {
        "a publish is running"
    } else if inputs.renders_in_flight > 0 {
        "a preview is rendering"
    } else if !inputs.holder_is_capture_one {
        "Capture One is not holding the camera"
    } else {
        return Ok(());
    };
    Err(TakeBackBlocked { reason })
}

impl From<TakeBackBlocked> for Error {
    fn from(value: TakeBackBlocked) -> Self {
        Error::TakeBackBlocked {
            reason: value.reason.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TakeBackReport {
    /// The last step that finished (5 = Capture One relaunched and the document reopened).
    pub completed_steps: u8,
    /// Our USB session holds the camera at the end (step 4 got `connection.ready` on USB).
    pub camera_held: bool,
}

fn failed(step: u8, cause: impl Into<String>) -> Error {
    Error::TakeBackFailed {
        step,
        cause: cause.into(),
    }
}

/// Built from the camera runtime's current binding (`CameraRuntime::take_back`).
pub struct TakeBackService {
    pub(crate) runtime: Arc<CameraRuntime>,
    pub(crate) adapter: Arc<DirectCameraAdapter>,
    pub(crate) store: Arc<dyn Store>,
    pub(crate) health: Arc<HealthService>,
    pub(crate) editing: Arc<dyn EditingProvider>,
    pub(crate) camera: Arc<dyn CameraProvider>,
    /// The ungated scripting (`save`, `quit`, `editor_info`).
    pub(crate) scripting: Arc<dyn CaptureOneScripting>,
    pub(crate) gate: Arc<LaunchGate>,
    pub(crate) provision: ProvisionInputs,
    pub(crate) document_id: String,
    pub(crate) clock: Arc<dyn Clock>,
    /// Process-wide: a second `run` while one is in progress is refused.
    pub(crate) running: Arc<AtomicBool>,
}

impl TakeBackService {
    /// The preconditions' inputs, read now.
    pub fn inputs(&self) -> Result<TakeBackInputs> {
        let now_ms = self.clock.now_ms();
        let youngest = self
            .store
            .exposures(Some(ExposureStatus::Expecting))?
            .iter()
            .map(|e| now_ms - e.exposed_at)
            .min();
        let running = |kind: &str| self.store.count_jobs(kind, "running", None);
        Ok(TakeBackInputs {
            uploads_in_progress: self.runtime.uploads_in_progress(),
            expected_transfer: self.adapter.transfer_expected(),
            youngest_expecting_exposure_age_ms: youngest,
            jobs_running: JobsRunning {
                render_master: running("render_master")?,
                render_output: running("render_output")?,
                publish_step: running("publish_step")?,
            },
            renders_in_flight: self.editing.renders_in_flight(),
            holder_is_capture_one: self.adapter.holder_is_capture_one(),
        })
    }
    /// Read-only form of the click guard, for the disabled button (R-I32).
    pub fn blocked_reason(&self) -> Result<Option<String>> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(Some(ALREADY_RUNNING.into()));
        }
        Ok(preconditions(&self.inputs()?)
            .err()
            .map(|blocked| blocked.reason.into()))
    }

    /// R-I32 steps 1–6 on the caller's thread.
    pub fn run(&self) -> Result<TakeBackReport> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(TakeBackBlocked {
                reason: ALREADY_RUNNING,
            }
            .into());
        }
        struct Done<'a>(&'a AtomicBool);
        impl Drop for Done<'_> {
            fn drop(&mut self) {
                self.0.store(false, Ordering::SeqCst);
            }
        }
        let _done = Done(&self.running);
        preconditions(&self.inputs()?)?;
        // 1. Hold the gate (released when `gate` drops, on every path: step 6). From here no script
        //    of the app's lanes reaches Capture One; the ones already running finish first, or
        //    their next Apple event would relaunch Capture One after the quit (P60 #22).
        let gate = self.gate.hold_quiet();
        if !self.gate.wait_idle(
            self.clock.as_ref(),
            Duration::from_millis(TAKE_BACK_DRAIN_WAIT_MS),
        ) {
            tracing::warn!("take back: a Capture One script is still running; quitting anyway");
        }
        // 2. Save; on failure Capture One is left untouched.
        self.scripting
            .save(&self.document_id)
            .map_err(|e| failed(2, e.to_string()))?;
        // 3. Ask Capture One to quit and wait until it is not running; never kill it (M39). No
        //    answer or still running: a window is open, and the header says so (P60 #23).
        let quit = self
            .scripting
            .quit()
            .map_err(|e| failed(3, e.to_string()))
            .and_then(|_| self.wait_quit());
        if let Err(error) = quit {
            self.adapter
                .set_auto_waiting(Some(CAPTURE_ONE_WINDOW_OPEN.into()));
            return Err(error);
        }
        self.adapter.set_auto_waiting(None);
        // 4. Retry the USB session (naming a stuck camera service, P60 #25).
        let camera_held = self.retry_usb()?;
        // 5. Whatever step 4 gave: launch Capture One (permitted: no start-order wait), ensure,
        //    front, capture folder — under the still-held gate.
        gate.permit(|| {
            EditingStartup {
                provider: self.editing.as_ref(),
                store: self.store.as_ref(),
                health: self.health.as_ref(),
                provision: &self.provision,
                camera: Some(self.camera.as_ref()),
                follow: None,
            }
            .run()
        })
        .map_err(|e| failed(5, e.to_string()))?;
        // 6. Release.
        drop(gate);
        Ok(TakeBackReport {
            completed_steps: 5,
            camera_held,
        })
    }

    fn elapsed(&self, since: Instant) -> Duration {
        self.clock.mono().saturating_duration_since(since)
    }

    fn running_now(&self) -> Option<bool> {
        self.scripting.editor_info().ok().map(|info| info.running)
    }

    fn wait_quit(&self) -> Result<()> {
        let start = self.clock.mono();
        loop {
            let polled = self.clock.mono();
            if self.running_now() == Some(false) {
                return Ok(());
            }
            loop {
                if self.elapsed(start) >= Duration::from_millis(TAKE_BACK_QUIT_WAIT_MS) {
                    return Err(failed(3, "Capture One did not quit (a dialog may be open)"));
                }
                if self.elapsed(polled) >= Duration::from_millis(QUIT_POLL_MS) {
                    break;
                }
                thread::sleep(REAL_POLL);
            }
        }
    }

    /// Step 4: `Ok(true)` once our USB session is ready, `Ok(false)` at the bound. Capture One
    /// found running (the photographer started it) at the start or on any poll stops the run.
    fn retry_usb(&self) -> Result<bool> {
        let started_meanwhile = || -> Result<()> {
            if self.running_now() == Some(true) {
                Err(failed(4, "Capture One was started meanwhile"))
            } else {
                Ok(())
            }
        };
        started_meanwhile()?;
        let send = || {
            if let Err(error) = self.runtime.send_command(Command::UsbRetry {
                cmd_id: String::new(),
            }) {
                tracing::warn!(%error, "take back: usb-retry not sent");
            }
        };
        let mut claims = self.adapter.claims_seen();
        let claims_at_quit = claims;
        send();
        let start = self.clock.mono();
        let mut last_send = start;
        let mut last_check = start;
        let mut stuck_named = false;
        loop {
            if self.adapter.usb_session() {
                return Ok(true);
            }
            let now = self.clock.mono();
            if now.saturating_duration_since(start)
                >= Duration::from_millis(TAKE_BACK_READY_WAIT_MS)
            {
                return Ok(false);
            }
            if now.saturating_duration_since(last_check) >= Duration::from_millis(STARTED_POLL_MS) {
                last_check = now;
                started_meanwhile()?;
                if !stuck_named
                    && now.saturating_duration_since(start)
                        >= Duration::from_millis(TAKE_BACK_STUCK_SERVICE_MS)
                    && self.adapter.claims_seen() > claims_at_quit
                    && self.stuck_service()
                {
                    stuck_named = true;
                    self.adapter
                        .set_auto_waiting(Some(CAMERA_SERVICE_STUCK.into()));
                }
            }
            let seen = self.adapter.claims_seen();
            if seen > claims
                && now.saturating_duration_since(start)
                    < Duration::from_millis(TAKE_BACK_FREE_WAIT_MS)
                && now.saturating_duration_since(last_send)
                    >= Duration::from_millis(RETRY_AFTER_CLAIM_MS)
            {
                claims = seen;
                last_send = now;
                send();
            }
            thread::sleep(REAL_POLL);
        }
    }

    /// P60 #25: after Capture One quit, the cable is still held (claimed again since the quit,
    /// checked by the caller) and no other program that uses cameras runs — macOS's camera
    /// service holds it on nobody's behalf.
    fn stuck_service(&self) -> bool {
        if !self.adapter.cable_held() {
            return false;
        }
        let running = self.runtime.process_list().running().unwrap_or_default();
        known_running(&running).is_empty()
    }

    /// "Test camera connection" (MC-34 §Setup check): the known programs running now, in every
    /// case; the holder and its sentence only while the status is `HeldByOtherProgram` (L19).
    pub fn holder_report(&self) -> HolderReport {
        let running = self
            .runtime
            .process_list()
            .running()
            .unwrap_or_else(|error| {
                tracing::warn!(%error, "process list unavailable");
                Vec::new()
            });
        let programs = known_running(&running);
        match self.adapter.status_holder_process() {
            Some(owner) => {
                let named = name_programs(&owner, &running);
                HolderReport {
                    holder: named.holder,
                    programs,
                    sentence: named.sentence,
                }
            }
            None => HolderReport {
                holder: None,
                programs,
                sentence: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready() -> TakeBackInputs {
        TakeBackInputs {
            holder_is_capture_one: true,
            ..Default::default()
        }
    }
    fn reason(inputs: TakeBackInputs) -> Option<&'static str> {
        preconditions(&inputs).err().map(|b| b.reason)
    }

    #[test]
    fn c4_all_clear() {
        assert_eq!(reason(ready()), None);
    }
    #[test]
    fn c4a_a_file_is_arriving() {
        let upload = TakeBackInputs {
            uploads_in_progress: 1,
            ..ready()
        };
        assert_eq!(reason(upload), Some("a file is still arriving"));
        let download = TakeBackInputs {
            expected_transfer: true,
            ..ready()
        };
        assert_eq!(reason(download), Some("a file is still arriving"));
    }
    #[test]
    fn c4b_a_photo_is_expected() {
        let young = TakeBackInputs {
            youngest_expecting_exposure_age_ms: Some(5_000),
            ..ready()
        };
        assert_eq!(reason(young), Some("a photo is still expected"));
        let old = TakeBackInputs {
            youngest_expecting_exposure_age_ms: Some(11_000),
            ..ready()
        };
        assert_eq!(reason(old), None);
    }
    #[test]
    fn c4c_export_running() {
        for jobs in [
            JobsRunning {
                render_master: 1,
                ..Default::default()
            },
            JobsRunning {
                render_output: 1,
                ..Default::default()
            },
        ] {
            let inputs = TakeBackInputs {
                jobs_running: jobs,
                ..ready()
            };
            assert_eq!(reason(inputs), Some("an export is running"));
        }
    }
    #[test]
    fn c4d_publish_running() {
        let inputs = TakeBackInputs {
            jobs_running: JobsRunning {
                publish_step: 1,
                ..Default::default()
            },
            ..ready()
        };
        assert_eq!(reason(inputs), Some("a publish is running"));
    }
    #[test]
    fn c4e_preview_rendering() {
        let inputs = TakeBackInputs {
            renders_in_flight: 1,
            ..ready()
        };
        assert_eq!(reason(inputs), Some("a preview is rendering"));
    }
    #[test]
    fn c4f_not_capture_one() {
        assert_eq!(
            reason(TakeBackInputs::default()),
            Some("Capture One is not holding the camera")
        );
    }
    #[test]
    fn c4_check_order() {
        let everything = TakeBackInputs {
            uploads_in_progress: 1,
            expected_transfer: true,
            youngest_expecting_exposure_age_ms: Some(0),
            jobs_running: JobsRunning {
                render_master: 1,
                render_output: 1,
                publish_step: 1,
            },
            renders_in_flight: 1,
            holder_is_capture_one: false,
        };
        let mut seen = Vec::new();
        let mut inputs = everything;
        while let Some(r) = reason(inputs.clone()) {
            seen.push(r);
            match r {
                "a file is still arriving" => {
                    inputs.uploads_in_progress = 0;
                    inputs.expected_transfer = false;
                }
                "a photo is still expected" => inputs.youngest_expecting_exposure_age_ms = None,
                "an export is running" => {
                    inputs.jobs_running.render_master = 0;
                    inputs.jobs_running.render_output = 0;
                }
                "a publish is running" => inputs.jobs_running.publish_step = 0,
                "a preview is rendering" => inputs.renders_in_flight = 0,
                _ => inputs.holder_is_capture_one = true,
            }
        }
        assert_eq!(
            seen,
            vec![
                "a file is still arriving",
                "a photo is still expected",
                "an export is running",
                "a publish is running",
                "a preview is rendering",
                "Capture One is not holding the camera",
            ]
        );
    }
}
