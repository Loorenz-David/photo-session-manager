#[cfg(test)]
mod a12_tests;
mod alerts;
mod commands;
pub mod search_window;
mod state;

use psm_core::{events::AppEvent, ports::events::EventSink};
use psm_infra::paths::AppPaths;
use state::{AppState, EditMenu};
use std::sync::Arc;
use tauri::{
    Emitter, Manager,
    menu::{Menu, MenuItem, MenuItemKind, PredefinedMenuItem, Submenu},
};

/// The bridge from core events to the window (and, for conversations, to the chime and the Mac
/// banner: C2, `alerts`). `focused` reads whether the main window has the focus.
struct TauriEvents<R: tauri::Runtime> {
    app: tauri::AppHandle<R>,
    alerts: Arc<dyn alerts::Alerts>,
    focused: Arc<dyn Fn() -> bool + Send + Sync>,
}
impl<R: tauri::Runtime> TauriEvents<R> {
    fn new(
        app: tauri::AppHandle<R>,
        alerts: Arc<dyn alerts::Alerts>,
        focused: Arc<dyn Fn() -> bool + Send + Sync>,
    ) -> Self {
        Self {
            app,
            alerts,
            focused,
        }
    }
    /// A bridge without alerts (tests that only look at the window events).
    #[cfg(test)]
    fn silent(app: tauri::AppHandle<R>) -> Self {
        Self::new(app, Arc::new(alerts::NoAlerts), Arc::new(|| true))
    }
}
#[derive(serde::Serialize, Clone)]
struct PhotoAddedPayload {
    photo_id: i64,
    session_id: i64,
}

#[cfg(test)]
mod p50_tests {
    use super::*;
    use psm_core::ports::store::Store;
    use psm_integrations::camera::support::manifest::{Manifest, ManifestFile};
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::{
        Listener,
        ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
        webview::InvokeRequest,
    };

    fn call(
        w: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        args: Value,
    ) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }
    #[test]
    fn a25_c23a_new_commands_reach_wired_app_and_bindings() {
        let a = psm_testkit::TestApp::with_research().unwrap();
        let s = a
            .items()
            .create_local("CMD-R11", "Chair", "Chair", None)
            .unwrap()
            .1
            .session
            .id;
        let photo = a.add_photo(s, "command-r11.ARW").unwrap();
        a.review()
            .classify(photo.id, psm_core::domain::photo::Classification::Main)
            .unwrap();
        let store = a.app.store.clone();
        let paths = a.app.paths.clone();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::sessions::session_arrange_next,
                commands::research::research_settings_reset,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        let result = call(&w, "session_arrange_next", json!({"sessionId": s})).unwrap();
        assert_eq!(result["next_session_id"], Value::Null);
        assert_eq!(result["research_queued"], Value::Bool(true));
        assert!(store.session(s).unwrap().session.arranged_at.is_some());
        let runs = store
            .research_transaction(|tx| psm_research::store::runs(tx.conn(), Some(s)))
            .unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].state, psm_research::domain::RunState::Queued);
        assert_eq!(runs[0].trigger.as_str(), "arranged");
        let mut emptied = psm_testkit::research_test_settings();
        emptied["prices"] = json!([]);
        let mut file = psm_infra::settings_file::Settings::open(&paths).unwrap();
        file.research = Some(emptied);
        psm_infra::settings_file::Settings::set(&paths, &file).unwrap();
        let reset = call(&w, "research_settings_reset", json!({})).unwrap();
        let value: Value = serde_json::from_str(reset.as_str().unwrap()).unwrap();
        assert!(!value["prices"].as_array().unwrap().is_empty());
        let saved = psm_infra::settings_file::Settings::open(&paths)
            .unwrap()
            .research
            .unwrap();
        assert_eq!(saved["prices"], value["prices"]);
        let bindings = include_str!("../../src/services/bindings.ts");
        for name in [
            "sessionArrangeNext",
            "researchSettingsReset",
            "arranged",
            "researching",
            "fill_keys",
            "extra_shop_hosts",
            "detailed_condition",
        ] {
            assert!(bindings.contains(name), "binding missing {name}");
        }
        assert!(!bindings.contains("quiet_seconds"));
        assert!(!bindings.contains("auto?:"));
        assert!(!bindings.contains("excluded_keys"));
    }

    #[test]
    fn a27_c7a_settings_commands_use_the_chosen_field_list() {
        let a = psm_testkit::TestApp::with_research().unwrap();
        let paths = a.app.paths.clone();
        let mut file = psm_infra::settings_file::Settings::open(&paths).unwrap();
        file.research = Some(json!({"excluded_keys": ["custom.wood_type"]}));
        psm_infra::settings_file::Settings::set(&paths, &file).unwrap();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::settings::settings_get,
                commands::settings::settings_set,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        let got = call(&w, "settings_get", json!({})).unwrap();
        assert_eq!(got["research"]["fill_keys"], json!([]));
        assert!(got["research"].get("excluded_keys").is_none());
        let mut research = got["research"].clone();
        research["fill_keys"] = json!(["custom.wood_type"]);
        call(&w, "settings_set", json!({"patch": {"research": research}})).unwrap();
        let saved = psm_infra::settings_file::Settings::open(&paths)
            .unwrap()
            .research
            .unwrap();
        assert_eq!(saved["fill_keys"], json!(["custom.wood_type"]));
        assert!(saved.get("excluded_keys").is_none());
        let bindings = include_str!("../../src/services/bindings.ts");
        assert!(bindings.contains("fill_keys: string[]"));
        assert!(!bindings.contains("excluded_keys"));
    }
    #[test]
    fn c8a_support_and_secret_commands_have_their_wire_shapes() {
        let mut a = psm_testkit::TestApp::with_fake_document().unwrap();
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        a.settings_set(&json!({"camera":{"receiver_port":port,"passive_port_start":50000,"passive_port_end":50009}})).unwrap();
        let names = [
            "libCr_Core.dylib",
            "libmonitor_protocol.dylib",
            "libmonitor_protocol_pf.dylib",
            "CrAdapter/libCr_PTP_USB.dylib",
            "CrAdapter/libCr_PTP_IP.dylib",
            "CrAdapter/libssh2.dylib",
            "CrAdapter/libusb-1.0.0.dylib",
        ];
        a.with_test_manifest(Manifest {
            version: "2.02.00".into(),
            files: names
                .iter()
                .map(|n| ManifestFile {
                    path: (*n).into(),
                    sha256: Sha256::digest(n.as_bytes())
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect(),
                })
                .collect(),
        });
        let bundled = a.bundled_sidecar_path();
        let complete: Vec<_> = names.iter().map(|n| (*n, n.as_bytes())).collect();
        let incomplete: Vec<_> = complete
            .iter()
            .copied()
            .filter(|(n, _)| *n != "CrAdapter/libssh2.dylib")
            .collect();
        let bad_archive = a.support_archive(&incomplete);
        let good_archive = a.support_archive(&complete);
        let root = a.app.paths.root.clone();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::camera::camera_support_get,
                commands::camera::camera_support_install,
                commands::camera::camera_secrets_set,
                commands::camera::camera_ftp_password_generate,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        assert_eq!(
            call(&w, "camera_support_get", json!({})).unwrap(),
            Value::Null
        );
        assert_eq!(
            call(
                &w,
                "camera_secrets_set",
                json!({"accessAuthUser":"u2","accessAuthPassword":"p2"})
            )
            .unwrap(),
            Value::Null
        );
        let pw = call(&w, "camera_ftp_password_generate", json!({})).unwrap()["password"]
            .as_str()
            .unwrap()
            .to_owned();
        assert_eq!(pw.len(), 4);
        assert_eq!(
            call(
                &w,
                "camera_support_install",
                json!({"archivePath":bad_archive.to_string_lossy()})
            )
            .unwrap_err(),
            json!({"code":"support_archive_invalid","message":"support_archive_invalid: missing CrAdapter/libssh2.dylib","data":{"file":"missing CrAdapter/libssh2.dylib"}})
        );
        let installed = call(
            &w,
            "camera_support_install",
            json!({"archivePath":good_archive.to_string_lossy()}),
        )
        .unwrap();
        assert_eq!(installed["make"], "sony");
        assert_eq!(installed["version"], "2.02.00");
        assert_eq!(
            installed["sidecar_path"],
            root.join("camera/sony/2.02.00/camera-sidecar")
                .to_string_lossy()
                .as_ref()
        );
        assert_eq!(
            call(&w, "camera_support_get", json!({})).unwrap(),
            installed
        );
        assert!(
            !std::fs::read_to_string(root.join("settings.json"))
                .unwrap()
                .contains(&pw)
        );
        let _ = bundled;
    }
    #[test]
    fn c8b_pending_fingerprint_reaches_tauri_event() {
        let a = psm_testkit::TestApp::with_fake_document().unwrap();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .build(mock_context(noop_assets()))
            .unwrap();
        let payloads = Arc::new(std::sync::Mutex::new(Vec::<Value>::new()));
        let seen = payloads.clone();
        tauri_app.listen_any("camera_fingerprint_pending", move |event| {
            seen.lock()
                .unwrap()
                .push(serde_json::from_str(event.payload()).unwrap())
        });
        EventSink::emit(
            &TauriEvents::silent(tauri_app.handle().clone()),
            AppEvent::FingerprintPending {
                fingerprint: "ab:cd:ef".into(),
                replace: true,
            },
        );
        assert_eq!(
            *payloads.lock().unwrap(),
            vec![json!({"fingerprint":"ab:cd:ef","replace":true})]
        );
    }
}

#[cfg(test)]
mod r2_research_command_tests {
    use super::*;
    use serde_json::{Value, json};
    use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::webview::InvokeRequest;

    fn call(
        w: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        args: Value,
    ) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }
    /// R2 F·C1(a), superseded by R7·U1: the commands reach the engine the app now builds (none
    /// answers `research_unavailable`), and none writes a form value.
    #[test]
    fn f_c1a_research_commands_reach_the_wired_engine() {
        let a = psm_testkit::TestApp::new().unwrap();
        let db_path = a.app.paths.db();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::research::research_status,
                commands::research::research_overview,
                commands::research::research_run,
                commands::research::research_secrets_state,
                commands::research::research_site_add,
                commands::sessions::session_active,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        let before: i64 = rusqlite::Connection::open(&db_path)
            .unwrap()
            .query_row("SELECT COUNT(*) FROM attribute_values", [], |r| r.get(0))
            .unwrap();
        for (cmd, args) in [
            ("research_status", json!({"sessionId":1})),
            ("research_overview", json!({"sessionId":null})),
            ("research_run", json!({"sessionId":1})),
            ("research_secrets_state", json!({})),
            ("research_site_add", json!({"site":"example.com"})),
        ] {
            if let Err(error) = call(&w, cmd, args) {
                assert_ne!(error["code"], "research_unavailable", "{cmd}: {error}");
            }
        }
        assert_eq!(
            call(&w, "research_secrets_state", json!({})).unwrap(),
            json!({"model": false, "search": false, "reader": false})
        );
        assert!(call(&w, "session_active", json!({})).is_ok());
        let after: i64 = rusqlite::Connection::open(&db_path)
            .unwrap()
            .query_row("SELECT COUNT(*) FROM attribute_values", [], |r| r.get(0))
            .unwrap();
        assert_eq!(after, before);
    }
}

#[cfg(test)]
mod r5_command_tests {
    use super::*;
    use psm_core::domain::attribute::{AiConfidence, AiStatus, Provenance};
    use psm_core::ports::store::Store;
    use serde_json::{Value, json};
    use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::webview::InvokeRequest;

    fn call(
        w: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        args: Value,
    ) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }

    /// A08·C5(a): each conflict command is one undo step and leaves the row of C1(a), C1(c),
    /// C1(a), C1(e).
    #[test]
    fn r5_a08_c5a_attribute_commands_are_one_step_each() {
        let mut a = psm_testkit::TestApp::new().unwrap();
        a.settings_set(
            &json!({"integration":{"kind":"shopify","shop_domain":"x.myshopify.com",
            "api_version":"2026-10","client_id":"c","client_secret":"s"}}),
        )
        .unwrap();
        let keys = ["k1", "k2", "k3", "k4"];
        a.research_fields(&keys).unwrap();
        let (item, created) = a
            .items()
            .create_local("R5-CMD", "Oak chair", "object", None)
            .unwrap();
        for key in keys {
            a.research_conflict(item.id, key, "D", "A", AiConfidence::Medium)
                .unwrap();
        }
        let session = created.session.id;
        let history = a.app.history.store();
        let store = a.app.store.clone();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::attributes::attribute_ai_accept,
                commands::attributes::attribute_conflict_resolve,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        let depth = || {
            history
                .lock()
                .unwrap()
                .get(&session)
                .map_or(0, |l| l.undo.len())
        };
        let row = |key: &str| {
            let mut out = None;
            store
                .transaction(&mut |tx| {
                    out = tx
                        .attribute_values(item.id)?
                        .into_iter()
                        .find(|v| v.key == key);
                    Ok(())
                })
                .unwrap();
            out.unwrap()
        };
        let calls = [
            (
                "attribute_ai_accept",
                json!({"itemId": item.id, "key": "k1"}),
            ),
            (
                "attribute_conflict_resolve",
                json!({"itemId": item.id, "key": "k2", "action": "keep_previous", "value": null}),
            ),
            (
                "attribute_conflict_resolve",
                json!({"itemId": item.id, "key": "k3", "action": "use_suggestion", "value": null}),
            ),
            (
                "attribute_conflict_resolve",
                json!({"itemId": item.id, "key": "k4", "action": "edit", "value": "\"Walnut\""}),
            ),
        ];
        for (n, (cmd, args)) in calls.into_iter().enumerate() {
            call(&w, cmd, args.clone()).unwrap_or_else(|e| panic!("{cmd} {args}: {e}"));
            assert_eq!(depth(), n + 1, "{cmd} {args}");
        }
        for key in ["k1", "k3"] {
            let r = row(key);
            assert_eq!(
                (r.provenance, r.ai_status, r.value.as_str(), r.ai_prior),
                (Provenance::Ai, Some(AiStatus::Accepted), "\"A\"", None),
                "{key}"
            );
        }
        let r = row("k2");
        assert_eq!(
            (r.provenance, r.ai_status, r.value.as_str(), r.ai_prior),
            (Provenance::Default, None, "\"D\"", None)
        );
        let r = row("k4");
        assert_eq!(
            (r.provenance, r.ai_status, r.value.as_str(), r.ai_prior),
            (Provenance::Human, None, "\"Walnut\"", None)
        );
    }

    /// A09·C2(a): the refusal reaches the UI as the code R6·U3 keys on.
    #[test]
    fn r5_a09_c2a_suggestions_changed_is_its_own_code() {
        let error = commands::items::CommandError::from(psm_core::Error::SuggestionsChanged);
        assert_eq!(error.code, "suggestions_changed");
        assert_eq!(
            error.message,
            "AI suggestions changed while you were confirming"
        );
    }
}

#[cfg(test)]
mod c1_tests {
    //! C1·U4 (`docs/implementation/plans/C1_connection_and_sync.md` C1·C19): the Settings →
    //! Pricing commands over the wired app with the fake Pricing server, and the event bridge.
    use super::*;
    use psm_core::{
        domain::conversation::{ConversationNotice, KEY_ACCOUNT, NoticeKind},
        ports::secrets::SecretStore,
    };
    use psm_integrations::conversations::beyo_pricing::DEFAULT_SERVER;
    use psm_testkit::test_app::PRICING_KEY;
    use serde_json::{Value, json};
    use tauri::Listener;
    use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::webview::InvokeRequest;

