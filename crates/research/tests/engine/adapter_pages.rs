//! R8·U3 (rows A17·C1–C3, C7(a)): `PageReader::read` on the plain-HTTP reader over
//! `FakeResearchHttp` with a recording waiter, replaying R1's page captures
//! (`crates/research/fixtures/pages/`); and `adapters::build`'s selection. A **derived** reply is
//! built here from the capture its row names (envelope kept).

use std::sync::{Arc, Mutex};

use psm_research::{
    adapters::{
        self,
        http::{HttpFailure, HttpReply},
        page_reader::HttpPageReader,
    },
    config::{ProviderIds, ResearchConfig, ResearchSettings},
    domain::{FailureKind, RunState},
    fakes::{FakeResearchHttp, reply_from_capture},
    gateway::Waiter,
    ports::{PageReader, PageReply},
    shop::ShopFacts,
};
use psm_testkit::{fake_secrets::FakeSecrets, fixtures::synthetic_jpeg::SyntheticJpeg};
use serde_json::json;

use crate::{
    adapter_openai::capture,
    harness::Harness,
    stages::{P_E1, P_E2, P_UNTRUSTED, full_script},
};

const SHOP_PAGE: &str = "https://beyovintage.se/en/products/swedish-bookshelf-in-oak-5";
/// The `og:image` of `pages/shop_listing_page.json`.
const OG_IMAGE: &str =
    "http://beyovintage.se/cdn/shop/files/1CaptureOneCatalog3275.jpg?v=1762945395";

fn shop_page() -> HttpReply {
    reply_from_capture(&capture("pages/shop_listing_page.json"))
}

fn jpeg() -> Vec<u8> {
    SyntheticJpeg::rgb(32, 24, [200, 120, 40])
}

/// Each wait the reader asked for, with the number of requests sent before it.
type Waits = Arc<Mutex<Vec<(u64, usize)>>>;

fn reader(http: &Arc<FakeResearchHttp>, timeout_s: u32) -> (HttpPageReader, Waits) {
    let waits: Waits = Arc::default();
    let (log, seen) = (waits.clone(), http.clone());
    let wait: Waiter = Arc::new(move |ms| log.lock().unwrap().push((ms, seen.requests().len())));
    (HttpPageReader::new(http.clone(), wait, timeout_s), waits)
}

fn page_fetch() -> Option<Vec<(String, i64)>> {
    Some(vec![("page_fetch".to_owned(), 1)])
}

fn host_of(url: &str) -> String {
    url::Url::parse(url).unwrap().host_str().unwrap().to_owned()
}

fn moved(to: &str) -> HttpReply {
    HttpReply {
        status: 301,
        headers: vec![("location".into(), to.into())],
        body: Vec::new(),
    }
}

/// Reads `SHOP_PAGE` answered by `pages/shop_listing_page.json`, its `og:image` by test bytes.
fn shop_read() -> (PageReply, Arc<FakeResearchHttp>, Waits) {
    let http = Arc::new(FakeResearchHttp::default());
    http.once("GET", SHOP_PAGE, Ok(shop_page()));
    http.always(
        "GET",
        OG_IMAGE,
        Ok(HttpReply {
            status: 200,
            headers: vec![("content-type".into(), "image/jpeg".into())],
            body: jpeg(),
        }),
    );
    let (reader, waits) = reader(&http, 60);
    let reply = reader.read(SHOP_PAGE).unwrap();
    (reply, http, waits)
}

#[test]
fn a17_c1a_a_shop_page_is_read_by_download() {
    let (reply, http, waits) = shop_read();
    assert!(reply.obtained);
    assert_eq!(reply.title, "Swedish bookshelf in oak");
    assert!(
        reply
            .text
            .contains("Bookshelf manufactured in Sweden in the 1950s-60s"),
        "{}",
        reply.text
    );
    assert!(reply.images.contains(&jpeg()));
    let requests = http.requests();
    assert_eq!(requests[0].url, SHOP_PAGE);
    assert!(requests.iter().skip(1).any(|r| r.url == OG_IMAGE));
    // A 2 000 ms wait before each image request to beyovintage.se, none before the page.
    let waits = waits.lock().unwrap().clone();
    let image_requests: Vec<usize> = (1..requests.len())
        .filter(|&i| host_of(&requests[i].url) == "beyovintage.se")
        .collect();
    assert!(!image_requests.is_empty());
    assert_eq!(
        waits,
        image_requests
            .iter()
            .map(|&i| (2_000, i))
            .collect::<Vec<_>>()
    );
}

