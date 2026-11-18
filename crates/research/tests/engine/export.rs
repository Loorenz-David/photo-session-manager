//! R7·U2 (A21·C1(a)–(c), C2–C6): `ResearchService::{purge_expired, export}` on the research
//! harness after scripted runs (08B MC-A11). Outcomes are table rows, files on disk and the
//! zip's entries.

use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use chrono::TimeZone;
use psm_core::domain::attribute::Provenance;
use psm_research::{
    api::ExportSelection,
    domain::{RunState, Stage},
    fakes,
    retention::DAY_MS,
    store as rstore,
};
use serde_json::{Value as Json, json};

use crate::harness::*;

/// A finished run of three Main photos: one observe call over the three (R12·U1), the cover's
/// visual search, and the decide call that failed once (transient) and was answered at its second
/// attempt: four calls, three sent images. Observe reads Teak (sure).
struct Four {
    h: Harness,
    run: i64,
    mains: Vec<i64>,
}

fn four_calls() -> Four {
    let h = Harness::chair();
    let mains: Vec<i64> = (0..3)
        .map(|_| h.add_main_photo_with_preview(h.session_id))
        .collect();
    h.model.reply(
        Stage::Observe,
        psm_research::stages::OBSERVE_UNIT,
        json!({"reads": {WOODS: {"value": ["Teak"], "sure": true}}, "marking_photos": []}),
    );
    h.model
        .fault_for(Stage::Decide, "decide", fakes::transient("busy"));
    h.changed();
    h.tick(1_000);
    let run = h.runs(h.session_id)[0].id;
    assert_eq!(h.run_row(run).state, RunState::Finished);
    assert_eq!(h.calls(run).len(), 4);
    Four { h, run, mains }
}

/// A fresh destination folder inside the test's directory.
fn dest(h: &Harness, name: &str) -> PathBuf {
    let dir = h.app.dir.path().join(name);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// The export folder of a zip path (`…/research-export-<stamp>.zip` → `…/research-export-<stamp>`).
fn folder_of(zip: &Path) -> PathBuf {
    zip.with_extension("")
}

/// Every file under `root`, relative, `/`-separated, sorted.
fn tree(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    out.sort();
    out
}

fn zip_entries(zip: &Path) -> Vec<String> {
    let archive = zip::ZipArchive::new(fs::File::open(zip).unwrap()).unwrap();
    let mut names: Vec<String> = archive.file_names().map(str::to_owned).collect();
    names.sort();
    names
}

fn zip_bytes(zip: &Path, name: &str) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(fs::File::open(zip).unwrap()).unwrap();
    let mut file = archive.by_name(name).unwrap();
    let mut out = Vec::new();
    file.read_to_end(&mut out).unwrap();
    out
}

