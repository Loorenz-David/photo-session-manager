use futures_util::StreamExt;
use psm_core::domain::live::{ExposureReadout, LiveEvent};
use psm_integrations::camera::{
    direct::live::exposure_readout,
    sidecar::protocol::{SidecarEvent, parse_line},
};
use psm_testkit::{TestApp, live_outlet::RecordingOutlet, phone::TestPhone};
use serde_json::{Value, json};
use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

const READY: &str = r#"{"timestamp":"2026-11-06T10:00:00.000Z","source":"sdk","event":"connection.ready","transport":"usb","model":"ILCE-7RM5","store_destination":"verified","store_value":"0x0003"}"#;
const REAL: &str = r#"{"timestamp":"2026-11-26T14:06:58.210Z","experiment":"adhoc","source":"sdk","event":"liveview.exposure","shutter_speed":{"code":"0x00000103","value":65616,"value_hex":"0x00010050","display":"1/80","writable":false,"readable":true},"f_number":{"code":"0x00000100","value":800,"value_hex":"0x00000320","display":"F8.0","writable":false,"readable":true},"iso":{"code":"0x00000104","value":2000,"value_hex":"0x000007D0","display":"ISO 2000","writable":false,"readable":true}}"#;
type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
}
fn app() -> TestApp {
    let a = TestApp::with_camera_script(READY).unwrap();
    settle(&a, || {
        a.camera_status()
            .is_ok_and(|s| matches!(s, psm_core::domain::camera::CameraStatus::Connected { .. }))
    });
    a
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
    panic!("sidecar event not delivered: {:?}", a.sidecar_events());
}
async fn next_kind(ws: &mut Ws, kind: &str) -> Value {
    let end = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        match tokio::time::timeout_at(end, ws.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                let v: Value = serde_json::from_str(text.as_str()).unwrap();
                if v["type"] == kind {
                    return v;
                }
            }
            Ok(Some(Ok(_))) => {}
            other => panic!("no {kind}: {other:?}"),
        }
    }
}
async fn connect(phone: &TestPhone, token: &str) -> Ws {
    connect_async(phone.ws_url(&format!("/api/phone/v1/ws?pair={token}")))
        .await
        .unwrap()
        .0
}

#[test]
fn lv5_c12a_the_real_helper_exposure_line_parses_its_displays() {
    let event = parse_line(REAL).unwrap().event;
    match event {
        SidecarEvent::LiveviewExposure {
            shutter_speed,
            f_number,
            iso,
        } => assert_eq!(
            (shutter_speed.as_str(), f_number.as_str(), iso.as_str()),
            ("1/80", "F8.0", "ISO 2000")
        ),
        other => panic!("{other:?}"),
    }
}
#[test]
fn lv5_c12b_real_exposure_reaches_the_hub_and_phone() {
    let a = app();
    let o = Arc::new(RecordingOutlet::default());
    a.live_hub().subscribe(o.clone());
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    rt().block_on(async{
        let mut ws=connect(&phone,&token).await;next_kind(&mut ws,"hello").await;
        a.inject("liveview.exposure",json!({
            "shutter_speed":{"code":"0x00000103","value":65616,"value_hex":"0x00010050","display":"1/80","writable":false,"readable":true},
            "f_number":{"code":"0x00000100","value":800,"value_hex":"0x00000320","display":"F8.0","writable":false,"readable":true},
            "iso":{"code":"0x00000104","value":2000,"value_hex":"0x000007D0","display":"ISO 2000","writable":false,"readable":true}
        })).unwrap();
        settle(&a,||o.events().iter().any(|e|matches!(e,LiveEvent::Exposure(_))));
        assert!(o.events().contains(&LiveEvent::Exposure(ExposureReadout{shutter:"1/80".into(),aperture:"f/8".into(),iso:"ISO 2000".into()})));
        loop {let m=next_kind(&mut ws,"camera").await;if m["exposure"].is_object(){assert_eq!(m["exposure"],json!({"shutter":"1/80","aperture":"f/8","iso":"ISO 2000"}));break;}}
    });
}
#[test]
fn lv5_c12d_an_empty_shutter_display_maps_to_a_dash() {
    let line = r#"{"timestamp":"2026-11-26T14:06:58.210Z","source":"sdk","event":"liveview.exposure","shutter_speed":"","f_number":{"display":"F8.0"},"iso":{"display":"ISO 100"}}"#;
    let SidecarEvent::LiveviewExposure {
        shutter_speed,
        f_number,
        iso,
    } = parse_line(line).unwrap().event
    else {
        panic!("not exposure")
    };
    let r = exposure_readout(&shutter_speed, &f_number, &iso);
    assert_eq!(
        (r.shutter.as_str(), r.aperture.as_str(), r.iso.as_str()),
        ("–", "f/8", "ISO 100")
    );
}
#[test]
fn lv5_c12e_a_numeric_iso_is_unparsable_and_never_reaches_the_hub() {
    let line = r#"{"timestamp":"2026-11-26T14:06:58.210Z","source":"sdk","event":"liveview.exposure","shutter_speed":"1/80","f_number":{"display":"F8.0"},"iso":2000}"#;
    assert!(parse_line(line).is_err());
    let a = app();
    let o = Arc::new(RecordingOutlet::default());
    a.live_hub().subscribe(o.clone());
    a.inject(
        "liveview.exposure",
        json!({"shutter_speed":"1/80","f_number":{"display":"F8.0"},"iso":2000}),
    )
    .unwrap();
    settle(&a, || {
        a.camera_runtime()
            .helper_log()
            .iter()
            .any(|s| s.contains("unparsable"))
    });
    assert!(
        !o.events()
            .iter()
            .any(|e| matches!(e, LiveEvent::Exposure(_)))
    );
}
#[test]
fn lv5_c12f_a_nonmovable_focus_result_reaches_the_phone_as_refused() {
    let a = app();
    let s = a
        .items()
        .create_local("LV5-C12F", "LV5-C12F", "Table", None)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(s).unwrap();
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    rt().block_on(async{
        let mut ws=connect(&phone,&token).await;next_kind(&mut ws,"hello").await;
        let answer=reqwest::Client::new().post(phone.url("/api/phone/v1/focus")).bearer_auth(&token)
            .json(&json!({"session_id":s,"x":0.5,"y":0.5})).send().await.unwrap();
        assert_eq!(answer.status(),reqwest::StatusCode::OK);let id=answer.json::<Value>().await.unwrap()["focus_id"].as_str().unwrap().to_owned();
        a.inject("focus.result",json!({"cmd_id":id,"x":320,"y":240,"af":"skipped","point_set":false,"refused":"focus_area_not_movable","point_ms":0,"af_ms":0,"total_ms":3})).unwrap();
        settle(&a,||a.sidecar_events().iter().any(|e|matches!(e.event,SidecarEvent::FocusResult{..})));
        let m=next_kind(&mut ws,"focus").await;
        assert_eq!(m,json!({"type":"focus","focus_id":id,"outcome":"refused","reason":"focus_area_not_movable"}));
    });
}
