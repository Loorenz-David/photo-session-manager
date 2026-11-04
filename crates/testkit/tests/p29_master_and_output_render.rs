//! P29 master and output render (10D MC-28, M34): the 16-bit master rendered on demand and cached,
//! the app's own `render_output` (turn, flip, straighten, pixel rect, Lanczos3, sRGB, one JPEG),
//! the output cache key, local export through it, `needs_review` refusals.

use psm_core::{
    Error,
    domain::{
        composition::{
            CompositionKind, CompositionState, CompositionValues, EditorGeometry, Flip, Frame,
            FrameSource, Orientation, Rect, ReviewReason, Shape,
        },
        photo::{Classification as C, NewPhoto, Photo, PhotoSource},
    },
    events::{AppEvent, ExportState},
    mechanisms::hash::sha256_file,
    ports::images::uid12,
};
use psm_testkit::{
    TestApp,
    fixtures::{
        synthetic_jpeg::SyntheticJpeg,
        synthetic_master::{BLACK, Meta, PAPER, Pattern, SyntheticMaster},
    },
};
use std::{fs, path::PathBuf};

const NOW_MS: i64 = 1_762_250_400_000; // 2025-11-04T10:00:00Z

fn app() -> TestApp {
    let a = TestApp::with_fake_document().unwrap();
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

fn variant(photo: &Photo) -> String {
    format!("v{}", photo.id)
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

/// A RAW on disk with its real SHA-256, known to the fake document, classified; with `preview`
/// its first adjusted preview is delivered at that size (the composition's frame).
fn photo_with(
    a: &TestApp,
    session_id: i64,
    name: &str,
    class: C,
    preview: Option<(u32, u32)>,
) -> Photo {
    let dir = a.dir.path().join("Capture/raws");
    fs::create_dir_all(&dir).unwrap();
    let raw = dir.join(format!("{name}.ARW"));
    fs::write(&raw, format!("raw bytes of {name}")).unwrap();
    let photo = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id,
            raw_path: raw.to_string_lossy().into_owned(),
            raw_hash: Some(sha256_file(&raw).unwrap()),
            captured_at: Some(NOW_MS),
            source: PhotoSource::Capture,
            created_at: NOW_MS,
        })
        .unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&variant(&photo)], false);
    if let Some((w, h)) = preview {
        a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
        a.app.previews.request_initial(&photo.raw_path).unwrap();
        a.complete_render_with(&last_issued(a), &SyntheticJpeg::rgb(w, h, [200, 100, 50]))
            .unwrap();
    }
    if class != C::Unreviewed {
        a.review().classify(photo.id, class).unwrap();
    }
    a.store().photo(photo.id).unwrap()
}

/// `K`'s photo: a preview of 2560×1707 gives the composition its frame.
fn photo(a: &TestApp, session_id: i64, name: &str, class: C) -> Photo {
    photo_with(a, session_id, name, class, Some((2560, 1707)))
}

/// A Capture One edit of the variant (outside the own-write window), then its new preview.
fn rerender(a: &TestApp, photo: &Photo, w: u32, h: u32) {
    a.clock.advance(31_000);
    a.deliver_hook(
        &serde_json::json!({"hook":"variant_adjusted","variant_id":variant(photo),"raw":photo.raw_path})
            .to_string(),
    )
    .unwrap();
    a.tick(20_000).unwrap();
    a.complete_render_with(&last_issued(a), &SyntheticJpeg::rgb(w, h, [200, 100, 50]))
        .unwrap();
}

fn issued(a: &TestApp) -> Vec<(String, String, String)> {
    a.fake.state.lock().unwrap().issued.clone()
}

/// Completes every open master render, the master chosen by the variant.
fn complete_masters(a: &TestApp, master_for: &dyn Fn(&str) -> SyntheticMaster) {
    loop {
        let open: Vec<(String, String)> = issued(a)
            .into_iter()
            .filter(|(job, _, recipe)| {
                recipe == "PSM Master" && a.app.editing.render_completion(job).is_some()
            })
            .map(|(job, variant, _)| (job, variant))
            .collect();
        if open.is_empty() {
            return;
        }
        for (job, variant) in open {
            a.complete_master_render(&job, &master_for(&variant))
                .unwrap();
        }
    }
}

fn render_output_jobs(a: &TestApp, statuses: &str) -> i64 {
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            &format!(
                "SELECT COUNT(*) FROM jobs WHERE kind='render_output' AND status IN ({statuses})"
            ),
            [],
            |r| r.get(0),
        )
        .unwrap()
}

/// `tick(0)` until no `render_output` job is queued or running.
fn drain_outputs(a: &TestApp) {
    for _ in 0..10 {
        a.tick(0).unwrap();
        if render_output_jobs(a, "'queued','running'") == 0 {
            return;
        }
    }
    panic!("render_output jobs did not drain");
}