fn read_json(path: &Path) -> Json {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// The run folders of an export: `<item id>-<folder key>/run-<id>`.
fn run_dirs(folder: &Path) -> BTreeSet<String> {
    tree(folder)
        .into_iter()
        .filter_map(|p| {
            let parts: Vec<&str> = p.split('/').collect();
            (parts.len() >= 3).then(|| format!("{}/{}", parts[0], parts[1]))
        })
        .collect()
}

fn folder_key(h: &Harness, item: i64) -> String {
    h.query(|c| rstore::folder_key(c, item).unwrap())
}

/// A call row written straight through the store.
fn insert_call(h: &Harness, run: i64, started_at: i64, request: &Json) -> i64 {
    h.query(|c| {
        c.execute(
            "INSERT INTO research_calls(run_id,stage,unit_key,attempt,provider,model,effort,state,\
             request,started_at) VALUES (?1,'observe','u',1,'fake-model','m1','low','answered',?2,?3)",
            rusqlite::params![run, request.to_string(), started_at],
        )
        .unwrap();
        c.last_insert_rowid()
    })
}

fn call_ids(h: &Harness) -> Vec<i64> {
    h.query(|c| {
        rstore::calls(c, None)
            .unwrap()
            .into_iter()
            .map(|call| call.id)
            .collect()
    })
}

fn images(shas: &[&str]) -> Json {
    json!({"images": shas.iter().map(|s| json!({"image_sha256": s})).collect::<Vec<_>>()})
}

fn count(h: &Harness, table: &str) -> i64 {
    h.query(|c| {
        c.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    })
}

fn local_ms(y: i32, mo: u32, d: u32, hh: u32, mm: u32) -> i64 {
    chrono::Local
        .with_ymd_and_hms(y, mo, d, hh, mm, 0)
        .single()
        .unwrap()
        .timestamp_millis()
}

/// A finished run of `item`'s session that started at `started_at`.
fn finished_run(h: &Harness, item: i64, session: i64, started_at: i64) -> i64 {
    let id = h.insert_run_for(item, session, RunState::Finished, None, None);
    h.query(|c| {
        c.execute(
            "UPDATE research_runs SET started_at=?1 WHERE id=?2",
            rusqlite::params![started_at, id],
        )
        .unwrap()
    });
    id
}

#[test]
fn a21_c1a_a_call_is_deleted_exactly_at_record_days() {
    let h = Harness::new();
    let run = h.insert_run(RunState::Finished, None, None);
    let now = h.now();
    let at_limit = insert_call(&h, run, now - 30 * DAY_MS, &json!({}));
    let younger = insert_call(&h, run, now - 30 * DAY_MS + 1, &json!({}));
    h.service.purge_expired(now).unwrap();
    let left = call_ids(&h);
    assert!(!left.contains(&at_limit));
    assert_eq!(left, vec![younger]);
}

#[test]
fn a21_c1b_an_image_a_kept_call_references_stays() {
    let h = Harness::new();
    let run = h.insert_run(RunState::Finished, None, None);
    let shared = h.files().put(b"shared image bytes").unwrap();
    let only_expired = h.files().put(b"image only expired calls sent").unwrap();
    let now = h.now();
    let old = now - 31 * DAY_MS;
    insert_call(&h, run, old, &images(&[&shared, &only_expired]));
    insert_call(&h, run, old, &images(&[&shared]));
    let kept = insert_call(&h, run, now - DAY_MS, &images(&[&shared]));
    h.service.purge_expired(now).unwrap();
    assert_eq!(call_ids(&h), vec![kept]);
    assert!(h.files().exists(&shared));
    assert!(!h.files().exists(&only_expired));
}

#[test]
fn a21_c1c_runs_steps_feedback_outbound_and_outputs_are_kept() {
    let f = four_calls();
    let h = &f.h;
    h.app
        .attributes(Some(INTEGRATION))
        .accept(h.item_id, WOODS)
        .unwrap();
    let kept = [
        "research_runs",
        "research_steps",
        "research_feedback",
        "research_outbound",
        "research_stage_outputs",
    ];
    let before: Vec<i64> = kept.iter().map(|t| count(h, t)).collect();
    assert!(before.iter().all(|n| *n > 0), "{before:?}");
    h.service.purge_expired(h.now() + 31 * DAY_MS).unwrap();
    assert_eq!(count(h, "research_calls"), 0);
    let after: Vec<i64> = kept.iter().map(|t| count(h, t)).collect();
    assert_eq!(after, before);
}

#[test]
fn a21_c2a_selection_by_local_date() {
    let h = Harness::new();
    let late = finished_run(&h, h.item_id, h.session_id, local_ms(2026, 11, 10, 23, 59));
    let early = finished_run(&h, h.item_id, h.session_id, local_ms(2026, 11, 11, 0, 1));
    let zip = h
        .service
        .export(
            ExportSelection::Range {
                from: 20261111,
                to: 20261111,
            },
            &dest(&h, "out"),
        )
        .unwrap();
    let key = folder_key(&h, h.item_id);
    assert_eq!(
        run_dirs(&folder_of(&zip)),
        BTreeSet::from([format!("{}-{key}/run-{early}", h.item_id)])
    );
    assert_ne!(late, early);
}

#[test]
fn a21_c2b_selection_by_items() {
    let h = Harness::new();
    let (other_item, other_session) = h.new_session("R7-OTHER");
    let a1 = finished_run(&h, h.item_id, h.session_id, h.now());
    let a2 = finished_run(&h, h.item_id, h.session_id, h.now() + 1);
    finished_run(&h, other_item, other_session, h.now());
    let zip = h
        .service
        .export(ExportSelection::Items(vec![h.item_id]), &dest(&h, "out"))
        .unwrap();
    let key = folder_key(&h, h.item_id);
    assert_eq!(
        run_dirs(&folder_of(&zip)),
        BTreeSet::from([
            format!("{}-{key}/run-{a1}", h.item_id),
            format!("{}-{key}/run-{a2}", h.item_id)
        ])
    );
}

#[test]
fn a21_c3a_layout_and_zip() {
    let f = four_calls();
    let h = &f.h;
    let zip = h
        .service
        .export(ExportSelection::Items(vec![h.item_id]), &dest(h, "out"))
        .unwrap();
    let folder = folder_of(&zip);
    let name = folder.file_name().unwrap().to_string_lossy().into_owned();
    assert!(
        name.starts_with("research-export-")
            && name.len() == "research-export-YYYYMMDD-HHMMSS".len(),
        "{name}"
    );
    let run_dir = format!("{}-{}/run-{}", h.item_id, folder_key(h, h.item_id), f.run);
    let mut expected: Vec<String> = ["run.json", "steps.json", "writes.json", "feedback.json"]
        .iter()
        .map(|f| format!("{run_dir}/{f}"))
        .collect();
    for call in h.calls(f.run) {
        expected.push(format!("{run_dir}/calls/{}.json", call.id));
    }
    let sent: BTreeSet<String> = h.outbound(f.run).into_iter().map(|o| o.sha256).collect();
    assert_eq!(sent.len(), 3);
    for sha in &sent {
        expected.push(format!("{run_dir}/images/{sha}.jpg"));
        assert_eq!(
            fs::read(
                folder
                    .join(&run_dir)
                    .join("images")
                    .join(format!("{sha}.jpg"))
            )
            .unwrap(),
            fs::read(h.files().path(sha)).unwrap()
        );
    }
    expected.sort();
    assert_eq!(tree(&folder), expected);
    let in_zip: Vec<String> = expected.iter().map(|p| format!("{name}/{p}")).collect();
    assert_eq!(zip_entries(&zip), in_zip);
    for entry in &expected {
        assert_eq!(
            zip_bytes(&zip, &format!("{name}/{entry}")),
            fs::read(folder.join(entry)).unwrap()
        );
    }
}

#[test]
fn a21_c3b_content_steps_calls_and_feedback() {
    let f = four_calls();
    let h = &f.h;
    // A later suggestion on the designer, then the photographer's two actions.
    h.suggest(DESIGNER, json!("Designer A"));
    let attributes = h.app.attributes(Some(INTEGRATION));
    attributes.accept(h.item_id, WOODS).unwrap();
    attributes
        .set_value(h.item_id, DESIGNER, json!("Designer B"), Provenance::Human)
        .unwrap();
    let zip = h
        .service
        .export(ExportSelection::Items(vec![h.item_id]), &dest(h, "out"))
        .unwrap();
    let dir = folder_of(&zip).join(format!(
        "{}-{}/run-{}",
        h.item_id,
        folder_key(h, h.item_id),
        f.run
    ));
    let steps: Vec<Json> = h
        .steps(f.run)
        .into_iter()
        .map(|s| {
            json!({"seq": s.seq, "stage": s.stage, "kind": s.kind, "summary": s.summary,
                   "detail": s.detail, "source_url": s.source_url, "source_title": s.source_title,
                   "photo_ids": s.photo_ids, "field_keys": s.field_keys, "created_at": s.created_at})
        })
        .collect();
    assert_eq!(read_json(&dir.join("steps.json")), Json::Array(steps));
    for call in h.calls(f.run) {
        let file = read_json(&dir.join("calls").join(format!("{}.json", call.id)));
        assert_eq!(file["request"].as_str(), Some(call.request.as_str()));
        assert_eq!(file["response"].as_str(), call.response.as_deref());
    }
    let feedback = read_json(&dir.join("feedback.json"));
    let actions: Vec<(Json, Json, Json, Json)> = feedback
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["key"].clone(),
                f["action"].clone(),
                f["suggested_value"].clone(),
                f["final_value"].clone(),
            )
        })
        .collect();
    assert_eq!(
        actions,
        vec![
            (
                json!(WOODS),
                json!("accepted"),
                json!(["Teak"]),
                json!(["Teak"])
            ),
            (
                json!(DESIGNER),
                json!("edited"),
                json!("Designer A"),
                json!("Designer B")
            ),
        ]
    );
    let writes = read_json(&dir.join("writes.json"));
    assert_eq!(writes["writes"][WOODS]["value"], json!(["Teak"]));
}

