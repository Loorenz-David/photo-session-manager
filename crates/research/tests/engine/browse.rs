//! Owner correction A12: the "Search this photo" window's research parts on a real store with the
//! crate's fakes — "Ask AI…" on selected text (suggestions only, the page as the source, the day
//! limit, the call in the record and the export, never one of the session's research runs), the
//! fallback matches, and the picture given to Google Lens.

use psm_core::domain::{
    attribute::{AiConfidence, AiStatus, Provenance},
    photo::Classification,
};
use psm_infra::imaging::ImageOpsImpl;
use psm_research::{
    api::ExportSelection,
    browse::{AskRequest, MAX_SELECTION_CHARS, lens_image},
    domain::{RunTrigger, Stage},
    ports::SearchCandidate,
    store as rstore,
};
use serde_json::json;

use crate::harness::*;

const PAGE: &str = "https://e1.example/sideboards/kai-k-teak";

fn ask(h: &Harness, text: &str, instruction: &str) -> psm_core::Result<psm_research::browse::AskAnswer> {
    h.service.browse_ask(&AskRequest {
        session_id: h.session_id,
        page_url: PAGE.to_owned(),
        page_title: "Kai K teak sideboard".to_owned(),
        text: text.to_owned(),
        instruction: instruction.to_owned(),
    })
}

fn script_answer(h: &Harness) {
    h.model.reply(
        Stage::Read,
        "browse",
        json!({"keys": {
            MAKER: {"stated": "designed by Kai Kristiansen", "value": "Kai Kristiansen"},
            WOOD: {"stated": "solid teak", "value": "Teak"},
        }}),
    );
}

#[test]
fn a12_ask_ai_writes_suggestions_with_the_page_as_source_and_applies_nothing() {
    let h = Harness::new();
    script_answer(&h);
    let answer = ask(&h, "Sideboard in solid teak, designed by Kai Kristiansen", "who made it").unwrap();
    let mut keys: Vec<&str> = answer.suggested.iter().map(|s| s.key.as_str()).collect();
    keys.sort();
    assert_eq!(keys, vec![MAKER, WOOD]);
    for key in [MAKER, WOOD] {
        let row = h.row(key).unwrap();
        assert_eq!(row.provenance, Provenance::Ai);
        assert_eq!(row.ai_status, Some(AiStatus::Suggested), "never applied");
        assert_eq!(row.ai_confidence, Some(AiConfidence::Medium));
        assert_eq!(row.ai_sources, Some(vec![PAGE.to_owned()]));
        assert!(row.ai_reason.as_deref().unwrap().contains("e1.example"));
    }
    assert_eq!(h.row(MAKER).unwrap().value, json!("Kai Kristiansen").to_string());
    // The request carried the selected text, the instruction and the page, as a Read call.
    let request = h.model.requests().pop().unwrap();
    assert_eq!(request.stage, Some(Stage::Read));
    assert_eq!(request.input["selected_text"], json!("Sideboard in solid teak, designed by Kai Kristiansen"));
    assert_eq!(request.input["instruction"], json!("who made it"));
    assert_eq!(request.input["page"]["url"], json!(PAGE));
    assert!(request.images.is_empty(), "no photo goes with Ask AI");
}

#[test]
fn a12_ask_ai_is_a_recorded_browse_run_outside_the_sessions_research() {
    let h = Harness::new();
    script_answer(&h);
    ask(&h, "solid teak", "").unwrap();
    // Not one of the session's research runs: the panel, the overview and Arranged never see it.
    assert!(h.runs(h.session_id).is_empty());
    let status = h.service.status(h.session_id).unwrap();
    assert!(status.run.is_none());
    assert!(h.service.overview(Some(h.session_id)).unwrap().iter().all(|s| s.state != "researching"));
    // But it is recorded: one browse run with one answered call and a cost.
    let all = h.query(|c| rstore::all_runs(c, Some(h.session_id)).unwrap());
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].trigger, RunTrigger::Browse);
    let calls = h.calls(all[0].id);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].stage, "read");
    assert_eq!(calls[0].unit_key, "browse");
    assert!(all[0].cost_micro_usd > 0);
    // And the export has it.
    let dest = h.app.dir.path().join("export-a12");
    let zip = h
        .service
        .export(ExportSelection::Items(vec![h.item_id]), &dest)
        .unwrap();
    assert!(zip.is_file());
    let folder = zip.with_extension("");
    let names: Vec<String> = walk(&folder);
    assert!(names.iter().any(|n| n.contains(&format!("run-{}", all[0].id))), "{names:?}");
}

fn walk(root: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            out.push(path.to_string_lossy().into_owned());
            if path.is_dir() {
                stack.push(path);
            }
        }
    }
    out
}

#[test]
fn a12_ask_ai_stops_at_the_day_limit() {
    let h = Harness::new();
    // Each call costs the 0.001 USD fallback; the day allows one.
    h.patch_settings(json!({"max_cost_per_day": "0.001"}));
    script_answer(&h);
    ask(&h, "solid teak", "").unwrap();
    let calls_before = h.model.requests().len();
    let refused = ask(&h, "solid teak", "").unwrap_err();
    assert!(refused.to_string().contains("Day limit reached"), "{refused}");
    assert_eq!(h.model.requests().len(), calls_before, "no call was sent");
}

