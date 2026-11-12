//! P60 — the automatic take-back runs cleanly; files deleted outside the app never break the flow;
//! recipes repair themselves before use (book changelog 43, 10F §5). One test per criterion row.
use psm_core::{
    Result,
    domain::camera::{CameraStatus, Link, NeedsActionReason},
    domain::photo::{Integrity, NewPhoto, PhotoSource},
    ports::clock::Clock,
};
use psm_integrations::camera::{
    direct::{CAMERA_SERVICE_STUCK, CAPTURE_ONE_WINDOW_OPEN, TakeBackReport},
    sidecar::protocol::SidecarEvent,
};
use psm_testkit::{TestApp, fake_capture_one::FakeCaptureOneScripting};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::Path,
    thread,
    time::{Duration, Instant},
};

const CLAIMED_WIFI: &str = include_str!("../../src/fixtures/sidecar_scripts/claimed_wifi.txt");
const USB_READY: &str = "{\"timestamp\":\"2026-11-06T10:00:02.000Z\",\"event\":\"connection.ready\",\"transport\":\"usb\",\"model\":\"ILCE-7RM5\",\"store_destination\":\"verified\"}\n";
const ARRIVED: &str = "{\"timestamp\":\"2026-11-06T10:00:00.000Z\",\"event\":\"usb.arrived\",\"detail\":\"camera\"}\n";
const CLAIMED: &str = "{\"timestamp\":\"2026-11-06T10:00:00.100Z\",\"event\":\"usb.claimed_by\",\"owners\":\"pid 512, ptpcamerad\"}\n";

fn settle(a: &TestApp, what: &str, until: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(20);
    while Instant::now() < end {
        a.tick(100).unwrap();
        if until() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!(
        "{what} did not happen: status={:?} ops={:?}",
        a.camera_status(),
        ops(a)
    );
}
fn ops(a: &TestApp) -> Vec<String> {
    a.fake.state.lock().unwrap().ops.clone()
}
fn count(a: &TestApp, op: &str) -> usize {
    ops(a).iter().filter(|o| *o == op).count()
}
fn header(a: &TestApp) -> Option<String> {
    match a.camera_status().unwrap() {
        CameraStatus::NeedsAction {
            reason: NeedsActionReason::HeldByOtherProgram,
            message,
            ..
        } => message,
        _ => None,
    }
}
fn on_cable(a: &TestApp) -> bool {
    matches!(
        a.camera_status(),
        Ok(CameraStatus::Connected {
            link: Link::Cable,
            ..
        })
    )
}
fn health_detail(a: &TestApp, code: &str) -> Option<String> {
    a.app
        .health
        .report("", true, true)
        .items
        .into_iter()
        .find(|i| i.code == code)
        .map(|i| i.detail)
}
/// Capture One running, the camera's cable held on its behalf (the fake quits after one poll and
/// comes back one poll after a launch).
fn capture_one_holds(a: &TestApp) {
    a.fake_processes.set(vec!["Capture One"]);
    let mut f = a.fake.state.lock().unwrap();
    f.quit_after_polls = Some(1);
    f.running_after_polls = Some(1);
}
/// A photo whose RAW (`name`'s bytes) is on disk with its real hash, a preview it still needs, and
/// its variant `v-<name>` in Capture One.
fn photo_needing_preview(a: &TestApp, session_id: i64, name: &str) -> (i64, String) {
    let path = a.dir.path().join(name);
    fs::write(&path, name.as_bytes()).unwrap();
    // Canonical, as every arrival records it.
    let raw = fs::canonicalize(&path)
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let photo = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id,
            raw_path: raw.clone(),
            raw_hash: Some(
                Sha256::digest(name.as_bytes())
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect(),
            ),
            captured_at: Some(a.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: a.clock.now_ms(),
        })
        .unwrap();
    a.store().upsert_raw_state(&raw, None).unwrap();
    a.store().ensure_initial_generation(&raw).unwrap();
    let variant = format!("v-{name}");
    a.fake.add_image(&raw, &[variant.as_str()], true);
    (photo.id, raw)
}
fn processed(a: &TestApp, variant: &str) -> usize {
    ops(a)
        .iter()
        .filter(|o| o.starts_with(&format!("process_started:{variant}:")))
        .count()
}
/// Finishes the render `job` as Capture One would, writing its export with `extension`.
fn finish(a: &TestApp, job: &str, extension: &str) {
    let raw = a.app.editing.render_completion(job).unwrap().raw;
    let mtime = std::time::UNIX_EPOCH + Duration::from_millis(a.clock.now_ms().max(0) as u64);
    let (_, reported) = a
        .fake
        .write_output(
            &raw,
            &a.dir.path().join("Output"),
            extension,
            &FakeCaptureOneScripting::output_bytes(job),
            mtime,
        )
        .unwrap();
    a.deliver_hook(
        &serde_json::json!({"hook":"processing_done","job":job,"raw":raw,"outputs":[reported]})
            .to_string(),
    )
    .unwrap();
}
fn last_job(a: &TestApp) -> String {
    a.fake
        .state
        .lock()
        .unwrap()
        .issued
        .last()
        .unwrap()
        .0
        .clone()
}

