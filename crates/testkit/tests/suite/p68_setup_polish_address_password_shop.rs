//! P68 behaviour tests (plan `P68_setup_polish_address_password_shop`): this Mac's address for
//! the camera and the "address changed" sentence, the easy FTP password, the cleaned shop
//! settings. One test per criterion row with a backend boundary; the UI rows (C4) live in
//! `apps/desktop/src/features/settings/__tests__/p68_setup_polish.test.tsx`.

use psm_core::{
    Error,
    domain::camera::secrets,
    mechanisms::secrets::FTP_PASSWORD_ALPHABET,
    ports::{commerce::CommerceAdapter, secrets::SecretStore},
    services::camera::CameraService,
};
use psm_integrations::commerce::shopify::{PollConfig, ShopifyAdapter, ShopifyConfig};
use psm_testkit::{TestApp, clock::ManualClock, fake_transport::FakeTransport};
use serde_json::json;
use std::{
    fs,
    net::IpAddr,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

const SENTENCE_B: &str = "This Mac's address changed to 192.168.1.20 — set Host Name on the camera";

fn settle(a: &TestApp, what: &str, pred: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(5);
    while Instant::now() < end {
        a.tick(0).unwrap();
        if pred() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("{what} did not settle");
}

fn health_detail(a: &TestApp, code: &str) -> Option<String> {
    a.app
        .health
        .report("", true, true)
        .items
        .into_iter()
        .find(|i| i.code == code && !i.ok)
        .map(|i| i.detail)
}

fn network(a: &TestApp, addresses: &[&str]) {
    let addresses: Vec<IpAddr> = addresses.iter().map(|a| a.parse().unwrap()).collect();
    a.camera_runtime()
        .set_network_addresses(Arc::new(move || addresses.clone()));
}

/// C1(a): the camera uploads to A (the receiver's accepted connection); the Mac's address then
/// becomes B: step 3's address is B and Health (which the header shows) carries the sentence.
#[test]
fn c1a_address_shown_and_change_named() {
    let a = TestApp::with_camera_script(include_str!(
        "../../src/fixtures/sidecar_scripts/ready_wifi.txt"
    ))
    .unwrap();
    settle(&a, "the receiver", || {
        a.camera_runtime().receiver_port().is_some()
    });
    // A = the address the receiver is reached on in the suite (loopback).
    network(&a, &["127.0.0.1"]);
    let _camera = a.ftp();
    let shown = a.camera_runtime().ftp_address();
    assert_eq!(shown.address, Some("127.0.0.1".parse().unwrap()));
    assert_eq!(shown.warning(), None);
    a.tick(0).unwrap();
    assert_eq!(health_detail(&a, "camera_address_changed"), None);

    network(&a, &["192.168.1.20"]);
    let shown = a.camera_runtime().ftp_address();
    assert_eq!(shown.address, Some("192.168.1.20".parse().unwrap()));
    assert_eq!(shown.warning().as_deref(), Some(SENTENCE_B));
    settle(&a, "the address health item", || {
        health_detail(&a, "camera_address_changed").is_some()
    });
    assert_eq!(
        health_detail(&a, "camera_address_changed").as_deref(),
        Some(SENTENCE_B)
    );

    // Back on A: the sentence goes.
    network(&a, &["192.168.1.20", "127.0.0.1"]);
    settle(&a, "the cleared health item", || {
        health_detail(&a, "camera_address_changed").is_none()
    });
    assert_eq!(
        a.camera_runtime().ftp_address().address,
        Some("127.0.0.1".parse().unwrap())
    );
}

/// C2(a): Generate makes 16 characters from the easy set; Show password reads the same value
/// back; the receiver accepts it. A password stored before P68 kept working until then.
#[test]
fn c2a_easy_password_generated_and_shown_again() {
    let a = TestApp::with_camera_script(include_str!(
        "../../src/fixtures/sidecar_scripts/ready_wifi.txt"
    ))
    .unwrap();
    settle(&a, "the receiver", || {
        a.camera_runtime().receiver_port().is_some()
    });
    // The pre-filled (old-style) password still logs in before any regeneration.
    a.ftp();
    let mut bytes = [0u8; 64];
    let mut pos = bytes.len();
    let generated = CameraService::generate_ftp_password(
        a.secrets.as_ref(),
        || {
            if pos == bytes.len() {
                getrandom::fill(&mut bytes).unwrap();
                pos = 0;
            }
            pos += 1;
            bytes[pos - 1]
        },
        |_| a.camera_runtime().ftp_password_changed(),
    )
    .unwrap();
    assert_eq!(generated.len(), 16);
    assert!(
        generated
            .bytes()
            .all(|b| FTP_PASSWORD_ALPHABET.contains(&b)),
        "{generated}"
    );
    assert!(!generated.contains(['0', 'o', '1', 'l', 'i']));
    assert_eq!(
        CameraService::ftp_password(a.secrets.as_ref())
            .unwrap()
            .as_deref(),
        Some(generated.as_str())
    );
    assert_eq!(
        a.secrets.get(secrets::FTP_PASSWORD).unwrap().as_deref(),
        Some(generated.as_str())
    );
    let mut ftp = psm_testkit::ftp_client::FtpClient::connect(a.receiver_port()).unwrap();
    assert!(
        ftp.login("sessionmanager", &generated)
            .unwrap()
            .starts_with("230")
    );
}

fn adapter_for(domain: &str) -> (Arc<FakeTransport>, ShopifyAdapter) {
    let fake = Arc::new(FakeTransport::new());
    let adapter = ShopifyAdapter::new(
        ShopifyConfig {
            shop_domain: domain.into(),
            api_version: "2026-10".into(),
            client_id: "id".into(),
            client_secret: "secret".into(),
            condition_photos_key: "custom.damage_report".into(),
        },
        fake.clone(),
        Arc::new(ManualClock::new(1_000_000)),
        PollConfig {
            interval_ms: 0,
            attempts: 1,
            retries: 0,
            retry_wait_ms: 0,
        },
    );
    (fake, adapter)
}

/// C3(a): the shop domain is stored normalised and the keys trimmed; a domain that cannot be a
/// host name is named in a sentence, with no request sent (never `http: builder error`).
#[test]
fn c3a_shop_domain_normalised_and_bad_domain_named() {
    let mut a = TestApp::new().unwrap();
    let stored = |a: &TestApp| -> serde_json::Value {
        serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(a.app.paths.settings()).unwrap(),
        )
        .unwrap()["integration"]
            .clone()
    };
    a.settings_set(&json!({"integration": {
        "kind": "shopify", "shop_domain": "https://X.myshopify.com/admin/ ", "api_version": "2026-10",
        "client_id": " key ", "client_secret": " secret-value ", "condition_photos_key": "custom.damage_report"
    }}))
    .unwrap();
    let integration = stored(&a);
    assert_eq!(integration["shop_domain"], "x.myshopify.com");
    assert_eq!(integration["client_id"], "key");
    assert_eq!(integration["client_secret"], "secret-value");

    a.settings_set(&json!({"integration": {
        "kind": "shopify", "shop_domain": "admin.shopify.com/store/x", "api_version": "2026-10",
        "client_id": "key", "client_secret": "secret-value", "condition_photos_key": "custom.damage_report"
    }}))
    .unwrap();
    assert_eq!(stored(&a)["shop_domain"], "x.myshopify.com");

    // A domain saved before P68 that cannot be a host name.
    let (fake, adapter) = adapter_for("my shop^.myshopify.com");
    let error = adapter.check_token().unwrap_err();
    assert!(matches!(error, Error::ShopDomainInvalid { .. }));
    assert_eq!(
        error.to_string(),
        "The shop domain looks wrong: my shop^.myshopify.com"
    );
    assert!(!error.to_string().contains("builder error"));
    assert!(fake.requests().is_empty());
}
