//! P69 faster export and publish (owner 2026-11-11, changelog 47): 8-bit masters, masters
//! prepared in the background once a session is Ready, a batch publish that lets Capture One
//! render the next item's masters while the current item uploads, and one `publish.timing` line
//! per published item. One test per criterion row of `plans/P69_faster_export_and_publish.md`,
//! at the service, the fake Capture One's recorded renders and recipes, the fake shop, the files
//! and the log.

use psm_core::{
    domain::{
        composition::{CompositionKind, CompositionValues, Flip, Orientation, Rect, Shape},
        photo::{Classification as C, NewPhoto, Photo, PhotoSource},
        session::{ListFilter, PublishStep, SessionStatus},
    },
    events::AppEvent,
    mechanisms::hash::sha256_file,
    ports::{commerce::CommerceAdapter, images::uid12},
    services::{prepare_masters::PrepareOutcome, publish::PublishService},
};
use psm_testkit::{
    TestApp,
    fixtures::{
        synthetic_jpeg::SyntheticJpeg,
        synthetic_master::{PAPER, SyntheticMaster},
    },
};
use std::{
    collections::{BTreeSet, HashMap},
    fs,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const NOW_MS: i64 = 1_762_250_400_000;
const GB: u64 = 1_000_000_000;

/// The tests of this file run one at a time: two of them read the `publish.timing` line through
/// a thread-scoped log subscriber, which misses lines whose log site another test thread is
/// hitting at the same moment without one.
static SERIAL: Mutex<()> = Mutex::new(());
fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

// ---------- logs ----------

#[derive(Clone)]
struct LogBytes(Arc<Mutex<Vec<u8>>>);
impl Write for LogBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
/// Runs `f` with the log lines it writes collected; returns its result and the log text.
fn logged<T>(f: impl FnOnce() -> T) -> (T, String) {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let output = LogBytes(buffer.clone());
    // P70: keeps every log site wanted while this thread-scoped capture runs (one shared process).
    psm_testkit::logs::keep_log_sites_interested();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || output.clone())
        .without_time()
        .with_ansi(false)
        .finish();
    let value = tracing::subscriber::with_default(subscriber, f);
    let text = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
    (value, text)
}

// ---------- fixtures ----------