/// `export(a, s, master_for)` of the plan: run, complete the masters, render the outputs.
fn export(a: &TestApp, s: i64, master_for: &dyn Fn(&str) -> SyntheticMaster) {
    a.app.export.run(s).unwrap();
    complete_masters(a, master_for);
    drain_outputs(a);
}

fn solid(w: u32, h: u32) -> impl Fn(&str) -> SyntheticMaster {
    move |_| SyntheticMaster::solid(w, h, PAPER)
}

fn exported(a: &TestApp, name: &str) -> PathBuf {
    a.export_root().join("K").join(name)
}
fn output(a: &TestApp, name: &str) -> image::RgbImage {
    a.decode(&exported(a, name)).unwrap()
}

fn dark(px: &image::Rgb<u8>) -> bool {
    px.0.iter().all(|c| *c < 100)
}
fn light(px: &image::Rgb<u8>) -> bool {
    px.0.iter().all(|c| *c > 160)
}
fn grey(px: &image::Rgb<u8>) -> bool {
    px.0.iter().all(|c| (100..=160).contains(c))
}

fn values(
    kind: CompositionKind,
    shape: Shape,
    turns: u8,
    flip: Flip,
    straighten: f64,
    rect: (f64, f64, f64, f64),
) -> CompositionValues {
    CompositionValues {
        kind,
        shape,
        orientation: Orientation::Named,
        quarter_turns: turns,
        flip,
        straighten,
        rect: Rect {
            x: rect.0,
            y: rect.1,
            w: rect.2,
            h: rect.3,
        },
    }
}
fn original() -> CompositionValues {
    values(
        CompositionKind::Normal,
        Shape::Original,
        0,
        Flip::None,
        0.0,
        (0.0, 0.0, 1.0, 1.0),
    )
}

fn export_states(a: &TestApp) -> Vec<ExportState> {
    a.events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AppEvent::ExportProgress { state, .. } => Some(*state),
            _ => None,
        })
        .collect()
}