#[test]
fn a21_c3c_expired_calls_are_named_in_run_json() {
    let f = four_calls();
    let h = &f.h;
    h.advance(31 * DAY_MS);
    h.service.purge_expired(h.now()).unwrap();
    let zip = h
        .service
        .export(ExportSelection::Items(vec![h.item_id]), &dest(h, "out"))
        .unwrap();
    let run_dir = format!("{}-{}/run-{}", h.item_id, folder_key(h, h.item_id), f.run);
    let dir = folder_of(&zip).join(&run_dir);
    assert_eq!(
        read_json(&dir.join("run.json"))["call_record"],
        json!("call record expired")
    );
    assert!(!tree(&folder_of(&zip)).iter().any(|p| p.contains("/calls/")));
    assert!(dir.join("steps.json").is_file());
    assert_eq!(
        read_json(&dir.join("writes.json"))["writes"][WOODS]["value"],
        json!(["Teak"])
    );
}

#[test]
fn a21_c4a_a_deleted_photo_is_withheld() {
    let f = four_calls();
    let h = &f.h;
    let cover = f.mains[0];
    let outbound = h.outbound(f.run);
    let cover_calls: Vec<i64> = outbound
        .iter()
        .filter(|o| o.photo_id == cover)
        .map(|o| o.call_id)
        .collect();
    assert_eq!(cover_calls.len(), 2);
    let cover_sha = outbound
        .iter()
        .find(|o| o.photo_id == cover)
        .unwrap()
        .sha256
        .clone();
    h.app.review().delete(cover).unwrap();
    let zip = h
        .service
        .export(ExportSelection::Items(vec![h.item_id]), &dest(h, "out"))
        .unwrap();
    let dir = folder_of(&zip).join(format!(
        "{}-{}/run-{}",
        h.item_id,
        folder_key(h, h.item_id),
        f.run
    ));
    assert!(!dir.join("images").join(format!("{cover_sha}.jpg")).exists());
    for call in cover_calls {
        let request = read_json(&dir.join("calls").join(format!("{call}.json")))["request"]
            .as_str()
            .unwrap()
            .to_owned();
        assert!(
            request.contains(r#"{"withheld":"photo deleted"}"#),
            "{request}"
        );
        assert!(!request.contains(&cover_sha), "{request}");
    }
    for o in outbound.iter().filter(|o| o.photo_id != cover) {
        assert!(
            dir.join("images")
                .join(format!("{}.jpg", o.sha256))
                .is_file()
        );
    }
}

#[test]
fn a21_c5a_a_secret_in_a_file_stops_the_export() {
    let f = four_calls();
    let h = &f.h;
    let call = h.calls(f.run)[0].id;
    h.query(|c| {
        c.execute(
            "UPDATE research_calls SET response=?1 WHERE id=?2",
            rusqlite::params![format!("{{\"echo\":\"{MODEL_KEY}\"}}"), call],
        )
        .unwrap()
    });
    let out = dest(h, "out");
    let error = h
        .service
        .export(ExportSelection::Items(vec![h.item_id]), &out)
        .unwrap_err()
        .to_string();
    assert!(error.contains(&format!("calls/{call}.json")), "{error}");
    assert!(!error.contains(MODEL_KEY), "{error}");
    assert_eq!(fs::read_dir(&out).unwrap().count(), 0, "something remained");
}

#[test]
fn a15_c5d_an_escaped_key_in_a_file_stops_the_export() {
    use psm_core::ports::secrets::SecretStore;
    let f = four_calls();
    let h = &f.h;
    let key = crate::adapter_openai::ESCAPED_KEY;
    h.app
        .secrets
        .set(psm_research::service::MODEL_ACCOUNT, key)
        .unwrap();
    let call = h.calls(f.run)[0].id;
    // Seeded as R7's test seeds its key; the call file then holds the key JSON-escaped.
    h.query(|c| {
        c.execute(
            "UPDATE research_calls SET response=?1 WHERE id=?2",
            rusqlite::params![format!("{{\"echo\":\"{key}\"}}"), call],
        )
        .unwrap()
    });
    let out = dest(h, "out");
    let error = h
        .service
        .export(ExportSelection::Items(vec![h.item_id]), &out)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("Export research stopped: ")
            && error.ends_with(&format!("calls/{call}.json contains a key")),
        "{error}"
    );
    assert!(!error.contains(key), "{error}");
    assert_eq!(fs::read_dir(&out).unwrap().count(), 0, "something remained");
}

