//! The one editor-launch gate (10E MC-34 R-I32, M40): while "Take the camera back" holds it, a
//! script that would launch Capture One runs only on the thread the take-back permitted (its
//! step 5). One gate per app process (P48 L14): created beside the scripting lanes and never
//! rebuilt by a settings save.
use std::{
    cell::RefCell,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

thread_local! {
    /// The gates (by address) this thread is currently permitted through.
    static PERMITS: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

#[derive(Default)]
pub struct LaunchGate {
    held: AtomicBool,
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
        self.gate.held.store(false, Ordering::SeqCst);
    }
}