fn file_names(dir: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .map(|entries| {
            entries
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

fn g(a: &TestApp, photo: &Photo) -> i64 {
    a.store().adjusted_gen(&photo.raw_path).unwrap()
}

/// `K` with main `[a, b]` exported once with solid 600×400 masters (C1(a)'s state).
fn exported_pair() -> (TestApp, i64, Photo, Photo) {
    let a = app();
    let s = session(&a, "K");
    let pa = photo(&a, s, "a", C::Main);
    let pb = photo(&a, s, "b", C::Main);
    export(&a, s, &solid(600, 400));
    (a, s, pa, pb)
}

#[test]
fn c1a_first_export_renders_one_master_per_raw_and_never_writes_the_raw() {
    let a = app();
    let s = session(&a, "K");
    let pa = photo(&a, s, "a", C::Main);
    let pb = photo(&a, s, "b", C::Main);
    let raw_hashes = [
        sha256_file(std::path::Path::new(&pa.raw_path)).unwrap(),
        sha256_file(std::path::Path::new(&pb.raw_path)).unwrap(),
    ];
    let before = issued(&a).len();
    a.app.export.run(s).unwrap();
    let requested: Vec<(String, String)> = issued(&a)[before..]
        .iter()
        .map(|(_, v, recipe)| (v.clone(), recipe.clone()))
        .collect();
    assert_eq!(
        requested,
        vec![
            (variant(&pa), "PSM Master".to_string()),
            (variant(&pb), "PSM Master".to_string())
        ]
    );
    assert!(!issued(&a).iter().any(|(_, _, recipe)| recipe == "PSM Web"));
    complete_masters(&a, &solid(600, 400));
    for p in [&pa, &pb] {
        assert!(
            a.masters_dir(&p.raw_path)
                .join(format!("master-{}.tif", g(&a, p)))
                .is_file()
        );
    }
    drain_outputs(&a);
    assert!(exported(&a, "K_01_main.jpg").is_file());
    assert!(exported(&a, "K_02_main.jpg").is_file());
    assert_eq!(
        [
            sha256_file(std::path::Path::new(&pa.raw_path)).unwrap(),
            sha256_file(std::path::Path::new(&pb.raw_path)).unwrap(),
        ],
        raw_hashes
    );
}

#[test]
fn c1b_export_with_every_output_cached_renders_nothing() {
    let (a, s, _, _) = exported_pair();
    let first = fs::read(exported(&a, "K_01_main.jpg")).unwrap();
    let before = issued(&a).len();
    a.app.export.run(s).unwrap();
    a.tick(0).unwrap();
    assert_eq!(issued(&a).len(), before);
    assert_eq!(export_states(&a).last(), Some(&ExportState::Done));
    assert_eq!(fs::read(exported(&a, "K_01_main.jpg")).unwrap(), first);
}

/// C1(c)'s first half: `a`'s master completes, `b`'s times out twice, the run fails.
fn failed_run() -> (TestApp, i64, Photo, Photo) {
    let a = app();
    let s = session(&a, "K");
    let pa = photo(&a, s, "a", C::Main);
    let pb = photo(&a, s, "b", C::Main);
    a.app.export.run(s).unwrap();
    let job_a = a.fake.jobs_for(&variant(&pa)).last().unwrap().clone();
    a.complete_master_render(&job_a, &SyntheticMaster::solid(600, 400, PAPER))
        .unwrap();
    a.tick(121_000).unwrap();
    a.tick(121_000).unwrap();
    assert_eq!(export_states(&a).last(), Some(&ExportState::Failed));
    (a, s, pa, pb)
}

#[test]
fn c1c_new_generation_replaces_the_kept_master() {
    let (a, s, pa, _) = failed_run();
    let first = g(&a, &pa);
    let old_master = a
        .masters_dir(&pa.raw_path)
        .join(format!("master-{first}.tif"));
    assert!(old_master.is_file());
    rerender(&a, &pa, 2560, 1707);
    assert_eq!(g(&a, &pa), first + 1);
    a.app.export.run(s).unwrap();
    complete_masters(&a, &solid(600, 400));
    assert!(
        a.masters_dir(&pa.raw_path)
            .join(format!("master-{}.tif", first + 1))
            .is_file()
    );
    assert!(!old_master.exists());
    drain_outputs(&a);
    assert_eq!(export_states(&a).last(), Some(&ExportState::Done));
    assert_eq!(
        file_names(&a.outputs_dir(&pa.raw_path)),
        vec![format!(
            "{}-single-g{}-r0-jpeg-2048-q90.jpg",
            uid12(&pa.uid),
            first + 1
        )]
    );
}

#[test]
fn c2a_plain_crop_cuts_the_pixel_rect() {
    let a = app();
    let s = session(&a, "K");
    let p = photo(&a, s, "a", C::Main);
    a.app
        .compositions
        .set(
            p.id,
            values(
                CompositionKind::Normal,
                Shape::Square,
                0,
                Flip::None,
                0.0,
                (0.25, 0.0, 0.5, 0.75),
            ),
        )
        .unwrap();
    export(&a, s, &|_| {
        SyntheticMaster::with(
            600,
            400,
            Pattern::Marks {
                cols: vec![298, 299],
                rows: vec![198, 199],
                ink: BLACK,
                paper: PAPER,
            },
        )
    });
    let img = output(&a, "K_01_main.jpg");
    assert_eq!(img.dimensions(), (300, 300));
    for y in [10, 150, 290] {
        assert!(
            dark(img.get_pixel(148, y)) && dark(img.get_pixel(149, y)),
            "row {y}"
        );
        assert!(
            light(img.get_pixel(147, y)) && light(img.get_pixel(150, y)),
            "row {y}"
        );
    }
    for x in [10, 150, 290] {
        assert!(
            dark(img.get_pixel(x, 198)) && dark(img.get_pixel(x, 199)),
            "col {x}"
        );
        assert!(
            light(img.get_pixel(x, 197)) && light(img.get_pixel(x, 200)),
            "col {x}"
        );
    }
}

#[test]
fn c2b_quarter_turn_then_flip_in_the_turned_frame() {
    let a = app();
    let s = session(&a, "K");
    let p = photo(&a, s, "a", C::Main);
    a.app
        .compositions
        .set(
            p.id,
            values(
                CompositionKind::Normal,
                Shape::Original,
                1,
                Flip::Horizontal,
                0.0,
                (0.0, 0.0, 1.0, 1.0),
            ),
        )
        .unwrap();
    export(&a, s, &|_| {
        SyntheticMaster::with(
            600,
            400,
            Pattern::Marks {
                cols: vec![100, 101],
                rows: vec![50, 51],
                ink: BLACK,
                paper: PAPER,
            },
        )
    });
    let img = output(&a, "K_01_main.jpg");
    assert_eq!(img.dimensions(), (400, 600));
    for x in [10, 200, 390] {
        assert!(
            dark(img.get_pixel(x, 100)) && dark(img.get_pixel(x, 101)),
            "col {x}"
        );
        assert!(
            light(img.get_pixel(x, 99)) && light(img.get_pixel(x, 102)),
            "col {x}"
        );
    }
    for y in [10, 300, 590] {
        assert!(
            dark(img.get_pixel(50, y)) && dark(img.get_pixel(51, y)),
            "row {y}"
        );
        assert!(
            light(img.get_pixel(49, y)) && light(img.get_pixel(52, y)),
            "row {y}"
        );
    }
}

#[test]
fn c2c_straighten_rotates_clockwise_about_the_centre() {
    let a = app();
    let s = session(&a, "K");
    let p = photo(&a, s, "a", C::Main);
    a.app
        .compositions
        .set(
            p.id,
            values(
                CompositionKind::Normal,
                Shape::Square,
                0,
                Flip::None,
                10.0,
                (0.333333333, 0.25, 0.333333333, 0.5),
            ),
        )
        .unwrap();
    export(&a, s, &|_| {
        SyntheticMaster::with(
            600,
            400,
            Pattern::Marks {
                cols: vec![299, 300],
                rows: vec![],
                ink: BLACK,
                paper: PAPER,
            },
        )
    });
    let img = output(&a, "K_01_main.jpg");
    assert_eq!(img.dimensions(), (200, 200));
    let darkest = |y: u32| -> u32 {
        (0..200)
            .min_by_key(|x| {
                let px = img.get_pixel(*x, y).0;
                px.iter().map(|c| u32::from(*c)).sum::<u32>()
            })
            .unwrap()
    };
    assert!((116..=119).contains(&darkest(0)), "row 0: {}", darkest(0));
    assert!(
        (98..=101).contains(&darkest(100)),
        "row 100: {}",
        darkest(100)
    );
    assert!(
        (81..=84).contains(&darkest(199)),
        "row 199: {}",
        darkest(199)
    );
    for (x, y) in [(0, 0), (199, 199)] {
        assert!(img.get_pixel(x, y).0.iter().all(|c| *c > 120), "({x},{y})");
    }
}

#[test]
fn c2d_default_materialised_from_the_master() {
    let a = app();
    let s = session(&a, "K");
    let p = photo_with(&a, s, "a", C::Main, None);
    export(&a, s, &|_| {
        SyntheticMaster::with(
            600,
            400,
            Pattern::Marks {
                cols: vec![98, 99, 500, 501],
                rows: vec![],
                ink: BLACK,
                paper: PAPER,
            },
        )
    });
    let img = output(&a, "K_01_main.jpg");
    assert_eq!(img.dimensions(), (400, 400));
    let c = a.composition(p.id).unwrap();
    let fingerprint = c.fingerprint.unwrap();
    assert_eq!(fingerprint.frame, Frame { w: 600, h: 400 });
    let rect = c.rect.unwrap();
    for (got, want) in [
        (rect.x, 0.166666667),
        (rect.y, 0.0),
        (rect.w, 0.666666667),
        (rect.h, 1.0),
    ] {
        assert!((got - want).abs() < 1e-6, "{rect:?}");
    }
    assert_eq!((c.state, c.revision), (CompositionState::Ok, 0));
    assert!(light(img.get_pixel(0, 200)));
    for x in 0..400 {
        for y in [10, 200, 390] {
            assert!(!dark(img.get_pixel(x, y)), "({x},{y})");
        }
    }
}

#[test]
fn c2e_original_keeps_the_whole_frame() {
    let a = app();
    let s = session(&a, "K");
    let p = photo(&a, s, "a", C::Main);
    a.app.compositions.set(p.id, original()).unwrap();
    export(&a, s, &solid(600, 400));
    assert_eq!(output(&a, "K_01_main.jpg").dimensions(), (600, 400));
}

#[test]
fn c3a_split_cuts_two_adjacent_halves() {
    let a = app();
    let s = session(&a, "K");
    let p = photo_with(&a, s, "a", C::Main, Some((1001, 601)));
    a.app
        .compositions
        .set(
            p.id,
            values(
                CompositionKind::Split,
                Shape::Square,
                0,
                Flip::None,
                0.0,
                // The plan's h = 0.831946755 misses P28's split check |w·W − 2·h·H| ≤ 1 by
                // 5e-7 px; 0.83194676 gives the same pixel rect {0, 50, 1001 → 1000, 500}.
                (0.0, 0.084026622, 1.0, 0.83194676),
            ),
        )
        .unwrap();
    export(&a, s, &|_| {
        SyntheticMaster::with(1001, 601, Pattern::ColumnMarker { mid: 500 })
    });
    let left = output(&a, "K_01_main.jpg");
    let right = output(&a, "K_02_main.jpg");
    assert_eq!(left.dimensions(), (500, 500));
    assert_eq!(right.dimensions(), (500, 500));
    for y in [10, 250, 490] {
        assert!(dark(left.get_pixel(499, y)), "left 499 row {y}");
        assert!(grey(left.get_pixel(498, y)), "left 498 row {y}");
        assert!(light(right.get_pixel(0, y)), "right 0 row {y}");
        assert!(grey(right.get_pixel(1, y)), "right 1 row {y}");
    }
    let item: serde_json::Value =
        serde_json::from_slice(&fs::read(exported(&a, "item.json")).unwrap()).unwrap();
    assert_eq!(
        item["main"],
        serde_json::json!(["K_01_main.jpg", "K_02_main.jpg"])
    );
}

#[test]
fn c4a_larger_rect_is_resized_to_the_long_edge() {
    let a = app();
    let s = session(&a, "K");
    let p = photo(&a, s, "a", C::Main);
    a.app.compositions.set(p.id, original()).unwrap();
    export(&a, s, &solid(3000, 2000));
    assert_eq!(output(&a, "K_01_main.jpg").dimensions(), (2048, 1365));
}

/// C4(b)'s export: a 600×400 `original` output.
fn small_original() -> TestApp {
    let a = app();
    let s = session(&a, "K");
    let p = photo(&a, s, "a", C::Main);
    a.app.compositions.set(p.id, original()).unwrap();
    export(&a, s, &solid(600, 400));
    a
}

#[test]
fn c4b_smaller_rect_is_never_upscaled() {
    let a = small_original();
    assert_eq!(output(&a, "K_01_main.jpg").dimensions(), (600, 400));
}

/// The JPEG's marker segments before the scan: `(marker, payload)`.
fn segments(bytes: &[u8]) -> Vec<(u8, Vec<u8>)> {
    let mut out = Vec::new();
    let mut i = 2;
    while i + 4 <= bytes.len() && bytes[i] == 0xFF {
        let marker = bytes[i + 1];
        let len = usize::from(u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]));
        out.push((marker, bytes[i + 4..i + 2 + len].to_vec()));
        if marker == 0xDA {
            break;
        }
        i += 2 + len;
    }
    out
}

#[test]
fn c4c_one_encode_with_srgb_profile_and_full_chroma() {
    let a = small_original();
    let bytes = fs::read(exported(&a, "K_01_main.jpg")).unwrap();
    let segs = segments(&bytes);
    let icc: Vec<u8> = segs
        .iter()
        .filter(|(m, p)| *m == 0xE2 && p.starts_with(b"ICC_PROFILE\0"))
        .flat_map(|(_, p)| p[14..].to_vec())
        .collect();
    assert!(!icc.is_empty(), "no ICC profile");
    let count = u32::from_be_bytes(icc[128..132].try_into().unwrap()) as usize;
    let desc = (0..count)
        .map(|n| &icc[132 + 12 * n..144 + 12 * n])
        .find(|e| &e[0..4] == b"desc")
        .map(|e| {
            let offset = u32::from_be_bytes(e[4..8].try_into().unwrap()) as usize;
            let len = u32::from_be_bytes(e[8..12].try_into().unwrap()) as usize;
            let tag = &icc[offset..offset + len];
            let text: Vec<u16> = tag[28..]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u16::from_be_bytes(*c))
                .collect();
            String::from_utf16_lossy(&text)
        })
        .expect("desc tag");
    assert!(desc.contains("sRGB"), "{desc}");
    let sof = segs
        .iter()
        .find(|(m, _)| *m == 0xC0)
        .map(|(_, p)| p.clone())
        .expect("SOF0");
    assert_eq!(sof[5], 3);
    for c in 0..3 {
        assert_eq!(sof[6 + 3 * c + 1], 0x11, "component {c}");
    }
    let pairs = |b: u8| bytes.windows(2).filter(|w| w == &[0xFF, b]).count();
    assert_eq!((pairs(0xD8), pairs(0xD9)), (1, 1));
    assert_eq!(&bytes[..2], &[0xFF, 0xD8]);
    assert_eq!(&bytes[bytes.len() - 2..], &[0xFF, 0xD9]);
}

