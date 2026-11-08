//! The start order (10E MC-34 R-I31, M40): at app start with camera support installed, Capture
//! One is launched only after the USB session had its chance — `connection.ready` on USB, no
//! camera on the cable, the Wi-Fi fallback, the sidecar unavailable or gone, Capture One already
//! running, or `START_ORDER_WAIT_MS`. One per app process (P48 L14); once satisfied it never
//! waits again.
use super::START_ORDER_WAIT_MS;
use psm_core::ports::clock::Clock;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

/// Why the start order no longer holds a launch back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartReason {
    /// No camera support package was installed at app start.
    NotNeeded,
    /// `connection.ready` on USB.
    UsbReady,
    /// `usb.initial { present: false }`.
    NoCameraOnCable,
    /// The sidecar fell back to Wi-Fi after its USB failures.
    FallbackToWifi,
    /// `sdk.unavailable`, a failed spawn or a sidecar exit.
    SidecarUnavailable,
    /// Capture One was already running when an ensure first asked (L4b).
    AlreadyRunning,
    /// `START_ORDER_WAIT_MS` passed since the first `wait()` (L4c).
    Timeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartState {
    Waiting,
    Satisfied(StartReason),
}

struct Inner {
    state: StartState,
    first_wait: Option<Instant>,
    usb_present: Option<bool>,
    usb_ready: bool,
}

pub struct StartOrder {
    inner: Mutex<Inner>,
    relaunch_retry: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

impl StartOrder {
    /// `needed` = a camera support package is installed at app start.
    pub fn new(needed: bool) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Inner {
                state: if needed {
                    StartState::Waiting
                } else {
                    StartState::Satisfied(StartReason::NotNeeded)
                },
                first_wait: None,
                usb_present: None,
                usb_ready: false,
            }),
            relaunch_retry: Mutex::new(None),
        })
    }
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|p| p.into_inner())
    }
    pub fn state(&self) -> StartState {
        self.lock().state
    }
    /// A `wait()` has started and the order is not satisfied yet.
    pub fn is_waiting(&self) -> bool {
        let inner = self.lock();
        inner.state == StartState::Waiting && inner.first_wait.is_some()
    }
    /// Live cable state is separate from the one-shot app-start gate. A later editor relaunch
    /// must give the helper another chance to claim an attached cable (V6).
    pub fn camera_state(&self, present: Option<bool>, ready: Option<bool>) {
        let mut inner = self.lock();
        if let Some(present) = present {
            inner.usb_present = Some(present);
            if !present {
                inner.usb_ready = false;
            }
        }
        if let Some(ready) = ready {
            inner.usb_ready = ready;
        }
    }
    pub fn needs_relaunch_wait(&self) -> bool {
        let inner = self.lock();
        inner.usb_present == Some(true) && !inner.usb_ready
    }
    /// Installs the helper command used to give USB another chance during a later relaunch wait.
    pub fn set_relaunch_retry(&self, retry: Arc<dyn Fn() + Send + Sync>) {
        *self
            .relaunch_retry
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some(retry);
    }
    pub fn wait_for_relaunch(&self, clock: &dyn Clock) {
        if self.needs_relaunch_wait()
            && let Some(retry) = self
                .relaunch_retry
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .clone()
        {
            retry();
        }
        let first = clock.mono();
        loop {
            let inner = self.lock();
            if inner.usb_present != Some(true)
                || inner.usb_ready
                || clock.mono().saturating_duration_since(first)
                    >= Duration::from_millis(START_ORDER_WAIT_MS)
            {
                return;
            }
            drop(inner);
            thread::sleep(Duration::from_millis(50));
        }
    }
    /// The first condition that holds wins; later ones change nothing.
    pub fn satisfy(&self, reason: StartReason) {
        let mut inner = self.lock();
        if inner.state == StartState::Waiting {
            inner.state = StartState::Satisfied(reason);
        }
    }
    /// Blocks until satisfied, polling every 50 ms; the bound counts on `clock` from the first
    /// call in the process.
    pub fn wait(&self, clock: &dyn Clock) -> StartReason {
        loop {
            {
                let mut inner = self.lock();
                if let StartState::Satisfied(reason) = inner.state {
                    return reason;
                }
                let now = clock.mono();
                let first = *inner.first_wait.get_or_insert(now);
                if now.saturating_duration_since(first)
                    >= Duration::from_millis(START_ORDER_WAIT_MS)
                {
                    inner.state = StartState::Satisfied(StartReason::Timeout);
                    return StartReason::Timeout;
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
    }
}
