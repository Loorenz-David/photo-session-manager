use psm_core::{
    Error,
    domain::live::{
        ExposureReadout, FocusFrameState, FocusOutcome, FocusPoint, FocusRefusal, LiveEvent,
        LiveState, ViewerDemand,
    },
    ports::camera::CameraProvider,
    services::live_view::{focus, focus_clear},
};
use psm_integrations::camera::{
    direct::live::{frame_norm, to_grid},
    sidecar::{
        frames::{FrameError, parse_record},
        protocol::{Command, SidecarEvent, Transport, parse_line},
    },
};
use psm_testkit::{TestApp, live_outlet::RecordingOutlet};
use serde_json::json;
use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

const READY: &str = r#"{"timestamp":"2026-11-06T10:00:00.000Z","source":"sdk","event":"connection.ready","transport":"usb","model":"ILCE-7RM5","store_destination":"verified","store_value":"0x0003"}
"#;
fn line(event: &str, fields: serde_json::Value) -> String {
    let mut v = json!({"timestamp":"2026-11-06T10:00:00.000Z","source":"sdk","event":event});
    v.as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    v.to_string()
}
fn settle(a: &TestApp, pred: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(5);
    while Instant::now() < end {
        a.tick(0).unwrap();
        if pred() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!(
        "live view did not settle: events={:?} log={:?} stdin={:?}",
        a.sidecar_events(),
        a.camera_runtime().helper_log(),
        a.fake_sidecar_recording().stdin
    );
}
fn app() -> TestApp {
    let a = TestApp::with_camera_script(READY).unwrap();
    settle(&a, || {
        a.camera_status()
            .is_ok_and(|s| matches!(s, psm_core::domain::camera::CameraStatus::Connected { .. }))
    });
    a
}
fn viewing(a: &TestApp) -> Arc<RecordingOutlet> {
    let o = Arc::new(RecordingOutlet::default());
    let id = a.live_hub().subscribe(o.clone());
    a.live_hub().set_demand(
        id,
        ViewerDemand {
            r#gen: 1,
            live: true,
            visible: true,
        },
    );
    o
}
fn start(a: &TestApp) {
    a.inject(
        "liveview.started",
        json!({"epoch":1,"transport":"usb","status":"enabled","frame_channel":"connected"}),
    )
    .unwrap();
}
fn stdin(a: &TestApp) -> Vec<String> {
    a.fake_sidecar_recording()
        .stdin
        .into_iter()
        .map(|(_, s)| s)
        .collect()
}
fn record(epoch: u32, number: u32, w: u16, h: u16, len: usize) -> Vec<u8> {
    let mut b = b"PSLV".to_vec();
    b.extend_from_slice(&[1, 1, 0, 0]);
    b.extend_from_slice(&epoch.to_le_bytes());
    b.extend_from_slice(&number.to_le_bytes());
    b.extend_from_slice(&w.to_le_bytes());
    b.extend_from_slice(&h.to_le_bytes());
    b.extend_from_slice(&(len as u32).to_le_bytes());
    b.extend((0..len).map(|i| ((epoch as usize + number as usize + i) % 251) as u8));
    b
}

#[test]
fn lv1_c12a_started_protocol() {
    let e = parse_line(&line(
        "liveview.started",
        json!({"epoch":1,"transport":"usb","status":"enabled","frame_channel":"connected"}),
    ))
    .unwrap()
    .event;
    assert_eq!(
        e,
        SidecarEvent::LiveviewStarted {
            epoch: 1,
            transport: Transport::Usb,
            status: "enabled".into(),
            frame_channel: "connected".into()
        }
    );
}
#[test]
fn lv1_c12b_focus_result_protocol() {
    let e = parse_line(&line(
        "focus.result",
        json!({"cmd_id":"k","x":320,"y":240,"af":"focused","point_set":true,"total_ms":412}),
    ))
    .unwrap()
    .event;
    assert_eq!(
        e,
        SidecarEvent::FocusResult {
            cmd_id: "k".into(),
            x: 320,
            y: 240,
            af: "focused".into(),
            point_set: true,
            refused: None,
            point_ms: None,
            af_ms: None,
            total_ms: 412
        }
    );
}
#[test]
fn lv1_c12c_focus_frame_protocol() {
    let e = parse_line(&line(
        "liveview.focus_frame",
        json!({"frames":[[1,2,320,640,240,480,38,38]]}),
    ))
    .unwrap()
    .event;
    assert_eq!(
        e,
        SidecarEvent::LiveviewFocusFrame {
            frames: vec![[1, 2, 320, 640, 240, 480, 38, 38]]
        }
    );
}
#[test]
fn lv1_c12d_focus_command_encoding() {
    assert_eq!(
        Command::Focus {
            x: 320,
            y: 240,
            af: true,
            cmd_id: "k1".into()
        }
        .encode(),
        "focus x=320 y=240 af=1 cmd_id=k1"
    );
    assert_eq!(
        Command::Focus {
            x: 0,
            y: 479,
            af: false,
            cmd_id: "k2".into()
        }
        .encode(),
        "focus x=0 y=479 af=0 cmd_id=k2"
    );
}
#[test]
fn lv1_c12e_unknown_event_ignored() {
    assert_eq!(
        parse_line(&line("liveview.zoom", json!({}))).unwrap().event,
        SidecarEvent::Unknown {
            event: "liveview.zoom".into()
        }
    );
}
#[test]
fn lv1_c13a_binary_frames_reach_hub() {
    let a = app();
    let o = viewing(&a);
    start(&a);
    a.inject_sidecar_directive("frames 1 1 5 20").unwrap();
    settle(&a, || o.offered().len() == 5);
    let got = o.offered();
    assert_eq!(
        got.iter().map(|(f, _)| f.seq).collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5]
    );
    for (i, (f, g)) in got.iter().enumerate() {
        assert_eq!(*g, 1);
        assert_eq!((f.width, f.height), (640, 424));
        assert_eq!(
            f.jpeg.as_ref(),
            &record(1, i as u32 + 1, 640, 424, 32)[24..]
        );
    }
}
#[test]
fn lv1_c13b_repeated_number_dropped() {
    let a = app();
    let o = viewing(&a);
    start(&a);
    a.inject_sidecar_directive("frame 1 5 640 424 8\nframe 1 5 640 424 8\nframe 1 6 640 424 8")
        .unwrap();
    settle(&a, || o.offered().len() >= 2);
    thread::sleep(Duration::from_millis(50));
    assert_eq!(o.offered().len(), 2);
    assert_eq!(
        o.offered()[1].0.jpeg.as_ref(),
        &record(1, 6, 640, 424, 8)[24..]
    );
}
#[test]
fn lv1_c13c_restart_is_new_source() {
    let a = app();
    let o = viewing(&a);
    a.inject_exit(1).unwrap();
    settle(&a, || a.fake_sidecar_recording().pids.len() >= 2);
    settle(&a, || {
        matches!(
            a.camera_status().unwrap(),
            psm_core::domain::camera::CameraStatus::Connected { .. }
        )
    });
    a.inject(
        "liveview.started",
        json!({"epoch":1,"transport":"usb","status":"enabled","frame_channel":"connected"}),
    )
    .unwrap();
    a.inject_sidecar_directive("frames 1 1 2 20").unwrap();
    settle(&a, || o.offered().len() >= 2);
    assert!(o.offered()[1].0.seq > o.offered()[0].0.seq);
}
#[test]
fn lv1_c13d_bad_record_closes_channel() {
    let a = app();
    let o = viewing(&a);
    start(&a);
    a.inject_sidecar_directive("badframe\nframe 1 2 640 424 8")
        .unwrap();
    settle(&a, || {
        a.camera_runtime()
            .helper_log()
            .iter()
            .any(|s| s == "live.stop reason=bad_record")
    });
    assert_eq!(a.live_hub().state(), LiveState::Off);
    assert!(o.offered().is_empty());
}
#[test]
fn lv1_c14a_capture_with_slow_viewer() {
    let a = app();
    let o = viewing(&a);
    start(&a);
    a.inject_sidecar_directive("frames 1 1 200 1").unwrap();
    let id = a.capture().unwrap().0;
    a.inject(
        "capture.exposed",
        json!({"capture_id":id,"transport":"usb","association":"latest_release_provisional"}),
    )
    .unwrap();
    settle(&a, || a.ledger().exposures(None).unwrap().len() == 1);
    assert_eq!(
        stdin(&a)
            .iter()
            .filter(|s| *s == &format!("capture id={id} af=1 cmd_id=")
                || s.starts_with(&format!("capture id={id} af=1")))
            .count(),
        1
    );
    assert_eq!(
        a.ledger().exposures(None).unwrap()[0].capture_id.as_deref(),
        Some(id.as_str())
    );
    let _ = o;
}
#[test]
fn lv1_c14b_capture_without_live() {
    let a = app();
    let id = a.capture().unwrap().0;
    a.inject(
        "capture.exposed",
        json!({"capture_id":id,"transport":"usb","association":"latest_release_provisional"}),
    )
    .unwrap();
    settle(&a, || a.ledger().exposures(None).unwrap().len() == 1);
    assert_eq!(
        stdin(&a)
            .iter()
            .filter(|s| s.starts_with(&format!("capture id={id} af=1")))
            .count(),
        1
    );
}
#[test]
fn lv1_c15a_focus_frame_mapping() {
    let a = app();
    let o = viewing(&a);
    a.inject(
        "liveview.focus_frame",
        json!({"frames":[[1,2,320,640,240,480,38,38]]}),
    )
    .unwrap();
    settle(&a, || {
        o.events()
            .iter()
            .any(|e| matches!(e, LiveEvent::FocusFrames(_)))
    });
    let frames = o
        .events()
        .into_iter()
        .find_map(|e| {
            if let LiveEvent::FocusFrames(f) = e {
                Some(f)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(frames.len(), 1);
    let f = &frames[0];
    assert_eq!((f.cx, f.cy), (0.5, 0.5));
    assert!((f.w - 0.059375).abs() < 1e-6);
    assert!((f.h - 0.0791667).abs() < 1e-6);
    assert_eq!(f.state, FocusFrameState::Focused);
}
#[test]
fn lv1_c15b_exposure_mapping() {
    let a = app();
    let o = viewing(&a);
    a.inject(
        "liveview.exposure",
        json!({"shutter_speed":"1/125","f_number":"8.0","iso":"100"}),
    )
    .unwrap();
    settle(&a, || {
        o.events()
            .iter()
            .any(|e| matches!(e, LiveEvent::Exposure(_)))
    });
    assert!(o.events().contains(&LiveEvent::Exposure(ExposureReadout {
        shutter: "1/125".into(),
        aperture: "f/8".into(),
        iso: "ISO 100".into(),
    })));
}
#[test]
fn lv1_c15c_focus_result_mapping() {
    let a = app();
    let o = viewing(&a);
    let id = focus(a.app.camera.as_ref(), FocusPoint { x: 0.5, y: 0.5 }).unwrap();
    a.inject(
        "focus.result",
        json!({"cmd_id":id.0,"x":320,"y":240,"af":"focused","point_set":true,"total_ms":412}),
    )
    .unwrap();
    settle(&a, || {
        o.events()
            .iter()
            .any(|e| matches!(e, LiveEvent::Focus { .. }))
    });
    assert!(o.events().contains(&LiveEvent::Focus {
        focus_id: id,
        outcome: FocusOutcome::Focused
    }));
}
#[test]
fn lv1_c15d_refused_focus_no_retry() {
    let a = app();
    let o = viewing(&a);
    let id = focus(a.app.camera.as_ref(), FocusPoint { x: 0.5, y: 0.5 }).unwrap();
    a.inject("focus.result",json!({"cmd_id":id.0,"x":320,"y":240,"af":"skipped","point_set":false,"refused":"focus_area_not_movable","total_ms":1})).unwrap();
    settle(&a, || {
        o.events()
            .iter()
            .any(|e| matches!(e, LiveEvent::Focus { .. }))
    });
    assert!(o.events().contains(&LiveEvent::Focus {
        focus_id: id,
        outcome: FocusOutcome::Refused(FocusRefusal::AreaNotMovable)
    }));
    let lines = stdin(&a);
    assert_eq!(lines.iter().filter(|s| s.starts_with("focus ")).count(), 1);
    assert!(lines.last().is_some_and(|s| s.starts_with("focus ")));
}
#[test]
fn lv1_c16a_focus_center_grid() {
    let a = app();
    let id = focus(a.app.camera.as_ref(), FocusPoint { x: 0.5, y: 0.5 }).unwrap();
    settle(&a, || {
        stdin(&a)
            .iter()
            .any(|s| s == &format!("focus x=320 y=240 af=1 cmd_id={}", id.0))
    });
}
#[test]
fn lv1_c16b_focus_corner_grid() {
    let a = app();
    let first = focus(a.app.camera.as_ref(), FocusPoint { x: 0.0, y: 1.0 }).unwrap();
    let second = focus(a.app.camera.as_ref(), FocusPoint { x: 1.0, y: 0.0 }).unwrap();
    settle(&a, || stdin(&a).iter().any(|s| s.contains(&second.0)));
    let lines = stdin(&a);
    assert!(lines.contains(&format!("focus x=0 y=479 af=1 cmd_id={}", first.0)));
    assert!(lines.contains(&format!("focus x=639 y=0 af=1 cmd_id={}", second.0)));
}
#[test]
fn lv1_c16c_clear_focus_center_without_af() {
    let a = app();
    let id = focus_clear(a.app.camera.as_ref()).unwrap();
    settle(&a, || {
        stdin(&a)
            .iter()
            .any(|s| s == &format!("focus x=320 y=240 af=0 cmd_id={}", id.0))
    });
}
#[test]
fn lv1_c16d_disconnected_focus_refused() {
    let a = TestApp::with_camera_script("").unwrap();
    let before = stdin(&a).len();
    assert!(matches!(
        focus(a.app.camera.as_ref(), FocusPoint { x: 0.5, y: 0.5 }),
        Err(Error::CameraNotConnected)
    ));
    assert_eq!(stdin(&a).len(), before);
}
#[test]
fn lv1_c17a_shutdown_quits_and_removes_socket() {
    let a = app();
    let _o = viewing(&a);
    start(&a);
    a.inject_sidecar_directive("frame 1 1 640 424 8").unwrap();
    settle(&a, || {
        matches!(a.live_hub().state(), LiveState::Live { .. })
    });
    let path = a.camera_runtime().frame_socket_path().unwrap();
    assert!(path.exists());
    a.camera_runtime().shutdown();
    settle(&a, || !a.fake_sidecar_recording().exits.is_empty());
    assert!(!path.exists());
    assert!(stdin(&a).last().unwrap() == "quit");
    assert!(
        a.fake_sidecar_recording()
            .exits
            .iter()
            .any(|(_, code)| *code == 0)
    );
}
#[test]
fn lv1_c17b_capabilities() {
    let a = app();
    let direct = a.camera_runtime().direct_adapter();
    let caps = direct.capabilities();
    assert!(caps.live_view && caps.focus_point);
    let co = psm_integrations::camera::via_capture_one::CameraViaCaptureOne {
        scripting: a.fake.clone(),
        document_id: String::new(),
    };
    let caps = co.capabilities();
    assert!(!caps.live_view && !caps.focus_point);
}
#[test]
fn lv1_c17c_unavailable_rejects_later_frame() {
    let a = app();
    let o = viewing(&a);
    a.inject("liveview.unavailable", json!({"reason":"not_supported"}))
        .unwrap();
    settle(&a, || {
        matches!(a.live_hub().state(), LiveState::Unavailable { .. })
    });
    a.inject_sidecar_directive("frame 1 1 640 424 8").unwrap();
    thread::sleep(Duration::from_millis(50));
    assert_eq!(
        a.live_hub().state(),
        LiveState::Unavailable {
            reason: "not_supported".into()
        }
    );
    assert!(o.offered().is_empty());
}

#[test]
fn lv1_parse_record_rejects_bad_length() {
    let mut b = record(1, 2, 640, 424, 8);
    assert_eq!(parse_record(&b).unwrap().jpeg.len(), 8);
    b[20..24].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(parse_record(&b), Err(FrameError::Length));
}
#[test]
fn lv1_grid_and_frame_pure() {
    assert_eq!(to_grid(FocusPoint { x: 0.5, y: 0.5 }), (320, 240));
    assert_eq!(
        frame_norm([1, 2, 320, 640, 240, 480, 38, 38])
            .unwrap()
            .state,
        FocusFrameState::Focused
    );
}
