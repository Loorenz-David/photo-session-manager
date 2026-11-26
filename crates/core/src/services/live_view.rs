use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::{
    Error, Result,
    domain::{
        camera::Link,
        live::{
            CameraFrame, FocusId, FocusPoint, LiveEvent, LiveFrame, LiveState, PublishOutcome,
            SourceStop, ViewerDemand, ViewerId,
        },
    },
    ports::{
        camera::CameraProvider,
        clock::{Clock, SystemClock},
    },
};

pub trait LiveOutlet: Send + Sync {
    fn offer(&self, frame: Arc<LiveFrame>, generation: u32);
    fn withdraw(&self);
    fn event(&self, event: &LiveEvent);
}

pub struct LatestSlot<T>(Mutex<Option<T>>);
impl<T> Default for LatestSlot<T> {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}
impl<T> LatestSlot<T> {
    pub fn put(&self, value: T) {
        *self.0.lock().unwrap() = Some(value);
    }
    pub fn take(&self) -> Option<T> {
        self.0.lock().unwrap().take()
    }
    pub fn clear(&self) {
        self.0.lock().unwrap().take();
    }
}

struct Viewer {
    outlet: Arc<dyn LiveOutlet>,
    demand: ViewerDemand,
    switched_at: Instant,
    last_offered: u64,
}
struct Inner {
    source: Option<u64>,
    link: Option<Link>,
    last_frame_no: Option<u32>,
    start: u32,
    seq: u64,
    newest: Option<Arc<LiveFrame>>,
    state: LiveState,
    next_viewer: u64,
    viewers: BTreeMap<u64, Viewer>,
}
pub struct LiveViewHub {
    inner: Mutex<Inner>,
    clock: Arc<dyn Clock>,
}
impl Default for LiveViewHub {
    fn default() -> Self {
        Self::new(Arc::new(SystemClock))
    }
}
impl LiveViewHub {
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self {
            clock,
            inner: Mutex::new(Inner {
                source: None,
                link: None,
                last_frame_no: None,
                start: 0,
                seq: 0,
                newest: None,
                state: LiveState::Off,
                next_viewer: 1,
                viewers: BTreeMap::new(),
            }),
        }
    }
    pub fn source_started(&self, link: Link, source: u64) {
        let mut g = self.inner.lock().unwrap();
        g.start += 1;
        g.source = Some(source);
        g.link = Some(link.clone());
        g.last_frame_no = None;
        g.newest = None;
        g.state = LiveState::Live { link };
        let event = LiveEvent::State(g.state.clone());
        for viewer in g.viewers.values() {
            viewer.outlet.withdraw();
            viewer.outlet.event(&event);
        }
    }
    pub fn source_stopped(&self, reason: SourceStop) {
        let mut g = self.inner.lock().unwrap();
        g.source = None;
        g.link = None;
        g.last_frame_no = None;
        g.newest = None;
        g.state = match reason {
            SourceStop::Unavailable { reason } => LiveState::Unavailable { reason },
            _ => LiveState::Off,
        };
        let event = LiveEvent::State(g.state.clone());
        for viewer in g.viewers.values() {
            viewer.outlet.withdraw();
            viewer.outlet.event(&event);
        }
    }
    pub fn publish(&self, frame: CameraFrame) -> PublishOutcome {
        let mut g = self.inner.lock().unwrap();
        if g.source != Some(frame.source) {
            return PublishOutcome::Stale;
        }
        if frame.frame_no != 0 && g.last_frame_no.is_some_and(|n| frame.frame_no <= n) {
            return PublishOutcome::Repeat;
        }
        if frame.frame_no != 0 {
            g.last_frame_no = Some(frame.frame_no);
        }
        g.seq += 1;
        let seq = g.seq;
        let received = Instant::now();
        let out = Arc::new(LiveFrame {
            seq,
            start: g.start,
            received,
            captured_at_ms: self.clock.now_ms(),
            width: frame.width,
            height: frame.height,
            jpeg: Arc::from(frame.jpeg),
            link: frame.link,
        });
        g.newest = Some(out.clone());
        for viewer in g.viewers.values_mut() {
            if viewer.demand.wanting() && seq > viewer.last_offered && received > viewer.switched_at
            {
                viewer.outlet.offer(out.clone(), viewer.demand.r#gen);
                viewer.last_offered = seq;
            }
        }
        PublishOutcome::Forwarded { seq }
    }
    pub fn subscribe(&self, outlet: Arc<dyn LiveOutlet>) -> ViewerId {
        let mut g = self.inner.lock().unwrap();
        let id = g.next_viewer;
        g.next_viewer += 1;
        g.viewers.insert(
            id,
            Viewer {
                outlet,
                demand: ViewerDemand {
                    r#gen: 0,
                    live: false,
                    visible: false,
                },
                switched_at: Instant::now(),
                last_offered: 0,
            },
        );
        ViewerId(id)
    }
    pub fn set_demand(&self, id: ViewerId, demand: ViewerDemand) {
        let mut g = self.inner.lock().unwrap();
        if let Some(viewer) = g.viewers.get_mut(&id.0) {
            if demand.wanting() && (!viewer.demand.wanting() || viewer.demand.r#gen != demand.r#gen)
            {
                viewer.switched_at = Instant::now();
                viewer.outlet.withdraw();
            } else if !demand.wanting() {
                viewer.outlet.withdraw();
            }
            viewer.demand = demand;
        }
    }
    pub fn unsubscribe(&self, id: ViewerId) {
        if let Some(viewer) = self.inner.lock().unwrap().viewers.remove(&id.0) {
            viewer.outlet.withdraw();
        }
    }
    pub fn report(&self, event: LiveEvent) {
        let g = self.inner.lock().unwrap();
        for viewer in g.viewers.values() {
            viewer.outlet.event(&event);
        }
    }
    pub fn newest(&self) -> Option<Arc<LiveFrame>> {
        self.inner.lock().unwrap().newest.clone()
    }
    pub fn state(&self) -> LiveState {
        self.inner.lock().unwrap().state.clone()
    }
}

pub fn focus(provider: &dyn CameraProvider, point: FocusPoint) -> Result<FocusId> {
    if !provider.capabilities().focus_point {
        return Err(Error::FocusUnsupported);
    }
    if !point.x.is_finite()
        || !point.y.is_finite()
        || !(0.0..=1.0).contains(&point.x)
        || !(0.0..=1.0).contains(&point.y)
    {
        return Err(Error::InvalidValue("focus point".into()));
    }
    provider.focus(point, true)
}
pub fn focus_clear(provider: &dyn CameraProvider) -> Result<FocusId> {
    if !provider.capabilities().focus_point {
        return Err(Error::FocusUnsupported);
    }
    provider.focus(FocusPoint { x: 0.5, y: 0.5 }, false)
}