#[test]
fn c4d_downscale_averages_in_linear_light() {
    let mut a = app();
    a.settings_set(&serde_json::json!({"output_format":{"long_edge":512,"quality":90}}))
        .unwrap();
    let s = session(&a, "K");
    let p = photo_with(&a, s, "a", C::Main, Some((2048, 2048)));
    a.app.compositions.set(p.id, original()).unwrap();
    export(&a, s, &|_| {
        SyntheticMaster::with(1024, 1024, Pattern::Checker1px)
    });
    let img = output(&a, "K_01_main.jpg");
    assert_eq!(img.dimensions(), (512, 512));
    for y in [40, 170, 300, 430] {
        for x in [60, 190, 320, 450] {
            let px = img.get_pixel(x, y);
            assert!(
                px.0.iter().all(|c| (178..=196).contains(c)),
                "({x},{y}) {px:?}"
            );
        }
    }
}

#[test]
fn c4e_output_format_setting_is_validated_and_stored() {
    let mut a = TestApp::new().unwrap();
    let shown = a.app.settings_get().unwrap();
    assert_eq!(
        shown["output_format"],
        serde_json::json!({"long_edge": 2048, "quality": 90})
    );
    assert!(shown.get("web_recipe").is_none());
    a.settings_set(&serde_json::json!({"output_format":{"long_edge":1024,"quality":90}}))
        .unwrap();
    let want = serde_json::json!({"long_edge": 1024, "quality": 90});
    assert_eq!(a.app.settings_get().unwrap()["output_format"], want);
    let settings_file = a.app.paths.settings();
    let stored = || -> serde_json::Value {
        serde_json::from_slice(&fs::read(&settings_file).unwrap()).unwrap()
    };
    assert_eq!(stored()["output_format"], want);
    for patch in [
        serde_json::json!({"output_format":{"long_edge":100,"quality":90}}),
        serde_json::json!({"output_format":{"long_edge":1024,"quality":101}}),
    ] {
        let result = a.settings_set(&patch);
        assert!(
            matches!(&result, Err(Error::InvalidValue(key)) if key == "output_format"),
            "{result:?}"
        );
    }
    assert_eq!(a.app.settings_get().unwrap()["output_format"], want);
    assert_eq!(stored()["output_format"], want);
    let result = a.settings_set(&serde_json::json!({"web_recipe":"x"}));
    assert!(
        matches!(&result, Err(Error::InvalidValue(key)) if key == "web_recipe"),
        "{result:?}"
    );
}