fn shop_app() -> TestApp {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(NOW_MS);
    a
}
fn session(a: &TestApp, sku: &str) -> i64 {
    a.items()
        .create_local(sku, "Oak", "Sideboard", None)
        .unwrap()
        .1
        .session
        .id
}
fn last_issued(a: &TestApp) -> String {
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
/// The variant id the fake document gives a photo.
fn variant(p: &Photo) -> String {
    format!("v{}", p.id)
}
/// A RAW known to the fake document; its adjusted preview is rendered when `preview` is true.
fn photo_with(a: &TestApp, s: i64, name: &str, class: C, preview: bool) -> Photo {
    let dir = a.dir.path().join("Capture/raws");
    fs::create_dir_all(&dir).unwrap();
    let raw = dir.join(format!("{name}.ARW"));
    fs::write(&raw, format!("raw bytes of {name}")).unwrap();
    let photo = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id: s,
            raw_path: raw.to_string_lossy().into_owned(),
            raw_hash: Some(sha256_file(&raw).unwrap()),
            captured_at: Some(NOW_MS),
            source: PhotoSource::Capture,
            created_at: NOW_MS,
        })
        .unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&variant(&photo)], false);
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    if preview {
        a.app.previews.request_initial(&photo.raw_path).unwrap();
        a.complete_render_with(
            &last_issued(a),
            &SyntheticJpeg::rgb(2560, 1707, [200, 100, 50]),
        )
        .unwrap();
    }
    if class != C::Unreviewed {
        a.review().classify(photo.id, class).unwrap();
    }
    a.store().photo(photo.id).unwrap()
}
fn photo(a: &TestApp, s: i64, name: &str) -> Photo {
    photo_with(a, s, name, C::Main, true)
}
fn service(a: &TestApp) -> Arc<PublishService> {
    a.app.publish.clone().unwrap()
}
fn status(a: &TestApp, s: i64) -> SessionStatus {
    a.sessions()
        .list(ListFilter::All, None)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .find(|r| r.id == s)
        .unwrap()
        .status
}
/// The variants of every master render the fake Capture One was asked for, in order.
fn master_variants(a: &TestApp) -> Vec<String> {
    a.fake
        .state
        .lock()
        .unwrap()
        .issued
        .iter()
        .filter(|(_, _, recipe)| recipe == "PSM Master")
        .map(|(_, variant, _)| variant.clone())
        .collect()
}
fn prepare(a: &TestApp) -> PrepareOutcome {
    a.app.prepare_masters.tick().unwrap()
}
fn paused(reason: &str) -> PrepareOutcome {
    PrepareOutcome {
        requested: None,
        paused: Some(reason.into()),
    }
}
/// Prepares masters one at a time until the job finds nothing more to do; the photos asked for.
fn prepare_all(a: &TestApp) -> Vec<i64> {
    let mut asked = Vec::new();
    for _ in 0..20 {
        let outcome = prepare(a);
        let Some(id) = outcome.requested else {
            assert_eq!(
                outcome,
                PrepareOutcome::default(),
                "nothing left to prepare"
            );
            return asked;
        };
        asked.push(id);
        // One at a time: nothing more is asked while that master is with Capture One.
        assert_eq!(prepare(a), paused("preparing"));
        a.complete_all_renders().unwrap();
    }
    panic!("preparing did not end: {asked:?}");
}
/// The master files in the app's master cache of a photo.
fn masters_of(a: &TestApp, p: &Photo) -> Vec<String> {
    fs::read_dir(a.masters_dir(&p.raw_path))
        .map(|dir| {
            dir.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}
fn drain(a: &TestApp) {
    for _ in 0..60 {
        a.tick(1100).unwrap();
        a.complete_all_renders().unwrap();
        if a.store().active_publish_items().unwrap().is_empty() && !a.app.export.running() {
            a.tick(1100).unwrap();
            return;
        }
    }
}
fn publish(a: &TestApp, sessions: &[i64]) {
    service(a).confirm(sessions, "Publish images").unwrap();
    drain(a);
}
fn step(a: &TestApp, s: i64) -> PublishStep {
    let item = a.store().latest_publish(s).unwrap().unwrap().item_id;
    a.store().publish_item(item).unwrap().step
}
fn upload_name(p: &Photo, key: &str) -> String {
    format!("{key}-{}.jpg", uid12(&p.uid))
}
fn gallery(a: &TestApp, sku: &str) -> Vec<String> {
    let product = a
        .fake_shop()
        .state
        .lock()
        .unwrap()
        .products
        .iter()
        .find(|p| p.sku.as_deref() == Some(sku))
        .unwrap()
        .external_id
        .clone();
    a.fake_shop()
        .item_get(&product)
        .unwrap()
        .media
        .into_iter()
        .map(|m| m.filename.unwrap_or_default())
        .collect()
}
/// The one `publish.timing` line of the log, as its `key=value` fields.
fn timing_fields(log: &str) -> HashMap<String, String> {
    let lines: Vec<&str> = log
        .lines()
        .filter(|l| l.contains("publish.timing"))
        .collect();
    assert_eq!(lines.len(), 1, "one timing line: {log}");
    assert!(lines[0].contains("INFO"), "{}", lines[0]);
    let text = &lines[0][lines[0].find("publish.timing").unwrap()..];
    text.split_whitespace()
        .skip(1)
        .filter_map(|pair| pair.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}
fn disk_free(a: &TestApp, bytes: u64) {
    a.app.disk_space().set(Arc::new(move |_: &Path| {
        Ok(psm_infra::disk::FreeSpace {
            available_bytes: bytes,
            disk: "the test disk".into(),
        })
    }));
}
fn original() -> CompositionValues {
    CompositionValues {
        kind: CompositionKind::Normal,
        shape: Shape::Original,
        orientation: Orientation::Named,
        quarter_turns: 0,
        flip: Flip::None,
        straighten: 0.0,
        rect: Rect {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
        },
    }
}
fn render_output_jobs(a: &TestApp) -> i64 {
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE kind='render_output'",
            [],
            |r| r.get(0),
        )
        .unwrap()
}

// ---------- C1(a) 8-bit master ----------

#[test]
fn c1a_the_master_is_an_8_bit_tiff_and_the_output_is_made_once_from_it() {
    let _serial = serial();
    let a = TestApp::with_fake_document().unwrap();
    a.clock.advance(NOW_MS);
    // The `PSM Master` recipe as earlier versions provisioned it: a 16-bit TIFF.
    a.fake
        .state
        .lock()
        .unwrap()
        .recipes
        .get_mut("PSM Master")
        .unwrap()
        .bits = 16;
    let s = session(&a, "K");
    let p = photo(&a, s, "a");
    a.app.export.run(s).unwrap();

    let recipe = a.fake.state.lock().unwrap().recipes["PSM Master"].clone();
    assert_eq!(
        (
            recipe.output_format.as_str(),
            recipe.bits,
            recipe.tiff_compression.as_str()
        ),
        ("TIFF", 8, "Uncompressed"),
        "the recipe is repaired before the master render"
    );
    let job = last_issued(&a);
    assert!(a.is_master_job(&job));
    assert_eq!(master_variants(&a), vec![variant(&p)]);

    // Capture One writes the 8-bit master; the app renders the output from it.
    a.complete_master_render_bytes(&job, &SyntheticMaster::solid(600, 400, PAPER).bytes_8bit())
        .unwrap();
    for _ in 0..3 {
        a.tick(1100).unwrap();
    }
    let output = a.export_root().join("K").join("K_01_main.jpg");
    let img = a.decode(&output).unwrap();
    assert_eq!(img.dimensions(), (400, 400), "the default 1:1 composition");
    let px = img.get_pixel(200, 200);
    assert!(
        px.0.iter().all(|v| v.abs_diff(0xC0) <= 2),
        "the master's colour: {px:?}"
    );
    assert_eq!(render_output_jobs(&a), 1, "the output is rendered once");
    let jpegs: Vec<_> = fs::read_dir(a.export_root().join("K"))
        .unwrap()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "jpg"))
        .collect();
    assert_eq!(jpegs.len(), 1);
}