    fn call(
        w: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        args: Value,
    ) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }

    struct Wired {
        _dir: tempfile::TempDir,
        _app: tauri::App<tauri::test::MockRuntime>,
        window: tauri::WebviewWindow<tauri::test::MockRuntime>,
        pricing: Arc<psm_testkit::fake_pricing::FakePricing>,
        secrets: Arc<psm_testkit::fake_secrets::FakeSecrets>,
    }

    fn wired() -> Wired {
        let a = psm_testkit::TestApp::with_pricing().unwrap();
        let pricing = a.pricing.clone().unwrap();
        let secrets = a.secrets.clone();
        let psm_testkit::TestApp { dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::conversations::conversations_settings_get,
                commands::conversations::conversations_settings_save,
                commands::conversations::conversations_key_set,
                commands::conversations::conversations_test,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        Wired {
            _dir: dir,
            _app: tauri_app,
            window,
            pricing,
            secrets,
        }
    }

    #[test]
    fn c1_c19a_settings_get_on_a_fresh_folder() {
        let w = wired();
        let got = call(&w.window, "conversations_settings_get", json!({})).unwrap();
        assert_eq!(got["server"], json!(DEFAULT_SERVER));
        assert_eq!(got["server_default"], json!(DEFAULT_SERVER));
        assert_eq!(got["key_saved"], json!(false));
        assert_eq!(got["sound"], json!(true));
        assert_eq!(got["banner"], json!(true));
        assert_eq!(got["connection"]["state"], json!("no_key"));
    }

    #[test]
    fn c1_c19b_saving_the_key_connects_and_never_returns_it() {
        let w = wired();
        let saved = call(
            &w.window,
            "conversations_key_set",
            json!({"key": PRICING_KEY}),
        )
        .unwrap();
        let got = call(&w.window, "conversations_settings_get", json!({})).unwrap();
        for answer in [&saved, &got] {
            assert_eq!(answer["key_saved"], json!(true));
            assert_eq!(answer["connection"]["state"], json!("ready"));
            assert_eq!(answer["connection"]["store_name"], json!("Dev store"));
            assert_eq!(answer["connection"]["shop"], json!("shop-a.myshopify.com"));
            assert!(!answer.to_string().contains(PRICING_KEY), "{answer}");
        }
    }

    #[test]
    fn c1_c19c_an_empty_key_removes_it() {
        let w = wired();
        call(
            &w.window,
            "conversations_key_set",
            json!({"key": PRICING_KEY}),
        )
        .unwrap();
        let removed = call(&w.window, "conversations_key_set", json!({"key": ""})).unwrap();
        assert_eq!(removed["key_saved"], json!(false));
        assert_eq!(removed["connection"]["state"], json!("no_key"));
        assert_eq!(w.secrets.get(KEY_ACCOUNT).unwrap(), None);
    }

    #[test]
    fn c1_c19d_test_connection_lists_the_visible_stores() {
        let w = wired();
        w.pricing.clear_stores();
        w.pricing.add_store("One", "one.myshopify.com");
        w.pricing.add_store("Two", "two.myshopify.com");
        w.secrets.set(KEY_ACCOUNT, PRICING_KEY).unwrap();
        let got = call(&w.window, "conversations_test", json!({})).unwrap();
        assert_eq!(got["connection"]["state"], json!("store_unmatched"));
        assert_eq!(got["connection"]["visible_stores"], json!(["One", "Two"]));
    }

    #[test]
    fn c1_c19e_an_http_server_is_refused_naming_server() {
        let w = wired();
        let error = call(
            &w.window,
            "conversations_settings_save",
            json!({"patch": {"server": "http://x.example"}}),
        )
        .unwrap_err();
        assert_eq!(error["code"], json!("invalid_value"));
        assert!(
            error["message"].as_str().unwrap().contains("server"),
            "{error}"
        );
    }

    #[test]
    fn c1_c19f_the_bridge_forwards_the_toast_words() {
        let tauri_app = mock_builder().build(mock_context(noop_assets())).unwrap();
        let payloads = Arc::new(std::sync::Mutex::new(Vec::<Value>::new()));
        let seen = payloads.clone();
        tauri_app.listen_any("conversations_changed", move |event| {
            seen.lock()
                .unwrap()
                .push(serde_json::from_str(event.payload()).unwrap())
        });
        EventSink::emit(
            &TauriEvents::silent(tauri_app.handle().clone()),
            AppEvent::ConversationsChanged {
                notices: vec![ConversationNotice {
                    conversation_id: "c1".into(),
                    sku: Some("BEYO-1880".into()),
                    product_title: "Vintage Brass Wall Mirror, Round".into(),
                    topic: "Condition".into(),
                    kind: NoticeKind::Question,
                }],
                chime: true,
                banner: true,
            },
        );
        assert_eq!(
            *payloads.lock().unwrap(),
            vec![json!({
                "toasts": [{"conversation_id": "c1", "text": "New question · BEYO-1880 — Condition"}],
                "more": 0
            })]
        );
    }
}

#[cfg(test)]
mod l1_tests {
    //! L1·U4 (`docs/implementation/plans/L1_labels_provider_and_marker.md` L1·C15): the label
    //! markers command, the settings answer's `labels` and the event bridge over the wired app with
    //! the fake Pricing server.
    use super::*;
    use psm_core::{
        domain::conversation::KEY_ACCOUNT,
        ports::{clock::Clock, secrets::SecretStore},
        services::items::ItemsService,
    };
    use psm_testkit::{fake_pricing::LabelSpec, test_app::PRICING_KEY};
    use serde_json::{Value, json};
    use tauri::Listener;
    use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::webview::InvokeRequest;

    const P1: &str = "15531808096409";
    /// 2026-11-20T09:12:00Z.
    const T: i64 = 1_795_165_920_000;

    fn call(
        w: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        args: Value,
    ) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }

    struct Wired {
        _dir: tempfile::TempDir,
        _app: tauri::App<tauri::test::MockRuntime>,
        window: tauri::WebviewWindow<tauri::test::MockRuntime>,
        pricing: Arc<psm_testkit::fake_pricing::FakePricing>,
        secrets: Arc<psm_testkit::fake_secrets::FakeSecrets>,
        /// `s1`: linked to `P1`, printed in the fake at `T`; `s2`: an item without a link.
        s1: i64,
        s2: i64,
    }

    /// L1·C13(a)'s app, wired; connected (key saved, first connection run) when `connect`.
    fn wired(connect: bool) -> Wired {
        let a = psm_testkit::TestApp::with_pricing().unwrap();
        a.clock.advance(1_000_000);
        let external = a.fake_shop().add_product_with_id(
            &format!("gid://shopify/Product/{P1}"),
            "TEST-1802",
            "Test table",
            None,
        );
        let item = ItemsService::new(a.store(), a.clock.as_ref())
            .with_commerce(Some(a.fake_shop()))
            .adopt(&external)
            .unwrap();
        let s1 = a.sessions().create(item.id).unwrap().session.id;
        let (_, local) = a
            .items()
            .create_local("LOCAL-1", "Local", "object", None)
            .unwrap();
        let s2 = local.session.id;
        let pricing = a.pricing.clone().unwrap();
        pricing.add_label_product(LabelSpec {
            product_id: P1.into(),
            sku: Some("TEST-1802".into()),
            title: "Test table".into(),
            price_minor: Some(890000),
            printed_at: Some(T),
            ..LabelSpec::default()
        });
        if connect {
            a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
            while a.app.conversations.tick(a.clock.now_ms()).unwrap().more {}
        }
        let secrets = a.secrets.clone();
        let psm_testkit::TestApp { dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::conversations::conversations_settings_get,
                commands::conversations::conversations_key_set,
                commands::conversations::conversations_test,
                commands::labels::labels_status,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        Wired {
            _dir: dir,
            _app: tauri_app,
            window,
            pricing,
            secrets,
            s1,
            s2,
        }
    }

    #[test]
    fn l1_c15a_labels_status_answers_markers_and_access() {
        let w = wired(true);
        let got = call(
            &w.window,
            "labels_status",
            json!({"sessionIds": [w.s1, w.s2], "refresh": true}),
        )
        .unwrap();
        assert_eq!(
            got,
            json!({
                "access": {"state": "allowed", "reason": null},
                "labels": [
                    {"session_id": w.s1, "product_id": P1, "marker":
                        {"status": "printed", "printed_at": T, "has_price": true, "reset": null}},
                    {"session_id": w.s2, "product_id": null, "marker": null}
                ]
            })
        );
    }

    #[test]
    fn l1_c15b_without_a_key_the_access_is_off_with_its_reason() {
        let w = wired(false);
        let got = call(
            &w.window,
            "labels_status",
            json!({"sessionIds": [w.s1], "refresh": true}),
        )
        .unwrap();
        assert_eq!(
            got,
            json!({
                "access": {"state": "off", "reason": "no_key"},
                "labels": [{"session_id": w.s1, "product_id": P1, "marker": null}]
            })
        );
    }

    #[test]
    fn l1_c15c_an_unreachable_pricing_is_labels_unreachable() {
        let w = wired(true);
        w.pricing.fail_matching("/labels/status", 1, 500);
        let error = call(
            &w.window,
            "labels_status",
            json!({"sessionIds": [w.s1], "refresh": true}),
        )
        .unwrap_err();
        assert_eq!(error["code"], json!("labels_unreachable"));
        assert_eq!(
            error["message"],
            json!("Pricing can't be reached right now.")
        );
    }

    #[test]
    fn l1_c15d_the_bridge_forwards_labels_changed() {
        let tauri_app = mock_builder().build(mock_context(noop_assets())).unwrap();
        let payloads = Arc::new(std::sync::Mutex::new(Vec::<Value>::new()));
        let seen = payloads.clone();
        tauri_app.listen_any("labels_changed", move |event| {
            seen.lock()
                .unwrap()
                .push(serde_json::from_str(event.payload()).unwrap())
        });
        EventSink::emit(
            &TauriEvents::silent(tauri_app.handle().clone()),
            AppEvent::LabelsChanged {
                product_ids: vec![P1.into()],
                access: false,
            },
        );
        assert_eq!(
            *payloads.lock().unwrap(),
            vec![json!({"product_ids": [P1], "access": false})]
        );
    }

    #[test]
    fn l1_c15e_test_connection_answers_the_labels_access() {
        let w = wired(false);
        w.secrets.set(KEY_ACCOUNT, PRICING_KEY).unwrap();
        let allowed = call(&w.window, "conversations_test", json!({})).unwrap();
        assert_eq!(allowed["labels"], json!("allowed"));
        w.pricing.deny_labels();
        let denied = call(&w.window, "conversations_test", json!({})).unwrap();
        assert_eq!(denied["labels"], json!("denied"));
        call(&w.window, "conversations_key_set", json!({"key": ""})).unwrap();
        let off = call(&w.window, "conversations_test", json!({})).unwrap();
        assert_eq!(off["labels"], json!("off"));
    }
}

#[cfg(test)]
mod l2_tests {
    //! L2·U2 (`docs/implementation/plans/L2_labels_box.md` L2·C7): the Labels box commands over the
    //! wired app with the fake Pricing server and the fake shop.
    use super::*;
    use psm_core::{ports::clock::Clock, services::items::ItemsService};
    use psm_testkit::{fake_pricing::LabelSpec, test_app::PRICING_KEY};
    use serde_json::{Value, json};
    use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::webview::InvokeRequest;

    /// 2026-11-20T09:12:00Z.
    const T: i64 = 1_795_165_920_000;

    fn call(
        w: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        args: Value,
    ) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }

    struct Wired {
        _dir: tempfile::TempDir,
        _app: tauri::App<tauri::test::MockRuntime>,
        window: tauri::WebviewWindow<tauri::test::MockRuntime>,
        pricing: Arc<psm_testkit::fake_pricing::FakePricing>,
        /// The linked session for P1 (none unless `with_s1`).
        s1: Option<i64>,
    }

    fn product(sku: Option<&str>, id: &str, created: i64) -> LabelSpec {
        LabelSpec {
            product_id: id.into(),
            sku: sku.map(Into::into),
            title: format!("Test table {id}"),
            product_type: Some("Test tables".into()),
            price_minor: Some(890_000),
            created_at: Some(T + created * 1_000),
            ..LabelSpec::default()
        }
    }

    /// The wired app: the fake Pricing holds `products`; `with_s1` adds a linked session for P1;
    /// `connect` saves the key and runs the first connection.
    fn wired(products: Vec<LabelSpec>, with_s1: bool, connect: bool) -> Wired {
        let a = psm_testkit::TestApp::with_pricing().unwrap();
        a.clock.advance(1_000_000);
        let s1 = with_s1.then(|| {
            let external = a.fake_shop().add_product_with_id(
                "gid://shopify/Product/1",
                "TEST-1",
                "Test table 1",
                None,
            );
            let item = ItemsService::new(a.store(), a.clock.as_ref())
                .with_commerce(Some(a.fake_shop()))
                .adopt(&external)
                .unwrap();
            a.sessions().create(item.id).unwrap().session.id
        });
        a.fake_shop()
            .add_product_with_id("gid://shopify/Product/4", "SH-4", "Test table", None);
        let pricing = a.pricing.clone().unwrap();
        for spec in products {
            pricing.add_label_product(spec);
        }
        if connect {
            a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
            while a.app.conversations.tick(a.clock.now_ms()).unwrap().more {}
        }
        let psm_testkit::TestApp { dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::conversations::conversations_key_set,
                commands::labels::labels_list,
                commands::labels::labels_summary,
                commands::labels::labels_select_all,
                commands::labels::labels_open,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        Wired {
            _dir: dir,
            _app: tauri_app,
            window,
            pricing,
            s1,
        }
    }

    fn list(w: &Wired) -> Result<Value, Value> {
        call(
            &w.window,
            "labels_list",
            json!({"filter": "missing_price_tag", "query": null, "cursor": null}),
        )
    }

    #[test]
    fn l2_c7a_labels_list_answers_rows_counts_and_work_from() {
        let w = wired(
            vec![
                product(Some("TEST-1"), "1", 1),
                product(Some("TEST-3"), "3", 3),
            ],
            true,
            true,
        );
        let mut got = list(&w).unwrap();
        let session = got["rows"][1]["session"].take();
        assert_eq!(session["id"], json!(w.s1.unwrap()));
        assert_eq!(session["sku"], json!("TEST-1"));
        let marker =
            json!({"status": "not_printed", "printed_at": null, "has_price": true, "reset": null});
        assert_eq!(
            got,
            json!({
                "rows": [
                    {"product_id": "3", "commerce_id": "gid://shopify/Product/3", "sku": "TEST-3",
                     "title": "Test table 3", "product_type": "Test tables", "price_minor": 890000,
                     "marker": marker, "created_at": T + 3_000, "session": null, "picture_path": null},
                    {"product_id": "1", "commerce_id": "gid://shopify/Product/1", "sku": "TEST-1",
                     "title": "Test table 1", "product_type": "Test tables", "price_minor": 890000,
                     "marker": marker, "created_at": T + 1_000, "session": null, "picture_path": null}
                ],
                "next_cursor": null,
                "counts": {"total": 2, "has_price": 2, "missing_price": 0, "missing_price_tag": 2,
                           "printed": 0, "not_printed": 2},
                "work_from": "2026-11-19"
            })
        );
    }

    #[test]
    fn l2_c7b_off_and_denied_are_their_codes() {
        let off = wired(vec![product(Some("TEST-1"), "1", 1)], false, false);
        let error = list(&off).unwrap_err();
        assert_eq!(
            error,
            json!({"code": "labels_off", "message": "", "data": {"reason": "no_key"}})
        );
        let w = wired(vec![product(Some("TEST-1"), "1", 1)], false, false);
        w.pricing.deny_labels();
        call(
            &w.window,
            "conversations_key_set",
            json!({"key": PRICING_KEY}),
        )
        .unwrap();
        let denied = list(&w).unwrap_err();
        assert_eq!(denied["code"], json!("labels_denied"));
        assert_eq!(
            denied["message"],
            json!("Session Manager isn't allowed to print labels. Ask a Pricing admin.")
        );
    }

    #[test]
    fn l2_c7c_an_unavailable_list_is_labels_list_unavailable() {
        let w = wired(vec![product(Some("TEST-1"), "1", 1)], false, true);
        w.pricing.fail_matching_with(
            "/labels/products",
            1,
            503,
            "product_list_unavailable",
            "The product list is unavailable.",
        );
        let error = list(&w).unwrap_err();
        assert_eq!(error["code"], json!("labels_list_unavailable"));
        assert_eq!(
            error["message"],
            json!("The product list isn't available right now.")
        );
    }

    #[test]
    fn l2_c7d_labels_summary_answers_the_number_or_null() {
        let w = wired(
            vec![
                product(Some("TEST-1"), "1", 1),
                product(Some("TEST-2"), "2", 2),
                product(Some("TEST-3"), "3", 3),
            ],
            false,
            true,
        );
        assert_eq!(
            call(&w.window, "labels_summary", json!({"refresh": true})).unwrap(),
            json!({"missing_price_tag": 3})
        );
        call(&w.window, "conversations_key_set", json!({"key": ""})).unwrap();
        assert_eq!(
            call(&w.window, "labels_summary", json!({"refresh": true})).unwrap(),
            json!({"missing_price_tag": null})
        );
    }

    #[test]
    fn l2_c7e_labels_select_all_answers_the_products_with_a_sku() {
        let w = wired(
            vec![
                product(Some("TEST-1"), "1", 1),
                product(None, "2", 2),
                product(Some("TEST-3"), "3", 3),
            ],
            false,
            true,
        );
        assert_eq!(
            call(
                &w.window,
                "labels_select_all",
                json!({"filter": "missing_price_tag", "query": null})
            )
            .unwrap(),
            json!([
                {"product_id": "3", "sku": "TEST-3"},
                {"product_id": "1", "sku": "TEST-1"}
            ])
        );
    }

    #[test]
    fn l2_c7f_labels_open_answers_the_new_session() {
        let w = wired(vec![], false, true);
        let got = call(&w.window, "labels_open", json!({"productId": "4"})).unwrap();
        let id = got["session_id"].as_i64().expect("a session id");
        assert_eq!(got, json!({"session_id": id}));
    }
}

#[cfg(test)]
mod c2_tests {
    //! C2·U1 (`docs/implementation/plans/C2_header_panel_notifications.md` C2·C7): the list,
    //! counts and badges commands over the wired app with the fake Pricing server.
    use super::*;
    use psm_core::domain::conversation::{ConversationScope, ConversationTab};
    use psm_core::ports::clock::Clock;
    use psm_testkit::test_app::PRICING_KEY;
    use serde_json::{Value, json};
    use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::webview::InvokeRequest;

