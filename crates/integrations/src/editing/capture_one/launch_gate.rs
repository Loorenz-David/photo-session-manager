//! The one editor-launch gate (10E MC-34 R-I32, M40): while "Take the camera back" holds it, a
//! script that would launch Capture One runs only on the thread the take-back permitted (its
//! step 5). One gate per app process (P48 L14): created beside the scripting lanes and never
//! rebuilt by a settings save.
use psm_core::ports::clock::Clock;
use std::{
    cell::RefCell,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

thread_local! {
    /// The gates (by address) this thread is currently permitted through.
    static PERMITS: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

#[derive(Default)]
pub struct LaunchGate {
    held: AtomicBool,
    /// A take-back holds it (P60 #22): every lane script is refused, not only launch-capable ones.
    quiet: AtomicBool,
    /// Scripts the scripting lanes are running against Capture One right now (P60 #22).
    in_flight: AtomicUsize,
}

/// One lane script in flight; dropping it ends it.
pub struct InFlight<'a>(&'a AtomicUsize);
impl Drop for InFlight<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

impl LaunchGate {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
    /// Holds the gate until the returned guard is dropped.
    pub fn hold(self: &Arc<Self>) -> GateGuard {
        self.held.store(true, Ordering::SeqCst);
        GateGuard { gate: self.clone() }
    }
    /// Holds the gate for "Take the camera back": also refuses every other lane script off the
    /// permitted thread until the guard drops (P60 #22).
    pub fn hold_quiet(self: &Arc<Self>) -> GateGuard {
        let guard = self.hold();
        self.quiet.store(true, Ordering::SeqCst);
        guard
    }
    /// Any lane script must be refused now: a take-back holds the gate on another thread.
    pub fn quiet_here(&self) -> bool {
        self.quiet.load(Ordering::SeqCst) && !self.permitted_here()
    }
    pub fn is_held(&self) -> bool {
        self.held.load(Ordering::SeqCst)
    }
    /// Whether the current thread runs inside `GateGuard::permit` of this gate.
    pub fn permitted_here(&self) -> bool {
        let id = self.id();
        PERMITS.with(|p| p.borrow().contains(&id))
    }
    /// A launch-capable script must be refused now: held, and not on the permitted thread.
    pub fn refuses_here(&self) -> bool {
        self.is_held() && !self.permitted_here()
    }
    /// Marks a lane script as running until the guard drops.
    pub fn enter(&self) -> InFlight<'_> {
        self.in_flight.fetch_add(1, Ordering::SeqCst);
        InFlight(&self.in_flight)
    }
    /// P60 #22: a script that was already talking to Capture One when the gate was taken would wake
    /// it again with its next Apple event after the quit. Waits until none is running (new ones are
    /// refused while the gate is held), at most `bound` on `clock`; `false` at the bound.
    pub fn wait_idle(&self, clock: &dyn Clock, bound: Duration) -> bool {
        let start = clock.mono();
        while self.in_flight.load(Ordering::SeqCst) > 0 {
            if clock.mono().saturating_duration_since(start) >= bound {
                return false;
            }
            thread::sleep(Duration::from_millis(10));
        }
        true
    }
    fn id(&self) -> usize {
        self as *const Self as usize
    }
}

/// The held gate; dropping it releases the gate (R-I32 step 6, also on every failure path).
pub struct GateGuard {
    gate: Arc<LaunchGate>,
}

impl GateGuard {
    /// Runs `f` with the current thread permitted through the gate.
    pub fn permit<T>(&self, f: impl FnOnce() -> T) -> T {
        struct Revoke(usize);
        impl Drop for Revoke {
            fn drop(&mut self) {
                PERMITS.with(|p| {
                    let mut p = p.borrow_mut();
                    if let Some(at) = p.iter().rposition(|id| *id == self.0) {
                        p.remove(at);
                    }
                });
            }
        }
        let id = self.gate.id();
        PERMITS.with(|p| p.borrow_mut().push(id));
        let _revoke = Revoke(id);
        f()
    }
}

impl Drop for GateGuard {
    fn drop(&mut self) {
        self.gate.quiet.store(false, Ordering::SeqCst);
        self.gate.held.store(false, Ordering::SeqCst);
    }
}
