use psm_core::{
    Error, Result,
    domain::{
        camera::{CameraCapabilities, CameraStatus, CaptureId, Link},
        live::{
            CameraFrame, CaptureReportState, ExposureReadout, FocusId, FocusOutcome, FocusPoint,
            LiveEvent, LiveState, PublishOutcome, SourceStop, ViewerDemand,
        },
    },
    ports::camera::CameraProvider,
    services::live_view::{LatestSlot, LiveViewHub, focus, focus_clear},
};
use psm_testkit::live_outlet::RecordingOutlet;
use std::sync::{Arc, Mutex};

fn frame(source: u64, number: u32, byte: u8) -> CameraFrame {
    CameraFrame {
        source,
        frame_no: number,
        width: 2,
        height: 1,
        jpeg: vec![byte],
        link: Link::Cable,
    }
}
fn want(r#gen: u32) -> ViewerDemand {
    ViewerDemand {
        r#gen,
        live: true,
        visible: true,
    }
}
fn setup() -> (
    LiveViewHub,
    Arc<RecordingOutlet>,
    psm_core::domain::live::ViewerId,
) {
    let h = LiveViewHub::default();
    let o = Arc::new(RecordingOutlet::default());
    let id = h.subscribe(o.clone());
    h.source_started(Link::Cable, 1);
    h.set_demand(id, want(1));
    (h, o, id)
}

