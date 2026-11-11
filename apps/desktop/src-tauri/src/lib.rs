mod commands;
mod state;

use psm_core::{events::AppEvent, ports::events::EventSink};
use psm_infra::paths::AppPaths;
use state::{AppState, EditMenu};
use std::sync::Arc;
use tauri::{
    Emitter, Manager,
    menu::{Menu, MenuItem, MenuItemKind, PredefinedMenuItem, Submenu},
};

struct TauriEvents<R: tauri::Runtime>(tauri::AppHandle<R>);
#[derive(serde::Serialize, Clone)]
struct PhotoAddedPayload {
    photo_id: i64,
    session_id: i64,
}

#[cfg(test)]
mod p50_tests {
    use super::*;
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
        assert_eq!(pw.len(), 32);
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
            &TauriEvents(tauri_app.handle().clone()),
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
            .set_target(Arc::new(TauriEvents(tauri_app.handle().clone())));
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
impl<R: tauri::Runtime> EventSink for TauriEvents<R> {
    fn emit(&self, event: AppEvent) {
        match event {
            AppEvent::CameraStatusChanged { status } => {
                let link = self
                    .0
                    .try_state::<AppState>()
                    .and_then(|state| state.handles().ok())
                    .and_then(|handles| handles.camera_runtime.direct_adapter().current_link());
                let _ = self.0.emit(
                    "camera_status_changed",
                    commands::camera::CameraStatusResponse::from(status).with_link(link),
                );
            }
            AppEvent::FingerprintPending {
                fingerprint,
                replace,
            } => {
                let _ = self.0.emit(
                    "camera_fingerprint_pending",
                    serde_json::json!({"fingerprint": fingerprint, "replace": replace}),
                );
            }
            AppEvent::HistoryChanged {
                session_id,
                undo,
                redo,
            } => {
                let _ = self.0.emit(
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
                let _ = self.0.emit(
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
                let _ = self.0.emit(
                    "photo_changed",
                    PhotoAddedPayload {
                        photo_id,
                        session_id,
                    },
                );
            }
            AppEvent::SessionChanged { session_id } => {
                let _ = self
                    .0
                    .emit("session_changed", SessionChangedPayload { session_id });
            }
            AppEvent::ExportProgress {
                session_id,
                done,
                total,
                state,
            } => {
                let _ = self.0.emit(
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
                let _ = self.0.emit(
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
                let _ = self.0.emit("health_changed", ());
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
        commands::sessions::session_activate,
        commands::sessions::session_deactivate,
        commands::sessions::session_active,
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
        commands::photos::photos_classify,
        commands::photos::photos_reorder,
        commands::photos::photos_delete,
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
        commands::attributes::presets_get,
        commands::attributes::presets_set,
        commands::attributes::form_get,
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
        commands::items::item_archive_get,
        commands::items::item_keep_files_set,
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
        .manage(AppState::new(app))
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
            Ok(menu)
        })
        .on_menu_event(|app, event| {
            let action = match event.id().as_ref() {
                "edit.undo" => "undo",
                "edit.redo" => "redo",
                _ => return,
            };
            let _ = app.emit("edit_menu", serde_json::json!({ "action": action }));
        })
        .setup(|handle| {
            let state = handle.state::<AppState>();
            let previews = {
                let app = state.handles().expect("app state");
                app.events
                    .set_target(Arc::new(TauriEvents(handle.handle().clone())));
                app.paths.root.join("previews")
            };
            // The workspace loads preview files through the asset protocol; only the previews folder is readable.
            handle
                .asset_protocol_scope()
                .allow_directory(previews, true)?;
            if let Some(window) = handle.get_webview_window("main") {
                let app_handle = handle.handle().clone();
                window.on_window_event(move |event| {
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
            .set_target(Arc::new(TauriEvents(tauri_app.handle().clone())));
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