// ---------- C2(a) prepared when Ready ----------

#[test]
fn c2a_masters_are_prepared_one_at_a_time_and_the_publish_renders_none() {
    let _serial = serial();
    let a = shop_app();
    let s = session(&a, "K");
    let photos: Vec<Photo> = ["a", "b", "c"].iter().map(|n| photo(&a, s, n)).collect();
    assert_eq!(status(&a, s), SessionStatus::Ready);

    let asked = prepare_all(&a);
    assert_eq!(asked, photos.iter().map(|p| p.id).collect::<Vec<_>>());
    assert_eq!(
        master_variants(&a),
        photos.iter().map(variant).collect::<Vec<_>>()
    );
    for p in &photos {
        assert_eq!(masters_of(&a, p), vec!["master-1.tif".to_string()]);
    }

    let before = master_variants(&a).len();
    let ((), log) = logged(|| publish(&a, &[s]));
    assert_eq!(step(&a, s), PublishStep::Done);
    assert_eq!(
        master_variants(&a).len(),
        before,
        "the publish asked for no master"
    );
    assert_eq!(timing_fields(&log)["masters_reused"], "3/3");
    assert_eq!(
        gallery(&a, "K"),
        photos
            .iter()
            .map(|p| upload_name(p, "K"))
            .collect::<Vec<_>>()
    );
    // MC-28: masters go after the publish, prepared ones included.
    assert!(photos.iter().all(|p| masters_of(&a, p).is_empty()));
}

// ---------- C2(b) stays out of the way ----------