fn exif_of(path: &std::path::Path) -> exif::Exif {
    exif::Reader::new()
        .read_from_container(&mut std::io::BufReader::new(fs::File::open(path).unwrap()))
        .unwrap()
}
fn ascii(exif: &exif::Exif, tag: exif::Tag) -> Option<Vec<u8>> {
    match &exif.get_field(tag, exif::In::PRIMARY)?.value {
        exif::Value::Ascii(parts) => parts.first().cloned(),
        _ => None,
    }
}
fn orientation(exif: &exif::Exif) -> Option<u32> {
    exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?
        .value
        .get_uint(0)
}

#[test]
fn c5a_metadata_allowlist_copied_gps_dropped_orientation_upright() {
    let a = app();
    let s = session(&a, "K");
    photo(&a, s, "a", C::Main);
    let master = SyntheticMaster {
        meta: Meta {
            make: Some("Sony".into()),
            model: Some("ILCE-7RM4".into()),
            date_time_original: Some("2026:11:04 10:11:12".into()),
            copyright: Some("© Loorenz".into()),
            orientation: Some(6),
            gps: true,
        },
        ..SyntheticMaster::solid(600, 400, PAPER)
    };
    // The fixture's own check: the master carries the tags and the GPS IFD.
    let probe = a.dir.path().join("probe.tif");
    fs::write(&probe, master.bytes()).unwrap();
    let source = exif_of(&probe);
    assert_eq!(ascii(&source, exif::Tag::Make), Some(b"Sony".to_vec()));
    assert!(
        source
            .get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY)
            .is_some()
    );
    assert_eq!(orientation(&source), Some(6));
    export(&a, s, &|_| master.clone());
    let path = exported(&a, "K_01_main.jpg");
    let out = exif_of(&path);
    assert_eq!(ascii(&out, exif::Tag::Make), Some(b"Sony".to_vec()));
    assert_eq!(ascii(&out, exif::Tag::Model), Some(b"ILCE-7RM4".to_vec()));
    assert_eq!(
        ascii(&out, exif::Tag::DateTimeOriginal),
        Some(b"2026:11:04 10:11:12".to_vec())
    );
    assert_eq!(
        ascii(&out, exif::Tag::Copyright),
        Some("© Loorenz".as_bytes().to_vec())
    );
    assert_eq!(orientation(&out), Some(1));
    let allowed = [
        exif::Tag::Make,
        exif::Tag::Model,
        exif::Tag::DateTimeOriginal,
        exif::Tag::Copyright,
        exif::Tag::Orientation,
        exif::Tag::ExifVersion,
        exif::Tag::ColorSpace,
        exif::Tag::PixelXDimension,
        exif::Tag::PixelYDimension,
        exif::Tag::ResolutionUnit,
        exif::Tag::XResolution,
        exif::Tag::YResolution,
        exif::Tag::YCbCrPositioning,
        // The structural pointer to the Exif IFD that holds the allowlisted Exif tags.
        exif::Tag::ExifIFDPointer,
    ];
    for field in out.fields() {
        assert!(
            field.tag.context() != exif::Context::Gps,
            "GPS field {}",
            field.tag
        );
        assert!(allowed.contains(&field.tag), "tag {}", field.tag);
    }
    let bytes = fs::read(&path).unwrap();
    assert!(
        !segments(&bytes)
            .iter()
            .any(|(m, p)| *m == 0xE1 && p.starts_with(b"http://ns.adobe.com/xap/1.0/"))
    );
}