/// A JPEG: SOI, an APP0 segment, an optional comment segment, SOS, `pixels`, EOI.
fn jpeg(comment: Option<&[u8]>, pixels: &[u8]) -> Vec<u8> {
    let mut out = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
    out.extend_from_slice(b"JFIF\0\x01\x01\0\0\x01\0\x01\0\0");
    if let Some(text) = comment {
        out.extend_from_slice(&[0xFF, 0xFE]);
        out.extend_from_slice(&u16::try_from(text.len() + 2).unwrap().to_be_bytes());
        out.extend_from_slice(text);
    }
    out.extend_from_slice(&[0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3F, 0x00]);
    out.extend_from_slice(pixels);
    out.extend_from_slice(&[0xFF, 0xD9]);
    out
}

/// Replaces the stored bytes of one image the run sent; returns its SHA (the exported name).
fn replace_sent_image(f: &Four, bytes: &[u8]) -> String {
    let sha = f.h.outbound(f.run)[0].sha256.clone();
    fs::write(f.h.files().path(&sha), bytes).unwrap();
    sha
}

#[test]
fn a7_a_value_by_chance_in_an_images_pixel_data_does_not_stop_the_export() {
    let f = four_calls();
    let h = &f.h;
    let bytes = jpeg(None, format!("\x12\x34{MODEL_KEY}\x56").as_bytes());
    let sha = replace_sent_image(&f, &bytes);
    let zip = h
        .service
        .export(ExportSelection::Items(vec![h.item_id]), &dest(h, "out"))
        .unwrap();
    let folder = folder_of(&zip);
    let run_dir = format!("{}-{}/run-{}", h.item_id, folder_key(h, h.item_id), f.run);
    assert_eq!(
        fs::read(
            folder
                .join(run_dir)
                .join("images")
                .join(format!("{sha}.jpg"))
        )
        .unwrap(),
        bytes
    );
}