    fn call(
        w: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        args: Value,
    ) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }

    struct Wired {
        _dir: tempfile::TempDir,
        _app: tauri::App<tauri::test::MockRuntime>,
        window: tauri::WebviewWindow<tauri::test::MockRuntime>,
        pricing: Arc<psm_testkit::fake_pricing::FakePricing>,
    }

    /// A connected app (key saved, store matched) whose copy was brought in by a first check.
    fn connected() -> psm_testkit::TestApp {
        let a = psm_testkit::TestApp::with_pricing().unwrap();
        a.clock.advance(1_000_000);
        a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
        a
    }

    fn store_id(a: &psm_testkit::TestApp) -> String {
        use psm_core::ports::conversations::ConversationStore;
        a.app.store.sync().unwrap().store.unwrap().id
    }

    fn run_check(a: &psm_testkit::TestApp) {
        while a.app.conversations.tick(a.clock.now_ms()).unwrap().more {}
    }

    fn wired(a: psm_testkit::TestApp) -> Wired {
        let pricing = a.pricing.clone().unwrap();
        let psm_testkit::TestApp { dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::conversations::conversations_list,
                commands::conversations::conversations_counts,
                commands::conversations::conversations_session_badges,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        Wired {
            _dir: dir,
            _app: tauri_app,
            window,
            pricing,
        }
    }

    fn list(w: &Wired, scope: Value, tab: &str) -> Value {
        call(
            &w.window,
            "conversations_list",
            json!({"scope": scope, "tab": tab}),
        )
        .unwrap()
    }

    #[test]
    fn c2_c7a_no_key_lists_nothing() {
        let a = connected();
        a.fake_pricing()
            .ask(&store_id(&a), Some("BEYO-1"), "Condition");
        run_check(&a);
        a.app.conversations.key_set(None).unwrap();
        let w = wired(a);
        let got = list(&w, json!({"kind": "all"}), "open");
        assert_eq!(got["state"], json!("no_key"));
        assert_eq!(got["rows"], json!([]));
    }

    #[test]
    fn c2_c7b_a_refused_key_lists_the_copy_with_its_problem() {
        let a = connected();
        let s = store_id(&a);
        let one = a.fake_pricing().ask(&s, Some("BEYO-1"), "Condition");
        let two = a.fake_pricing().ask(&s, Some("BEYO-2"), "Condition");
        run_check(&a);
        a.fake_pricing().fail_matching("/events", 1, 401);
        a.clock.advance(10_000);
        run_check(&a);
        let w = wired(a);
        let got = list(&w, json!({"kind": "all"}), "open");
        assert_eq!(got["state"], json!("refused"));
        assert_eq!(
            got["problem"],
            json!("Pricing refused the key: check Settings → Pricing")
        );
        let mut ids: Vec<&str> = got["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        ids.sort();
        let mut want = [one.as_str(), two.as_str()];
        want.sort();
        assert_eq!(ids, want);
    }

    #[test]
    fn c2_c7c_an_unmatched_store_counts_nothing_and_shows_no_badge() {
        let a = connected();
        a.items()
            .create_local("BEYO-1", "BEYO-1", "object", None)
            .unwrap();
        a.fake_pricing()
            .ask(&store_id(&a), Some("BEYO-1"), "Condition");
        run_check(&a);
        assert_eq!(a.app.conversations.counts(None).unwrap().here, 1);
        a.fake_pricing().clear_stores();
        a.fake_pricing().add_store("Other", "other.myshopify.com");
        a.app.conversations.test_connection().unwrap();
        let w = wired(a);
        let counts = call(
            &w.window,
            "conversations_counts",
            json!({"sessionId": null}),
        )
        .unwrap();
        assert_eq!(counts, json!({"here": 0, "elsewhere": 0}));
        let badges = call(&w.window, "conversations_session_badges", json!({})).unwrap();
        assert_eq!(badges, json!([]));
    }

    #[test]
    fn c2_c7d_a_session_scope_and_the_resolved_tab_reach_the_service() {
        let a = connected();
        let (_, session) = a
            .items()
            .create_local("BEYO-1", "BEYO-1", "object", None)
            .unwrap();
        let session_id = session.session.id;
        let s = store_id(&a);
        let resolved = a.fake_pricing().ask(&s, Some("BEYO-1"), "Condition");
        a.fake_pricing().resolve(&resolved);
        // A30: a product with an open question is on the Open tab, so the open one is another product.
        a.fake_pricing().ask(&s, Some("BEYO-3"), "Other");
        a.fake_pricing().ask(&s, Some("BEYO-2"), "Other");
        run_check(&a);
        let want: Vec<String> = a
            .app
            .conversations
            .list(
                ConversationScope::Session(session_id),
                ConversationTab::Resolved,
                a.clock.now_ms(),
            )
            .unwrap()
            .rows
            .into_iter()
            .map(|r| r.id)
            .collect();
        assert_eq!(want, std::slice::from_ref(&resolved));
        let w = wired(a);
        let got = list(
            &w,
            json!({"kind": "session", "session_id": session_id}),
            "resolved",
        );
        let ids: Vec<&str> = got["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, want);
        assert_eq!(got["rows"][0]["session_id"], json!(session_id));
        assert_eq!(got["rows"][0]["status"], json!("resolved"));
    }

    // ---------- C2·C18, C2·C19 the bridge: chime and banner ----------

    use psm_core::domain::conversation::{ConversationNotice, NoticeKind};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tauri::Listener;

    #[derive(Default)]
    struct Recorded {
        chimes: Mutex<usize>,
        banners: Mutex<Vec<(String, String)>>,
    }
    impl alerts::Alerts for Recorded {
        fn chime(&self) {
            *self.chimes.lock().unwrap() += 1;
        }
        fn banner(&self, title: &str, body: &str) {
            self.banners
                .lock()
                .unwrap()
                .push((title.to_owned(), body.to_owned()));
        }
    }

    struct Bridge {
        _app: tauri::App<tauri::test::MockRuntime>,
        events: TauriEvents<tauri::test::MockRuntime>,
        alerts: Arc<Recorded>,
        window: Arc<Mutex<Vec<Value>>>,
    }

    fn bridge(focused: bool) -> Bridge {
        let tauri_app = mock_builder().build(mock_context(noop_assets())).unwrap();
        let window = Arc::new(Mutex::new(Vec::<Value>::new()));
        let seen = window.clone();
        tauri_app.listen_any("conversations_changed", move |event| {
            seen.lock()
                .unwrap()
                .push(serde_json::from_str(event.payload()).unwrap())
        });
        let alerts = Arc::new(Recorded::default());
        let flag = Arc::new(AtomicBool::new(focused));
        let events = TauriEvents::new(
            tauri_app.handle().clone(),
            alerts.clone(),
            Arc::new(move || flag.load(Ordering::Relaxed)),
        );
        Bridge {
            _app: tauri_app,
            events,
            alerts,
            window,
        }
    }

    fn notice(sku: &str) -> ConversationNotice {
        ConversationNotice {
            conversation_id: format!("c-{sku}"),
            sku: Some(sku.into()),
            product_title: "Vintage Brass Wall Mirror, Round".into(),
            topic: "Condition".into(),
            kind: NoticeKind::Question,
        }
    }

    fn send(b: &Bridge, notices: Vec<ConversationNotice>, chime: bool, banner: bool) {
        EventSink::emit(
            &b.events,
            AppEvent::ConversationsChanged {
                notices,
                chime,
                banner,
            },
        );
    }

    #[test]
    fn c2_c18a_a_notice_with_chime_chimes_once_and_reaches_the_window() {
        let b = bridge(false);
        send(&b, vec![notice("BEYO-1880")], true, false);
        assert_eq!(*b.alerts.chimes.lock().unwrap(), 1);
        assert!(b.alerts.banners.lock().unwrap().is_empty());
        let window = b.window.lock().unwrap();
        assert_eq!(window.len(), 1);
        assert_eq!(window[0]["toasts"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn c2_c18b_three_notices_make_one_chime() {
        let b = bridge(true);
        send(
            &b,
            vec![notice("BEYO-1"), notice("BEYO-2"), notice("BEYO-3")],
            true,
            false,
        );
        assert_eq!(*b.alerts.chimes.lock().unwrap(), 1);
    }

    #[test]
    fn c2_c18c_muted_means_no_chime_but_the_window_still_hears() {
        let b = bridge(false);
        send(&b, vec![notice("BEYO-1880")], false, false);
        assert_eq!(*b.alerts.chimes.lock().unwrap(), 0);
        assert!(b.alerts.banners.lock().unwrap().is_empty());
        assert_eq!(b.window.lock().unwrap().len(), 1);
    }

    #[test]
    fn c2_c19a_an_unfocused_window_gets_a_banner_with_the_toast_words() {
        let b = bridge(false);
        send(&b, vec![notice("BEYO-1880")], false, true);
        assert_eq!(
            *b.alerts.banners.lock().unwrap(),
            [(
                "Session Manager".to_owned(),
                "New question · BEYO-1880 — Condition".to_owned()
            )]
        );
    }

    #[test]
    fn c2_c19b_a_focused_window_gets_no_banner() {
        let b = bridge(true);
        send(&b, vec![notice("BEYO-1880")], false, true);
        assert!(b.alerts.banners.lock().unwrap().is_empty());
    }

    #[test]
    fn c2_c7e_the_three_commands_never_reach_pricing() {
        let a = connected();
        a.items()
            .create_local("BEYO-1", "BEYO-1", "object", None)
            .unwrap();
        a.fake_pricing()
            .ask(&store_id(&a), Some("BEYO-1"), "Condition");
        run_check(&a);
        let w = wired(a);
        w.pricing.clear_requests();
        list(&w, json!({"kind": "all"}), "open");
        list(&w, json!({"kind": "all"}), "resolved");
        call(&w.window, "conversations_counts", json!({"sessionId": 2})).unwrap();
        call(&w.window, "conversations_session_badges", json!({})).unwrap();
        assert!(w.pricing.request_lines().is_empty());
    }
}

#[cfg(test)]
mod c3_tests {
    //! C3·U1 (`docs/implementation/plans/C3_conversation_and_answering.md` C3·C8(e)): a text reply
    //! to a conversation that is not on this Mac, through `conversation_reply` over the wired app.
    use super::*;
    use psm_core::ports::clock::Clock;
    use psm_testkit::test_app::PRICING_KEY;
    use serde_json::{Value, json};
    use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::webview::InvokeRequest;

    fn call(
        w: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        args: Value,
    ) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }

    #[test]
    fn c3_c8e_a_text_reply_works_for_a_conversation_not_on_this_mac() {
        use psm_core::ports::conversations::ConversationStore;
        let a = psm_testkit::TestApp::with_pricing().unwrap();
        a.clock.advance(1_000_000);
        a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
        let store = a.app.store.sync().unwrap().store.unwrap().id;
        let case = a.fake_pricing().ask(&store, Some("BEYO-9999"), "Condition");
        while a.app.conversations.tick(a.clock.now_ms()).unwrap().more {}
        let pricing = a.pricing.clone().unwrap();
        let service = a.app.conversations.clone();
        let sqlite = a.app.store.clone();
        let clock = a.clock.clone();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::conversations::conversation_reply
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        pricing.clear_requests();
        let pending = call(
            &window,
            "conversation_reply",
            json!({"input": {
                "conversation_id": case,
                "client_mutation_id": "u-1",
                "text": "Answered from elsewhere.",
                "reply_to_message_id": null,
                "attachment_media_ids": [],
            }}),
        )
        .unwrap();
        assert_eq!(pending["client_mutation_id"], json!("u-1"));
        assert_eq!(pending["state"], json!("sending"));
        service.tick(clock.now_ms()).unwrap();
        let posts: Vec<_> = pricing
            .requests()
            .into_iter()
            .filter(|r| r.method == "POST")
            .collect();
        assert_eq!(posts.len(), 1);
        assert_eq!(
            posts[0].body.as_ref().unwrap()["clientMutationId"],
            json!("u-1")
        );
        let row = sqlite
            .outbox()
            .unwrap()
            .into_iter()
            .find(|r| r.client_mutation_id == "u-1")
            .unwrap();
        assert_eq!(row.state, psm_core::domain::conversation::OutboxState::Sent);
    }
}

#[cfg(test)]
mod r6_command_tests {
    use super::*;
    use psm_core::domain::attribute::AiConfidence;
    use psm_core::domain::photo::{Classification, NewPhoto, PhotoSource};
    use psm_core::domain::session::PublishStep;
    use psm_core::events::AppEvent;
    use psm_core::mechanisms::hash::sha256_file;
    use serde_json::{Value, json};
    use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::webview::InvokeRequest;

    const NOW_MS: i64 = 1_762_840_000_000;

    fn call(
        w: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        args: Value,
    ) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }

    /// R6·U3 (R5 review): resolving the last conflict of a held publish item tells the UI
    /// (`session_changed`, `publish_progress`), so the row flips from "Waiting for you" to
    /// "Publishing" without a manual refresh; resolving one of two tells it nothing.
    #[test]
    fn r6_a20_release_emits_session_changed_and_publish_progress() {
        let a = psm_testkit::TestApp::with_fake_shop().unwrap();
        a.clock.advance(NOW_MS);
        a.research_fields(&["k1", "k2"]).unwrap();
        let (item, created) = a
            .items()
            .create_local("R6-REL", "Oak", "Sideboard", None)
            .unwrap();
        let session = created.session.id;
        let dir = a.dir.path().join("Capture/raws");
        std::fs::create_dir_all(&dir).unwrap();
        let raw = dir.join("R6-REL.ARW");
        std::fs::write(&raw, "raw bytes").unwrap();
        let photo = a
            .store()
            .photo_insert_or_get(NewPhoto {
                session_id: session,
                raw_path: raw.to_string_lossy().into_owned(),
                raw_hash: Some(sha256_file(&raw).unwrap()),
                captured_at: Some(NOW_MS),
                source: PhotoSource::Capture,
                created_at: NOW_MS,
            })
            .unwrap();
        a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
        a.review().classify(photo.id, Classification::Main).unwrap();
        a.fake.add_image(&photo.raw_path, &["v1"], false);
        a.research_conflict(item.id, "k1", "Oak", "Walnut", AiConfidence::High)
            .unwrap();
        a.research_conflict(item.id, "k2", "Pine", "Teak", AiConfidence::High)
            .unwrap();
        a.confirm_as_shown(&[session], "Publish").unwrap();
        let events = a.events.clone();
        let psm_testkit::TestApp {
            dir: _dir, mut app, ..
        } = a;
        // The commands read the active definitions of the configured integration's kind.
        app.settings.integration = Some(json!({"kind": psm_testkit::RESEARCH_INTEGRATION}));
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::attributes::attribute_conflict_resolve,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        let told = || {
            let mut out = Vec::new();
            for event in events.0.lock().unwrap().drain(..) {
                match event {
                    AppEvent::SessionChanged { session_id } => {
                        out.push(format!("session_changed {session_id}"))
                    }
                    AppEvent::PublishProgress {
                        session_id, step, ..
                    } => out.push(format!("publish_progress {session_id} {step:?}")),
                    _ => {}
                }
            }
            out
        };
        told();
        call(
            &w,
            "attribute_conflict_resolve",
            json!({"itemId": item.id, "key": "k1", "action": "keep_previous", "value": null}),
        )
        .unwrap();
        assert_eq!(told(), Vec::<String>::new(), "one conflict still waits");
        call(
            &w,
            "attribute_conflict_resolve",
            json!({"itemId": item.id, "key": "k2", "action": "use_suggestion", "value": null}),
        )
        .unwrap();
        assert_eq!(
            told(),
            vec![
                format!("session_changed {session}"),
                format!("publish_progress {session} {:?}", PublishStep::Prepare),
            ]
        );
    }
}

/// H2 (`plans/H2_list_pictures_pages_search.md`): the command rows, through Tauri's IPC.
#[cfg(test)]
mod h2_tests {
    use super::*;
    use serde_json::{Value, json};
    use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::webview::InvokeRequest;

    type Window = tauri::WebviewWindow<tauri::test::MockRuntime>;

    fn call(w: &Window, cmd: &str, args: Value) -> Value {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
        .unwrap_or_else(|error| panic!("{cmd}: {error}"))
    }

    /// The app of `a` behind the H2 commands (the test keeps the temp folder alive).
    fn wired(
        a: psm_testkit::TestApp,
    ) -> (
        tempfile::TempDir,
        tauri::App<tauri::test::MockRuntime>,
        Window,
    ) {
        let psm_testkit::TestApp { dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::sessions::sessions_page,
                commands::sessions::sessions_list,
                commands::items::items_search,
                commands::items::items_search_shop,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        (dir, tauri_app, w)
    }

    fn thirty_sessions() -> psm_testkit::TestApp {
        let a = psm_testkit::TestApp::with_fake_shop().unwrap();
        for i in 0..30 {
            a.clock.advance(60_000);
            a.items()
                .create_local(&format!("H2-{i:02}"), &format!("Item {i}"), "object", None)
                .unwrap();
        }
        a
    }

    fn rows(groups: &Value) -> Vec<Value> {
        groups
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|g| g["rows"].as_array().unwrap().clone())
            .collect()
    }

    #[test]
    fn h2_c8a_sessions_page_answers_one_page() {
        let (_dir, _app, w) = wired(thirty_sessions());
        let page = call(
            &w,
            "sessions_page",
            json!({"filter": "all", "query": null, "createdFrom": null, "onlyIds": null, "page": 2}),
        );
        let rows = rows(&page["groups"]);
        assert_eq!(
            (&page["page"], &page["pages"], &page["total"], rows.len()),
            (&json!(2), &json!(2), &json!(30), 5)
        );
        assert!(rows.iter().all(|r| r["cover_pending"].is_boolean()));
    }

    #[test]
    fn h2_c8b_sessions_list_stays_unpaged() {
        let (_dir, _app, w) = wired(thirty_sessions());
        let list = call(&w, "sessions_list", json!({"filter": "all", "query": null}));
        assert_eq!(rows(&list["groups"]).len(), 30);
    }

    // H2·U4 the searches' pictures (MC-H2.4, MC-H2.5).

    use psm_core::{
        domain::{
            photo::{Classification, NewPhoto, PhotoSource},
            publish::{RemoteMedia, RemoteMediaStatus},
            remote::PictureTarget,
            storage::ArchivedImage,
        },
        events::AppEvent,
        mechanisms::remote::search_name,
        services::remote::RemoteItems,
    };
    use psm_testkit::fake_commerce::{FakeCommerce, ImageFault, fake_image_url};
    use std::sync::Arc;

    /// What a search test keeps of the app after it is behind the commands.
    struct Kept {
        remote: Arc<RemoteItems>,
        shop: Arc<FakeCommerce>,
        events: Arc<psm_testkit::test_app::CollectingEventSink>,
        previews: std::path::PathBuf,
    }

    fn keep(a: &psm_testkit::TestApp) -> Kept {
        Kept {
            remote: a.app.remote.clone(),
            shop: a.commerce.clone().unwrap(),
            events: a.events.clone(),
            previews: a.app.paths.root.join("previews"),
        }
    }

    fn search(w: &Window, cmd: &str, query: &str) -> Vec<Value> {
        call(w, cmd, json!({"query": query}))
            .as_array()
            .unwrap()
            .clone()
    }

    fn pictures_ready(kept: &Kept) -> Vec<PictureTarget> {
        kept.events
            .0
            .lock()
            .unwrap()
            .iter()
            .filter_map(|e| match e {
                AppEvent::PictureReady { target, .. } => Some(target.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn h2_c11a_a_local_result_carries_its_session_picture() {
        let a = psm_testkit::TestApp::with_fake_shop().unwrap();
        let (_, created) = a
            .items()
            .create_local("PIC-1", "Oak chair", "Chair", None)
            .unwrap();
        let session = created.session.id;
        let raw = a.dir.path().join("PIC-1.ARW");
        std::fs::write(&raw, "raw").unwrap();
        let photo = a
            .store()
            .photo_insert_or_get(NewPhoto {
                session_id: session,
                raw_path: raw.to_string_lossy().into_owned(),
                raw_hash: None,
                captured_at: None,
                source: PhotoSource::Capture,
                created_at: 0,
            })
            .unwrap();
        a.review().classify(photo.id, Classification::Main).unwrap();
        let preview = a.dir.path().join("pic-1-preview.jpg");
        std::fs::write(&preview, "jpeg").unwrap();
        rusqlite::Connection::open(a.app.paths.db())
            .unwrap()
            .execute(
                "UPDATE photos SET adjusted_preview_path=?1 WHERE id=?2",
                rusqlite::params![preview.to_string_lossy(), photo.id],
            )
            .unwrap();
        let (_dir, _app, w) = wired(a);
        let found = search(&w, "items_search", "PIC-1");
        assert_eq!(
            (&found[0]["picture_path"], &found[0]["session_id"]),
            (&json!(preview.to_string_lossy()), &json!(session))
        );
    }

    #[test]
    fn h2_c11b_a_local_result_without_a_session_has_no_picture() {
        let a = psm_testkit::TestApp::with_fake_shop().unwrap();
        // A shop product adopted from the search and not opened yet: an item without a session.
        let product = a.fake_shop().add_product("NOSESS-1", "Lamp", None);
        let item = psm_core::services::items::ItemsService::new(a.store(), a.clock.as_ref())
            .with_commerce(Some(a.fake_shop()))
            .adopt(&product)
            .unwrap();
        assert!(a.store().session_for_item(item.id).unwrap().is_none());
        let (_dir, _app, w) = wired(a);
        let found = search(&w, "items_search", "NOSESS-1");
        assert_eq!(
            (&found[0]["picture_path"], &found[0]["session_id"]),
            (&Value::Null, &Value::Null)
        );
    }

    #[test]
    fn h2_c11c_an_archived_result_requests_its_cover() {
        let a = psm_testkit::TestApp::with_fake_shop().unwrap();
        let (item, created) = a
            .items()
            .create_local("ARCH-S", "Desk", "Desk", None)
            .unwrap();
        let product = a.fake_shop().add_product("ARCH-S", "Desk", None);
        a.fake_shop()
            .state
            .lock()
            .unwrap()
            .products
            .iter_mut()
            .find(|p| p.external_id == product)
            .unwrap()
            .media = vec![RemoteMedia {
            id: "arch-m1".into(),
            filename: Some("arch-m1.jpg".into()),
            status: RemoteMediaStatus::Ready,
            position: 1,
        }];
        a.store()
            .transaction(&mut |tx| tx.insert_item_link(item.id, "shopify", &product, None))
            .unwrap();
        a.store()
            .mark_item_cleared(
                item.id,
                0,
                &[ArchivedImage {
                    role: "gallery".into(),
                    position: 1,
                    media_id: "arch-m1".into(),
                    url: fake_image_url("arch-m1"),
                    bytes: None,
                }],
            )
            .unwrap();
        let session = created.session.id;
        let kept = keep(&a);
        let (_dir, _app, w) = wired(a);
        let found = search(&w, "items_search", "ARCH-S");
        assert_eq!(found[0]["picture_path"], Value::Null);
        kept.remote.run_pending().unwrap();
        let cover = kept
            .previews
            .join("remote")
            .join(item.id.to_string())
            .join(psm_core::mechanisms::remote::cover_name("arch-m1"));
        assert!(cover.is_file());
        assert_eq!(
            pictures_ready(&kept),
            vec![PictureTarget::Cover {
                session_id: session
            }]
        );
    }

    /// A fake shop product whose picture address is `url`.
    fn shop_product(a: &psm_testkit::TestApp, sku: &str, url: Option<&str>) -> String {
        let product = a
            .fake_shop()
            .add_product(sku, &format!("{sku} chair"), None);
        a.fake_shop()
            .state
            .lock()
            .unwrap()
            .products
            .iter_mut()
            .find(|p| p.external_id == product)
            .unwrap()
            .thumbnail_url = url.map(Into::into);
        product
    }

    const U: &str = "https://cdn.fake-shop.test/files/shop-u.jpg";

    #[test]
    fn h2_c12c_a_shop_result_picture_is_fetched_small_and_kept() {
        let a = psm_testkit::TestApp::with_fake_shop().unwrap();
        let product = shop_product(&a, "SHOP-U", Some(U));
        let kept = keep(&a);
        let (_dir, _app, w) = wired(a);
        assert_eq!(
            search(&w, "items_search_shop", "SHOP-U")[0]["picture_path"],
            Value::Null
        );
        kept.remote.run_pending().unwrap();
        let file = kept
            .previews
            .join("remote")
            .join("search")
            .join(search_name(U));
        assert!(file.is_file());
        assert_eq!(kept.shop.image_fetches(), vec![(U.to_owned(), 160)]);
        assert_eq!(
            pictures_ready(&kept),
            vec![PictureTarget::Search {
                external_id: product
            }]
        );
        let again = search(&w, "items_search_shop", "SHOP-U");
        kept.remote.run_pending().unwrap();
        assert_eq!(again[0]["picture_path"], json!(file.to_string_lossy()));
        assert_eq!(kept.shop.image_fetches().len(), 1);
    }

    #[test]
    fn h2_c12d_a_shop_result_without_a_picture() {
        let a = psm_testkit::TestApp::with_fake_shop().unwrap();
        shop_product(&a, "SHOP-N", None);
        let kept = keep(&a);
        let (_dir, _app, w) = wired(a);
        assert_eq!(
            search(&w, "items_search_shop", "SHOP-N")[0]["picture_path"],
            Value::Null
        );
        kept.remote.run_pending().unwrap();
        assert!(kept.shop.image_fetches().is_empty());
    }

    #[test]
    fn h2_c12e_a_gone_search_picture_is_dropped() {
        let a = psm_testkit::TestApp::with_fake_shop().unwrap();
        shop_product(&a, "SHOP-G", Some(U));
        a.fake_shop().fail_image(U, vec![ImageFault::Gone]);
        let kept = keep(&a);
        let (_dir, _app, w) = wired(a);
        search(&w, "items_search_shop", "SHOP-G");
        kept.remote.run_pending().unwrap();
        assert!(
            !kept
                .previews
                .join("remote")
                .join("search")
                .join(search_name(U))
                .exists()
        );
        assert!(pictures_ready(&kept).is_empty());
        assert!(kept.shop.url_reads().is_empty());
    }
}

/// H4 (`plans/H4_remote_workspace.md`): the command rows H4·C2(c) and H4·C4, through Tauri's IPC.
#[cfg(test)]
mod h4_tests {
    use super::*;
    use psm_core::{
        domain::{
            publish::{RemoteMedia, RemoteMediaStatus},
            storage::ArchivedImage,
        },
        ports::store::Store,
        services::items::ItemsService,
    };
    use psm_testkit::fake_commerce::fake_image_url;
    use serde_json::{Value, json};
    use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::webview::InvokeRequest;

    type Window = tauri::WebviewWindow<tauri::test::MockRuntime>;

    fn result(w: &Window, cmd: &str, args: Value) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }

    fn call(w: &Window, cmd: &str, args: Value) -> Value {
        result(w, cmd, args).unwrap_or_else(|error| panic!("{cmd}: {error}"))
    }

    fn wired(
        a: psm_testkit::TestApp,
    ) -> (
        tempfile::TempDir,
        tauri::App<tauri::test::MockRuntime>,
        Window,
    ) {
        let psm_testkit::TestApp { dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::sessions::session_activate,
                commands::remote::session_remote_get,
                commands::remote::session_remote_refresh,
                commands::remote::session_bring_back,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        (dir, tauri_app, w)
    }

    fn image(role: &str, position: i64, media_id: &str) -> ArchivedImage {
        ArchivedImage {
            role: role.into(),
            position,
            media_id: media_id.into(),
            url: fake_image_url(media_id),
            bytes: None,
        }
    }

    /// A local item whose files were cleared with a record of 3 gallery and 2 condition pictures
    /// whose sizes are not known. (session id)
    fn archived(a: &psm_testkit::TestApp, sku: &str) -> i64 {
        let (item, session) = a
            .items()
            .create_local(sku, &format!("{sku} title"), "Sideboard", None)
            .unwrap();
        let images = [
            image("gallery", 1, &format!("{sku}-g1")),
            image("gallery", 2, &format!("{sku}-g2")),
            image("gallery", 3, &format!("{sku}-g3")),
            image("condition", 1, &format!("{sku}-c1")),
            image("condition", 2, &format!("{sku}-c2")),
        ];
        a.store()
            .mark_item_cleared(item.id, 1_000, &images)
            .unwrap();
        session.session.id
    }

    #[test]
    fn h4_c2c_session_activate_of_an_archived_item_keeps_the_filing_target() {
        let a = psm_testkit::TestApp::with_camera_and_shop().unwrap();
        let (_, b) = a
            .items()
            .create_local("B-CAM", "Chair", "Chair", None)
            .unwrap();
        let b = b.session.id;
        let archived = archived(&a, "A-CAM");
        let fake = a.fake.clone();
        let store = a.app.store.clone();
        let (_dir, _app, w) = wired(a);
        call(&w, "session_activate", json!({"sessionId": b}));
        assert_eq!(store.active_session_id().unwrap(), Some(b));
        let before = fake.state.lock().unwrap().ops.len();
        assert_eq!(
            call(&w, "session_activate", json!({"sessionId": archived})),
            Value::Null
        );
        assert_eq!(store.active_session_id().unwrap(), Some(b));
        assert_eq!(
            fake.state.lock().unwrap().ops[before..].to_vec(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn h4_c4a_session_remote_get_answers_the_view() {
        let a = psm_testkit::TestApp::with_fake_shop().unwrap();
        let session = archived(&a, "GET-A");
        let (_dir, _app, w) = wired(a);
        let view = call(&w, "session_remote_get", json!({"sessionId": session}));
        assert_eq!(view["state"], json!("archived"));
        assert_eq!(view["pictures"].as_array().unwrap().len(), 5);
        assert_eq!(view["bring_back_bytes"], json!(7_500_000));
        assert_eq!(view["check_failed"], json!(false));
        assert_eq!(view["bring_back"], Value::Null);
        assert_eq!(
            call(&w, "session_remote_get", json!({"sessionId": 1})),
            Value::Null
        );
    }

    #[test]
    fn h4_c4b_session_bring_back_of_a_local_session_is_not_remote() {
        let a = psm_testkit::TestApp::with_fake_shop().unwrap();
        let (_, local) = a
            .items()
            .create_local("LOC-B", "Lamp", "Lamp", None)
            .unwrap();
        let (_dir, _app, w) = wired(a);
        let error = result(
            &w,
            "session_bring_back",
            json!({"sessionId": local.session.id}),
        )
        .unwrap_err();
        assert_eq!(error["code"], json!("not_remote"));
    }

    #[test]
    fn h4_c4c_session_remote_refresh_answers_the_new_order() {
        let a = psm_testkit::TestApp::with_fake_shop().unwrap();
        let product = a.fake_shop().add_product("REF-C", "Cabinet", None);
        let set_gallery = |ids: &[&str]| {
            let mut s = a.fake_shop().state.lock().unwrap();
            let p = s
                .products
                .iter_mut()
                .find(|p| p.external_id == product)
                .unwrap();
            p.media = ids
                .iter()
                .enumerate()
                .map(|(i, id)| RemoteMedia {
                    id: (*id).into(),
                    filename: Some(format!("{id}.jpg")),
                    status: RemoteMediaStatus::Ready,
                    position: i as i64 + 1,
                })
                .collect();
        };
        set_gallery(&["r-m1", "r-m2", "r-m3"]);
        let item = ItemsService::new(a.store(), a.clock.as_ref())
            .with_commerce(Some(a.fake_shop()))
            .adopt(&product)
            .unwrap();
        let session = a.sessions().create(item.id).unwrap().session.id;
        a.store()
            .mark_item_cleared(
                item.id,
                1_000,
                &[
                    image("gallery", 1, "r-m1"),
                    image("gallery", 2, "r-m2"),
                    image("gallery", 3, "r-m3"),
                ],
            )
            .unwrap();
        // Reordered in the shop's admin.
        set_gallery(&["r-m3", "r-m1", "r-m2"]);
        let (_dir, _app, w) = wired(a);
        let view = call(&w, "session_remote_refresh", json!({"sessionId": session}));
        let gallery: Vec<(Value, Value)> = view["pictures"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["role"] == "gallery")
            .map(|p| (p["position"].clone(), p["media_id"].clone()))
            .collect();
        assert_eq!(
            gallery,
            vec![
                (json!(1), json!("r-m3")),
                (json!(2), json!("r-m1")),
                (json!(3), json!("r-m2")),
            ]
        );
    }
}

#[cfg(test)]
mod p39_tests {
    use super::*;
    use psm_core::domain::photo::Classification;
    use serde_json::{Value, json};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::{
        Listener,
        ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
        webview::InvokeRequest,
    };

    fn result(
        webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        sid: i64,
    ) -> Result<Value, Value> {
        get_ipc_response(
            webview,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(json!({"sessionId":sid})),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }
    fn setup() -> (psm_testkit::TestApp, i64, Vec<i64>) {
        let a = psm_testkit::TestApp::with_fake_document().unwrap();
        let sid = a
            .items()
            .create_local("s", "s", "object", None)
            .unwrap()
            .1
            .session
            .id;
        let ids = ["a", "b"]
            .iter()
            .map(|name| {
                let id = a.add_photo(sid, name).unwrap().id;
                a.review().classify(id, Classification::Main).unwrap();
                id
            })
            .collect();
        (a, sid, ids)
    }
    #[test]
    fn c7a_history_commands() {
        let (a, s, ids) = setup();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::history::history_get,
                commands::history::history_undo,
                commands::history::history_redo
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        assert_eq!(
            result(&w, "history_get", s).unwrap(),
            json!({"undo":"Approve 02","redo":null})
        );
        let undo = result(&w, "history_undo", s).unwrap();
        assert_eq!(
            (
                undo["label"].as_str(),
                undo["direction"].as_str(),
                undo["active_photo_id"].as_i64()
            ),
            (Some("Approve 02"), Some("undone"), Some(ids[1]))
        );
        assert_eq!(undo["sessions"][0]["id"], s);
        assert_eq!(
            result(&w, "history_get", s).unwrap(),
            json!({"undo":"Approve 01","redo":"Approve 02"})
        );
        let redo = result(&w, "history_redo", s).unwrap();
        assert_eq!(
            (redo["label"].as_str(), redo["direction"].as_str()),
            (Some("Approve 02"), Some("redone"))
        );
        assert_eq!(
            result(&w, "history_redo", s).unwrap_err()["code"],
            "nothing_to_redo"
        );
    }
    #[test]
    fn c7c_tauri_history_changed_event() {
        let (a, s, _ids) = setup();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![commands::history::history_undo])
            .build(mock_context(noop_assets()))
            .unwrap();
        let handles = tauri_app.state::<AppState>().handles().unwrap();
        handles
            .events
            .set_target(Arc::new(TauriEvents::silent(tauri_app.handle().clone())));
        let payloads = Arc::new(std::sync::Mutex::new(Vec::<Value>::new()));
        let seen = payloads.clone();
        tauri_app.listen_any("history_changed", move |event| {
            seen.lock()
                .unwrap()
                .push(serde_json::from_str(event.payload()).unwrap());
        });
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        result(&w, "history_undo", s).unwrap();
        assert_eq!(
            *payloads.lock().unwrap(),
            vec![json!({"session_id":s,"undo":"Approve 01","redo":"Approve 02"})]
        );
    }
}

#[cfg(test)]
mod p40_tests {
    use super::*;
    use serde_json::{Value, json};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::{
        ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
        webview::InvokeRequest,
    };

    fn call(w: &tauri::WebviewWindow<tauri::test::MockRuntime>, cmd: &str, args: Value) -> Value {
        let body = get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .unwrap();
        match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        }
    }
    #[test]
    fn c2c_composition_apply_command_is_one_step() {
        let a = psm_testkit::TestApp::with_fake_document().unwrap();
        let s = a
            .items()
            .create_local("K", "Oak", "object", None)
            .unwrap()
            .1
            .session
            .id;
        let ids: Vec<_> = ["q.ARW", "r.ARW"]
            .iter()
            .map(|name| {
                let p = a.add_photo(s, name).unwrap();
                a.store().upsert_raw_state(&p.raw_path, None).unwrap();
                a.fake
                    .add_image(&p.raw_path, &[&format!("v{}", p.id)], false);
                a.app.previews.request_initial(&p.raw_path).unwrap();
                let job = a
                    .fake
                    .state
                    .lock()
                    .unwrap()
                    .issued
                    .last()
                    .unwrap()
                    .0
                    .clone();
                a.complete_render_with(
                    &job,
                    &psm_testkit::fixtures::synthetic_jpeg::SyntheticJpeg::rgb(
                        2560,
                        1707,
                        [80, 80, 80],
                    ),
                )
                .unwrap();
                p.id
            })
            .collect();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::compositions::composition_apply,
                commands::history::history_get
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        let v1 = json!({"kind":"normal","shape":"1:1","orientation":"named","quarter_turns":0,"flip":"none","straighten":0.0,"rect":{"x":0.25,"y":(1.0-1280.0/1707.0)/2.0,"w":0.5,"h":1280.0/1707.0}});
        let mut v2 = v1.clone();
        v2["rect"]["x"] = json!(0.3);
        assert_eq!(
            call(
                &w,
                "composition_apply",
                json!({"items":[{"photo_id":ids[0],"values":v1},{"photo_id":ids[1],"values":v2}]})
            ),
            json!({"applied":ids,"refused":[]})
        );
        assert_eq!(
            call(&w, "history_get", json!({"sessionId":s})),
            json!({"undo":"Apply framing to 2 photos","redo":null})
        );
    }
    #[test]
    fn c7b_attribute_command_undo_restores_absence() {
        let a = psm_testkit::TestApp::with_fake_document().unwrap();
        let (item, session) = a.items().create_local("K", "Oak", "object", None).unwrap();
        let s = session.session.id;
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::attributes::attribute_values_set,
                commands::attributes::attribute_values_get,
                commands::history::history_undo
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        call(
            &w,
            "attribute_values_set",
            json!({"itemId":item.id,"key":"product.description","value":"\"Scuffed top\""}),
        );
        let out = call(&w, "history_undo", json!({"sessionId":s}));
        assert_eq!(out["label"], "Edit Description");
        assert_eq!(out["direction"], "undone");
        assert_eq!(out["active_photo_id"], Value::Null);
        assert_eq!(out["sessions"].as_array().unwrap().len(), 1);
        assert_eq!(out["sessions"][0]["id"], s);
        assert!(
            call(&w, "attribute_values_get", json!({"itemId":item.id}))
                .as_array()
                .unwrap()
                .iter()
                .all(|v| v["key"] != "product.description")
        );
    }
}
/// C1 (`master_plan_conversations.md` §4 Event): `conversations_changed {toasts, more}`.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
struct ConversationToast {
    conversation_id: String,
    text: String,
}
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
struct ConversationsChangedPayload {
    toasts: Vec<ConversationToast>,
    more: usize,
}
impl ConversationsChangedPayload {
    fn from_notices(notices: &[psm_core::domain::conversation::ConversationNotice]) -> Self {
        let (lines, more) = psm_core::mechanisms::conversation_rules::notice_lines(notices);
        Self {
            toasts: lines
                .into_iter()
                .map(|(conversation_id, text)| ConversationToast {
                    conversation_id,
                    text,
                })
                .collect(),
            more,
        }
    }
}
#[derive(serde::Serialize, Clone)]
struct SessionChangedPayload {
    session_id: i64,
}
#[derive(serde::Serialize, Clone)]
struct HistoryChangedPayload {
    session_id: i64,
    undo: Option<String>,
    redo: Option<String>,
}
#[derive(serde::Serialize, Clone)]
struct ExportProgressPayload {
    session_id: i64,
    done: usize,
    total: usize,
    state: psm_core::events::ExportState,
}
#[derive(serde::Serialize, Clone)]
struct PublishProgressPayload {
    session_id: i64,
    item_id: i64,
    step: psm_core::domain::session::PublishStep,
    uploaded: i64,
    total: i64,
}
/// H2, H4 (master plan archive §4 Events): `picture_ready {kind: 'cover' | 'search' | 'remote',
/// session_id?, external_id?, role?, position?, path}`.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
struct PictureReadyPayload {
    #[serde(flatten)]
    target: psm_core::domain::remote::PictureTarget,
    path: String,
}
/// L1 (`master_plan_labels.md` §4 Event): `labels_changed {product_ids, access}`.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
struct LabelsChangedPayload {
    product_ids: Vec<String>,
    access: bool,
}
/// H3 (master plan archive §4 Events): `bring_back_progress {session_id, done, total, state,
/// error}`.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
struct BringBackPayload {
    session_id: i64,
    done: usize,
    total: usize,
    state: psm_core::domain::remote::BringBackState,
    error: Option<String>,
}
impl<R: tauri::Runtime> EventSink for TauriEvents<R> {
    fn emit(&self, event: AppEvent) {
        match event {
            AppEvent::ResearchChanged { session_id } => {
                let _ = self
                    .app
                    .emit("research_changed", SessionChangedPayload { session_id });
            }
            AppEvent::CameraStatusChanged { status } => {
                let link = self
                    .app
                    .try_state::<AppState>()
                    .and_then(|state| state.handles().ok())
                    .and_then(|handles| handles.camera_runtime.direct_adapter().current_link());
                let _ = self.app.emit(
                    "camera_status_changed",
                    commands::camera::CameraStatusResponse::from(status).with_link(link),
                );
            }
            AppEvent::FingerprintPending {
                fingerprint,
                replace,
            } => {
                let _ = self.app.emit(
                    "camera_fingerprint_pending",
                    serde_json::json!({"fingerprint": fingerprint, "replace": replace}),
                );
            }
            AppEvent::HistoryChanged {
                session_id,
                undo,
                redo,
            } => {
                let _ = self.app.emit(
                    "history_changed",
                    HistoryChangedPayload {
                        session_id,
                        undo,
                        redo,
                    },
                );
            }
            AppEvent::PhotoAdded {
                photo_id,
                session_id,
            } => {
                let _ = self.app.emit(
                    "photo_added",
                    PhotoAddedPayload {
                        photo_id,
                        session_id,
                    },
                );
            }
            AppEvent::PhotoChanged {
                photo_id,
                session_id,
            } => {
                let _ = self.app.emit(
                    "photo_changed",
                    PhotoAddedPayload {
                        photo_id,
                        session_id,
                    },
                );
            }
            AppEvent::SessionChanged { session_id } => {
                let _ = self
                    .app
                    .emit("session_changed", SessionChangedPayload { session_id });
            }
            AppEvent::ExportProgress {
                session_id,
                done,
                total,
                state,
            } => {
                let _ = self.app.emit(
                    "export_progress",
                    ExportProgressPayload {
                        session_id,
                        done,
                        total,
                        state,
                    },
                );
            }
            AppEvent::PublishProgress {
                session_id,
                item_id,
                step,
                uploaded,
                total,
            } => {
                let _ = self.app.emit(
                    "publish_progress",
                    PublishProgressPayload {
                        session_id,
                        item_id,
                        step,
                        uploaded,
                        total,
                    },
                );
            }
            AppEvent::HealthChanged => {
                let _ = self.app.emit("health_changed", ());
            }
            AppEvent::PictureReady { target, path } => {
                let _ = self
                    .app
                    .emit("picture_ready", PictureReadyPayload { target, path });
            }
            AppEvent::BringBack {
                session_id,
                done,
                total,
                state,
                error,
            } => {
                let _ = self.app.emit(
                    "bring_back_progress",
                    BringBackPayload {
                        session_id,
                        done,
                        total,
                        state,
                        error,
                    },
                );
            }
            // C1: the toasts' words; C2: one chime, and a Mac banner per line while the main
            // window does not have the focus (09B §9).
            AppEvent::ConversationsChanged {
                notices,
                chime,
                banner,
            } => {
                let payload = ConversationsChangedPayload::from_notices(&notices);
                let _ = self.app.emit("conversations_changed", payload.clone());
                if chime {
                    self.alerts.chime();
                }
                if banner && !(self.focused)() {
                    let lines: Vec<String> =
                        payload.toasts.iter().map(|t| t.text.clone()).collect();
                    for body in alerts::banners(&lines, payload.more) {
                        self.alerts.banner(alerts::BANNER_TITLE, &body);
                    }
                }
            }
            // L1 (`master_plan_labels.md` §4 Event): the markers' products, or the access.
            AppEvent::LabelsChanged {
                product_ids,
                access,
            } => {
                let _ = self.app.emit(
                    "labels_changed",
                    LabelsChangedPayload {
                        product_ids,
                        access,
                    },
                );
            }
        }
    }
}
use tauri_specta::{Builder, collect_commands};

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::health::health_get,
        commands::health::app_paths_get,
        commands::items::item_create,
        commands::items::item_types_list,
        commands::items::item_type_set,
        commands::items::item_refresh,
        commands::items::item_adopt,
        commands::items::item_update_identity,
        commands::items::items_search,
        commands::items::items_search_shop,
        commands::items::items_next_sku,
        commands::sessions::session_create,
        commands::sessions::session_get,
        commands::sessions::sessions_list,
        commands::sessions::sessions_page,
        commands::sessions::session_activate,
        commands::sessions::session_deactivate,
        commands::sessions::session_active,
        commands::sessions::session_arrange_next,
        commands::sessions::session_delete_preview,
        commands::sessions::session_delete,
        commands::sessions::session_files_misplaced,
        commands::sessions::session_files_fix,
        commands::camera::camera_status,
        commands::camera::camera_retry,
        commands::camera::camera_take_back_availability,
        commands::camera::camera_capture,
        commands::camera::camera_reconnect,
        commands::camera::camera_files_not_received_dismiss,
        commands::camera::camera_take_back,
        commands::camera::camera_test_connection,
        commands::camera::camera_helper_log,
        commands::camera::camera_support_get,
        commands::camera::camera_support_install,
        commands::camera::camera_secrets_set,
        commands::camera::camera_ftp_password_generate,
        commands::camera::camera_fingerprint_confirm,
        commands::camera::camera_fingerprint_reject,
        commands::camera::camera_fingerprint_pending,
        commands::camera::camera_ftp_password_get,
        commands::camera::camera_ftp_address,
        commands::photos::photos_classify,
        commands::photos::photos_reorder,
        commands::photos::photos_delete,
        commands::photos::session_keep_published,
        commands::photos::photos_preview_focus,
        commands::photos::photos_move,
        commands::photos::photos_copy,
        commands::photos::photos_import,
        commands::photos::photos_assign_unassigned,
        commands::history::history_get,
        commands::history::history_undo,
        commands::history::history_redo,
        commands::history::edit_menu_set,
        commands::compositions::composition_set,
        commands::compositions::composition_apply,
        commands::compositions::composition_reset,
        commands::export::export_run,
        commands::export::export_open_folder,
        commands::export::integration_get,
        commands::attributes::attributes_definitions_list,
        commands::attributes::attribute_values_get,
        commands::attributes::attribute_values_set,
        commands::attributes::attribute_ai_accept_all,
        commands::attributes::attribute_ai_clear,
        commands::attributes::attribute_ai_accept,
        commands::attributes::attribute_conflict_resolve,
        commands::attributes::presets_get,
        commands::attributes::presets_set,
        commands::attributes::form_get,
        commands::attributes::form_read_integration,
        commands::attributes::attribute_default_set,
        commands::attributes::attribute_saved_value_add,
        commands::attributes::attribute_saved_value_remove,
        commands::attributes::attribute_fill_defaults,
        commands::research::research_status,
        commands::research::research_overview,
        commands::research::research_run,
        commands::research::research_settings_reset,
        commands::research::research_secrets_set,
        commands::research::research_secrets_state,
        commands::research::research_site_add,
        commands::research::research_site_remove,
        commands::research::research_sites_reorder,
        commands::research::research_export,
        commands::writer::writer_note,
        commands::writer::writer_look_again,
        commands::writer::writer_status,
        commands::writer::writer_request,
        commands::writer::writer_finding_answer,
        commands::writer::writer_finding_dismiss,
        commands::writer::writer_settings_get,
        commands::writer::writer_settings_save,
        commands::writer::writer_settings_export,
        commands::writer::writer_settings_import,
        commands::writer::writer_instruction_reset,
        commands::writer::writer_preview,
        commands::conversations::conversations_settings_get,
        commands::conversations::conversations_settings_save,
        commands::conversations::conversations_key_set,
        commands::conversations::conversations_test,
        commands::labels::labels_status,
        commands::labels::labels_list,
        commands::labels::labels_summary,
        commands::labels::labels_select_all,
        commands::labels::labels_open,
        commands::conversations::conversations_list,
        commands::conversations::conversations_counts,
        commands::conversations::conversations_session_badges,
        commands::conversations::conversation_get,
        commands::conversations::conversation_thread,
        commands::conversations::conversation_mark_read,
        commands::conversations::conversation_reply,
        commands::conversations::conversation_retry,
        commands::conversations::conversation_discard,
        commands::conversations::conversation_mark_answered,
        commands::conversations::conversation_attachable,
        commands::search::search_window_open,
        commands::search::search_web_place,
        commands::search::search_web_go,
        commands::search::search_web_step,
        commands::search::search_web_zoom,
        commands::search::search_tab_close,
        commands::search::search_lens_start,
        commands::search::search_lens_give_up,
        commands::search::search_lens_results,
        commands::search::search_web_selection,
        commands::search::search_matches,
        commands::search::search_visual,
        commands::search::search_pages,
        commands::search::search_page_save,
        commands::search::search_page_remove,
        commands::search::search_put_text,
        commands::search::search_put_back,
        commands::search::search_put_value,
        commands::search::search_open_review,
        commands::search::search_ask,
        commands::publish::publish_preview,
        commands::publish::publish_confirm,
        commands::publish::publish_retry,
        commands::publish::publish_activity,
        commands::publish::publish_cancel,
        commands::publish::stock_locations,
        commands::publish::stock_default_get,
        commands::publish::stock_default_set,
        commands::publish::item_stock_get,
        commands::publish::item_stock_set,
        commands::publish::notifications_list,
        commands::settings::settings_get,
        commands::settings::settings_set,
        commands::settings::editor_styles,
        commands::settings::storage_get,
        commands::settings::storage_min_free_set,
        commands::settings::storage_retention_get,
        commands::settings::storage_retention_set,
        commands::items::item_keep_files_set,
        commands::remote::session_remote_get,
        commands::remote::session_remote_refresh,
        commands::remote::session_bring_back,
        commands::templates::templates_list,
        commands::templates::template_get,
        commands::templates::template_create,
        commands::templates::template_delete,
        commands::templates::template_container_add,
        commands::templates::template_container_remove,
        commands::templates::template_container_example_set,
        commands::templates::template_containers_reorder,
        commands::templates::template_shooting_order_set,
        commands::templates::template_container_presets_set,
        commands::templates::template_container_label_set,
        commands::templates::template_session_view,
        commands::templates::template_drop,
        commands::templates::template_back,
        commands::templates::template_shift_back,
    ])
}

pub fn open_logged(
    paths: AppPaths,
    open: impl FnOnce(AppPaths) -> psm_core::Result<psm_integrations::App>,
) -> psm_core::Result<(
    Option<tracing_appender::non_blocking::WorkerGuard>,
    psm_integrations::App,
)> {
    paths.create()?;
    let guard = psm_infra::logging::init(&paths);
    let app = open(paths)?;
    Ok((guard, app))
}

pub fn run() {
    let paths = AppPaths::from_env_or_default();
    let (_logging, app) =
        open_logged(paths, psm_integrations::App::open).expect("open Session Manager data");
    let builder = specta_builder();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // C2: the Mac banners for conversations (used from Rust only).
        .plugin(tauri_plugin_notification::init())
        // A6: research sources open in the default browser (capability limited to http/https).
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new(app))
        // A12: the "Search this photo" windows' Google Lens routes.
        .manage(search_window::SearchWindows::default())
        .menu(|handle| {
            let menu = Menu::default(handle)?;
            let undo = MenuItem::with_id(handle, "edit.undo", "Undo", false, Some("CmdOrCtrl+Z"))?;
            let redo = MenuItem::with_id(handle, "edit.redo", "Redo", false, Some("CmdOrCtrl+Shift+Z"))?;
            let edit = Submenu::with_id_and_items(handle, "edit", "Edit", true, &[
                &undo, &redo, &PredefinedMenuItem::separator(handle)?,
                &PredefinedMenuItem::cut(handle, None)?,
                &PredefinedMenuItem::copy(handle, None)?,
                &PredefinedMenuItem::paste(handle, None)?,
                &PredefinedMenuItem::select_all(handle, None)?,
            ])?;
            let position = menu.items()?.iter().position(|item| matches!(item, MenuItemKind::Submenu(sub) if sub.text().ok().as_deref() == Some("Edit"))).expect("default Edit menu");
            menu.remove_at(position)?;
            menu.insert(&edit, position)?;
            handle.manage(EditMenu { undo, redo });
            // A18: page zoom of the search window's current tab (⌘+ / ⌘− / ⌘0). Menu key
            // equivalents reach the app also while the web page has the keyboard, and the page
            // itself never talks to the app.
            if let Some(MenuItemKind::Submenu(view)) = menu.items()?.into_iter().find(|item| matches!(item, MenuItemKind::Submenu(sub) if sub.text().ok().as_deref() == Some("View"))) {
                view.append_items(&[
                    &PredefinedMenuItem::separator(handle)?,
                    &MenuItem::with_id(handle, "view.zoom_in", "Zoom In", true, Some("CmdOrCtrl+NumpadAdd"))?,
                    &MenuItem::with_id(handle, "view.zoom_in_eq", "Zoom In", true, Some("CmdOrCtrl+="))?,
                    &MenuItem::with_id(handle, "view.zoom_out", "Zoom Out", true, Some("CmdOrCtrl+-"))?,
                    &MenuItem::with_id(handle, "view.zoom_reset", "Actual Size", true, Some("CmdOrCtrl+0"))?,
                ])?;
            }
            Ok(menu)
        })
        .on_menu_event(|app, event| {
            let zoom = match event.id().as_ref() {
                "view.zoom_in" | "view.zoom_in_eq" => Some("in"),
                "view.zoom_out" => Some("out"),
                "view.zoom_reset" => Some("reset"),
                _ => None,
            };
            if let Some(action) = zoom {
                // Only a focused search window zooms (its app page applies it to the current tab).
                let focused = app.windows().into_iter().find_map(|(label, window)| {
                    search_window::session_of_window(&label).filter(|_| window.is_focused().unwrap_or(false))
                });
                if let Some(session_id) = focused {
                    let _ = app.emit_to(
                        search_window::ui_label(session_id).as_str(),
                        "search_zoom",
                        search_window::ZoomPayload { session_id, action },
                    );
                }
                return;
            }
            let action = match event.id().as_ref() {
                "edit.undo" => "undo",
                "edit.redo" => "redo",
                _ => return,
            };
            let _ = app.emit("edit_menu", serde_json::json!({ "action": action }));
        })
        .setup(|handle| {
            let state = handle.state::<AppState>();
            // C2: whether the main window has the focus, kept by its focus events (the banner rule).
            let focused = Arc::new(std::sync::atomic::AtomicBool::new(
                handle
                    .get_webview_window("main")
                    .and_then(|window| window.is_focused().ok())
                    .unwrap_or(true),
            ));
            let (previews, templates) = {
                let app = state.handles().expect("app state");
                let reader = focused.clone();
                app.events.set_target(Arc::new(TauriEvents::new(
                    handle.handle().clone(),
                    Arc::new(alerts::SystemAlerts::new(handle.handle().clone(), app.paths.root.clone())),
                    Arc::new(move || reader.load(std::sync::atomic::Ordering::Relaxed)),
                )));
                (app.paths.root.join("previews"), app.paths.root.join("templates"))
            };
            // The workspace loads preview files through the asset protocol; only the previews folder is readable.
            handle
                .asset_protocol_scope()
                .allow_directory(previews, true)?;
            handle.asset_protocol_scope().allow_directory(templates, true)?;
            if let Some(window) = handle.get_webview_window("main") {
                let app_handle = handle.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(now) = event {
                        focused.store(*now, std::sync::atomic::Ordering::Relaxed);
                    }
                    if matches!(event, tauri::WindowEvent::Focused(true)) {
                        let state = app_handle.state::<AppState>();
                        if let Ok(handles) = state.handles() {
                            handles.on_window_focus();
                        }
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(builder.invoke_handler())
        .build(tauri::generate_context!())
        .expect("build Session Manager")
        .run(|handle, event| {
            if let tauri::RunEvent::Exit = event
                && let Ok(h) = handle.state::<AppState>().handles()
            {
                h.camera_runtime.shutdown();
            }
        });
}

#[cfg(test)]
mod p42_tests {
    use super::*;
    use serde_json::{Value, json};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::{
        Listener,
        ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
        webview::InvokeRequest,
    };
    fn call(w: &tauri::WebviewWindow<tauri::test::MockRuntime>, cmd: &str) -> Value {
        let body = get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(json!({})),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .unwrap();
        match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        }
    }
    #[test]
    fn c8a_camera_commands_emit_retry_through_hub() {
        let a = psm_testkit::TestApp::with_fake_document().unwrap();
        let fake = a.fake.clone();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::camera::camera_status,
                commands::camera::camera_retry
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let handles = tauri_app.state::<AppState>().handles().unwrap();
        handles
            .events
            .set_target(Arc::new(TauriEvents::silent(tauri_app.handle().clone())));
        let payloads = Arc::new(std::sync::Mutex::new(Vec::<Value>::new()));
        let seen = payloads.clone();
        tauri_app.listen_any("camera_status_changed", move |event| {
            seen.lock()
                .unwrap()
                .push(serde_json::from_str(event.payload()).unwrap());
        });
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        assert_eq!(
            call(&w, "camera_status"),
            json!({"kind":"connected","name":"Sony ZV-E10","link":"cable"})
        );
        assert!(payloads.lock().unwrap().is_empty());
        fake.state.lock().unwrap().camera = None;
        assert_eq!(call(&w, "camera_retry"), json!({"kind":"disconnected"}));
        assert_eq!(
            *payloads.lock().unwrap(),
            vec![json!({"kind":"disconnected"})]
        );
    }
}

#[cfg(test)]
mod p46_tests {
    use super::*;
    use psm_core::{domain::camera::ExposureStatus, ports::clock::Clock};
    use psm_integrations::camera::sidecar::protocol::SidecarEvent;
    use serde_json::{Value, json};
    use std::time::{Duration, Instant};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::{
        ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
        webview::InvokeRequest,
    };
    const SCRIPTS: &str = "../../../crates/testkit/src/fixtures/sidecar_scripts";

    fn script(names: &[&str]) -> String {
        names
            .iter()
            .map(|n| std::fs::read_to_string(format!("{SCRIPTS}/{n}")).unwrap())
            .collect()
    }
    fn settle(a: &psm_testkit::TestApp, pred: impl Fn() -> bool) {
        let end = Instant::now() + Duration::from_secs(10);
        while Instant::now() < end {
            a.tick(0).unwrap();
            if pred() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("camera result did not settle: {:?}", a.sidecar_events());
    }
    /// The command's answer: `Ok(value)` or `Err(CommandError as JSON)`.
    fn call(w: &tauri::WebviewWindow<tauri::test::MockRuntime>, cmd: &str) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(json!({})),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }
    fn mock(
        app: psm_integrations::App,
    ) -> (
        tauri::App<tauri::test::MockRuntime>,
        tauri::WebviewWindow<tauri::test::MockRuntime>,
    ) {
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::camera::camera_capture,
                commands::camera::camera_reconnect,
                commands::camera::camera_files_not_received_dismiss,
                commands::health::health_get
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        (tauri_app, w)
    }
    fn connected(a: &psm_testkit::TestApp) -> bool {
        matches!(
            a.camera_status().unwrap(),
            psm_core::domain::camera::CameraStatus::Connected { .. }
        )
    }
    fn active_session(a: &psm_testkit::TestApp) -> i64 {
        let (_, session) = a
            .items()
            .create_local("P46-CMD", "P46 command", "object", None)
            .unwrap();
        a.sessions().activate(session.session.id).unwrap();
        session.session.id
    }

    #[test]
    fn c8b_camera_capture_answers_the_capture_id() {
        let a = psm_testkit::TestApp::with_camera_script(&script(&[
            "ready_usb.txt",
            "capture_usb.txt",
        ]))
        .unwrap();
        settle(&a, || connected(&a));
        active_session(&a);
        let runtime = a.camera_runtime();
        let psm_testkit::TestApp {
            dir: _dir,
            app,
            clock,
            ..
        } = a;
        let (tauri_app, w) = mock(app);
        let answer = call(&w, "camera_capture").unwrap();
        let id = answer["capture_id"].as_str().unwrap().to_owned();
        assert_eq!(answer, json!({ "capture_id": id }));
        // A v4 uuid: 8-4-4-4-12 hex digits.
        let groups: Vec<usize> = id.split('-').map(str::len).collect();
        assert_eq!(groups, vec![8, 4, 4, 4, 12], "{id}");
        assert!(
            id.chars().all(|c| c == '-' || c.is_ascii_hexdigit()),
            "{id}"
        );
        let end = Instant::now() + Duration::from_secs(10);
        while !runtime
            .sidecar_events()
            .iter()
            .any(|e| matches!(e.event, SidecarEvent::CmdDone { .. }))
        {
            assert!(Instant::now() < end, "no cmd.done");
            runtime.tick(clock.mono());
            std::thread::sleep(Duration::from_millis(10));
        }
        let handles = tauri_app.state::<AppState>().handles().unwrap();
        let rows = handles
            .camera_runtime
            .ledger()
            .unwrap()
            .exposures(None)
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].capture_id.as_deref(), Some(id.as_str()));
        assert_eq!(
            call(&w, "camera_reconnect").unwrap_err()["code"],
            "reconnect_on_cable_refused"
        );
    }

    #[test]
    fn c6c_dismiss_command_clears_the_notice() {
        let a =
            psm_testkit::TestApp::with_camera_script(&script(&["ready_wifi.txt", "capture.txt"]))
                .unwrap();
        settle(&a, || connected(&a));
        active_session(&a);
        a.capture().unwrap();
        settle(&a, || a.ledger().exposures(None).unwrap().len() == 1);
        a.clock.advance(61_000);
        for _ in 0..3 {
            a.tick(1_000).unwrap();
        }
        assert!(a.app.health.has("camera_files_not_received"));
        let runtime = a.camera_runtime();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let (_tauri_app, w) = mock(app);
        assert_eq!(
            call(&w, "camera_files_not_received_dismiss").unwrap(),
            Value::Null
        );
        let health = call(&w, "health_get").unwrap();
        assert!(
            !health["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|i| i["code"] == "camera_files_not_received"),
            "{health}"
        );
        assert_eq!(
            runtime.ledger().unwrap().exposures(None).unwrap()[0].status,
            ExposureStatus::Dismissed
        );
    }
}

#[cfg(test)]
mod p48_tests {
    use super::*;
    use psm_core::ports::clock::Clock;
    use serde_json::{Value, json};
    use std::{
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        time::{Duration, Instant},
    };
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::{
        ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
        webview::InvokeRequest,
    };
    const SCRIPTS: &str = "../../../crates/testkit/src/fixtures/sidecar_scripts";
    const CO_SENTENCE: &str =
        "Capture One is holding the camera's USB connection — Take the camera back";

    fn script(name: &str) -> String {
        std::fs::read_to_string(format!("{SCRIPTS}/{name}")).unwrap()
    }
    fn settle(a: &psm_testkit::TestApp, pred: impl Fn() -> bool) {
        let end = Instant::now() + Duration::from_secs(10);
        while Instant::now() < end {
            a.tick(0).unwrap();
            if pred() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("camera result did not settle: {:?}", a.sidecar_events());
    }
    fn call(w: &tauri::WebviewWindow<tauri::test::MockRuntime>, cmd: &str) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(json!({})),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }
    fn mock(
        app: psm_integrations::App,
    ) -> (
        tauri::App<tauri::test::MockRuntime>,
        tauri::WebviewWindow<tauri::test::MockRuntime>,
    ) {
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::camera::camera_take_back,
                commands::camera::camera_test_connection
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        (tauri_app, w)
    }
    /// Fixture rule (iv): while the command blocks, a ticker thread advances the clock 500 ms and
    /// ticks the camera runtime every 10 ms.
    fn with_ticker<T>(
        runtime: Arc<psm_integrations::camera::runtime::CameraRuntime>,
        clock: Arc<psm_testkit::clock::ManualClock>,
        f: impl FnOnce() -> T,
    ) -> T {
        let stop = Arc::new(AtomicBool::new(false));
        let ticker = {
            let stop = stop.clone();
            std::thread::spawn(move || {
                while !stop.load(Ordering::SeqCst) {
                    clock.advance(500);
                    runtime.tick(clock.mono());
                    std::thread::sleep(Duration::from_millis(10));
                }
            })
        };
        let out = f();
        stop.store(true, Ordering::SeqCst);
        ticker.join().unwrap();
        out
    }
    /// C1(e)/C5(a)'s start: Capture One holds the interface over a Wi-Fi session.
    fn held(name: &str) -> psm_testkit::TestApp {
        let a = psm_testkit::TestApp::with_camera_script(&script(name)).unwrap();
        a.fake_processes.set(vec!["Capture One"]);
        settle(&a, || a.app.health.has("camera_held"));
        a
    }

    #[test]
    fn c5g_camera_take_back_command() {
        let a = held("take_back.txt");
        let (_, session) = a
            .items()
            .create_local("P48-CMD", "P48 command", "object", None)
            .unwrap();
        a.sessions().activate(session.session.id).unwrap();
        {
            let mut f = a.fake.state.lock().unwrap();
            f.quit_after_polls = Some(1);
            f.running_after_polls = Some(1);
        }
        let runtime = a.camera_runtime();
        let psm_testkit::TestApp {
            dir: _dir,
            app,
            clock,
            ..
        } = a;
        let (_tauri_app, w) = mock(app);
        let answer = with_ticker(runtime, clock, || call(&w, "camera_take_back"));
        assert_eq!(
            answer.unwrap(),
            json!({"completed_steps":5,"camera_held":true})
        );

        let b = held("claimed_wifi.txt");
        let mut ftp = b.ftp();
        let (reply, _data) = ftp.open_stor("DSC04807.ARW").unwrap();
        assert!(reply.starts_with("150"), "{reply}");
        settle(&b, || b.camera_runtime().uploads_in_progress() == 1);
        let runtime = b.camera_runtime();
        let psm_testkit::TestApp {
            dir: _dir_b,
            app,
            clock,
            ..
        } = b;
        let (_tauri_app_b, w) = mock(app);
        let error = with_ticker(runtime, clock, || call(&w, "camera_take_back")).unwrap_err();
        assert_eq!(error["code"], "take_back_blocked");
        assert_eq!(error["data"], json!({"reason":"a file is still arriving"}));
    }

    #[test]
    fn c8a_camera_test_connection_command() {
        let a = held("claimed_wifi.txt");
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let (_tauri_app, w) = mock(app);
        assert_eq!(
            call(&w, "camera_test_connection").unwrap(),
            json!({"holder":"Capture One","programs":["Capture One"],"sentence":CO_SENTENCE})
        );

        let b = psm_testkit::TestApp::with_camera_script(&script("ready_usb.txt")).unwrap();
        b.fake_processes.set(vec!["Photos"]);
        settle(&b, || {
            matches!(
                b.camera_status().unwrap(),
                psm_core::domain::camera::CameraStatus::Connected { .. }
            )
        });
        let psm_testkit::TestApp {
            dir: _dir_b, app, ..
        } = b;
        let (_tauri_app_b, w) = mock(app);
        assert_eq!(
            call(&w, "camera_test_connection").unwrap(),
            json!({"holder":null,"programs":["Photos"],"sentence":null})
        );
    }
}

#[cfg(test)]
mod p54_tests {
    use super::*;
    use psm_core::domain::camera::CameraStatus;
    use serde_json::{Value, json};
    use std::{
        thread,
        time::{Duration, Instant},
    };
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::{
        ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
        webview::InvokeRequest,
    };

    fn call(w: &tauri::WebviewWindow<tauri::test::MockRuntime>, cmd: &str) -> Value {
        let body = get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(json!({})),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .unwrap();
        match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        }
    }

    #[test]
    fn c4b_retry_command_sends_reconnect_and_returns_connecting() {
        let script =
            include_str!("../../../../crates/testkit/src/fixtures/sidecar_scripts/ready_wifi.txt");
        let a = psm_testkit::TestApp::with_camera_script(script).unwrap();
        let end = Instant::now() + Duration::from_secs(10);
        while !matches!(a.camera_status().unwrap(), CameraStatus::Connected { .. }) {
            assert!(Instant::now() < end);
            a.tick(0).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
        a.inject_all(&[
            (
                "session.closed",
                json!({"transport":"wifi","reason":"lost"}),
            ),
            (
                "transport.state",
                json!({"from":"CONNECTED_WIFI","to":"DISCONNECTED","reason":"session closed","transport":"none"}),
            ),
        ])
        .unwrap();
        while a.camera_status().unwrap() != CameraStatus::Disconnected {
            assert!(Instant::now() < end);
            a.tick(0).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
        let rec = a.app.paths.root.join("camera/sony/test/rec");
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![commands::camera::camera_retry])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        assert_eq!(
            call(&w, "camera_retry"),
            json!({"kind":"connecting","link":"wireless"})
        );
        let sent = loop {
            let sent = std::fs::read_to_string(rec.join("stdin.log"))
                .unwrap_or_default()
                .lines()
                .any(|line| {
                    line.split_once(' ')
                        .is_some_and(|(_, command)| command.starts_with("usb-retry"))
                });
            if sent || Instant::now() >= end {
                break sent;
            }
            thread::sleep(Duration::from_millis(10));
        };
        assert!(sent, "Retry did not reach the helper");
    }
}

/// W4·U1 (rows W4·C1–C5): Settings → Writer's commands. Boundary: the wired app (`TestApp` with
/// the research fakes) through the Tauri command layer; outcomes are the answers, the saved
/// settings file, the run and call rows and the fake model's requests.
#[cfg(test)]
mod w4_tests {
    use super::*;
    use psm_core::ports::clock::Clock;
    use psm_research::{
        domain::{RunTrigger, Stage},
        fakes::FakeModel,
        store::{self as rstore, RunRow},
        writer::defaults::INSTRUCTION_EN,
    };
    use serde_json::{Value, json};
    use std::sync::Arc;
    use tauri::test::{
        INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder, mock_context, noop_assets,
    };
    use tauri::{
        App as TauriApp, WebviewWindow,
        ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
        webview::InvokeRequest,
    };

    const NOW_MS: i64 = 1_763_570_000_000;
    const DESIGNER: &str = "custom.designer";

    struct Wired {
        _dir: tempfile::TempDir,
        tauri: TauriApp<MockRuntime>,
        w: WebviewWindow<MockRuntime>,
        clock: Arc<psm_testkit::clock::ManualClock>,
        model: Arc<FakeModel>,
        store: Arc<psm_infra::sqlite::store::SqliteStore>,
        paths: psm_infra::paths::AppPaths,
    }

    fn call(w: &WebviewWindow<MockRuntime>, cmd: &str, args: Value) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }

    /// The wired research app, the writer's models priced for the fake provider; `prepare` runs
    /// on the test app before it is handed to the command layer. Returns the app and the session.
    fn wired(prepare: impl FnOnce(&psm_testkit::TestApp, i64, i64)) -> (Wired, i64) {
        let a = psm_testkit::TestApp::with_research().unwrap();
        a.clock.advance(NOW_MS);
        let mut research = psm_testkit::research_test_settings();
        for model in ["gpt-6.1-sol", "gpt-6.1-terra"] {
            research["prices"].as_array_mut().unwrap().push(json!({
                "provider": FakeModel::ID, "model": model, "unit": "input_token",
                "usd_per_million": "2", "fallback_call_usd": "0.001"
            }));
        }
        a.set_research_settings(research).unwrap();
        let (item, session) = a.items().create_local("W4-1", "", "Chair", None).unwrap();
        let s = session.session.id;
        let photo = a.add_photo(s, "w4-1.ARW").unwrap();
        a.review()
            .classify(photo.id, psm_core::domain::photo::Classification::Main)
            .unwrap();
        prepare(&a, s, item.id);
        let store = a.app.store.clone();
        let paths = a.app.paths.clone();
        let model = a.research_fakes().model.clone();
        let psm_testkit::TestApp {
            dir, app, clock, ..
        } = a;
        let tauri = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::writer::writer_settings_get,
                commands::writer::writer_settings_save,
                commands::writer::writer_settings_export,
                commands::writer::writer_settings_import,
                commands::writer::writer_instruction_reset,
                commands::writer::writer_preview,
                commands::writer::writer_request,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri, "main", Default::default())
            .build()
            .unwrap();
        (
            Wired {
                _dir: dir,
                tauri,
                w,
                clock,
                model,
                store,
                paths,
            },
            s,
        )
    }

    impl Wired {
        fn call(&self, cmd: &str, args: Value) -> Result<Value, Value> {
            call(&self.w, cmd, args)
        }
        fn get(&self) -> Value {
            self.call("writer_settings_get", json!({})).unwrap()
        }
        fn save(&self, value: Value) -> Result<Value, Value> {
            self.call("writer_settings_save", json!({ "value": value }))
        }
        fn slot(&self) -> Option<Value> {
            psm_infra::settings_file::Settings::open(&self.paths)
                .unwrap()
                .writer
        }
        fn runs(&self) -> Vec<RunRow> {
            self.store
                .research_transaction(|tx| rstore::all_runs(tx.conn(), None))
                .unwrap()
        }
        fn calls(&self) -> Vec<rstore::CallRow> {
            self.store
                .research_transaction(|tx| rstore::calls(tx.conn(), None))
                .unwrap()
        }
        /// "Write texts", then the research runner's turns until the run ends.
        fn write(&self, s: i64) -> RunRow {
            let requested = self
                .call(
                    "writer_request",
                    json!({"sessionId": s, "origin": "write_texts"}),
                )
                .unwrap();
            assert_eq!(requested["queued"], json!(true), "{requested}");
            let state = self.tauri.state::<AppState>();
            for _ in 0..3 {
                self.clock.advance(1_100);
                let now = self.clock.now_ms();
                state.mutate(|app| app.research_tick(now)).unwrap();
            }
            let run = self
                .runs()
                .into_iter()
                .rfind(|r| r.trigger == RunTrigger::Write)
                .unwrap();
            assert_eq!(run.state.as_str(), "finished", "{:?}", run.error);
            run
        }
    }

    /// The stored English instruction text (`null` when it is the shipped one).
    fn with_english(mut value: Value, text: &str) -> Value {
        value["instructions"]["English"]["text"] = json!(text);
        value
    }

    fn reply(questions: Value) -> Value {
        json!({
            "texts": {
                "title": {"text": "Danish Beech Dining Chair", "facts": [], "reason": "Fields."},
            },
            "findings": [],
            "questions": questions,
        })
    }

    #[test]
    fn a23_export_then_import_restores_the_writer_settings_only() {
        let (app, _) = wired(|_, _, _| {});
        let mut first = with_english(app.get(), "First instruction.");
        first["title_formula"] = json!(true);
        first["shop_name"] = json!("Shop");
        app.save(first).unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("writer.json");
        let path_text = path.to_string_lossy().to_string();
        app.call("writer_settings_export", json!({ "path": path_text }))
            .unwrap();
        let file: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let mut top: Vec<&str> = file
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        top.sort_unstable();
        assert_eq!(top, ["kind", "version", "writer"]);
        assert!(file["writer"].get("research").is_none());
        // Change the settings, then import the file back.
        let mut second = with_english(app.get(), "Second instruction.");
        second["title_formula"] = json!(false);
        app.save(second).unwrap();
        let imported = app
            .call("writer_settings_import", json!({ "path": path_text }))
            .unwrap();
        assert_eq!(imported["title_formula"], json!(true));
        assert_eq!(imported["shop_name"], json!("Shop"));
        let english = &imported["instructions"]["English"];
        assert_eq!(english["text"], json!("First instruction."));
        assert_eq!(english["versions"][0]["text"], json!("Second instruction."));
        // A file that is not writer settings is refused and saves nothing.
        let bad = tmp.path().join("bad.json");
        std::fs::write(
            &bad,
            r#"{"kind":"session-manager-writer-settings","writer":{"grade_word":"yes"}}"#,
        )
        .unwrap();
        let before = app.slot();
        let refused = app
            .call(
                "writer_settings_import",
                json!({ "path": bad.to_string_lossy() }),
            )
            .unwrap_err();
        assert!(refused.to_string().contains("grade_word"), "{refused}");
        assert_eq!(app.slot(), before);
    }

    #[test]
    fn w4_c1a_without_a_writer_slot_every_field_shows_its_default() {
        let (app, _) = wired(|_, _, _| {});
        assert_eq!(app.slot(), None);
        let got = app.get();
        let english = &got["instructions"]["English"];
        assert_eq!(english["effective_instruction"], json!(INSTRUCTION_EN));
        assert_eq!(english["is_default"], json!(true));
        assert_eq!(english["text"], Value::Null);
        assert_eq!(english["versions"], json!([]));
        assert_eq!(got["instructions"]["Swedish"]["is_default"], json!(true));
        let zero = json!({"answered": 0, "dismissed": 0});
        assert_eq!(
            got["counts"],
            json!({
                "field_vs_photos": zero, "field_vs_field": zero, "field_vs_listings": zero,
                "missing_in_text": zero, "text_vs_facts": zero,
            })
        );
        let names = |language: &str| -> Vec<Value> {
            got["examples"][language]
                .as_array()
                .unwrap()
                .iter()
                .map(|e| e["name"].clone())
                .collect()
        };
        assert_eq!(names("English"), [json!("A"), json!("B"), json!("C")]);
        assert_eq!(names("Swedish"), [json!("D")]);
        let mut rest = got.as_object().unwrap().clone();
        for key in ["instructions", "examples", "counts"] {
            rest.remove(key);
        }
        assert_eq!(
            Value::Object(rest),
            json!({
                "auto": true,
                "language": "English",
                "model": "gpt-6.1-sol",
                "effort": "medium",
                "banned_extra": [],
                "lengths": {
                    "title_max": 70, "description_min_words": 40, "description_max_words": 100,
                    "condition_max_words": 35, "seo_title_max": 60, "seo_description_min": 120,
                    "seo_description_max": 155,
                },
                "visible_keys": null,
                "safeguard": {
                    "field_vs_photos": "ask", "field_vs_field": "note",
                    "field_vs_listings": "note", "missing_in_text": "ask", "text_vs_facts": "ask",
                },
                "max_questions": 3,
                "shop_name": "",
                "condition_report_key": "custom.condition_report_description",
                "damage_terms": {},
                "grade_word": false,
                "stability_sentence": false,
                "title_formula": false,
                "dimensions_line": false,
            })
        );
    }

    #[test]
    fn w4_c1b_the_counts_are_the_answered_and_dismissed_findings_per_type() {
        let (app, _) = wired(|a, s, item| {
            a.app
                .store
                .research_transaction(|tx| {
                    for (n, state) in ["answered", "answered", "dismissed", "open"]
                        .into_iter()
                        .enumerate()
                    {
                        tx.conn()
                            .execute(
                                "INSERT INTO writer_findings (item_id, session_id, run_id, kind, \
                                 type, keys, text_field, identity, text, mode, state, created_at) \
                                 VALUES (?1, ?2, 0, 'finding', 'field_vs_field', '[]', '', ?3, \
                                 'The era does not fit', 'note', ?4, ?5)",
                                rusqlite::params![item, s, format!("id-{n}"), state, NOW_MS],
                            )
                            .unwrap();
                    }
                    Ok(())
                })
                .unwrap();
        });
        let counts = &app.get()["counts"];
        assert_eq!(
            counts["field_vs_field"],
            json!({"answered": 2, "dismissed": 1})
        );
        let zero = json!({"answered": 0, "dismissed": 0});
        for other in [
            "field_vs_photos",
            "field_vs_listings",
            "missing_in_text",
            "text_vs_facts",
        ] {
            assert_eq!(counts[other], zero, "{other}");
        }
    }

    #[test]
    fn w4_c2a_a_changed_instruction_keeps_the_text_it_replaced_with_its_time() {
        let (app, _) = wired(|_, _, _| {});
        app.save(with_english(app.get(), "Guide 1")).unwrap();
        app.clock.advance(60_000);
        let at = app.clock.now_ms();
        let saved = app.save(with_english(app.get(), "Guide 2")).unwrap();
        let english = &saved["instructions"]["English"];
        assert_eq!(english["text"], json!("Guide 2"));
        assert_eq!(english["is_default"], json!(false));
        assert_eq!(
            english["versions"][0],
            json!({"saved_at": at, "text": "Guide 1"})
        );
        let slot = app.slot().unwrap();
        assert_eq!(slot["instructions"]["English"]["text"], json!("Guide 2"));
        assert_eq!(
            slot["instructions"]["English"]["versions"][0],
            json!({"saved_at": at, "text": "Guide 1"})
        );
    }

    #[test]
    fn w4_c2c_an_unchanged_text_keeps_no_version_and_an_invalid_value_saves_nothing() {
        let (app, _) = wired(|_, _, _| {});
        app.save(with_english(app.get(), "Guide 1")).unwrap();
        let versions = app.get()["instructions"]["English"]["versions"].clone();
        let again = app.save(with_english(app.get(), "Guide 1")).unwrap();
        assert_eq!(again["instructions"]["English"]["versions"], versions);
        let before = std::fs::read(app.paths.settings()).unwrap();
        let mut german = app.get();
        german["language"] = json!("German");
        let error = app.save(german).unwrap_err();
        assert!(
            error["message"].as_str().unwrap().contains("language"),
            "{error}"
        );
        assert_eq!(std::fs::read(app.paths.settings()).unwrap(), before);
    }

    #[test]
    fn w4_c3a_reset_returns_to_the_shipped_instruction_keeping_the_saved_text() {
        let (app, _) = wired(|_, _, _| {});
        app.save(with_english(app.get(), "Guide 2")).unwrap();
        let reset = app
            .call("writer_instruction_reset", json!({"language": "English"}))
            .unwrap();
        let english = &reset["instructions"]["English"];
        assert_eq!(english["text"], Value::Null);
        assert_eq!(english["effective_instruction"], json!(INSTRUCTION_EN));
        assert_eq!(english["is_default"], json!(true));
        assert_eq!(english["versions"][0]["text"], json!("Guide 2"));
        assert_eq!(
            app.slot().unwrap()["instructions"]["English"]["text"],
            Value::Null
        );
    }

    #[test]
    fn w4_c4a_the_preview_is_the_request_for_the_example_item_under_the_saved_settings() {
        let (app, _) = wired(|_, _, _| {});
        let mut value = with_english(app.get(), "Guide 2");
        value["shop_name"] = json!("Beyo Vintage");
        app.save(value).unwrap();
        let preview = app
            .call("writer_preview", json!({"language": "English"}))
            .unwrap();
        let instructions = preview["instructions"].as_str().unwrap();
        assert!(instructions.starts_with("Guide 2"), "{instructions}");
        assert!(instructions.contains("Example A"), "{instructions}");
        let input: Value = serde_json::from_str(preview["input"].as_str().unwrap()).unwrap();
        assert!(input["texts"].as_array().is_some_and(|t| !t.is_empty()));
        let values: Vec<&str> = input["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["value"].as_str().unwrap())
            .collect();
        assert_eq!(
            values,
            [
                "dining chair",
                "Børge Mogensen",
                "FDB Møbler",
                "J39",
                "1960s",
                "Denmark",
                "solid beech, papercord",
                "W 49 D 46 H 78, seat 45",
                "light scratches",
                "stable",
                "sanded, oiled, seat rewoven",
                "photo 4 shows scratch",
            ]
        );
        assert_eq!(input["shop_name"], json!("Beyo Vintage"));
        let schema: Value = serde_json::from_str(preview["schema"].as_str().unwrap()).unwrap();
        for key in ["texts", "findings", "questions"] {
            assert!(schema["properties"].get(key).is_some(), "{key}: {schema}");
        }
    }

    #[test]
    fn w4_c4b_the_preview_makes_no_run_no_call_and_no_model_request() {
        let (app, _) = wired(|_, _, _| {});
        app.save(with_english(app.get(), "Guide 2")).unwrap();
        app.model.clear_requests();
        app.call("writer_preview", json!({"language": "English"}))
            .unwrap();
        assert!(app.runs().is_empty());
        assert!(app.calls().is_empty());
        assert!(app.model.requests().is_empty());
    }

    #[test]
    fn w4_c5a_the_run_sees_only_the_saved_fields_and_asks_at_most_the_saved_number() {
        let (app, s) = wired(|_, _, _| {});
        let mut value = app.get();
        value["visible_keys"] = json!([DESIGNER]);
        value["max_questions"] = json!(1);
        app.save(value).unwrap();
        app.model.reply(
            Stage::Write,
            "texts",
            reply(json!([
                {"keys": [DESIGNER], "text": "Who designed it?"},
                {"keys": [DESIGNER], "text": "Is there a maker's mark?"},
                {"keys": [DESIGNER], "text": "Which year was it made?"},
            ])),
        );
        let run = app.write(s);
        let input: Value = serde_json::from_str(run.input_snapshot.as_deref().unwrap()).unwrap();
        let keys: Vec<&str> = input["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["key"].as_str().unwrap())
            .collect();
        assert_eq!(keys, [DESIGNER]);
        let asks: i64 = app
            .store
            .research_transaction(|tx| {
                Ok(tx
                    .conn()
                    .query_row(
                        "SELECT COUNT(*) FROM writer_findings WHERE run_id=?1 AND mode='ask'",
                        [run.id],
                        |r| r.get(0),
                    )
                    .unwrap())
            })
            .unwrap();
        assert_eq!(asks, 1);
    }

    #[test]
    fn w4_c5b_the_run_calls_the_saved_model_and_effort() {
        let (app, s) = wired(|_, _, _| {});
        let mut value = app.get();
        value["model"] = json!("gpt-6.1-terra");
        value["effort"] = json!("low");
        app.save(value).unwrap();
        app.model.reply(Stage::Write, "texts", reply(json!([])));
        let run = app.write(s);
        let calls: Vec<_> = app
            .calls()
            .into_iter()
            .filter(|c| c.run_id == run.id)
            .collect();
        assert!(!calls.is_empty());
        for c in calls {
            assert_eq!(
                (c.model.as_str(), c.effort.as_str()),
                ("gpt-6.1-terra", "low")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn export_bindings() {
        super::specta_builder()
            .export(
                specta_typescript::Typescript::default(),
                "../src/services/bindings.ts",
            )
            .unwrap();
    }
}

#[cfg(test)]
mod t1_tests {
    use super::*;
    use serde_json::{Value, json};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::{
        ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
        webview::InvokeRequest,
    };

    fn call(w: &tauri::WebviewWindow<tauri::test::MockRuntime>, cmd: &str, args: Value) -> Value {
        let body = get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .unwrap();
        match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        }
    }

    #[test]
    fn t1_c3b_commands_reach_wired_app_and_bindings() {
        let a = psm_testkit::TestApp::new().unwrap();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::templates::template_create,
                commands::templates::template_container_add,
                commands::templates::template_get,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        let created = call(&w, "template_create", json!({"itemType":"Chair"}));
        let id = created["id"].as_i64().unwrap();
        call(
            &w,
            "template_container_add",
            json!({"templateId":id,"source":null}),
        );
        let got = call(&w, "template_get", json!({"itemType":"chair"}));
        assert_eq!(got["containers"][0]["final_no"], 1);
        assert_eq!(got["containers"][0]["shoot_no"], 1);
        let bindings = include_str!("../../src/services/bindings.ts");
        for name in [
            "templatesList",
            "templateGet",
            "templateCreate",
            "templateDelete",
            "templateContainerAdd",
            "templateContainerRemove",
            "templateContainerExampleSet",
            "templateContainersReorder",
            "templateShootingOrderSet",
            "templateContainerPresetsSet",
        ] {
            assert!(bindings.contains(name), "missing {name}");
        }
    }

    #[test]
    fn t2_c6b_session_view_command_and_bindings() {
        let a = psm_testkit::TestApp::new().unwrap();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::templates::template_session_view
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        let view = call(&w, "template_session_view", json!({"sessionId":1}));
        assert_eq!(view["templated"], false);
        assert_eq!(view["containers"], json!([]));
        assert!(include_str!("../../src/services/bindings.ts").contains("template_session_view"));
    }
}

#[cfg(test)]
mod t3_tests {
    use super::*;
    use serde_json::{Value, json};
    use tauri::{
        ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
        test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets},
        webview::InvokeRequest,
    };

    fn call(
        w: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        args: Value,
    ) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }

    #[test]
    fn t3_c3g_template_commands_answer_no_template() {
        let a = psm_testkit::TestApp::new().unwrap();
        let session = a
            .items()
            .create_local("C3G", "C3G", "Table", None)
            .unwrap()
            .1
            .session
            .id;
        let photo = a.add_photo(session, "DSC00001.ARW").unwrap();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::templates::template_back,
                commands::templates::template_drop,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        let back = call(&w, "template_back", json!({"sessionId":session})).unwrap_err();
        assert_eq!(back["code"], "no_template");
        let dropped = call(
            &w,
            "template_drop",
            json!({"photoId":photo.id,"containerNo":1}),
        )
        .unwrap_err();
        assert_eq!(dropped["code"], "no_template");
        let bindings = include_str!("../../src/services/bindings.ts");
        assert!(bindings.contains("template_drop") && bindings.contains("template_back"));
    }

    #[test]
    fn t4_c2g_shift_back_command_answers_no_template() {
        let a = psm_testkit::TestApp::new().unwrap();
        let session = a
            .items()
            .create_local("C2G", "C2G", "Table", None)
            .unwrap()
            .1
            .session
            .id;
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::templates::template_shift_back
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        let shifted = call(&w, "template_shift_back", json!({"sessionId":session})).unwrap_err();
        assert_eq!(shifted["code"], "no_template");
        // T4·C2(e) refusal as the UI sees it.
        let nothing = commands::items::CommandError::from(psm_core::Error::NothingToShift);
        assert_eq!(nothing.code, "nothing_to_shift");
        let bindings = include_str!("../../src/services/bindings.ts");
        assert!(bindings.contains("template_shift_back"));
    }
}

#[cfg(test)]
mod p25_tests {
    use super::*;
    use psm_testkit::{clock::ManualClock, fake_capture_one::FakeCaptureOneScripting};

    #[test]
    fn c6a_startup_marker_is_written_before_app_open() {
        let tmp = tempfile::tempdir().unwrap();
        let mut paths = AppPaths::for_dir(tmp.path());
        paths.home = tmp.path().to_path_buf();
        let (guard, app) = open_logged(paths, |p| {
            let doc = tmp
                .path()
                .join("Capture One Session")
                .to_string_lossy()
                .into_owned();
            let root = tmp
                .path()
                .join("Capture One Session/Capture")
                .to_string_lossy()
                .into_owned();
            psm_integrations::App::open_with(
                p,
                Arc::new(FakeCaptureOneScripting::new(doc, root)),
                Arc::new(ManualClock::new(0)),
                false,
            )
        })
        .unwrap();
        drop(app);
        drop(guard);
        let entries: Vec<_> = std::fs::read_dir(tmp.path().join("logs"))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].file_name().to_string_lossy().starts_with("psm."));
        let log = std::fs::read_to_string(entries[0].path()).unwrap();
        assert!(log.contains("app open"));
        assert!(log.contains(&tmp.path().to_string_lossy().to_string()));
    }
}

#[cfg(test)]
mod p17_tests {
    use super::*;
    use std::{
        fs,
        sync::{Arc, mpsc},
        time::{Duration, Instant},
    };
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::{
        ipc::{CallbackFn, InvokeBody},
        webview::InvokeRequest,
    };

    #[test]
    fn commands_are_async() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/commands");
        let mut command_count = 0;
        let mut async_count = 0;
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let code = fs::read_to_string(entry.path()).unwrap();
            command_count += code.matches("#[tauri::command]").count();
            async_count += code.matches("pub async fn ").count();
            assert!(
                !code.contains(".0.lock(")
                    && !code.contains(".0.read(")
                    && !code.contains(".0.write(")
            );
        }
        assert_eq!(command_count, async_count);
        assert!(command_count > 30);
    }

    fn request(cmd: &str, args: serde_json::Value) -> InvokeRequest {
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(args),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        }
    }

    #[test]
    fn commands_do_not_wait_on_each_other() {
        let test = psm_testkit::TestApp::with_fake_document().unwrap();
        test.fake.state.lock().unwrap().delay_document_info_ms = 1_500;
        let psm_testkit::TestApp { dir: _dir, app, .. } = test;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::health::health_get,
                commands::sessions::sessions_list
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        let webview = Arc::new(webview);
        let (tx, rx) = mpsc::channel();
        std::thread::scope(|scope| {
            let slow = webview.clone();
            let slow_tx = tx.clone();
            let slow_thread = scope.spawn(move || {
                let answer =
                    get_ipc_response(slow.as_ref(), request("health_get", serde_json::json!({})));
                slow_tx
                    .send(("health", answer.is_ok(), Instant::now()))
                    .unwrap();
            });
            std::thread::sleep(Duration::from_millis(100));
            let sent = Instant::now();
            let fast = get_ipc_response(
                webview.as_ref(),
                request(
                    "sessions_list",
                    serde_json::json!({"filter":"open","query":null}),
                ),
            );
            assert!(fast.is_ok());
            assert!(sent.elapsed() < Duration::from_millis(500));
            assert!(
                rx.try_recv().is_err(),
                "health_get should still be waiting for Capture One"
            );
            slow_thread.join().unwrap();
        });
        assert!(rx.recv().unwrap().1);
    }
}

#[cfg(test)]
mod p21_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn c2a_concurrent_settings_saves_preserve_every_change() {
        let test = psm_testkit::TestApp::new().unwrap();
        let path = test.app.paths.settings();
        let state = Arc::new(AppState::new(test.app));
        let patches = [
            json!({"preview_recipe":"R1"}),
            json!({"master_recipe":"M1"}),
            json!({"style_name":"S1"}),
            json!({"sku_prefix":"PX"}),
        ];
        let threads: Vec<_> = patches
            .into_iter()
            .map(|patch| {
                let state = state.clone();
                std::thread::spawn(move || state.settings_set(&patch))
            })
            .collect();
        for thread in threads {
            thread.join().unwrap().unwrap();
        }
        let shown = state.handles().unwrap().settings_get().unwrap();
        let stored: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        for (key, value) in [
            ("preview_recipe", "R1"),
            ("master_recipe", "M1"),
            ("style_name", "S1"),
            ("sku_prefix", "PX"),
        ] {
            assert_eq!(shown[key], value);
            assert_eq!(stored[key], value);
        }
    }
}

#[cfg(test)]
mod p28_tests {
    use super::*;
    use psm_core::ports::store::Store;
    use serde_json::{Value, json};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::{
        ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
        webview::InvokeRequest,
    };
    fn request(cmd: &str, args: Value) -> InvokeRequest {
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(args),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        }
    }
    fn result(
        webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        args: Value,
    ) -> Result<Value, Value> {
        get_ipc_response(webview, request(cmd, args)).map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw response"),
        })
    }
    #[test]
    fn c7b_commands_validate_and_serialize_compositions() {
        let test = psm_testkit::TestApp::with_fake_document().unwrap();
        let sid = test
            .items()
            .create_local("S1", "S1", "object", None)
            .unwrap()
            .1
            .session
            .id;
        let p = test.add_photo(sid, "a.ARW").unwrap();
        test.app.store.upsert_raw_state(&p.raw_path, None).unwrap();
        test.fake.add_image(&p.raw_path, &["7"], false);
        test.app.previews.request_initial(&p.raw_path).unwrap();
        test.complete_render_with(
            "job-1",
            &psm_testkit::fixtures::synthetic_jpeg::SyntheticJpeg::rgb(2560, 1707, [200, 100, 50]),
        )
        .unwrap();
        let frameless = test.add_photo(sid, "b.ARW").unwrap();
        let psm_testkit::TestApp { dir: _dir, app, .. } = test;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::compositions::composition_set,
                commands::compositions::composition_reset,
                commands::sessions::session_get
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        let values = json!({"kind":"normal","shape":"1:1","orientation":"turned","quarter_turns":1,"flip":"horizontal","straighten":2.5,"rect":{"x":0.1,"y":0.3,"w":0.8,"h":0.5334375}});
        let good = result(
            &webview,
            "composition_set",
            json!({"photoId":p.id,"values":values}),
        )
        .unwrap();
        assert_eq!(
            (
                good["revision"].as_i64(),
                good["origin"].as_str(),
                good["state"].as_str()
            ),
            (Some(1), Some("user"), Some("ok"))
        );
        assert_eq!(good["review_reason"], Value::Null);
        assert_eq!(good["rect"]["w"], json!(0.8));
        assert_eq!(good["frame"], json!({"w":2560,"h":1707}));
        assert!(good.get("geometry").is_none());
        let bad=result(&webview,"composition_set",json!({"photoId":p.id,"values":{"kind":"normal","shape":"1:1","orientation":"named","quarter_turns":0,"flip":"none","straighten":0,"rect":{"x":0.2,"y":0,"w":0.666,"h":1}}})).unwrap_err();
        assert_eq!(
            (bad["code"].as_str(), bad["data"]["reason"].as_str()),
            (Some("composition_invalid"), Some("off_shape"))
        );
        let unknown = result(
            &webview,
            "composition_set",
            json!({"photoId":frameless.id,"values":values}),
        )
        .unwrap_err();
        assert_eq!(unknown["code"], "frame_unknown");
        let reset = result(&webview, "composition_reset", json!({"photoId":p.id})).unwrap();
        assert_eq!(reset["origin"], "default");
        let view = result(&webview, "session_get", json!({"sessionId":sid})).unwrap();
        let photo = view["photos"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == p.id)
            .unwrap();
        assert_eq!(photo["composition"], reset);
        assert!(
            photo["thumbnail_path"]
                .as_str()
                .unwrap()
                .ends_with("thumb-1.jpg")
        );
    }
}

