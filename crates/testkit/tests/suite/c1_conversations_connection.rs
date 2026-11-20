//! C1 (`docs/implementation/plans/C1_connection_and_sync.md`): the adapter over the fake Pricing
//! server (C1·C6(b)), and Settings → Pricing: the settings slot, the key in the Keychain, the
//! connection states and the store match (U2, C1·C7–C10). Every test runs the real
//! `BeyoPricingAdapter` over `FakePricing`; nothing calls Pricing or Shopify.

use psm_core::ports::conversations::ConversationsProvider;
use psm_integrations::conversations::beyo_pricing::BeyoPricingAdapter;
use psm_testkit::fake_pricing::FakePricing;
use std::{
    io::Write,
    sync::{Arc, Mutex},
};

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

/// Runs `f` with every log line it writes collected **raw** (no redacting writer, so a key that
/// reached a log call would show); returns its result and the log text.
fn logged_raw<T>(f: impl FnOnce() -> T) -> (T, String) {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let output = LogBytes(buffer.clone());
    psm_testkit::logs::keep_log_sites_interested();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || output.clone())
        .with_max_level(tracing::Level::TRACE)
        .without_time()
        .with_ansi(false)
        .finish();
    let value = tracing::subscriber::with_default(subscriber, f);
    let text = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
    (value, text)
}

#[test]
fn c1_c6b_the_key_never_reaches_an_error_a_log_or_the_request_record() {
    const KEY: &str = "secret-key-123";
    let fake = Arc::new(FakePricing::new());
    let adapter = BeyoPricingAdapter::new(
        fake.clone(),
        || "https://pricing.example".into(),
        || Some(KEY.into()),
    );
    let (errors, log) = logged_raw(|| {
        // 401: the fake does not accept this key.
        let refused = adapter.stores().unwrap_err();
        fake.refuse_connection_next(1);
        let unreachable = adapter.stores().unwrap_err();
        fake.fail_matching("/stores", 1, 500);
        let server_error = adapter.stores().unwrap_err();
        vec![refused, unreachable, server_error]
    });
    assert_eq!(errors.len(), 3);
    assert_eq!(
        errors.iter().map(|e| e.kind()).collect::<Vec<_>>(),
        ["refused", "unreachable", "unreachable"]
    );
    for error in &errors {
        assert!(!error.to_string().contains(KEY), "{error}");
        assert!(!format!("{error:?}").contains(KEY), "{error:?}");
        let sentence = adapter.describe(error, Some("shop-a.myshopify.com"));
        assert!(!sentence.contains(KEY), "{sentence}");
    }
    assert!(!log.contains(KEY), "{log}");
    let requests = fake.requests();
    assert_eq!(requests.len(), 3);
    for request in &requests {
        assert!(!format!("{request:?}").contains(KEY), "{request:?}");
        assert!(!request.authorized);
    }
}