#[test]
fn a17_c1b_redirects_are_followed_up_to_five_hops() {
    // Derived (HTTP semantics; no redirect was recorded, V8).
    let http = Arc::new(FakeResearchHttp::default());
    http.once(
        "GET",
        "https://a.example/x",
        Ok(moved("https://b.example/y")),
    );
    http.once("GET", "https://b.example/y", Ok(shop_page()));
    let (r, _) = reader(&http, 60);
    let reply = r.read("https://a.example/x").unwrap();
    assert!(reply.obtained);
    assert_eq!(reply.final_url.as_deref(), Some("https://b.example/y"));

    let chain = |hops: usize| {
        let http = Arc::new(FakeResearchHttp::default());
        for i in 0..hops {
            http.once(
                "GET",
                &format!("https://a.example/{i}"),
                Ok(moved(&format!("https://a.example/{}", i + 1))),
            );
        }
        http.once("GET", &format!("https://a.example/{hops}"), Ok(shop_page()));
        let (r, _) = reader(&http, 60);
        r.read("https://a.example/0").unwrap()
    };
    let five = chain(5);
    assert!(five.obtained);
    assert_eq!(five.final_url.as_deref(), Some("https://a.example/5"));
    assert!(!chain(6).obtained);
}

#[test]
fn a17_c2a_one_page_fetch_per_read() {
    let (reply, http, _) = shop_read();
    assert!(http.requests().len() > 1, "images were requested too");
    assert_eq!(reply.usage, page_fetch());
}

#[test]
fn a17_c3a_a_refused_page_is_could_not_be_read() {
    let http = Arc::new(FakeResearchHttp::default());
    let etsy = capture("pages/www_etsy_com.json");
    let url = etsy["request"]["url"].as_str().unwrap().to_owned();
    http.once("GET", &url, Ok(reply_from_capture(&etsy)));
    let (r, _) = reader(&http, 60);
    let reply = r.read(&url).unwrap();
    assert!(!reply.obtained);
    assert_eq!(reply.usage, page_fetch());
}

/// The full-run script with this reader in place of the fake one; `script` answers the pages.
fn full_run_with_reader(script: impl FnOnce(&FakeResearchHttp), max_pages: u32) -> (Harness, i64) {
    let h = Harness::chair();
    full_script(&h);
    let http = Arc::new(FakeResearchHttp::default());
    script(&http);
    let (reader, _) = reader(&http, 60);
    let reader = Arc::new(reader);
    let service =
        h.service_with(|deps| deps.providers.reader = Some(reader as Arc<dyn PageReader>));
    let mut config = h.stage_config();
    config.max_pages = max_pages;
    let run = h.start_run(&config);
    service.execute(run).unwrap();
    (h, run)
}

/// `(summary, has a title)` of each `read` step that names a page.
fn read_steps(h: &Harness, run: i64) -> Vec<(String, Option<String>, bool)> {
    h.steps(run)
        .into_iter()
        .filter(|s| s.kind == "read")
        .map(|s| {
            let titled = s.source_title.as_deref().is_some_and(|t| !t.is_empty());
            (s.summary, s.source_url, titled)
        })
        .collect()
}

#[test]
fn a17_c3a_full_run_skips_the_refused_page() {
    let etsy = reply_from_capture(&capture("pages/www_etsy_com.json"));
    let (h, run) = full_run_with_reader(
        |http| {
            // Reading order: e1, e2, then the untrusted page.
            http.once("GET", P_E1, Ok(etsy));
            http.once("GET", P_E2, Ok(shop_page()));
            http.once("GET", P_UNTRUSTED, Ok(shop_page()));
        },
        2,
    );
    let steps = read_steps(&h, run);
    assert_eq!(steps.len(), 3, "{steps:?}");
    assert_eq!(
        (steps[0].0.as_str(), steps[0].1.as_deref(), steps[0].2),
        ("could not be read", Some(P_E1), false)
    );
    assert!(steps[1].2 && steps[2].2, "{steps:?}");
    assert_eq!(steps[1].0, "Read Swedish bookshelf in oak");
    assert_eq!(h.run_row(run).state, RunState::Finished);
}

