//! R8·U2 (rows A16·C1–C6): `VisualSearchProvider::search` on the SerpApi Lens adapter over
//! `FakeResearchHttp` replaying R1's captures (`crates/research/fixtures/serpapi_lens/`). A
//! **derived** reply is built here from the capture its row names (envelope kept, one named part
//! replaced).

use std::sync::Arc;

use psm_core::ports::secrets::SecretStore;
use psm_research::{
    adapters::{http::HttpReply, visual_search::SerpApiLens},
    domain::{ErrorClass, FailureKind},
    fakes::{FakeResearchHttp, reply_from_capture},
    files::sha256_hex,
    images::ResearchImage,
    ports::{SearchCandidate, VisualSearchProvider},
    rules::calls::classify,
};
use serde_json::{Value as Json, json};

use crate::{
    adapter_openai::capture,
    harness::{Harness, SEARCH_KEY},
    stages::full_script,
};

const UPLOAD: &str = "https://serpapi.com/image";
const SEARCH: &str = "https://serpapi.com/search.json";

fn search_capture() -> Json {
    capture("serpapi_lens/search_cut_to_3.json")
}

/// A harness, one rendered research copy of 1024 and a fake answering the upload with
/// `upload.json` and the search with `search`.
fn setup(search: HttpReply) -> (Harness, ResearchImage, Arc<FakeResearchHttp>) {
    let h = Harness::new();
    let photo = h.add_main_photo_with_preview(h.session_id);
    let run = h.running_run(&h.config());
    let image = h.images.render(photo, run, 1024).unwrap();
    let http = Arc::new(FakeResearchHttp::default());
    http.once(
        "POST",
        UPLOAD,
        Ok(reply_from_capture(&capture("serpapi_lens/upload.json"))),
    );
    http.once("GET", SEARCH, Ok(search));
    (h, image, http)
}

fn adapter(h: &Harness, http: &Arc<FakeResearchHttp>) -> SerpApiLens {
    SerpApiLens::new(http.clone(), h.app.secrets.clone() as Arc<dyn SecretStore>)
}