#[test]
fn c5b_master_without_metadata_copies_nothing() {
    let a = app();
    let s = session(&a, "K");
    photo(&a, s, "a", C::Main);
    export(&a, s, &solid(600, 400));
    let path = exported(&a, "K_01_main.jpg");
    a.decode(&path).unwrap();
    let out = exif_of(&path);
    assert_eq!(orientation(&out), Some(1));
    for tag in [
        exif::Tag::Make,
        exif::Tag::Model,
        exif::Tag::DateTimeOriginal,
        exif::Tag::Copyright,
    ] {
        assert!(out.get_field(tag, exif::In::PRIMARY).is_none(), "{tag}");
    }
}

/// C6's chain on `K` main `[a, b]` with 3000×2000 masters: the first export (C1(a) shape), then
/// (a) a new rect for `a`, then (b) the output size 1024. Returns the app after `steps` steps.
fn c6_chain(steps: usize) -> (TestApp, Photo, i64) {
    let mut a = app();
    let s = session(&a, "K");
    let pa = photo(&a, s, "a", C::Main);
    photo(&a, s, "b", C::Main);
    export(&a, s, &solid(3000, 2000));
    let first = g(&a, &pa);
    a.app
        .compositions
        .set(
            pa.id,
            values(
                CompositionKind::Normal,
                Shape::Square,
                0,
                Flip::None,
                0.0,
                (0.0, 0.0, 1707.0 / 2560.0, 1.0),
            ),
        )
        .unwrap();
    assert_eq!(a.composition(pa.id).unwrap().revision, 1);
    export(&a, s, &solid(3000, 2000));
    if steps >= 2 {
        a.settings_set(&serde_json::json!({"output_format":{"long_edge":1024,"quality":90}}))
            .unwrap();
        export(&a, s, &solid(3000, 2000));
    }
    (a, pa, first)
}