#[test]
fn a17_c3b_throttled_shop_waits_once_then_reads() {
    let throttled = reply_from_capture(&capture("pages/shop_429_throttled.json"));
    let http = Arc::new(FakeResearchHttp::default());
    http.once("GET", SHOP_PAGE, Ok(throttled.clone()));
    http.once("GET", SHOP_PAGE, Ok(shop_page()));
    let (r, waits) = reader(&http, 60);
    let reply = r.read(SHOP_PAGE).unwrap();
    assert!(reply.obtained);
    assert_eq!(reply.usage, page_fetch());
    let waits = waits.lock().unwrap().clone();
    let requests = http.requests();
    assert_eq!(
        waits.iter().filter(|w| w.0 == 60_000).collect::<Vec<_>>(),
        vec![&(60_000, 1)],
        "one 60 s wait, before the second page request"
    );
    assert_eq!(requests[1].url, SHOP_PAGE);
    for (ms, before) in waits.iter().filter(|w| w.0 != 60_000) {
        assert_eq!(*ms, 2_000);
        assert!(*before >= 2, "a pace before a page request");
        assert_ne!(
            requests[*before].url, SHOP_PAGE,
            "a pace before a page request"
        );
    }

    let http = Arc::new(FakeResearchHttp::default());
    http.once("GET", SHOP_PAGE, Ok(throttled.clone()));
    http.once("GET", SHOP_PAGE, Ok(throttled));
    let (r, waits) = reader(&http, 60);
    let reply = r.read(SHOP_PAGE).unwrap();
    assert!(!reply.obtained);
    assert_eq!(*waits.lock().unwrap(), vec![(60_000, 1)]);
}

#[test]
fn a17_c3c_transport_failures_are_could_not_be_read() {
    let failure = |sent: bool, kind: FailureKind, blocked: bool| HttpFailure {
        sent,
        kind,
        text: "scripted".into(),
        blocked,
    };
    let cases = [
        (failure(true, FailureKind::Timeout, false), page_fetch()),
        (failure(true, FailureKind::Reset, false), page_fetch()),
        (
            failure(false, FailureKind::Connect, false),
            Some(Vec::new()),
        ),
        (failure(false, FailureKind::Connect, true), Some(Vec::new())),
    ];
    for (scripted, usage) in cases {
        let http = Arc::new(FakeResearchHttp::default());
        http.once("GET", SHOP_PAGE, Err(scripted.clone()));
        let (r, _) = reader(&http, 60);
        let reply = r.read(SHOP_PAGE).expect("read never returns Err");
        assert!(!reply.obtained, "{scripted:?}");
        assert_eq!(reply.usage, usage, "{scripted:?}");
    }
}

#[test]
fn a17_c3d_a_dead_listing_domain_does_not_hold_the_run() {
    let dead = HttpFailure {
        sent: false,
        kind: FailureKind::Connect,
        text: "dns error".into(),
        blocked: false,
    };
    let (h, run) = full_run_with_reader(
        |http| {
            http.once("GET", P_E1, Ok(shop_page()));
            http.once("GET", P_E2, Err(dead));
            http.once("GET", P_UNTRUSTED, Ok(shop_page()));
        },
        10,
    );
    let steps = read_steps(&h, run);
    assert_eq!(steps.len(), 3, "{steps:?}");
    assert_eq!(
        (steps[1].0.as_str(), steps[1].1.as_deref(), steps[1].2),
        ("could not be read", Some(P_E2), false)
    );
    assert!(steps[0].2 && steps[2].2, "{steps:?}");
    assert_eq!(h.run_row(run).state, RunState::Finished);
}