#[test]
fn a12_ask_ai_refuses_empty_or_oversize_text_and_non_web_pages() {
    let h = Harness::new();
    assert!(ask(&h, "   ", "").is_err());
    let long = "x".repeat(MAX_SELECTION_CHARS + 1);
    assert!(ask(&h, &long, "").is_err());
    let refused = h.service.browse_ask(&AskRequest {
        session_id: h.session_id,
        page_url: "tauri://localhost/index.html".into(),
        page_title: String::new(),
        text: "teak".into(),
        instruction: String::new(),
    });
    assert!(refused.is_err());
    assert!(h.model.requests().is_empty(), "nothing was sent");
}

#[test]
fn a12_ask_ai_never_writes_over_a_human_value() {
    let h = Harness::new();
    h.app
        .attributes(Some(INTEGRATION))
        .set_value(h.item_id, MAKER, json!("Arne Vodder"), Provenance::Human)
        .unwrap();
    script_answer(&h);
    let answer = ask(&h, "designed by Kai Kristiansen", "").unwrap();
    assert!(answer.suggested.iter().all(|s| s.key != MAKER));
    let row = h.row(MAKER).unwrap();
    assert_eq!(row.provenance, Provenance::Human);
    assert_eq!(row.value, json!("Arne Vodder").to_string());
}

#[test]
fn a12_fallback_matches_come_from_research_already_done() {
    let h = Harness::new();
    assert!(h.service.browse_matches(h.session_id).unwrap().is_empty());
    let run = h.insert_run(psm_research::domain::RunState::Finished, None, None);
    let response = json!({"body": {"upload": {}, "search": {"status": 200, "headers": {}, "body": {
        "visual_matches": [
            {"link": "https://e1.example/a", "title": "Teak sideboard", "source": "E1 Shop",
             "thumbnail": "https://img.example/a.jpg"},
            {"title": "no link"},
            {"link": "https://e2.example/b", "title": "Another"}
        ]}}}});
    h.query(|c| {
        c.execute(
            "INSERT INTO research_calls(run_id,stage,unit_key,attempt,provider,model,effort,state,\
             request,response,started_at) VALUES (?1,'find','cover',1,'fake-search','','','answered','{}',?2,?3)",
            rusqlite::params![run, response.to_string(), T0],
        )
        .unwrap()
    });
    let matches = h.service.browse_matches(h.session_id).unwrap();
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].url, "https://e1.example/a");
    assert_eq!(matches[0].source, "E1 Shop");
    assert_eq!(matches[0].thumbnail.as_deref(), Some("https://img.example/a.jpg"));
    assert_eq!(matches[1].source, "e2.example");
}

#[test]
fn a12_a_new_search_of_the_pinned_photo_is_counted_and_returns_its_matches() {
    let h = Harness::new();
    let photo = h.add_main_photo_with_preview(h.session_id);
    h.search.set_results(vec![SearchCandidate {
        url: "https://e2.example/x".into(),
        rank: 1,
        title: "X".into(),
    }]);
    let matches = h.service.browse_search(h.session_id, photo).unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].url, "https://e2.example/x");
    let all = h.query(|c| rstore::all_runs(c, Some(h.session_id)).unwrap());
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].trigger, RunTrigger::Browse);
    assert_eq!(h.outbound(all[0].id).len(), 1, "the photo sent is recorded");
    assert!(h.runs(h.session_id).is_empty());
}

#[test]
fn a12_the_lens_picture_is_the_framed_photo_and_only_a_photo_of_the_session() {
    let h = Harness::new();
    let photo = h.add_main_photo_with_preview(h.session_id);
    let bytes = lens_image(h.store(), &ImageOpsImpl, &h.app.app.paths.root, h.session_id, photo).unwrap();
    assert_eq!(&bytes[..2], &[0xFF, 0xD8], "a JPEG");
    let path = h.app.dir.path().join("lens-a12.jpg");
    std::fs::write(&path, &bytes).unwrap();
    let frame = psm_core::ports::images::Imaging::dimensions(&ImageOpsImpl, &path)
        .unwrap()
        .unwrap();
    assert!(frame.w.max(frame.h) <= 1_600, "{frame:?}");
    // Not a photo of this session.
    let (_, other_session) = h.new_session("A12-2");
    assert!(lens_image(h.store(), &ImageOpsImpl, &h.app.app.paths.root, other_session, photo).is_err());
    // A deleted photo is refused.
    h.classify(photo, Classification::Deleted);
    assert!(lens_image(h.store(), &ImageOpsImpl, &h.app.app.paths.root, h.session_id, photo).is_err());
    // Without an adjusted preview, the camera's original preview is used.
    let raw_only = h.add_photo_without_preview(h.session_id, Classification::Main);
    assert!(lens_image(h.store(), &ImageOpsImpl, &h.app.app.paths.root, h.session_id, raw_only).is_err());
    h.give_original_preview(raw_only);
    let bytes = lens_image(h.store(), &ImageOpsImpl, &h.app.app.paths.root, h.session_id, raw_only).unwrap();
    assert_eq!(&bytes[..2], &[0xFF, 0xD8]);
}