#[cfg(test)]
mod p29_tests {
    use super::*;
    use psm_core::{
        domain::{composition::EditorGeometry, photo::Classification},
        ports::store::Store,
    };
    use psm_testkit::fixtures::synthetic_jpeg::SyntheticJpeg;
    use serde_json::{Value, json};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::{
        ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
        webview::InvokeRequest,
    };

    fn last_job(test: &psm_testkit::TestApp) -> String {
        test.fake
            .state
            .lock()
            .unwrap()
            .issued
            .last()
            .unwrap()
            .0
            .clone()
    }

    /// C8(a) at the command boundary: `export_run` refuses with `needs_review`, naming the photo.
    #[test]
    fn c8a_export_run_rejects_with_needs_review() {
        let test = psm_testkit::TestApp::with_fake_document().unwrap();
        let sid = test
            .items()
            .create_local("K", "Oak", "Sideboard", None)
            .unwrap()
            .1
            .session
            .id;
        let mut photos = Vec::new();
        for name in ["a.ARW", "b.ARW"] {
            // The integrity check (MC-3) runs before the review gate: a RAW with its real hash.
            let raw = test.dir.path().join(name);
            std::fs::write(&raw, name).unwrap();
            let p = test
                .app
                .store
                .photo_insert_or_get(psm_core::domain::photo::NewPhoto {
                    session_id: sid,
                    raw_path: raw.to_string_lossy().into_owned(),
                    raw_hash: Some(psm_core::mechanisms::hash::sha256_file(&raw).unwrap()),
                    captured_at: Some(0),
                    source: psm_core::domain::photo::PhotoSource::Capture,
                    created_at: 0,
                })
                .unwrap();
            test.app.store.upsert_raw_state(&p.raw_path, None).unwrap();
            test.fake
                .add_image(&p.raw_path, &[&format!("v{}", p.id)], false);
            test.app.previews.request_initial(&p.raw_path).unwrap();
            test.complete_render_with(
                &last_job(&test),
                &SyntheticJpeg::rgb(2560, 1707, [200, 100, 50]),
            )
            .unwrap();
            test.review().classify(p.id, Classification::Main).unwrap();
            photos.push(p);
        }
        let b = &photos[1];
        test.fake.set_variant_geometry(
            &format!("v{}", b.id),
            EditorGeometry {
                rotation: 3.0,
                ..Default::default()
            },
        );
        test.clock.advance(31_000);
        test.app
            .previews
            .on_variant_adjusted(&format!("v{}", b.id), &b.raw_path)
            .unwrap();
        test.tick(20_000).unwrap();
        test.complete_render_with(
            &last_job(&test),
            &SyntheticJpeg::rgb(2560, 1707, [200, 100, 50]),
        )
        .unwrap();
        let psm_testkit::TestApp { dir: _dir, app, .. } = test;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![commands::export::export_run])
            .build(mock_context(noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        let error = get_ipc_response(
            &webview,
            InvokeRequest {
                cmd: "export_run".into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(json!({"sessionId": sid})),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str::<Value>(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw response"),
        })
        .unwrap_err();
        assert_eq!(error["code"], "needs_review");
        assert_eq!(error["data"]["photos"], "Main 02");
        assert_eq!(error["message"], "Re-frame first: Main 02");
    }
}

#[cfg(test)]
mod w1_writer_command_tests {
    use super::*;
    use serde_json::{Value, json};
    use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
    use tauri::webview::InvokeRequest;

    fn call(
        w: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        args: Value,
    ) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| match body {
            InvokeResponseBody::Json(s) => serde_json::from_str(&s).unwrap(),
            InvokeResponseBody::Raw(_) => panic!("raw"),
        })
    }

