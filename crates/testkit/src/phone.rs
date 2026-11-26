//! LV2 (`master_plan_live.md` §4.7): `TestPhone` — the real `PhoneServer` on `127.0.0.1:0` over a
//! TestApp's handles, a temporary web root (`phone.html` = `<p>phone</p>`, `assets/app.js`, and a
//! file `secret` beside the web root) and short `PhoneTimings`.
use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use psm_integrations::phone::{
    host::HostInfo,
    pairing::PhonePairings,
    server::{PhoneConfig, PhoneServer, PhoneTimings},
};

use crate::TestApp;

/// The Mac's name the test server reports.
pub const TEST_MAC_NAME: &str = "Test-Mac";
/// The bytes of `assets/app.js` in the test web root.
pub const APP_JS: &[u8] = b"console.log('phone');\n";

struct TestHost;
impl HostInfo for TestHost {
    fn local_host_name(&self) -> Option<String> {
        Some(TEST_MAC_NAME.into())
    }
    fn private_ipv4(&self) -> Option<Ipv4Addr> {
        Some(Ipv4Addr::LOCALHOST)
    }
}

pub struct TestPhone {
    pub server: PhoneServer,
    /// Holds `web/` (the web root) and `secret` beside it.
    pub dir: tempfile::TempDir,
}

impl TestPhone {
    /// Short timings: ping 200 ms, idle close 600 ms, write timeout 500 ms.
    pub fn start(a: &TestApp) -> Self {
        Self::start_with(
            a,
            PhoneTimings {
                ping: Duration::from_millis(200),
                idle_close: Duration::from_millis(600),
                write_timeout: Duration::from_millis(500),
            },
        )
    }

    pub fn start_with(a: &TestApp, timings: PhoneTimings) -> Self {
        let dir = tempfile::tempdir().expect("temp web root");
        let web = dir.path().join("web");
        std::fs::create_dir_all(web.join("assets")).unwrap();
        std::fs::write(web.join("phone.html"), b"<p>phone</p>").unwrap();
        std::fs::write(web.join("assets/app.js"), APP_JS).unwrap();
        std::fs::write(dir.path().join("secret"), b"secret").unwrap();
        let handles = a.app.handles();
        let server = PhoneServer::start(
            PhoneConfig {
                enabled: true,
                bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
                port: 0,
                web_root: Some(web),
                timings,
                host: Arc::new(TestHost),
            },
            Arc::new(move || Ok(handles.clone())),
            a.live_hub(),
        );
        assert!(server.local_addr().is_some(), "{:?}", server.status());
        Self { server, dir }
    }

    /// `http://127.0.0.1:<port><path>`.
    pub fn url(&self, path: &str) -> String {
        format!(
            "http://{}{path}",
            self.server.local_addr().expect("listening")
        )
    }

    /// `ws://127.0.0.1:<port><path>`.
    pub fn ws_url(&self, path: &str) -> String {
        format!(
            "ws://{}{path}",
            self.server.local_addr().expect("listening")
        )
    }

    /// A new pairing's token.
    pub fn pair(&self, a: &TestApp) -> String {
        PhonePairings::new(a.store(), a.clock.as_ref())
            .create(None)
            .expect("pairing")
            .token
    }

    /// The web root.
    pub fn web_root(&self) -> PathBuf {
        self.dir.path().join("web")
    }
}