#[test]
fn lv1_c6a_forwards_in_order() {
    let (h, o, _) = setup();
    for n in 1..=3 {
        assert_eq!(
            h.publish(frame(1, n, n as u8)),
            PublishOutcome::Forwarded { seq: n as u64 }
        );
    }
    let got = o.offered();
    assert_eq!(
        got.iter().map(|(f, _)| f.seq).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(
        got.iter().map(|(f, g)| (*g, f.jpeg[0])).collect::<Vec<_>>(),
        vec![(1, 1), (1, 2), (1, 3)]
    );
}
#[test]
fn lv1_c6b_repeats_drop() {
    let (h, o, _) = setup();
    h.publish(frame(1, 3, 3));
    assert_eq!(h.publish(frame(1, 3, 7)), PublishOutcome::Repeat);
    assert_eq!(h.publish(frame(1, 2, 7)), PublishOutcome::Repeat);
    assert_eq!(o.offered().len(), 1);
}
#[test]
fn lv1_c6c_new_source_resets_number_not_sequence() {
    let (h, o, _) = setup();
    for n in 1..=3 {
        h.publish(frame(1, n, 1));
    }
    h.source_started(Link::Cable, 2);
    assert_eq!(
        h.publish(frame(2, 1, 9)),
        PublishOutcome::Forwarded { seq: 4 }
    );
    assert_eq!(o.offered().last().unwrap().0.jpeg.as_ref(), &[9]);
}
#[test]
fn lv1_c6d_no_start_is_stale() {
    let h = LiveViewHub::default();
    let o = Arc::new(RecordingOutlet::default());
    h.subscribe(o.clone());
    assert_eq!(h.publish(frame(1, 1, 1)), PublishOutcome::Stale);
    assert!(h.newest().is_none());
    assert!(o.offered().is_empty());
}
#[test]
fn lv1_c6e_unnumbered_both_forwarded() {
    let (h, o, _) = setup();
    assert_eq!(
        h.publish(frame(1, 0, 1)),
        PublishOutcome::Forwarded { seq: 1 }
    );
    assert_eq!(
        h.publish(frame(1, 0, 1)),
        PublishOutcome::Forwarded { seq: 2 }
    );
    assert_eq!(o.offered().len(), 2);
}
#[test]
fn lv1_c7a_template_demand_gets_nothing() {
    let (h, o, id) = setup();
    h.set_demand(
        id,
        ViewerDemand {
            r#gen: 1,
            live: false,
            visible: true,
        },
    );
    for n in 1..=5 {
        h.publish(frame(1, n, 1));
    }
    assert!(o.offered().is_empty());
}
#[test]
fn lv1_c7b_hidden_demand_gets_nothing() {
    let (h, o, id) = setup();
    h.set_demand(
        id,
        ViewerDemand {
            r#gen: 1,
            live: true,
            visible: false,
        },
    );
    for n in 1..=5 {
        h.publish(frame(1, n, 1));
    }
    assert!(o.offered().is_empty());
}
#[test]
fn lv1_c7c_switch_does_not_offer_old_frame() {
    let h = LiveViewHub::default();
    let o = Arc::new(RecordingOutlet::default());
    let id = h.subscribe(o.clone());
    h.source_started(Link::Cable, 1);
    h.publish(frame(1, 1, 1));
    h.set_demand(id, want(1));
    assert!(o.offered().is_empty());
    h.publish(frame(1, 2, 2));
    assert_eq!(
        o.offered()
            .iter()
            .map(|(f, _)| f.jpeg[0])
            .collect::<Vec<_>>(),
        vec![2]
    );
}
#[test]
fn lv1_c7d_switch_off_withdraws() {
    let (h, o, id) = setup();
    h.publish(frame(1, 1, 1));
    let before = o.withdrawals();
    h.set_demand(
        id,
        ViewerDemand {
            r#gen: 2,
            live: false,
            visible: true,
        },
    );
    h.publish(frame(1, 2, 2));
    assert_eq!(o.withdrawals(), before + 1);
    assert!(o.take().is_none());
    assert_eq!(o.offered().len(), 1);
}
#[test]
fn lv1_c7e_switch_back_generation() {
    let (h, o, id) = setup();
    h.publish(frame(1, 1, 1));
    h.set_demand(
        id,
        ViewerDemand {
            r#gen: 2,
            live: false,
            visible: true,
        },
    );
    h.publish(frame(1, 2, 2));
    h.set_demand(id, want(3));
    h.publish(frame(1, 3, 3));
    let got = o.offered();
    assert_eq!(got.len(), 2);
    assert_eq!(got[1].1, 3);
    assert!(got[1].0.seq > got[0].0.seq);
}
#[test]
fn lv1_c8a_latest_slot_replaces() {
    let s = LatestSlot::default();
    s.put("a");
    s.put("b");
    assert_eq!(s.take(), Some("b"));
    assert_eq!(s.take(), None);
}
#[test]
fn lv1_c8b_slow_viewer_never_delays_other() {
    let h = LiveViewHub::default();
    let a = Arc::new(RecordingOutlet::default());
    let b = Arc::new(RecordingOutlet::default());
    let ai = h.subscribe(a.clone());
    let bi = h.subscribe(b.clone());
    h.source_started(Link::Cable, 1);
    h.set_demand(ai, want(1));
    h.set_demand(bi, want(1));
    for n in 1..=10 {
        assert_eq!(
            h.publish(frame(1, n, n as u8)),
            PublishOutcome::Forwarded { seq: n as u64 }
        );
        assert_eq!(b.take().unwrap().0.seq, n as u64);
    }
    assert_eq!(a.take().unwrap().0.seq, 10);
    assert!(a.take().is_none());
}
#[test]
fn lv1_c9a_events_reach_subscribed_regardless_of_demand() {
    let h = LiveViewHub::default();
    let a = Arc::new(RecordingOutlet::default());
    let b = Arc::new(RecordingOutlet::default());
    let c = Arc::new(RecordingOutlet::default());
    let ai = h.subscribe(a.clone());
    h.set_demand(ai, want(1));
    h.subscribe(b.clone());
    let ci = h.subscribe(c.clone());
    h.unsubscribe(ci);
    let trace = Arc::new(Mutex::new(vec![]));
    *a.trace.lock().unwrap() = Some(("a".into(), trace.clone()));
    *b.trace.lock().unwrap() = Some(("b".into(), trace.clone()));
    let e = LiveEvent::Focus {
        focus_id: FocusId("k".into()),
        outcome: FocusOutcome::Focused,
    };
    h.report(e.clone());
    assert_eq!(a.events(), vec![e.clone()]);
    assert_eq!(b.events(), vec![e]);
    assert!(c.events().is_empty());
    assert_eq!(*trace.lock().unwrap(), vec!["a", "b"]);
}
#[test]
fn lv1_c9b_event_order() {
    let h = LiveViewHub::default();
    let o = Arc::new(RecordingOutlet::default());
    h.subscribe(o.clone());
    let events = vec![
        LiveEvent::FocusFrames(vec![]),
        LiveEvent::Exposure(ExposureReadout {
            shutter: "1/125".into(),
            aperture: "f/8".into(),
            iso: "ISO 100".into(),
        }),
        LiveEvent::Capture {
            capture_id: "c".into(),
            state: CaptureReportState::Exposed,
        },
    ];
    for e in &events {
        h.report(e.clone());
    }
    assert_eq!(o.events(), events);
}
struct Provider {
    caps: bool,
    calls: Mutex<Vec<(FocusPoint, bool)>>,
}
impl Provider {
    fn new(caps: bool) -> Self {
        Self {
            caps,
            calls: Mutex::new(vec![]),
        }
    }
}
impl CameraProvider for Provider {
    fn raw_extensions(&self) -> &'static [&'static str] {
        &[]
    }
    fn status(&self) -> Result<CameraStatus> {
        Ok(CameraStatus::Disconnected)
    }
    fn capture(&self) -> Result<CaptureId> {
        Ok(CaptureId("c".into()))
    }
    fn capabilities(&self) -> CameraCapabilities {
        CameraCapabilities {
            status: true,
            capture: true,
            transfer: false,
            exposure_events: false,
            settings_read: false,
            settings_write_store_destination: false,
            depends_on_editing_provider: false,
            live_view: self.caps,
            focus_point: self.caps,
        }
    }
    fn focus(&self, p: FocusPoint, af: bool) -> Result<FocusId> {
        self.calls.lock().unwrap().push((p, af));
        Ok(FocusId("id".into()))
    }
}
#[test]
fn lv1_c10a_unsupported_no_call() {
    let p = Provider::new(false);
    assert!(matches!(
        focus(&p, FocusPoint { x: 0.5, y: 0.5 }),
        Err(Error::FocusUnsupported)
    ));
    assert!(p.calls.lock().unwrap().is_empty());
}
#[test]
fn lv1_c10b_valid_focus_passes_point_and_af() {
    let p = Provider::new(true);
    assert_eq!(
        focus(&p, FocusPoint { x: 0.25, y: 0.75 }).unwrap(),
        FocusId("id".into())
    );
    assert_eq!(
        *p.calls.lock().unwrap(),
        vec![(FocusPoint { x: 0.25, y: 0.75 }, true)]
    );
}
#[test]
fn lv1_c10c_invalid_points_do_not_call() {
    let p = Provider::new(true);
    for point in [
        FocusPoint { x: 1.5, y: 0.5 },
        FocusPoint {
            x: f64::NAN,
            y: 0.5,
        },
        FocusPoint { x: 0.5, y: -0.01 },
    ] {
        assert!(matches!(focus(&p, point), Err(Error::InvalidValue(_))));
    }
    assert!(p.calls.lock().unwrap().is_empty());
}
#[test]
fn lv1_c10d_clear_recenters_without_af() {
    let p = Provider::new(true);
    focus_clear(&p).unwrap();
    assert_eq!(
        *p.calls.lock().unwrap(),
        vec![(FocusPoint { x: 0.5, y: 0.5 }, false)]
    );
}
#[test]
fn lv1_c11a_stop_clears_newest_and_rejects_frames() {
    let (h, o, _) = setup();
    h.publish(frame(1, 1, 1));
    h.source_stopped(SourceStop::SessionClosed);
    assert_eq!(h.state(), LiveState::Off);
    assert!(h.newest().is_none());
    assert!(o.events().contains(&LiveEvent::State(LiveState::Off)));
    assert_eq!(h.publish(frame(1, 2, 2)), PublishOutcome::Stale);
}
#[test]
fn lv1_c11b_unavailable_state() {
    let (h, o, _) = setup();
    h.source_stopped(SourceStop::Unavailable {
        reason: "not_supported".into(),
    });
    let expected = LiveState::Unavailable {
        reason: "not_supported".into(),
    };
    assert_eq!(h.state(), expected);
    assert!(o.events().contains(&LiveEvent::State(expected)));
}
#[test]
fn lv1_c11c_unsubscribe_stops_one_viewer() {
    let (h, a, ai) = setup();
    let b = Arc::new(RecordingOutlet::default());
    let bi = h.subscribe(b.clone());
    h.set_demand(bi, want(1));
    h.unsubscribe(ai);
    for n in 1..=3 {
        h.publish(frame(1, n, 1));
    }
    assert!(a.offered().is_empty());
    assert_eq!(b.offered().len(), 3);
}
#[test]
fn lv1_c11d_new_subscription_defaults_not_wanting() {
    let h = LiveViewHub::default();
    h.source_started(Link::Cable, 1);
    let o = Arc::new(RecordingOutlet::default());
    h.subscribe(o.clone());
    for n in 1..=3 {
        h.publish(frame(1, n, 1));
    }
    assert!(o.offered().is_empty());
}