    /// W1·C13(c) (HW7, MA7): an app whose research engine could not be built answers
    /// `research_unavailable` for both writer commands; the bindings carry both commands.
    #[test]
    fn w1_c13c_writer_commands_without_the_engine_answer_research_unavailable() {
        let a = psm_testkit::TestApp::with_research().unwrap();
        let s = a
            .items()
            .create_local("CMD-W1", "Chair", "Chair", None)
            .unwrap()
            .1
            .session
            .id;
        let psm_testkit::TestApp {
            dir: _dir, mut app, ..
        } = a;
        // What `App::build` leaves when `build_research` fails.
        app.research = None;
        app.writer = None;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::writer::writer_note,
                commands::writer::writer_look_again,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        for cmd in ["writer_note", "writer_look_again"] {
            let error = call(&w, cmd, json!({"sessionId": s})).unwrap_err();
            assert_eq!(error["code"], "research_unavailable", "{cmd}: {error}");
        }
        let bindings = include_str!("../../src/services/bindings.ts");
        for name in [
            "writerNote",
            "writerLookAgain",
            "writer_note",
            "writer_look_again",
        ] {
            assert!(bindings.contains(name), "binding missing {name}");
        }
    }

    /// W3·C6(a) (HW7, MA7): without the research engine the four W3 writer commands answer
    /// `research_unavailable`; the bindings carry all four.
    #[test]
    fn w3_c6a_writer_commands_without_the_engine_answer_research_unavailable() {
        let a = psm_testkit::TestApp::with_research().unwrap();
        let s = a
            .items()
            .create_local("CMD-W3", "Chair", "Chair", None)
            .unwrap()
            .1
            .session
            .id;
        let psm_testkit::TestApp {
            dir: _dir, mut app, ..
        } = a;
        app.research = None;
        app.writer = None;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![
                commands::writer::writer_status,
                commands::writer::writer_request,
                commands::writer::writer_finding_answer,
                commands::writer::writer_finding_dismiss,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        for (cmd, args) in [
            ("writer_status", json!({"sessionId": s})),
            (
                "writer_request",
                json!({"sessionId": s, "origin": "write_texts"}),
            ),
            (
                "writer_finding_answer",
                json!({"findingId": 1, "answer": "Teak"}),
            ),
            ("writer_finding_dismiss", json!({"findingId": 1})),
        ] {
            let error = call(&w, cmd, args).unwrap_err();
            assert_eq!(error["code"], "research_unavailable", "{cmd}: {error}");
        }
        let bindings = include_str!("../../src/services/bindings.ts");
        for name in [
            "writerStatus",
            "writerRequest",
            "writerFindingAnswer",
            "writerFindingDismiss",
            "writer_status",
            "writer_request",
            "writer_finding_answer",
            "writer_finding_dismiss",
        ] {
            assert!(bindings.contains(name), "binding missing {name}");
        }
    }
}
