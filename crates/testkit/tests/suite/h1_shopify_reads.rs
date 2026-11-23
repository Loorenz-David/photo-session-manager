//! H1·U1 behaviour tests (`plans/H1_points_and_remote_record.md`, rows H1·C1, C4–C6): the real
//! `ShopifyAdapter` over `FakeTransport`. The outcome is the value returned and the requests the
//! transport received. No network. H1·C2 (`projected`) and H1·C3 (`must_wait`) are inline unit
//! tests of their modules.

use psm_core::{
    Error,
    domain::{publish::MediaImage, remote::CallBudget},
    ports::commerce::CommerceAdapter,
};
use psm_integrations::commerce::shopify::{
    PollConfig, ShopifyAdapter, ShopifyConfig,
    transport::{Download, HttpReply},
};
use psm_testkit::{
    clock::ManualClock,
    fake_transport::FakeTransport,
    fixtures::shopify::{Fixture, fixture},
};
use serde_json::{Value as Json, json};
use std::sync::Arc;

const TOKEN_URL: &str = "https://example.myshopify.com/admin/oauth/access_token";
const JOB: &str = "gid://shopify/Job/1";
const CDN: &str = "https://cdn.shopify.com/s/files/1/0000/0000/0000/files/";

struct Shop {
    fake: Arc<FakeTransport>,
    adapter: ShopifyAdapter,
}

/// An adapter for a placeholder shop whose token endpoint always answers the SV-0 token reply.
fn shop() -> Shop {
    let fake = Arc::new(FakeTransport::new());
    let adapter = ShopifyAdapter::new(
        ShopifyConfig {
            shop_domain: "example.myshopify.com".into(),
            api_version: "2026-10".into(),
            client_id: "psm-test-client-id".into(),
            client_secret: "SECRET-XYZ".into(),
            condition_photos_key: "custom.damage_report".into(),
        },
        fake.clone(),
        Arc::new(ManualClock::new(1_000_000)),
        PollConfig {
            interval_ms: 0,
            attempts: 5,
            retries: 3,
            retry_wait_ms: 0,
        },
    );
    fake.on_form_always(TOKEN_URL, fixture("sv0_token").reply());
    Shop { fake, adapter }
}

fn budget(available: f64, maximum: f64) -> Option<CallBudget> {
    Some(CallBudget { available, maximum })
}

/// The captured `sv4_job_poll_done.json` reply with `restoreRate` set to `0.0` (so the projection
/// stays at the reading).
fn job_done_no_restore() -> HttpReply {
    let mut reply = fixture("sv4_job_poll_done").reply();
    reply.body["extensions"]["cost"]["throttleStatus"]["restoreRate"] = json!(0.0);
    reply
}

/// One `JobPoll` query (a read the adapter makes) answered with `reply`.
fn query_with(shop: &Shop, reply: HttpReply) {
    shop.fake.on_graphql("JobPoll", None, vec![reply]);
    shop.adapter.poll_job(JOB).unwrap();
}

fn job_done_with(throttle_status: Json) -> HttpReply {
    HttpReply::new(
        200,
        json!({
            "data": { "job": { "id": JOB, "done": true } },
            "extensions": { "cost": {
                "requestedQueryCost": 1,
                "actualQueryCost": 1,
                "throttleStatus": throttle_status,
            } },
        }),
    )
}

// ---------------------------------------------------------------------------------------------
// H1·C1 the call budget is read from every answer (MC-H5.1, MC-H5.2)

#[test]
fn h1_c1a_no_call_no_budget() {
    let shop = shop();
    assert_eq!(shop.adapter.budget(), None);
}

#[test]
fn h1_c1b_budget_read_from_a_captured_answer() {
    let shop = shop();
    query_with(&shop, job_done_no_restore());
    assert_eq!(shop.adapter.budget(), budget(1999.0, 2000.0));
}