#[test]
fn a16_c1a_candidates_in_the_reply_order() {
    let (h, image, http) = setup(reply_from_capture(&search_capture()));
    let reply = adapter(&h, &http).search(&image).unwrap();
    let candidate = |url: &str, rank: u32, title: &str| SearchCandidate {
        url: url.into(),
        rank,
        title: title.into(),
    };
    assert_eq!(
        reply.candidates,
        vec![
            candidate(
                "https://auctionet.com/en/5091518-poul-hundevad-extendable-dining-table-teak-1960s",
                1,
                "POUL HUNDEVAD. Extendable dining table, teak, 1960s ...",
            ),
            candidate(
                "https://beyovintage.se/en/products/swedish-side-table-in-teak-by-alberts-in-tibro",
                2,
                "Vintage Swedish side table in teak by Alberts in Tibro buy ...",
            ),
            candidate(
                "https://www.pamono.se/small-vintage-extendable-dining-table-in-teak-1950",
                3,
                "Small Vintage Extendable Dining Table in Teak, 1950 for sale ...",
            ),
        ]
    );
    // The search asked for the image id the upload returned.
    let upload_id = capture("serpapi_lens/upload.json")["response"]["body"]["image_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let search_url = &http.requests()[1].url;
    assert!(
        search_url.contains(&format!("image_id={upload_id}&")),
        "{search_url}"
    );
    assert!(search_url.contains("engine=google_lens&"), "{search_url}");
    assert!(
        search_url.contains("&type=all&hl=en&country=se&"),
        "{search_url}"
    );
}

#[test]
fn a16_c1b_no_visual_matches_is_no_candidate() {
    // Derived, not captured: the search capture with (1) an empty list, (2) the key removed.
    let mut empty = search_capture();
    empty["response"]["body"]["visual_matches"] = json!([]);
    let mut absent = search_capture();
    absent["response"]["body"]
        .as_object_mut()
        .unwrap()
        .remove("visual_matches");
    for derived in [empty, absent] {
        let (h, image, http) = setup(reply_from_capture(&derived));
        let reply = adapter(&h, &http).search(&image).unwrap();
        assert!(reply.candidates.is_empty());
        assert_eq!(reply.usage, Some(vec![("image_search_call".to_owned(), 1)]));
    }
}

#[test]
fn a16_c2a_one_search_credit_for_upload_and_search() {
    let (h, image, http) = setup(reply_from_capture(&search_capture()));
    let reply = adapter(&h, &http).search(&image).unwrap();
    assert_eq!(reply.usage, Some(vec![("image_search_call".to_owned(), 1)]));
}

#[test]
fn a16_c3a_api_key_is_redacted_in_the_exchange() {
    let (h, image, http) = setup(reply_from_capture(&search_capture()));
    let reply = adapter(&h, &http).search(&image).unwrap();
    let request = &reply.exchange.request;
    assert_eq!(
        request["upload"]["body"]["multipart"]["api_key"],
        json!("<redacted>")
    );
    let url = request["search"]["url"].as_str().unwrap();
    assert!(url.ends_with("&api_key=<redacted>"), "{url}");
    let shown = serde_json::to_string(&reply.exchange).unwrap();
    assert!(!shown.contains(SEARCH_KEY), "{shown}");
    // The key itself went out, read from the secrets port.
    let sent = http.requests();
    assert_eq!(
        sent[0].body.part("api_key").unwrap().data,
        SEARCH_KEY.as_bytes()
    );
    assert!(sent[1].url.ends_with(&format!("&api_key={SEARCH_KEY}")));
}

#[test]
fn a16_c4a_upload_too_large_is_a_request_error() {
    let h = Harness::new();
    let photo = h.add_main_photo_with_preview(h.session_id);
    let run = h.running_run(&h.config());
    let image = h.images.render(photo, run, 1024).unwrap();
    let http = Arc::new(FakeResearchHttp::default());
    http.once(
        "POST",
        UPLOAD,
        Ok(reply_from_capture(&capture(
            "serpapi_lens/error_upload_too_large.json",
        ))),
    );
    let failure = adapter(&h, &http).search(&image).unwrap_err();
    assert_eq!(
        (failure.status, failure.kind, failure.quota),
        (Some(400), FailureKind::Status, false)
    );
    assert_eq!(classify(&failure), ErrorClass::Request);
    assert_eq!(http.requests().len(), 1, "a search was sent");
}

#[test]
fn a16_c5a_the_stored_copy_is_uploaded_unchanged() {
    let (h, image, http) = setup(reply_from_capture(&search_capture()));
    adapter(&h, &http).search(&image).unwrap();
    let sent = http.requests()[0].body.part("image").unwrap().data.clone();
    let stored = std::fs::read(image.path()).unwrap();
    assert_eq!(sha256_hex(&sent), sha256_hex(&stored));
    assert_eq!(sha256_hex(&sent), image.sha256());
}

#[test]
fn a16_c6a_full_run_reads_the_lens_results_in_site_order() {
    let h = Harness::chair();
    let (mains, _condition) = full_script(&h);
    let http = Arc::new(FakeResearchHttp::default());
    http.once(
        "POST",
        UPLOAD,
        Ok(reply_from_capture(&capture("serpapi_lens/upload.json"))),
    );
    http.once("GET", SEARCH, Ok(reply_from_capture(&search_capture())));
    let search = Arc::new(adapter(&h, &http));
    let service = h.service_with(|deps| {
        deps.providers.search = Some(search as Arc<dyn VisualSearchProvider>);
    });
    let mut config = h.stage_config();
    config.shop_hosts = vec!["beyovintage.se".into()];
    config.trusted_sites = vec!["auctionet.com".into()];
    config.own_listing_url = None;
    config.own_listing_path_tail = Vec::new();
    let run = h.start_run(&config);
    service.execute(run).unwrap();
    let read: Vec<String> = h
        .reader
        .requests()
        .into_iter()
        .filter_map(|c| c.url)
        .collect();
    assert_eq!(
        read,
        vec![
            "https://beyovintage.se/en/products/swedish-side-table-in-teak-by-alberts-in-tibro",
            "https://auctionet.com/en/5091518-poul-hundevad-extendable-dining-table-teak-1960s",
            "https://www.pamono.se/small-vintage-extendable-dining-table-in-teak-1950",
        ]
    );
    let search_call = h
        .calls(run)
        .into_iter()
        .find(|c| c.stage == "find")
        .unwrap();
    assert_eq!(search_call.provider, "serpapi_lens");
    let outbound: Vec<(i64, i64)> = h
        .outbound(run)
        .into_iter()
        .filter(|o| o.call_id == search_call.id)
        .map(|o| (o.call_id, o.photo_id))
        .collect();
    assert_eq!(outbound, vec![(search_call.id, mains[0])]);
}