fn settle(what: &str, done: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(10);
    while !done() {
        assert!(Instant::now() < end, "{what} did not happen");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn c2b_nothing_is_asked_during_a_preview_render_another_publish_or_with_the_setting_off() {
    let _serial = serial();
    let mut a = shop_app();
    let s = session(&a, "K");
    let p = photo(&a, s, "a");

    // A preview render is with Capture One (another session's new photo).
    let other = session(&a, "L");
    let q = photo_with(&a, other, "q", C::Main, false);
    a.app.previews.request_initial(&q.raw_path).unwrap();
    assert_eq!(prepare(&a), paused("render"));
    assert!(master_variants(&a).is_empty());
    a.complete_render_with(
        &last_issued(&a),
        &SyntheticJpeg::rgb(2560, 1707, [200, 100, 50]),
    )
    .unwrap();

    // Another session publishes.
    service(&a).confirm(&[other], "Publish images").unwrap();
    assert_eq!(prepare(&a), paused("publish"));
    drain(&a);
    assert_eq!(step(&a, other), PublishStep::Done);
    assert!(
        !master_variants(&a).contains(&variant(&p)),
        "K's master was not asked for during L's publish"
    );

    // The setting off: nothing is prepared, even with Capture One idle.
    a.settings_set(&serde_json::json!({ "export": { "prepare_masters": false } }))
        .unwrap();
    assert_eq!(prepare(&a), paused("off"));
    assert!(!master_variants(&a).contains(&variant(&p)));
    let saved = psm_infra::settings_file::Settings::open(&a.app.paths).unwrap();
    assert!(!saved.export.prepare_masters);

    // On again, idle: K's master is asked for.
    a.settings_set(&serde_json::json!({ "export": { "prepare_masters": true } }))
        .unwrap();
    assert_eq!(prepare(&a).requested, Some(p.id));
    assert_eq!(master_variants(&a).last(), Some(&variant(&p)));
}

#[test]
fn c2b_nothing_is_asked_while_a_capture_is_on_its_way() {
    let _serial = serial();
    let a = TestApp::with_camera().unwrap();
    a.clock.advance(NOW_MS);
    let s = session(&a, "K");
    let p = photo(&a, s, "a");
    settle("the receiver", || {
        a.app.camera_runtime.receiver_port().is_some()
    });
    let mut ftp = a.ftp();
    let (reply, data) = ftp.open_stor("DSC07101.ARW").unwrap();
    assert!(reply.starts_with("150"), "{reply}");
    let data = data.unwrap();
    settle("the upload under way", || {
        a.app.camera_runtime.uploads_in_progress() > 0
    });
    assert_eq!(prepare(&a), paused("capture"));
    assert!(master_variants(&a).is_empty());
    drop(data);
    drop(ftp);
    settle("the upload ended", || {
        a.app.camera_runtime.uploads_in_progress() == 0
    });
    assert_eq!(prepare(&a).requested, Some(p.id));
}

// ---------- C2(c) stale and cleared ----------

/// Publishes and returns the variants of the masters Capture One was asked for meanwhile.
fn publish_recording(a: &TestApp, sessions: &[i64]) -> Vec<String> {
    let before = master_variants(a).len();
    publish(a, sessions);
    master_variants(a)[before..].to_vec()
}

fn manifest(a: &TestApp, key: &str) -> serde_json::Value {
    serde_json::from_slice(&fs::read(a.export_root().join(key).join("manifest.json")).unwrap())
        .unwrap()
}

#[test]
fn c2c_a_stale_master_is_rendered_again_and_cleared_ones_are_made_again() {
    let _serial = serial();
    // Stale: after preparing, photo a is edited in Capture One and photo b is recomposed.
    let a = shop_app();
    let s = session(&a, "K");
    let photos: Vec<Photo> = ["a", "b", "c"].iter().map(|n| photo(&a, s, n)).collect();
    assert_eq!(prepare_all(&a).len(), 3);
    a.clock.advance(31_000);
    a.deliver_hook(
        &serde_json::json!({"hook":"variant_adjusted","variant_id":variant(&photos[0]),"raw":photos[0].raw_path})
            .to_string(),
    )
    .unwrap();
    a.tick(20_000).unwrap();
    a.complete_render_with(
        &last_issued(&a),
        &SyntheticJpeg::rgb(2560, 1707, [200, 100, 50]),
    )
    .unwrap();
    assert_eq!(a.store().adjusted_gen(&photos[0].raw_path).unwrap(), 2);
    a.app.compositions.set(photos[1].id, original()).unwrap();

    let asked = publish_recording(&a, &[s]);
    assert_eq!(step(&a, s), PublishStep::Done);
    assert_eq!(asked, vec![variant(&photos[0])], "only the stale master");
    let outputs = manifest(&a, "K")["outputs"].clone();
    for (index, p) in photos.iter().enumerate() {
        let entry = &outputs[format!("K_{:02}_main.jpg", index + 1)];
        assert_eq!(
            entry["adjusted_gen"],
            a.store().adjusted_gen(&p.raw_path).unwrap(),
            "{index}"
        );
        assert_eq!(
            entry["composition_rev"],
            a.composition(p.id).unwrap().revision,
            "{index}"
        );
    }
    // Never a wrong frame: the recomposed photo is its whole frame, the others square.
    let dims = |n: usize| {
        a.decode(&a.export_root().join("K").join(format!("K_{n:02}_main.jpg")))
            .unwrap()
            .dimensions()
    };
    assert_eq!(
        (dims(1), dims(2), dims(3)),
        ((400, 400), (600, 400), (400, 400))
    );

    // Cleared: the storage budget clears prepared masters; the publish makes them again.
    let b = shop_app();
    let t = session(&b, "M");
    let kept: Vec<Photo> = ["d", "e"].iter().map(|n| photo(&b, t, n)).collect();
    assert_eq!(prepare_all(&b).len(), 2);
    // 100 MB short of the 20 GB default minimum: more than every master here.
    disk_free(&b, 20 * GB - 100_000_000);
    assert_eq!(
        prepare(&b),
        paused("disk"),
        "not made again below the minimum"
    );
    for _ in 0..10 {
        b.clock.advance(3_000);
        b.app.storage.tick().unwrap();
    }
    assert!(kept.iter().all(|p| masters_of(&b, p).is_empty()));
    disk_free(&b, 1 << 50);
    let asked = publish_recording(&b, &[t]);
    assert_eq!(step(&b, t), PublishStep::Done);
    assert_eq!(asked, kept.iter().map(variant).collect::<Vec<_>>());
    assert_eq!(
        gallery(&b, "M"),
        kept.iter().map(|p| upload_name(p, "M")).collect::<Vec<_>>()
    );
}

// ---------- C3(a) overlap ----------

/// At one upload: the file being uploaded (its staged name) and the master variants asked for.
type UploadSeen = (String, Vec<String>);

#[test]
fn c3a_the_next_items_masters_render_while_this_item_uploads() {
    let _serial = serial();
    let a = shop_app();
    let skus = ["K1", "K2", "K3"];
    let sessions: Vec<i64> = skus.iter().map(|k| session(&a, k)).collect();
    let photos: Vec<Vec<Photo>> = sessions
        .iter()
        .zip(skus)
        .map(|(s, k)| (0..2).map(|i| photo(&a, *s, &format!("{k}-{i}"))).collect())
        .collect();
    let item_of_variant: HashMap<String, usize> = photos
        .iter()
        .enumerate()
        .flat_map(|(i, ps)| ps.iter().map(move |p| (variant(p), i)))
        .collect();
    a.fake_shop().set_upload_delay(40);
    // At every upload: which item is uploading (its staged file) and whose masters were asked for.
    let seen: Arc<Mutex<Vec<UploadSeen>>> = Arc::default();
    {
        let (seen, fake, shop) = (seen.clone(), a.fake.clone(), a.commerce.clone().unwrap());
        a.fake_shop().set_hook(Some(Box::new(move |op| {
            if op != "upload" {
                return;
            }
            let staged = shop.staged_filenames().last().cloned().unwrap_or_default();
            let masters: Vec<String> = fake
                .state
                .lock()
                .unwrap()
                .issued
                .iter()
                .filter(|(_, _, recipe)| recipe == "PSM Master")
                .map(|(_, variant, _)| variant.clone())
                .collect();
            seen.lock().unwrap().push((staged, masters));
        })));
    }
    publish(&a, &sessions);
    a.fake_shop().set_hook(None);

    for s in &sessions {
        assert_eq!(step(&a, *s), PublishStep::Done);
    }
    let seen = seen.lock().unwrap().clone();
    assert!(!seen.is_empty());
    for (staged, masters) in &seen {
        let uploading = skus
            .iter()
            .position(|k| staged.starts_with(&format!("{k}-")))
            .unwrap();
        let ahead = masters.iter().map(|v| item_of_variant[v]).max().unwrap();
        assert!(
            ahead <= uploading + 1,
            "item {} uploads while item {} renders",
            uploading + 1,
            ahead + 1
        );
    }
    // While item 1 (then item 2) uploads, the next item's masters are already asked for.
    for item in [0, 1] {
        let (_, masters) = seen
            .iter()
            .find(|(staged, _)| staged.starts_with(&format!("{}-", skus[item])))
            .unwrap();
        let next: BTreeSet<&String> = masters
            .iter()
            .filter(|v| item_of_variant[*v] == item + 1)
            .collect();
        assert_eq!(
            next.len(),
            2,
            "item {}'s masters during item {}'s upload",
            item + 2,
            item + 1
        );
    }
    // Final order and results as without overlap.
    let done: Vec<i64> = a
        .events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AppEvent::PublishProgress {
                session_id,
                step: PublishStep::Done,
                ..
            } => Some(*session_id),
            _ => None,
        })
        .collect();
    assert_eq!(done, sessions);
    for (i, k) in skus.iter().enumerate() {
        assert_eq!(
            gallery(&a, k),
            photos[i]
                .iter()
                .map(|p| upload_name(p, k))
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(master_variants(&a).len(), 6, "each master once");
}

// ---------- C4(a) timing line ----------

#[test]
fn c4a_one_timing_line_per_item_and_the_popover_counts_masters_ready() {
    let _serial = serial();
    let a = shop_app();
    let s = session(&a, "K");
    let first = photo(&a, s, "a");
    photo(&a, s, "b");
    // Only the first photo's master is prepared before the publish.
    assert_eq!(prepare(&a).requested, Some(first.id));
    a.complete_all_renders().unwrap();

    let shown: Arc<Mutex<Vec<Option<String>>>> = Arc::default();
    {
        let (shown, svc) = (shown.clone(), service(&a));
        a.fake_shop().set_hook(Some(Box::new(move |op| {
            if op == "upload" {
                shown
                    .lock()
                    .unwrap()
                    .push(svc.activity(s).unwrap().unwrap().masters_ready);
            }
        })));
    }
    let ((), log) = logged(|| publish(&a, &[s]));
    a.fake_shop().set_hook(None);
    let item = a.store().latest_publish(s).unwrap().unwrap().item_id;
    let item = a.store().publish_item(item).unwrap();
    assert_eq!((item.step, item.error), (PublishStep::Done, None), "{log}");

    let fields = timing_fields(&log);
    assert_eq!(fields["item"], "K");
    for key in [
        "masters_ms",
        "outputs_ms",
        "upload_ms",
        "shopify_ms",
        "total_ms",
    ] {
        assert!(
            fields
                .get(key)
                .is_some_and(|v| v.parse::<i64>().is_ok_and(|n| n >= 0)),
            "{key}: {fields:?}"
        );
    }
    assert_eq!(fields["masters_reused"], "1/2");
    let shown = shown.lock().unwrap().clone();
    assert!(!shown.is_empty());
    assert!(
        shown
            .iter()
            .all(|line| line.as_deref() == Some("Masters ready in advance: 1 of 2")),
        "{shown:?}"
    );
    // Not shown once the publish is over.
    assert_eq!(
        service(&a).activity(s).unwrap().unwrap().masters_ready,
        None
    );
}