#[test]
fn h1_c1c_an_answer_without_extensions_keeps_the_reading() {
    let shop = shop();
    query_with(&shop, job_done_no_restore());
    query_with(
        &shop,
        HttpReply::new(
            200,
            json!({ "data": { "job": { "id": JOB, "done": true } } }),
        ),
    );
    assert_eq!(shop.adapter.budget(), budget(1999.0, 2000.0));
}

#[test]
fn h1_c1d_integer_numbers_are_read() {
    let shop = shop();
    query_with(
        &shop,
        job_done_with(json!({
            "maximumAvailable": 1000,
            "currentlyAvailable": 954,
            "restoreRate": 0,
        })),
    );
    assert_eq!(shop.adapter.budget(), budget(954.0, 1000.0));
}

#[test]
fn h1_c1e_the_newest_reading_wins() {
    let shop = shop();
    query_with(&shop, job_done_no_restore());
    query_with(
        &shop,
        job_done_with(json!({
            "maximumAvailable": 2000.0,
            "currentlyAvailable": 500,
            "restoreRate": 0.0,
        })),
    );
    assert_eq!(shop.adapter.budget(), budget(500.0, 2000.0));
}

#[test]
fn h1_c1f_a_throttled_answer_is_read() {
    let shop = shop();
    // Schema-grounded; not captured (shopify.dev "GraphQL Admin API rate limits": a throttled
    // reply carries `errors[].extensions.code = "THROTTLED"` beside the top-level cost block).
    shop.fake.on_graphql_always(
        "JobPoll",
        None,
        HttpReply::new(
            200,
            json!({
                "errors": [{ "message": "Throttled", "extensions": { "code": "THROTTLED" } }],
                "extensions": { "cost": {
                    "requestedQueryCost": 10,
                    "actualQueryCost": null,
                    "throttleStatus": {
                        "maximumAvailable": 2000.0,
                        "currentlyAvailable": 12,
                        "restoreRate": 0.0,
                    },
                } },
            }),
        ),
    );
    let error = shop.adapter.poll_job(JOB).unwrap_err();
    assert!(matches!(error, Error::Provider(_)), "{error:?}");
    assert_eq!(shop.adapter.budget(), budget(12.0, 2000.0));
}

// ---------------------------------------------------------------------------------------------
// H1·C4 the batched URL read (MC-H1.4, MC-H5.5)