struct TakeBackRun {
    a: TestApp,
    result: Result<TakeBackReport>,
    /// Capture One was launched while our USB session was not reported yet.
    launched_early: bool,
    variants: Vec<String>,
}

/// "Take the camera back" while a script is already talking to Capture One, with five previews
/// asked for after Capture One quit (every Apple event reaching a quit Capture One relaunches it,
/// as on the dev Mac). The renders are then completed one by one.
fn take_back_with_queued_renders() -> TakeBackRun {
    let script = format!("{CLAIMED_WIFI}await usb-retry\nsleep 800\n{USB_READY}");
    let a = TestApp::with_camera_script(&script).unwrap();
    capture_one_holds(&a);
    settle(&a, "the claim", || a.app.health.has("camera_held"));
    let (_, session) = a
        .items()
        .create_local("P60-C1", "Take back", "object", None)
        .unwrap();
    let session = session.session.id;
    a.sessions().activate(session).unwrap();
    {
        let mut f = a.fake.state.lock().unwrap();
        f.apple_events_launch = true;
        f.delay_document_info_ms = 400;
        f.ops.clear();
    }
    let take_back = a.take_back();
    let mut launched_early = false;
    let mut raws = Vec::new();
    let result = thread::scope(|scope| {
        let in_flight = scope.spawn(|| a.app.editing.inspect_document());
        let end = Instant::now() + Duration::from_secs(3);
        while !ops(&a).iter().any(|o| o == "document_info_started") {
            assert!(Instant::now() < end, "no script in flight");
            thread::sleep(Duration::from_millis(2));
        }
        let handle = scope.spawn(|| take_back.run());
        let end = Instant::now() + Duration::from_secs(30);
        while !handle.is_finished() {
            assert!(Instant::now() < end, "take-back did not finish");
            let launches = count(&a, "launch");
            if launches > 0 && !on_cable(&a) {
                launched_early = true;
            }
            let quit = ops(&a).iter().any(|o| o == "quit");
            if raws.is_empty() && quit && !a.fake.state.lock().unwrap().running {
                for n in 1..=5 {
                    let (_, raw) = photo_needing_preview(&a, session, &format!("DSC0600{n}.ARW"));
                    a.app.previews.request_initial(&raw).unwrap();
                    raws.push(raw);
                }
            }
            a.tick(100).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
        let _ = in_flight.join().unwrap();
        handle.join().unwrap()
    });
    assert_eq!(raws.len(), 5, "the take-back never got past its quit step");
    let variants: Vec<String> = (1..=5).map(|n| format!("v-DSC0600{n}.ARW")).collect();
    let end = Instant::now() + Duration::from_secs(20);
    while variants.iter().any(|v| processed(&a, v) == 0) {
        assert!(Instant::now() < end, "renders did not run: {:?}", ops(&a));
        a.tick(100).unwrap();
        a.complete_all_renders().unwrap();
        thread::sleep(Duration::from_millis(5));
    }
    TakeBackRun {
        a,
        result,
        launched_early,
        variants,
    }
}

#[test]
fn c1a_nothing_launches_capture_one_during_the_take_back() {
    let run = take_back_with_queued_renders();
    let a = &run.a;
    assert!(
        !run.launched_early,
        "Capture One was launched before our USB session: {:?}",
        ops(a)
    );
    assert_eq!(count(a, "launch"), 1, "{:?}", ops(a));
    let ops = ops(a);
    let quit = ops.iter().position(|o| o == "quit").unwrap();
    let launch = ops.iter().position(|o| o == "launch").unwrap();
    assert!(quit < launch);
    for variant in &run.variants {
        let first = ops
            .iter()
            .position(|o| o.starts_with(&format!("process_started:{variant}:")))
            .unwrap();
        assert!(first > launch, "{variant} rendered before the relaunch");
    }
}

#[test]
fn c1b_take_back_reports_success_never_did_not_quit() {
    let run = take_back_with_queued_renders();
    let a = &run.a;
    assert_eq!(
        run.result.unwrap(),
        TakeBackReport {
            completed_steps: 5,
            camera_held: true
        }
    );
    assert!(on_cable(a));
    let status = format!("{:?}", a.camera_status().unwrap());
    let health = format!("{:?}", a.app.health.report("", true, true).items);
    assert!(!status.contains("did not quit"), "{status}");
    assert!(!health.contains("did not quit"), "{health}");
}

#[test]
fn c2a_open_dialog_is_named_and_the_same_plug_in_is_retried() {
    let script = format!("{ARRIVED}{CLAIMED}await usb-retry\n{USB_READY}sleep 100000\n");
    let a = TestApp::with_camera_script(&script).unwrap();
    capture_one_holds(&a);
    a.fake.state.lock().unwrap().quit_after_polls = None; // a sign-in window is open
    settle(&a, "the dialog sentence", || {
        header(&a).as_deref() == Some(CAPTURE_ONE_WINDOW_OPEN)
    });
    assert_eq!(count(&a, "launch"), 0);
    assert!(a.fake.state.lock().unwrap().running);
    // The photographer closes the window: Capture One answers again.
    a.fake.state.lock().unwrap().quit_after_polls = Some(1);
    settle(&a, "the take-back", || {
        on_cable(&a) && count(&a, "launch") == 1
    });
    assert!(count(&a, "quit") >= 2);
    let plug_ins = a
        .sidecar_events()
        .iter()
        .filter(|e| matches!(e.event, SidecarEvent::UsbArrived { .. }))
        .count();
    assert_eq!(plug_ins, 1);
}

#[test]
fn c3a_stuck_camera_service_is_named() {
    let script = format!("{ARRIVED}{CLAIMED}await usb-retry\n{CLAIMED}sleep 100000\n");
    let a = TestApp::with_camera_script(&script).unwrap();
    capture_one_holds(&a);
    settle(&a, "the quit", || {
        count(&a, "quit") == 1 && !a.fake.state.lock().unwrap().running
    });
    a.fake_processes.set(vec![]); // Capture One is gone; nothing else uses cameras
    let quit_at = a.clock.now_ms();
    settle(&a, "the stuck-service sentence", || {
        header(&a).as_deref() == Some(CAMERA_SERVICE_STUCK)
    });
    // Measured from when the test saw the quit (up to one tick after it).
    assert!(a.clock.now_ms() - quit_at >= 9_900);
    settle(&a, "the relaunch", || count(&a, "launch") == 1);
    assert_eq!(header(&a).as_deref(), Some(CAMERA_SERVICE_STUCK));
}

#[test]
fn c4a_deleted_file_heals_and_returns() {
    let mut a = TestApp::with_fake_document().unwrap();
    let (_, session) = a
        .items()
        .create_local("P60-C4", "Deleted", "object", None)
        .unwrap();
    let session = session.session.id;
    let (gone, gone_raw) = photo_needing_preview(&a, session, "DSC06101.ARW");
    photo_needing_preview(&a, session, "DSC06102.ARW");
    photo_needing_preview(&a, session, "DSC06103.ARW");
    let in_view = |a: &TestApp| -> Vec<i64> {
        a.sessions()
            .view(session)
            .unwrap()
            .photos
            .iter()
            .map(|p| p.id)
            .collect()
    };
    let asked_for_gone = |a: &TestApp| {
        ops(a)
            .iter()
            .any(|o| o.contains("DSC06101") || o.contains("v-DSC06101"))
    };
    // Deleted by hand while the app was closed; the app starts.
    fs::remove_file(&gone_raw).unwrap();
    a.kill_and_restart().unwrap();
    for _ in 0..20 {
        a.tick(1_000).unwrap();
        a.complete_all_renders().unwrap();
    }
    a.focus().unwrap();
    assert_eq!(in_view(&a).len(), 2);
    assert!(!in_view(&a).contains(&gone));
    assert_eq!(
        health_detail(&a, "file_missing").as_deref(),
        Some("1 photos' files were deleted outside Session Manager")
    );
    assert!(!asked_for_gone(&a), "{:?}", ops(&a));
    // The file comes back at the same path: the photo returns as it was.
    fs::write(&gone_raw, b"DSC06101.ARW").unwrap();
    a.focus().unwrap();
    assert_eq!(in_view(&a).len(), 3);
    assert_eq!(a.store().photo(gone).unwrap().integrity, Integrity::Ok);
    assert_eq!(health_detail(&a, "file_missing"), None);

    // Separately, while the app runs: Capture One no longer knows the photo (it would answer
    // `variant_not_found`) and its RAW is deleted. It is found missing without being asked for.
    let (lost, lost_raw) = photo_needing_preview(&a, session, "DSC06104.ARW");
    a.fake.state.lock().unwrap().folder_images.remove(&lost_raw);
    fs::remove_file(&lost_raw).unwrap();
    for _ in 0..20 {
        a.tick(1_000).unwrap();
    }
    assert!(!in_view(&a).contains(&lost));
    assert_eq!(
        health_detail(&a, "file_missing").as_deref(),
        Some("1 photos' files were deleted outside Session Manager")
    );
    assert!(
        !ops(&a).iter().any(|o| o.contains("DSC06104")),
        "{:?}",
        ops(&a)
    );
}

#[test]
fn c4b_missing_files_never_block_the_take_back() {
    let script = format!("{ARRIVED}{CLAIMED}await usb-retry\n{USB_READY}sleep 100000\n");
    let a = TestApp::with_camera_script(&script).unwrap();
    a.camera_runtime().set_auto_take_back(false);
    capture_one_holds(&a);
    settle(&a, "the claim", || a.app.health.has("camera_held"));
    let (_, session) = a
        .items()
        .create_local("P60-C4B", "Storage", "object", None)
        .unwrap();
    let session = session.session.id;
    let mut raws = Vec::new();
    for n in 0..50 {
        let (_, raw) = photo_needing_preview(&a, session, &format!("DSC07{n:03}.ARW"));
        a.app.previews.request_initial(&raw).unwrap();
        raws.push(raw);
    }
    assert!(a.app.editing.render_in_flight(&raws[0]));
    for raw in &raws {
        fs::remove_file(raw).unwrap();
    }
    a.camera_runtime().set_auto_take_back(true);
    let end = Instant::now() + Duration::from_secs(20);
    while !(on_cable(&a) && count(&a, "launch") == 1) {
        assert!(
            Instant::now() < end,
            "take-back did not run: {:?}",
            header(&a)
        );
        let shown = format!("{:?}", a.camera_status().unwrap());
        assert!(!shown.contains("a preview is rendering"), "{shown}");
        a.tick(100).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(count(&a, "quit"), 1);
}

#[test]
fn c5a_no_render_storm_when_capture_one_reopens() {
    let a = TestApp::with_fake_document().unwrap();
    let (_, session) = a
        .items()
        .create_local("P60-C5", "Reopen", "object", None)
        .unwrap();
    let session = session.session.id;
    for n in 0..100 {
        let (_, raw) = photo_needing_preview(&a, session, &format!("DSC08{n:03}.ARW"));
        let generation = a.store().preview_state(&raw).unwrap().unwrap().adjusted_gen;
        let preview = a.dir.path().join(format!("preview-{n}.jpg"));
        fs::write(&preview, b"jpeg").unwrap();
        a.store()
            .complete_preview(&raw, generation, &preview.to_string_lossy(), None, None)
            .unwrap();
    }
    a.tick(0).unwrap();
    // Capture One quits and is opened again.
    a.fake.state.lock().unwrap().running = false;
    a.tick(1_000).unwrap();
    {
        let mut f = a.fake.state.lock().unwrap();
        f.running = true;
        f.ops.clear();
    }
    for _ in 0..5 {
        a.tick(1_000).unwrap();
    }
    a.focus().unwrap();
    // A new photo arrives.
    let raw = a
        .dir
        .path()
        .join("Capture One Session/Capture/DSC08999.ARW");
    fs::create_dir_all(raw.parent().unwrap()).unwrap();
    fs::write(&raw, b"new").unwrap();
    a.deliver_hook(&serde_json::json!({"hook":"capture_done","argv":[raw]}).to_string())
        .unwrap();
    let arrived = a.store().photos_in_session(1).unwrap().remove(0);
    a.fake.add_image(&arrived.raw_path, &["v-new"], false);
    for _ in 0..5 {
        a.tick(1_000).unwrap();
    }
    let sent: Vec<String> = ops(&a)
        .into_iter()
        .filter(|o| o.starts_with("process_started:"))
        .collect();
    assert_eq!(sent, vec!["process_started:v-new:PSM Preview".to_string()]);
}

#[test]
fn c6a_recipes_repair_themselves_once_without_a_deferral_loop() {
    let a = TestApp::with_fake_document().unwrap();
    let (_, session) = a
        .items()
        .create_local("P60-C6", "Recipes", "object", None)
        .unwrap();
    let session = session.session.id;
    let (_, first) = photo_needing_preview(&a, session, "DSC09001.ARW");
    let (removed, removed_raw) = photo_needing_preview(&a, session, "DSC09002.ARW");
    a.app.previews.request_initial(&first).unwrap();
    a.app.previews.request_initial(&removed_raw).unwrap();
    assert_eq!(
        processed(&a, "v-DSC09002.ARW"),
        0,
        "queued behind the first"
    );
    // `PSM Preview` is deleted in Capture One in the middle of the batch.
    a.fake.state.lock().unwrap().recipes.remove("PSM Preview");
    a.fake.state.lock().unwrap().ops.clear();
    finish(&a, &last_job(&a), "jpg");
    assert_eq!(count(&a, "make_recipe:PSM Preview"), 1);
    assert_eq!(processed(&a, "v-DSC09002.ARW"), 2, "{:?}", ops(&a));
    finish(&a, &last_job(&a), "jpg");
    assert!(
        a.store()
            .photo(removed)
            .unwrap()
            .adjusted_preview_path
            .is_some()
    );

    // `PSM Preview` is switched to TIFF while a render is out: its TIFF is not a preview.
    let (tiff, tiff_raw) = photo_needing_preview(&a, session, "DSC09003.ARW");
    a.app.previews.request_initial(&tiff_raw).unwrap();
    a.fake
        .state
        .lock()
        .unwrap()
        .recipes
        .get_mut("PSM Preview")
        .unwrap()
        .output_format = "TIFF".into();
    a.fake.state.lock().unwrap().ops.clear();
    finish(&a, &last_job(&a), "tif");
    assert_eq!(a.store().photo(tiff).unwrap().adjusted_preview_path, None);
    let repairs: Vec<String> = ops(&a)
        .into_iter()
        .filter(|o| o.starts_with("set_recipe_props:PSM Preview:"))
        .collect();
    assert_eq!(repairs.len(), 1, "{:?}", ops(&a));
    assert!(repairs[0].contains("outputFormat"));
    assert_eq!(processed(&a, "v-DSC09003.ARW"), 1);
    let retry = last_job(&a);
    finish(&a, &retry, "jpg");
    let path = a
        .store()
        .photo(tiff)
        .unwrap()
        .adjusted_preview_path
        .unwrap();
    assert_eq!(
        fs::read(Path::new(&path)).unwrap(),
        FakeCaptureOneScripting::output_bytes(&retry)
    );
    for _ in 0..30 {
        a.tick(1_000).unwrap();
    }
    assert_eq!(processed(&a, "v-DSC09003.ARW"), 1);
    assert_eq!(processed(&a, "v-DSC09002.ARW"), 0);
    assert_eq!(health_detail(&a, "recipe_missing"), None);
}