#[test]
fn c6a_composition_change_replaces_the_cached_output() {
    let (a, pa, g0) = c6_chain(1);
    let name = format!("{}-single-g{g0}-r1-jpeg-2048-q90.jpg", uid12(&pa.uid));
    assert_eq!(file_names(&a.outputs_dir(&pa.raw_path)), vec![name.clone()]);
    assert_eq!(
        fs::read(exported(&a, "K_01_main.jpg")).unwrap(),
        fs::read(a.outputs_dir(&pa.raw_path).join(name)).unwrap()
    );
}

#[test]
fn c6b_output_format_change_replaces_the_cached_output() {
    let (a, pa, g0) = c6_chain(2);
    assert_eq!(
        file_names(&a.outputs_dir(&pa.raw_path)),
        vec![format!(
            "{}-single-g{g0}-r1-jpeg-1024-q90.jpg",
            uid12(&pa.uid)
        )]
    );
    let (w, h) = output(&a, "K_01_main.jpg").dimensions();
    assert_eq!(w.max(h), 1024);
}

#[test]
fn c6c_manifest_records_every_files_cache_key() {
    let (a, pa, g0) = c6_chain(2);
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(exported(&a, "manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["format_key"], "jpeg-1024-q90");
    assert_eq!(
        manifest["outputs"]["K_01_main.jpg"],
        serde_json::json!({"photo_id": pa.id, "slot": "single", "adjusted_gen": g0, "composition_rev": 1})
    );
    let item: serde_json::Value =
        serde_json::from_slice(&fs::read(exported(&a, "item.json")).unwrap()).unwrap();
    assert_eq!(
        item["main"],
        serde_json::json!(["K_01_main.jpg", "K_02_main.jpg"])
    );
}

#[test]
fn c7a_masters_released_after_a_completed_local_export() {
    let (a, _, pa, pb) = exported_pair();
    for p in [&pa, &pb] {
        assert!(!a.masters_dir(&p.raw_path).exists());
        assert_eq!(file_names(&a.outputs_dir(&p.raw_path)).len(), 1);
    }
}

#[test]
fn c7b_failed_export_keeps_its_master() {
    let (a, _, pa, _) = failed_run();
    assert!(
        a.masters_dir(&pa.raw_path)
            .join(format!("master-{}.tif", g(&a, &pa)))
            .is_file()
    );
    assert!(!a.export_root().join("K").exists());
}

#[test]
fn c7c_cached_outputs_need_no_master() {
    let (a, s, _, _) = exported_pair();
    let files = [
        fs::read(exported(&a, "K_01_main.jpg")).unwrap(),
        fs::read(exported(&a, "K_02_main.jpg")).unwrap(),
    ];
    let before = issued(&a).len();
    let jobs = render_output_jobs(&a, "'queued','running','done','failed'");
    a.app.export.run(s).unwrap();
    a.tick(0).unwrap();
    assert_eq!(issued(&a).len(), before);
    assert_eq!(
        render_output_jobs(&a, "'queued','running','done','failed'"),
        jobs
    );
    assert_eq!(export_states(&a).last(), Some(&ExportState::Done));
    assert_eq!(
        [
            fs::read(exported(&a, "K_01_main.jpg")).unwrap(),
            fs::read(exported(&a, "K_02_main.jpg")).unwrap(),
        ],
        files
    );
}

/// The P28 C5(a) shape: a Capture One geometry change on the next preview.
fn flag(a: &TestApp, p: &Photo) {
    a.fake.set_variant_geometry(
        &variant(p),
        EditorGeometry {
            rotation: 3.0,
            ..Default::default()
        },
    );
    rerender(a, p, 2560, 1707);
    assert_eq!(
        a.composition(p.id).unwrap().state,
        CompositionState::NeedsReview
    );
}

#[test]
fn c8a_export_refuses_while_a_main_photo_needs_review() {
    let a = app();
    let s = session(&a, "K");
    photo(&a, s, "a", C::Main);
    let pb = photo(&a, s, "b", C::Main);
    flag(&a, &pb);
    let before = issued(&a).len();
    let result = a.app.export.run(s);
    assert!(
        matches!(&result, Err(Error::NeedsReview { photos }) if photos == &vec!["Main 02".to_string()]),
        "{result:?}"
    );
    assert_eq!(issued(&a).len(), before);
    assert!(
        !file_names(&a.export_root())
            .iter()
            .any(|n| n.starts_with(".K.partial-"))
    );
    assert_eq!(export_states(&a).last(), Some(&ExportState::Failed));
}

#[test]
fn c8b_condition_label_and_unreviewed_photos_do_not_block() {
    let a = app();
    let s = session(&a, "K");
    photo(&a, s, "a", C::Main);
    let pc = photo(&a, s, "c", C::Condition);
    flag(&a, &pc);
    let result = a.app.export.run(s);
    assert!(
        matches!(&result, Err(Error::NeedsReview { photos }) if photos == &vec!["Condition 01".to_string()]),
        "{result:?}"
    );

    let b = app();
    let s = session(&b, "K");
    photo(&b, s, "a", C::Main);
    let pu = photo(&b, s, "u", C::Unreviewed);
    flag(&b, &pu);
    export(&b, s, &solid(600, 400));
    assert_eq!(export_states(&b).last(), Some(&ExportState::Done));
    assert!(b.export_root().join("K/K_01_main.jpg").is_file());
}

#[test]
fn c8c_master_with_moved_geometry_flags_review_and_fails_the_run() {
    let a = app();
    let s = session(&a, "K");
    let p = photo(&a, s, "a", C::Main);
    let before = a.composition(p.id).unwrap().fingerprint.unwrap();
    assert_eq!(
        (before.frame, before.source),
        (Frame { w: 2560, h: 1707 }, FrameSource::Preview)
    );
    let geometry = EditorGeometry {
        rotation: 3.0,
        ..Default::default()
    };
    a.fake.set_variant_geometry(&variant(&p), geometry.clone());
    a.app.export.run(s).unwrap();
    complete_masters(&a, &solid(600, 400));
    let c = a.composition(p.id).unwrap();
    assert_eq!(
        (c.state, c.review_reason),
        (CompositionState::NeedsReview, Some(ReviewReason::Geometry))
    );
    let fingerprint = c.fingerprint.unwrap();
    assert_eq!(
        (fingerprint.frame, fingerprint.source, fingerprint.geometry),
        (Frame { w: 600, h: 400 }, FrameSource::Master, geometry)
    );
    assert_eq!(export_states(&a).last(), Some(&ExportState::Failed));
    assert!(
        a.masters_dir(&p.raw_path)
            .join(format!("master-{}.tif", g(&a, &p)))
            .is_file()
    );
    assert_eq!(
        render_output_jobs(&a, "'queued','running','done','failed'"),
        0
    );
    let result = a.app.export.run(s);
    assert!(
        matches!(&result, Err(Error::NeedsReview { photos }) if photos == &vec!["Main 01".to_string()]),
        "{result:?}"
    );
}

#[test]
fn c8d_master_size_change_against_the_stored_master_flags_review() {
    let a = app();
    let s = session(&a, "K");
    let p = photo(&a, s, "a", C::Main);
    export(&a, s, &solid(600, 400));
    let stored = a.composition(p.id).unwrap().fingerprint.unwrap();
    assert_eq!(
        (stored.frame, stored.source, stored.geometry.clone()),
        (
            Frame { w: 600, h: 400 },
            FrameSource::Master,
            EditorGeometry::default()
        )
    );
    rerender(&a, &p, 2560, 1707);
    let c = a.composition(p.id).unwrap();
    assert_eq!(c.state, CompositionState::Ok);
    assert_eq!(c.fingerprint.unwrap(), stored);
    a.app.export.run(s).unwrap();
    complete_masters(&a, &solid(640, 400));
    let c = a.composition(p.id).unwrap();
    assert_eq!(
        (c.state, c.review_reason),
        (CompositionState::NeedsReview, Some(ReviewReason::Frame))
    );
    let fingerprint = c.fingerprint.unwrap();
    assert_eq!(
        (fingerprint.frame, fingerprint.source, fingerprint.geometry),
        (
            Frame { w: 640, h: 400 },
            FrameSource::Master,
            EditorGeometry::default()
        )
    );
    assert_eq!(export_states(&a).last(), Some(&ExportState::Failed));
}
