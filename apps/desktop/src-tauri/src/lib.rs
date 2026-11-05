mod commands;
mod state;

use psm_core::{events::AppEvent, ports::events::EventSink};
use psm_infra::paths::AppPaths;
use state::AppState;
use std::sync::Arc;
use tauri::{Emitter, Manager};

struct TauriEvents<R: tauri::Runtime>(tauri::AppHandle<R>);
#[derive(serde::Serialize, Clone)]
struct PhotoAddedPayload {
    photo_id: i64,
    session_id: i64,
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
    fn c7b_photos_undo_compatibility() {
        let a = psm_testkit::TestApp::with_fake_document().unwrap();
        let s = a
            .items()
            .create_local("s", "s", "object", None)
            .unwrap()
            .1
            .session
            .id;
        let id = a.add_photo(s, "a").unwrap().id;
        a.review().classify(id, Classification::Main).unwrap();
        a.review().delete(id).unwrap();
        let psm_testkit::TestApp { dir: _dir, app, .. } = a;
        let tauri_app = mock_builder()
            .manage(AppState::new(app))
            .invoke_handler(tauri::generate_handler![commands::photos::photos_undo])
            .build(mock_context(noop_assets()))
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&tauri_app, "main", Default::default())
            .build()
            .unwrap();
        let first = result(&w, "photos_undo", s).unwrap();
        assert_eq!(first["active_photo_id"], id);
        assert_eq!(first["sessions"][0]["id"], s);
        assert_eq!(first["sessions"][0]["main_count"], 1);
        let second = result(&w, "photos_undo", s).unwrap();
        assert_eq!(second["active_photo_id"], id);
        assert_eq!(second["sessions"][0]["unreviewed_count"], 1);
        assert_eq!(
            result(&w, "photos_undo", s).unwrap_err()["code"],
            "nothing_to_undo"
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
            _ => {}
        }
    }
}
use tauri_specta::{Builder, collect_commands};

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::health::health_get,
        commands::health::app_paths_get,
        commands::items::item_create,
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
        commands::camera::camera_capture,
        commands::photos::photos_classify,
        commands::photos::photos_reorder,
        commands::photos::photos_delete,
        commands::photos::photos_undo,
        commands::photos::photos_move,
        commands::photos::photos_copy,
        commands::photos::photos_import,
        commands::photos::photos_assign_unassigned,
        commands::history::history_get,
        commands::history::history_undo,
        commands::history::history_redo,
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
        commands::publish::notifications_list,
        commands::settings::settings_get,
        commands::settings::settings_set,
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
        .run(tauri::generate_context!())
        .expect("run Session Manager");
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