#[test]
fn a7_a_value_in_an_images_header_or_a_non_jpeg_stops_the_export() {
    for bytes in [
        jpeg(Some(MODEL_KEY.as_bytes()), b"\x01\x02"),
        format!("not a jpeg {MODEL_KEY}").into_bytes(),
        // A JPEG whose segments run past the end is read whole.
        [
            &[0xFF, 0xD8, 0xFF, 0xE1, 0x7F, 0xFF][..],
            MODEL_KEY.as_bytes(),
        ]
        .concat(),
    ] {
        let f = four_calls();
        let h = &f.h;
        let sha = replace_sent_image(&f, &bytes);
        let out = dest(h, "out");
        let error = h
            .service
            .export(ExportSelection::Items(vec![h.item_id]), &out)
            .unwrap_err()
            .to_string();
        assert!(
            error.ends_with(&format!("images/{sha}.jpg contains a key")),
            "{error}"
        );
        assert!(!error.contains(MODEL_KEY), "{error}");
        assert_eq!(fs::read_dir(&out).unwrap().count(), 0, "something remained");
    }
}

/// A WebP: RIFF header, one `VP8 ` chunk holding `pixels`, then the optional extra chunks.
fn webp(pixels: &[u8], extra: &[(&[u8; 4], &[u8])]) -> Vec<u8> {
    let mut body = Vec::new();
    for (kind, data) in std::iter::once((b"VP8 ", pixels)).chain(extra.iter().copied()) {
        body.extend_from_slice(kind);
        body.extend_from_slice(&u32::try_from(data.len()).unwrap().to_le_bytes());
        body.extend_from_slice(data);
        if data.len() % 2 == 1 {
            body.push(0);
        }
    }
    let mut out = b"RIFF".to_vec();
    out.extend_from_slice(&u32::try_from(body.len() + 4).unwrap().to_le_bytes());
    out.extend_from_slice(b"WEBP");
    out.extend(body);
    out
}