fn ids_of(shop: &Shop) -> Vec<Vec<String>> {
    shop.fake
        .graphql("MediaUrls")
        .into_iter()
        .map(|(_, variables)| {
            variables["ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| id.as_str().unwrap().to_owned())
                .collect()
        })
        .collect()
}

fn nodes_reply(nodes: Json) -> HttpReply {
    HttpReply::new(200, json!({ "data": { "nodes": nodes } }))
}

#[test]
fn h1_c4a_urls_sizes_and_a_missing_media() {
    let shop = shop();
    let urls: Fixture = fixture("h1_media_urls");
    shop.fake.replay(&urls);
    let ids: Vec<String> = urls.variables()["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_str().unwrap().to_owned())
        .collect();
    let found = shop.adapter.media_urls(&ids).unwrap();
    assert_eq!(
        found,
        vec![
            Some(MediaImage {
                id: "gid://shopify/MediaImage/1001".into(),
                url: Some(format!("{CDN}a.jpg")),
                bytes: Some(1_843_211),
            }),
            None,
            Some(MediaImage {
                id: "gid://shopify/GenericFile/1003".into(),
                url: Some(format!("{CDN}c.jpg")),
                bytes: Some(904_112),
            }),
        ]
    );
    assert_eq!(ids_of(&shop), vec![ids]);
}

#[test]
fn h1_c4b_twenty_six_ids_take_two_requests() {
    let shop = shop();
    let ids: Vec<String> = (1..=26)
        .map(|n| format!("gid://shopify/MediaImage/{n}"))
        .collect();
    let node = |id: &str| json!({ "id": id, "image": { "url": format!("{CDN}{}.jpg", id.rsplit('/').next().unwrap()) } });
    shop.fake.on_graphql(
        "MediaUrls",
        Some(&json!({ "ids": ids[..25] })),
        vec![nodes_reply(Json::Array(
            ids[..25].iter().map(|id| node(id)).collect(),
        ))],
    );
    shop.fake.on_graphql(
        "MediaUrls",
        Some(&json!({ "ids": ids[25..] })),
        vec![nodes_reply(json!([node(&ids[25])]))],
    );
    let found = shop.adapter.media_urls(&ids).unwrap();
    let got: Vec<String> = found.into_iter().map(|m| m.unwrap().id).collect();
    assert_eq!(got, ids);
    let asked = ids_of(&shop);
    assert_eq!(asked.iter().map(Vec::len).collect::<Vec<_>>(), vec![25, 1]);
}

#[test]
fn h1_c4c_no_ids_no_request() {
    let shop = shop();
    assert_eq!(shop.adapter.media_urls(&[]).unwrap(), vec![]);
    assert!(shop.fake.requests().is_empty());
}

#[test]
fn h1_c4d_a_media_still_processing_has_no_url() {
    let shop = shop();
    let id = "gid://shopify/MediaImage/1001".to_owned();
    shop.fake.on_graphql(
        "MediaUrls",
        None,
        vec![nodes_reply(
            json!([{ "id": id, "image": null, "originalSource": null }]),
        )],
    );
    assert_eq!(
        shop.adapter.media_urls(std::slice::from_ref(&id)).unwrap(),
        vec![Some(MediaImage {
            id,
            url: None,
            bytes: None,
        })]
    );
}

#[test]
fn h1_c4e_a_short_answer_is_refused() {
    let shop = shop();
    let ids = vec![
        "gid://shopify/MediaImage/1".to_owned(),
        "gid://shopify/MediaImage/2".to_owned(),
    ];
    shop.fake.on_graphql(
        "MediaUrls",
        None,
        vec![nodes_reply(
            json!([{ "id": ids[0], "image": { "url": format!("{CDN}a.jpg") } }]),
        )],
    );
    match shop.adapter.media_urls(&ids).unwrap_err() {
        Error::Provider(text) => assert!(text.contains("1 of 2"), "{text}"),
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------------------------
// H1·C5 picture sizes in the readback (MC-H3.2, MC-H4.1, 10F M41)

fn readback_shop() -> Shop {
    let shop = shop();
    shop.fake.replay(&fixture("h1_media_readback_sizes"));
    shop
}

#[test]
fn h1_c5a_gallery_carries_sizes() {
    let shop = readback_shop();
    let readback = shop
        .adapter
        .media_readback("gid://shopify/Product/1000")
        .unwrap();
    assert_eq!(
        readback.gallery,
        vec![
            MediaImage {
                id: "gid://shopify/MediaImage/1101".into(),
                url: Some(format!("{CDN}g1.jpg")),
                bytes: Some(2_100_345),
            },
            MediaImage {
                id: "gid://shopify/MediaImage/1102".into(),
                url: Some(format!("{CDN}g2.jpg")),
                bytes: None,
            },
        ]
    );
}

#[test]
fn h1_c5b_condition_files_carry_sizes_in_the_value_order() {
    let shop = readback_shop();
    let readback = shop
        .adapter
        .media_readback("gid://shopify/Product/1000")
        .unwrap();
    assert_eq!(
        readback.condition,
        vec![
            MediaImage {
                id: "gid://shopify/MediaImage/2001".into(),
                url: Some(format!("{CDN}d1.jpg")),
                bytes: Some(1_500_111),
            },
            MediaImage {
                id: "gid://shopify/GenericFile/2002".into(),
                url: Some(format!("{CDN}d2.jpg")),
                bytes: Some(700_222),
            },
            MediaImage {
                id: "gid://shopify/MediaImage/2003".into(),
                url: None,
                bytes: None,
            },
        ]
    );
}

#[test]
fn h1_c5c_original_source_fallback_keeps_its_size() {
    let shop = shop();
    shop.fake.on_graphql(
        "MediaReadback",
        None,
        vec![HttpReply::new(
            200,
            json!({ "data": { "product": {
                "id": "gid://shopify/Product/1000",
                "media": { "nodes": [{
                    "id": "gid://shopify/MediaImage/1101",
                    "status": "READY",
                    "image": null,
                    "originalSource": { "url": format!("{CDN}g1-original.jpg"), "fileSize": 4321 },
                }] },
                "metafield": null,
            } } }),
        )],
    );
    let readback = shop
        .adapter
        .media_readback("gid://shopify/Product/1000")
        .unwrap();
    assert_eq!(
        readback.gallery,
        vec![MediaImage {
            id: "gid://shopify/MediaImage/1101".into(),
            url: Some(format!("{CDN}g1-original.jpg")),
            bytes: Some(4321),
        }]
    );
}

#[test]
fn h1_c5d_a_missing_product_is_not_found() {
    let shop = shop();
    shop.fake.on_graphql(
        "MediaReadback",
        None,
        vec![HttpReply::new(200, json!({ "data": { "product": null } }))],
    );
    let error = shop
        .adapter
        .media_readback("gid://shopify/Product/1000")
        .unwrap_err();
    assert!(matches!(error, Error::NotFound(_)), "{error:?}");
}

// ---------------------------------------------------------------------------------------------
// H1·C6 typed downloads (MC-H1.2, MC-H1.4, MC-H4.3)

fn answer(status: u16, bytes: &[u8]) -> Vec<Download> {
    vec![Download {
        status,
        bytes: bytes.to_vec(),
    }]
}

#[test]
fn h1_c6a_fetch_image_asks_for_the_width() {
    let shop = shop();
    let url = format!("{CDN}a.jpg");
    let sized = format!("{url}?width=320");
    shop.fake.on_download(&sized, answer(200, b"B"));
    assert_eq!(shop.adapter.fetch_image(&url, 320).unwrap(), b"B".to_vec());
    assert_eq!(shop.fake.downloads(), vec![sized]);
}

#[test]
fn h1_c6b_fetch_original_asks_for_the_url_unchanged() {
    let shop = shop();
    let url = format!("{CDN}a.jpg?v=17");
    shop.fake.on_download(&url, answer(200, b"B"));
    assert_eq!(shop.adapter.fetch_original(&url).unwrap(), b"B".to_vec());
    assert_eq!(shop.fake.downloads(), vec![url]);
}

#[test]
fn h1_c6c_fetch_image_404_is_gone() {
    let shop = shop();
    let url = format!("{CDN}a.jpg");
    shop.fake
        .on_download(&format!("{url}?width=320"), answer(404, b""));
    let error = shop.adapter.fetch_image(&url, 320).unwrap_err();
    assert!(matches!(error, Error::ImageGone), "{error:?}");
}

#[test]
fn h1_c6d_fetch_original_403_is_gone() {
    let shop = shop();
    let url = format!("{CDN}a.jpg");
    shop.fake.on_download(&url, answer(403, b""));
    let error = shop.adapter.fetch_original(&url).unwrap_err();
    assert!(matches!(error, Error::ImageGone), "{error:?}");
}

#[test]
fn h1_c6e_another_status_is_a_provider_error_without_the_url() {
    let shop = shop();
    let url = format!("{CDN}a.jpg");
    shop.fake
        .on_download(&format!("{url}?width=320"), answer(500, b""));
    match shop.adapter.fetch_image(&url, 320).unwrap_err() {
        Error::Provider(text) => {
            assert!(
                !text.contains("cdn.shopify.com") && !text.contains("a.jpg"),
                "{text}"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn h1_c6f_fetch_original_refuses_http() {
    let shop = shop();
    let error = shop
        .adapter
        .fetch_original("http://example.test/a.jpg")
        .unwrap_err();
    assert!(matches!(error, Error::InvalidData(_)), "{error:?}");
    assert!(shop.fake.requests().is_empty());
}