#[test]
fn a17_c7a_build_selects_the_default_adapters() {
    let wait: Waiter = Arc::new(|_| {});
    let files = Harness::new().files();
    let ids = |providers: &psm_research::ports::Providers| {
        (
            providers.model.as_ref().map(|p| p.id()),
            providers.search.as_ref().map(|p| p.id()),
            providers.reader.as_ref().map(|p| p.id()),
        )
    };
    let all = (
        Some("openai".to_owned()),
        Some("serpapi_lens".to_owned()),
        Some("page_http".to_owned()),
    );
    let blank = ResearchSettings::from_slot(Some(&json!({
        "providers": {"model": "", "visual_search": "", "page_reader": ""}
    })));
    for settings in [ResearchSettings::default(), blank] {
        let built = adapters::build(
            &settings,
            Arc::new(FakeSecrets::default()),
            files.clone(),
            wait.clone(),
        );
        assert_eq!(ids(&built), all);
        let snapshot = ResearchConfig::from_settings(&settings, &ShopFacts::default());
        assert_eq!(
            snapshot.providers,
            ProviderIds {
                model: "openai".into(),
                visual_search: "serpapi_lens".into(),
                page_reader: "page_http".into(),
            }
        );
    }
    let mut unknown = ResearchSettings::default();
    unknown.providers.model = "nope".into();
    let built = adapters::build(&unknown, Arc::new(FakeSecrets::default()), files, wait);
    assert_eq!(
        ids(&built),
        (None, Some("serpapi_lens".into()), Some("page_http".into()))
    );
}

/// R9·U1 task 0 (R8 review): a page in windows-1252, by the reply's `charset` and by its own
/// `<meta charset>`, reads its non-ASCII letters right.
#[test]
fn r9_t0a_a_windows_1252_page_reads_mork() {
    let url = "https://dealer.example/byra";
    let body =
        b"<html><head><title>Byr\xE5</title></head><body><p>F\xE4rg: m\xF6rk ek</p></body></html>";
    for (content_type, prefix) in [
        ("text/html; charset=windows-1252", &b""[..]),
        ("text/html", &b"<meta charset=\"ISO-8859-1\">"[..]),
    ] {
        let mut page = prefix.to_vec();
        page.extend_from_slice(body);
        let http = Arc::new(FakeResearchHttp::default());
        http.once(
            "GET",
            url,
            Ok(HttpReply {
                status: 200,
                headers: vec![("content-type".into(), content_type.into())],
                body: page,
            }),
        );
        let (r, _) = reader(&http, 60);
        let reply = r.read(url).unwrap();
        assert!(reply.obtained, "{content_type}");
        assert!(
            reply.text.contains("mörk"),
            "{content_type}: {}",
            reply.text
        );
        assert_eq!(reply.title, "Byrå", "{content_type}");
    }
}

/// R9·U1 task 0: a reply whose `content-type` is not HTML is "could not be read".
#[test]
fn r9_t0b_a_pdf_reply_is_not_obtained() {
    let url = "https://dealer.example/catalogue.pdf";
    let http = Arc::new(FakeResearchHttp::default());
    http.once(
        "GET",
        url,
        Ok(HttpReply {
            status: 200,
            headers: vec![("content-type".into(), "application/pdf".into())],
            body: b"<html><body><p>Designer: Hans Wegner</p></body></html>".to_vec(),
        }),
    );
    let (r, _) = reader(&http, 60);
    let reply = r.read(url).unwrap();
    assert!(!reply.obtained);
    assert_eq!(reply.text, "");
    assert_eq!(reply.usage, page_fetch());
}

/// R9·U1 task 0: each added range is refused by the `PublicOnly` guard; public neighbours are not.
#[test]
fn r9_t0c_added_address_ranges_are_refused() {
    use psm_research::adapters::http::is_public;
    use std::net::IpAddr;
    let refused = [
        "100.64.0.1",
        "100.127.255.254",
        "192.0.0.8",
        "198.18.0.1",
        "198.19.255.255",
        "224.0.0.1",
        "239.255.255.250",
        "ff02::1",
        "ff0e::fb",
        "fec0::1",
        "64:ff9b::a00:1",
        "64:ff9b::808:808",
        "2002:c000:0204::1",
        "2002:0808:0808::1",
    ];
    for text in refused {
        let ip: IpAddr = text.parse().unwrap();
        assert!(!is_public(ip), "{text} must be refused");
    }
    let public = [
        "100.63.255.255",
        "100.128.0.1",
        "192.0.1.1",
        "198.17.255.255",
        "198.20.0.1",
        "223.255.255.254",
        "8.8.8.8",
        "2a00:1450:4001::1",
        "2001:db9::1",
    ];
    for text in public {
        let ip: IpAddr = text.parse().unwrap();
        assert!(is_public(ip), "{text} must pass");
    }
}