/// A PNG with one `IDAT` chunk holding `pixels` and an optional `tEXt` chunk (CRCs not checked).
fn png(pixels: &[u8], text: Option<&[u8]>) -> Vec<u8> {
    fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        out.extend_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        out.extend_from_slice(kind);
        out.extend_from_slice(data);
        out.extend_from_slice(&[0; 4]);
    }
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    chunk(&mut out, b"IHDR", &[0; 13]);
    if let Some(text) = text {
        chunk(&mut out, b"tEXt", &[b"Comment\0".as_slice(), text].concat());
    }
    chunk(&mut out, b"IDAT", pixels);
    chunk(&mut out, b"IEND", &[]);
    out
}

#[test]
fn a8_a_value_by_chance_in_a_webp_or_png_listing_image_does_not_stop_the_export() {
    // Listing images are stored as the shop sent them, under a `.jpg` name.
    let pixels = format!("\x12\x34{MODEL_KEY}\x56");
    for bytes in [webp(pixels.as_bytes(), &[]), png(pixels.as_bytes(), None)] {
        let f = four_calls();
        let h = &f.h;
        let sha = replace_sent_image(&f, &bytes);
        let zip = h
            .service
            .export(ExportSelection::Items(vec![h.item_id]), &dest(h, "out"))
            .unwrap();
        let run_dir = format!("{}-{}/run-{}", h.item_id, folder_key(h, h.item_id), f.run);
        let exported = folder_of(&zip)
            .join(run_dir)
            .join("images")
            .join(format!("{sha}.jpg"));
        assert_eq!(fs::read(exported).unwrap(), bytes);
    }
}

#[test]
fn a8_a_value_in_a_webp_or_png_text_part_stops_the_export() {
    for bytes in [
        webp(b"\x01\x02", &[(b"XMP ", MODEL_KEY.as_bytes())]),
        webp(b"\x01\x02", &[(b"EXIF", MODEL_KEY.as_bytes())]),
        png(b"\x01\x02", Some(MODEL_KEY.as_bytes())),
    ] {
        let f = four_calls();
        let h = &f.h;
        let sha = replace_sent_image(&f, &bytes);
        let out = dest(h, "out");
        let error = h
            .service
            .export(ExportSelection::Items(vec![h.item_id]), &out)
            .unwrap_err()
            .to_string();
        assert!(
            error.ends_with(&format!("images/{sha}.jpg contains a key")),
            "{error}"
        );
        assert!(!error.contains(MODEL_KEY), "{error}");
        assert_eq!(fs::read_dir(&out).unwrap().count(), 0, "something remained");
    }
}

#[test]
fn a8_a_choice_with_no_runs_writes_nothing_and_says_so() {
    let h = Harness::new();
    finished_run(&h, h.item_id, h.session_id, local_ms(2026, 11, 10, 12, 0));
    let out = dest(&h, "out");
    let error = h
        .service
        .export(
            ExportSelection::Range {
                from: 20261111,
                to: 20261112,
            },
            &out,
        )
        .unwrap_err();
    assert!(matches!(error, psm_core::Error::NothingToExport), "{error}");
    assert_eq!(error.to_string(), "No research runs match this choice.");
    assert_eq!(
        fs::read_dir(&out).unwrap().count(),
        0,
        "something was written"
    );
    let (other_item, _) = h.new_session("A8-NO-RUNS");
    assert!(matches!(
        h.service
            .export(ExportSelection::Items(vec![other_item]), &out),
        Err(psm_core::Error::NothingToExport)
    ));
    assert_eq!(
        fs::read_dir(&out).unwrap().count(),
        0,
        "something was written"
    );
}

#[test]
fn a21_c6a_only_json_and_jpg_never_a_raw() {
    let f = four_calls();
    let h = &f.h;
    let zip = h
        .service
        .export(ExportSelection::Items(vec![h.item_id]), &dest(h, "out"))
        .unwrap();
    let raws: Vec<Vec<u8>> = f
        .mains
        .iter()
        .map(|id| fs::read(h.photo(*id).raw_path).unwrap())
        .collect();
    let name = folder_of(&zip)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let entries = zip_entries(&zip);
    assert!(!entries.is_empty());
    for entry in entries {
        assert!(
            entry.ends_with(".json") || entry.ends_with(".jpg"),
            "{entry}"
        );
        let bytes = zip_bytes(&zip, &entry);
        assert!(!raws.contains(&bytes), "{entry} is a RAW");
        assert!(entry.starts_with(&format!("{name}/")));
    }
}
